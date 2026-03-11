===========================================
infrastructure/web/handlers/iot_handlers.rs
===========================================

:Fichier: ``backend/src/infrastructure/web/handlers/iot_handlers.rs``
:Type: RUST
:Lignes de Code: 535
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **iot**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_iot_reading()``
- ``create_iot_readings_bulk()``
- ``query_iot_readings()``
- ``get_consumption_stats()``
- ``get_daily_aggregates()``
- ``get_monthly_aggregates()``
- ``detect_anomalies()``
- ``configure_linky_device()``
- ``get_linky_device()``
- ``delete_linky_device()``
- ``sync_linky_data()``
- ``toggle_linky_sync()``
- ``find_devices_needing_sync()``
- ``find_devices_with_expired_tokens()``
- ``configure_routes()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/iot_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

