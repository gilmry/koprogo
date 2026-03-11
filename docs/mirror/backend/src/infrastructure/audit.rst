=======================
infrastructure/audit.rs
=======================

:Fichier: ``backend/src/infrastructure/audit.rs``
:Type: RUST
:Lignes de Code: 455
:Couche: Infrastructure (Adaptateurs)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Module Rust **audit**. Fait partie de la couche Infrastructure (Adaptateurs).

API Publique
============

Structures
----------

- ``AuditLogEntry``

Énumérations
------------

- ``AuditEventType``

Fonctions
---------

- ``new()``
- ``with_resource()``
- ``with_client_info()``
- ``with_metadata()``
- ``with_error()``
- ``with_details()``
- ``log()``
- ``log_audit_event()``

Code Source
===========

Voir: ``backend/src/infrastructure/audit.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

