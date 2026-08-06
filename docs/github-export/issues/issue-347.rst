========================================================================================
Issue #347: Seeds multi-roles: creer 8 jeux de donnees avec faker + teardown (etape 2/5)
========================================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: enhancement,priority:high testing
:Assignees: Unassigned
:Created: 2026-03-28
:Updated: 2026-03-28
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/347>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Etape 2 : Seeds backend avec faker + teardown
   
   Parent : #345
   Depend de : #346 (specs multi-roles)
   
   ### Objectif
   
   Creer des jeux de donnees pre-configures dans `backend/src/infrastructure/database/seed.rs` pour chaque workflow multi-role. Chaque seed :
   - Respecte les invariants du domaine (quorum valide, tantiemes, ownership)
   - Utilise la crate `fake` (deja importee) pour les donnees realistes
   - Est idempotent (verifie si existe avant de creer)
   - Fournit un endpoint de cleanup pour le teardown des tests
   
   ### Seeds a creer
   
   | Seed | Contenu | Pre-conditions domaine |
   |------|---------|----------------------|
   | `seed_ag_with_quorum` | Building + 3 units (300/200/500 tantiemes) + 3 owners assignes + meeting 2e convocation (quorum exempt) + 1 resolution Pending | Meeting.is_second_convocation=true |
   | `seed_sel_marketplace` | Building + 2 owners (Alice/Bob) avec credit balances + 3 echanges (Offered/Requested/Completed) | OwnerCreditBalance initialise |
   | `seed_ticket_workflow` | Building + 1 owner + 1 ticket Open + 1 contractor (magic link) | Ticket avec requester_id |
   | `seed_poll_active` | Building + 1 owner + 1 poll Active (YesNo) avec options | Poll.status=Active, dates valides |
   | `seed_notice_board` | Building + 3 notices publiees + 1 brouillon | Notice.status=Published |
   | `seed_quote_comparison` | Building + 3 quotes soumises (montants/durees/garanties differents) | Quotes en status Received |
   | `seed_age_request` | Building + 3 owners avec tantiemes + 1 demande AGE en status Open | AgeRequest prete pour cosignatures |
   | `seed_expense_approval` | Building + 1 expense Draft + accounts PCMN | Expense prete pour workflow approbation |
   
   ### API endpoints
   
   ```
   POST /seed/scenario/{name}     — Cree le seed (SuperAdmin only)
   DELETE /seed/scenario/{name}   — Supprime les donnees seedees (teardown)
   ```
   
   Chaque seed retourne un JSON avec les IDs crees :
   ```json
   {
     "org_id": "...",
     "building_id": "...",
     "owner_ids": ["...", "..."],
     "meeting_id": "...",
     "resolution_id": "...",
     "syndic_email": "...",
     "syndic_password": "...",
     "owner_email": "...",
     "owner_password": "..."
   }
   ```
   
   Le teardown supprime dans l'ordre inverse des FK (resolutions, meetings, owners, units, building, users, org).
   
   ### Implementation
   
   - Etendre `DatabaseSeeder` dans `seed.rs` avec une methode par scenario
   - Ajouter les endpoints dans `seed_handlers.rs`
   - Marquer les donnees avec `is_seed_data = true` pour le cleanup
   - Utiliser `fake::faker` pour noms, adresses, emails realistes belges
   
   ### Definition of Done
   
   - [ ] 8 seeds crees dans seed.rs
   - [ ] Endpoint POST /seed/scenario/{name} fonctionnel
   - [ ] Endpoint DELETE /seed/scenario/{name} (teardown) fonctionnel
   - [ ] Chaque seed respecte les invariants domaine
   - [ ] Tests unitaires pour chaque seed
   - [ ] Les scenarios E2E beforeAll utilisent les seeds au lieu de creer manuellement

.. raw:: html

   </div>

