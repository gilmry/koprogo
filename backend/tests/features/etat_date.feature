# Feature: État Daté Generation for Property Sales
# Belgian Legal Requirement: Article 577-2 Civil Code - Required for ALL property sales
# Issue #80

Feature: État Daté Generation
  As a Syndic
  I want to generate États Datés for property sales
  So that notaries can complete real estate transactions legally

  Background:
    Given the system is initialized
    And an organization "État Daté ASBL" exists with id "org-789"
    And a building "Residence Vente" exists in organization "org-789"
    And a unit "Apartment 101" exists in building "Residence Vente"
    And a user "Syndic Vente" exists with email "syndic@vente.be" in organization "org-789"
    And the user is authenticated as Syndic

  Scenario: Notary requests État Daté for property sale
    When I request an État Daté for unit "Apartment 101" with:
      | reference_date    | 2026-02-15                |
      | requestor_name    | Notaire Jean Dupont       |
      | requestor_email   | jdupont@notaire.be        |
      | requestor_phone   | +32 2 123 45 67           |
    Then the État Daté should be created successfully
    And the status should be "Requested"
    And a reference number should be generated like "ED-2026-001"
    And the due date should be "2026-03-02" # 15 days later

  Scenario: Syndic marks État Daté as in progress
    Given an État Daté in status "Requested" exists
    When I mark the État Daté as in progress
    Then the status should be "InProgress"
    And the in_progress_at timestamp should be set

  Scenario: Syndic fills 16 mandatory legal sections
    Given an État Daté in status "InProgress" exists
    When I update financial data with:
      | quota_ordinary              | 0.0250   |
      | quota_extraordinary         | 0.0250   |
      | provisions_paid_amount_cents | 150000   |
      | outstanding_amount_cents    | 0        |
      | pending_works_amount_cents  | 500000   |
      | pending_litigation          | false    |
      | insurance_policy_number     | BE-ASSUR-12345 |
      | reserve_fund_amount_cents   | 5000000  |
      | building_debt_amount_cents  | 0        |
      | building_credit_amount_cents | 1000000  |
    And I update additional data with:
      | regulation_copy_url         | /documents/reglement.pdf |
      | recent_ag_minutes_urls      | /documents/pv_ag_2025_01.pdf,/documents/pv_ag_2024_12.pdf |
      | budget_url                  | /documents/budget_2026.pdf |
      | insurance_certificate_url   | /documents/assurance.pdf |
      | guarantees_and_mortgages    | None |
      | observations                | Elevator renovation approved, starts 2026-03-01 |
    Then all 16 legal sections should be filled
    And the État Daté should be ready for PDF generation

  Scenario: Generate État Daté PDF document
    Given an État Daté with all sections filled exists
    When I generate the PDF document
    Then the status should be "Generated"
    And a PDF file should be created at "/documents/etat_date_ED-2026-001.pdf"
    And the PDF should contain all 16 legal sections
    And the generated_at timestamp should be set

  Scenario: Deliver État Daté to notary
    Given an État Daté in status "Generated" exists with reference "ED-2026-001"
    When I mark the État Daté as delivered
    Then the status should be "Delivered"
    And the delivered_at timestamp should be set
    And the notary should receive an email with PDF attachment

  Scenario: Detect overdue État Daté (> 15 days)
    Given an État Daté requested on "2026-01-01" in status "InProgress"
    And the current date is "2026-01-20"
    When I request overdue États Datés
    Then I should see the État Daté in the list
    And the État Daté should be marked as "5 days overdue"
    And the syndic should receive an alert

  Scenario: État Daté expires after 3 months if not used
    Given an État Daté delivered on "2025-11-01"
    And the current date is "2026-02-10" # 3 months + 10 days later
    When I request expired États Datés
    Then I should see the État Daté in the list
    And the status should be "Expired"
    And the notary should request a new État Daté for the sale

  Scenario: Search État Daté by reference number
    Given an État Daté exists with reference "ED-2026-015"
    When I search for État Daté "ED-2026-015"
    Then I should find the État Daté
    And all details should be displayed

  Scenario: List États Datés by unit
    Given 3 États Datés exist for unit "Apartment 101" with statuses:
      | ED-2024-012 | Delivered |
      | ED-2025-089 | Expired   |
      | ED-2026-015 | InProgress |
    When I request États Datés for unit "Apartment 101"
    Then I should receive 3 États Datés
    And they should be ordered by requested date descending

  Scenario: Calculate État Daté statistics for building
    Given 10 États Datés exist for the building with statuses:
      | Requested  | 2 |
      | InProgress | 3 |
      | Generated  | 1 |
      | Delivered  | 3 |
      | Expired    | 1 |
    When I request État Daté statistics
    Then I should see:
      | total_count           | 10   |
      | delivered_count       | 3    |
      | overdue_count         | 2    |
      | average_delivery_days | 12   |

  Scenario: Prevent État Daté generation without all 16 sections
    Given an État Daté in status "InProgress" exists
    And only 10 of 16 legal sections are filled
    When I try to generate the PDF
    Then the generation should fail
    And I should see error "All 16 legal sections must be completed"

  Scenario: Track État Daté workflow for audit trail
    Given an État Daté exists with reference "ED-2026-001"
    When I request the audit trail
    Then I should see all state transitions:
      | Requested   | 2026-01-15 10:00 | Notaire Jean Dupont |
      | InProgress  | 2026-01-16 14:30 | Syndic Vente        |
      | Generated   | 2026-01-25 09:15 | Syndic Vente        |
      | Delivered   | 2026-01-25 10:00 | Syndic Vente        |
