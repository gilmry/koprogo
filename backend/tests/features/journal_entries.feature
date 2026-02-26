# Feature: Journal Entries - Double-Entry Bookkeeping (Issue #200)
# Journal types: ACH (Achats), VEN (Ventes), FIN (Financier), ODS (Opérations Diverses)
# Inspired by Noalyss Belgian accounting software

Feature: Journal Entries (Double-Entry Bookkeeping)
  As an accountant
  I want to create manual journal entries
  So that all financial operations follow double-entry accounting rules

  Background:
    Given the system is initialized
    And an organization "Compta Copro ASBL" exists with id "org-compta"
    And a building "Residence Comptable" exists in organization "org-compta"

  Scenario: Create a balanced journal entry
    When I create a journal entry:
      | journal_type  | ODS                           |
      | description   | Provision pour travaux        |
      | document_ref  | ODS-2026-001                  |
    And I add the following lines:
      | account_code | debit  | credit | description           |
      | 604000       | 500.00 | 0.00   | Charges entretien     |
      | 440000       | 0.00   | 500.00 | Fournisseurs          |
    Then the journal entry should be created
    And the total debits should equal total credits

  Scenario: Reject unbalanced journal entry
    When I create a journal entry:
      | journal_type  | ACH                           |
      | description   | Facture fournisseur           |
    And I add the following lines:
      | account_code | debit  | credit | description           |
      | 604000       | 500.00 | 0.00   | Charges               |
      | 440000       | 0.00   | 400.00 | Fournisseurs          |
    Then the journal entry creation should fail
    And the error should contain "balanced" or "debit" or "credit"

  Scenario: Create purchase journal entry (ACH)
    When I create a journal entry:
      | journal_type  | ACH                           |
      | description   | Facture électricité           |
      | document_ref  | ACH-2026-001                  |
    And I add the following lines:
      | account_code | debit   | credit  | description           |
      | 612100       | 1000.00 | 0.00    | Electricité           |
      | 411000       | 210.00  | 0.00    | TVA à récupérer 21%  |
      | 440000       | 0.00    | 1210.00 | Fournisseurs          |
    Then the journal entry should be created

  Scenario: List journal entries with date range filter
    Given 5 journal entries exist in the current month
    When I list journal entries for the current month
    Then I should get 5 journal entries

  Scenario: List journal entries by type
    Given journal entries of types ACH and ODS exist
    When I list journal entries with type "ACH"
    Then all returned entries should have type "ACH"

  Scenario: Get journal entry with lines
    Given a journal entry with 3 lines exists
    When I get the journal entry by ID
    Then the entry should include 3 lines
    And each line should have account_code, debit, credit

  Scenario: Filter journal entries by building
    Given journal entries for 2 different buildings exist
    When I list journal entries for building "Residence Comptable"
    Then I should only get entries for that building

  Scenario: Delete a journal entry
    Given a journal entry exists
    When I delete the journal entry
    Then the journal entry should be deleted
