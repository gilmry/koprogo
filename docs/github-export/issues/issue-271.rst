====================================================================
Issue #271: fix(legal): Quorum 50%+ validation AG (Art. 3.87 §5 CC)
====================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: bug,priority:critical legal-compliance,governance release:0.1.0
:Assignees: Unassigned
:Created: 2026-03-11
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/271>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Problème
   Lacune CRITIQUE identifiée dans `docs/legal/audit_conformite.rst` : aucune validation du quorum avant vote en AG.
   
   ## Impact
   Sans cette validation, des résolutions peuvent être adoptées illégalement sans que la moitié des quotes-parts soit représentée. Risque de nullité des décisions (Art. 3.92 §3 — délai 4 mois pour contestation).
   
   ## Solution
   - Ajouter `quorum_validated: bool`, `quorum_percentage: f64` à l'entité `Meeting`
   - Use case `validate_quorum(meeting_id)` : calcule % quotes-parts présentes + représentées par procuration
   - Bloquer la création de résolutions si quorum < 50% (`is_valid_for_voting()`)
   - Workflow 2e convocation déclenchée si quorum non atteint (voir issue liée)
   - Migration : `add_quorum_to_meetings.sql`
   
   ## Base légale
   **Art. 3.87 §5 CC** : L'assemblée générale est valablement constituée si plus de la moitié des copropriétaires, représentant plus de la moitié des quotes-parts, sont présents ou représentés.
   
   ## Fichiers à modifier
   - `backend/src/domain/entities/meeting.rs`
   - `backend/src/application/use_cases/resolution_use_cases.rs`
   - `backend/migrations/YYYYMMDD_add_quorum_to_meetings.sql`
   
   ## Definition of Done
   - [ ] Tests unitaires domain entity (Meeting::validate_quorum)
   - [ ] Erreur métier explicite si quorum non atteint lors d'un vote
   - [ ] `cargo test --lib` passe
   - [ ] Matrice conformité mise à jour : quorum → CONFORME

.. raw:: html

   </div>

