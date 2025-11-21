=====================================================
application/use_cases/owner_contribution_use_cases.rs
=====================================================

:Fichier: ``backend/src/application/use_cases/owner_contribution_use_cases.rs``
:Type: RUST
:Lignes de Code: 122
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **owner contribution**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``OwnerContributionUseCases``

Fonctions
---------

- ``new()``
- ``create_contribution()``
- ``record_payment()``
- ``get_contribution()``
- ``get_contributions_by_organization()``
- ``get_contributions_by_owner()``
- ``get_outstanding_contributions()``
- ``get_overdue_contributions()``
- ``get_outstanding_amount()``

Code Source
===========

Voir: ``backend/src/application/use_cases/owner_contribution_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

