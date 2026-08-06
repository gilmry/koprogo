================================================================================================
Issue #275: feat: Backoffice prestataires PWA — photo + compte-rendu + validation CdC → paiement
================================================================================================

:State: **CLOSED**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,track:software maintenance,release:0.1.0
:Assignees: Unassigned
:Created: 2026-03-11
:Updated: 2026-03-24
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/275>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   Interface ultra-simplifiée pour corps de métier accessible via **magic link JWT 72h** (sans compte). La validation par le Conseil de Copropriété déclenche automatiquement le paiement, rétablissant la confiance entre parties.
   
   Clôture R&D #235.
   
   ## Workflow complet
   ```
   Ticket/Quote assigné → Magic link envoyé au corps de métier (email)
           ↓
   [PWA Corps de Métier — mobile-first]
     1. Voir description de la mission
     2. Photos avant travaux (Camera API)
     3. Réaliser les travaux
     4. Photos après + pièces remplacées (nom, réf, quantité, photo)
     5. Compte-rendu (texte ou voice-to-text Web Speech API)
     6. Soumettre
           ↓
   [Notification CdC + Syndic]
     - Voir photos avant/après côte à côte
     - Valider OU demander corrections (avec commentaire)
           ↓
   [Si validé → automatiquement]
     → Payment use case déclenché
     → Ticket → status Closed
     → Trigger ContractEvaluation (satisfaction survey L13)
     → Visible copropriétaires dans espace commun
   ```
   
   ## Livrables
   
   ### Backend
   - Entité `ContractorReport` : `ticket_id`, `quote_id`, `contractor_id`, `building_id`, `work_date`, `status` (Draft/Submitted/UnderReview/Validated/Rejected/RequiresCorrection), `compte_rendu`, `photos_before[]`, `photos_after[]`, `parts_replaced[]` (JSON)
   - Magic link JWT 72h : `POST /contractor-reports/magic-link`, `GET /contractor-reports/token/:token`
   - 10 endpoints REST
   - Migration : `create_contractor_reports.sql`
   
   ### Frontend PWA
   - `frontend/src/pages/contractor/[token].astro`
   - Camera API native (`navigator.mediaDevices`)
   - Voice-to-text (Web Speech API)
   - Offline capable (IndexedDB) → sync réseau
   - Taille JS < 50KB
   
   ## Closes
   #235
   
   ## Definition of Done
   - [ ] ContractorReport entity avec state machine
   - [ ] Magic link JWT 72h généré et validé
   - [ ] Workflow Draft → Validated → Payment déclenché
   - [ ] PWA testée sur mobile (Chrome DevTools Device Mode)

.. raw:: html

   </div>

