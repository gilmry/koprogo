=========================================================================================
Issue #554: AG cycle de vie : invariants transition Completed + seed world-model cohérent
=========================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug,javascript track:software,priority:high rust,legal-compliance governance
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-06-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/554>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   Bug observé en live : l'AG **« Assemblée Générale Ordinaire 2026 »** affiche le statut **« Terminée »** (badge vert) alors qu'aucune des pré-conditions légales/métier n'est remplie.
   
   C'est une instance concrète de la règle d'agent `validate-before-compute` étendue aux **transitions d'état** (state machine) — pas seulement aux calculs.
   
   ---
   
   ## Bug 1 — État `Completed` atteint sans pré-conditions
   
   ### Constat (screenshot fourni par l'utilisateur)
   
   ```
   Assemblée Générale Ordinaire 2026          [Terminée]
   Type : Ordinaire
   Date : 19 février 2026 à 11:00
   Lieu : Salle polyvalente
   Immeuble : Résidence Grand Place
   Description : Assemblée générale annuelle
   Ordre du jour : Approbation des comptes / Travaux à prévoir / Questions diverses
   
   — Convocations —
   Aucune convocation n'a été créée pour cette assemblée.
   Le syndic n'a pas encore créé de convocation.
   
   — Quorum / Présences —
   Tantièmes présents : 0     Tantièmes totaux : 1000
   Aperçu : 0.0% ⚠️ Quorum non atteint (< 50%)
   
   — Résolutions —
   Aucune résolution pour cette assemblée.
   
   — Documents associés —
   Aucun document associé
   ```
   
   → AG marquée « Terminée » alors que : 0 convocation, 0 quorum validé, 0 résolution, 0 PV.
   
   ### Cause
   
   [backend/src/application/use_cases/meeting_use_cases.rs:142-157](backend/src/application/use_cases/meeting_use_cases.rs#L142-L157) :
   
   ```rust
   pub async fn complete_meeting(
       &self,
       id: Uuid,
       request: CompleteMeetingRequest,
   ) -> Result<MeetingResponse, String> {
       let mut meeting = self.repository.find_by_id(id).await?
           .ok_or_else(|| "Meeting not found".to_string())?;
       meeting.complete(request.attendees_count)?;   // ← AUCUNE pré-condition métier vérifiée
       let updated = self.repository.update(&meeting).await?;
       Ok(MeetingResponse::from(updated))
   }
   ```
   
   `meeting.complete()` se contente d'un changement de status. Pas de check `convocations.is_sent()`, pas de check `quorum.is_validated()`, pas de check `resolutions.all_voted()`, pas de check `pv_attached()`.
   
   (Note hors-scope : ce use-case retourne `Result<_, String>` au lieu de `Result<_, AppError>` typé — violation [CRITICAL.md rule 4](.claude/rules/CRITICAL.md). À corriger dans une slice séparée.)
   
   ### Recette
   
   État cible de la state machine `Meeting` :
   
   ```
   Draft → Scheduled → Convocations_Sent → Held → Completed
                                        ↓
                                     Cancelled
   ```
   
   Invariants de transition vers `Completed` (cf. Art. 3.87 §3-5 CC) :
   
   1. Au moins une `Convocation` créée et `sent_at IS NOT NULL`.
   2. `Quorum.attendees_count` > 0 ET quorum_validated == true (Art. 3.87 §5 — 50% des tantièmes au moins).
   3. Au moins une `Resolution` saisie avec status `Voted` (ou `Adjourned` explicite).
   4. Au moins un `Document` de type `MeetingMinutes` (PV) attaché.
   
   Implémentation :
   
   - Entité domaine `Meeting` : méthode `assert_can_complete(deps: &MeetingCompletionContext) -> Result<(), MeetingError>`.
   - Use-case `complete_meeting` : charger les agrégats nécessaires (convocations, quorum, resolutions, documents) puis `meeting.assert_can_complete(&ctx)?`.
   - Erreur typée : `MeetingNotReadyToComplete { missing: Vec<MissingPiece> }` où `MissingPiece` enum = `Convocations | Quorum | Resolutions | Minutes`.
   - API : 422 Unprocessable Entity avec détail des pièces manquantes.
   - FE : sur le bouton « Marquer terminée », désactiver tant que les 4 conditions ne sont pas remplies, afficher un panneau de complétude (4 checkmarks).
   
   ### Critères d'acceptation
   
   - [ ] `Meeting.complete()` refuse si convocations absentes / non envoyées
   - [ ] `Meeting.complete()` refuse si quorum non validé (Art. 3.87 §5)
   - [ ] `Meeting.complete()` refuse si pas au moins 1 résolution avec statut terminal
   - [ ] `Meeting.complete()` refuse si pas de PV (`Document` type `MeetingMinutes`) attaché
   - [ ] Erreur 422 avec détail `missing: [...]` (jamais 500)
   - [ ] UI : bouton désactivé + checklist visuelle des 4 pré-conditions
   - [ ] Tests BDD 4 catégories : `@happy` (les 4 OK), `@negative` (au moins 1 manquant), `@security` (bypass via API directe → 422), `@edge` (quorum à exactement 50.0%, document attaché mais mauvais type)
   
   ---
   
   ## Bug 2 — Seed incohérent → besoin d'un **world model seed**
   
   ### Constat
   
   Le seed actuel ([backend/src/infrastructure/database/seed.rs](backend/src/infrastructure/database/seed.rs)) produit des entités ponctuelles **sans respecter les invariants métier de leur cycle de vie**.
   
   Conséquences observables :
   - Building avec `total_units=15` mais seulement 3 units créés (cf. #553).
   - AG marquée `Completed` sans convocations / quorum / résolutions / PV (ce ticket).
   - Probable : appels de fonds générés sur buildings non conformes, etc.
   
   Le seed actuel ressemble à un script `INSERT INTO ...` direct qui contourne la couche use-case, donc tous les invariants métier.
   
   ### Proposition — **world model seed**
   
   Un seed qui simule un **monde cohérent** : pas des données ponctuelles, mais un **scénario complet** où chaque entité respecte les invariants de son cycle de vie.
   
   #### Principes
   
   1. **Passer par les use-cases**, pas directement par le repository. Le seed devient un orchestrateur qui appelle les mêmes use-cases que ferait un humain.
   2. **Scénarios narratifs** : « Immeuble A — syndic Sylvie crée, encode tous les lots (SUM=1000), créé une AG avec convocations, simule présences, vote 3 résolutions, ajoute le PV ». À la fin → AG `Completed` cohérente.
   3. **Multi-scénarios** : plusieurs immeubles à différents états du cycle (en cours d'encodage, conforme draft, AG planifiée, AG terminée, conflit, contentieux, etc.) → couvre toutes les facettes UI.
   4. **Idempotent** : peut être ré-exécuté sans dupliquer (use-cases idempotents ou hash check).
   5. **Réutilisable BDD + E2E** : les `Given` BDD et les `setupX` Playwright partent du même builder de monde.
   
   #### Implémentation suggérée (à confirmer en ADR)
   
   - Module `backend/src/infrastructure/seed/world_builder.rs` exposant une API fluent :
     ```rust
     WorldBuilder::new(pool)
         .with_organization("Résidence Grand Place SPRL")
         .with_admin("admin@koprogo.com")
         .with_syndic("sylvie@example.com")
         .with_building("Résidence Grand Place", units: 15, quotas_total: 1000)
             .with_owner("Alice", lots: [101, 103])
             .with_owner("Bob", lots: [102])
             .conform()  // ← force la complétude (15 lots créés, SUM=1000)
         .with_meeting("AG 2026", date: "+30d")
             .with_convocation(sent_at: "+0d")
             .with_quorum(75.0)
             .with_resolution("Approbation des comptes", voted: 100%, approved: true)
             .with_minutes_document()
             .complete()  // ← passe par le use-case complete_meeting (refuse si pas conforme)
         .build()
         .await?;
     ```
   - Le `WorldBuilder` peut produire des scénarios "happy path" mais aussi des scénarios "edge" délibérément non conformes (pour tester les UI de non-conformité, les blocages, etc.) — mais alors le statut métier est honnête (`Draft` / `Pending` au lieu de `Completed`).
   
   ### Critères d'acceptation
   
   - [ ] Module `world_builder` (ou équivalent) qui passe par les use-cases, pas direct au repository
   - [ ] Au moins 3 scénarios pré-définis : "happy syndic", "draft non-conforme", "AG en cours"
   - [ ] Le seed actuel `seed.rs` est ré-écrit pour utiliser `WorldBuilder` (ou supprimé au profit de lui)
   - [ ] Tests BDD `@happy` partent d'un `WorldBuilder` (pattern réutilisable)
   - [ ] Playwright `loginAsSyndicWithMeeting` peut s'appuyer sur un `WorldBuilder` côté backend (un endpoint admin de seed paramétrable, OU un fixture pré-populée)
   - [ ] ADR documente le choix d'architecture du seed et la liste des scénarios canoniques
   
   ---
   
   ## Liens
   
   - Issue #553 (admin garant conformité buildings + drift quotas)
   - Mémoires d'agent :
     - `project_validate-before-compute.md` (cette transition d'AG en est une instance — étendre à `validate-before-transition`)
     - `project_admin-publishes-conform-buildings.md`
     - `project_no-f64-in-money.md`
   - Cadre légal : Art. 3.87 §3-5 du Code civil belge (quorum, convocations, PV)
   - Documents existants : [docs/CONVOCATIONS_AG.rst](docs/CONVOCATIONS_AG.rst)
   
   ## Hors-scope
   
   - Le `Result<_, String>` du use-case `complete_meeting` doit être migré à `Result<_, AppError>` — slice séparée (rule 4 CRITICAL.md).
   - Pas lié à #549 (gate go-live), #550 (auth), #552 (work-reports 400).

.. raw:: html

   </div>

