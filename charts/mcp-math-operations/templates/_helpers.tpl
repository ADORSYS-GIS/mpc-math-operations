{{/* Expand the name of the chart. */}}
{{- define "mcp-math-operations.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/* Fully qualified app name. */}}
{{- define "mcp-math-operations.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{- define "mcp-math-operations.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/* Common labels. */}}
{{- define "mcp-math-operations.labels" -}}
helm.sh/chart: {{ include "mcp-math-operations.chart" . }}
{{ include "mcp-math-operations.selectorLabels" . }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/* Selector labels. */}}
{{- define "mcp-math-operations.selectorLabels" -}}
app.kubernetes.io/name: {{ include "mcp-math-operations.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{/* Service account name. */}}
{{- define "mcp-math-operations.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "mcp-math-operations.fullname" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- default "default" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{/* Image reference: repository:tag, defaulting tag to the chart appVersion. */}}
{{- define "mcp-math-operations.image" -}}
{{- printf "%s:%s" .Values.image.repository (default .Chart.AppVersion .Values.image.tag) -}}
{{- end -}}

{{/* ALLOWED_HOSTS = service name + loopback + any user-supplied hosts. */}}
{{- define "mcp-math-operations.allowedHosts" -}}
{{- $base := list (include "mcp-math-operations.fullname" .) "localhost" "127.0.0.1" -}}
{{- concat $base .Values.config.allowedHosts | uniq | join "," -}}
{{- end -}}
