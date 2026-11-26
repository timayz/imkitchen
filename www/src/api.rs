//! The commands www sends to the api, over HTTP.
//!
//! www writes nothing of imkitchen itself: what a visitor does that is to
//! be kept is a command of the api (`api/README.md` says how they are
//! made), POSTed here. A command the api refuses comes back as it worded
//! it, in the language of the page it was made from, field by field when
//! the visitor can correct it ([`Refusal`]).

use std::time::Duration;

use anyhow::Context;
use reqwest::header::ACCEPT_LANGUAGE;
use serde::{Deserialize, Serialize};
use topcoat::l10n::Locale;

use crate::config::ApiConfig;

/// How long a command waits for the api before it is given up.
const TIMEOUT: Duration = Duration::from_secs(10);

/// The api, for the commands of www.
#[derive(Debug, Clone)]
pub struct Api {
    http: reqwest::Client,
    /// Where the api answers HTTP, without a trailing slash.
    url: String,
}

/// Why a command was not done.
#[derive(Debug)]
pub enum CommandError {
    /// The api refused it, and says why.
    Refused(Refusal),
    /// The api did not answer, or not the way it answers.
    Unreachable(anyhow::Error),
}

/// A refusal of the api, as it worded it for the visitor.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Refusal {
    /// The status it was answered with.
    #[serde(skip)]
    pub status: u16,
    /// The stable word of the refusal: `validation_failed`, …
    pub code: String,
    /// The refusal said to a person, in the language of the page.
    pub message: String,
    /// Under `validation_failed`: what each field does not hold.
    #[serde(default)]
    pub fields: Vec<FieldRefusal>,
}

/// What one field of a command does not hold.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FieldRefusal {
    /// The path of the field from the body of the command.
    pub field: String,
    pub code: String,
    pub message: String,
}

impl Refusal {
    /// What the api says of `field`, when it refuses it.
    pub fn field(&self, field: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|refused| refused.field == field)
            .map(|refused| refused.message.as_str())
    }
}

/// The one shape of the api's refusals.
#[derive(Deserialize)]
struct ErrorBody {
    error: Refusal,
}

/// What a visitor writes to imkitchen: the body of
/// `POST /v1/contact-messages`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ContactMessage {
    pub name: String,
    pub email: String,
    pub message: String,
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

    /// Sends `message` to imkitchen, written in `language`.
    pub async fn send_contact_message(
        &self,
        language: &Locale,
        message: &ContactMessage,
    ) -> Result<(), CommandError> {
        self.post("/v1/contact-messages", language, message).await
    }

    /// POSTs the command `body` to `path`, for a visitor who reads
    /// `language`: the api words what it refuses in it.
    async fn post<B: Serialize>(
        &self,
        path: &str,
        language: &Locale,
        body: &B,
    ) -> Result<(), CommandError> {
        let response = self
            .http
            .post(format!("{}{path}", self.url))
            .header(ACCEPT_LANGUAGE, language.id().to_string())
            .json(body)
            .send()
            .await
            .with_context(|| format!("the api does not answer at {}", self.url))
            .map_err(CommandError::Unreachable)?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let refusal = response
            .json::<ErrorBody>()
            .await
            .with_context(|| format!("{path} answered {status}, and not as the api refuses"))
            .map_err(CommandError::Unreachable)?;
        Err(CommandError::Refused(Refusal {
            status: status.as_u16(),
            ..refusal.error
        }))
    }
}
