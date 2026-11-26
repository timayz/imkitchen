{{- define "api.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "api.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else if contains (include "api.name" .) .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name (include "api.name" .) | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{- define "api.selectorLabels" -}}
app.kubernetes.io/name: {{ include "api.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "api.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
{{ include "api.selectorLabels" . }}
app.kubernetes.io/version: {{ .Values.image.tag | default .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/* The headless service the nodes reach each other through. */}}
{{- define "api.nodes" -}}
{{- printf "%s-nodes" (include "api.fullname" .) }}
{{- end }}

{{/* Whether the nodes replicate the store between them. */}}
{{- define "api.clustered" -}}
{{- if gt (int .Values.replicaCount) 1 }}true{{- end }}
{{- end }}

{{- define "api.image" -}}
{{- printf "%s:%s" .Values.image.repository (.Values.image.tag | default (printf "api-%s" .Chart.AppVersion)) }}
{{- end }}

{{- define "api.envSecret" -}}
{{- .Values.existingSecret | default (printf "%s-env" (include "api.fullname" .)) }}
{{- end }}

{{- define "api.nodesTlsSecret" -}}
{{- .Values.cluster.tls.secretName | default (printf "%s-nodes-tls" (include "api.fullname" .)) }}
{{- end }}

{{/* Each node as the others reach it, in the order of their ordinals. */}}
{{- define "api.peerHosts" -}}
{{- $hosts := list }}
{{- range $i := until (int .Values.replicaCount) }}
{{- $hosts = append $hosts (printf "%s-%d.%s" (include "api.fullname" $) $i (include "api.nodes" $)) }}
{{- end }}
{{- $hosts | toJson }}
{{- end }}

{{/* What the api and its `migrate` both run with. */}}
{{- define "api.env" -}}
{{- if include "api.clustered" . }}
# This node's place among the peers: its ordinal in the StatefulSet.
- name: IMKITCHEN_API__CLUSTER__NODE_ID
  valueFrom:
    fieldRef:
      fieldPath: metadata.labels['apps.kubernetes.io/pod-index']
{{- end }}
{{- range $name, $value := .Values.extraEnv }}
- name: {{ $name }}
  value: {{ $value | quote }}
{{- end }}
{{- end }}

{{- define "api.volumeMounts" -}}
- name: config
  readOnly: true
  mountPath: /etc/imkitchen
- name: data
  mountPath: /var/lib/imkitchen
- name: tmp
  mountPath: /tmp
{{- if and (include "api.clustered" .) .Values.cluster.tls.enabled }}
- name: nodes-tls
  readOnly: true
  mountPath: /etc/imkitchen-tls/nodes
{{- end }}
{{- if and .Values.sidecars.enabled .Values.sidecars.tls.secretName }}
- name: sidecars-tls
  readOnly: true
  mountPath: /etc/imkitchen-tls/sidecars
{{- end }}
{{- end }}

{{- define "api.containerSecurityContext" -}}
allowPrivilegeEscalation: false
readOnlyRootFilesystem: true
capabilities:
  drop: ["ALL"]
{{- end }}
