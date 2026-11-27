//! `/settings`: how the list of who reads the page is generated, the
//! language they read, and their account: the name it shows, its password
//! and the devices it is signed in on.
//!
//! Each part is a form of its own, posted to an address of its own under
//! `/settings` and saved as a whole by a command of the api's. What the api
//! refuses is said in the part it was asked of, the others as they stand.
//!
//! The settings are read from the store of app, which follows them: what
//! is set here is set on every device the user signs in on, the language
//! among it ([`super::signed_in_only`] reads every page in it). The account
//! is the api's alone: its name comes with who the session signs in.

use reqwest::Method;
use serde::{Deserialize, Serialize};
use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{content::Form, error::see_other, href, page, query_params},
    view::{View, class, component, view},
};

use super::recipes::_form::Number;
use super::{
    BUTTON, BUTTON_QUIET, FIELD_REFUSED, Failure, H1, H2, INPUT, LABEL, MAIN, account, asking,
    failed, followed, read, spoken,
};
use crate::api::CommandError;
use crate::app::{api, db, language, t};
use crate::settings::{SOURCES, Settings};

/// A part of the settings: what one form sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    List,
    Language,
    Name,
    Password,
    Sessions,
}

impl Part {
    /// What the address of the page says once it was set.
    fn word(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Language => "language",
            Self::Name => "name",
            Self::Password => "password",
            Self::Sessions => "sessions",
        }
    }
}

#[query_params]
struct Query {
    /// The part that was just set.
    done: Option<String>,
}

/// The settings as the api tells of them: what app waits to have followed
/// before it shows them. No page is read from it.
#[derive(Debug, Deserialize)]
struct Told {
    list: ToldList,
    #[serde(default)]
    language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ToldList {
    servings: i64,
    meals: i64,
    sources: String,
}

impl Told {
    /// Whether `settings`, as app followed them, say the same.
    fn is(&self, settings: &Settings) -> bool {
        self.list.servings == settings.servings
            && self.list.meals == settings.meals
            && self.list.sources == settings.sources
            && self.language == settings.language
    }
}

/// Waits until app followed the settings as far as the api has written
/// them. An api that does not say is not waited for: the page shows them a
/// moment later.
async fn settled(cx: &Cx) {
    let language = language(cx);
    let told = api(cx)
        .read::<Told>(asking(cx, &language), &["settings"])
        .await;
    let Ok(told) = told else {
        return;
    };
    let (db, user_id) = (db(cx), account(cx).id.as_str());
    followed(|| async { Ok(told.is(&crate::settings::of(db, user_id).await?)) }).await;
}

/// How a form that was posted went, when it did not send the browser on.
#[derive(Default)]
struct Again {
    /// The part the api refused, and why.
    failure: Option<(Part, Failure)>,
    /// What was typed of the list, and of the name, when it was refused.
    list: Option<ListForm>,
    name: Option<String>,
}

/// Sends the browser back to the settings once `part` is set, saying so;
/// or says what the api refused of it.
fn set(cx: &Cx, part: Part, done: Result<(), CommandError>) -> Result<Failure> {
    match done {
        Ok(()) => {
            let then = href!(show).query([("done", part.word())]).resolve(cx);
            Err(see_other(then).into())
        }
        Err(err) => Failure::of(cx, err),
    }
}

#[page(with(Metadata::new().title(t!(settings_meta_title))))]
pub async fn show(cx: &Cx) -> Result<impl View> {
    Ok(view! { content(again: Again::default()) })
}

/// What the form of the list posts.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ListForm {
    servings: String,
    meals: String,
    sources: String,
}

/// The body of `PUT /v1/settings/list`: a number the api cannot read as
/// one is sent as it was typed, for the api to refuse.
#[derive(Serialize)]
struct ListBody<'a> {
    servings: Option<Number<'a>>,
    meals: Option<Number<'a>>,
    sources: &'a str,
}

#[page(POST "./list", with(Metadata::new().title(t!(settings_meta_title))))]
async fn set_list(cx: &Cx, Form(posted): Form<ListForm>) -> Result<impl View> {
    let language = language(cx);
    let body = ListBody {
        servings: Number::whole(&posted.servings),
        meals: Number::whole(&posted.meals),
        sources: &posted.sources,
    };
    let done = api(cx)
        .command(
            asking(cx, &language),
            Method::PUT,
            &["settings", "list"],
            Some(&body),
        )
        .await;
    if done.is_ok() {
        settled(cx).await;
    }
    let failure = set(cx, Part::List, done)?;

    Ok(view! {
        content(again: Again {
            failure: Some((Part::List, failure)),
            list: Some(posted),
            ..Again::default()
        })
    })
}

/// What the form of the language posts: nothing for the language of the
/// device.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct LanguageForm {
    language: Option<String>,
}

/// The body of `PUT /v1/settings/language`.
#[derive(Serialize)]
struct LanguageBody<'a> {
    language: Option<&'a str>,
}

#[page(POST "./language", with(Metadata::new().title(t!(settings_meta_title))))]
async fn set_language(cx: &Cx, Form(posted): Form<LanguageForm>) -> Result<impl View> {
    let language = language(cx);
    let body = LanguageBody {
        language: posted.language.as_deref().filter(|tag| !tag.is_empty()),
    };
    let done = api(cx)
        .command(
            asking(cx, &language),
            Method::PUT,
            &["settings", "language"],
            Some(&body),
        )
        .await;
    if done.is_ok() {
        // The page the browser is sent to is read in it.
        settled(cx).await;
    }
    let failure = set(cx, Part::Language, done)?;

    Ok(view! {
        content(again: Again {
            failure: Some((Part::Language, failure)),
            ..Again::default()
        })
    })
}

/// What the form of the name posts, and the body of
/// `PUT /v1/account/profile`.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct NameForm {
    name: String,
}

#[page(POST "./name", with(Metadata::new().title(t!(settings_meta_title))))]
async fn set_name(cx: &Cx, Form(posted): Form<NameForm>) -> Result<impl View> {
    let language = language(cx);
    let done = api(cx)
        .command(
            asking(cx, &language),
            Method::PUT,
            &["account", "profile"],
            Some(&posted),
        )
        .await;
    let failure = set(cx, Part::Name, done)?;

    Ok(view! {
        content(again: Again {
            failure: Some((Part::Name, failure)),
            name: Some(posted.name),
            ..Again::default()
        })
    })
}

/// What the form of the password posts, and the body of
/// `PUT /v1/account/password`.
#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct PasswordForm {
    current_password: String,
    password: String,
}

#[page(POST "./password", with(Metadata::new().title(t!(settings_meta_title))))]
async fn set_password(cx: &Cx, Form(posted): Form<PasswordForm>) -> Result<impl View> {
    let language = language(cx);
    let done = api(cx)
        .command(
            asking(cx, &language),
            Method::PUT,
            &["account", "password"],
            Some(&posted),
        )
        .await;
    let failure = set(cx, Part::Password, done)?;

    // Neither password is typed back.
    Ok(view! {
        content(again: Again {
            failure: Some((Part::Password, failure)),
            ..Again::default()
        })
    })
}

/// Ends every other session of who reads the page: this one stays.
#[page(POST "./sign-out-everywhere", with(Metadata::new().title(t!(settings_meta_title))))]
async fn sign_out_everywhere(cx: &Cx) -> Result<impl View> {
    let language = language(cx);
    let done = api(cx)
        .command(
            asking(cx, &language),
            Method::POST,
            &["account", "sign-out-everywhere"],
            None::<&()>,
        )
        .await;
    let failure = set(cx, Part::Sessions, done)?;

    Ok(view! {
        content(again: Again {
            failure: Some((Part::Sessions, failure)),
            ..Again::default()
        })
    })
}

/// A language as its own speakers name it.
fn named(tag: &str) -> &str {
    match tag {
        "en" => "English",
        "fr" => "Français",
        tag => tag,
    }
}

/// The settings and the account, each part with its form: as they stand,
/// or as `again` says the form that was just posted went.
#[component]
async fn content(cx: &Cx, again: Again) -> Result<impl View> {
    let Again {
        failure,
        list,
        name,
    } = again;
    let settings = read(crate::settings::of(db(cx), &account(cx).id).await)?;
    let done = query_params::<Query>(cx)
        .ok()
        .and_then(|query| query.done.clone())
        // What a form that was just refused says replaces it.
        .filter(|_| failure.is_none());
    let done = |part: Part| done.as_deref() == Some(part.word());
    let (list_done, language_done, name_done, password_done, sessions_done) = (
        done(Part::List),
        done(Part::Language),
        done(Part::Name),
        done(Part::Password),
        done(Part::Sessions),
    );

    let refused_part = failure.as_ref().map(|(part, _)| *part);
    let refused = |part: Part, field: &str| {
        let (_, failure) = failure.as_ref().filter(|(refused, _)| *refused == part)?;
        failure.field(field)
    };
    let servings_refused = refused(Part::List, "servings");
    let meals_refused = refused(Part::List, "meals");
    let sources_refused = refused(Part::List, "sources");
    let language_refused = refused(Part::Language, "language");
    let name_refused = refused(Part::Name, "name");
    let current_refused = refused(Part::Password, "current_password");
    let password_refused = refused(Part::Password, "password");
    let mut failure = failure.map(|(_, failure)| failure);
    let mut failure_of = |part: Part| failure.take_if(|_| refused_part == Some(part));
    let (list_failure, language_failure, name_failure, password_failure, sessions_failure) = (
        failure_of(Part::List),
        failure_of(Part::Language),
        failure_of(Part::Name),
        failure_of(Part::Password),
        failure_of(Part::Sessions),
    );

    let list = list.unwrap_or_else(|| ListForm {
        servings: settings.servings.to_string(),
        meals: settings.meals.to_string(),
        sources: settings.sources.clone(),
    });
    let sources = [
        (SOURCES[0], t!(settings_sources_plats_and_recipes)),
        (SOURCES[1], t!(settings_sources_plats)),
        (SOURCES[2], t!(settings_sources_recipes)),
    ]
    .map(|(word, name)| (word, name, list.sources == word));
    // A language app does not speak is one that was not chosen.
    let chosen = settings.language.filter(|tag| spoken(tag).is_some());
    let languages: Vec<(String, bool)> = t::locales()
        .iter()
        .map(|locale| locale.id().to_string())
        .map(|tag| {
            let selected = chosen.as_deref() == Some(tag.as_str());
            (tag, selected)
        })
        .collect();
    let of_the_device = chosen.is_none();
    let name = name.unwrap_or_else(|| account(cx).name.clone());
    let email = account(cx).email.clone();

    let invalid = |refused: &Option<String>| refused.is_some().then_some("true");
    let section = "mt-10 border-t border-stone-200 pt-2";
    let set = "rounded-lg border border-green-300 bg-green-50 px-4 py-3 text-green-900";
    let hint = "mt-1 text-sm text-stone-500";

    Ok(view! {
        <main class=(MAIN)>
            <h1 class=(H1)>(t!(settings_title))</h1>

            <section class=(section) data-part="list">
                <h2 class=(H2)>(t!(settings_list))</h2>
                <p class=(hint)>(t!(settings_list_hint))</p>
                <form class="mt-6 grid gap-5" method="post" action=(href!(set_list))>
                    if list_done {
                        <p class=(set) role="status" data-done="list">(t!(settings_saved))</p>
                    }
                    failed(failure: list_failure)
                    <div class="grid grid-cols-2 gap-3">
                        <div>
                            <label class=(LABEL) for="settings-servings">(t!(settings_servings))</label>
                            <input
                                id="settings-servings"
                                class=(INPUT)
                                type="number"
                                name="servings"
                                value=(list.servings)
                                min="1"
                                inputmode="numeric"
                                aria-invalid=(invalid(&servings_refused))
                                required=""
                            >
                            if let Some(message) = servings_refused {
                                <p class=(FIELD_REFUSED) data-field="servings">(message)</p>
                            }
                        </div>
                        <div>
                            <label class=(LABEL) for="settings-meals">(t!(settings_meals))</label>
                            <input
                                id="settings-meals"
                                class=(INPUT)
                                type="number"
                                name="meals"
                                value=(list.meals)
                                min="1"
                                inputmode="numeric"
                                aria-invalid=(invalid(&meals_refused))
                                required=""
                            >
                            if let Some(message) = meals_refused {
                                <p class=(FIELD_REFUSED) data-field="meals">(message)</p>
                            }
                        </div>
                    </div>
                    <fieldset>
                        <legend class=(LABEL)>(t!(settings_sources))</legend>
                        <div class="mt-2 grid gap-2">
                            for (word, name, checked) in sources {
                                <label class="flex items-center gap-2">
                                    <input
                                        type="radio"
                                        name="sources"
                                        value=(word)
                                        checked=(checked.then_some(""))
                                    >
                                    (name)
                                </label>
                            }
                        </div>
                        if let Some(message) = sources_refused {
                            <p class=(FIELD_REFUSED) data-field="sources">(message)</p>
                        }
                    </fieldset>
                    <div>
                        <button class=(BUTTON) type="submit">(t!(settings_save))</button>
                    </div>
                </form>
            </section>

            <section class=(section) data-part="language">
                <h2 class=(H2)>(t!(settings_language))</h2>
                <p class=(hint)>(t!(settings_language_hint))</p>
                <form class="mt-6 grid gap-5" method="post" action=(href!(set_language))>
                    if language_done {
                        <p class=(set) role="status" data-done="language">(t!(settings_saved))</p>
                    }
                    failed(failure: language_failure)
                    <div>
                        <label class=(LABEL) for="settings-language">(t!(settings_language))</label>
                        <select
                            id="settings-language"
                            class=(INPUT)
                            name="language"
                            aria-invalid=(invalid(&language_refused))
                        >
                            <option value="" selected=(of_the_device.then_some(""))>
                                (t!(settings_language_device))
                            </option>
                            for (tag, selected) in languages {
                                <option
                                    value=(tag.as_str())
                                    lang=(tag.as_str())
                                    selected=(selected.then_some(""))
                                >
                                    (named(&tag).to_owned())
                                </option>
                            }
                        </select>
                        if let Some(message) = language_refused {
                            <p class=(FIELD_REFUSED) data-field="language">(message)</p>
                        }
                    </div>
                    <div>
                        <button class=(BUTTON) type="submit">(t!(settings_save))</button>
                    </div>
                </form>
            </section>

            <section class=(section) data-part="account">
                <h2 class=(H2)>(t!(settings_account))</h2>
                <p class=(hint) data-account=(email.as_str())>
                    (t!(settings_account_email, email: email.as_str()))
                </p>
                <form class="mt-6 grid gap-5" method="post" action=(href!(set_name))>
                    if name_done {
                        <p class=(set) role="status" data-done="name">(t!(settings_saved))</p>
                    }
                    failed(failure: name_failure)
                    <div>
                        <label class=(LABEL) for="settings-name">(t!(settings_name))</label>
                        <input
                            id="settings-name"
                            class=(INPUT)
                            type="text"
                            name="name"
                            value=(name)
                            autocomplete="name"
                            aria-invalid=(invalid(&name_refused))
                            required=""
                        >
                        if let Some(message) = name_refused {
                            <p class=(FIELD_REFUSED) data-field="name">(message)</p>
                        }
                    </div>
                    <div>
                        <button class=(BUTTON) type="submit">(t!(settings_name_save))</button>
                    </div>
                </form>

                <h3 class="mt-10 font-semibold">(t!(settings_password))</h3>
                <p class=(hint)>(t!(settings_password_hint))</p>
                <form class="mt-4 grid gap-5" method="post" action=(href!(set_password))>
                    if password_done {
                        <p class=(set) role="status" data-done="password">
                            (t!(settings_password_changed))
                        </p>
                    }
                    failed(failure: password_failure)
                    <div>
                        <label class=(LABEL) for="settings-current-password">
                            (t!(settings_current_password))
                        </label>
                        <input
                            id="settings-current-password"
                            class=(INPUT)
                            type="password"
                            name="current_password"
                            autocomplete="current-password"
                            aria-invalid=(invalid(&current_refused))
                            required=""
                        >
                        if let Some(message) = current_refused {
                            <p class=(FIELD_REFUSED) data-field="current_password">(message)</p>
                        }
                    </div>
                    <div>
                        <label class=(LABEL) for="settings-new-password">
                            (t!(settings_new_password))
                        </label>
                        <input
                            id="settings-new-password"
                            class=(INPUT)
                            type="password"
                            name="password"
                            autocomplete="new-password"
                            aria-invalid=(invalid(&password_refused))
                            required=""
                        >
                        if let Some(message) = password_refused {
                            <p class=(FIELD_REFUSED) data-field="password">(message)</p>
                        }
                    </div>
                    <div>
                        <button class=(BUTTON) type="submit">(t!(settings_password_save))</button>
                    </div>
                </form>

                <h3 class="mt-10 font-semibold">(t!(settings_sessions))</h3>
                <p class=(hint)>(t!(settings_sessions_hint))</p>
                <form class="mt-4 grid gap-5" method="post" action=(href!(sign_out_everywhere))>
                    if sessions_done {
                        <p class=(set) role="status" data-done="sessions">
                            (t!(settings_sessions_ended))
                        </p>
                    }
                    failed(failure: sessions_failure)
                    <div>
                        <button class=(class!(BUTTON_QUIET, "text-sm")) type="submit">
                            (t!(settings_sessions_end))
                        </button>
                    </div>
                </form>
            </section>
        </main>
    })
}
