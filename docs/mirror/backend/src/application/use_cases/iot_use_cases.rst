======================================
application/use_cases/iot_use_cases.rs
======================================

:Fichier: ``backend/src/application/use_cases/iot_use_cases.rs``
:Type: RUST
:Lignes de Code: 652
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **iot**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``IoTUseCases``
- ``LinkyUseCases``

Fonctions
---------

- ``new()``
- ``create_reading()``
- ``create_readings_bulk()``
- ``query_readings()``
- ``get_consumption_stats()``
- ``get_daily_aggregates()``
- ``get_monthly_aggregates()``
- ``detect_anomalies()``
- ``new()``
- ``configure_linky_device()``
- ``sync_linky_data()``
- ``get_linky_device()``
- ``delete_linky_device()``
- ``toggle_sync()``
- ``find_devices_needing_sync()``

*... et 1 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/iot_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

