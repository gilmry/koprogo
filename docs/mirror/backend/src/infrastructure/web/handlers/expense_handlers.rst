===============================================
infrastructure/web/handlers/expense_handlers.rs
===============================================

:Fichier: ``backend/src/infrastructure/web/handlers/expense_handlers.rs``
:Type: RUST
:Lignes de Code: 560
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **expense**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Fonctions
---------

- ``create_expense()``
- ``get_expense()``
- ``list_expenses()``
- ``list_expenses_by_building()``
- ``mark_expense_paid()``
- ``mark_expense_overdue()``
- ``cancel_expense()``
- ``reactivate_expense()``
- ``unpay_expense()``
- ``create_invoice_draft()``
- ``update_invoice_draft()``
- ``submit_invoice_for_approval()``
- ``approve_invoice()``
- ``reject_invoice()``
- ``get_pending_invoices()``

*... et 1 autres fonctions*

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/expense_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

