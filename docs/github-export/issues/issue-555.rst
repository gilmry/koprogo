======================================================================================================
Issue #555: EPIC: migrer Result<_, String> → Result<_, AppError> (1263 violations, CRITICAL.md rule 4)
======================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug,track:software priority:medium,rust
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/555>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Violation systémique de **CRITICAL.md rule 4** :
   > `Result<E>` typé, pas `Result<_, String>`. `unwrap()` / `expect()` interdits hors tests — utiliser `?` + `AppError`.
   
   ### Ampleur (mesurée)
   
   ```
   backend/src/application/     1021 occurrences de Result<_, String>
   backend/src/domain/           242 occurrences de Result<_, String>
   ─────────────────────────────────────
   Total                        ~1263 violations
   ```
   
   Affecte **57 ports** et **51 use_cases** (quasi tous les domaines : auth, building, meeting, expense, owner, ticket, gdpr, etc.).
   
   ### Symptômes pratiques observés
   
   - **Loss-of-context** : `format!("Meeting not found")` à la place d'un `AppError::NotFound { resource: "Meeting", id }` typé → impossible de mapper vers le bon code HTTP côté handler.
   - **Lossy bridge** : `impl From<String> for AppError` traite tout en `AppError::Internal(500)` (vu dans la dernière analyse #549 — résumé conversation).
   - **Pas de différenciation** entre erreur métier (`AssertionFailed`, `Conflict`, `NotFound`) et erreur infra (`DatabaseError`, `Timeout`).
   - **Hook PostToolUse warn** sur introduction → on continue à ajouter du `Result<_, String>` malgré le hook (1263 fois).
   
   Instance la plus récente : `meeting_use_cases.rs:142-157` `complete_meeting` → `Result<MeetingResponse, String>`, devrait être `Result<MeetingResponse, AppError>` avec variant `MeetingNotReadyToComplete{missing:Vec<MissingPiece>}` typé (cf. [#554](https://github.com/gilmry/koprogo/issues/554)).
   
   ## Cause
   
   Historique : le projet a démarré avec `Result<T, String>` (rapide, simple) avant que `AppError` ne soit défini dans `application/error.rs`. Personne n'a fait la migration à grande échelle ; chaque nouvelle PR perpétue le pattern par mimétisme.
   
   ## Recette (migration par bounded context)
   
   ### Approche
   
   **PAS** de PR géante touchant 1263 lignes — refactor itératif **par bounded context** (1 PR = 1 contexte) :
   
   1. **Étape 0** : Sceller un convention guide (`docs/architecture/error_handling.md` ou ADR-0003) :
      - Domaine : enum d'erreur par entité (ex. `VoteError`, `MeetingError`, `BuildingError`)
      - Application : `AppError` consume les enums domaines via `impl From<DomainError> for AppError`
      - Infra : map `sqlx::Error` → `AppError::DatabaseError`
      - Web handlers : `AppError` → HTTP status via `ResponseError`
   
   2. **Étape 1 — Stop the bleeding** :
      - Hook PreToolUse **deny** sur introduction de nouveau `Result<_, String>` dans `application/` et `domain/`
      - Tolérance temporaire dans `infrastructure/` (mapping sqlx)
      - Clippy lint custom ou regex CI fail
   
   3. **Étapes 2..N — Migration progressive** (1 contexte / 1 PR, ~50-150 lignes touchées par PR) :
      - Ordre suggéré (criticité juridique décroissante) :
        1. `meeting_use_cases` + `resolution_use_cases` + `vote_use_cases` (AG / quorum — Art. 3.87 CC)
        2. `expense_use_cases` + `journal_entry_use_cases` + `call_for_funds_use_cases` (PCMN belge)
        3. `building_use_cases` + `unit_use_cases` + `owner_use_cases` (fiches d'identité — cf. #553)
        4. `auth_use_cases` + `gdpr_use_cases` (sécurité)
        5. ... le reste
   
   4. **Étape N+1 — Lock** : enlever la tolérance infra, retirer `impl From<String> for AppError`.
   
   ### Précédent
   
   Cf. `domain/entities/vote.rs` qui définit déjà `ProxyValidationError` — pattern correct à généraliser.
   
   ### Tests
   
   Chaque PR de migration doit :
   - Conserver les tests existants verts
   - Ajouter des tests `@negative` qui assertent le **variant exact** de l'erreur (`assert!(matches!(err, AppError::NotFound { .. }))`) — pas juste `is_err()`
   - Pas de régression sur les codes HTTP côté handlers (couverts par tests intégration / contract-types)
   
   ## Critères d'acceptation (par sous-issue / PR de migration)
   
   - [ ] 0 nouveau `Result<_, String>` introduit (hook deny + CI fail)
   - [ ] Bounded context X migré entièrement (use_cases + ports + repository impls)
   - [ ] Tests `@negative` assertent le variant exact d'erreur
   - [ ] Pas de régression sur les codes HTTP des endpoints du contexte
   - [ ] Documentation ADR mise à jour si nouveau pattern d'erreur émerge
   
   ## Critères d'acceptation (epic global)
   
   - [ ] **0** occurrence de `Result<_, String>` dans `backend/src/application/` (sauf bridging temporaire documenté)
   - [ ] **0** occurrence dans `backend/src/domain/`
   - [ ] `impl From<String> for AppError` retiré (lossy bridge)
   - [ ] ADR `docs/adr/0003-typed-errors-everywhere.md` (numéro à confirmer) publié
   - [ ] Hook PreToolUse deny actif et testé
   - [ ] Métrique CI : `grep -rn "Result<[^>]*, String>" backend/src/application backend/src/domain | wc -l` retourne 0
   
   ## Hors-scope
   
   - Pas un blocker #549 (gate go-live) — la migration peut se faire dans plusieurs releases.
   - Pas lié à #550/#552/#553/#554 (ces issues l'utilisent comme observation mais ne le résolvent pas).
   - Ne touche pas `infrastructure/` initialement (tolérance temporaire le temps de l'epic).
   
   ## Priorité
   
   - **medium** — pas un blocker fonctionnel, mais bloque la qualité long terme (mappage HTTP correct, observabilité erreurs, robustesse refactor).
   - **À démarrer après stabilisation #549 / #550 / #553 / #554** (sinon trop de fronts ouverts).
   
   ## Liens
   
   - [CRITICAL.md règle 4](.claude/rules/CRITICAL.md)
   - Issue #554 (instance la plus récente — `complete_meeting`)
   - Précédent correct : `backend/src/domain/entities/vote.rs` (`ProxyValidationError`)

.. raw:: html

   </div>

