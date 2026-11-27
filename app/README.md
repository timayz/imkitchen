# imkitchen app

The imkitchen application in a browser, on a desktop as on a phone, built
with [topcoat](https://github.com/tokio-rs/topcoat).

app owns no aggregate and no command. Its pages are read from read models of
its own, never from the api: it derives them from the api's events, which it
follows as an evento sidecar into a store of its own.

It owns no account either, and writes nothing itself: a user registers and
signs in at the api, through app, and what they do is a command app sends to
the api over HTTP.

## Running it

```sh
make dev        # the api, at http://localhost:3000, its events on 4000
make app.dev    # app, at http://localhost:3030, in another terminal
```

`make app.dev` creates app's store (`data/app.db`), then builds, serves,
rebuilds on a change and reloads the page (`topcoat dev`). The two share a
token of the Makefile's, for local use only. app starts whether the api runs
or not, and catches up with its events once the api answers:
`GET /_app/health` says how it stands with them (`"api": "connected"`).
Nobody gets past signing in while the api is away.

<http://localhost:3030> sends a browser that is not signed in to
`/auth/sign-in`. Root (`auth.root` of the api) signs in with its password
alone; anyone else registers at `/auth/register` and types the code the api
mails (`[mail]` of the api: locally the maildev of `compose.yml`).

With `make up` (Traefik, after `make cert` once) app is at
<https://imkitchen.localhost>, the main domain, which it shares with www
(`make www.dev`): a visitor is shown www there, and app once signed in
(<https://imkitchen.localhost/auth/sign-in>). The api is under its `/api`,
and www whoever asks at <https://about.imkitchen.localhost>. The Makefile
gives app and www these addresses to link to each other at.

`make app.run` runs it once, without the reload. The stylesheet is an asset
bundled next to the binary (`topcoat asset bundle -p imkitchen-app`): a plain
`cargo run -p imkitchen-app` refuses to start without the bundle.

The `topcoat` command comes from `topcoat-cli`, installed by the Nix shell at
the version of the workspace's `topcoat`.

## Configuration

`config/default.toml` documents every key. A file given with `--config` (or
`IMKITCHEN_APP_CONFIG`) overrides the defaults, and
`IMKITCHEN_APP__<SECTION>__<KEY>` environment variables override both.
`app/config/dev.local.toml`, ignored by git, is picked up by the Makefile.

What a deployment sets:

- `server.url`: the address users reach app at, the main domain of
  imkitchen. Over `https` the cookies of a session are `Secure`.
- `www.url`: the address that always serves www (`about.`), where the pages
  link to the public ones.
- `api.url`: where the api answers HTTP.
- `api.public_url`: where the browsers reach the api, which the pictures of
  the recipes are shown from: `/api` of the main domain.
- `session.key`: what the cookie of a session is encrypted under, the same
  for every replica. Without one a key is drawn at each start, and whoever
  was signed in signs in again.
- `api.events`: the `[sidecars]` listener of the api (evento's remote
  protocol over TCP, not HTTP). One node, or a name in front of several.
- `api.token`: `sidecars.tokens.app` of the api.
- `api.tls_ca`: the certificate the listener's is checked against, when the
  api gives its listener one. Without it the token travels in clear text.
- `database.path`: the store. Each replica of app has its own, and a store
  that is lost is built again from the api's events.

`imkitchen-app migrate` creates the store and applies its pending
migrations; `serve` refuses a store that is not there or not migrated.

## Deploying it

`app/Dockerfile` builds the image (`timayz/imkitchen:app-<version>`): the
binary and, next to it, its assets. `app/helm` is its chart, whose README
says what it runs and what to set: a StatefulSet, each pod keeping its
store on a volume of its own.

```sh
docker build -f app/builder.Dockerfile -t timayz/imkitchen:app-builder .   # once
docker build -f app/Dockerfile -t timayz/imkitchen:app-local .
```

The builder image holds the two tools the build needs besides cargo:
`topcoat-cli`, at the version of `topcoat` in `Cargo.lock`, and the Tailwind
CLI, pinned with its checksum (`TAILWIND_VERSION` in
`app/builder.Dockerfile`, the release topcoat pins): the one to move by hand
when topcoat moves its own.

## Where things are

| Path | Contents |
| --- | --- |
| `src/app.rs` | The router, the language of a page and the document every page is in. |
| `src/app/_signed_in.rs`, `src/app/_signed_in/` | What a signed-in user reads: who is let through, the layout and its navigation, the home, each screen at the address of its module. |
| `src/app/_signed_in/recipes.rs`, `src/app/_signed_in/recipes/` | The recipes: the list, a recipe, the form one is written and rewritten with, and the import. |
| `src/app/_signed_in/dishes.rs`, `src/app/_signed_in/dishes/` | The dishes: the list, a dish, and the form one is composed and recomposed with. |
| `src/app/auth.rs`, `src/app/auth/` | What a visitor is shown, under `/auth`: signing in, registering, and signing out. |
| `src/app/_app.rs`, `src/app/_app/` | What is not a page, under `/_app`: the health. |
| `src/assets.rs` | The assets, served under `/_app/assets`. |
| `src/app/l10n/` | The sentences of the pages, one Fluent file per language. |
| `src/api.rs` | The api over HTTP: registering, signing in and out, who a session is, and the commands. |
| `src/session.rs` | What a browser keeps of its session: the two cookies. |
| `src/events.rs` | The connection to the api's events. |
| `src/recipes.rs`, `src/dishes.rs` | The read models: the subscription of each and what the pages read from it. |
| `src/subscriptions.rs` | Every subscription app runs. |
| `src/db.rs` | The store and the migrations of its tables. |
| `Dockerfile`, `builder.Dockerfile`, `helm/` | The image and the chart app is deployed with. |
| `styles.css`, `build.rs` | The Tailwind stylesheet, compiled at build time. |

## One address with www

The main domain of imkitchen is www for a visitor and app for a signed-in
user. The gateway in front of the two chooses (Traefik locally,
`.docker/traefik/dynamic.yaml`; Envoy Gateway deployed, the `HTTPRoute` of
each chart), on what the browser sends:

| Request to the main domain | Goes to |
| --- | --- |
| `/api/…` | the api, as its `/…` |
| `/auth/…`, `/_app/…` | app, whoever asks |
| anything else, with the `signed_in` cookie among those sent | app |
| anything else | www |

`about.` of the main domain is www whoever asks: it is where app links to
the public pages (`www.url`), the terms and the privacy page under the forms
of `/auth` and "About imkitchen" beside the navigation.

The gateway only sees that the marker is there, not that the session is one:
app tells, and takes the marker back when it is not (see below), which
brings www back.

What app serves a browser without the marker is under the two prefixes:
the pages of a visitor under `/auth`, and under `/_app` what is not a page.
That is its health (`/_app/health`; `/health` of the main domain is www's)
and its assets: topcoat serves those of any application under
`/_topcoat/assets`, where a visitor reading `/auth/sign-in` would ask www
for app's stylesheet, so app serves its own under `/_app/assets`
(`src/assets.rs`). A page or a route added for visitors goes under one of
the two, or the gateway hands it to www.

## Screens and layout

`/` is the home. Each screen of the application has a module under
`src/app/_signed_in/` and is at its address: `/recipes`, `/dishes`, `/list`,
`/groceries`, `/settings`. Nothing is shown on them yet but the recipes and
the dishes: the others are there for the navigation to lead to.

One layout (`signed_in` in `src/app/_signed_in.rs`) serves the desktop and
the phone. The navigation between the screens is a bar that stays at the
bottom of a phone, and from Tailwind's `md` up a column that stays beside
the page. It is the same links either way, and the screen being read says
so (`aria-current`). Signing out is beside it.

A screen added under `src/app/_signed_in/` is also added to `screens` in
`src/app/_signed_in.rs`, which is what the navigation lists.

Classes are Tailwind's, found in the `.rs` files when the crate is built.

## Languages

A page is read in the language the browser prefers (`Accept-Language`) among
those app speaks, English otherwise: its address is the same in each. A page
names a sentence by its key, `t!(home_title)`. A language is spoken once it
has a file in `src/app/l10n/`. `make l10n` tells a key a language lacks, a
sentence no page uses and an argument a page does not give.

No search engine is to list a page (`noindex, nofollow`).

## Signing in

Signing in is the api's (`api/README.md`): app hands it what the forms of
`/auth/sign-in` and `/auth/register` post and keeps the token it answers.
Both take an address and a password, then the code the api mails to the
address.

A signed-in browser keeps two cookies, for as long as the api says the
session lasts, closed or not:

- `session`: the token, encrypted under `session.key`. No script reads it.
- `signed_in`: the marker. It says a session is there and nothing else, and
  opens nothing by itself. It is what the gateway in front of imkitchen
  routes the main domain on: to app when the browser sends it, to www when
  it does not.

The two are written together and taken back together. Every page a user
reads is declared under `src/app/_signed_in/`, which asks the api whose
session the browser keeps on each request (`signed_in_only`):

- A browser that keeps nothing is sent to `/auth/sign-in`. Behind the
  gateway this does not happen: without the marker the main domain is www.
- A session the api no longer knows (signed out elsewhere, expired,
  suspended), or a marker without a session, has both cookies taken back
  and the browser sent to `/`: the main domain, which is then www.
- While the api does not answer, the page is a `503` and the cookies stay.

`POST /auth/sign-out` ends the session at the api, takes back both cookies
and sends the browser to `/` the same way.

Everything a visitor who is not signed in is shown is under `/auth`: the
one prefix the gateway routes to app whether the marker is there or not.

## Commands

app writes nothing: what a user does is a command of the api's, sent with
the token of their session (`Api::command` in `src/api.rs`): the method, the
address under `/v1` and the JSON the command takes.

```rust
let asking = Asking { language: &language(cx), token: Some(&token) };
api(cx).command(asking, Method::PUT, &["recipes", id], Some(&recipe)).await
```

What the api refuses comes back as it worded it (`Refusal`), in the language
the page is read in (app asks for it with `Accept-Language`): `message` for
the whole, and under `validation_failed` one message per field
(`refusal.field("title")`), for the form to show beside the field. A `401`
(`refusal.signed_out()`) is a session that ended. An api that did not answer
is not a refusal (`CommandError::Unreachable`): whether it did what was
asked is not known.

A page that posts a command (`src/app/_signed_in.rs`) sends it with the
token of who reads it (`asking`), says what failed above its form
(`Failure`, `failed`) and, once the api did it, waits for app to have
followed what was written before it sends the browser on (`followed`): the
next page is read from the store, a moment behind the api. Not for long:
past five seconds the browser is sent on anyway.

## Recipes

| Address | What it is |
| --- | --- |
| `/recipes` | The recipes of who reads it, the most recent first, 24 at a time; `q` searches their titles and their ingredients, whatever the case and the accents. |
| `/recipes/new` | The form a recipe is written with. |
| `/recipes/import` | Where the address of a page is pasted: the api fetches it and makes a recipe of it. |
| `/recipes/{id}` | A recipe, and the button that deletes it. |
| `/recipes/{id}/edit` | The same form, to rewrite it. |

A recipe is its owner's alone: someone else's is one that does not exist,
read or written. One that was imported says nothing of where it came from,
and is read, rewritten and deleted as one that was written.

The form works without a script. An ingredient and a step are a row each,
the rows left blank are left out, and "More rows" posts the form to get it
back longer. It posts as `multipart/form-data`, the picture with it: the
recipe is written first (`POST /v1/recipes`, `PUT /v1/recipes/{id}`), then
its picture (`PUT /v1/recipes/{id}/image`). A picture the api refuses leaves
the recipe written without it, and the form says both. app checks nothing of
what is typed: what the api refuses is said under the field, or under the
row.

A picture is shown from the api, where the browsers reach it
(`api.public_url`): app keeps the name of its files and stores none.

## Dishes

A dish is what the api calls a plat: a name, and recipes of the user in the
order they are served.

| Address | What it is |
| --- | --- |
| `/dishes` | The dishes of who reads it, the most recent first, 24 at a time; `q` searches their names, whatever the case and the accents. |
| `/dishes/new` | The form a dish is composed with. |
| `/dishes/{id}` | A dish: its recipes, each leading to the recipe, and the button that deletes it. |
| `/dishes/{id}/edit` | The same form, to recompose it. |

Like a recipe, a dish is its owner's alone. It names its recipes and keeps
nothing of them: each is shown as it now stands. A recipe that is deleted
leaves the dishes it was part of, and a dish left with none stays, to be
recomposed or deleted. Deleting a dish deletes none of its recipes.

The form works without a script. It shows the recipes of the dish, a ticked
box each, then the user's other recipes: the 24 most recent, or those a
search finds (by title or ingredient). Searching, and moving a recipe up or
down, post the form to get it back as it was left: what was ticked among the
others has joined the dish, after its recipes, and what was unticked has
left it. The Enter key in a field searches. Nothing is written until the
form is saved (`POST /v1/plats`, `PUT /v1/plats/{id}`), and what the api
refuses is said under the name or under the recipes.

## Read models

app reads the aggregates `imkitchen-aggregates` lists for it
(`Sidecar::App`); the api refuses it anything else, and every write. Today
that is the recipes, of which app keeps all each one says, whose it is and
the picture it shows (`src/recipes.rs`), and the plats, its dishes: the name
of each, whose it is and the recipes it names (`src/dishes.rs`).

To show something else of imkitchen:

1. Add the aggregate to `Sidecar::App` in `imkitchen-aggregates`.
2. Give its table a migration in `db::MIGRATIONS`.
3. Write its subscription, whose handlers leave the table as it should be
   when an event is delivered twice, and start it in `subscriptions::start`.
