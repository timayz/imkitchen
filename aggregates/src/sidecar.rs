//! Who follows the api's events, and which.

use evento::Aggregate;

use crate::identity::{AccountInvitation, Session, User};
use crate::recipe::Recipe;

/// One of the applications that build read models of their own from the
/// api's events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sidecar {
    /// The public and landing pages.
    Www,
    /// The administration.
    Admin,
}

impl Sidecar {
    pub const ALL: [Sidecar; 2] = [Sidecar::Www, Sidecar::Admin];

    /// The name it goes by in the api's configuration and logs.
    pub fn name(self) -> &'static str {
        match self {
            Sidecar::Www => "www",
            Sidecar::Admin => "admin",
        }
    }

    /// The aggregate types it reads. Whatever is not listed stays in the
    /// api: an aggregate is added here when a read model needs it.
    ///
    /// The indexes and the mailed codes of the identity (`EmailAccount`,
    /// `OAuth2Account`, `PasskeyCredential`, `EmailChallenge`) are on no
    /// list: they are what signing in is checked against, and only the api
    /// signs anyone in.
    pub fn reads(self) -> Vec<&'static str> {
        match self {
            // Nothing of a user is public: of the recipes www keeps and
            // shows how many there are, and nothing they say.
            Sidecar::Www => vec![Recipe::aggregate_type()],
            // The users, of whom admin keeps how many there are, with their
            // sessions and the invitations: what
            // `timada_identity::read_model_subscription` follows.
            Sidecar::Admin => vec![
                User::aggregate_type(),
                Session::aggregate_type(),
                AccountInvitation::aggregate_type(),
            ],
        }
    }
}
