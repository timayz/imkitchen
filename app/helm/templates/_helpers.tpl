{{- define "app.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "app.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else if contains (include "app.name" .) .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name (include "app.name" .) | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{- define "app.selectorLabels" -}}
app.kubernetes.io/name: {{ include "app.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "app.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" }}
{{ include "app.selectorLabels" . }}
app.kubernetes.io/version: {{ .Values.image.tag | default .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "app.image" -}}
{{- printf "%s:%s" .Values.image.repository (.Values.image.tag | default (printf "app-%s" .Chart.AppVersion)) }}
{{- end }}

{{- define "app.envSecret" -}}
{{- .Values.existingSecret | default (printf "%s-env" (include "app.fullname" .)) }}
{{- end }}

{{/* What app and its `migrate` both run with. */}}
{{- define "app.env" -}}
{{- range $name, $value := .Values.extraEnv }}
- name: {{ $name }}
  value: {{ $value | quote }}
{{- end }}
{{- end }}

{{- define "app.volumeMounts" -}}
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

{{- define "app.containerSecurityContext" -}}
allowPrivilegeEscalation: false
readOnlyRootFilesystem: true
capabilities:
  drop: ["ALL"]
{{- end }}
