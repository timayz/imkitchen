# imkitchen www chart

Deploys www as a StatefulSet: two pods by default, each following the api's
events into a store of its own, on a volume of its own, and serving the
pages from it.
`values.yaml` says what each value does.

```sh
helm install imkitchen-www www/helm \
  --set secretEnv.IMKITCHEN_WWW__API__TOKEN="$WWW_TOKEN" \
  --set route.enabled=true --set route.hostname=imkitchen.app \
  --set route.aboutHostname=about.imkitchen.app
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
- **`migrate` runs before `serve`**, as an init container, on the pod's own
  volume.
- **`/health` is every probe.** www answers whether the api does or not: a
  pod is not taken out of the service for an api that is away, it serves
  what its store holds and catches up once the api is back. `/health` says
  how it stands with it (`"api": "connected"`).
- **An upgrade replaces one pod at a time**, each ready again before the
  next, and a PodDisruptionBudget keeps a drain from taking more than one.
  With a single pod, www is away for the time it takes to start again.

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
- `route.aboutHostname`: a second hostname that is www whoever asks. On the
  main domain, www answers what no other route takes: app's chart, released
  on the same hostname, takes the browsers that are signed in and its own
  `/auth` and `/_app`, the api's chart `/api` (`app/helm/README.md`). The
  certificate asked here names both hostnames, and is the main domain's for
  the three charts.
- `config.app` once the application is listed: the links the home gives,
  and `config.app.web` once app is released on the main domain, where a
  visitor signs in (`https://<route.hostname>/auth/sign-in`).

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
