==============================
domain/entities/iot_reading.rs
==============================

:Fichier: ``backend/src/domain/entities/iot_reading.rs``
:Type: RUST
:Lignes de Code: 485
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant une **lecture IoT** (compteur Linky/Ores). Stockage TimescaleDB avec compression et détection anomalies.

API Publique
============

Structures
----------

- ``IoTReading``

Énumérations
------------

- ``DeviceType``
- ``MetricType``

Fonctions
---------

- ``new()``
- ``with_metadata()``
- ``is_anomalous()``
- ``normalized_value()``

Code Source
===========

Voir: ``backend/src/domain/entities/iot_reading.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

