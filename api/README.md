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
| Set a part of a thing to what the body says | `PUT /v1/recipes/{id}/{part}` | `204` |
| Delete a thing | `DELETE /v1/recipes/{id}` | `204` |
| Anything else that is done to a thing | `POST /v1/recipes/{id}/{verb}` | `204`, or `201` and what it created |

- `PUT` is for commands that give the same result sent twice
  (`PUT /v1/account/profile`); a command that is not is a `POST` on a verb
  (`POST /v1/auth/sign-out`, `POST /v1/admin/users/{id}/sign-out`).
- There is no `PATCH`: a command says what it does, it is not a diff.
- A command whose first step leaves something for the person to do (a code
  to read in a mail) answers `202`; the same request with what was missing
  finishes it.
- `GET` reads and never changes anything.

A command has answered once its events are written. The lists www and admin
build from those events follow a moment later.

## Bodies

- Bodies are JSON, sent with `Content-Type: application/json`. Fields are in
  `snake_case`.
- Ids are strings. Moments are unix seconds.
- A field left out and a field set to `null` are the same.
- An answer without anything to say is a `204` with no body.

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
  of the body by its path (`name`, `ingredients[2].quantity`), with a code
  and a message of its own and, when the value is held to bounds, the bounds
  in `params`.

| Status | Meaning | Codes |
| --- | --- | --- |
| `400` | The body is not JSON | `invalid_request` |
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
