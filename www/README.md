# imkitchen www

The public and landing pages of imkitchen, built with
[topcoat](https://github.com/tokio-rs/topcoat).

www owns no aggregate and no command. What it shows of imkitchen it derives
from the api's events, which it follows as an evento sidecar into a store of
its own; what it writes, it sends to the api as a command over HTTP.

## Running it

```sh
make dev        # the api, at http://localhost:3000, its events on 4000
make www.dev    # www, at http://localhost:3010, in another terminal
```

`make www.dev` creates www's store (`data/www.db`), then builds, serves,
rebuilds on a change and reloads the page (`topcoat dev`). The two share a
token of the Makefile's, for local use only. www starts whether the api runs
or not: it serves what its store holds and catches up once the api answers.
`GET /health` says how it stands with it (`"api": "connected"`).

With `make up` (Traefik, after `make cert` once) www is also at
<https://imkitchen.localhost>, and the api at
<https://app.imkitchen.localhost/api>.

`make www.run` runs it once, without the reload. The stylesheet is an asset
bundled next to the binary (`topcoat asset bundle -p imkitchen-www`): a plain
`cargo run -p imkitchen-www` refuses to start without the bundle.

The `topcoat` command comes from `topcoat-cli`, installed by the Nix shell at
the version of the workspace's `topcoat`.

## Configuration

`config/default.toml` documents every key. A file given with `--config` (or
`IMKITCHEN_WWW_CONFIG`) overrides the defaults, and
`IMKITCHEN_WWW__<SECTION>__<KEY>` environment variables override both.
`www/config/dev.local.toml`, ignored by git, is picked up by the Makefile.

What a deployment sets:

- `api.events`: the `[sidecars]` listener of the api (evento's remote
  protocol over TCP, not HTTP). One node, or a name in front of several.
- `api.token`: `sidecars.tokens.www` of the api.
- `api.tls_ca`: the certificate the listener's is checked against, when the
  api gives its listener one. Without it the token travels in clear text.
- `database.path`: the store. Each replica of www has its own, and a store
  that is lost is built again from the api's events.

`imkitchen-www migrate` creates the store and applies its pending
migrations; `serve` refuses a store that is not there or not migrated.

## Where things are

| Path | Contents |
| --- | --- |
| `src/app.rs`, `src/app/` | The pages: the router, the layout, each page at the address of its module. |
| `src/app/l10n/` | The sentences of the pages, one Fluent file per language. |
| `src/events.rs` | The connection to the api's events. |
| `src/recipes.rs` | A read model: its subscription and what the pages read from it. |
| `src/subscriptions.rs` | Every subscription www runs. |
| `src/db.rs` | The store and the migrations of its tables. |
| `styles.css`, `build.rs` | The Tailwind stylesheet, compiled at build time. |

## Pages and languages

Every page is under its language, the first part of its address (`/en`,
`/fr`): a module under `src/app/language/` is a page of every language. `/`
sends a visitor to the home of the language their browser prefers.

A page names a sentence by its key, `t!(home_title)`, and the sentence is
read in the language of the address. A language is spoken once it has a file
in `src/app/l10n/`. `make l10n` tells a key a language lacks, a sentence no
page uses and an argument a page does not give.

Classes are Tailwind's, found in the `.rs` files when the crate is built.

## Read models

www reads the aggregates `imkitchen-aggregates` lists for it
(`Sidecar::Www`); the api refuses it anything else, and every write. Today
that is the recipes, of which www keeps how many there are and nothing they
say.

To show something else of imkitchen:

1. Add the aggregate to `Sidecar::Www` in `imkitchen-aggregates`.
2. Give its table a migration in `db::MIGRATIONS`.
3. Write its subscription, whose handlers leave the table as it should be
   when an event is delivered twice, and start it in `subscriptions::start`.
