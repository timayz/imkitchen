# imkitchen api chart

Deploys the api as a StatefulSet: three nodes by default, each over a store
and a copy of the pictures of its own, replicating every write between them.
`values.yaml` says what each value does.

```sh
helm install imkitchen api/helm \
  --set secretEnv.IMKITCHEN_API__AUTH__SECRET="$(openssl rand -base64 32)"
```

## What it runs

- **A node per pod.** Pod `N` is node `N` of the cluster: its `node_id` is
  its ordinal, read from the `apps.kubernetes.io/pod-index` label
  (Kubernetes 1.28 or later). The peers are the pods' names on a headless
  service, `<fullname>-nodes`.
- **All pods start at once** (`podManagementPolicy: Parallel`), and the
  headless service publishes them before they are ready: a node is ready
  once a majority confirmed it, so they have to find each other first.
- **`/ready` is the readiness probe.** A node that takes no writes is left
  out of the service while the others serve. An upgrade replaces one pod at
  a time, each ready again before the next, and a PodDisruptionBudget keeps
  a drain from taking more than one.
- **`migrate` runs before `serve`**, as an init container, on each node's own
  volume.
- **A volume per node** (`persistence`), mounted at `/var/lib/imkitchen`: the
  store and the pictures. A node that lost its volume copies the store back
  from a peer when it starts, and takes the pictures as they are asked for.

## What to set

- `secretEnv.IMKITCHEN_API__AUTH__SECRET`, required with more than one node,
  and the rest of the secrets (`values.yaml` lists them), or
  `existingSecret` for a Secret made elsewhere.
- `replicaCount`, once: the api has no command yet to add a node to a
  cluster that lives or to take one out. Changing it on a release that runs
  is not supported.
- `cluster.tls` for mutual TLS between the nodes, `sidecars` for www and
  admin, `route` to publish the api through a Gateway.

## The image

`timayz/imkitchen-api`, built from the root of the repository:

```sh
docker build -f api/Dockerfile -t timayz/imkitchen-api .
```

It is pushed when a version is tagged (`.github/workflows/publish.yaml`).
`image.tag` defaults to the chart's `appVersion`.
