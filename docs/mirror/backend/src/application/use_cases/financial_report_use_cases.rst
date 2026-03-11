===================================================
application/use_cases/financial_report_use_cases.rs
===================================================

:Fichier: ``backend/src/application/use_cases/financial_report_use_cases.rs``
:Type: RUST
:Lignes de Code: 580
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **financial report**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``FinancialReportUseCases``
- ``BalanceSheetReport``
- ``IncomeStatementReport``
- ``AccountSection``
- ``AccountLine``

Fonctions
---------

- ``new()``
- ``generate_balance_sheet()``
- ``generate_income_statement()``
- ``generate_balance_sheet_for_building()``
- ``generate_income_statement_for_building()``

Code Source
===========

Voir: ``backend/src/application/use_cases/financial_report_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

