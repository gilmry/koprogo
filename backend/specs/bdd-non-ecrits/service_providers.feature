# Feature: Service Provider (Contractor) Marketplace (Issue #276)
# Trade categories: Syndic, Plombier, Electricien, Chauffagiste, etc.
# Belgian BCE/KBO company number, IPI registration, VCA/BOSEC certifications

Feature: Service Provider Management
  As a syndic or administrator
  I want to manage service providers in the marketplace
  So that buildings can find qualified contractors for maintenance work

  Background:
    Given the system is initialized
    And an organization "Provider Copro ASBL" exists with id "org-provider"
    And the user is authenticated as syndic "Sophie Syndic"

  # === CREATION ===

  Scenario: Create a plumber service provider with BCE number
    When I create a service provider:
      | company_name   | ABC Plomberie     |
      | trade_category | Plombier          |
      | bce_number     | BE0123456789      |
    Then the provider should be created successfully
    And the provider company name should be "ABC Plomberie"
    And the provider trade category should be "Plombier"
    And the provider BCE number should be "BE0123456789"
    And the provider should have a public profile slug "abc-plomberie"
    And the provider reviews count should be 0
    And the provider should not be verified

  Scenario: Create an electrician provider without BCE number
    When I create a service provider:
      | company_name   | ElectroPro Brussels |
      | trade_category | Electricien         |
    Then the provider should be created successfully
    And the provider trade category should be "Electricien"
    And the provider BCE number should be empty

  Scenario: Create provider with special characters generates clean slug
    When I create a service provider:
      | company_name   | Jean-Pierre & Fils (SPRL) |
      | trade_category | Couvreur                  |
    Then the provider should be created successfully
    And the provider should have a public profile slug "jean-pierre-fils-sprl"

  # === VALIDATION ===

  Scenario: Reject provider with empty company name
    When I create a service provider:
      | company_name   |            |
      | trade_category | Plombier   |
    Then the provider creation should fail
    And the error should contain "company_name cannot be empty"

  Scenario: Reject invalid trade category
    When I parse trade category "InvalidCategory"
    Then the parsing should fail
    And the error should contain "Invalid trade category"

  # === TRADE CATEGORY LISTING ===

  Scenario: List providers by trade category Plombier
    Given a service provider "ABC Plomberie" exists with trade "Plombier"
    And a service provider "XYZ Electricite" exists with trade "Electricien"
    And a service provider "DEF Plomberie" exists with trade "Plombier"
    When I list providers by trade category "Plombier"
    Then I should see 2 providers
    And all providers should have trade category "Plombier"

  Scenario: List providers by trade category returns empty for unused trade
    Given a service provider "ABC Plomberie" exists with trade "Plombier"
    When I list providers by trade category "Ascensoriste"
    Then I should see 0 providers

  # === RATING ===

  Scenario: Update provider rating with first review
    Given a service provider "ABC Plomberie" exists with trade "Plombier"
    When I update the provider rating with score 4.0
    Then the provider reviews count should be 1
    And the provider average rating should be 4.0

  Scenario: Update provider rating with multiple reviews calculates running average
    Given a service provider "ABC Plomberie" exists with trade "Plombier"
    When I update the provider rating with score 4.0
    And I update the provider rating with score 5.0
    Then the provider reviews count should be 2
    And the provider average rating should be 4.5

  Scenario: Reject rating above 5.0
    Given a service provider "ABC Plomberie" exists with trade "Plombier"
    When I update the provider rating with score 6.0
    Then the rating update should fail
    And the error should contain "Rating must be between 0.0 and 5.0"

  Scenario: Reject negative rating
    Given a service provider "ABC Plomberie" exists with trade "Plombier"
    When I update the provider rating with score -1.0
    Then the rating update should fail
    And the error should contain "Rating must be between 0.0 and 5.0"

  # === ALL TRADE CATEGORIES ===

  Scenario: All Belgian trade categories are supported
    Then the following trade categories should be valid:
      | Syndic                |
      | BureauEtude           |
      | Architecte            |
      | AssistantMaitreOeuvre  |
      | IngenieurStabilite    |
      | Plombier              |
      | Electricien           |
      | Chauffagiste          |
      | Menuisier             |
      | Peintre               |
      | Maconnerie            |
      | Etancheite            |
      | Ascensoriste          |
      | Jardinier             |
      | Nettoyage             |
      | Securite              |
      | Deboucheur            |
      | Couvreur              |
      | Carreleur             |
      | TechniquesSpeciales   |
