{{/*
Expand the name of the chart.
*/}}
{{- define "koprogo.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "koprogo.fullname" -}}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- printf "%s" $name | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "koprogo.labels" -}}
helm.sh/chart: {{ .Chart.Name }}-{{ .Chart.Version }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: koprogo
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
gdpr-compliant: "true"
{{- end }}

{{/*
Backend selector labels
*/}}
{{- define "koprogo.backend.selectorLabels" -}}
app.kubernetes.io/name: koprogo-backend
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Frontend selector labels
*/}}
{{- define "koprogo.frontend.selectorLabels" -}}
app.kubernetes.io/name: koprogo-frontend
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Database URL
*/}}
{{- define "koprogo.databaseUrl" -}}
postgresql://{{ .Values.postgres.user }}:{{ .Values.secrets.postgresPassword }}@{{ include "koprogo.fullname" . }}-postgres:{{ .Values.postgres.port }}/{{ .Values.postgres.database }}
{{- end }}
