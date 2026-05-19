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
| A | Fix 3 panics latents (stats/budget/payment-reminder) + migration `Result<_, String>` → `AppError` aux 3 niveaux (port/use_case/impl) | Bugfix + Refactor | L (2-3j) | `/stats/syndic/urgent-tasks` opérationnel |
| B | Audit & classification des 30 f64 restants | Audit | S (~0.5j) | Scope clair pour Story C |
| C | Migration backlog f64 → Decimal | Refactor | L (dépend B) | Conformité `no-f64-in-money` complète |

**Ordre** : A débloque le 502 immédiat. B sans dépendance sur A. C dépend de B (output classification).

---

## Story A : Fix panics latents f64 sur colonnes NUMERIC (3 endpoints)

- **ID** : STORY-521-A | **Type** : Bugfix + Refactor | **Taille** : L (2-3j)
- **Issue parent** : #521
- **Scope élargi (audit 2026-05-13)** : la violation `Result<_, String>` (CRITICAL.md §4) existe AUSSI au niveau trait port et use case, pas seulement repository impl. Donc 9 signatures à migrer (3 niveaux × 3 features : stats / budget / payment_reminder) en plus du f64 → Decimal.
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
  6. [ ] **Migrer `Result<_, String>` → `Result<_, AppError>`** sur les 9 signatures (3 features × 3 niveaux) :
     - Trait port : `stats_repository.rs`, `budget_repository.rs`, `payment_reminder_repository.rs`
     - Use case : `stats_use_cases.rs`, `budget_use_cases.rs`, `payment_reminder_use_cases.rs` (+ mocks dans tests in-module)
     - Impl : 3 fichiers `*_repository_impl.rs`
     - Handlers REST appelants : mettre à jour le mapping `AppError → HTTP` si pas déjà via `ResponseError`
  7. [ ] Vérifier que les mocks `#[cfg(test)] mod tests` dans les 3 use_cases (lignes ~120 dans stats_use_cases.rs) sont mis à jour avec la nouvelle signature.
  8. [ ] **GREEN** : `make ci` doit passer ; BDD `@bug521` doit virer rouge → vert.
  9. [ ] Vérif manuelle browser : reload `/syndic` dashboard, aucun 502.
  10. [ ] Commit + PR vers `feature/dev` (per GitFlow buffer, memory `project_gitflow-feature-dev-buffer.md`) avec body référençant `Refs #521` et le scénario BDD.

- **Critères d'acceptation** :
  - [ ] `grep "let.*: f64 = .*\.get(" backend/src/infrastructure/database/repositories/{stats,budget,payment_reminder}_repository_impl.rs` → 0 résultat
  - [ ] `grep "Result<.*,\s*String>" backend/src/application/{ports,use_cases}/{stats,budget,payment_reminder}*` → 0 résultat
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

## Story C : Migration backlog f64 → Decimal (3 grappes)

Story B (audit, [commentaire #521](https://github.com/gilmry/koprogo/issues/521)) a découpé C en **3 sous-stories par grappe sémantique**. Aucune n'est une décision : ADR-0007/0008/0009 (Accepted) tranchent déjà. C = remédiation de non-conformité, pas itération de directive.

| Sous-story | Grappe | Issue tracking | Priorité |
|---|---|---|---|
| C1 | Gouvernance (quota/voting power/AGE shares/quorum) | **#534** (non-conformité ADR-0008), ferme **#525** | Haute (panic actif + enjeu légal) |
| C2 | Monétaire calc (payment_reminder/budget/stats) | (à ouvrir) | Haute (perte précision PCMN) |
| C3 | Évaluations (contract_evaluation/service_provider) | (à ouvrir) | Moyenne |

---

### Story C1 : Remédiation ADR-0008 — Gouvernance

- **ID** : STORY-521-C1 | **Type** : Refactor + Migration SQL | **Taille** : L (2-3j)
- **Issue parent** : #521 · **Tracking** : #534 · **Ferme** : #525
- **Directive source (déjà signée, pas de re-décision)** : [ADR-0008](../docs/adr/0008-numeric-vs-double-precision-postgresql.md) « Pourcentages (quotités, taux, voting power) → `NUMERIC(7,4)` » + [ADR-0007](../docs/adr/0007-decimal-vs-f64-for-money.md) (f64 interdit hors IoT/énergie [ADR-0009](../docs/adr/0009-iot-energy-keep-f64.md)).

- **User Story** : En tant que syndic clôturant une AG, je veux que le calcul de quorum et de majorité (Art. 3.87 §5 CC) repose sur une arithmétique décimale exacte, afin qu'un arrondi IEEE754 ne rende pas une délibération juridiquement contestable.

- **Scenarios BDD** (4 catégories — `@security`/`@edge` **juridiquement critiques**) :

```gherkin
@negative @adr8 @story521-C1
Scenario: voting_power decodé en Decimal ne panique pas (regression #525)
  Given une résolution avec des votes de puissance "12.3456" et "7.8900"
  When le syndic agrège les puissances de vote
  Then l'opération réussit sans panic ColumnDecode
  And la somme est exactement "20.2356" (pas d'arrondi binaire)

@security @adr8 @story521-C1
Scenario: quorum à la borne légale exacte n'est pas faussé par arrondi
  Given un immeuble de total quotas "1000.0000"
  And des présents cumulant "500.0001" quotas
  When le syndic vérifie le quorum (seuil 50% Art. 3.87 §5 CC)
  Then le quorum est ATTEINT (500.0001 > 500.0000 exact, pas 500.0 == 500.0)

@edge @adr8 @story521-C1
Scenario Outline: quotités aux bornes PCMN préservées
  Given une quotité "<q>" sur units.quota
  When l'unité est relue depuis la DB
  Then la quotité vaut exactement "<q>"
  Examples:
    | q          |
    | 0.0001     |
    | 999.9999   |
    | 333.3333   |

@happy @adr8 @story521-C1
Scenario: AGE request shares_pct exact
  Given une AGE request avec shares_pct "0.250000"
  When elle est relue
  Then shares_pct vaut exactement "0.250000"
```

- **Tâches techniques** (suivre § Migration pattern d'ADR-0008) :
  1. [ ] **RED** : écrire les 4 scénarios ci-dessus (feature `governance_decimal.feature` ou extension existante), vérifier rouge contre `feature/dev` (panic #525 sur @negative).
  2. [ ] Migration SQL `migrations/2026..._alter_governance_to_numeric.sql` : `units.quota`, `meetings.total_quotas`, `meetings.present_quotas` `DOUBLE PRECISION → NUMERIC(10,4)` (idempotent, `USING col::NUMERIC(10,4)`).
  3. [ ] Entités `f64 → rust_decimal::Decimal` : `AgeRequestSignature.shares_pct`, `EtatDate.ordinary_charges_quota`/`extraordinary_charges_quota` (+ invariants `::new()`).
  4. [ ] Repos : `unit_repository_impl` (lecture Decimal — ferme #525), `age_request_repository_impl:31,32,52` (retirer `row.get::<f64>`), `etat_date_repository_impl:541,542`, `vote_repository_impl` (retirer casts `::FLOAT8`, lecture Decimal), `resolution_repository_impl:323-325` (params `Decimal`).
  5. [ ] Cascade : use_cases + handlers + DTOs gouvernance (sérialisation Decimal→String JSON cohérente, cf. Story A).
  6. [ ] **GREEN** : `make ci` + BDD 4 catégories vertes ; @negative #525 rouge→vert.
  7. [ ] Vérif `sqlx migrate run` propre sur DB neuve (la migration ALTER ne casse pas le schéma).

- **Critères d'acceptation** :
  - [ ] `grep -rn "DOUBLE PRECISION" backend/migrations/` → 0 sur colonnes gouvernance
  - [ ] `grep -rnE "::FLOAT8|get::<f64" backend/src/infrastructure/database/repositories/{unit,age_request,vote,resolution,etat_date}_repository_impl.rs` → 0
  - [ ] 0 `f64` sur champ entité à poids légal (gouvernance)
  - [ ] BDD `@adr8 @story521-C1` 4 catégories vertes (dont @security quorum borne exacte)
  - [ ] #525 fermé ; #534 critères cochés
  - [ ] `sqlx migrate run` clean sur base vierge

- **Risques** :
  - ALTER COLUMN sur données existantes : v0.1.0 sans prod (memory `project_koprogo-current-state.md`) → pas de perte réelle, mais tester migration neuve + rollback mental.
  - Enjeu légal : un test @security faux donnerait fausse confiance sur quorum. Le scénario borne exacte (500.0001 > 500.0000) est le garde-fou — ne pas l'affaiblir pour faire passer.
  - DTO JSON Decimal→String : même précaution que Story A (frontend ne doit pas `parseFloat`).

- **Dépendances** : ADR-0008 (signé ✅). Indépendant de C2/C3.

---

### Story C2 : Monétaire calc (payment_reminder / budget / stats)

- **ID** : STORY-521-C2 | **Taille** : M-L | **Issue** : à ouvrir
- Scope : ~10 occ. 🟠 MON-CALC — supprimer downcast `.to_f64()`, retours `f64`→`Decimal` end-to-end (`get_total_owed_by_organization`, `budget` projection, `stats pending_total`). Pas de panic actif (try_get/unwrap_or) mais viole ADR-0007.
- À détailler quand C1 mergée (réutilise le pattern Story A/C1).

### Story C3 : Évaluations (contract_evaluation / service_provider)

- **ID** : STORY-521-C3 | **Taille** : S-M | **Issue** : à ouvrir
- Scope : `avg_score`/`global_score`/`rating_avg` NUMERIC(3,2) lus en f64 → Decimal. Panic risk mais non-légal → priorité moyenne.

---

## Tracking

- Issue racine : #521 · Non-conformité ADR-0008 : #534 · Panic quota : #525
- Stories : STORY-521-A ✅ (PR #532 merged) / -B ✅ (audit #521) / -C1 (this) / -C2 / -C3
- Effets de bord : #524 ✅ (BDD harness) · #526 (amount=0, domaine)
- PRs : A=#532 ✅ · C1/C2/C3 = 1 PR chacune vers `feature/dev`

## Refs

- [memory `project_no-f64-in-money.md`](#)
- [memory `feedback_tdd-bdd-four-categories.md`](#)
- [memory `feedback_audit-to-issue-first.md`](#)
- [memory `feedback_maury-token-economy.md`](#)
- [CRITICAL.md §4 et §3](../.claude/rules/CRITICAL.md)
- [Maury/Méthode Maury.md](Méthode%20Maury.md)
