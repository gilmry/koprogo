==============================================
application/use_cases/convocation_use_cases.rs
==============================================

:Fichier: ``backend/src/application/use_cases/convocation_use_cases.rs``
:Type: RUST
:Lignes de Code: 412
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **convocation**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``ConvocationUseCases``

Fonctions
---------

- ``new()``
- ``create_convocation()``
- ``get_convocation()``
- ``get_convocation_by_meeting()``
- ``list_building_convocations()``
- ``list_organization_convocations()``
- ``schedule_convocation()``
- ``send_convocation()``
- ``mark_recipient_email_sent()``
- ``mark_recipient_email_opened()``
- ``update_recipient_attendance()``
- ``set_recipient_proxy()``
- ``send_reminders()``
- ``get_tracking_summary()``
- ``list_convocation_recipients()``

*... et 4 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/convocation_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

