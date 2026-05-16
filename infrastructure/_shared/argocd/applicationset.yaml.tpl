# ApplicationSet template — rendered by gitops-bootstrap.sh via envsubst.
#
# Substitutes ${CLUSTER_TYPE} from the env var set at bootstrap time
# (auto-detected from kubectl context or passed as CLI arg).
#
# Stacks Helm values in the order:
#   1. infrastructure/_shared/helm/koprogo/values.yaml         (chart defaults — implicit)
#   2. infrastructure/_shared/cluster-profiles/${CLUSTER_TYPE}.yaml  (cluster overrides)
#   3. infrastructure/monosite/k3s/{environment}/helm-values.yaml  (env business config)
#
# goTemplate: true + templatePatch: boolean fields (syncPolicy.automated.prune /
# selfHeal) are templated through a YAML string (templatePatch) so they remain
# typed booleans after substitution. The legacy fasttemplate "{{var}}" form
# always produced strings, which the ApplicationSet CRD rejects
# ("must be of type boolean: string").

# ApplicationSet 1: Infrastructure (Kustomize)
# Per RFC 0001 (alternative F — hybrid symmetric app/infra), the infra
# generator targets the `infra-*` branches, distinct from the app generator
# below which targets `dev`/`integration`/`staging`/`production`.
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: koprogo-infra
  namespace: argocd
spec:
  goTemplate: true
  goTemplateOptions: ["missingkey=error"]
  generators:
    - list:
        elements:
          - branch: infra-prod
            environment: production
            namespace: koprogo-production
            kustomizePath: infrastructure/monosite/k3s/production/kustomize
            autoSync: true
            prune: true
          - branch: infra-staging
            environment: staging
            namespace: koprogo-staging
            kustomizePath: infrastructure/monosite/k3s/staging/kustomize
            autoSync: true
            prune: true
          - branch: infra-integration
            environment: integration
            namespace: koprogo-integration
            kustomizePath: infrastructure/monosite/k3s/integration/kustomize
            autoSync: true
            prune: false
          - branch: infra-dev
            environment: dev
            namespace: koprogo-dev
            kustomizePath: infrastructure/monosite/k3s/dev/kustomize
            autoSync: false
            prune: false
  template:
    metadata:
      name: "koprogo-infra-{{ .environment }}"
      namespace: argocd
      labels:
        app.kubernetes.io/name: koprogo
        component: infrastructure
        environment: "{{ .environment }}"
      annotations:
        # Sync wave 0 — infrastructure layer (namespace, RBAC, ingress, middlewares)
        # must converge before app layer (wave 1). Note: cross-Application sync-wave
        # ordering only takes effect under an App-of-Apps parent. Without one, the
        # annotation is a soft hint — Applications still auto-sync in parallel, and
        # the app layer relies on retry/backoff to wait for SAs to exist.
        argocd.argoproj.io/sync-wave: "0"
    spec:
      project: koprogo
      source:
        repoURL: https://github.com/gilmry/koprogo.git
        targetRevision: "{{ .branch }}"
        path: "{{ .kustomizePath }}"
      destination:
        server: https://kubernetes.default.svc
        namespace: "{{ .namespace }}"
      syncPolicy:
        syncOptions:
          - CreateNamespace=true
          - PrunePropagationPolicy=foreground
          # ServerSideApply reduces 3-way-merge drift on cluster-managed fields
          # (e.g. CreateNamespace=true adds annotations to the namespace that
          # kustomize doesn't track — without SSA, ArgoCD oscillates Synced ↔
          # OutOfSync on the namespace alone — Issue #515 Gap 8).
          - ServerSideApply=true
        retry:
          limit: 5
          backoff:
            duration: 5s
            factor: 2
            maxDuration: 3m
      revisionHistoryLimit: 10
  # syncPolicy.automated.prune / selfHeal injected as real booleans here
  # (templatePatch is a string rendered post-template, so unquoted boolean
  # tokens are preserved).
  templatePatch: |
    spec:
      syncPolicy:
        automated:
          prune: {{ .prune }}
          selfHeal: {{ .autoSync }}
---
# ApplicationSet 2: Application (Helm) — STACKS cluster profile + env values
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: koprogo-app
  namespace: argocd
  annotations:
    koprogo.io/cluster-type: "${CLUSTER_TYPE}"
spec:
  goTemplate: true
  goTemplateOptions: ["missingkey=error"]
  generators:
    - list:
        elements:
          - branch: production
            environment: production
            namespace: koprogo-production
            envValuesFile: infrastructure/monosite/k3s/production/helm-values.yaml
            autoSync: true
            prune: true
          - branch: staging
            environment: staging
            namespace: koprogo-staging
            envValuesFile: infrastructure/monosite/k3s/staging/helm-values.yaml
            autoSync: true
            prune: true
          - branch: integration
            environment: integration
            namespace: koprogo-integration
            envValuesFile: infrastructure/monosite/k3s/integration/helm-values.yaml
            autoSync: true
            prune: false
          - branch: dev
            environment: dev
            namespace: koprogo-dev
            envValuesFile: infrastructure/monosite/k3s/dev/helm-values.yaml
            autoSync: false
            prune: false
  template:
    metadata:
      name: "koprogo-app-{{ .environment }}"
      namespace: argocd
      labels:
        app.kubernetes.io/name: koprogo
        component: application
        environment: "{{ .environment }}"
        cluster-type: "${CLUSTER_TYPE}"
      annotations:
        # Sync wave 1 — application layer depends on SAs from infra layer (wave 0)
        # and Traefik CRDs from prerequisites (wave -1). See sister AppSet comment.
        argocd.argoproj.io/sync-wave: "1"
    spec:
      project: koprogo
      sources:
        - repoURL: https://github.com/gilmry/koprogo.git
          targetRevision: "{{ .branch }}"
          ref: values
        - repoURL: https://github.com/gilmry/koprogo.git
          targetRevision: "{{ .branch }}"
          path: infrastructure/_shared/helm/koprogo
          helm:
            valueFiles:
              # 1. Cluster profile (storage class, ingress class, TLS, secrets backend)
              - $values/infrastructure/_shared/cluster-profiles/${CLUSTER_TYPE}.yaml
              # 2. Per-env business config (replicas, log level, image tag)
              - $values/{{ .envValuesFile }}
      destination:
        server: https://kubernetes.default.svc
        namespace: "{{ .namespace }}"
      syncPolicy:
        syncOptions:
          - CreateNamespace=true
          - ServerSideApply=true
        retry:
          limit: 5
          backoff:
            duration: 5s
            factor: 2
            maxDuration: 3m
      ignoreDifferences:
        - group: apps
          kind: Deployment
          jsonPointers:
            - /spec/replicas
      revisionHistoryLimit: 10
  templatePatch: |
    spec:
      syncPolicy:
        automated:
          prune: {{ .prune }}
          selfHeal: {{ .autoSync }}
