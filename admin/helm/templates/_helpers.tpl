{{- define "admin.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "admin.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else if contains (include "admin.name" .) .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name (include "admin.name" .) | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{- define "admin.selectorLabels" -}}
app.kubernetes.io/name: {{ include "admin.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "admin.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
{{ include "admin.selectorLabels" . }}
app.kubernetes.io/version: {{ .Values.image.tag | default .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "admin.image" -}}
{{- printf "%s:%s" .Values.image.repository (.Values.image.tag | default (printf "admin-%s" .Chart.AppVersion)) }}
{{- end }}

{{- define "admin.envSecret" -}}
{{- .Values.existingSecret | default (printf "%s-env" (include "admin.fullname" .)) }}
{{- end }}

{{/* The address administrators reach admin at: said, or the one it is published at. */}}
{{- define "admin.url" -}}
{{- if .Values.config.server.url }}
{{- .Values.config.server.url }}
{{- else if and .Values.route.enabled .Values.route.hostname }}
{{- printf "https://%s" .Values.route.hostname }}
{{- else }}
{{- fail "config.server.url is required without a route: the address administrators reach admin at" }}
{{- end }}
{{- end }}

{{/* What admin and its `migrate` both run with. */}}
{{- define "admin.env" -}}
{{- range $name, $value := .Values.extraEnv }}
- name: {{ $name }}
  value: {{ $value | quote }}
{{- end }}
{{- end }}

{{- define "admin.volumeMounts" -}}
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

{{- define "admin.containerSecurityContext" -}}
allowPrivilegeEscalation: false
readOnlyRootFilesystem: true
capabilities:
  drop: ["ALL"]
{{- end }}
