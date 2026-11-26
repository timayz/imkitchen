# imkitchen www chart

Deploys www as a Deployment: two pods by default, each following the api's
events into a store of its own and serving the pages from it.
`values.yaml` says what each value does.

```sh
helm install imkitchen-www www/helm \
  --set secretEnv.IMKITCHEN_WWW__API__TOKEN="$WWW_TOKEN" \
  --set route.enabled=true --set route.hostname=imkitchen.app
```

## What it runs

- **Pods that hold nothing of their own.** The store of a pod is an
  `emptyDir`: its read models, built from the api's events, and gone with
  the pod. A new pod builds its own from the first event and serves what it
  has meanwhile, so pods are added, replaced and taken away at will.
- **`migrate` runs before `serve`**, as an init container, on the pod's
  store.
- **`/health` is every probe.** www answers whether the api does or not: a
  pod is not taken out of the service for an api that is away, it serves
  what its store holds and catches up once the api is back. `/health` says
  how it stands with it (`"api": "connected"`).
- **An upgrade starts a new pod before it stops an old one**, and a
  PodDisruptionBudget keeps a drain from taking more than one at a time.

## What to set

- `secretEnv.IMKITCHEN_WWW__API__TOKEN`, required: the token the api gives
  www (`IMKITCHEN_API__SIDECARS__TOKENS__WWW` there), or `existingSecret`
  for a Secret made elsewhere.
- `config.api.url` and `config.api.events`: the service of the api, its
  HTTP port and its `sidecars` port (`sidecars.enabled` in the api's
  chart). The defaults are those of an api released as `imkitchen-api` in
  the same namespace.
- `config.api.tls.secretName` when the api gives its sidecars port a
  certificate: the Secret whose `ca.crt` it is checked against. Without it
  the token travels in clear.
- `route` to publish www through a Gateway. The address visitors reach it
  at (`config.server.url`, what the canonical addresses and the sitemap
  start with) is `https://<route.hostname>` unless said otherwise; without
  a route it has to be said.
- `config.app` once the application is listed: the links the home gives.

## The image

`timayz/imkitchen:www-<version>`, built from the root of the repository:

```sh
docker build -f www/Dockerfile -t timayz/imkitchen:www-local .
```

It holds the binary and, next to it, the assets it serves (the stylesheet),
bundled from the same build. It is pushed when a version is tagged
(`.github/workflows/publish.yaml`); the same workflow run by hand builds it
and pushes nothing. `image.tag` defaults to `www-<the chart's appVersion>`.
The build starts from `timayz/imkitchen:www-builder`
(`www/builder.Dockerfile`, the Builder workflow), which has to be there
first.
