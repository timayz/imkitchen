# imkitchen admin

The administration of imkitchen, built with
[topcoat](https://github.com/tokio-rs/topcoat).

admin owns no aggregate and no command. What it shows of imkitchen it derives
from the api's events, which it follows as an evento sidecar into a store of
its own.

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
- `api.url`: where the api answers HTTP, for signing in and out and for
  asking who reads a page.
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
| `src/app/_admin.rs` | What an administrator reads: the guard, and each page at the address of its module. |
| `src/api.rs` | The client of the api: signing in and out, and who a session is. |
| `src/session.rs` | The cookie a browser keeps its session in. |
| `src/events.rs` | The connection to the api's events. |
| `src/users.rs` | A read model: its subscription and what the pages read from it. |
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

## Read models

admin reads the aggregates `imkitchen-aggregates` lists for it
(`Sidecar::Admin`); the api refuses it anything else, and every write. Today
that is the users, of whom admin keeps how many there are.

To show something else of imkitchen:

1. Add the aggregate to `Sidecar::Admin` in `imkitchen-aggregates`.
2. Give its table a migration in `db::MIGRATIONS`.
3. Write its subscription, whose handlers leave the table as it should be
   when an event is delivered twice, and start it in `subscriptions::start`.
