=======================
domain/entities/user.rs
=======================

:Fichier: ``backend/src/domain/entities/user.rs``
:Type: RUST
:Lignes de Code: 519
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **user**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``User``

Énumérations
------------

- ``UserRole``

Fonctions
---------

- ``new()``
- ``full_name()``
- ``update_profile()``
- ``deactivate()``
- ``activate()``
- ``can_access_building()``
- ``rectify_data()``
- ``restrict_processing()``
- ``unrestrict_processing()``
- ``set_marketing_opt_out()``
- ``can_process_data()``
- ``can_send_marketing()``

Code Source
===========

Voir: ``backend/src/domain/entities/user.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

