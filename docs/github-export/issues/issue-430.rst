========================================================================
Issue #430: 🏃 Sprint Planning W18 — Team A (Backend) — pilote simulation
========================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: documentation,track:software priority:medium
:Assignees: Unassigned
:Created: 2026-04-29
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/430>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ---
   ceremony: sprint-planning
   sprint_id: W18
   sprint_dates: 2026-04-29 → 2026-05-12 (2 semaines)
   team: A (Backend)
   art: koprogo-art-1
   pi: 2026-Q2
   status: pilot (premier sprint planning auto-généré dans la simulation)
   participants_lead: scrum-master-A
   participants_active:
     - product-owner-A (priorise backlog)
     - dev-A (estime + commit)
     - qa-A (DoR/DoD check, matrice 4×N)
   participants_observer:
     - safe-rte (sync ART)
     - rust-expert (input architectural Rust)
     - code-reviewer (input cohérence cross-cutting)
   human_supervisor: gilmry (final commitment approval)
   ---
   
   # 🏃 Sprint Planning W18 — Team A (Backend)
   
   > **Sprint pilote** — premier sprint planning auto-généré dans la simulation organisationnelle. Démontre la chaîne décrite en [#428 matrice personas × cérémonies](https://github.com/gilmry/koprogo/issues/428#issuecomment-4347406461).
   
   ## Sprint Goal (proposé par `product-owner-A`)
   
   **Démarrer la migration `Result<_, String>` → `AppError` typé** sur 3 modules domain critiques (auth, building, expense), pour réduire la dette technique signalée à l'audit (#425/#427) et préparer une couche d'erreurs propre avant les futures features.
   
   **Definition of "Sprint succès"** : les 3 modules ciblés ont leurs ports + use cases + handlers en `Result<T, AppError>`, avec hook PostToolUse warn-unwrap actif et tests `@negative` ajoutés selon la matrice 4×N (#427).
   
   ## Capacity (Team A backend, 2 semaines, vélocité IA Maury ÷3)
   
   | Membre simulé | Capacité brute (humain) | Vélocité IA appliquée | Capacité effective |
   |---|---|---|---|
   | `dev-A` (Rust focus) | 80h | ÷3 (backend complex) | ~27h IA-équivalent |
   | `qa-A` (tests) | 80h | ÷3 | ~27h IA-équivalent |
   | Buffer (impedimenta + retro) | 20% | — | -10h |
   
   **Capacité commitée** : ~44h IA-équivalent. Cible commitment : 5 stories S/M (~3 jours IA chacune max).
   
   ## Backlog candidates (priorisé par WSJF par `product-owner-A`)
   
   | # | Story | Taille | Module | Score WSJF | Sélectionné |
   |---|---|---|---|---|---|
   | AUTH-001 | Migrer `auth_use_cases.rs` vers `AppError` typé (login/refresh/me) | M | auth | 24 | ✅ |
   | AUTH-002 | Tester `@security` sur scenarios login (rate limit, brute force, JWT expired) | S | auth | 22 | ✅ |
   | BLD-001 | Migrer `building_use_cases.rs` vers `AppError` typé | M | building | 20 | ✅ |
   | BLD-002 | Refactor `Building::new()` pour invariants stricts (name non-empty, total_units > 0) | S | building | 18 | ✅ |
   | EXP-001 | Migrer `expense_use_cases.rs` vers `AppError` typé | L | expense (1179 LOC) | 16 | ⏸️ peut-être trop gros, à splitter |
   | EXP-002 | Audit f64 dans expense.rs et payment.rs (cf. memory `project_no-f64-in-money.md`) | S | expense + payment | 28 | ✅ critical |
   
   **Stories commitées** : AUTH-001, AUTH-002, BLD-001, BLD-002, EXP-002. Total ~5 stories, capacité respectée.
   **Stories reportées** : EXP-001 → splittée en 2-3 sub-stories en sprint W19 (refinement à venir).
   
   ## Definition of Ready (DoR check par `qa-A` + `code-reviewer`)
   
   Pour chaque story commitée :
   
   - [ ] User story format `As X, I want Y so that Z`
   - [ ] BDD scenarios écrits (≥ 1 par catégorie : `@happy`, `@edge`, `@security`, `@negative` cf. matrice 4×N #427)
   - [ ] Fichiers exacts listés (chemins absolus)
   - [ ] Dependencies identifiées et résolues
   - [ ] Estimation Maury cohérente (S/M/L)
   
   **Status DoR** : 4/5 stories OK. AUTH-002 a besoin d'une review BDD scenarios par `qa-A` avant DoR-validated.
   
   ## Definition of Done (DoD enforced par `qa-A`)
   
   À la merge de chaque PR :
   
   - [ ] Tous tests verts (unit + BDD + E2E si user-facing)
   - [ ] Matrice 4×N respectée pour chaque FR touché (cf. #427 §A.3)
   - [ ] `cargo clippy -W clippy::unwrap_used -W clippy::expect_used` : 0 nouveau warning
   - [ ] `cargo audit` : 0 nouvelle vuln
   - [ ] gitleaks : 0 secret
   - [ ] `rust-expert` review approve (LGTM côté Rust)
   - [ ] `code-reviewer` review approve (LGTM côté holistique)
   - [ ] Doc à jour (CLAUDE.md mention si nouvelle convention)
   
   ## Risks ROAMing
   
   | Risque | Type | Owner | Mitigation |
   |---|---|---|---|
   | AppError trait coverage trop large dès sprint 1 | Owned | `dev-A` | Limiter aux 3 modules ; pattern réutilisable établi |
   | Impact sur API responses (status codes) | Mitigated | `rust-expert` + `safe-system-architect` | `impl ResponseError` mappe AppError → HTTP propre |
   | Tests `@security` sur auth nécessitent setup spécifique | Resolved | `qa-A` | Bibliothèque baseline `_security_baseline.feature` à venir #427 §A.2 |
   | f64 audit révèle plus de cas que prévu | Accepted | `rust-expert` | Si > 20 occurrences, splitter en sprint suivant |
   
   ## Confidence Vote (simulé)
   
   | Persona | Vote | Justification |
   |---|---|---|
   | `scrum-master-A` | 🟢 5/5 | Sprint goal clair, capacité respectée |
   | `product-owner-A` | 🟢 4/5 | Backlog priorisé, mais EXP-001 splittage non finalisé |
   | `dev-A` | 🟢 4/5 | Faisable, modulo apprentissage `thiserror` patterns |
   | `qa-A` | 🟢 4/5 | DoR partial sur AUTH-002, à clore en debut sprint |
   | `rust-expert` | 🟢 5/5 | Excellent goal pour la qualité Rust ; supportera reviewing |
   | `code-reviewer` | 🟢 5/5 | Cohérence projet alignée avec Maury v1.1 garde-fous |
   | `safe-rte` | 🟡 4/5 | Confidence ART OK, attention aux dépendances cross-team avec frontend (Team B) si auth touche les contracts API |
   
   **Vote moyen** : 4.4/5 → ✅ commitment confirmé.
   
   ## Action items immédiats (avant fin de Day 1)
   
   - [ ] `qa-A` finalise BDD scenarios AUTH-002 (DoR clôturé)
   - [ ] `dev-A` ouvre PR draft pour AUTH-001 (migration pattern visible)
   - [ ] `rust-expert` poste un commentaire avec le pattern `AppError` recommandé (référencé pour les autres stories)
   - [ ] `code-reviewer` configure le label `sprint-W18-team-A` sur les PRs
   - [ ] `documentation-writer` ouvre une issue `daily-standup-W18-day-1` pour la cérémonie de demain
   
   ## Liens
   
   - Sprint W17 retro : (n/a — premier sprint pilote)
   - PI 2026-Q2 objectives : (à formaliser par `safe-rte` en sprint W19)
   - Maury method version : v1.1 (cf. [`Maury/CHANGELOG.md`](../Maury/CHANGELOG.md))
   - Issue `track:software` parent : aucune (sprint pilote, démontre le flux)
   
   ---
   
   🤖 **Sprint planning auto-généré (pilote)** — démontre la chaîne ceremony × personas × Tier model. Tier 2 logué (issue créée par agent + commentaires Tier 2). Tier 1 (commitment final) = signature humaine sur cette issue par @gilmry.
   
   Refs: #428 (simulation org) #427 (validation 4-cat) #425 (guardrails)

.. raw:: html

   </div>

