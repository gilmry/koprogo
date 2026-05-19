# ApplicationSet — Prerequisites (sync wave -1)
#
# Installs cluster-wide prerequisites that the koprogo-infra and koprogo-app
# ApplicationSets depend on. Issue #515 (Gap 1): without this, the kustomize
# layer fails on first sync because Traefik CRDs (Middleware) are absent.
#
# Substitutes ${CLUSTER_TYPE} at bootstrap time (envsubst), matching the sister
# applicationset.yaml.tpl convention.
#
# IMPORTANT:
#   - On `k3s-self-hosted`: k3s ships Traefik bundled in kube-system. Applying
#     this AppSet would create a SECOND Traefik install — DO NOT APPLY on k3s.
#     The bootstrap script gitops-bootstrap.sh guards on cluster type.
#   - On `docker-desktop` and `k8s-managed`: apply this AppSet first, wait for
#     Healthy, then apply the main applicationset.yaml.
#
# Traefik chart version is pinned for reproducibility. Bump deliberately via PR.

apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: koprogo-prerequisites
  namespace: argocd
  annotations:
    koprogo.io/cluster-type: "${CLUSTER_TYPE}"
spec:
  goTemplate: true
  goTemplateOptions: ["missingkey=error"]
  generators:
    - list:
        elements:
          - component: traefik
            namespace: traefik-system
            chartRepo: https://traefik.github.io/charts
            chartName: traefik
            # Traefik 3.x — uses `traefik.io/v1alpha1` API group (matches the
            # migration in _shared/kustomize/base/ingress.yaml, Issue #515 Gap 2).
            chartVersion: "32.1.0"
  template:
    metadata:
      name: "koprogo-prereq-{{ .component }}"
      namespace: argocd
      labels:
        app.kubernetes.io/name: koprogo
        component: prerequisite
        prerequisite: "{{ .component }}"
        cluster-type: "${CLUSTER_TYPE}"
      annotations:
        # Sync wave -1 — must converge before infra (wave 0) and app (wave 1).
        # See applicationset.yaml.tpl for the cross-Application ordering note:
        # this is a soft hint; in practice it ensures CRDs are installed early
        # in the bootstrap sequence.
        argocd.argoproj.io/sync-wave: "-1"
    spec:
      project: koprogo
      source:
        repoURL: "{{ .chartRepo }}"
        chart: "{{ .chartName }}"
        targetRevision: "{{ .chartVersion }}"
        helm:
          releaseName: traefik
          values: |
            # Minimal Traefik install — only what koprogo's kustomize base needs:
            # IngressClass `traefik`, plus Middleware CRD.
            providers:
              kubernetesCRD:
                enabled: true
                allowCrossNamespace: true
              kubernetesIngress:
                enabled: true
                publishedService:
                  enabled: true
            ingressClass:
              enabled: true
              isDefaultClass: true
              name: traefik
            # No LB on Docker Desktop — ClusterIP + NodePort is enough for
            # sandbox. On k8s-managed, override via cluster-profile.
            service:
              type: ClusterIP
            ports:
              web:
                expose:
                  default: true
              websecure:
                expose:
                  default: true
            # Sandbox defaults; cluster-profile may scale up resources.
            resources:
              requests:
                cpu: 100m
                memory: 128Mi
              limits:
                cpu: 500m
                memory: 256Mi
      destination:
        server: https://kubernetes.default.svc
        namespace: "{{ .namespace }}"
      syncPolicy:
        automated:
          prune: true
          selfHeal: true
        syncOptions:
          - CreateNamespace=true
          - ServerSideApply=true  # required for large Traefik CRDs
        retry:
          limit: 5
          backoff:
            duration: 5s
            factor: 2
            maxDuration: 3m
      revisionHistoryLimit: 10
