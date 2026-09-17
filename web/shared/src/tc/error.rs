use topcoat::context::Cx;

use super::context::i18n;
use crate::template::{FORBIDDEN, SERVER_ERROR_MESSAGE};

/// Translated toast text for a domain error. Same mapping as the askama
/// `try_response!` macro: server errors are logged and replaced by a generic
/// message, everything else is shown as is.
pub fn toast_message(cx: &Cx, error: &imkitchen_core::Error) -> String {
    let message = match error {
        imkitchen_core::Error::Server(err) => {
            tracing::error!("{err}");
            SERVER_ERROR_MESSAGE.to_owned()
        }
        imkitchen_core::Error::Forbidden(_) => FORBIDDEN.to_owned(),
        err => err.to_string(),
    };

    i18n(cx).t(&message)
}

/// [`toast_message`] for errors that are always server errors (`anyhow`).
pub fn server_toast_message(cx: &Cx, error: &anyhow::Error) -> String {
    tracing::error!("{error}");
    i18n(cx).t(SERVER_ERROR_MESSAGE)
}

/// Outcome of a procedure the browser can branch on. A procedure `Err` is not
/// observable client-side, so domain failures travel as the `Ok` value:
/// `Ok(value)` on success, `Err(translated toast text)` on failure.
pub type Outcome<T> = Result<T, String>;

pub fn toastify<T>(cx: &Cx, result: imkitchen_core::Result<T>) -> Outcome<T> {
    result.map_err(|err| toast_message(cx, &err))
}
