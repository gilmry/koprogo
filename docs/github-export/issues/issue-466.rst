=========================================================================================================
Issue #466: RFC: Stratégie GitOps multi-environnement — branches infra/* + main + ApplicationSet refactor
=========================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: documentation,track:infrastructure governance
:Assignees: Unassigned
:Created: 2026-05-01
:Updated: 2026-05-13
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/466>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Suite aux PRs #465 (gitops bootstrap unblock) et la PR en cours (kustomize patches fix), on a découvert un **chicken-and-egg structurel** dans le flow GitOps actuel :
   
   - Les `ApplicationSet`s ArgoCD (`koprogo-infra`, `koprogo-app`) ciblent les branches `dev`, `staging`, `integration`, `production` via `targetRevision: {{ .branch }}`.
   - Les fichiers d'infra (`cluster-profiles/`, `kustomize/base/`, `helm/`, `monosite/k3s/<env>/`) vivent **uniquement** sur `feature/dev`.
   - Conséquence directe : sur le cluster docker-desktop, les 8 Applications générées sont en `SYNC=Unknown` car ArgoCD checkout `dev` → fichiers absents → manifest generation fail :
     - `koprogo-app-*` : `Error: open .../cluster-profiles/docker-desktop.yaml: no such file or directory`
     - `koprogo-infra-*` : (avant fix) patch kustomize `[noNs]` ne match pas + `commonLabels` deprecated
   
   Le diagnostic complet est dans la conversation Maury récente (validation cluster docker-desktop, 1er mai 2026).
   
   ## Problème à trancher
   
   > **Comment l'infra GitOps (kustomize, helm, cluster-profiles, ApplicationSet) doit-elle être propagée aux environnements, et qui en a la source de vérité ?**
   
   Le besoin réel : permettre à un dev de modifier l'infra (ex: bump une image, ajouter un cluster-profile, modifier un patch ingress) sans devoir merger toute l'intégration `feature/dev` sur 4 branches d'env. Et inversement, qu'une feature applicative ne soit pas bloquée par un infra broken.
   
   ## Tenants (ce que la solution doit garantir)
   
   1. **Itérable rapidement** : un infra fix (ex: la PR templatePatch booleans) ne doit pas attendre 4 PR successives sur 4 branches.
   2. **Traçable** : chaque change infra a une PR, un reviewer, des tests CI dédiés. Pas de cherry-pick silencieux.
   3. **Testable en CI** : `kubectl kustomize`, `helm template`, validation schémas (kubeconform), security scan (kubescape/polaris) — ces checks doivent tourner sur **chaque** PR infra, **rapidement** (< 2 min), sans déclencher la suite Rust+Playwright.
   4. **Sécurisé** : modifications de `production/helm-values.yaml` (image tag, replicas, secrets) doivent passer un environment approval GitHub.
   5. **Rollback explicite** : revenir à N-1 doit être un git revert + sync, pas un cherry-pick complexe.
   6. **Découplé du cycle applicatif** : une feature dev en cours ne doit pas embarquer l'infra ; un fix infra ne doit pas embarquer 30 commits dev.
   7. **Compatible avec les règles CLAUDE.md** : pas d'opération prod autonome (#2), pas de `--no-verify` (#1), tier 1/2 respecté (#11).
   
   ## Aboutissants (implications à analyser)
   
   ### Sur le modèle de branches
   - Si on adopte des branches `infra/*` distinctes des `feature/*`, où mergent-elles ? `main` ? `feature/dev` ? Une nouvelle branche `infra/main` ?
   - Que deviennent les branches d'env (`dev`, `staging`, `integration`, `production`) ? Source de vérité ? Miroir de `main` ? Suppression ?
   - Comment se synchronise l'image tag d'une release applicative avec un commit d'infra qui bump le tag ?
   
   ### Sur l'ApplicationSet
   - Si `main` devient source unique pour l'infra : `targetRevision: main`, et l'identité de l'env devient un **path** (`infrastructure/monosite/k3s/<env>/`) ou un **values file**, pas une branche.
   - Si on garde les branches d'env mais reformulées comme "branches de promotion" (un dev fait PR vers `dev`, un release manager fast-forward `dev → staging → integration → production` après validation) — alors les ApplicationSets restent OK mais le flow dev change radicalement.
   - Configuration multi-source ArgoCD (chart à un endroit, values à un autre) — déjà utilisée dans `koprogo-app` mais pas `koprogo-infra`.
   
   ### Sur la CI
   - Workflow dédié `ci-infra.yml` : trigger `branches: ['infra/**', 'main']` + `paths: ['infrastructure/**']`.
   - Steps proposés : kubectl kustomize × 4 envs ; helm template × (4 envs × 3 cluster-profiles = 12 combos) ; kubeconform ; kubescape ; `gitops-bootstrap.sh --dry-run` (à créer).
   - Coexistence avec `make ci` actuel : ne pas tout doubler, juste skip les steps Rust/Playwright sur paths infra-only.
   
   ### Sur la promotion d'image
   - Aujourd'hui : tag fixé statiquement dans `monosite/k3s/<env>/helm-values.yaml` (à confirmer).
   - Demain : workflow auto qui ouvre une PR `infra/bump-image-{env}-{tag}` après chaque release ? Avec auto-merge sur dev/staging et environment approval sur production ?
   - Alternative : Argo Image Updater (controller K8s qui patch le helm-values directement). Pro : pas de PR. Con : pas de traçabilité git.
   
   ### Sur les RBAC / sécurité
   - Branche `infra/*` = qui peut la créer ? Qui peut la merger ?
   - Settings GitHub : branch protection `main` (required reviews, required checks ci-infra), environment protection `production` (required reviewers).
   - Secrets : qui a accès au repo `infrastructure/_shared/cluster-profiles/<sealed-secrets>` ?
   
   ### Sur la migration depuis l'état actuel
   - Étape 1 : merger les 2 PR pendantes (#465 mergée, kustomize-fix en cours sur `feature/dev`).
   - Étape 2 : bootstrap les branches d'env (cherry-pick infra-only, ou full merge `feature/dev` une fois).
   - Étape 3 : refactor ApplicationSet selon stratégie retenue (PR séparée, testable cluster docker-desktop d'abord).
   - Étape 4 : ci-infra.yml + branch protections.
   
   ## Alternatives à évaluer
   
   ### A. Status quo amélioré
   - Garder le modèle actuel (4 branches d'env + `feature/dev` intégration).
   - Ajouter un script ou workflow GH qui sync `feature/dev → dev → staging → integration → production` automatiquement.
   - ➕ Minimum de change.
   - ➖ Le couplage infra ↔ applicatif reste. Une feature dev en cours bloque un fix infra urgent.
   
   ### B. `infra/*` branches → `main` source unique pour l'infra (proposition Maury)
   - Branches `infra/*` mergent sur `main`.
   - ApplicationSet pointe sur `main`, identité env = path/values file.
   - ➕ Découplage net infra/apps. Un fix infra prend 1 PR, pas 4 sync.
   - ➖ Refactor ApplicationSet non trivial. Nécessite revoir les branches d'env (suppression ? rétention historique ?).
   
   ### C. `main` source unique pour TOUT (infra + apps), env via tags d'image
   - Plus de branches d'env du tout. `main` fait foi.
   - Promotion entre envs = bump du tag d'image dans `<env>/helm-values.yaml` via PR.
   - ➕ Modèle GitOps moderne le plus propre (cf. ArgoCD Autopilot, FluxCD).
   - ➖ Refactor le plus lourd. Doit régler : (a) review process, (b) rollback, (c) hotfix flow.
   
   ### D. Découpler en deux repos
   - Repo `koprogo` (apps) ↔ Repo `koprogo-infra` (manifests, helm, kustomize).
   - ApplicationSet pointe sur `koprogo-infra:main`.
   - ➕ Découplage maximal. Permissions différenciées.
   - ➖ Coordination cross-repo (image tag bump = PR sur 2 repos). Lourd pour un projet de la taille de KoproGo en v0.1.0.
   
   ### E. Ne rien faire (encore)
   - v0.1.0 n'est pas en prod (cf. règle CLAUDE.md #10). On reporte la décision.
   - ➕ 0 effort. Permet de stabiliser l'app d'abord.
   - ➖ La dette s'accumule. Chaque infra fix nécessite manuellement de propager aux branches d'env.
   
   ## Questions ouvertes (à trancher dans la RFC)
   
   1. **Source de vérité infra** : `main` (B/C/D) ou statu quo (A) ?
   2. **Branches d'env** : suppression (B/C), miroir auto de main (B/C), gardées pour rollback rapide ?
   3. **ApplicationSet** : refactor pour pointer sur `main` ? Multi-source ? Image Updater ?
   4. **Promotion image** : PR auto, manuelle, Image Updater ?
   5. **CI infra** : créer `ci-infra.yml` séparé ? Ou ajouter un job conditionnel à `ci.yml` actuel ?
   6. **Validation** : kubeconform suffit-il ? Kubescape ? Polaris ? Quels schémas K8s versions ?
   7. **Sécurité prod** : environment approval GH ? CODEOWNERS pour `infrastructure/monosite/k3s/production/` ?
   8. **Migration** : big-bang (1 PR refactor complet) ou incrémentale (cherry-pick infra-only progressif) ?
   9. **Timing** : maintenant (bloque la stack GitOps) ou après v0.1.0 stabilisée (CLAUDE.md #10) ?
   
   ## Lien avec le travail en cours
   
   - ✅ Merged : #465 (gitops bootstrap unblock — server-side apply + idempotent + templatePatch)
   - 🚧 En cours : branche `chore/fix-gitops-env-branches-targets` (kustomize patches target + commonLabels migration), 5 fichiers staged localement, commit en attente
   - 📋 Bloqué par cette RFC : finalisation de la stratégie de propagation aux branches d'env
   
   ## Critères de sortie de la RFC
   
   La RFC sera considérée comme "prête à code" lorsque :
   
   - Les 9 questions ouvertes ci-dessus auront une réponse arrêtée
   - L'alternative retenue (A-E) aura un plan d'implémentation décomposé en 3-5 PR successives
   - Les impacts CLAUDE.md (Tier 1/2, branches main/dev, hooks deny) auront été passés en revue
   - Un humain (mainteneur) aura signé la décision (label `accepted`)
   
   ## Références
   
   - PR #465 : https://github.com/gilmry/koprogo/pull/465
   - Template RFC : `docs/governance/rfc/template.rst`
   - CLAUDE.md règles : #2 (pas de prod autonome), #10 (v0.1.0 pas en prod), #11 (Tier 1/2)
   - ADR existants `docs/adr/0001..0044`
   
   ---
   
   *Issue créée pour cadrer la discussion avant la rédaction de la RFC formelle. Une fois les choix faits ici, la RFC sera rédigée dans `docs/governance/rfc/0001-gitops-multi-environment-strategy.rst` (ou numéro disponible).*

.. raw:: html

   </div>

