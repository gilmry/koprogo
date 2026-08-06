===============================================================================================
Issue #427: Validation — taxonomie tests 4 catégories + revue humaine+Cowork comme gate release
===============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: documentation,track:infrastructure priority:critical,security
:Assignees: Unassigned
:Created: 2026-04-29
:Updated: 2026-04-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/427>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'audit du 2026-04-29 a montré que le projet revendique 921 scénarios BDD et "1 191 tests" mais a livré v0.1.0 avec :
   - bouton **Créer ticket** cassé (commit `49f8a2a`)
   - panneau de vote **non gated** par ownership (faille RBAC)
   - **1 967 `unwrap()`/`expect()`** non couverts par des tests d'erreur
   - contrainte DB `voting_power ≤ 1000` incompatible avec un seed à 1 280 (BUG-WF2-1)
   - `docs/HUMAN_REVIEW_REPORT_v0.1.0.md` qui conclut lui-même *"NO-GO pour release publique"*
   
   La couverture existe, **la discipline manque**. Cette issue acte le constat que la directive TDD/BDD donnée par l'utilisateur depuis le début du projet n'a pas été mécaniquement appliquée, et propose deux couches mécaniques complémentaires : **(A) tests automatisés à 4 catégories** + **(B) revue humaine + Cowork-Chrome comme gate de release**, le tout intégré au workflow de release.
   
   Issues liées : #425 (garde-fous IA techniques), #426 (cleanup docs), #427 (Maury automatisé — à venir, bloqué par cette issue).
   
   ---
   
   ## 1. Constats — discipline insuffisante
   
   ### 1.1 La métrique cache la maladie
   - **921 scénarios BDD** annoncés ≠ 921 chemins testés. La majorité sont des variations *happy path*.
   - **1 191 `#[test]`** ≠ 1 191 cas d'erreur testés. Le ratio happy/edge/security/negative n'est pas mesuré.
   - `cargo tarpaulin` rapporte une couverture en lignes flatteuse mais ne distingue pas "ligne happy path exécutée" de "ligne d'erreur testée".
   - 12 scénarios `@skip`/`@wip` non documentés (raison du skip ?).
   
   ### 1.2 Bugs livrés en v0.1.0 attribuables à la non-application TDD/BDD
   | Bug livré | Catégorie de test manquante | Coût aval observé |
   |---|---|---|
   | BUG-WF1-1 (pas de bouton "Nouvelle réunion") | `@happy` E2E "syndic clique → formulaire ouvre" | Hotfix v0.1.0 |
   | BUG-WF1-2 (POST /convocations omet `building_id` → 400 silencieux) | `@negative` "payload incomplet → erreur typée explicite" | Workflow AG inutilisable |
   | BUG-WF2-1 (DB constraint vs seed à 1 280 tantièmes) | `@edge` "voting_power max légal ?" | Vote impossible pour gros lots |
   | Vote panel non gated ownership | `@security` "non-propriétaire tente vote → 403" | Faille RBAC en prod |
   | 1 967 `unwrap()` | `@negative` "DB indisponible / ressource absente → AppError" | 1 967 panics potentiels |
   | `s3SecretKey: koprogo123` recopié 8× | (pas un test, mais hook gitleaks #425) | Secrets en clair |
   
   ### 1.3 Revue humaine non mécanisée
   `docs/HUMAN_REVIEW_REPORT_v0.1.0.md` et `HUMAN_REVIEW_PLAN_v0.1.0.md` **existent et fonctionnent** : ils ont permis de poser le NO-GO. **Mais** :
   - Pas de template versionné réutilisable pour les futures releases.
   - Pas de hook qui refuse `git tag vX.Y.Z` sans rapport signé `verdict: GO` pour cette version.
   - Pas d'intégration Cowork-Chrome formalisée (le plan est manuel, le report aussi).
   - Le rapport est un fichier ad-hoc, pas un artefact de PR/release.
   
   ---
   
   ## 2. Cause racine
   
   **Aucun enforcement mécanique** :
   1. Pas de taxonomie `@happy / @edge / @security / @negative` imposée → l'agent écrit ce qu'il veut.
   2. Pas de matrice 4×N par FR dans la PRD → la directive métier elle-même n'exige pas la complétude.
   3. Pas de `PreToolUse Edit` qui refuse l'écriture d'un handler sans test RED préexistant.
   4. Pas de CI step qui vérifie présence de chaque catégorie par FR.
   5. Pas de mutation testing → tests testés mais pas la qualité des tests.
   6. Pas de hook sur `git tag` exigeant un `HUMAN_REVIEW_REPORT_vX.Y.Z.md` signé GO.
   7. Cowork-Chrome utilisé en mode artisanal, sans skill local ni template structurant.
   
   L'agent IA optimise pour les métriques mesurées. Si on mesure "coverage en lignes", il monte les lignes. Si on mesure "matrice 4×N par FR + revue humaine signée", il livre les deux.
   
   ---
   
   ## 3. Recette — Partie A : TDD/BDD discipline 4 catégories
   
   ### A.1 Taxonomie obligatoire
   Tout `Scenario:` Gherkin / `#[test]` / `*.spec.ts` porte **exactement un** tag parmi :
   - **`@happy`** — chemin nominal end-to-end
   - **`@edge`** — bornes (max/min/empty/0/1/N)
   - **`@security`** — RBAC, auth, injection, rate limit, escalade
   - **`@negative`** / **`@catastrophic`** — défaillance correcte (pas de panic, erreur typée, message correct)
   
   ### A.2 Bibliothèque de scénarios baseline réutilisables
   `backend/tests/features/_security_baseline.feature` (importable via `Background:` ou tags dans toute feature) :
   - Utilisateur non authentifié → 401
   - Utilisateur authentifié hors rôle requis → 403
   - Body JSON malformé → 400 typé
   - Champ obligatoire manquant → 422 + contexte
   - Rate limit dépassé → 429
   - Service externe indisponible → 503 sans panic
   - Idempotency key réutilisée → 409 ou no-op selon spec
   
   ### A.3 Matrice 4×N dans la PRD/story (sign-gate)
   Dans `docs/maury/<feature>/prd.md` et `stories.md`, chaque FR/story expose :
   ```markdown
   | FR-007 (Créer ticket)   | @happy | @edge | @security | @negative |
   |-------------------------|--------|-------|-----------|-----------|
   | Scénario `.feature` ref | TKT-01 | TKT-02| TKT-03    | TKT-04    |
   ```
   Hook `PreToolUse Edit` sur le frontmatter `status: signed` du fichier : refuse signature si une cellule est vide.
   
   ### A.4 Hook RED-first (bloquant local)
   `PreToolUse Edit` sur `backend/src/{infrastructure/web/handlers,application/use_cases,domain/entities}/*.rs` (non-test) :
   - Cherche un test correspondant à l'unité éditée
   - Vérifie que ce test est **rouge** (`cargo test --no-run` puis `cargo test -- --no-capture` doit montrer un échec récent ou test absent)
   - Si pas de test rouge → blocage : *"Pas de test RED en cours pour `Building::create_with_units`. Écris-le d'abord. RED-first."*
   
   ### A.5 Hook category-completeness (bloquant en CI)
   GitHub Action :
   ```bash
   for fr in $(grep -oE 'FR-[0-9]+' docs/maury/*/prd.md | sort -u); do
     for cat in happy edge security negative; do
       count=$(cargo test --features cucumber -- --tags "@$cat and @$fr" 2>&1 | grep -c "scenarios passed")
       [ "$count" -eq 0 ] && echo "::error ::FR=$fr cat=$cat: 0 scenario" && exit 1
     done
   done
   ```
   PR rouge si une cellule est manquante.
   
   ### A.6 Mutation testing périodique
   - `cargo install cargo-mutants` ; `make mutation-test` lance sur `backend/src/domain/`.
   - Cron CI hebdo. Si > 5 % mutants survivants dans `domain/` → issue auto-créée listant les survivants.
   - Métrique de **qualité** des tests (pas leur quantité).
   
   ### A.7 Sous-agent `tdd-coverage-auditor`
   Déclenché sur PR : compare diff aux tests. Pour chaque nouvelle `pub fn` / nouveau handler / nouveau composant Svelte interactif → vérifie présence des 4 tags. Comment auto sur la PR avec exemple manquant.
   
   ---
   
   ## 4. Recette — Partie B : revue humaine + Cowork-Chrome comme gate release
   
   ### B.1 Templates versionnés (réutilisables version après version)
   
   `.claude/templates/HUMAN_REVIEW_PLAN.template.md` — basé sur le format existant `HUMAN_REVIEW_PLAN_v0.1.0.md` :
   - Conventions (`[RÔLE]` / `→` / `✓ Attendu` / `✗ Bug`)
   - Sessions par thème (Conformité Légale, Financier, Multi-rôle, Mobile/A11y, Sécurité)
   - Workflows numérotés `WF<n>` référençant articles légaux quand pertinent
   - Tableau personas/comptes test
   - Checklist navigable
   
   `.claude/templates/HUMAN_REVIEW_REPORT.template.md` — basé sur `HUMAN_REVIEW_REPORT_v0.1.0.md` :
   ```yaml
   ---
   version: vX.Y.Z
   date: YYYY-MM-DD
   reviewer: <human-name>
   cowork_session_id: <claude-cowork-session>
   environment: staging|preview|prod
   duration_hours: <n>
   verdict: GO | GO_CONDITIONAL | NO_GO   # gate field
   verdict_justification: <text>
   signed_by: <human-email>
   signed_at: <iso-timestamp>
   ---
   ```
   - Résumé exécutif
   - Verdict explicite
   - Workflows testés (table Step / Result ✅⚠️❌ / Détails)
   - Bug list avec sévérité (`BUG-WFn-m [CRITIQUE|MAJEUR|MINEUR]`)
   - Conditions de re-review
   
   ### B.2 Skill `cowork-release-review` (`.claude/skills/cowork-release-review/SKILL.md`)
   Instructions pour Claude en mode Cowork-Chrome :
   1. Charger `HUMAN_REVIEW_PLAN_<version>.md`.
   2. Naviguer chaque workflow dans Chrome (déjà connecté avec personas seed).
   3. Pour chaque étape : screenshot, observer attendu vs réel, noter bug si mismatch.
   4. Captures vidéo des bugs reproductibles.
   5. À la fin de session : générer brouillon `HUMAN_REVIEW_REPORT_<version>.md` (verdict provisoire `GO_CONDITIONAL` par défaut).
   6. **L'humain seul** peut éditer le verdict en `GO` et signer.
   
   ### B.3 Hook gate sur `git tag`
   `PreToolUse Bash(git tag:*)` :
   ```bash
   version=$(printf '%s' "$tool_input.command" | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+')
   report="docs/maury/releases/$version/human-review-report.md"
   if [ ! -f "$report" ]; then
     echo "GUARDRAIL BLOCK: pas de $report. Lancer la session Cowork+human review d'abord." >&2
     exit 2
   fi
   verdict=$(grep -oE '^verdict: (GO|GO_CONDITIONAL|NO_GO)' "$report" | head -1 | awk '{print $2}')
   case "$verdict" in
     GO) exit 0 ;;
     GO_CONDITIONAL) echo "GUARDRAIL BLOCK: verdict GO_CONDITIONAL. Conditions à clore avant tag." >&2 ; exit 2 ;;
     NO_GO|"") echo "GUARDRAIL BLOCK: verdict NO_GO ou absent." >&2 ; exit 2 ;;
   esac
   ```
   Et même check via GitHub Action sur push du tag (defense in depth).
   
   ### B.4 Workflow PR release intégré
   1. Branche `release/vX.Y.Z` créée → CI build préview env (Vercel/Netlify/staging k8s) avec URL unique.
   2. PR template release (`.github/PULL_REQUEST_TEMPLATE/release.md`) liste obligatoires :
      - [ ] CI green (lint, tests par catégorie A.5, gitleaks #425, tfsec)
      - [ ] Mutation test sur master ≥ 95 % killed
      - [ ] `docs/maury/releases/vX.Y.Z/human-review-plan.md` rempli
      - [ ] Session Cowork-Chrome lancée (lien session)
      - [ ] `docs/maury/releases/vX.Y.Z/human-review-report.md` signé `verdict: GO` par humain
      - [ ] `CHANGELOG.md` à jour
      - [ ] Pas de bug `[CRITIQUE]` non résolu dans le report
   3. GitHub Action vérifie chaque case avant d'autoriser merge sur `production`.
   4. Tag créé seulement après merge : hook B.3 vérifie une dernière fois.
   5. `gh release create` inclut le lien vers `human-review-report.md` dans la description publique.
   
   ---
   
   ## 5. Critères d'acceptation
   
   ### Partie A (mécanique)
   - [ ] Tous les `Scenario:` du repo portent un tag `@happy|@edge|@security|@negative`.
   - [ ] `backend/tests/features/_security_baseline.feature` créé et importé dans ≥ 5 features.
   - [ ] PRD template (lié au #427 Maury) inclut matrice 4×N obligatoire par FR.
   - [ ] `cargo clippy -W clippy::unwrap_used -W clippy::expect_used` warning en CI ; bloquant après remédiation.
   - [ ] Hook `PreToolUse` RED-first actif et testé (édition refusée si pas de test rouge).
   - [ ] CI step category-completeness actif et testé (FR sans `@security` rouge sur PR de démo).
   - [ ] `make mutation-test` exécutable et documenté ; cron hebdo configuré.
   - [ ] Sous-agent `tdd-coverage-auditor` matérialisé et testé sur une PR.
   
   ### Partie B (humaine + Cowork)
   - [ ] `.claude/templates/HUMAN_REVIEW_PLAN.template.md` extrait du v0.1.0, anonymisé, paramétré.
   - [ ] `.claude/templates/HUMAN_REVIEW_REPORT.template.md` avec frontmatter `verdict` + signature.
   - [ ] `.claude/skills/cowork-release-review/SKILL.md` documenté avec instructions step-by-step.
   - [ ] Hook `PreToolUse Bash(git tag:*)` actif ; tentative `git tag v0.1.1` sans report → bloquée.
   - [ ] GitHub Action de gate release rouge sans report signé `GO`.
   - [ ] PR template release créé.
   - [ ] Une release pilote (par exemple v0.1.1 hotfix) prouve le flux : CI green → préview → Cowork session → report signé GO → tag autorisé.
   
   ---
   
   ## 6. Priorisation
   
   | Sprint | Livrables | Effet |
   |---|---|---|
   | **S1 — taxonomie** | A.1 (tags 4 cat sur tout l'existant), A.2 (baseline feature), templates B.1 extraits du v0.1.0 | Le projet sait mesurer ce qui compte ; les futures revues humaines ont un format stable |
   | **S2 — enforcement local** | A.3 (matrice PRD/story sign-gate), A.4 (hook RED-first), skill B.2 | L'agent ne peut plus écrire du code sans test RED ; Cowork a son skill |
   | **S3 — enforcement CI + release** | A.5 (CI category-completeness), A.6 (mutation testing), A.7 (sous-agent), B.3 (hook git tag), B.4 (PR template + GH Action) | Le flux release complet est mécanisé ; rien ne sort sans GO signé |
   | **S4 — remédiation legacy** | Tagger les 921 scénarios existants, écrire les `@security`/`@negative` manquants par FR, remplacer les 1967 `unwrap()` par `AppError`, première release pilote v0.1.1 via le nouveau flux | La dette pré-discipline est soldée ; le flux fait ses preuves |
   
   ---
   
   ## 7. Lien avec les autres issues
   
   - **Bloque #427** (Maury automatisé) : la PRD template Maury doit inclure la matrice 4×N — sinon on automatiserait une discipline incomplète.
   - **Complète #425** (garde-fous IA) : #425 sécurise au niveau outil (deny secrets, gitleaks, fmt), #428 sécurise au niveau livrable (validation complète).
   - **Bénéficie de #426** (cleanup docs) : un CLAUDE.md trim + des docs sans doublons = contexte propre pour l'agent qui lit la matrice 4×N.
   
   ---
   
   🤖 Issue générée par Claude Opus 4.7 après directive utilisateur sur la discipline TDD/BDD non appliquée et l'intégration Cowork-Chrome dans le workflow release.

.. raw:: html

   </div>

