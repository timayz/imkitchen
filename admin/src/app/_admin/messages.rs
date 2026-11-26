//! `/messages`: what was written to imkitchen, the most recent first, and
//! `/messages/{message_id}`: one message, as it was written.

use topcoat::{
    Result,
    context::Cx,
    router::{error::not_found, href, page, path_param, query_params},
    view::{View, view},
};

use super::{H1, LINK, MAIN, TABLE, TD, TH, pager, read, when};
use crate::app::db;
use crate::listing::{PAGE_SIZE, Pages};

/// How much of a message the list shows, in characters.
const EXCERPT: usize = 120;

path_param!(message_id);

#[query_params]
struct Query {
    page: Option<u32>,
}

/// The start of `message`, on one line.
fn excerpt(message: &str) -> String {
    let line = message.split_whitespace().collect::<Vec<_>>().join(" ");
    match line.char_indices().nth(EXCERPT) {
        Some((end, _)) => format!("{}…", &line[..end]),
        None => line,
    }
}

#[page]
pub async fn list(cx: &Cx) -> Result<impl View> {
    let db = db(cx);
    let query = query_params::<Query>(cx).ok();
    let pages = Pages::new(
        query.and_then(|query| query.page),
        read(crate::messages::count(db).await)?,
    );
    let messages = read(crate::messages::list(db, pages.offset(), PAGE_SIZE).await)?;
    let at = |page: u32| href!(list).query([("page", page)]).resolve(cx);
    let (previous, next) = (pages.previous().map(at), pages.next().map(at));

    Ok(view! {
        <main class=(MAIN)>
            <h1 class=(H1)>"Messages"</h1>
            <table class=(TABLE)>
                <thead>
                    <tr>
                        <th class=(TH)>"When"</th>
                        <th class=(TH)>"From"</th>
                        <th class=(TH)>"Message"</th>
                    </tr>
                </thead>
                <tbody>
                    for message in &messages {
                        <tr data-message=(message.id.as_str())>
                            <td class=(TD)>(when(message.sent_at))</td>
                            <td class=(TD)>
                                (message.name.as_str())
                                <span class="block text-stone-500">(message.email.as_str())</span>
                            </td>
                            <td class=(TD)>
                                <a class=(LINK) href=(href!(show, message_id: message.id.as_str()))>
                                    (excerpt(&message.message))
                                </a>
                            </td>
                        </tr>
                    }
                </tbody>
            </table>
            if messages.is_empty() {
                <p class="mt-6 text-stone-600" data-empty="">"Nobody wrote yet."</p>
            }
            pager(pages: pages, previous: previous, next: next)
        </main>
    })
}

#[page("./{message_id}")]
pub async fn show(cx: &Cx) -> Result<impl View> {
    let Some(message) = read(crate::messages::get(db(cx), message_id(cx)).await)? else {
        return Err(not_found().into());
    };

    Ok(view! {
        <main class=(MAIN) data-message=(message.id.as_str())>
            <p class="text-sm"><a class=(LINK) href=(href!(list))>"Messages"</a></p>
            <h1 class=(H1)>"Message from " (message.name.as_str())</h1>
            <dl class="mt-6 grid gap-x-8 gap-y-3 text-sm sm:grid-cols-2">
                <div>
                    <dt class="text-stone-500">"Answer at"</dt>
                    <dd>
                        <a class=(LINK) href=(format!("mailto:{}", message.email))>
                            (message.email.as_str())
                        </a>
                    </dd>
                </div>
                <div>
                    <dt class="text-stone-500">"Reads"</dt>
                    <dd data-language=(message.language.as_str())>(message.language.as_str())</dd>
                </div>
                <div><dt class="text-stone-500">"Written"</dt><dd>(when(message.sent_at))</dd></div>
            </dl>
            <p class="mt-6 rounded-lg border border-stone-200 bg-white p-6 whitespace-pre-line" data-body="">
                (message.message.as_str())
            </p>
        </main>
    })
}
