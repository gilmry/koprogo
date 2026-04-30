---
persona: devops-engineer
created: 2026-04-29
last_updated: 2026-04-29
---

# Mémoire persistante — `devops-engineer`

> Cette mémoire est lue à chaque invocation du persona pour assurer la continuité inter-sprints et la cohérence des décisions.

## Décisions architecturales acceptées

- **GitOps via ArgoCD** comme moteur de déploiement (cf. `infrastructure/_shared/argocd/`).
- **Production : `syncPolicy.automated: false`** obligatoire (cf. #429). Sync prod = workflow_dispatch ou UI MFA.
- **Image policy** : digest pinning obligatoire (`@sha256:...`), jamais `:latest` (cf. CRITICAL.md ligne rouge).
- **Pre-release** : 12 étapes auto-doc-refresh avant `git tag` (#428 §8bis).

## Findings audit 2026-04-29 à corriger

- ⚠️ ArgoCD `autoSync: true` + `prune: true` détecté en production (`_shared/argocd/applicationset.yaml:15-16,81-82`). À retirer.
- ⚠️ Tags `:latest` sur backend/frontend/minio (`_shared/helm/koprogo/values.yaml:7,40,93`).
- ⚠️ `dtolnay/rust-toolchain@stable` (floating tag) en CI — pinner par SHA.
- ⚠️ Playwright en CI loggue "Some scenarios failed (non-blocking)" — silencing failures, à corriger.

## Conventions CI/CD acceptées

- Workflows GH Actions : permissions minimales par job (`contents: read` par défaut).
- Caches : `actions/cache@v4` par hash de `Cargo.lock` / `package-lock.json`.
- Matrix builds limités à OS supportés (Linux x86_64 prioritaire).
- Runs concurrency cancel sur push (éviter doublons).

## Décisions en attente

- RFC : `terraform-apply-production.yml` workflow (S3 #429).
- RFC : retirer `autoSync` ArgoCD prod + processus de sync manuel formel.
- ADR : politique digest pinning (qui décide du digest, comment on le bumpe).

## Lessons learned

- (à enrichir après chaque sprint)

## Anti-patterns à signaler en review PR

- Workflow GH avec `permissions: write-all` ou pas de section `permissions:` (défaut trop large).
- Action third-party non pinnée par SHA (juste `@v4` accepte version drift).
- `continue-on-error: true` sans justification documentée.
- Workflow qui invoque `terraform apply` sans `environment` GH avec required reviewers.

## Liens

- [`.claude/agents/devops-engineer.md`](devops-engineer.md) — system prompt
- [`.claude/AGENT_GUARDRAILS.md`](../AGENT_GUARDRAILS.md)
- Issues : [#425](https://github.com/gilmry/koprogo/issues/425), [#429](https://github.com/gilmry/koprogo/issues/429)
