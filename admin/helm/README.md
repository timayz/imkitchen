# imkitchen admin chart

Deploys admin as a StatefulSet: one pod by default, following the api's
events into a store of its own, on a volume of its own, and serving the
screens from it.
`values.yaml` says what each value does.

```sh
helm install imkitchen-admin admin/helm \
  --set secretEnv.IMKITCHEN_ADMIN__API__TOKEN="$ADMIN_TOKEN" \
  --set secretEnv.IMKITCHEN_ADMIN__SESSION__KEY="$(openssl rand -base64 48)" \
  --set route.enabled=true --set route.hostname=admin.imkitchen.app
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
- **One key for the sessions**, the same for every pod and from one start
  to the next: whoever is signed in stays so through an upgrade, and
  whichever pod answers.
- **All pods start at once** (`podManagementPolicy: Parallel`): none waits
  on another.
- **`migrate` runs before `serve`**, as an init container, on the pod's own
  volume.
- **`/health` is every probe.** admin answers whether the api does or not,
  and `/health` says how it stands with it (`"api": "connected"`). While
  the api is away nobody is let past the sign-in page: every screen asks it
  who its reader is.
- **An upgrade replaces one pod at a time**, each ready again before the
  next, and a PodDisruptionBudget keeps a drain from taking more than one.
  With a single pod, admin is away for the time it takes to start again.

## What to set

- `secretEnv.IMKITCHEN_ADMIN__API__TOKEN`, required: the token the api
  gives admin (`IMKITCHEN_API__SIDECARS__TOKENS__ADMIN` there).
- `secretEnv.IMKITCHEN_ADMIN__SESSION__KEY`, required: what the cookie of a
  session is encrypted under, 32 bytes or more. Changing it signs everyone
  out.
- `existingSecret` for a Secret made elsewhere that holds the two, in place
  of `secretEnv`.
- `config.api.url` and `config.api.events`: the service of the api, its
  HTTP port and its `sidecars` port (`sidecars.enabled` in the api's
  chart). The defaults are those of an api released as `imkitchen-api` in
  the same namespace.
- `config.api.publicUrl`: the address the api is published at, for the
  pictures of the recipes, which the browsers get from it. Without it the
  screens name the pictures and show none.
- `config.api.tls.secretName` when the api gives its sidecars port a
  certificate: the Secret whose `ca.crt` it is checked against. Without it
  the token travels in clear.
- `route` to publish admin through a Gateway. The address administrators
  reach it at (`config.server.url`) is `https://<route.hostname>` unless
  said otherwise; without a route it has to be said. Over `https` the
  cookie of a session is `Secure`.

Who may read admin is the api's to say: an administrator there (root, or
whoever root made one) signs in with their address and password.

## The image

`timayz/imkitchen:admin-<version>`, built from the root of the repository:

```sh
docker build -f admin/Dockerfile -t timayz/imkitchen:admin-local .
```

It holds the binary and, next to it, the assets it serves (the stylesheet),
bundled from the same build. It is pushed when a version is tagged
(`.github/workflows/publish.yaml`); the same workflow run by hand builds it
and pushes nothing. `image.tag` defaults to `admin-<the chart's appVersion>`.
The build starts from `timayz/imkitchen:admin-builder`
(`admin/builder.Dockerfile`, the Builder workflow), which has to be there
first.
