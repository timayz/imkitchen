# imkitchen admin

The administration of imkitchen, built with
[topcoat](https://github.com/tokio-rs/topcoat).

admin owns no aggregate and no command. What it shows of imkitchen it derives
from the api's events, which it follows as an evento sidecar into a store of
its own.

Nothing asks who reads its pages yet (#766): until then it shows a count and
nothing of anyone, and is not deployed.

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
| `src/app.rs`, `src/app/` | The pages: the router, the layout, each page at the address of its module. |
| `src/events.rs` | The connection to the api's events. |
| `src/users.rs` | A read model: its subscription and what the pages read from it. |
| `src/subscriptions.rs` | Every subscription admin runs. |
| `src/db.rs` | The store and the migrations of its tables. |
| `styles.css`, `build.rs` | The Tailwind stylesheet, compiled at build time. |

## Pages

admin is read by the people who run imkitchen, in English: the sentences are
in the pages, and no search engine is to list any of them (`noindex`).

Classes are Tailwind's, found in the `.rs` files when the crate is built.

## Read models

admin reads the aggregates `imkitchen-aggregates` lists for it
(`Sidecar::Admin`); the api refuses it anything else, and every write. Today
that is the users, of whom admin keeps how many there are.

To show something else of imkitchen:

1. Add the aggregate to `Sidecar::Admin` in `imkitchen-aggregates`.
2. Give its table a migration in `db::MIGRATIONS`.
3. Write its subscription, whose handlers leave the table as it should be
   when an event is delivered twice, and start it in `subscriptions::start`.
