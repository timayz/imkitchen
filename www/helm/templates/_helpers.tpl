{{- define "www.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "www.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else if contains (include "www.name" .) .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name (include "www.name" .) | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{- define "www.selectorLabels" -}}
app.kubernetes.io/name: {{ include "www.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "www.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
{{ include "www.selectorLabels" . }}
app.kubernetes.io/version: {{ .Values.image.tag | default .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "www.image" -}}
{{- printf "%s:%s" .Values.image.repository (.Values.image.tag | default (printf "www-%s" .Chart.AppVersion)) }}
{{- end }}

{{- define "www.envSecret" -}}
{{- .Values.existingSecret | default (printf "%s-env" (include "www.fullname" .)) }}
{{- end }}

{{/* The address visitors reach www at: said, or the one it is published at. */}}
{{- define "www.url" -}}
{{- if .Values.config.server.url }}
{{- .Values.config.server.url }}
{{- else if and .Values.route.enabled .Values.route.hostname }}
{{- printf "https://%s" .Values.route.hostname }}
{{- else }}
{{- fail "config.server.url is required without a route: the address visitors reach www at" }}
{{- end }}
{{- end }}

{{/* What www and its `migrate` both run with. */}}
{{- define "www.env" -}}
{{- range $name, $value := .Values.extraEnv }}
- name: {{ $name }}
  value: {{ $value | quote }}
{{- end }}
{{- end }}

{{- define "www.volumeMounts" -}}
- name: config
  readOnly: true
  mountPath: /etc/imkitchen
- name: data
  mountPath: /var/lib/imkitchen
- name: tmp
  mountPath: /tmp
{{- if .Values.config.api.tls.secretName }}
- name: api-ca
  readOnly: true
  mountPath: /etc/imkitchen-tls/api
{{- end }}
{{- end }}

{{- define "www.containerSecurityContext" -}}
allowPrivilegeEscalation: false
readOnlyRootFilesystem: true
capabilities:
  drop: ["ALL"]
{{- end }}
