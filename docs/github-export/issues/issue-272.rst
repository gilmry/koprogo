========================================================================================
Issue #272: fix(legal): Workflow 2e convocation si quorum non atteint (Art. 3.87 §5 CC)
========================================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: bug,priority:critical legal-compliance,governance release:0.1.0
:Assignees: Unassigned
:Created: 2026-03-11
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/272>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Problème
   Lacune CRITIQUE : aucun workflow de 2e convocation dans `convocation.rs`. Si le quorum n'est pas atteint à une AG, il est légalement requis d'organiser une 2e convocation avec des règles différentes.
   
   ## Règles légales
   - **Délai minimum** : 15 jours après la 1ère convocation (même délai que convocation ordinaire)
   - **Quorum 2e AG** : Pas de quorum minimum requis — l'AG délibère valablement quel que soit le nombre de présents
   - **Même OdJ** : La 2e convocation ne peut traiter que les points inscrits à l'OdJ de la 1ère
   
   ## Solution
   - Ajouter `ConvocationType::SecondConvocation` (variant de l'enum existant)
   - Use case `schedule_second_convocation(meeting_id, first_convocation_id)` déclenché depuis `validate_quorum()`
   - Validation : date 2e convocation ≥ date 1ère convocation + 15 jours
   - Flag `no_quorum_required: bool` sur la 2e convocation (pour que le workflow de vote ne bloque pas)
   
   ## Fichiers à modifier
   - `backend/src/domain/entities/convocation.rs`
   - `backend/src/application/use_cases/convocation_use_cases.rs`
   - `backend/migrations/YYYYMMDD_add_second_convocation_type.sql`
   
   ## Lien
   Prérequis : #271 (quorum validation doit exister pour déclencher la 2e convocation)
   
   ## Definition of Done
   - [ ] ConvocationType::SecondConvocation implémenté
   - [ ] Délai 15j validé en domain entity
   - [ ] schedule_second_convocation() déclenché automatiquement si quorum KO
   - [ ] Tests unitaires couvrent le scénario quorum KO → 2e convocation programmée

.. raw:: html

   </div>

