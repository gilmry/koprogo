# Feature: Stats syndic urgent tasks robustness vs NUMERIC columns
# Issue #521 — fix panic on f64 decoding of NUMERIC amount column
# Story A — STORY-521-A
#
# Scope: GET /api/v1/stats/syndic/urgent-tasks
# Bug: when an `expenses` row has payment_status='overdue', the repository
# tries to decode the NUMERIC `amount` column into f64 via Row::get(), which
# panics — killing the Actix worker and surfacing as 502 Bad Gateway to the
# frontend. Root fix: decode as rust_decimal::Decimal via try_get.

Feature: Stats syndic urgent tasks robustness vs NUMERIC columns
  As Marc (syndic)
  I want the urgent tasks endpoint to return overdue expenses with exact amounts
  So that the panic on NUMERIC->f64 decoding no longer crashes my dashboard

  Background:
    Given the system is initialized
    And an organization "Test Org" exists with slug "test-org"
    And a syndic user "Marc" exists in "Test Org"
    And a building "Résidence Soleil" exists in "Test Org"

  @negative @bug521 @story521-A
  Scenario: Urgent-tasks endpoint does not panic with an overdue expense (regression issue 521)
    Given an expense "Facture chauffage 2025-Q4" of "1234.5678" EUR exists for "Résidence Soleil"
    And the expense payment status is "overdue"
    When Marc requests GET /api/v1/stats/syndic/urgent-tasks
    Then the HTTP response status is 200
    And the response body contains a task of type "expense"
    And the task title displays the amount as "1234.57"
    And no Actix worker panic is logged during the request

  @happy @story521-A
  Scenario: Standard 2-decimal monetary amount
    Given an expense "Eau" of "123.45" EUR exists for "Résidence Soleil"
    And the expense payment status is "overdue"
    When Marc requests GET /api/v1/stats/syndic/urgent-tasks
    Then the task title is "Charge en retard - 123.45€"

  @edge @story521-A
  Scenario Outline: Monetary amount edge cases preserved across NUMERIC roundtrip
    Given an expense "<desc>" of "<input>" EUR exists for "Résidence Soleil"
    And the expense payment status is "overdue"
    When Marc requests GET /api/v1/stats/syndic/urgent-tasks
    Then the task title displays the amount as "<displayed>"

    Examples:
      | desc       | input             | displayed       |
      | Zero       | 0.0000            | 0.00            |
      | Four-dec   | 12.3456           | 12.35           |
      | Max-usuel  | 999999999.9999    | 1000000000.00   |
      | Tiny       | 0.0001            | 0.00            |

  @security @story521-A
  Scenario: Owner from another organization does not see Marc's overdue expenses
    Given an organization "Other Org" exists with slug "other-org"
    And an owner user "Bob" exists in "Other Org"
    And an expense "Facture privée Marc" of "500.00" EUR exists for "Résidence Soleil"
    And the expense payment status is "overdue"
    When Bob requests GET /api/v1/stats/syndic/urgent-tasks
    Then the HTTP response status is 403
    Or the response body contains no task referencing "Facture privée Marc"
