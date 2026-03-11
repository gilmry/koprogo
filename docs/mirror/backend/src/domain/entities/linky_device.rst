===============================
domain/entities/linky_device.rs
===============================

:Fichier: ``backend/src/domain/entities/linky_device.rs``
:Type: RUST
:Lignes de Code: 442
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **linky device**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``LinkyDevice``

Énumérations
------------

- ``LinkyProvider``

Fonctions
---------

- ``new()``
- ``with_refresh_token()``
- ``set_sync_enabled()``
- ``enable_sync()``
- ``disable_sync()``
- ``mark_synced()``
- ``update_tokens()``
- ``is_token_expired()``
- ``needs_sync()``
- ``api_endpoint()``

Code Source
===========

Voir: ``backend/src/domain/entities/linky_device.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

