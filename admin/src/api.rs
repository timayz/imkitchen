//! Who reads admin, as the api says, over HTTP.
//!
//! admin knows nobody itself: an administrator signs in at the api, which
//! answers a session token (`api/README.md` says how), and admin asks the
//! api whose token it is on every page. What the api refuses of a sign-in
//! comes back as it worded it ([`Refusal`]).

use std::time::Duration;

use anyhow::{Context, anyhow};
use reqwest::header::{ACCEPT_LANGUAGE, USER_AGENT};
use reqwest::{RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};

use crate::config::ApiConfig;

/// How long the api is waited for before it is given up.
const TIMEOUT: Duration = Duration::from_secs(10);

/// The language admin is read in: what the api words its refusals in.
const LANGUAGE: &str = "en";

/// The api, for the sessions of admin.
#[derive(Debug, Clone)]
pub struct Api {
    http: reqwest::Client,
    /// Where the api answers HTTP, without a trailing slash.
    url: String,
}

/// Why a sign-in was not done.
#[derive(Debug)]
pub enum SignInError {
    /// The api refused it, and says why.
    Refused(Refusal),
    /// The api did not answer, or not the way it answers.
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
}

/// The one shape of the api's refusals.
#[derive(Deserialize)]
struct ErrorBody {
    error: Refusal,
}

/// What signs in: the body of `POST /v1/auth/sign-in`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
    /// The mailed code: left out to ask for it, given to finish.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// How far a sign-in went.
#[derive(Debug)]
pub enum SignIn {
    /// The password is the account's, and a code was mailed to its address:
    /// the same again with the code finishes.
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
    pub role: Role,
}

impl Account {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }
}

/// What a user may do. Only an administrator reads admin: whatever else the
/// api comes to call a role is not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    #[serde(other)]
    Other,
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

    /// Signs in with `credentials`, from the browser that names itself
    /// `user_agent`: what the account's history keeps of the sign-in.
    pub async fn sign_in(
        &self,
        credentials: &Credentials,
        user_agent: Option<&str>,
    ) -> Result<SignIn, SignInError> {
        let path = "/v1/auth/sign-in";
        let mut request = self.http.post(self.at(path)).json(credentials);
        if let Some(user_agent) = user_agent {
            request = request.header(USER_AGENT, user_agent);
        }
        let response = self.send(request).await.map_err(SignInError::Unreachable)?;
        let status = response.status();
        match status {
            StatusCode::ACCEPTED => Ok(SignIn::CodeSent),
            StatusCode::CREATED => response
                .json()
                .await
                .map(SignIn::Done)
                .with_context(|| format!("{path} answered {status}, and not a session"))
                .map_err(SignInError::Unreachable),
            _ => {
                let refusal = response
                    .json::<ErrorBody>()
                    .await
                    .with_context(|| {
                        format!("{path} answered {status}, and not as the api refuses")
                    })
                    .map_err(SignInError::Unreachable)?;
                Err(SignInError::Refused(Refusal {
                    status: status.as_u16(),
                    ..refusal.error
                }))
            }
        }
    }

    /// Who `token` signs in: nobody once its session ended, whatever ended
    /// it.
    pub async fn account(&self, token: &str) -> anyhow::Result<Option<Me>> {
        let path = "/v1/account";
        let response = self
            .send(self.http.get(self.at(path)).bearer_auth(token))
            .await?;
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
            .send(self.http.post(self.at(path)).bearer_auth(token))
            .await?;
        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::UNAUTHORIZED => Ok(()),
            status => Err(anyhow!("{path} answered {status}")),
        }
    }

    fn at(&self, path: &str) -> String {
        format!("{}{path}", self.url)
    }

    async fn send(&self, request: RequestBuilder) -> anyhow::Result<Response> {
        request
            .header(ACCEPT_LANGUAGE, LANGUAGE)
            .send()
            .await
            .with_context(|| format!("the api does not answer at {}", self.url))
    }
}
