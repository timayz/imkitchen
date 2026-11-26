# imkitchen api

The single source of truth of imkitchen: every aggregate and every command
lives here. www, admin and the app never write to a store themselves; they
send their commands to this api over HTTP, as described below.

This page is the convention every endpoint follows. The reference of the
endpoints themselves is generated from the code: `GET /v1/openapi.json` is
the OpenAPI document, `GET /v1/docs` shows it.

## Versions

Every endpoint lives under a version, `/v1`. Adding an endpoint, an optional
field to a body or a field to an answer does not change the version: clients
ignore the fields they do not know. Anything that would break a client
(removing or renaming a field or an endpoint, changing what a status means)
goes to `/v2`, and `/v1` keeps answering as before for as long as a client
uses it.

`/health` and `/ready` are for the probes of the node, not for clients: they
have no version.

## Addresses

An address names a thing, in lowercase `kebab-case`; the method says what is
done with it.

- A collection is a plural noun, one of its members is under its id:
  `/v1/recipes`, `/v1/recipes/{id}`.
- What a user has exactly one of is singular and needs no id, the token says
  whose it is: `/v1/account`.
- What only an administrator may touch is under `/v1/admin`.

| Command | Method and address | Answer |
| --- | --- | --- |
| Create a thing | `POST /v1/recipes` | `201` and the thing, with its `id` |
| Set a thing to what the body says, as a whole | `PUT /v1/recipes/{id}` | `204` |
| Set a part of a thing to what the body says | `PUT /v1/recipes/{id}/{part}` | `204` |
| Take a part off a thing | `DELETE /v1/recipes/{id}/{part}` | `204` |
| Delete a thing | `DELETE /v1/recipes/{id}` | `204` |
| Create a thing out of something else | `POST /v1/recipes/{verb}` (`import`) | `201` and the thing, with its `id` |
| Anything else that is done to a thing | `POST /v1/recipes/{id}/{verb}` | `204`, or `201` and what it created |

- `PUT` is for commands that give the same result sent twice
  (`PUT /v1/account/profile`); a command that is not is a `POST` on a verb
  (`POST /v1/auth/sign-out`, `POST /v1/admin/users/{id}/sign-out`).
- There is no `PATCH`: a command says what it does, it is not a diff.
- A command whose first step leaves something for the person to do (a code
  to read in a mail) answers `202`; the same request with what was missing
  finishes it.
- `GET` reads and never changes anything.

A command has answered once its events are written, and the thing it wrote
reads as it left it (`GET /v1/recipes/{id}`). The lists follow a moment
later: the api's own (`GET /v1/recipes`) and those www and admin build from
the events.

## Lists

A list is read a page at a time, the most recent first.

- `limit` is how many things a page holds at most.
- The answer holds the things under their plural name and, while there is a
  page beyond, `next`: what `after` takes to read it.
- `q` keeps the things what was typed is found in, whatever the case and the
  accents.

What belongs to a user is theirs alone: someone else's thing is answered as
one that does not exist (`404`), read or written.

## Bodies

- Bodies are JSON, sent with `Content-Type: application/json`. Fields are in
  `snake_case`.
- Ids are strings. Moments are unix seconds.
- A field left out and a field set to `null` are the same.
- An answer without anything to say is a `204` with no body.
- A picture is the one body that is not JSON: it is sent as it is
  (`PUT /v1/recipes/{id}/image`), with its own `Content-Type`.

## Pictures

A recipe that shows a picture answers it under `image`, in the two sizes it
is kept in, each with its address from the root of the api and its
dimensions:

```json
"image": {
  "small": { "url": "/v1/images/400/01JDQ0B6V5K1X5C3E8W4T9Z2YH.webp", "width": 400, "height": 267 },
  "large": { "url": "/v1/images/1200/01JDQ0B6V5K1X5C3E8W4T9Z2YH.webp", "width": 1200, "height": 800 }
}
```

- What is sent as a picture (a JPEG, PNG, WebP or GIF) is not kept as it
  is: it is turned into two WebP files, 400 and 1200 pixels on their longest
  side at most, never enlarged.
- A picture never changes at its address: a recipe given another picture
  answers another address. It is served with a `Cache-Control` that lets a
  browser or a cache keep it for good.
- Its address needs no token, so that whatever shows a picture can fetch
  it. It is only handed out with the recipe, and cannot be guessed.
- The same picture given twice, by one user or by two, is stored once: two
  users who import the same recipe show the same files. It is kept for as
  long as one recipe shows it.
- With several nodes, every node keeps every picture and any of them serves
  it: a picture is on a majority of the nodes before the recipe that shows
  it is written, and a node that lacks one takes it from another.

## Plats

A plat is a composed dish: a name and recipes of the user, in the order they
are served (`recipe_ids`, 1 to 20, none twice). It holds nothing of its
recipes: it answers each under `recipes` as the list of the recipes shows
it, as the recipe now stands.

- A plat is only composed of its owner's recipes: any other id is refused
  under its field (`recipe_ids[1]`, `recipe_not_found`).
- Deleting a recipe is never refused for the plats it is part of: the
  recipe leaves each of them a moment later, the others as they were. A
  plat left with no recipe stays, for its owner to recompose or delete.
- Deleting a plat deletes none of its recipes.

## Settings

A user has one set of settings, there from the start: `GET /v1/settings`
answers the defaults until they set something. Each part is set as a whole,
on every device they sign in on.

```json
{
  "list": { "servings": 4, "meals": 7, "sources": "plats_and_recipes" },
  "language": "fr"
}
```

- `PUT /v1/settings/list` sets how their list is generated: how many people
  it is cooked for (`servings`, 1 to 1000, 4 by default), how many meals it
  holds (`meals`, 1 to 31, 7 by default) and what they are drawn from
  (`sources`: `plats_and_recipes` by default, `plats` or `recipes`). The
  list as it stands is left as it is: the next one generated follows.
- `PUT /v1/settings/language` sets the language they read, one of those the
  api speaks. Without one (`language` left out, as it is until it is set),
  each device goes by its own. It is for the clients to show themselves in:
  the api words its refusals as `Accept-Language` asks, whatever it says.

## List

A user has one list, there from the start and empty: the meals they plan to
cook, in the order they were planned. A meal is a recipe of theirs or one of
their plats, cooked for so many people. The same one may be planned more
than once, so each entry has an id of its own. `GET /v1/list` answers each
with its `recipe` or its `plat`, as it now stands:

```json
{
  "entries": [
    { "id": "01JDQ0B6V5K1X5C3E8W4T9Z2YH", "servings": 4, "recipe": { "id": "…", "title": "Soup" } },
    { "id": "01JDQ0B7M2R8N4P6S0V3X5Z7AB", "servings": 2, "plat": { "id": "…", "name": "Sunday roast", "recipes": [] } }
  ]
}
```

- `POST /v1/list/generate` draws the list anew as the settings say, in
  place of what it held: `meals` of them at random, none twice, each for
  `servings`. Going by `sources` they are the user's plats, their recipes,
  or both: then a recipe that is part of a plat comes with its plat, not on
  its own. A plat made of no recipe is never drawn.
- There are fewer meals than asked when the user has not as many, and what
  the list held is drawn last, so that a list generated again changes as
  much as it can. With nothing to draw it is refused
  (`nothing_to_generate`).
- It draws from what `GET /v1/recipes` and `GET /v1/plats` list: a recipe
  just written is drawn from a moment later.
- `POST /v1/list/entries` puts a meal after the others (`recipe_id` or
  `plat_id`, one of the two, and `servings`, the settings' when left out);
  `PUT /v1/list/entries/{id}` replaces one in its place, as a whole;
  `DELETE` takes it off. A list holds 50 meals at most (`list_full`).
- A recipe or a plat that is deleted leaves the list a moment later, the
  other meals as they were.
- Changing the settings changes nothing of the list as it stands.

## Groceries

A user has one set of groceries: what their list takes, as it stands.
`GET /v1/groceries` works it out at every read from the list and from its
recipes, so there is nothing to generate and nothing to wait for: a meal
picked, replaced or taken off, a list generated again or a recipe rewritten
is in the next answer.

```json
{
  "items": [
    { "id": "5e0f…", "name": "eggs", "quantity": 6, "checked": true },
    { "id": "9b1c…", "name": "flour", "unit": "g", "quantity": 530, "adjusted_quantity": 1000, "checked": false },
    { "id": "c47a…", "name": "salt", "checked": false }
  ]
}
```

- Every ingredient of a meal is scaled to the people the meal is cooked for
  (`servings` of its entry over those of the recipe); a plat is each of its
  recipes. Amounts are rounded to the thousandth.
- The same thing in the same unit is one item, of the sum: names and units
  are the same whatever their case and their spacing. No unit is turned
  into another: "200 g" and "1 cup" of flour are two items. An ingredient
  no recipe gives an amount of ("salt") has no `quantity`.
- Items come by name. An item's `id` says what it is of and in what unit:
  it is the same whatever the list holds.
- `PUT /v1/groceries/items/{id}/check` puts an item in the basket, `DELETE`
  takes it out. `PUT /v1/groceries/items/{id}/quantity` sets how much of it
  the user buys (`adjusted_quantity`, next to what the list calls for),
  `DELETE` goes back to what the list calls for. Each may be sent again:
  the same as it stands writes nothing.
- What is said of an item is said of the amount the list then called for,
  and holds for as long as it calls for that much. A list change that
  leaves an item as it was leaves it checked and adjusted; one that changes
  how much is needed has it to buy again, as the list calls for it.
- An item the list does not call for is refused
  (`grocery_item_not_found`).

## Tutorial

The app walks a new user through five steps. How far they are is kept here,
so it is the same on every device: `GET /v1/tutorial` answers every step, in
order, and whether it is done.

```json
{
  "steps": [
    { "step": "recipe", "completed": true },
    { "step": "plat", "completed": false },
    { "step": "settings", "completed": false },
    { "step": "list", "completed": false },
    { "step": "groceries", "completed": false }
  ],
  "skipped": false,
  "completed": false
}
```

- No command does a step for its own sake: each is done by what it names,
  wherever it is made from, and shows a moment later. `recipe` by writing a
  recipe or importing one, `plat` by composing a plat, `settings` by setting
  how the list is generated or the language, `list` by generating the list
  or picking a meal, `groceries` by checking or adjusting an item.
- The steps are done in any order; `completed` says they all are.
- `POST /v1/tutorial/skip` skips it: the steps done stay done, and no other
  is noted. `POST /v1/tutorial/restart` starts it over, skipped or not: no
  step is done, and each is done again by the next command that does it,
  whatever the user already has.

## Who asks

Signing in answers a token. It is sent back with every request as
`Authorization: Bearer <token>`. An endpoint that needs one says so in the
reference.

## Refusals

Whenever the api does not do what was asked, it answers with a status of
`400` or more and this body, always of the same shape:

```json
{
  "error": {
    "code": "validation_failed",
    "message": "Some fields are not valid.",
    "fields": [
      {
        "field": "name",
        "code": "too_long",
        "message": "This must be at most 255 characters.",
        "params": { "max": 255 }
      }
    ]
  }
}
```

- `code` is a stable word: what a client acts on. It never changes within a
  version.
- `message` is the same thing said to a person, ready to be shown.
- `fields` is only there with `validation_failed`. Each entry names a field
  of the body, or a parameter of a list, by its path (`name`, `ingredients[2].quantity`), with a code
  and a message of its own and, when the value is held to bounds, the bounds
  in `params`.

| Status | Meaning | Codes |
| --- | --- | --- |
| `400` | The body is not JSON, or what follows the `?` cannot be read | `invalid_request` |
| `401` | Nobody is signed in: sign in (again) | `sign_in_required`, `signed_out` |
| `403` | The user may not do this | `not_allowed` |
| `404` | Nothing at this address, or the thing does not exist | `not_found`, … |
| `405` | The address does not take this method | `method_not_allowed` |
| `409` | The command no longer fits the state of things: start again | `invalid_state` |
| `422` | The person can correct it: a field, a wrong code | `validation_failed`, `invalid_request` (a body of the wrong shape), the command's own codes |
| `429` | Asked again too soon | `code_recently_sent`, `too_many_sign_ins` |
| `500` | The api failed; nothing is said of why | `internal` |
| `503` | The store is not taking writes for the moment: send it again | `writes_paused`, `write_not_taken` |

### Language

Messages are worded in the language the request asks for with
`Accept-Language`, among those the api speaks: English (`en`, the default)
and French (`fr`). The answer says which it used with `Content-Language`.
Codes are the same in every language.

## Events for www and admin

www and admin keep read models of their own and build them from the api's
events. They do not ask for them over HTTP: with a `[sidecars]` section in
its configuration, the api listens on a second port that speaks evento's
remote protocol, and each of them follows it as an evento `Sidecar`, the
events read from the api, the cursors of its subscriptions and its read
models kept in a store of its own.

```rust
let client = evento::RemoteClient::builder("api:4000")
    .token(token)          // sidecars.tokens.www or .admin
    .connect_lazy();       // the sidecar starts whether the api answers or not
let executor = evento::Sidecar::new(client, local_store);
subscription.supervised().start(&executor).await?;
```

- A connection opens with the token of its sidecar and is refused without
  one. With `tls_cert` and `tls_key` the listener speaks TLS; without them
  the token travels in clear text.
- A sidecar reads the aggregates the `imkitchen-aggregates` crate lists for
  it (`Sidecar::reads`), which is also where their event types are. Anything
  else is refused.
- Nothing is written this way: an event, a cursor or a snapshot. Commands go
  through the HTTP endpoints.
- A sidecar that was away, or whose api was, goes on from its own cursor
  once it is connected again. It may connect to any node of the cluster.

An aggregate is exposed by adding it to `imkitchen-aggregates` and to the
list of the sidecars that need it.

## Adding a command

1. Give its body a struct deriving `Deserialize`, `Validate` and `ToSchema`,
   with a `#[validate(...)]` on every field held to a rule.
2. Write the handler: it takes the body as `Body<T>`, which refuses what is
   not valid before the handler runs, and `Session` or `Admin` when it needs
   someone signed in. It returns `ApiResult<_>`; domain errors turn into an
   `ApiError` with a code.
3. Describe it with `#[utoipa::path(...)]` and register it with
   `routes!(handler)`: the router only takes handlers that are described, so
   the reference cannot fall behind the code.
4. Give every new code its sentence in each file of `locales/`.

A list takes its parameters as `Params<T>`, checked the way a body is, and
reads a table of the store a subscription fills from the events: the table
goes in `db::MIGRATIONS`, the subscription in `subscriptions::start`.
