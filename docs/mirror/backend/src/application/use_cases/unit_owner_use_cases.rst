=============================================
application/use_cases/unit_owner_use_cases.rs
=============================================

:Fichier: ``backend/src/application/use_cases/unit_owner_use_cases.rs``
:Type: RUST
:Lignes de Code: 315
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **unit owner**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``UnitOwnerUseCases``

Fonctions
---------

- ``new()``
- ``add_owner_to_unit()``
- ``remove_owner_from_unit()``
- ``update_ownership_percentage()``
- ``transfer_ownership()``
- ``get_unit_owners()``
- ``get_owner_units()``
- ``get_unit_ownership_history()``
- ``get_owner_ownership_history()``
- ``set_primary_contact()``
- ``get_unit_owner()``
- ``has_active_owners()``
- ``get_total_ownership_percentage()``

Code Source
===========

Voir: ``backend/src/application/use_cases/unit_owner_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

