# imkitchen admin

The administration of imkitchen, built with
[topcoat](https://github.com/tokio-rs/topcoat).

admin owns no aggregate and no command. What it shows of imkitchen it derives
from the api's events, which it follows as an evento sidecar into a store of
its own. What an administrator does there is a command of the api's, sent
over HTTP: admin writes nothing except through the api.

Only an administrator reads it: every page asks the api who its reader is,
and sends anyone else to sign in. It is not deployed yet.

## Running it

```sh
make dev          # the api, at http://localhost:3000, its events on 4000
make admin.dev    # admin, at http://localhost:3020, in another terminal
```

`make admin.dev` creates admin's store (`data/admin.db`), then builds,
serves, rebuilds on a change and reloads the page (`topcoat dev`). The two
share a token of the Makefile's, for local use only. admin starts whether the
api runs or not: it serves what its store holds and catches up once the api
answers. `GET /health` says how it stands with it (`"api": "connected"`).

Sign in at <http://localhost:3020/sign-in> with an administrator's address
and password. Locally that is the root account of the api, which needs no
mailed code: give the api one in `api/config/dev.local.toml` (`[auth.root]`,
`api/config/default.toml` says how).

With `make up` (Traefik, after `make cert` once) admin is also at
<https://admin.imkitchen.localhost>.

`make admin.run` runs it once, without the reload. The stylesheet is an asset
bundled next to the binary (`topcoat asset bundle -p imkitchen-admin`): a
plain `cargo run -p imkitchen-admin` refuses to start without the bundle.

The `topcoat` command comes from `topcoat-cli`, installed by the Nix shell at
the version of the workspace's `topcoat`.

## Configuration

`config/default.toml` documents every key. A file given with `--config` (or
`IMKITCHEN_ADMIN_CONFIG`) overrides the defaults, and
`IMKITCHEN_ADMIN__<SECTION>__<KEY>` environment variables override both.
`admin/config/dev.local.toml`, ignored by git, is picked up by the Makefile.

- `server.url`: the address administrators reach admin at. Over `https`
  the cookie of a session is `Secure`.
- `session.key`: what the cookie of a session is encrypted under, 32 bytes
  or more, the same for every replica. Without it admin draws one when it
  starts, and whoever was signed in signs in again.
- `api.url`: where the api answers HTTP, for signing in and out, for asking
  who reads a page and for the commands of the administrators.
- `api.public_url`: where the browsers reach the api, for the pictures of
  the recipes. Without it admin shows none.
- `api.events`: the `[sidecars]` listener of the api (evento's remote
  protocol over TCP, not HTTP). One node, or a name in front of several.
- `api.token`: `sidecars.tokens.admin` of the api.
- `api.tls_ca`: the certificate the listener's is checked against, when the
  api gives its listener one. Without it the token travels in clear text.
- `database.path`: the store. Each replica of admin has its own, and a store
  that is lost is built again from the api's events.

`imkitchen-admin migrate` creates the store and applies its pending
migrations; `serve` refuses a store that is not there or not migrated.

## Where things are

| Path | Contents |
| --- | --- |
| `src/app.rs`, `src/app/` | The pages: the router, the layout, signing in and out. |
| `src/app/_admin.rs`, `src/app/_admin/` | What an administrator reads: the guard, the dashboard, and each screen at the address of its module. |
| `src/api.rs` | The client of the api: signing in and out, who a session is, and the commands of the administrators. |
| `src/session.rs` | The cookie a browser keeps its session in. |
| `src/events.rs` | The connection to the api's events. |
| `src/users.rs`, `recipes.rs`, `imports.rs`, `images.rs`, `messages.rs` | The read models: each its subscription and what the pages read from it. |
| `src/listing.rs` | How the lists are read: a page at a time, and searched. |
| `src/subscriptions.rs` | Every subscription admin runs. |
| `src/db.rs` | The store and the migrations of its tables. |
| `styles.css`, `build.rs` | The Tailwind stylesheet, compiled at build time. |

## Pages

admin is read by the people who run imkitchen, in English: the sentences are
in the pages, and no search engine is to list any of them (`noindex`).

A page is declared in `src/app/_admin.rs` or in a module below it: that is
what puts it behind the guard, and the module adds nothing to its address.
Only signing in, signing out and `/health` are declared outside.

Classes are Tailwind's, found in the `.rs` files when the crate is built.

## Who reads admin

Identity is the api's (`timada-identity`): admin holds no account, no
password and no session.

- Signing in posts the address and the password to `POST /v1/auth/sign-in`.
  The api mails a code to the address, and the same again with the code
  opens a session (root needs none). The password is typed at both steps: it
  is never written into a page or kept anywhere. Someone the api signs in
  who is not an administrator is signed out at once.
- The browser keeps the token of the session in a private (encrypted,
  authenticated) cookie, `HttpOnly` and `SameSite=Lax`, for as long as the
  api says the session lasts.
- Every page asks `GET /v1/account` whose token it is. A session that ended
  sends the browser to sign in; one whose user is an administrator no more
  is ended and forgotten; while the api does not answer, nobody is let
  through (503).
- Signing out ends the session at the api and forgets the cookie.

## Screens

| Address | What it shows |
| --- | --- |
| `/` | The key numbers: users, administrators, recipes, imports and those that failed, sources, pictures and those that are shared, messages. Nothing of who visits. |
| `/users`, `/users/{id}` | The users, searched by address or name; one of them, their recipes and their sessions, whether they are suspended and by whom. |
| `/recipes`, `/recipes/{id}` | The recipes, searched by title or ingredient; one of them with all it says, its owner, its picture and the page it was imported from. |
| `/imports`, `/imports/sources` | Each try at importing a recipe (`?failed=1` those that gave none, `?host=` those of a site); the sites they are made from. |
| `/images`, `/images/{id}` | The pictures that are used (`?shared=1` those several recipes show); one of them and the recipes that show it. |
| `/messages`, `/messages/{id}` | What was written to imkitchen; one message, as it was written. |

A list shows 50 rows a page (`?page=`), and `?q=` searches it whatever the
case and the accents. With `api.public_url` the pictures are shown, from the
api, which asks no token for them; without it the screens name them.

## Actions

What an administrator does is a button of a screen, which posts to admin;
admin hands it to the api as a command of `/v1/admin`, with the token of who
pressed it. The api decides whether they may, and writes; admin shows it
once it followed the events.

| Screen | Button | Posts to | Command of the api |
| --- | --- | --- | --- |
| `/users/{id}` | Suspend | `/users/{id}/suspend` | `PUT /v1/admin/users/{id}/suspension` |
| `/users/{id}` | Reactivate | `/users/{id}/reactivate` | `DELETE /v1/admin/users/{id}/suspension` |
| `/users/{id}` | Delete the account | `/users/{id}/delete` | `DELETE /v1/admin/users/{id}` |
| `/recipes/{id}` | Delete the recipe | `/recipes/{id}/delete` | `DELETE /v1/admin/recipes/{id}` |
| `/images/{id}` | Take it off the recipes | `/images/{id}/remove` | `DELETE /v1/admin/images/{id}` |

- What is not undone (deleting, taking a picture off) is behind a
  disclosure that says what it does; there is no script on the pages.
- Once the api did it, the browser is sent on to the screen, which says
  what was done. The read models are a second or two behind the api: the
  page waits for them, five seconds at most.
- What the api refuses is said in its words, with its status, and nothing
  is done: an administrator does not suspend or delete another one (root
  does), nobody root or themself. The buttons of an account are not shown
  to its own reader, nor on root.
- A session that ended meanwhile is sent to sign in. When the api does not
  answer, the page says that it may have been done all the same.

To add one: a method of `Api` in `src/api.rs` over `command`, and a
`#[page(POST ...)]` next to the screen that calls it and answers with `done`
(`src/app/_admin.rs`). Its command is the api's to have first.

## Read models

Every screen is read from the store of admin and from nothing else: no page
asks the api for what it shows. admin reads the aggregates
`imkitchen-aggregates` lists for it (`Sidecar::Admin`); the api's listener
refuses it anything else, and every write.

| Read model | Tables | Follows |
| --- | --- | --- |
| `users` | `users`, `user_sessions`, `user_suspensions` | `timada-identity/User`, `timada-identity/Session`, `imkitchen/Suspension` |
| `recipes` | `recipes` | `imkitchen/Recipe` |
| `imports` | `recipe_imports` | `imkitchen/RecipeImport` |
| `images` | `images`, `image_uses` | `imkitchen/Image` |
| `messages` | `contact_messages` | `imkitchen/ContactMessage` |

Each has a subscription and a cursor of its own, so one that fails holds no
other up. A store that is lost, or one given a new path, is built again from
the first event of the api: delete `data/admin.db`, run `migrate`, serve.

To show something else of imkitchen:

1. Add the aggregate to `Sidecar::Admin` in `imkitchen-aggregates`.
2. Give its table a migration in `db::MIGRATIONS`.
3. Write its subscription, whose handlers leave the table as it should be
   when an event is delivered twice, and start it in `subscriptions::start`.
4. Give it a screen under `src/app/_admin/`.
