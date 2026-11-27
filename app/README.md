# imkitchen app

The imkitchen application in a browser, on a desktop as on a phone, built
with [topcoat](https://github.com/tokio-rs/topcoat).

app owns no aggregate and no command. Its pages are read from read models of
its own, never from the api: it derives them from the api's events, which it
follows as an evento sidecar into a store of its own.

## Running it

```sh
make dev        # the api, at http://localhost:3000, its events on 4000
make app.dev    # app, at http://localhost:3030, in another terminal
```

`make app.dev` creates app's store (`data/app.db`), then builds, serves,
rebuilds on a change and reloads the page (`topcoat dev`). The two share a
token of the Makefile's, for local use only. app starts whether the api runs
or not: it serves what its store holds and catches up once the api answers.
`GET /health` says how it stands with it (`"api": "connected"`).

With `make up` (Traefik, after `make cert` once) app is also at
<https://app.imkitchen.localhost>, the api under its `/api`.

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
| `src/app.rs`, `src/app/` | The pages: the router, the layout and its navigation, the home, each screen at the address of its module. |
| `src/app/l10n/` | The sentences of the pages, one Fluent file per language. |
| `src/events.rs` | The connection to the api's events. |
| `src/recipes.rs` | A read model: its subscription and what the pages read from it. |
| `src/subscriptions.rs` | Every subscription app runs. |
| `src/db.rs` | The store and the migrations of its tables. |
| `Dockerfile`, `builder.Dockerfile`, `helm/` | The image and the chart app is deployed with. |
| `styles.css`, `build.rs` | The Tailwind stylesheet, compiled at build time. |

## Screens and layout

`/` is the home. Each screen of the application has a module under
`src/app/` and is at its address: `/recipes`, `/dishes`, `/list`,
`/groceries`, `/settings`. Nothing is shown on them yet: they are there for
the navigation to lead to.

One layout (`root` in `src/app.rs`) serves the desktop and the phone. The
navigation between the screens is a bar that stays at the bottom of a phone,
and from Tailwind's `md` up a column that stays beside the page. It is the
same links either way, and the screen being read says so (`aria-current`).

A screen added under `src/app/` is also added to `screens` in `src/app.rs`,
which is what the navigation lists.

Classes are Tailwind's, found in the `.rs` files when the crate is built.

## Languages

A page is read in the language the browser prefers (`Accept-Language`) among
those app speaks, English otherwise: its address is the same in each. A page
names a sentence by its key, `t!(home_title)`. A language is spoken once it
has a file in `src/app/l10n/`. `make l10n` tells a key a language lacks, a
sentence no page uses and an argument a page does not give.

No search engine is to list a page (`noindex, nofollow`).

## Read models

app reads the aggregates `imkitchen-aggregates` lists for it
(`Sidecar::App`); the api refuses it anything else, and every write. Today
that is the recipes, of which app keeps how many there are and nothing they
say: nobody signs in yet, so no page knows whose a recipe is.

To show something else of imkitchen:

1. Add the aggregate to `Sidecar::App` in `imkitchen-aggregates`.
2. Give its table a migration in `db::MIGRATIONS`.
3. Write its subscription, whose handlers leave the table as it should be
   when an event is delivered twice, and start it in `subscriptions::start`.
