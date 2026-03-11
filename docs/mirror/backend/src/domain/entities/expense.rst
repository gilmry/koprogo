==========================
domain/entities/expense.rs
==========================

:Fichier: ``backend/src/domain/entities/expense.rs``
:Type: RUST
:Lignes de Code: 1094
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant une **charge de copropriété**. Implémente le workflow d'approbation (Draft → PendingApproval → Approved/Rejected) et la gestion TVA belge.

API Publique
============

Structures
----------

- ``Expense``

Énumérations
------------

- ``ExpenseCategory``
- ``PaymentStatus``
- ``ApprovalStatus``

Fonctions
---------

- ``new()``
- ``new_with_vat()``
- ``recalculate_vat()``
- ``submit_for_approval()``
- ``approve()``
- ``reject()``
- ``can_be_modified()``
- ``is_approved()``
- ``mark_as_paid()``
- ``mark_as_overdue()``
- ``cancel()``
- ``reactivate()``
- ``unpay()``
- ``is_paid()``

Code Source
===========

Voir: ``backend/src/domain/entities/expense.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

