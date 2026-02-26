# Feature: Energy Buying Groups (Issue #110)
# Achats Groupes d'Energie - GDPR compliant
# K-anonymity >= 5 participants for statistics

Feature: Energy Buying Groups
  As a syndic
  I want to organize group energy buying campaigns
  So that co-owners can benefit from collective negotiation power

  Background:
    Given the system is initialized
    And an organization "Energie Copro ASBL" exists with id "org-energy"
    And a building "Residence Verte" with 10 units exists
    And 10 owners exist in the building

  # === CAMPAIGN MANAGEMENT ===

  Scenario: Create an energy campaign
    When I create an energy campaign:
      | campaign_name        | Electricity 2026              |
      | energy_types         | electricity                   |
      | deadline_participation | 2026-04-30T23:59:59Z       |
    Then the campaign should be created
    And the status should be "Open"

  Scenario: List campaigns for organization
    Given 2 campaigns exist
    When I list campaigns
    Then I should get 2 campaigns

  Scenario: Update campaign status
    Given an open campaign exists
    When I update the status to "Closed"
    Then the campaign status should be "Closed"

  # === ENERGY BILL UPLOADS ===

  Scenario: Upload energy bill with GDPR consent
    Given an open campaign exists
    When an owner uploads their energy bill:
      | total_kwh    | 3500                          |
      | energy_type  | electricity                   |
      | bill_period  | 2025                          |
      | consent      | true                          |
    Then the upload should be accepted
    And the data should be stored encrypted

  Scenario: Get my energy bill uploads
    Given I have uploaded 2 energy bills
    When I list my uploads
    Then I should get 2 uploads

  Scenario: Verify an upload (admin only)
    Given an unverified upload exists
    When an admin verifies the upload
    Then the upload should be marked as verified

  # === GDPR COMPLIANCE ===

  Scenario: GDPR - Withdraw consent deletes data immediately
    Given an owner has uploaded energy data
    When the owner withdraws their GDPR consent
    Then the energy data should be deleted immediately
    And no trace of the data should remain

  Scenario: GDPR - Delete energy bill (right to erasure)
    Given an owner has an energy bill upload
    When the owner deletes their upload
    Then the upload should be deleted

  # === PROVIDER OFFERS ===

  Scenario: Add provider offer to campaign
    Given an open campaign exists
    When I add a provider offer:
      | provider_name               | Engie Belgium          |
      | price_kwh_electricity       | 0.28                   |
      | fixed_monthly_fee           | 5.50                   |
      | green_energy_pct            | 100                    |
      | contract_duration_months    | 12                     |
      | estimated_savings_pct       | 15                     |
    Then the offer should be added

  Scenario: List offers for a campaign
    Given 3 provider offers exist for the campaign
    When I list offers for the campaign
    Then I should get 3 offers

  Scenario: Select winning offer
    Given multiple offers exist
    When I select the winning offer
    Then the selected offer should be recorded

  # === STATISTICS (K-ANONYMITY) ===

  Scenario: Campaign statistics with k-anonymity >= 5
    Given 6 participants have uploaded energy data
    When I get campaign statistics
    Then the statistics should be anonymized
    And the statistics should aggregate consumption data

  Scenario: Statistics blocked with fewer than 5 participants
    Given only 3 participants have uploaded energy data
    When I get campaign statistics
    Then the statistics should be restricted for privacy

  # === DELETION ===

  Scenario: Delete a campaign
    Given a campaign exists
    When I delete the campaign
    Then the campaign should be deleted
