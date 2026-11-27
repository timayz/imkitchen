//! Who follows the api's events, and which.

use evento::Aggregate;

use crate::contact::ContactMessage;
use crate::identity::{AccountInvitation, Session, User};
use crate::image::Image;
use crate::plat::Plat;
use crate::recipe::Recipe;
use crate::recipe_import::RecipeImport;
use crate::settings::Settings;
use crate::suspension::Suspension;

/// One of the applications that build read models of their own from the
/// api's events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sidecar {
    /// The public and landing pages.
    Www,
    /// The administration.
    Admin,
    /// The application in a browser.
    App,
}

impl Sidecar {
    pub const ALL: [Sidecar; 3] = [Sidecar::Www, Sidecar::Admin, Sidecar::App];

    /// The name it goes by in the api's configuration and logs.
    pub fn name(self) -> &'static str {
        match self {
            Sidecar::Www => "www",
            Sidecar::Admin => "admin",
            Sidecar::App => "app",
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
            // The users, with their sessions and the invitations (what
            // `timada_identity::read_model_subscription` follows) and who
            // among them is suspended, and what the people who run
            // imkitchen look after: the recipes, how their imports went,
            // their pictures and the messages written to imkitchen.
            Sidecar::Admin => vec![
                User::aggregate_type(),
                Session::aggregate_type(),
                AccountInvitation::aggregate_type(),
                Suspension::aggregate_type(),
                Recipe::aggregate_type(),
                RecipeImport::aggregate_type(),
                Image::aggregate_type(),
                ContactMessage::aggregate_type(),
            ],
            // What a user reads in app: their recipes, the plats they
            // compose of them, and their settings.
            Sidecar::App => vec![
                Recipe::aggregate_type(),
                Plat::aggregate_type(),
                Settings::aggregate_type(),
            ],
        }
    }
}
