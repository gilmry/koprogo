---
date: 2026-05-01
persona: platform-engineer
session: gitops-multi-env-strategy-discovery
tier: 2 # diagnostic + proposals + comments only ; merges = Tier 1 humain
---

# Activity log — platform-engineer 2026-05-01

## Context

Suite directe de la PR #465 (gitops bootstrap unblock, mergée 2026-04-30) : validation du flow `gitops-bootstrap.sh docker-desktop` end-to-end sur cluster local. ArgoCD opérationnel, 8 Applications créées par les ApplicationSets, mais toutes en `SYNC=Unknown`. Diagnostic conduit avec l'humain ; mise à plat des problèmes structurels (chicken-and-egg branches d'env vs `feature/dev`).

## Actions (Tier 2)

| Action                                                                              | Cible                                                    | Tier |
| ----------------------------------------------------------------------------------- | -------------------------------------------------------- | ---- |
| Diagnostic ArgoCD `koprogo-infra-dev` (kustomize patch [noNs] error)                | cluster docker-desktop                                   | 2    |
| Diagnostic ArgoCD `koprogo-app-dev` (cluster-profile file missing on `dev` branch)  | cluster docker-desktop                                   | 2    |
| Création branche `chore/fix-gitops-env-branches-targets`                            | repo local                                               | 2    |
| Migration `commonLabels` → `labels:` (Kustomize 5+)                                 | `_shared/kustomize/base/kustomization.yaml` + 4 envs     | 2    |
| Ajout `target:` selector aux patches Ingress (résolution `[noNs]`)                  | 4 envs `monosite/k3s/<env>/kustomize/kustomization.yaml` | 2    |
| Validation `kubectl kustomize` × 4 envs (no errors, hosts patched correctement)     | local                                                    | 2    |
| Création issue #466 (RFC framing — alternatives A à E)                              | GitHub                                                   | 2    |
| Comment alternative F (hybride symétrique app/infra) sur #466                       | GitHub                                                   | 2    |
| Comment décisions verrouillées (alt F + 12 réponses + plan 7 PRs) sur #466          | GitHub                                                   | 2    |
| Rédaction RFC 0001 `docs/governance/rfc/0001-gitops-multi-environment-strategy.rst` | repo local (774 lignes)                                  | 2    |

## État branche `chore/fix-gitops-env-branches-targets`

5 fichiers staged, **non committés** (à la demande de l'humain — pause). Stash WIP créé pour les 3 deletions non-liées (`issues/important/028..030.md`).

## Décisions reportées (Tier 1 humain requis)

- **Stratégie GitOps multi-env** : décision en cours sur issue #466 (alternative A/B/C/D/E/F). Bloque la propagation des fixes infra aux branches d'env.
- **Commit + PR de la branche kustomize-fix** : en attente de relance utilisateur.
- **Rédaction de la RFC formelle** dans `docs/governance/rfc/XXXX-gitops-multi-environment-strategy.rst` : bloquée par les choix sur #466.

## Découvertes secondaires

- **Pre-push hook OOM** : `astro check` peut OOM si `playwright-report/` ou `test-results/` polluent le scan TypeScript (cas reproductible si on push deux fois consécutivement). Fix proposé en follow-up : `exclude` dans `frontend/tsconfig.json`. Pas inclus dans la PR en cours (scope creep).
- **Pre-commit hook auto-stage** : le hook lance `git add -u` après format, ce qui peut auto-stager des modifications non destinées à la PR. Mitigation : `git stash --keep-index` avant commit pour isoler les changes voulus.

## Liens

- PR #465 (mergée) : https://github.com/gilmry/koprogo/pull/465
- Issue #466 (RFC framing) : https://github.com/gilmry/koprogo/issues/466
- Comment alt-F : https://github.com/gilmry/koprogo/issues/466#issuecomment-4360851132
- Branche locale : `chore/fix-gitops-env-branches-targets` (5 fichiers staged, commit en attente)
