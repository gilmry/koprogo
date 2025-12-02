==================================================
infrastructure/web/handlers/admin_gdpr_handlers.rs
==================================================

:Fichier: ``backend/src/infrastructure/web/handlers/admin_gdpr_handlers.rs``
:Type: RUST
:Lignes de Code: 467
:Couche: Infrastructure (Adaptateurs)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **admin gdpr**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Structures
----------

- ``AuditLogQuery``
- ``AuditLogsResponse``
- ``AuditLogDto``

Fonctions
---------

- ``list_audit_logs()``
- ``admin_export_user_data()``
- ``admin_erase_user_data()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/admin_gdpr_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

