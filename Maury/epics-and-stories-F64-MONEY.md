# Epics & User Stories — KoproGo f64 → Decimal Migration

## Methode Maury — Phase TOGAF E (Solutions)

**Auteurs** : Gilles Maury & Farah Maury
**Agent** : Claude (Scrum Master + Architect)
**Date** : 2026-05-13
**Version** : 1.0 (draft, awaiting human sign-off)
**Source issue** : [#521](https://github.com/gilmry/koprogo/issues/521) — f64 sur colonnes NUMERIC : panic /stats/syndic/urgent-tasks + 32 occurrences à migrer
**Source diagnostic** : [docs/agent-activity/2026-05-13-issue-draft-f64-money.md](../docs/agent-activity/2026-05-13-issue-draft-f64-money.md)
**Règle violée** : [memory `project_no-f64-in-money.md`](#) + [CRITICAL.md §4](../.claude/rules/CRITICAL.md)

**Disciplines** : SOLID + DDD + Hexagonal + BDD + TDD
**Production** : ITIL + IaC ISO 27001
**Contexte** : Bug observé en dev local (502 sur `/api/v1/stats/syndic/urgent-tasks`). Cause : `f64` sur colonnes NUMERIC + `Row::get()` panic-on-error. 3 panics latents identifiés, 30 autres `f64` à classifier. Pas un incident prod (v0.1.0, cf. memory `project_koprogo-current-state.md`) — c'est de la dette structurelle visible en sandbox.

---

## Vue d'ensemble

| Phase | Story | Type | Effort | Débloque |
|---|---|---|---|---|
| A | Fix 3 panics latents (stats/budget/payment-reminder) | Bugfix | M (~1j) | `/stats/syndic/urgent-tasks` opérationnel |
| B | Audit & classification des 30 f64 restants | Audit | S (~0.5j) | Scope clair pour Story C |
| C | Migration backlog f64 → Decimal | Refactor | L (dépend B) | Conformité `no-f64-in-money` complète |

**Ordre** : A débloque le 502 immédiat. B sans dépendance sur A. C dépend de B (output classification).

---

## Story A : Fix panics latents f64 sur colonnes NUMERIC (3 endpoints)

- **ID** : STORY-521-A | **Type** : Bugfix | **Taille** : M
- **Issue parent** : #521
- **Endpoints touchés** :
  - `GET /api/v1/stats/syndic/urgent-tasks` (panic confirmé en local)
  - Endpoint(s) consommant `BudgetRepository::get_budget_summary` (panic latent ligne 439)
  - Endpoint(s) consommant `PaymentReminderRepository` méthode utilisant l.492 (panic latent)
- **User Story** : En tant que Marc (syndic), quand je charge mon dashboard et qu'il existe au moins une charge en retard de paiement (`expenses.payment_status='overdue'`) dans ma copropriété, je veux que la liste des tâches urgentes s'affiche correctement avec le montant exact, **sans 502**, afin de pouvoir agir sur les retards.

- **Scenarios BDD** (4 catégories obligatoires) :

```gherkin
Feature: Stats syndic urgent tasks robustness vs NUMERIC columns

Background:
  Given Marc est syndic authentifié de l'organisation "Test Org"
  And une copropriété "Résidence Soleil" appartient à "Test Org"

@negative @bug521
Scenario: Endpoint urgent-tasks ne plante plus avec une charge overdue (regression test)
  Given une charge "Facture chauffage 2025-Q4" de 1234.5678 EUR existe pour "Résidence Soleil"
  And son payment_status est "overdue"
  When Marc appelle GET /api/v1/stats/syndic/urgent-tasks
  Then la réponse a le statut 200
  And la liste contient une tâche de type "expense" avec montant "1234.57" EUR
  And le worker Actix n'a généré aucun panic dans les logs

@happy
Scenario: Affichage montant standard 2 décimales
  Given une charge "Eau" de 123.45 EUR overdue
  When Marc appelle GET /api/v1/stats/syndic/urgent-tasks
  Then la tâche retournée a le titre "Charge en retard - 123.45€"

@edge
Scenario Outline: Montants aux bornes PCMN
  Given une charge "<desc>" de <montant> EUR overdue
  When Marc appelle GET /api/v1/stats/syndic/urgent-tasks
  Then la tâche retournée a le titre "Charge en retard - <affiche>€"

  Examples:
    | desc      | montant           | affiche  |
    | Zero      | 0.0000            | 0.00     |
    | 4 dec     | 12.3456           | 12.35    |
    | Max usuel | 999999999.9999    | 999999999.9999 |

@security
Scenario: Owner d'une autre orga ne voit pas les overdue de Marc
  Given un owner Bob authentifié d'une autre organisation
  And une charge overdue existe chez Marc
  When Bob appelle GET /api/v1/stats/syndic/urgent-tasks
  Then la réponse a le statut 403 (ou la liste est vide selon RBAC actuel — préciser pendant impl.)
```

Note BDD step "le worker Actix n'a généré aucun panic" : à implémenter via assertion sur les logs container (testcontainers) OU sur un compteur prometheus si dispo. Si trop coûteux, dégrader en assertion implicite (l'endpoint répond 200 = pas de panic). Décision en cours d'impl., logger dans la PR.

- **Tâches techniques** :
  1. [ ] **RED** : écrire les 4 scénarios BDD ci-dessus dans `backend/tests/features/stats_urgent_tasks.feature`. Vérifier qu'ils échouent contre `main` actuel (rouge confirmé sur `@negative @bug521`).
  2. [ ] [stats_repository_impl.rs:364](../backend/src/infrastructure/database/repositories/stats_repository_impl.rs#L364) — remplacer `let amount: f64 = expense.get("amount")` par `let amount: rust_decimal::Decimal = expense.try_get("amount").map_err(AppError::from)?`. Adapter le `format!` pour Decimal (`amount.round_dp(2)`).
  3. [ ] Idem [budget_repository_impl.rs:439](../backend/src/infrastructure/database/repositories/budget_repository_impl.rs#L439) (`total_amount`).
  4. [ ] Idem [payment_reminder_repository_impl.rs:492](../backend/src/infrastructure/database/repositories/payment_reminder_repository_impl.rs#L492) (`amount`).
  5. [ ] Vérifier que `UrgentTask` (DTO) et `BudgetSummary` / DTOs payment reminder sérialisent Decimal correctement (string ou number ?). Si number → risque perte précision en JSON → utiliser `serialize_with` rust_decimal::serde::str ou `arbitrary_precision`. Logger choix dans PR.
  6. [ ] Si signature de fonction change (e.g. `Result<_, String>` → `Result<_, AppError>`) propager dans les use cases / handlers appelants.
  7. [ ] **GREEN** : `make ci` doit passer ; BDD `@bug521` doit virer rouge → vert.
  8. [ ] Vérif manuelle browser : reload `/syndic` dashboard, aucun 502.
  9. [ ] Commit + PR vers `dev` avec body référençant `Refs #521` et le scénario BDD.

- **Critères d'acceptation** :
  - [ ] `grep "let.*: f64 = .*\.get(" backend/src/infrastructure/database/repositories/{stats,budget,payment_reminder}_repository_impl.rs` → 0 résultat
  - [ ] BDD `@bug521 @negative` vert (rouge avant fix)
  - [ ] `@happy @edge @security` verts
  - [ ] Aucun nouvel `unwrap()` ni `Result<_, String>` introduit
  - [ ] PR contient un script de seed démontrant la repro avant/après (peut être un fichier `.sql` éphémère ou step BDD)

- **Dépendances** : Aucune.
- **Risques** :
  - DTO JSON Decimal serialization → précision perdue côté frontend si pas géré (déjà flag dans la tâche 5). Solution : Decimal → String en JSON ; frontend parse comme `string` (pas `parseFloat`).
  - Tests testcontainers peuvent ne pas capturer panic via logs facilement → fallback assertion implicite.

- **Fichiers** :
  - Modifiés : 3 repositories cités, éventuellement DTOs `UrgentTask`, `BudgetSummary`, `PaymentReminderDTO`.
  - Nouveaux : `backend/tests/features/stats_urgent_tasks.feature` + step impl si pas déjà présent.

---

## Story B : Audit & classification des 30 f64 restants

- **ID** : STORY-521-B | **Type** : Audit | **Taille** : S
- **Issue parent** : #521
- **Output** : **commentaire structuré sur issue #521** (pas de sous-issue, pas d'ADR — décision utilisateur Q2=a) listant chaque occurrence f64 restante avec classification.

- **User Story** : En tant qu'agent IA exécutant le fix C, je veux une classification exhaustive des 30 `f64` restantes dans `backend/src/infrastructure/database/repositories/` distinguant **monétaire** (Decimal obligatoire) de **ratio/coefficient non monétaire** (f64 acceptable), afin que le scope de la Story C soit prévisible et que je ne casse rien d'extra.

- **Tâches techniques** :
  1. [ ] Pour chaque fichier listé (11) :
     - Lister chaque `f64` du fichier avec numéro de ligne
     - Identifier la colonne SQL associée (NUMERIC vs FLOAT8 vs DOUBLE PRECISION) en croisant avec les migrations `backend/migrations/*.sql`
     - Classifier : **(mon)** = monétaire/compta PCMN, **(ratio)** = ratio/coefficient pur, **(autre)** = à clarifier (préciser pourquoi)
  2. [ ] Vérifier hypothèse : tous les `NUMERIC` en DB correspondent à du monétaire (probable mais pas certain — `tantiemes` peuvent être NUMERIC sans être monétaires)
  3. [ ] Compiler dans un tableau Markdown à poster en commentaire sur #521 :
     ```
     | Fichier | Ligne | Variable | Colonne SQL | Type SQL | Classif | Note |
     ```
  4. [ ] Recommandation finale en bas du commentaire : nb monétaire / nb ratio / nb autre + plan de découpe Story C (1 PR groupée ou N PRs par domaine).

- **Scenarios BDD** : N/A (story d'audit, pas de code livrable).

- **Critères d'acceptation** :
  - [ ] Commentaire posté sur #521 avec tableau exhaustif (33 lignes = 30 restantes + 3 déjà fixées en A)
  - [ ] Chaque ligne `(autre)` justifiée
  - [ ] Recommandation de découpe Story C explicite

- **Dépendances** : Aucune (peut tourner en parallèle de A).

- **Fichiers** :
  - Lus uniquement : 11 repositories sous `backend/src/infrastructure/database/repositories/`, migrations sous `backend/migrations/`.
  - Aucun fichier modifié.

---

## Story C : Migration backlog f64 → Decimal

- **ID** : STORY-521-C | **Type** : Refactor | **Taille** : L (à raffiner après B)
- **Issue parent** : #521
- **Dépend de** : Story B (output = scope)

- **User Story** : En tant que codebase KoproGo, je veux que toutes les colonnes monétaires soient décodées en `rust_decimal::Decimal` (pas `f64`), afin que :
  - La règle `project_no-f64-in-money.md` soit appliquée à 100 %
  - Aucun panic latent type `ColumnDecode f64 vs NUMERIC` ne subsiste
  - La précision PCMN (4 décimales) soit préservée bout en bout

- **Scenarios BDD** (4 catégories par domaine touché — à détailler après B) :
  - `@happy` : montant round-trip Decimal exact pour chaque endpoint REST touché
  - `@edge` : montants aux bornes (0, max, 4 décimales)
  - `@security` : RBAC inchangé
  - `@negative` : DB renvoie NULL ou type incohérent → `AppError` typée propre, pas de panic

- **Tâches techniques** : À détailler après Story B. Squelette :
  1. [ ] Migration code par grappe sémantique (ex: budgets, votes, IoT energy, etc. selon découpe B)
  2. [ ] Refactor DTOs JSON impactés (Decimal → String sérialisation cohérente)
  3. [ ] Tests RED-first 4 catégories par endpoint
  4. [ ] Vérif frontend : aucun `parseFloat` sur amount → si présent, ouvrir story complémentaire frontend

- **Critères d'acceptation** :
  - [ ] `grep "f64" backend/src/infrastructure/database/repositories/ | grep -v ratio_non_monetaire_whitelist` → 0
  - [ ] Tous les endpoints touchés ont leurs 4 catégories BDD vertes
  - [ ] PR(s) référencent `Refs #521 / Closes after C completes`

- **Risques** :
  - DTO serialization JSON Decimal → side-effect frontend (string vs number)
  - Tests existants peuvent attendre `f64` en assertion (à mettre à jour)
  - Possible besoin migration SQL si une colonne s'avère FLOAT8 et doit devenir NUMERIC (rare attendu, mais à valider)

- **Dépendances** : Story B signée.

---

## Tracking

- Issue racine : #521
- Stories : STORY-521-A / -B / -C
- PRs (à venir) : 1 PR par story (A et B en parallèle, C séquentiel)

## Refs

- [memory `project_no-f64-in-money.md`](#)
- [memory `feedback_tdd-bdd-four-categories.md`](#)
- [memory `feedback_audit-to-issue-first.md`](#)
- [memory `feedback_maury-token-economy.md`](#)
- [CRITICAL.md §4 et §3](../.claude/rules/CRITICAL.md)
- [Maury/Méthode Maury.md](Méthode%20Maury.md)
