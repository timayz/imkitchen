//! Fetches the page a recipe is imported from, and the picture the page
//! gives it.
//!
//! The address comes from a user, and the picture's from the page, so a
//! fetch is held to what a stranger may make the api do ([`Web`]): only
//! `http` and `https`, only towards public addresses (checked on the address
//! the connection is made to, each redirect included, so a name cannot be
//! pointed elsewhere between the check and the fetch), for [`TIMEOUT`] at
//! most and [`PAGE_MAX`] at most, [`MAX_IMAGE_BYTES`] for a picture.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use bytes::Bytes;
use encoding_rs::{Encoding, UTF_8};
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::redirect::Policy;
use reqwest::{Client, Response, Url};

use super::PAGE_MAX;
use crate::images::MAX_IMAGE_BYTES;

/// How long a page gets to answer as a whole, its redirects included.
const TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_REDIRECTS: usize = 5;
/// Who the api says it is to the sites it reads: a browser, as the person
/// who pasted the address was when they saw the page. Many sites answer
/// nothing else.
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
                          (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// Why a page was not fetched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageError {
    /// Nothing was read: the address is not one the api goes to, nothing
    /// answered there in time, or what did was refused.
    Unreachable(String),
    /// What is at the address is not an HTML page.
    NotHtml,
}

/// What fetches the pages.
#[async_trait::async_trait]
pub trait Pages: Send + Sync {
    /// The HTML at `url`.
    async fn fetch(&self, url: &Url) -> Result<String, PageError>;

    /// What is at `url`, where a page says the picture of its recipe is:
    /// the bytes as they are, for whoever keeps pictures to say whether
    /// they are one.
    async fn image(&self, url: &Url) -> Result<Bytes, PageError>;
}

/// The web, as far as a stranger's address may take the api.
pub struct Web {
    client: Client,
    /// Whether only public addresses are fetched: always, outside of the
    /// tests of this module.
    guarded: bool,
}

impl Web {
    pub fn new() -> anyhow::Result<Self> {
        Self::build(true)
    }

    fn build(guarded: bool) -> anyhow::Result<Self> {
        let mut client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            // A proxy of the environment would make the connections, to
            // addresses this node never checked.
            .no_proxy()
            .redirect(Policy::custom(move |attempt| {
                if attempt.previous().len() > MAX_REDIRECTS {
                    attempt.error("too many redirects")
                } else if guarded && !allowed(attempt.url()) {
                    attempt.error("redirected to an address the api does not go to")
                } else {
                    attempt.follow()
                }
            }));
        if guarded {
            client = client.dns_resolver(Arc::new(PublicOnly));
        }
        Ok(Self {
            client: client.build().context("cannot build the HTTP client")?,
            guarded,
        })
    }
}

impl Web {
    /// What answers at `url` when asked for `accept`, once it answered
    /// that it has it.
    async fn get(&self, url: &Url, accept: &str) -> Result<Response, PageError> {
        if self.guarded && !allowed(url) {
            return Err(unreachable("not an address the api goes to"));
        }
        let response = self
            .client
            .get(url.clone())
            .header(ACCEPT, accept)
            .send()
            .await
            .map_err(failed)?;
        if !response.status().is_success() {
            return Err(unreachable(format!("answered {}", response.status())));
        }
        Ok(response)
    }
}

/// What `response` sends, `max` bytes of it at most, whether its server
/// says how much it sends or not.
async fn body(mut response: Response, max: usize) -> Result<Vec<u8>, PageError> {
    let too_large = || unreachable(format!("weighs more than {max} bytes"));
    if response.content_length().unwrap_or(0) > max as u64 {
        return Err(too_large());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(failed)? {
        if bytes.len() + chunk.len() > max {
            return Err(too_large());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

#[async_trait::async_trait]
impl Pages for Web {
    async fn fetch(&self, url: &Url) -> Result<String, PageError> {
        let response = self.get(url, "text/html,application/xhtml+xml").await?;
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let mut parts = content_type.split(';').map(str::trim);
        if !matches!(parts.next(), Some("text/html" | "application/xhtml+xml")) {
            return Err(PageError::NotHtml);
        }
        let encoding = parts
            .find_map(|part| part.strip_prefix("charset="))
            .and_then(|label| Encoding::for_label(label.trim_matches('"').as_bytes()))
            .unwrap_or(UTF_8);

        let bytes = body(response, PAGE_MAX).await?;
        Ok(encoding.decode(&bytes).0.into_owned())
    }

    async fn image(&self, url: &Url) -> Result<Bytes, PageError> {
        // The kinds that are kept: a site that has others sends one of
        // these when asked.
        let response = self
            .get(url, "image/webp,image/jpeg,image/png,image/gif")
            .await?;
        Ok(body(response, MAX_IMAGE_BYTES).await?.into())
    }
}

fn unreachable(reason: impl Into<String>) -> PageError {
    PageError::Unreachable(reason.into())
}

fn failed(err: reqwest::Error) -> PageError {
    unreachable(format!("{:#}", anyhow::Error::from(err)))
}

/// Resolves a name to its public addresses only, so that the connection
/// itself never goes anywhere else.
struct PublicOnly;

impl Resolve for PublicOnly {
    fn resolve(&self, name: Name) -> Resolving {
        Box::pin(async move {
            let addrs: Vec<SocketAddr> = tokio::net::lookup_host((name.as_str(), 0))
                .await?
                .filter(|addr| is_public(addr.ip()))
                .collect();
            if addrs.is_empty() {
                return Err("the name has no public address".into());
            }
            Ok(Box::new(addrs.into_iter()) as Addrs)
        })
    }
}

/// Whether `url` is one the api may ask: over HTTP, without credentials
/// and, when its host is an address rather than a name, a public one. A
/// name is checked when it is resolved ([`PublicOnly`]).
fn allowed(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };
    let public = host
        .trim_matches(['[', ']'])
        .parse::<IpAddr>()
        .map_or(true, is_public);
    matches!(url.scheme(), "http" | "https")
        && url.username().is_empty()
        && url.password().is_none()
        && public
}

/// Whether `ip` is an address of the internet at large: not this machine,
/// not the network it sits in, nor anything reserved.
fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_v4(ip),
        IpAddr::V6(ip) => match embedded_v4(ip) {
            Some(ip) => is_public_v4(ip),
            None => is_public_v6(ip),
        },
    }
}

fn is_public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        // "This network".
        || a == 0
        // Shared between a provider and its customers (100.64.0.0/10).
        || (a == 100 && b & 0xc0 == 64)
        // Protocol assignments (192.0.0.0/24).
        || (a == 192 && b == 0 && c == 0)
        // Benchmarks (198.18.0.0/15).
        || (a == 198 && b & 0xfe == 18)
        // Reserved (240.0.0.0/4).
        || a >= 240)
}

/// The IPv4 address `ip` stands for, when it is one written as IPv6:
/// mapped (`::ffff:a.b.c.d`) or translated (`64:ff9b::a.b.c.d`).
fn embedded_v4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    match ip.segments() {
        [0, 0, 0, 0, 0, 0xffff, high, low] | [0x64, 0xff9b, 0, 0, 0, 0, high, low] => {
            let [a, b] = high.to_be_bytes();
            let [c, d] = low.to_be_bytes();
            Some(Ipv4Addr::new(a, b, c, d))
        }
        _ => None,
    }
}

fn is_public_v6(ip: Ipv6Addr) -> bool {
    let [first, second, ..] = ip.segments();
    // Global unicast (2000::/3), less what is set apart within it:
    // protocol assignments and Teredo (2001::/23), documentation
    // (2001:db8::/32) and 6to4 (2002::/16), which names an IPv4 address.
    first & 0xe000 == 0x2000
        && !(first == 0x2001 && second < 0x0200)
        && !(first == 0x2001 && second == 0x0db8)
        && first != 0x2002
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    #[test]
    fn only_public_addresses_are_public() {
        for public in [
            "93.184.216.34",
            "1.1.1.1",
            "2606:4700::1111",
            "::ffff:1.1.1.1",
        ] {
            assert!(is_public(public.parse().unwrap()), "{public}");
        }
        for internal in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "192.0.0.1",
            "198.18.0.1",
            "224.0.0.1",
            "255.255.255.255",
            "::1",
            "::",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
            "64:ff9b::a00:1",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "ff02::1",
            "2001:db8::1",
            "2001::1",
            "2002:7f00:1::1",
        ] {
            assert!(!is_public(internal.parse().unwrap()), "{internal}");
        }
    }

    #[test]
    fn only_public_web_addresses_are_allowed() {
        for allowed_url in [
            "https://www.marmiton.org/recettes/recette_81518.aspx",
            "http://example.com:8080/",
            "http://1.1.1.1/",
            "http://[2606:4700::1111]/",
        ] {
            assert!(allowed(&url(allowed_url)), "{allowed_url}");
        }
        for refused in [
            "ftp://example.com/",
            "file:///etc/passwd",
            "https://user:password@example.com/",
            "http://127.0.0.1/",
            "http://127.1/",
            "http://2130706433/",
            "http://0x7f.0.0.1/",
            "http://[::1]/",
            "http://[::ffff:127.0.0.1]/",
            "http://169.254.169.254/latest/meta-data/",
            "http://192.168.0.1:8080/",
        ] {
            assert!(!allowed(&url(refused)), "{refused}");
        }
    }

    /// A server that answers every connection with `response`, and the
    /// address it listens on.
    async fn serve(response: Vec<u8>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let response = response.clone();
                tokio::spawn(async move {
                    let mut request = [0; 4096];
                    let _ = stream.read(&mut request).await;
                    let _ = stream.write_all(&response).await;
                    let _ = stream.shutdown().await;
                });
            }
        });
        format!("http://127.0.0.1:{port}/")
    }

    fn answer(head: &str, body: &[u8]) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {head}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend_from_slice(body);
        response
    }

    #[tokio::test]
    async fn a_page_is_read_as_its_server_says_it_is_written() {
        let web = Web::build(false).unwrap();

        let page = serve(answer(
            "200 OK\r\nContent-Type: text/html; charset=ISO-8859-1",
            b"<p>cr\xeapes</p>",
        ))
        .await;
        assert_eq!(web.fetch(&url(&page)).await.unwrap(), "<p>crêpes</p>");

        let page = serve(answer(
            "200 OK\r\nContent-Type: Text/HTML",
            "<p>crêpes</p>".as_bytes(),
        ))
        .await;
        assert_eq!(web.fetch(&url(&page)).await.unwrap(), "<p>crêpes</p>");
    }

    #[tokio::test]
    async fn what_is_not_a_page_within_bounds_is_refused() {
        let web = Web::build(false).unwrap();

        let pdf = serve(answer("200 OK\r\nContent-Type: application/pdf", b"%PDF")).await;
        assert_eq!(web.fetch(&url(&pdf)).await, Err(PageError::NotHtml));

        let missing = serve(answer("404 Not Found\r\nContent-Type: text/html", b"")).await;
        let err = web.fetch(&url(&missing)).await.unwrap_err();
        assert!(matches!(&err, PageError::Unreachable(reason) if reason.contains("404")));

        // Whether the server says how much it sends or not.
        let body = vec![b'a'; PAGE_MAX + 1];
        let large = serve(answer("200 OK\r\nContent-Type: text/html", &body)).await;
        let err = web.fetch(&url(&large)).await.unwrap_err();
        assert!(matches!(&err, PageError::Unreachable(reason) if reason.contains("weighs")));
        let mut unsized_body =
            b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n".to_vec();
        unsized_body.extend_from_slice(&body);
        let large = serve(unsized_body).await;
        let err = web.fetch(&url(&large)).await.unwrap_err();
        assert!(matches!(&err, PageError::Unreachable(reason) if reason.contains("weighs")));
    }

    #[tokio::test]
    async fn a_picture_is_fetched_as_it_is_within_its_bound() {
        let web = Web::build(false).unwrap();

        let picture = serve(answer("200 OK\r\nContent-Type: image/webp", b"RIFF")).await;
        assert_eq!(web.image(&url(&picture)).await.unwrap(), &b"RIFF"[..]);

        let body = vec![b'a'; MAX_IMAGE_BYTES + 1];
        let large = serve(answer("200 OK\r\nContent-Type: image/jpeg", &body)).await;
        let err = web.image(&url(&large)).await.unwrap_err();
        assert!(matches!(&err, PageError::Unreachable(reason) if reason.contains("weighs")));

        let guarded = Web::new().unwrap();
        let err = guarded.image(&url(&picture)).await.unwrap_err();
        assert!(
            matches!(err, PageError::Unreachable(_)),
            "an internal address"
        );
    }

    #[tokio::test]
    async fn an_internal_address_is_never_asked() {
        let web = Web::new().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        for internal in [
            format!("http://127.0.0.1:{port}/"),
            // A name that resolves to this machine.
            format!("http://localhost:{port}/"),
        ] {
            let err = web.fetch(&url(&internal)).await.unwrap_err();
            assert!(matches!(err, PageError::Unreachable(_)), "{internal}");
        }
        let asked = tokio::time::timeout(Duration::from_millis(100), listener.accept()).await;
        assert!(asked.is_err(), "nothing connected");
    }
}
