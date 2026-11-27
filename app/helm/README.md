# imkitchen app chart

Deploys app as a StatefulSet: two pods by default, each following the api's
events into a store of its own, on a volume of its own, and serving the
pages from it, never from the api. The api is asked who reads them, and
handed what they do.
`values.yaml` says what each value does.

```sh
helm install imkitchen-app app/helm \
  --set secretEnv.IMKITCHEN_APP__API__TOKEN="$APP_TOKEN" \
  --set secretEnv.IMKITCHEN_APP__SESSION__KEY="$(openssl rand -base64 48)" \
  --set config.www.url=https://about.example.com \
  --set route.enabled=true --set route.hostname=example.com
```

## What it runs

- **A volume per pod** (`persistence`), mounted at `/var/lib/imkitchen`: its
  store, the read models it builds from the api's events and how far it
  followed them. A pod that starts again goes on from where it stopped.
- **Pods that hold nothing of their own.** Everything in a store comes from
  the api: a pod that lost its volume, or a new one, builds its store from
  the first event and serves what it has meanwhile. `replicaCount` is
  changed at will; the volume of a pod that was taken away stays until it
  is deleted.
- **All pods start at once** (`podManagementPolicy: Parallel`): none waits
  on another.
- **One key for the sessions**, the same for every pod and from one start
  to the next: whoever is signed in stays so through an upgrade, and
  whichever pod answers.
- **`migrate` runs before `serve`**, as an init container, on the pod's own
  volume.
- **`/_app/health` is every probe.** app answers whether the api does or
  not: a pod is not taken out of the service for an api that is away, and
  catches up once the api is back. It says how it stands with its events
  (`"api": "connected"`). While the api is away nobody is let past the
  pages of `/auth`: every other page asks it who its reader is.
- **An upgrade replaces one pod at a time**, each ready again before the
  next, and a PodDisruptionBudget keeps a drain from taking more than one.
  With a single pod, app is away for the time it takes to start again.

## What to set

- `secretEnv.IMKITCHEN_APP__API__TOKEN`, required: the token the api gives
  app (`IMKITCHEN_API__SIDECARS__TOKENS__APP` there).
- `secretEnv.IMKITCHEN_APP__SESSION__KEY`, required: what the cookie of a
  session is encrypted under, 32 bytes or more. Changing it signs everyone
  out.
- `existingSecret` for a Secret made elsewhere that holds the two, in place
  of `secretEnv`.
- `config.api.url` and `config.api.events`: the service of the api, its
  HTTP port and its `sidecars` port (`sidecars.enabled` in the api's
  chart). The defaults are those of an api released as `imkitchen-api` in
  the same namespace.
- `config.api.publicUrl`: where the browsers reach the api, for the
  pictures of the recipes. Left out, it is `/api` of the address users reach
  app at, where the gateway serves the api.
- `config.api.tls.secretName` when the api gives its sidecars port a
  certificate: the Secret whose `ca.crt` it is checked against. Without it
  the token travels in clear.
- `config.www.url`, required: the address that always serves www
  (`route.aboutHostname` of www's chart), where the pages link to the
  public ones.
- `route` to publish app through a Gateway, on the main domain. The address
  users reach it at (`config.server.url`) is `https://<route.hostname>`
  unless said otherwise; without a route it has to be said. Over `https`
  the cookies of a session are `Secure`.

## The main domain

app is published on the hostname www is published on, the main domain of
imkitchen: `route.hostname` is the same in the two charts, and in the api's
`route.mainHostname`. Three `HTTPRoute`s then share it, and the Gateway
picks the most specific match:

| Request | Route | Goes to |
| --- | --- | --- |
| `/api/…` | the api's chart | the api, as its `/…` |
| `/auth/…`, `/_app/…` | this chart | app, whoever asks |
| anything else, with the `signed_in` cookie | this chart | app |
| anything else | www's chart | www |

A signed-in browser keeps two cookies, written and taken back together:
`session`, encrypted, which holds the token of its session at the api, and
`signed_in`, which says a session is there and nothing else. The second is
the one the route matches, among the others the browser sends: a regular
expression on the `Cookie` header (`^(.*;\s*)?signed_in=.*$`, the whole
header as Envoy matches it), which the Gateway API leaves to the
implementation. Envoy Gateway has it. The route only sees that the cookie
is there: a session that is no longer one is app's to find out, which then
takes the cookie back and sends the browser to `/`, www again.

www's chart asks for the certificate of the main domain
(`route.certificate.enabled` is off here), and publishes `about.`, where
www answers a signed-in user.

## The image

`timayz/imkitchen:app-<version>`, built from the root of the repository:

```sh
docker build -f app/Dockerfile -t timayz/imkitchen:app-local .
```

It holds the binary and, next to it, the assets it serves (the stylesheet),
bundled from the same build. It is pushed when a version is tagged
(`.github/workflows/publish.yaml`); the same workflow run by hand builds it
and pushes nothing. `image.tag` defaults to `app-<the chart's appVersion>`.
The build starts from `timayz/imkitchen:app-builder`
(`app/builder.Dockerfile`, the Builder workflow), which has to be there
first.
