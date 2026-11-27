# imkitchen app chart

Deploys app as a StatefulSet: two pods by default, each following the api's
events into a store of its own, on a volume of its own, and serving the
pages from it, never from the api.
`values.yaml` says what each value does.

```sh
helm install imkitchen-app app/helm \
  --set secretEnv.IMKITCHEN_APP__API__TOKEN="$APP_TOKEN" \
  --set route.enabled=true --set route.hostname=app.example.com
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
- **`/health` is every probe.** app answers whether the api does or not: a
  pod is not taken out of the service for an api that is away, it serves
  what its store holds and catches up once the api is back. `/health` says
  how it stands with it (`"api": "connected"`).
- **An upgrade replaces one pod at a time**, each ready again before the
  next, and a PodDisruptionBudget keeps a drain from taking more than one.
  With a single pod, app is away for the time it takes to start again.

## What to set

- `secretEnv.IMKITCHEN_APP__API__TOKEN`, required: the token the api gives
  app (`IMKITCHEN_API__SIDECARS__TOKENS__APP` there), or `existingSecret`
  for a Secret made elsewhere.
- `config.api.events`: the `sidecars` port of the api's service
  (`sidecars.enabled` in the api's chart). The default is that of an api
  released as `imkitchen-api` in the same namespace.
- `config.api.tls.secretName` when the api gives its sidecars port a
  certificate: the Secret whose `ca.crt` it is checked against. Without it
  the token travels in clear.
- `route` to publish app through a Gateway.

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
