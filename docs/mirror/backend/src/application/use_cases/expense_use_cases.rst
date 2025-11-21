==========================================
application/use_cases/expense_use_cases.rs
==========================================

:Fichier: ``backend/src/application/use_cases/expense_use_cases.rs``
:Type: RUST
:Lignes de Code: 467
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **expense**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``ExpenseUseCases``

Fonctions
---------

- ``new()``
- ``with_accounting_service()``
- ``create_expense()``
- ``get_expense()``
- ``list_expenses_by_building()``
- ``list_expenses_paginated()``
- ``mark_as_paid()``
- ``mark_as_overdue()``
- ``cancel_expense()``
- ``reactivate_expense()``
- ``unpay_expense()``
- ``create_invoice_draft()``
- ``update_invoice_draft()``
- ``submit_for_approval()``
- ``approve_invoice()``

*... et 3 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/expense_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

