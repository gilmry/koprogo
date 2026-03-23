# Feature: Individual Members - Energy Campaign Participation (Issue #280)
# Non-authenticated individuals joining energy buying group campaigns
# Endpoints: POST /energy-campaigns/:id/join-as-individual,
#            POST /energy-campaigns/:id/members/:id/consent,
#            PUT /energy-campaigns/:id/members/:id/consumption,
#            DELETE /energy-campaigns/:id/members/:id/withdraw

Feature: Individual Members - Energy Campaign Participation
  As an individual (non-co-owner)
  I want to join energy buying group campaigns
  So that I can benefit from group purchasing power for lower energy rates

  Background:
    Given the system is initialized
    And an organization "Energy Copro ASBL" exists with id "org-energy"
    And an energy campaign "Achat Groupe 2026" exists in organization "org-energy"
    And the campaign is open to individual participants

  # === JOIN CAMPAIGN ===

  Scenario: Individual joins energy campaign successfully
    When I join the campaign as an individual:
      | email       | marie.dupont@example.com |
      | postal_code | 1000                     |
    Then the response status should be 201
    And the member should be created successfully
    And the member email should be "marie.dupont@example.com"
    And the member postal_code should be "1000"

  Scenario: Individual joins with valid Belgian postal code
    When I join the campaign as an individual:
      | email       | pierre@example.com |
      | postal_code | 4000               |
    Then the response status should be 201
    And the member should be created successfully

  Scenario: Join fails with invalid email format
    When I join the campaign as an individual:
      | email       | not-an-email   |
      | postal_code | 1000           |
    Then the response status should be 400
    And the error should indicate invalid email

  Scenario: Join fails with empty email
    When I join the campaign as an individual:
      | email       |        |
      | postal_code | 1000   |
    Then the response status should be 400
    And the error should indicate email is required

  Scenario: Join fails with empty postal code
    When I join the campaign as an individual:
      | email       | valid@example.com |
      | postal_code |                   |
    Then the response status should be 400
    And the error should indicate postal code is required

  Scenario: Join fails for non-existent campaign
    Given the campaign UUID is "00000000-0000-0000-0000-000000000000"
    When I join the campaign as an individual:
      | email       | user@example.com |
      | postal_code | 1000             |
    Then the join should fail
    And the error should indicate the campaign was not found

  Scenario: Joining does not require authentication
    Given I am not authenticated
    When I join the campaign as an individual:
      | email       | anonymous@example.com |
      | postal_code | 1050                  |
    Then the response status should be 201

  # === GRANT GDPR CONSENT ===

  Scenario: Grant GDPR consent for campaign participation
    Given an individual member "marie.dupont@example.com" exists in the campaign
    When I grant GDPR consent for the member:
      | consent_given | true |
    Then the response status should be 200
    And the response message should be "Consent granted successfully"

  Scenario: Grant consent for non-existent member
    Given the member UUID is "00000000-0000-0000-0000-000000000000"
    When I grant GDPR consent for the member:
      | consent_given | true |
    Then the consent grant should eventually fail when repository is implemented
    And the error should indicate the member was not found

  # === UPDATE CONSUMPTION DATA ===

  Scenario: Update consumption data for a member
    Given an individual member "marie.dupont@example.com" exists in the campaign
    And the member has granted GDPR consent
    When I update consumption data for the member:
      | annual_kwh_electricity | 3500 |
      | annual_kwh_gas         | 1200 |
    Then the response status should be 200
    And the response message should be "Consumption data updated"

  Scenario: Update consumption with electricity only
    Given an individual member "marie.dupont@example.com" exists in the campaign
    When I update consumption data for the member:
      | annual_kwh_electricity | 4200 |
    Then the response status should be 200

  Scenario: Update consumption with zero values
    Given an individual member "marie.dupont@example.com" exists in the campaign
    When I update consumption data for the member:
      | annual_kwh_electricity | 0 |
      | annual_kwh_gas         | 0 |
    Then the response status should be 200

  # === WITHDRAW FROM CAMPAIGN (GDPR RIGHT TO ERASURE) ===

  Scenario: Individual withdraws from campaign
    Given an individual member "marie.dupont@example.com" exists in the campaign
    When I withdraw the member from the campaign
    Then the response status should be 200
    And the response should confirm successful withdrawal
    And the response message should contain "Successfully withdrawn from campaign"

  Scenario: Withdrawal triggers GDPR data deletion preparation
    Given an individual member "marie.dupont@example.com" exists in the campaign
    When I withdraw the member from the campaign
    Then the response status should be 200
    And the member data should be scheduled for anonymization per GDPR Article 17

  Scenario: Withdraw non-existent member
    Given the member UUID is "00000000-0000-0000-0000-000000000000"
    When I withdraw the member from the campaign
    Then the withdrawal should eventually fail when repository is implemented
    And the error should indicate the member was not found

  # === DUPLICATE PREVENTION ===

  Scenario: Duplicate email in same campaign is rejected
    Given an individual member "marie.dupont@example.com" exists in the campaign
    When I join the campaign as an individual:
      | email       | marie.dupont@example.com |
      | postal_code | 1050                     |
    Then the join should eventually fail with duplicate email when UNIQUE constraint is enforced
