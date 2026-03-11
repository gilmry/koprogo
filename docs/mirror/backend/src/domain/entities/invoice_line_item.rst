====================================
domain/entities/invoice_line_item.rs
====================================

:Fichier: ``backend/src/domain/entities/invoice_line_item.rs``
:Type: RUST
:Lignes de Code: 256
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **invoice line item**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``InvoiceLineItem``

Fonctions
---------

- ``new()``
- ``recalculate()``
- ``total_excl_vat()``
- ``total_vat()``
- ``total_incl_vat()``

Code Source
===========

Voir: ``backend/src/domain/entities/invoice_line_item.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

