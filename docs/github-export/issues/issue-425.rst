===============================================================================================
Issue #425: Méta — Garde-fous IA: audit qualité+sécurité IaC, cause racine, plan de remédiation
===============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: documentation,track:infrastructure priority:critical,security
:Assignees: Unassigned
:Created: 2026-04-29
:Updated: 2026-04-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/425>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Ce dépôt est volontairement une **expérimentation d'industrialisation de la production de code par agents IA** (Claude.ai cowork, Claude Code, agents distants type `claude/bold-burnell`, `claude/eager-cray`) **avec garde-fous et validations humaines aux bons moments**.
   
   L'absence actuelle de garde-fous au niveau de l'IA est délibérée : on observe en grandeur réelle ce que produit un agent sans filet, pour définir ensuite les **recettes** (hooks, deny lists, skills, points de validation humaine) qui empêchent les pathologies observées.
   
   Cette issue acte le **constat** (audit du 2026-04-29), la **cause racine** (absence volontaire de garde-fous IA), et le **plan de remédiation** (les recettes à industrialiser).
   
   ---
   
   ## 1. Constats — audit du 2026-04-29
   
   ### 1.1 Qualité backend (Rust)
   - Architecture hexagonale revendiquée mais ~60 % implémentée. Entités à champs `pub` (ex. `expense.rs` 1245 LOC, 26 `pub`) → invariants contournables après construction.
   - **`Result<T, String>` partout** (~80 % des ports/use cases) ; `thiserror` et `anyhow` importés mais utilisés sur 3 ports seulement.
   - **1 967 occurrences `.unwrap()` / `.expect()`** dans `backend/src/` (production). Chaque ligne est un `panic!` potentiel.
   - **8 fichiers > 1 000 LOC** : `seed.rs` 4 152, `mcp_sse_handlers.rs` 2 010, `local_exchange_use_cases.rs` 1 530, `gamification_use_cases.rs` 1 427, `expense.rs` 1 245, `payment_use_cases.rs` 1 179, `shared_object_use_cases.rs` 1 111, `convocation_use_cases.rs` 1 098.
   - 84 migrations dont **13 le 2026-03-23** en batch monolithique (rollback partiel impossible).
   - `tracing` + `log` en dual-stack, logging fragmenté.
   - 7 `#[ignore]` sur 1 191 tests : ratio sain en surface, mais aucun test sur les use cases monstres listés.
   
   ### 1.2 Qualité frontend (Astro + Svelte)
   - 🔴 **JWT en `localStorage`** (`frontend/src/stores/auth.ts:16-37`) → vol session sur la moindre XSS.
   - 🔴 88 pages Astro toutes en `client:load` → SSG/SSR contourné, anti-pattern Astro complet.
   - Composants monstres : `InvoiceWorkflow.svelte` 887, `AdminGdprPanel.svelte` 660, `InvoiceForm.svelte` 633, `UserForm.svelte` 591, `GdprDataPanel.svelte` 569.
   - 18 modals quasi-copiés (`UserCreateModal`, `OwnerCreateModal`, `UnitCreateModal`…), aucune abstraction.
   - Migration Svelte 5 incomplète : 16 `fetch()` directs malgré `lib/api.ts` (443 LOC) ; commit `f89954e` corrige des `// Svelte 5 runes mode` placés hors du `<script>` (crash silencieux).
   - Tests frontend = **13 / 181 composants ≈ 7 %**. Aucun test sur les composants critiques (Invoice, GDPR, UserForm).
   - TS strict activé mais `payload: any` toléré dans tous les forms admin ; `as any` × 5.
   - i18n 99 % réel mais CLAUDE.md affiche 73 % → désinformation marketing.
   
   ### 1.3 Sécurité IaC (le plus grave)
   
   **P0 — bloquants** :
   - Secrets en clair versionnés : `infrastructure/_shared/helm/koprogo/values.yaml:111-114` (`postgresPassword: koprogo123`, `jwtSecret: changeme-to-32-chars-minimum`, `s3SecretKey: koprogo123`), recopiés dans **8 environnements** (`monosite/k3s/{env}/helm-values.yaml`, `multisite/k8s/{env}/helm-values.yaml`).
   - `infrastructure/_shared/helm/velero/values.yaml:30-33` : creds AWS S3 backups en clair (`aws_secret_access_key=changeme`).
   - `infrastructure/_shared/helm/monitoring/values.yaml:20` : Grafana `adminPassword: changeme`.
   - `infrastructure/_shared/helm/vault/values.yaml:17` : `tls_disable = 1` → secrets en clair sur le réseau cluster.
   - `infrastructure/_shared/secrets/.sops.yaml` : clé `age1placeholder_replace_with_your_public_key` → SOPS configuré sur **placeholder** = fausse sécurité plus dangereuse que pas de chiffrement.
   - `infrastructure/_shared/terraform/modules/ovh-k3s/main.tf:56` : SSH `0.0.0.0/0` sur **8 environnements**.
   - 12 fichiers `backend.tf` S3 sans `encrypt`, sans DynamoDB lock, sans KMS → tfstate en clair (contient IPs, outputs, parfois passwords interpolés).
   - `infrastructure/_shared/helm/monitoring/values.yaml:73-86` : Elasticsearch `xpack.security.enabled: false`, Kibana bind `0.0.0.0` → tous logs (avec PII GDPR) accessibles sans auth.
   
   **P1 — haut risque** :
   - Kubeconfig admin en `0644` (`_shared/ansible/roles/k3s-master/tasks/main.yml:10`).
   - K3s `NODE_TOKEN` passé en var d'env via `curl ... | sh` (visible `/proc/`, debug logs Ansible) — `_shared/ansible/roles/k3s-agent/tasks/main.yml:6-8`.
   - Outputs Terraform sans `sensitive = true` (kubeconfig path, `master_ip_public`, `kubectl_command`) — `_shared/terraform/modules/ovh-k3s/outputs.tf:19-34`.
   - MinIO + Postgres StatefulSet **sans `securityContext`** (root par défaut) — `_shared/helm/koprogo/templates/`.
   - `_shared/helm/koprogo/templates/security/network-policy.yaml:41` : `egress: - {}` (tout autorisé) avec commentaire "restrict further in production" jamais fait.
   - ArgoCD `autoSync: true` + `prune: true` **y compris en production** — `_shared/argocd/applicationset.yaml:15-16,81-82`.
   - Tags `:latest` sur backend / frontend / minio — `_shared/helm/koprogo/values.yaml:7,40,93`.
   - `CORS_ALLOWED_ORIGINS: "*"` au niveau Helm (alors que CLAUDE.md affirme "no wildcards") — `_shared/helm/koprogo/values.yaml:21`.
   - ExternalSecrets sur HTTP (`http://vault.vault:8200`) — `_shared/helm/koprogo/templates/external-secret.yaml:41`.
   - Aucun `prevent_destroy` sur volumes Postgres prod ni instances K3s.
   - User `koprogo` dans groupe `docker` **sans userns-remap** → équivalent root container.
   - Vault audit log non configuré.
   
   ### 1.4 Cohérence claims vs réalité (CLAUDE.md)
   - "137 k+ LOC Rust" → mesuré **29 726** (×4.6 inflation).
   - "Jalon 0 ✅ COMPLÉTÉ" → **74 issues OPEN** dont #220-237, #295-317 (Jalon 0).
   - v0.1.0 release mais commit `49f8a2a` corrige bouton créer-ticket bloqué + vote panel non gated par ownership (faille RBAC élémentaire).
   - `docs/HUMAN_REVIEW_REPORT_v0.1.0.md` conclut lui-même "NO-GO pour release publique".
   - CLAUDE.md = 1469 lignes, 92 ✅ NOUVEAU (journal de release marketing, pas un guide d'agent).
   
   ### 1.5 CI/CD et dépôt
   - `docker-compose.yml` : `koprogo123` versionné en clair (pollue tous les forks).
   - Playwright en CI loggue "Some scenarios failed (non-blocking for CI)" → failures masquées.
   - `dtolnay/rust-toolchain@stable` (floating tag, non reproductible).
   - 28 branches `dependabot/*` zombies sur origin.
   - 4.5 Mo de `.docx`/`.pdf` versionnés au root (cf issue de nettoyage docs).
   - Doublons docs (CONVOCATIONS, NOTIFICATIONS) — cf issue de nettoyage docs.
   - `.env.loadtest` committé avec `JWT_SECRET=load-test-secret-key-not-for-production`.
   
   ---
   
   ## 2. Cause racine — manque de garde-fous IA volontaire
   
   **Le pipeline de production de code par agents IA n'a aucun garde-fou actif** :
   
   - `.claude/settings.json` ne contient **aucun hook**, **aucune `deny` list**, juste un allow-list `WebFetch` + quelques Bash. → l'agent peut écrire dans `**/.env`, `**/values.yaml`, `**/secrets/**` sans aucune friction.
   - **pas de `gitleaks`** (ni en hook Claude Code, ni en pre-commit, ni en CI bloquante).
   - **pas de scan IaC** (tfsec, checkov, kube-linter, helm lint en CI).
   - **pas de skills locaux** (`.claude/skills/` n'existait pas).
   - **pas de sous-agents** spécialisés (`.claude/agents/` n'existait pas).
   - **pas de slash commands** matérialisant les checkpoints (`.claude/commands/` n'existait pas).
   - **pas de fichier de règles** injecté à chaque prompt (`UserPromptSubmit`).
   - **`.claude/hooks.md`** documente un format **obsolète** (`pre-commit:` / `post-commit:`) qui ne correspond plus aux events Claude Code actuels (`PreToolUse` / `PostToolUse` / `Stop` / `UserPromptSubmit` / `SessionStart`) — la doc rassure sans rien protéger.
   
   **Pathologies observées comme conséquence directe** :
   - Agents IA qui écrivent `:latest` partout (pas de provenance imposée).
   - Agents IA qui copient `koprogo123` dans 8 environnements (pas de detect-and-block).
   - Agents IA qui ajoutent `unwrap()` à 1 967 occurrences (pas de PostToolUse warning).
   - Agents IA qui produisent `Result<_, String>` partout (pas de règle CRITICAL injectée).
   - Agents IA qui dupliquent 18 modals (pas de skill `extract-shared-component`).
   - v0.1.0 release avec un bouton de création de ticket bloqué et un vote panel non gated (pas de checkpoint humain matérialisé avant tag).
   
   C'était **délibéré** (expérimentation), mais le constat est posé : **sans hooks ni deny ni skills, l'agent optimise pour "compile + lint passent", pas pour "code sûr et propre"**.
   
   ---
   
   ## 3. Garde-fous à industrialiser — 4 couches
   
   ### L1 — Permissions `.claude/settings.json`
   - **deny** : `terraform apply`, `terraform destroy`, `helm upgrade`, `helm install`, `kubectl apply/delete/exec/patch`, `argocd app sync/delete`, `git push --force`, `git commit --no-verify`, `rm -rf`, `curl|sh`, `wget|sh`, écriture dans `**/.env`, `**/secrets/**`, `**/*.pem`, `**/*.key`, `**/*.tfstate*`, `**/age.key`, `**/.vault_pass`, `**/kubeconfig`, `**/id_rsa`, `**/id_ed25519`.
   - **ask** : `git push`, `git commit`, `git rebase/merge`, `gh pr create/merge`, `make migrate`, édition `backend/migrations/**`, `infrastructure/**/*.tf`, `infrastructure/**/values.yaml`, `.github/workflows/**`, `CLAUDE.md`, `docker-compose.yml`, `Cargo.toml`, `package.json`, `seed.rs`, `Dockerfile`.
   - **allow** : commandes routinières en lecture/test (lint, fmt, test, plan, helm template/lint, kubectl get/describe, gh issue/pr list, etc.).
   
   ### L2 — Hooks `.claude/hooks/`
   - `PreToolUse Edit|Write` : bloque écriture vers chemin sensible **OU** contenu détecté comme secret (AWS access key, GitHub PAT, Slack token, PEM, mot de passe hardcodé sans placeholder reconnu).
   - `PreToolUse Bash` : seconde ligne contre `terraform apply`, `helm upgrade`, etc. (defense in depth).
   - `PostToolUse Edit|Write` : auto-format selon extension (`cargo fmt`, `prettier`, `terraform fmt`) ; warning si `unwrap()`/`expect()` introduit dans un `.rs` ; warning si nouveau `: any` ou `as any` en `.ts`/`.svelte`.
   - `UserPromptSubmit` : injecte `.claude/rules/CRITICAL.md` (résumé top-10 règles) au début du contexte.
   - `Stop` : `gitleaks detect --staged --no-banner` + `git diff --cached --check` ; bloque la fin de tour si fuite détectée.
   - `SessionStart` : vérifie deps (`gitleaks`, `gh`, `cargo`, `npm`, `tfsec`, `kube-linter`), warn si branche actuelle ∈ {`main`, `production`, `staging`}, affiche bandeau "guardrails actifs".
   
   ### L3 — Skills + sous-agents + slash commands
   
   **Skills** (`.claude/skills/`) :
   - `safe-iac-change` : impose plan diff + checklist sécurité (TLS, securityContext, NetworkPolicy, secrets) avant tout edit IaC.
   - `hexagonal-feature` : génère une feature en respectant Domain → Port → Use Case → Adapter → Handler avec `AppError` typé (pas `Result<_, String>`) + tests.
   - `human-checkpoint` : matérialise une pause "j'attends ta validation" sur les points critiques (avant push, avant migration, avant changement RBAC).
   - `bdd-e2e-pair` : impose la création BDD + E2E couplée pour toute nouvelle user-facing behavior.
   - `secure-component` : pour les nouveaux composants Svelte, force httpOnly cookie + checklist i18n 4 langues.
   
   **Sous-agents** (`.claude/agents/`) :
   - `security-iac-reviewer` : revue sécurité ciblée sur diff IaC, tfsec/checkov/kube-linter en série.
   - `unwrap-fixer` : remplace `unwrap()`/`expect()` par `AppError` typé avec contexte.
   - `hexagonal-purity-checker` : détecte fuites infra (`use sqlx`/`use actix`) dans `domain/`, fields `pub` non justifiés.
   - `i18n-coverage-checker` : compare clés FR/NL/EN/DE et signale les manquantes.
   
   **Slash commands** (`.claude/commands/`) :
   - `/check-quality` : lint + tests rapides + gitleaks staged + svelte-check.
   - `/secret-scan` : gitleaks full-history + audit `git ls-files` filtré.
   - `/human-review` : checkpoint humain explicite (l'agent affiche diff résumé + risques + demande approbation).
   - `/safe-pr` : crée la PR avec checklist gating (gitleaks ok, tests ok, CHANGELOG mis à jour, pas de `:latest`).
   - `/iac-plan` : `terraform plan` + `tfsec` + `helm template ... | kube-linter` agrégés.
   
   ### L4 — Outillage
   - `.gitleaks.toml` (config + allowlist documentée pour faux positifs).
   - `Makefile` : cibles `secret-scan`, `iac-lint` (tfsec + checkov + kube-linter + helm lint + ansible-lint), `claude-check` (validation des hooks et permissions).
   - CI bloquante : retirer le "non-blocking" Playwright, ajouter `gitleaks-action`, `tfsec-action`, `trivy fs`, `checkov-action`.
   - `pre-commit` framework en complément des hooks Claude Code, pour les humains qui n'utilisent pas l'agent.
   - `.gitignore` : `*.tfstate*`, `**/age.key`, `**/.vault_pass`, `.claude/worktrees/`, `.claude/settings.local.json`.
   
   ---
   
   ## 4. Critères d'acceptation
   
   - [ ] `.claude/settings.json` contient `deny`/`ask`/`allow` et 5 events de hooks (`PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Stop`, `SessionStart`).
   - [ ] 8 scripts `.claude/hooks/*.sh` exécutables et testés à vide (exit 0 sur input neutre).
   - [ ] `gitleaks detect` bloque effectivement un commit contenant `AKIA...` (test reproductible).
   - [ ] Tentative d'`Edit` sur `backend/.env` retournée bloquée par hook (`exit 2`).
   - [ ] Tentative de `Bash(terraform apply)` refusée par `permissions.deny`.
   - [ ] `cargo clippy -W clippy::unwrap_used -W clippy::expect_used` activé en CI (warning d'abord, bloquant après remédiation).
   - [ ] `.claude/hooks.md` (obsolète) supprimé ou remplacé par `.claude/AGENT_GUARDRAILS.md` à jour.
   - [ ] `.gitignore` couvre `*.tfstate*`, `**/age.key`, `**/.vault_pass`, `.claude/worktrees/`.
   - [ ] 4-5 skills locaux + 3-4 sous-agents + 4-5 slash commands matérialisés et documentés dans `AGENT_GUARDRAILS.md`.
   - [ ] CI bloquante : Playwright sans "non-blocking", `gitleaks-action`, `tfsec`, `trivy fs`, `dependency-review-action`.
   - [ ] Une PR de test prouve le flux : agent commence → hook bloque secret → checkpoint humain → push autorisé.
   
   ---
   
   ## 5. Priorisation
   
   | Sprint | Livrables | Effet attendu |
   |---|---|---|
   | **S1 — stop the bleed** | `settings.json` deny/ask, hooks `PreToolUse` secret-write + `Stop` gitleaks, `.gitleaks.toml`, `.gitignore` | L'agent ne peut plus introduire un nouveau secret en clair. |
   | **S2 — qualité durable** | hooks `PostToolUse` fmt + warn-unwrap, slash commands `/check-quality` `/human-review`, skills `safe-iac-change` + `human-checkpoint` | L'agent corrige spontanément, demande validation aux moments-clés. |
   | **S3 — sous-agents + CI** | sous-agents `security-iac-reviewer` `unwrap-fixer` `hexagonal-purity-checker`, CI `gitleaks/tfsec/trivy` bloquants, `Makefile` targets | Vérification multi-étages indépendante de l'agent. |
   | **S4 — remédiation legacy** | rotation des secrets fuités, `:latest` → digest, `unwrap` → `AppError`, JWT cookie httpOnly, `.env.loadtest` purgé de l'historique | Solde la dette d'avant garde-fous. |
   
   ---
   
   ## 6. Liens
   
   - Audit conversation Claude Code 2026-04-29 (qualité backend/frontend/CI/IaC).
   - `docs/HUMAN_REVIEW_REPORT_v0.1.0.md` (NO-GO release).
   - `infrastructure/SECURITY.md` (couche périmètre, à compléter par couche IaC).
   - `.claude/hooks.md` (doc obsolète à remplacer).
   - Recettes existantes ailleurs : `scripts/install-hooks.sh` (Git hooks bash), `Makefile` (target `make ci`).
   - Issue compagnon : nettoyage docs / archivage binaires.
   
   ---
   
   🤖 Issue générée par Claude Opus 4.7 (1M context) après audit multi-agents.

.. raw:: html

   </div>

