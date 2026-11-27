//! The api over HTTP: who reads app, and what they do.
//!
//! app knows nobody itself: a user registers and signs in at the api, which
//! answers a session token (`api/README.md` says how), and app asks the api
//! whose token it is on every page.
//!
//! app writes nothing itself either: what a user does is a command of the
//! api's, sent with their token ([`Api::command`]). What the api refuses
//! comes back as it worded it ([`Refusal`]), in the language the page is
//! read in, each field that is not valid under its name.

use std::time::Duration;

use anyhow::{Context, anyhow};
use reqwest::header::{ACCEPT_LANGUAGE, USER_AGENT};
use reqwest::{Method, RequestBuilder, Response, StatusCode, Url};
use serde::{Deserialize, Serialize};

use crate::config::ApiConfig;

/// How long the api is waited for before it is given up.
const TIMEOUT: Duration = Duration::from_secs(10);

/// The api, for the sessions of app and the commands of its users.
#[derive(Debug, Clone)]
pub struct Api {
    http: reqwest::Client,
    /// Where the api answers HTTP, without a trailing slash.
    url: String,
}

/// Who asks, as far as the api is told: the language it words its refusals
/// in and, for a command, the token of who sends it.
#[derive(Debug, Clone, Copy)]
pub struct Asking<'a> {
    /// What `Accept-Language` says: the language the page is read in.
    pub language: &'a str,
    pub token: Option<&'a str>,
}

/// Why the api did not do what it was asked: a sign-in, or a command.
#[derive(Debug)]
pub enum CommandError {
    /// The api refused it, and says why.
    Refused(Refusal),
    /// The api did not answer, or not the way it answers: whether it did
    /// what was asked is not known.
    Unreachable(anyhow::Error),
}

/// A refusal of the api, as it worded it for the person.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Refusal {
    /// The status it was answered with.
    #[serde(skip)]
    pub status: u16,
    /// The stable word of the refusal: `invalid_credentials`, …
    pub code: String,
    /// The refusal said to a person.
    pub message: String,
    /// With `validation_failed`: what each field does not hold.
    #[serde(default)]
    pub fields: Vec<FieldRefusal>,
}

impl Refusal {
    /// What the api says of the field `name`, when it refused it.
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|refused| refused.field == name)
            .map(|refused| refused.message.as_str())
    }

    /// Whether the session of who asked ended: they sign in again.
    pub fn signed_out(&self) -> bool {
        self.status == StatusCode::UNAUTHORIZED.as_u16()
    }
}

/// A field of a body the api refused, as it worded it for the person.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FieldRefusal {
    /// The path of the field in the body: `name`, `ingredients[2].quantity`.
    pub field: String,
    pub code: String,
    pub message: String,
}

/// The one shape of the api's refusals.
#[derive(Deserialize)]
struct ErrorBody {
    error: Refusal,
}

/// What registers and what signs in: the body of `POST /v1/auth/register`
/// and of `POST /v1/auth/sign-in`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
    /// The mailed code: left out to ask for it, given to finish.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// How far a registration or a sign-in went.
#[derive(Debug)]
pub enum SignIn {
    /// A code was mailed to the address: the same again with the code
    /// finishes.
    CodeSent,
    Done(Session),
}

/// A session just opened: the only time the api says its token.
#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    pub token: String,
    /// Unix seconds.
    pub expires_at: u64,
    pub user: Account,
}

/// Who a token signs in, and until when (unix seconds) if it is not used
/// again.
#[derive(Debug, Clone, Deserialize)]
pub struct Me {
    pub expires_at: u64,
    pub user: Account,
}

/// A user, as the api tells of them.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Account {
    pub id: String,
    /// The address they sign in with.
    pub email: String,
    pub name: String,
}

impl Api {
    pub fn new(config: &ApiConfig) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .context("cannot build the client of the api")?;
        Ok(Self {
            http,
            url: config.url.trim_end_matches('/').to_owned(),
        })
    }

    /// Creates the account of `credentials`, from the browser that names
    /// itself `user_agent`, and signs it in.
    pub async fn register(
        &self,
        credentials: &Credentials,
        user_agent: Option<&str>,
        language: &str,
    ) -> Result<SignIn, CommandError> {
        self.open_session("/v1/auth/register", credentials, user_agent, language)
            .await
    }

    /// Signs in with `credentials`, from the browser that names itself
    /// `user_agent`: what the account's history keeps of the sign-in.
    pub async fn sign_in(
        &self,
        credentials: &Credentials,
        user_agent: Option<&str>,
        language: &str,
    ) -> Result<SignIn, CommandError> {
        self.open_session("/v1/auth/sign-in", credentials, user_agent, language)
            .await
    }

    /// The two steps registering and signing in share: a 202 says a code
    /// was mailed, a 201 the session.
    async fn open_session(
        &self,
        path: &str,
        credentials: &Credentials,
        user_agent: Option<&str>,
        language: &str,
    ) -> Result<SignIn, CommandError> {
        let mut request = self.http.post(self.at(path)).json(credentials);
        if let Some(user_agent) = user_agent {
            request = request.header(USER_AGENT, user_agent);
        }
        let response = self
            .send(request, language)
            .await
            .map_err(CommandError::Unreachable)?;
        let status = response.status();
        match status {
            StatusCode::ACCEPTED => Ok(SignIn::CodeSent),
            StatusCode::CREATED => response
                .json()
                .await
                .map(SignIn::Done)
                .with_context(|| format!("{path} answered {status}, and not a session"))
                .map_err(CommandError::Unreachable),
            _ => Err(refused(path, response).await),
        }
    }

    /// Who `token` signs in: nobody once its session ended, whatever ended
    /// it.
    pub async fn account(&self, token: &str) -> anyhow::Result<Option<Me>> {
        let path = "/v1/account";
        let response = self
            .http
            .get(self.at(path))
            .bearer_auth(token)
            .send()
            .await
            .with_context(|| format!("the api does not answer at {}", self.url))?;
        match response.status() {
            StatusCode::OK => response
                .json()
                .await
                .map(Some)
                .with_context(|| format!("{path} answered, and not an account")),
            StatusCode::UNAUTHORIZED => Ok(None),
            status => Err(anyhow!("{path} answered {status}")),
        }
    }

    /// Ends the session `token` opened. One that ended already is ended.
    pub async fn sign_out(&self, token: &str) -> anyhow::Result<()> {
        let path = "/v1/auth/sign-out";
        let response = self
            .http
            .post(self.at(path))
            .bearer_auth(token)
            .send()
            .await
            .with_context(|| format!("the api does not answer at {}", self.url))?;
        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::UNAUTHORIZED => Ok(()),
            status => Err(anyhow!("{path} answered {status}")),
        }
    }

    /// Sends the command `method` on `thing` (the segments of its address
    /// under `/v1`, an id among them written as it is) with `body` as its
    /// JSON, when it takes one. Done is a 204, or a 201 for a command that
    /// creates something: what it created is the next page's to read, from
    /// what app follows.
    pub async fn command<B: Serialize + ?Sized>(
        &self,
        asking: Asking<'_>,
        method: Method,
        thing: &[&str],
        body: Option<&B>,
    ) -> Result<(), CommandError> {
        let path = format!("/v1/{}", thing.join("/"));
        // An id comes from the address of a page: it stays one segment
        // whatever it holds.
        let url = Url::parse(&self.url)
            .ok()
            .and_then(|mut url| {
                url.path_segments_mut()
                    .ok()?
                    .pop_if_empty()
                    .push("v1")
                    .extend(thing);
                Some(url)
            })
            .ok_or_else(|| anyhow!("{} is not an address commands are sent to", self.url))
            .map_err(CommandError::Unreachable)?;
        let mut request = self.http.request(method, url);
        if let Some(token) = asking.token {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = self
            .send(request, asking.language)
            .await
            .map_err(CommandError::Unreachable)?;
        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::CREATED => Ok(()),
            _ => Err(refused(&path, response).await),
        }
    }

    fn at(&self, path: &str) -> String {
        format!("{}{path}", self.url)
    }

    /// Sends `request`, asking for the refusals in `language`.
    async fn send(&self, request: RequestBuilder, language: &str) -> anyhow::Result<Response> {
        request
            .header(ACCEPT_LANGUAGE, language)
            .send()
            .await
            .with_context(|| format!("the api does not answer at {}", self.url))
    }
}

/// What `path` answered when it did not do what was asked: the refusal of
/// the api, or an answer that is not one of its own.
async fn refused(path: &str, response: Response) -> CommandError {
    let status = response.status();
    match response.json::<ErrorBody>().await {
        Ok(body) => CommandError::Refused(Refusal {
            status: status.as_u16(),
            ..body.error
        }),
        Err(err) => CommandError::Unreachable(anyhow::Error::new(err).context(format!(
            "{path} answered {status}, and not as the api refuses"
        ))),
    }
}
