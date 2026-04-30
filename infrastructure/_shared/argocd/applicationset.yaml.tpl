# ApplicationSet template — rendered by gitops-bootstrap.sh via envsubst.
#
# Substitutes ${CLUSTER_TYPE} from the env var set at bootstrap time
# (auto-detected from kubectl context or passed as CLI arg).
#
# Stacks Helm values in the order:
#   1. infrastructure/_shared/helm/koprogo/values.yaml         (chart defaults — implicit)
#   2. infrastructure/_shared/cluster-profiles/${CLUSTER_TYPE}.yaml  (cluster overrides)
#   3. infrastructure/monosite/k3s/{{environment}}/helm-values.yaml  (env business config)
#
# This is the cluster-agnostic version. The legacy applicationset.yaml is kept
# in parallel for backward compat until validation is complete (see PR-A/PR-B).

# ApplicationSet 1: Infrastructure (Kustomize — unchanged from legacy)
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: koprogo-infra
  namespace: argocd
spec:
  generators:
    - list:
        elements:
          - branch: production
            environment: production
            namespace: koprogo-production
            kustomizePath: infrastructure/monosite/k3s/production/kustomize
            autoSync: "true"
            prune: "true"
          - branch: staging
            environment: staging
            namespace: koprogo-staging
            kustomizePath: infrastructure/monosite/k3s/staging/kustomize
            autoSync: "true"
            prune: "true"
          - branch: integration
            environment: integration
            namespace: koprogo-integration
            kustomizePath: infrastructure/monosite/k3s/integration/kustomize
            autoSync: "true"
            prune: "false"
          - branch: dev
            environment: dev
            namespace: koprogo-dev
            kustomizePath: infrastructure/monosite/k3s/dev/kustomize
            autoSync: "false"
            prune: "false"
  template:
    metadata:
      name: "koprogo-infra-{{environment}}"
      namespace: argocd
      labels:
        app.kubernetes.io/name: koprogo
        component: infrastructure
        environment: "{{environment}}"
    spec:
      project: koprogo
      source:
        repoURL: https://github.com/gilmry/koprogo.git
        targetRevision: "{{branch}}"
        path: "{{kustomizePath}}"
      destination:
        server: https://kubernetes.default.svc
        namespace: "{{namespace}}"
      syncPolicy:
        automated:
          prune: "{{prune}}"
          selfHeal: "{{autoSync}}"
        syncOptions:
          - CreateNamespace=true
          - PrunePropagationPolicy=foreground
        retry:
          limit: 5
          backoff:
            duration: 5s
            factor: 2
            maxDuration: 3m
      revisionHistoryLimit: 10
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
  generators:
    - list:
        elements:
          - branch: production
            environment: production
            namespace: koprogo-production
            envValuesFile: infrastructure/monosite/k3s/production/helm-values.yaml
            autoSync: "true"
            prune: "true"
          - branch: staging
            environment: staging
            namespace: koprogo-staging
            envValuesFile: infrastructure/monosite/k3s/staging/helm-values.yaml
            autoSync: "true"
            prune: "true"
          - branch: integration
            environment: integration
            namespace: koprogo-integration
            envValuesFile: infrastructure/monosite/k3s/integration/helm-values.yaml
            autoSync: "true"
            prune: "false"
          - branch: dev
            environment: dev
            namespace: koprogo-dev
            envValuesFile: infrastructure/monosite/k3s/dev/helm-values.yaml
            autoSync: "false"
            prune: "false"
  template:
    metadata:
      name: "koprogo-app-{{environment}}"
      namespace: argocd
      labels:
        app.kubernetes.io/name: koprogo
        component: application
        environment: "{{environment}}"
        cluster-type: "${CLUSTER_TYPE}"
    spec:
      project: koprogo
      sources:
        - repoURL: https://github.com/gilmry/koprogo.git
          targetRevision: "{{branch}}"
          ref: values
        - repoURL: https://github.com/gilmry/koprogo.git
          targetRevision: "{{branch}}"
          path: infrastructure/_shared/helm/koprogo
          helm:
            valueFiles:
              # 1. Cluster profile (storage class, ingress class, TLS, secrets backend)
              - $values/infrastructure/_shared/cluster-profiles/${CLUSTER_TYPE}.yaml
              # 2. Per-env business config (replicas, log level, image tag)
              - $values/{{envValuesFile}}
      destination:
        server: https://kubernetes.default.svc
        namespace: "{{namespace}}"
      syncPolicy:
        automated:
          prune: "{{prune}}"
          selfHeal: "{{autoSync}}"
        syncOptions:
          - CreateNamespace=true
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
