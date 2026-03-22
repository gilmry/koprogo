================================================================================================
Issue #274: feat: AG Visioconférence — entité AgSession + quorum combiné (Art. 3.87 §1 CC)
================================================================================================

:State: **OPEN**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,track:software legal-compliance,governance release:0.1.0
:Assignees: Unassigned
:Created: 2026-03-11
:Updated: 2026-03-14
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/274>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   **Art. 3.87 §1 CC** autorise explicitement la participation à l'AG *«physiquement ou à distance, si la convocation le prévoit»*. La base légale est déjà documentée dans `docs/legal/coproprietaire.rst` (CP01).
   
   Clôture R&D #237.
   
   ## Livrables
   
   ### Backend
   - Nouvelle entité `AgSession` : `meeting_id`, `platform` (enum Jitsi/Teams/Zoom/Meet/Custom), `video_url`, `status` (Scheduled/Live/Ended), `quorum_remote_pct`, `participants_count_remote`, `recording_url`
   - Use cases : `create_session`, `start_session`, `end_session`, `register_remote_participant`, `calculate_combined_quorum`, `generate_attendance_report`
   - 7 endpoints REST :
     - `POST /meetings/:id/ag-session`
     - `GET /meetings/:id/ag-session`
     - `PUT /ag-sessions/:id/start`
     - `PUT /ag-sessions/:id/end`
     - `POST /ag-sessions/:id/join`
     - `GET /ag-sessions/:id/participants`
     - `GET /ag-sessions/:id/quorum`
   - Migration : `create_ag_sessions.sql`
   
   ### Convocation enrichie
   Ajouter à l'entité `Convocation` : `has_video_option: bool`, `video_platform`, `video_url`
   
   ### Frontend
   - Page AG avec lien visio et QR code
   - Indicateur quorum combiné (présentiel + distanciel)
   
   ## Calcul quorum combiné
   `quorum_combined = quotes_parts_present_physically + quotes_parts_remote_participants`
   Validé si > 50% (ou sans minimum pour 2e convocation — issue #272)
   
   ## Closes
   #237
   
   ## Definition of Done
   - [ ] AgSession entity avec tests unitaires
   - [ ] calculate_combined_quorum() retourne valeur correcte
   - [ ] Convocation enrichie avec champs vidéo
   - [ ] 7 endpoints REST opérationnels

.. raw:: html

   </div>

