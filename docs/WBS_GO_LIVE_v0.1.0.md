# WBS — Mise en ligne KoproGo v0.1.0 (bêta privée fermée, VPS-first)

`WBS-v0.1.0-beta-r1` · 2026-05-16 · base : `feature/dev` + branche `story/521-C1-governance-decimal` (HEAD `98c1651`, 1 commit devant `origin/feature/dev`).

> **Provenance & portée** : artefact de planification (Tier 2, logué). Aucune issue/milestone GitHub créée (forme « document WBS versionné unique » choisie pour éviter le gate Tier-1). Les numéros d'issues sont *référencés*, pas créés.
>
> **Supersession** : ce document remplace, pour le go-live, les WBS périmés du 2026-04-01 — [`WBS_RELEASE_0_1_0.md`](WBS_RELEASE_0_1_0.md), [`WBS_BUGFIX_UI_v0.1.0.md`](WBS_BUGFIX_UI_v0.1.0.md), [`WBS_CORRECTIONS_v0.1.0.md`](WBS_CORRECTIONS_v0.1.0.md) — qui restent valables comme contexte historique mais ne reflètent plus l'état du code (Story A Decimal mergée, #515 mergé, TLS déjà câblé).

## Context

KoproGo v0.1.0 n'a jamais été taggé ni mis en ligne (aucun tag git, aucune release). Le besoin : **mettre v0.1.0 en ligne en bêta privée fermée (5-10 copropriétés)** avec une infrastructure opérationnelle réelle et tous les tests requis (4 catégories `@happy/@edge/@security/@negative`, RED-first). Le rapport de revue humaine `docs/HUMAN_REVIEW_REPORT_v0.1.0.md` date du **2026-04-01 (~6 semaines, périmé)** et concluait NO-GO public / GO-conditionnel bêta — il doit être **re-rejoué** sur le code courant, pas pris pour argent comptant (plusieurs bugs sont probablement déjà corrigés : #523 dashboard %, #521 Story A mergée).

Décisions produit verrouillées :
1. **Déploiement : VPS d'abord, puis k3s en Phase 2.** Phase 1 = OVH VPS + docker-compose (`infrastructure/monosite/vps/production`, poller systemd `gitops-deploy.sh`). k3s/ArgoCD = Phase 2 post-v0.1.0.
2. **Périmètre : bêta privée fermée.** Gate = bloquants critiques (sécurité réelle mais allégée vs gate public #427).
3. **Decimal : umbrella #433 COMPLÈTE** (EXP-003..008 + gouvernance C1 #521/#534/#525). Exactitude PCMN obligatoire même en bêta.
4. **Forme : document WBS versionné unique.** Pas de création d'issue/milestone GitHub maintenant (évite le gate Tier-1). Issues existantes référencées, non créées.

## Faits vérifiés (corrigent le chemin critique — ne pas re-dériver)

| Hypothèse initiale | Réalité vérifiée (code) | Impact |
|---|---|---|
| EXP-006 `journal_entry` = gros chantier PRIORITÉ MAX | **Déjà `Decimal`** : `journal_entry.rs:69-78` debit/credit `Decimal`, `BALANCE_TOLERANCE=dec!(0.011)`, validation débit==crédit exacte ; SQL déjà `DECIMAL(12,2)` + trigger DB | Long pole se réduit : reste `Result<_,String>`→`AppError` + BDD 4-cat. Pas de migration SQL. |
| EXP-005 `charge_distribution` à faire | Déjà `Decimal` (tolérances `dec!(1.0001)`/`dec!(0.01)`) | `Result→AppError` + BDD 4-cat seulement |
| #453 TLS = bloquant go-live | `infrastructure/monosite/vps/production/docker-compose.override.yml:10-40` **câble déjà Traefik + Let's Encrypt ACME HTTP-01** (443, redirect http→https, certresolver backend+frontend) ; `ACME_EMAIL` dans `.env.example` + ansible group_vars | **TLS PAS bloquant.** Go-live n'exige que : DNS A→IP VPS, ports 80/443 ouverts, `ACME_EMAIL` set. #453 (DNS-01 non-prod + SOPS/age) = Phase 2 |
| #515 5 gaps ArgoCD bloquants | **Mergés sur `origin/main`** (PR #516, `645b3cb`/`badb049`) — concernent k3s | Phase 2 |
| Migration gouvernance large | `20260516000000_alter_governance_to_numeric.sql` minimale/correcte : `units.quota` + `meetings.total/present_quotas`→`NUMERIC(10,4)`, idempotente, no-down, no prod data | DDL petite/sûre ; risque = call-sites + #525 ColumnDecode |
| EXP-007 `etat_date` mineur | `etat_date.rs` = **17 occ. f64** (plus gros résidu umbrella, doc légal Art. 577 CC) | WP-A5 = item le plus long de Track A |
| #443 cascade énorme | `bdd_financial.rs` ~23 occ. f64 (pas 120) + e2e_*.rs | Cascade réelle mais bornée |
| JWT stockage | `frontend/src/stores/auth.ts:139-141` (read) / `233-235` (write) / `169-192` (refresh) en `localStorage` | **Bloquant sécurité bêta fermée** confirmé |

**Vrai long pole = WP-A2 (#443 cascade tests) → WP-A5 (EXP-007 etat_date)**, à paralléliser avec la chaîne Ops-VPS (latence Tier-1 humaine).

## WBS Phase 1 — v0.1.0 EN LIGNE (VPS, bêta fermée)

Légende : Tier 1 = humain exécute (agent diagnostique/propose). Taille S≤0.5j / M 1-2j / L 3-5j. Tout livrable public : 4-cat RED-first, `Result<E,AppError>`, pas de `unwrap/expect` hors `#[cfg(test)]`, outillage via `docker compose run --rm`.

### Track A — Backend Decimal

- **WP-A1 — Clôture C1 gouvernance** · #534/#521-C1 ferme #525 · T2 · M · *critique, premier*
  Appliquer `backend/migrations/20260516000000_alter_governance_to_numeric.sql` (DB test) ; `bdd_governance` 4 scénarios VERTS ; tuer le panic #525 ColumnDecode `units.quota`. Décisions bloquantes : (a) **ADR-0008 ratio** pour proxy validation `vote.rs:~312-342` (draft `docs/agent-activity/2026-05-16-adr8-noncompliance-body.md`) ; (b) **politique f64 d'affichage** — `resolution.rs:185-212` `pour/contre/abstention_percentage()` renvoient `f64` depuis comptes entiers : **reco = carve-out ADR-0008 explicite (affichage seul, jamais seuil légal)** + assertion que le chemin quorum/majorité légal n'a aucun aller-retour Decimal→f64→Decimal. Fichiers : la migration, `tests/bdd_governance.rs`, `features/governance_decimal.feature`, `entities/{vote,resolution,meeting,age_request}.rs`, repos impl correspondants. Deps : aucune.

- **WP-A2 — Cascade tests #443 (LONG POLE)** · #443 · T2 · L · *critique*
  `docker compose run --rm backend cargo check --tests` propre (`--lib` déjà propre). Literals f64→`dec!()` aux call-sites, assertions Decimal-equality, zéro régression scénario @security/@negative. Fichiers : `tests/bdd_financial.rs` (~23 occ.), `tests/e2e_*.rs`, glue `features/*.feature`. Deps : conventions WP-A1. **Concurrent Track F.**

- **WP-A3 — EXP-006 journal_entry (réduit)** · #433 · T2 · M
  `Result<_,String>`→`AppError` sur `JournalEntry/JournalEntryLine`. `features/journal_entries.feature` 4-cat RED-first : @negative **débit≠crédit rejeté**, @edge 0.1+0.2=0.3, @security isolation cross-org, @happy écriture équilibrée. Pas de SQL. Deps : A1, A2.

- **WP-A4 — EXP-005 charge_distribution** · #433 · T2 · M · *parallèle A3*
  `Result→AppError` ; 4-cat : @security somme quotas==100% à `dec!(1.0001)`, @negative quota>1/négatif rejeté. Deps : A1.

- **WP-A5 — EXP-007 quote/etat_date (Art. 577 CC, plus gros résidu)** · #433 · T2 · L
  f64→Decimal `etat_date` (17 occ.) + `quote` + DTO/use_case/repo ; migration SQL **seulement si** colonnes `DOUBLE PRECISION` (vérifier `20251115000000_create_etats_dates.sql`, `20251120150000_create_quotes.sql`) ; `Result→AppError` ; 4-cat `etat_date.feature`/`quotes.feature`. Deps : A1, A2.

- **WP-A6 — EXP-008 owner_contribution/call_for_funds/gamification** · #433 · T2 · M
  Monétaire→Decimal+AppError+4-cat. Gamification/ratings = scores non-PCMN → carve-out ADR-0008, f64 conservé (@happy+@negative légers). Deps : A1, A2.

- **WP-A7 — Finaliser ADR-0008 + politique #526/#339** · #526/#339/ADR-0008 · T2 (accept=humain) · M
  (a) ADR-0008 finalisé (ratio + %-affichage + carve-out gamification). (b) #526 : garder `expenses_amount_check > 0`, modéliser les annulations en contre-écritures journal (pas de relâche schéma), documenter. (c) #339 rotate 501 `api_key_handlers.rs:506` : **reco = implémenter rotate minimal** derrière 4-cat (@security ancienne clé invalide post-rotate, @negative rotate non-propriétaire→403) ; alternative = retirer la route + documenter pour qu'aucun 501 ne parte en bêta.

### Track B — Backend autre

- **WP-B1 — Re-vérifier bugs revue humaine** · BUG-WF*/#523 · T2 · M (L si WF14-2 réel) · *J1*
  Re-jouer `HUMAN_REVIEW_REPORT_v0.1.0.md` comme checklist vs `feature/dev` courant : **BUG-WF14-2 fuite bâtiments cross-org (Alice voit 3 bâtiments) = bloquant sécurité bêta si reproductible** — tracer scoping `organization_id` repo buildings ; BUG-WF2-1 `voting_power≤1000` vs seed>1280 (vérifier `20260401000000_fix_voting_power_constraint.sql`) ; BUG-WF7-1 ticket 400 ; NaN% compteurs (probablement corrigé par #523 `7c2d664` — vérifier). Repro RED par bug confirmé. Deps : aucune.

- **WP-B2 — Gate sécurité dependabot #432** · #432 · T2 · S-M · *parallèle*
  Trier 14 vulns (5H/3M/6L) ; patcher tous HIGH + MOD atteignables (`cargo update -p`) ; documenter résiduel accepté. Fichiers : `backend/Cargo.{toml,lock}`, `security.yml`. Deps : aucune.

### Track C — Frontend sécurité (refacto #343 / SSR client:load DIFFÉRÉS post-bêta)

- **WP-FE1 — JWT hors localStorage (vol session XSS)** · BLOQUANT SÉCURITÉ · T2 · L · *critique*
  `auth.ts:128-235` : refresh token → cookie backend `HttpOnly; Secure; SameSite=Strict` ; access token en mémoire seule + silent-refresh au load. Backend login/refresh : set-cookie + read-cookie + CORS credentials. 4-cat RED-first : @security token illisible JS/`document.cookie` & absent localStorage ; @negative cookie forgé/expiré→401 ; @happy login→refresh→protégé ; @edge refresh à la borne d'expiration. Deps : coordination WP-B1 ; moitié backend nourrit moitié FE.

- **WP-FE2 — Corriger bugs FE revue (WF1-1/2/3)** · T2 · M
  Bouton "Nouvelle réunion" `/meetings` (syndic) ; POST `/convocations` envoie `building_id` ; lister convocations créées via API. Re-vérifier vs courant. 4-cat Playwright (@happy créer+envoyer via UI, @negative building_id manquant → erreur visible pas 400 silencieux). Deps : WP-B1.

### Track D — E2E/QA

- **WP-D1 — Réparer specs Playwright skippés** · #331 · T2 · M
  Un-skip 21 ApiKeys/SecurityIncidents : normaliser case rôle `SYNDIC`↔`syndic` + `building_id` dans `global-setup.ts`/TestWorld ; câbler/justifier 32 specs hors-CI. Plancher smoke ≈219/240 sans régression, jugement par-scénario. Fichiers : `frontend/tests/e2e/global-setup.ts`, `*.scenario.ts` (dont `meeting-vote.scenario.ts`), `{api-keys,security-incidents}*.spec.ts`, job playwright `ci.yml`. Deps : WP-FE1 (auth ripple global-setup), WP-B1.

- **WP-D2 — Câbler vitest au gate** · #343 · T2 · S-M
  Job `vitest` existe (`ci.yml:402`) ; couvrir auth store (WP-FE1) + composants convocation/réunion @happy/@negative ; cible = composants critiques bêta (pas 181/181). Deps : WP-FE1, WP-FE2.

### Track E — Tests IaC (sous-ensemble VPS de #354)

- **WP-E1 — Lint IaC minimal viable** · #354 · T2 · M · *100% parallèle*
  Job lint dans `ci-infra.yml`, assets VPS seulement : `terraform fmt -check`/`validate` (modules OVH + `monosite/vps/production/terraform`), `ansible-lint` (14 rôles + playbook prod), `yamllint`, `shellcheck` (**gate dur sur `gitops-deploy.sh`** — il exécute le déploiement prod). conftest/molecule/terratest = Phase 2. Deps : aucune.

### Track F — Ops VPS (concurrent Track A — aucun fichier partagé)

- **WP-F1 — Provision VPS : Terraform + Ansible** · T1 · L
  `terraform plan/apply` `monosite/vps/production/terraform` ; `ansible-playbook playbook.yml` (common, hardening, docker, security, monitoring, backup, gitops, dns) ; vérifier LUKS/fail2ban/Suricata/CrowdSec/SSH+kernel hardening, Prometheus/Grafana/ELK, backup GPG+S3. **`terraform apply`/`ansible-playbook` prod = HUMAIN** ; agent fournit plan/diff revus + runbook. Deps : WP-E1 (souhaitable).

- **WP-F2 — Vérification TLS (déjà câblé)** · T1 · S
  Confirmer émission cert Let's Encrypt HTTP-01 domaine bêta. Pré-requis humain : DNS A→IP VPS, ports 80/443 ouverts, `ACME_EMAIL` dans `.env`. Aucun nouveau pipeline. Deps : WP-F1.

- **WP-F3 — Bring-up poller gitops-deploy.sh + secrets** · T1 · M
  `gitops-deploy.sh watch` en unit systemd (TOPOLOGY=vps, ENV=production, BRANCH=<release>). Ansible Vault suffisant v0.1.0 (SOPS/age = Phase 2). Valider `.env`, retry pull/up, tag image `<branch>-<sha7>`. Deps : WP-F1, WP-F2, images pushées (`docker-build-push.yml`).

- **WP-F4 — État Terraform distant + RUNBOOK VPS** · T2 (doc) + T1 (state) · M
  Configurer/vérifier `backend.tf` état distant (pas d'état prod local). Rédiger `docs/RUNBOOK_VPS_PRODUCTION.md` (absent vérifié) : deploy, rollback (revert commit→poller redéploie), restore GPG+S3, endpoints santé, logs `/var/log/koprogo-gitops-production.log`. Deps : WP-F1.

### Track G — Gate de release

- **WP-G1 — Revue humaine fraîche** · T1 · M
  Nouvelle revue vs branche release sur host provisionné ; rapport périmé 2026-04-01 = checklist re-vérification (WF1-1..4, WF2-1, WF7-1, WF14-2, NaN%). Produire successeur daté `docs/HUMAN_REVIEW_REPORT_v0.1.0_<date>.md` : ✔/✘ par bug + GO/NO-GO signé. **HUMAIN exécute & signe.** Deps : A1-A6, B1, FE1, FE2, D1 VERTS + VPS up (F3).

- **WP-G2 — Tag git v0.1.0** · T1 · S
  HUMAIN tagge `v0.1.0` + déclenche `release-tag.yml`, seulement après WP-G1 GO signé + checklist GO verte.

## Graphe de dépendances / chemin critique

```
WP-A1 (C1 + ADR-0008) ─┬─► A3 (EXP-006) ─┐
   └─► A7 (ADR/#526/#339)├─► A4 (EXP-005) │
        ▼               ├─► A5 (EXP-007)──┼─► #433 VERT ─┐
WP-A2 (#443 LONG POLE) ─├─► A6 (EXP-008) ─┘             │
                        └────────────────────────────────┤
B1 (bugs revue; WF14-2 fuite) ─────────────────────────  ├─► make ci ─► G1 ─► G2
B2 (#432 deps) ─────────────────────────────────────────  ┤   VERT    (humain)(TAG)
FE1 (JWT→cookie SEC) ─► FE2 ─► D1 ─► D2 ────────────────  ┤    ▲
E1 (lint IaC) ─► F1 (TF/Ansible,T1) ─► F2 (TLS,T1) ─► F3 (poller,T1) ─► F4 ──┘
        └──────────────── concurrent Track A ────────────────┘
```

**Chemin critique** : `A1(M) → A2(L) → A5(L etat_date) → #433 VERT → make ci VERT → G1(T1) → G2(T1)`, convergeant avec `FE1(L)→FE2→D1` et `E1→F1(T1)→F2(T1)→F3(T1)`.
**Démarrages J1 sans inter-dép** : A1, B1, B2, FE1(moitié backend), E1, F1(terraform plan). Ops est court en effort mais borné par la latence Tier-1 humaine → **lancer F1-prep + E1 dès J1** pour que Ops finisse en parallèle du long pole A2→A5, pas après.

## Critères GO (Definition of Done — bêta fermée)

- [ ] `cargo check --tests` propre (#443 — A2)
- [ ] `bdd_governance` 4 scénarios VERTS par-scénario ; panic #525 disparu ; migration `20260516000000` appliquée DB test (A1)
- [ ] ADR-0008 finalisé & accepté humain (ratio + %-affichage + carve-out gamification) (A7)
- [ ] #433 EXP-005/006/007/008 Decimal + `Result<_,AppError>` ; **débit==crédit @negative VERT** (A3-A6)
- [ ] Aucun `Result<_,String>`/`unwrap/expect` (hors `#[cfg(test)]`) sur fichiers touchés du chemin bloquant
- [ ] #432 tous HIGH + MOD atteignables résolus ; résiduel documenté (B2)
- [ ] #526 décidé & documenté ; #339 rotate implémenté 4-cat OU aucun 501 ne part (A7)
- [ ] BUG-WF14-2 fuite bâtiments cross-org NON reproductible — e2e @security VERT (B1)
- [ ] BUG-WF2-1 réconcilié ; compteurs NaN% disparus (vérifier #523) (B1)
- [ ] Refresh token PAS en localStorage ; cookie HttpOnly+Secure+SameSite ; @security VERT (FE1)
- [ ] BUG-WF1-1/2/3 re-vérifiés corrigés (FE2)
- [ ] `make ci` VERT en local avant push ; BDD jugé par-scénario, zéro régression @security/@negative
- [ ] Plancher Playwright smoke ≈219/240 ; specs skippés un-skippés ou documentés (D1) ; vitest VERT composants critiques (D2)
- [ ] Lint IaC VERT : terraform fmt/validate, ansible-lint, yamllint, shellcheck(`gitops-deploy.sh`) (E1)
- [ ] Terraform appliqué + état distant (F1/F4) ; rôles Ansible convergés (F1)
- [ ] Cert Let's Encrypt valide sur 443, http→https OK (F2)
- [ ] `gitops-deploy.sh watch` systemd actif ; drill deploy+rollback + drill restore GPG+S3 faits (F3/F4)
- [ ] `docs/RUNBOOK_VPS_PRODUCTION.md` rédigé & revu (F4)
- [ ] Rapport revue humaine daté frais — GO signé bêta fermée ; rapport 2026-04-01 archivé non utilisé (G1)
- [ ] Tag `v0.1.0` créé par HUMAIN après GO signé (G2)

## Vérification — commandes exactes & gate humain

Backend (agent) :
```
docker compose run --rm backend cargo check --lib
docker compose run --rm backend cargo check --tests        # propre post WP-A2
docker compose run --rm backend cargo clippy --all-targets --all-features -- -D warnings
docker compose run --rm backend cargo test --lib --all-features
docker compose run --rm backend sqlx migrate run            # DB test seulement
docker compose run --rm backend cargo test --no-fail-fast --test bdd --test bdd_governance --test bdd_financial --test bdd_operations --test bdd_community
docker compose run --rm backend cargo test --test e2e
docker compose run --rm backend cargo audit                 # #432
```
Frontend :
```
docker compose run --rm frontend npm run build
docker compose run --rm frontend npx svelte-check --threshold warning
docker compose run --rm frontend npx vitest run
docker compose run --rm frontend npx playwright test --project=chromium    # plancher smoke
docker compose run --rm frontend npx playwright test --project=scenarios   # par-scénario
docker compose run --rm frontend npx prettier --check .
```
Gate push / contrat :
```
make ci                # VERT obligatoire avant tout push
make openapi-check     # si DTOs touchés
make types-sync        # si spec changée
```
IaC (post WP-E1) :
```
terraform -chdir=infrastructure/monosite/vps/production/terraform fmt -check
terraform -chdir=infrastructure/monosite/vps/production/terraform validate
ansible-lint infrastructure/monosite/vps/production/ansible/playbook.yml
yamllint infrastructure/monosite/vps/production
shellcheck infrastructure/_shared/scripts/gitops-deploy.sh
```
Gate humain (Tier 1 — agent diagnostique/propose seulement) :
1. `terraform apply` (agent fournit plan revu) — F1
2. `ansible-playbook -i ansible/inventory.ini ansible/playbook.yml` prod — F1
3. DNS A→IP VPS, ouvrir 80/443, `ACME_EMAIL` dans `.env` prod — F2
4. Activer unit systemd poller ; 1 deploy + 1 rollback + 1 restore GPG+S3 drill — F3/F4
5. Conduire & signer revue humaine GO/NO-GO datée — G1
6. `git tag v0.1.0` + `release-tag.yml` seulement après #5 GO signé — G2
7. Tout merge vers branche protégée via PR revue humaine

## Phase 2 (post-v0.1.0 : k3s/ArgoCD) — allégé

- **#453** ACME DNS-01 (OVH) + SOPS/age en CI/deploy (non-prod + k3s ; prod bêta déjà HTTP-01).
- **#466** promotion GitOps dev→integration→staging→production (branche `feat/gitops-promotion-workflows-466`) ; ArgoCD remplace le poller systemd.
- **#515** gaps k3s (déjà mergés main PR #516) activés au bootstrap k3s.
- **#354 approfondi** : conftest ISO-27001, molecule, terratest, helm lint, matrice kubeconform complète.
- **Vault + Velero** câblage k3s (secrets dynamiques remplacent Ansible Vault ; test restore Velero RTO/RPO/MTTR).
- **Bootstrap k3s** : `infrastructure/monosite/k3s/production` + apps ArgoCD ; migration données VPS→k3s ; bascule DNS.
- **Différé (verrouillé)** : refacto hexagonale frontend #343, remédiation SSR/`client:load`, gate public complet #427, `unwrap/Result<_,String>` résiduel sur ports non-bloquants (vote/resolution/age_request), 32 specs Playwright hors-CI.

## Fichiers critiques

- `backend/tests/bdd_financial.rs` — cascade #443 (~23 occ. f64)
- `backend/src/domain/entities/resolution.rs:185-212` — décision f64 %-affichage / ADR-0008
- `backend/migrations/20260516000000_alter_governance_to_numeric.sql` — gate C1 / #525
- `backend/src/domain/entities/etat_date.rs` — EXP-007, 17 occ. f64 (plus gros résidu umbrella)
- `frontend/src/stores/auth.ts:128-235` — JWT localStorage (bloquant sécurité bêta)
- `infrastructure/_shared/scripts/gitops-deploy.sh` — poller go-live (gate shellcheck dur)
- `infrastructure/monosite/vps/production/docker-compose.override.yml` — TLS ACME déjà câblé (vérifier seulement)
