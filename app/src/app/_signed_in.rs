//! What a signed-in user reads: every page of app but those of [`auth`].
//! `/` is the home, and each screen of the application (the recipes, the
//! dishes, the list, the groceries, the settings) is at the address of its
//! module below.
//!
//! The module adds nothing to the addresses of its pages (its name starts
//! with `_`). What it adds is who may read them: whatever is declared here
//! or below is behind [`signed_in_only`].
//!
//! One layout serves the desktop and the phone: the navigation between the
//! screens is a column beside the page where there is room for one, and a
//! bar under it where there is not.
//!
//! [`auth`]: super::auth

pub mod dishes;
pub mod groceries;
pub mod list;
pub mod recipes;
pub mod settings;

use topcoat::{
    Error, Result,
    context::{Cx, context},
    l10n::Translation,
    router::{
        Body, Next, Slot, StatusCode,
        error::{NotFoundError, see_other, service_unavailable},
        href, layer, layout, not_found, page,
        request::uri,
        response::Response,
    },
    view::{View, class, component, error_boundary, view},
};

use super::{api, auth, db, t};
use crate::api::Account;
use crate::session::{self, Kept};

/// How long a browser waits before it asks again, when the api could not
/// say who it is: in seconds.
const RETRY_AFTER_SECS: u64 = 5;

// Anything else is the "not found" page, of a signed-in user's too.
not_found!();

/// Who reads the page.
pub(crate) fn account(cx: &Cx) -> &Account {
    context(cx)
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
    next.run(&cx.with(me.user), body).await
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
                </form>
            </header>
            // The bar of a phone covers the bottom of the page: the page
            // ends above it.
            <div class="min-w-0 flex-1 pb-16 md:pb-0">
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

/// The home: what the application is, and how many recipes app has
/// followed. Whose they are is for the screens to tell.
#[page]
pub async fn home(cx: &Cx) -> Result<impl View> {
    let recipes = crate::recipes::count(db(cx))
        .await
        .map_err(Error::from_anyhow)?;

    Ok(view! {
        <main class="mx-auto max-w-3xl px-4 py-8 md:px-8 md:py-12">
            <h1 class="text-3xl font-bold tracking-tight md:text-4xl">(t!(home_title))</h1>
            <p class="mt-4 text-lg leading-relaxed text-stone-600">(t!(home_text))</p>
            <p class="mt-8 text-sm text-stone-500" data-recipes=(recipes)>
                (t!(home_recipes, count: recipes))
            </p>
        </main>
    })
}

/// A screen nothing is shown on yet: what it is, and what it will hold.
#[component]
pub(crate) async fn screen(
    cx: &Cx,
    title: Translation<'static>,
    text: Translation<'static>,
) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-3xl px-4 py-8 md:px-8 md:py-12">
            <h1 class="text-3xl font-bold tracking-tight md:text-4xl">(title)</h1>
            <p class="mt-4 text-lg leading-relaxed text-stone-600">(text)</p>
            <p class="mt-8 text-sm text-stone-500" data-screen="coming">(t!(screen_coming))</p>
        </main>
    })
}
