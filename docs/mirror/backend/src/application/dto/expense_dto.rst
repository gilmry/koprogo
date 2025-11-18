==============================
application/dto/expense_dto.rs
==============================

:Fichier: ``backend/src/application/dto/expense_dto.rs``
:Type: RUST
:Lignes de Code: 201
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Data Transfer Object (DTO) pour **expense**. Définit les contrats d'API REST (requêtes/réponses) avec validation et sérialisation JSON.

API Publique
============

Structures
----------

- ``CreateExpenseDto``
- ``ExpenseResponseDto``
- ``CreateInvoiceDraftDto``
- ``UpdateInvoiceDraftDto``
- ``SubmitForApprovalDto``
- ``ApproveInvoiceDto``
- ``RejectInvoiceDto``
- ``CreateInvoiceLineItemDto``
- ``InvoiceResponseDto``
- ``InvoiceLineItemResponseDto``
- ``ChargeDistributionResponseDto``
- ``PendingInvoicesListDto``

Code Source
===========

Voir: ``backend/src/application/dto/expense_dto.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

