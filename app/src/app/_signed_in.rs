//! What a signed-in user reads: every page of app but those of [`auth`].
//! `/` is the home, and each screen of the application (the recipes, the
//! dishes, the list, the groceries, the settings) is at the address of its
//! module below.
//!
//! The module adds nothing to the addresses of its pages (its name starts
//! with `_`). What it adds is who may read them: whatever is declared here
//! or below is behind [`signed_in_only`].
//!
//! A page shows what the store of app holds, and nothing a user does is
//! written there: a form posts to its page, which hands it to the api as a
//! command (with [`token`], what the api refuses said as [`Failure`]) and
//! sends the browser on once app followed what the api wrote
//! ([`followed`]).
//!
//! The pages are read in the language their reader set, when they set one
//! ([`settings`]), whatever device they are on.
//!
//! One layout serves the desktop and the phone: the navigation between the
//! screens is a column beside the page where there is room for one, and a
//! bar under it where there is not. While the reader goes through the
//! tutorial, the step they are at is above every page but the home, where
//! the whole tutorial is ([`tutorial`]).
//!
//! [`auth`]: super::auth

pub mod dishes;
pub mod groceries;
pub mod list;
pub mod recipes;
pub mod settings;
pub mod tutorial;

use std::future::Future;
use std::time::Duration;

use tokio::time::{Instant, sleep};
use topcoat::{
    Error, Result,
    context::{Cx, context},
    l10n::{Locale, Translation},
    router::{
        Body, Next, Slot, StatusCode,
        error::{self, NotFoundError, see_other, service_unavailable},
        href, layer, layout, not_found, page,
        request::uri,
        response::Response,
    },
    view::{View, class, component, error_boundary, view},
};

use super::{api, auth, db, public, t};
use crate::api::{Account, Asking, CommandError, Refusal};
use crate::session::{self, Kept};

/// How long a browser waits before it asks again, when the api could not
/// say who it is: in seconds.
const RETRY_AFTER_SECS: u64 = 5;

// Anything else is the "not found" page, of a signed-in user's too.
not_found!();

/// How long a page waits for app to follow what a command just wrote,
/// before it sends the browser on anyway.
const FOLLOWED_WITHIN: Duration = Duration::from_secs(5);

/// Who reads the page.
pub(crate) fn account(cx: &Cx) -> &Account {
    context(cx)
}

/// The token of the session of who reads the page.
struct Token(String);

/// What the commands of who reads the page are sent with: their token,
/// asking for the refusals in `language`, the one the page is read in.
pub(crate) fn asking<'a>(cx: &'a Cx, language: &'a str) -> Asking<'a> {
    let Token(token) = context(cx);
    Asking {
        language,
        token: Some(token),
    }
}

/// Why a command was not done, as a page says it.
pub(crate) enum Failure {
    /// The api refused it, and says why.
    Refused(Refusal),
    /// The api could not be asked.
    Unreachable,
}

impl Failure {
    /// What a command that failed leaves a page to say. A session that
    /// ended is none of it: the browser is sent to the main domain, as on
    /// any page. Nor is a thing that does not exist, or is someone else's:
    /// that is the "not found" page.
    pub(crate) fn of(cx: &Cx, err: CommandError) -> Result<Self> {
        match err {
            CommandError::Refused(refusal) if refusal.signed_out() => Err(signed_out(cx)),
            CommandError::Refused(refusal) if refusal.status == StatusCode::NOT_FOUND.as_u16() => {
                Err(error::not_found().into())
            }
            CommandError::Refused(refusal) => Ok(Self::Refused(refusal)),
            CommandError::Unreachable(err) => {
                tracing::warn!(
                    err = format!("{err:#}"),
                    "a command was not handed to the api"
                );
                Ok(Self::Unreachable)
            }
        }
    }

    /// The status of the page that says it.
    pub(crate) fn status(&self) -> StatusCode {
        match self {
            Self::Refused(refusal) => {
                StatusCode::from_u16(refusal.status).unwrap_or(StatusCode::BAD_GATEWAY)
            }
            Self::Unreachable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// What the api says of the field `name`, when it refused it.
    pub(crate) fn field(&self, name: &str) -> Option<String> {
        match self {
            Self::Refused(refusal) => refusal.field(name).map(str::to_owned),
            Self::Unreachable => None,
        }
    }
}

/// Waits until app followed what a command just wrote, so that the page
/// the browser is sent to shows it: `shown` says whether it does. Not for
/// long: past [`FOLLOWED_WITHIN`] the page shows it a moment later.
pub(crate) async fn followed<F>(mut shown: impl FnMut() -> F)
where
    F: Future<Output = anyhow::Result<bool>>,
{
    let deadline = Instant::now() + FOLLOWED_WITHIN;
    // A read that fails is the next page's to say.
    while !shown().await.unwrap_or(true) && Instant::now() < deadline {
        sleep(Duration::from_millis(50)).await;
    }
}

/// A read of the store that failed, as the error of a page.
pub(crate) fn read<T>(read: anyhow::Result<T>) -> Result<T> {
    read.map_err(Error::from_anyhow)
}

/// The classes the pages share.
pub(crate) const MAIN: &str = "mx-auto max-w-3xl px-4 py-8 md:px-8 md:py-12";
pub(crate) const H1: &str = "text-3xl font-bold tracking-tight md:text-4xl";
pub(crate) const H2: &str = "mt-10 text-xl font-semibold tracking-tight";
pub(crate) const LINK: &str = "font-medium underline hover:text-stone-600";
pub(crate) const LABEL: &str = "text-sm font-medium";
pub(crate) const INPUT: &str =
    "mt-1 block w-full rounded-lg border border-stone-300 bg-white px-3 py-2";
pub(crate) const BUTTON: &str =
    "rounded-lg bg-stone-900 px-5 py-3 font-semibold text-stone-50 hover:bg-stone-700";
pub(crate) const BUTTON_QUIET: &str =
    "rounded-lg border border-stone-300 bg-white px-5 py-3 font-semibold hover:border-stone-500";
pub(crate) const FIELD_REFUSED: &str = "mt-1 text-sm text-red-800";

/// What a command that failed leaves to say above its form: the refusal as
/// the api worded it, or that it could not be asked.
#[component]
pub(crate) async fn failed(cx: &Cx, failure: Option<Failure>) -> Result<impl View> {
    let alert = "rounded-lg border border-red-300 bg-red-50 px-4 py-3 text-red-900";
    let status = failure.as_ref().map(Failure::status);
    let (refused, unreachable) = match failure {
        Some(Failure::Refused(refusal)) => (Some((refusal.code, refusal.message)), false),
        Some(Failure::Unreachable) => (None, true),
        None => (None, false),
    };

    Ok(view! {
        if let Some(status) = status {
            (status)
        }
        if let Some((code, message)) = refused {
            <p class=(alert) role="alert" data-command="refused" data-code=(code)>(message)</p>
        }
        if unreachable {
            <p class=(alert) role="alert" data-command="failed">(t!(command_failed))</p>
        }
    })
}

/// Lets a signed-in user through, and nobody else: the api is asked whose
/// session the browser keeps on every request, so one that ended reads
/// nothing further.
///
/// A browser that keeps nothing is sent to sign in. One that keeps a
/// session the api no longer knows, or the marker of one without it, has
/// both cookies taken back and is sent to the main domain: what the
/// gateway then serves it is www.
#[layer]
async fn signed_in_only(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let Some(kept) = session::kept(cx) else {
        if session::claimed(cx) {
            return Err(signed_out(cx));
        }
        return Err(see_other(href!(auth::sign_in::sign_in).resolve(cx)).into());
    };
    let me = match api(cx).account(&kept.token).await {
        Ok(Some(me)) => me,
        Ok(None) => return Err(signed_out(cx)),
        Err(err) => {
            tracing::warn!(
                err = format!("{err:#}"),
                "the api does not say who reads app"
            );
            return Err(service_unavailable(RETRY_AFTER_SECS).into());
        }
    };
    let token = Token(kept.token.clone());
    // The api extends a session that is used: so does the browser. And a
    // session never goes without its marker.
    if me.expires_at > kept.expires_at || !session::marked(cx) {
        session::keep(
            cx,
            Kept {
                expires_at: me.expires_at,
                ..kept
            },
        )?;
    }
    // The language they chose is the one their pages are read in, on every
    // device: the browser's is for who chose none.
    let chosen = read(crate::settings::of(db(cx), &me.user.id).await)?
        .language
        .and_then(|language| spoken(&language));
    let cx = cx.with(me.user).with(token);
    match chosen {
        Some(locale) => next.run(&cx.with(locale), body).await,
        None => next.run(&cx, body).await,
    }
}

/// The language `tag` names, when app speaks it.
pub(crate) fn spoken(tag: &str) -> Option<Locale> {
    let mut locales = t::locales().iter();
    locales
        .find(|locale| locale.id().to_string() == tag)
        .cloned()
}

/// Takes back what the browser keeps of a session the api does not know,
/// and sends it to the main domain.
pub(crate) fn signed_out(cx: &Cx) -> Error {
    session::forget(cx);
    see_other(href!(home).resolve(cx)).into()
}

/// A screen of the application, as the navigation leads to it.
struct Screen {
    address: String,
    name: Translation<'static>,
    /// Whether it is what is being read.
    current: bool,
}

/// The screens, in the order the navigation gives them.
fn screens(cx: &Cx) -> Vec<Screen> {
    let reading = uri(cx).path();
    [
        (href!(recipes::list).resolve(cx), t!(recipes_title)),
        (href!(dishes::list).resolve(cx), t!(dishes_title)),
        (href!(list::show).resolve(cx), t!(list_title)),
        (href!(groceries::show).resolve(cx), t!(groceries_title)),
        (href!(settings::show).resolve(cx), t!(settings_title)),
    ]
    .into_iter()
    .map(|(address, name)| {
        let address = address.to_string();
        Screen {
            // A screen is current on the pages under it too.
            current: reading
                .strip_prefix(address.as_str())
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('/')),
            address,
            name,
        }
    })
    .collect()
}

#[layout]
async fn signed_in(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let email = account(cx).email.clone();
    let screens = screens(cx);
    let brand = href!(home).resolve(cx).to_string();
    let back = brand.clone();
    // The home shows the whole tutorial, and so does skipping it when that
    // failed: the step the reader is at is above the other pages.
    let reading = uri(cx).path();
    let skipping = href!(tutorial::skip).resolve(cx).to_string();
    let whole_tutorial = reading == brand || reading == skipping;

    Ok(view! {
        // A phone: the name on top, the page, the screens in a bar that
        // stays at the bottom. From `md` up: the name and the screens in
        // a column that stays beside the page.
        <div class="min-h-dvh md:flex">
            <header
                class="flex items-center justify-between border-b border-stone-200 bg-white md:sticky md:top-0 md:h-dvh md:w-56 md:shrink-0 md:flex-col md:items-stretch md:justify-start md:border-r md:border-b-0"
            >
                <a
                    class="block px-4 py-3 text-xl font-semibold tracking-tight md:px-5 md:py-6"
                    href=(brand)
                    aria-label=(t!(nav_home))
                >
                    "imkitchen"
                </a>
                <nav
                    class="fixed inset-x-0 bottom-0 z-10 flex border-t border-stone-200 bg-white md:static md:flex-col md:gap-1 md:border-t-0 md:px-3"
                    aria-label=(t!(nav_screens))
                >
                    for screen in screens {
                        <a
                            class=(class!(
                                "min-w-0 flex-1 truncate px-1 py-3 text-center text-xs md:flex-none md:rounded-lg md:px-3 md:py-2 md:text-left md:text-sm",
                                "font-semibold text-stone-900 md:bg-stone-100" if screen.current,
                                "text-stone-500 hover:text-stone-900" if !screen.current,
                            ))
                            href=(screen.address)
                            aria-current=(screen.current.then_some("page"))
                        >
                            (screen.name)
                        </a>
                    }
                </nav>
                // Who is signed in, and the way out: beside the name on a
                // phone, under the screens from `md` up.
                <form
                    class="px-4 text-sm md:mt-auto md:px-5 md:py-6"
                    method="post"
                    action=(href!(auth::sign_out))
                >
                    <p class="hidden truncate text-stone-500 md:block" data-account=(email.as_str())>
                        (email.as_str())
                    </p>
                    <button class="font-medium underline hover:text-stone-600 md:mt-1" type="submit">
                        (t!(nav_sign_out))
                    </button>
                    // The public pages: www's, where it answers a signed-in
                    // user too.
                    <a
                        class="hidden text-stone-500 hover:text-stone-900 hover:underline md:mt-4 md:block"
                        href=(public(cx, ""))
                    >
                        (t!(public_about))
                    </a>
                </form>
            </header>
            // The bar of a phone covers the bottom of the page: the page
            // ends above it.
            <div class="min-w-0 flex-1 pb-16 md:pb-0">
                if !whole_tutorial {
                    tutorial::banner()
                }
                error_boundary(
                    fallback: |error| {
                        if error.downcast_ref::<NotFoundError>().is_none() {
                            return Err(error);
                        }

                        Ok(
                            view! {
                                (StatusCode::NOT_FOUND)
                                <main class="mx-auto max-w-3xl px-4 py-8 md:px-8 md:py-12">
                                    <h1 class="text-3xl font-bold tracking-tight">
                                        (t!(not_found_title))
                                    </h1>
                                    <p class="mt-4 text-stone-600">(t!(not_found_text))</p>
                                    <p class="mt-6">
                                        <a class="underline" href=(back)>
                                            (t!(not_found_home))
                                        </a>
                                    </p>
                                </main>
                            },
                        )
                    },
                    (slot)
                )
            </div>
        </div>
    })
}

/// The home: what the application is, how many recipes app has followed
/// (whose they are is for the screens to tell), and the tutorial of who
/// reads it, until they skip it.
#[page]
pub async fn home(cx: &Cx) -> Result<impl View> {
    Ok(view! { welcome(failure: None) })
}

/// The home, and what `failure` says of skipping the tutorial, when that
/// was just refused.
#[component]
pub(crate) async fn welcome(cx: &Cx, failure: Option<Failure>) -> Result<impl View> {
    let recipes = crate::recipes::count(db(cx))
        .await
        .map_err(Error::from_anyhow)?;

    Ok(view! {
        <main class="mx-auto max-w-3xl px-4 py-8 md:px-8 md:py-12">
            <h1 class="text-3xl font-bold tracking-tight md:text-4xl">(t!(home_title))</h1>
            <p class="mt-4 text-lg leading-relaxed text-stone-600">(t!(home_text))</p>
            tutorial::steps(failure: failure)
            <p class="mt-8 text-sm text-stone-500" data-recipes=(recipes)>
                (t!(home_recipes, count: recipes))
            </p>
        </main>
    })
}
