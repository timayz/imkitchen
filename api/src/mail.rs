//! Mails the one-time codes that prove an address.
//!
//! The command that issues a code mails nothing: it records the intent, and
//! [`code_mails_subscription`] sees it on every node. The node that writes
//! its claim on the challenge first is the one that mails; it derives the
//! code from the nonce the event carries, under the secret the nodes share,
//! and records how the send went. A server that refuses the address for good
//! leaves the challenge undeliverable, and the person may ask again at once;
//! one that does not answer is tried again later, for as long as the code
//! lives. The send runs off the subscription's cursor, so a mail server
//! that is down holds nothing else up; a node that stops in between loses
//! the mail, and the person asks for another code.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use lettre::message::Mailbox;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use timada_core::{Claimant, Sending, unix_now};
use timada_identity::aggregator::CodeIssued;
use timada_identity::{ChallengePurpose, CodeMail, CodeVerifier, Command};

use crate::cluster::Executor;
use crate::config::MailConfig;

/// The cursor identity of the subscription, in the store. One per node.
pub const CODE_MAILS_SUBSCRIPTION: &str = "imkitchen-api/code-mails";

/// Why a mail did not go out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MailError {
    /// The server refused the address for good, or it is not one.
    Undeliverable(String),
    /// Worth another try: the server did not answer, or not for good.
    NotSent(String),
}

/// What sends the mails.
#[async_trait::async_trait]
pub trait Mailer: Send + Sync {
    async fn send_code(&self, mail: &CodeMail) -> Result<(), MailError>;
}

/// The configured SMTP server.
pub struct Smtp {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl Smtp {
    pub fn new(config: &MailConfig) -> anyhow::Result<Self> {
        Ok(Self {
            transport: AsyncSmtpTransport::<Tokio1Executor>::from_url(config.smtp_url.expose())
                .context("mail.smtp_url is not an SMTP URL")?
                .build(),
            from: config
                .from
                .parse()
                .context("mail.from is not `name <address>`")?,
        })
    }
}

#[async_trait::async_trait]
impl Mailer for Smtp {
    async fn send_code(&self, mail: &CodeMail) -> Result<(), MailError> {
        let to = mail
            .email
            .parse::<Mailbox>()
            .map_err(|err| MailError::Undeliverable(err.to_string()))?;
        let message = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(subject(mail.purpose))
            .body(body(mail))
            .map_err(|err| MailError::NotSent(err.to_string()))?;
        match self.transport.send(message).await {
            Ok(_) => Ok(()),
            Err(err) if err.is_permanent() => Err(MailError::Undeliverable(err.to_string())),
            Err(err) => Err(MailError::NotSent(err.to_string())),
        }
    }
}

fn subject(purpose: ChallengePurpose) -> &'static str {
    match purpose {
        ChallengePurpose::Register => "Your imkitchen code to create your account",
        ChallengePurpose::SignIn => "Your imkitchen code to sign in",
        ChallengePurpose::ResetPassword => "Your imkitchen code to change your password",
    }
}

fn body(mail: &CodeMail) -> String {
    let what = match mail.purpose {
        ChallengePurpose::Register => "create your account",
        ChallengePurpose::SignIn => "sign in",
        ChallengePurpose::ResetPassword => "change your password",
    };
    format!(
        "Your code to {what}: {}\n\n\
         It works for 10 minutes. If you did not ask for it, ignore this mail: \
         nothing happens without the code.\n",
        mail.code
    )
}

/// What a send needs, shared by the handler and the sends it starts.
#[derive(Clone)]
struct Mails {
    identity: Arc<Command<Executor>>,
    mailer: Arc<dyn Mailer>,
    /// What this node signs its claims with.
    node: Arc<str>,
}

/// Follows the challenges and mails their codes, writing through
/// `executor`. Every node runs it; the node that claims a code's mail first
/// is the one that sends it. `node` names this one among them.
pub fn code_mails_subscription(
    executor: &Executor,
    mailer: Arc<dyn Mailer>,
    codes: Arc<dyn CodeVerifier>,
    node: &str,
) -> SubscriptionBuilder<Executor> {
    SubscriptionBuilder::new(CODE_MAILS_SUBSCRIPTION)
        .data(Mails {
            identity: Arc::new(Command::new(executor.clone(), codes)),
            mailer,
            node: node.into(),
        })
        .handler(on_code_issued())
        .retry(5)
}

#[evento::subscription]
async fn on_code_issued<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<CodeIssued>,
) -> anyhow::Result<()> {
    // A code of the past, met again by a cursor that starts over.
    if event.data.expires_at <= unix_now() {
        return Ok(());
    }
    let mails: Mails = context.extract();
    let challenge_id = event.aggregate_id.clone();
    let issued_version = event.version;
    tokio::spawn(async move {
        if let Err(err) = deliver(&mails, &challenge_id, issued_version).await {
            tracing::error!(err = format!("{err:#}"), challenge_id, "cannot mail a code");
        }
    });
    Ok(())
}

/// Claims the mail of the code `CodeIssued` at `issued_version` recorded,
/// sends it and records how it went, again after each failure worth a retry
/// until the challenge says to stop: the code was used, replaced or expired,
/// another node took the send, or the tries ran out.
async fn deliver(mails: &Mails, challenge_id: &str, issued_version: u16) -> anyhow::Result<()> {
    let identity = &mails.identity;
    loop {
        let now = unix_now();
        let claimant = Claimant {
            node: &mails.node,
            now,
        };
        let Some(mail) = identity
            .claim_code_mail(challenge_id, issued_version, claimant)
            .await?
        else {
            return Ok(());
        };
        let reason = match mails.mailer.send_code(&mail).await {
            Ok(()) => {
                identity
                    .record_code_mail_sent(challenge_id, issued_version)
                    .await?;
                return Ok(());
            }
            Err(MailError::Undeliverable(reason)) => {
                tracing::info!(reason, "a code's address was refused");
                identity
                    .record_code_mail_undeliverable(challenge_id, issued_version, &reason)
                    .await?;
                return Ok(());
            }
            Err(MailError::NotSent(reason)) => reason,
        };
        tracing::warn!(reason, "a code was not mailed");
        identity
            .record_code_mail_not_sent(challenge_id, issued_version, &mails.node, &reason, now)
            .await?;
        let retry_at = match identity.load_email_challenge_by_id(challenge_id).await? {
            Some(challenge) if challenge.issued_version == issued_version => match challenge.mail {
                Sending::Deferred { retry_at, .. } => retry_at,
                _ => return Ok(()),
            },
            _ => return Ok(()),
        };
        tokio::time::sleep(Duration::from_secs(retry_at.saturating_sub(unix_now()))).await;
    }
}
