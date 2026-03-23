# Feature: Marketplace - Service Provider Directory
# Belgian contractor marketplace with BCE registration
# Endpoints: GET /marketplace/providers, GET /marketplace/providers/:slug,
#            POST /service-providers, GET /buildings/:id/reports/contract-evaluations/annual

Feature: Marketplace - Service Provider Directory
  As a syndic or co-owner
  I want to search and manage service providers in the marketplace
  So that I can find qualified contractors for building maintenance

  Background:
    Given the system is initialized
    And an organization "Marketplace Copro ASBL" exists with id "org-marketplace"
    And a building "Residence Marketplace" exists in organization "org-marketplace"
    And a syndic user "Jean Syndic" exists for building "Residence Marketplace"

  # === SEARCH PROVIDERS (PUBLIC) ===

  Scenario: Search all service providers without filters
    When I request GET "/api/v1/marketplace/providers"
    Then the response status should be 200
    And the response should be an array of service providers

  Scenario: Search service providers by trade category
    When I request GET "/api/v1/marketplace/providers?trade_category=Plumbing"
    Then the response status should be 200
    And each returned provider should have trade_category "Plumbing"

  Scenario: Search service providers by postal code
    When I request GET "/api/v1/marketplace/providers?postal_code=1000"
    Then the response status should be 200
    And results should be filtered by postal code "1000"

  Scenario: Search service providers with minimum rating filter
    When I request GET "/api/v1/marketplace/providers?min_rating=4.0"
    Then the response status should be 200
    And each returned provider should have a rating of at least 4.0

  Scenario: Search with verified-only filter
    When I request GET "/api/v1/marketplace/providers?is_verified_only=true"
    Then the response status should be 200
    And each returned provider should be verified

  Scenario: Search with multiple filters combined
    When I request GET "/api/v1/marketplace/providers?trade_category=Electrical&postal_code=1050&min_rating=3.5"
    Then the response status should be 200
    And results should match all applied filters

  Scenario: Search providers does not require authentication
    Given I am not authenticated
    When I request GET "/api/v1/marketplace/providers"
    Then the response status should be 200

  # === GET PROVIDER BY SLUG (PUBLIC) ===

  Scenario: Get provider profile by slug
    Given a service provider "Plomberie Dupont" exists with slug "plomberie-dupont"
    When I request GET "/api/v1/marketplace/providers/plomberie-dupont"
    Then the response status should be 200
    And the provider name should be "Plomberie Dupont"

  Scenario: Get provider by non-existent slug returns 404
    When I request GET "/api/v1/marketplace/providers/nonexistent-provider-slug"
    Then the response status should be 404
    And the error message should contain "Provider not found: nonexistent-provider-slug"

  Scenario: Get provider by slug does not require authentication
    Given I am not authenticated
    When I request GET "/api/v1/marketplace/providers/plomberie-dupont"
    Then the response status should be 404

  # === CREATE SERVICE PROVIDER (AUTHENTICATED) ===

  Scenario: Syndic creates a new service provider
    Given the user is authenticated as syndic "Jean Syndic"
    When I create a service provider:
      | company_name   | Electricite Martin SPRL         |
      | trade_category | Electrical                      |
      | bce_number     | 0123.456.789                    |
    Then the response status should be 201
    And the provider should be created successfully
    And the provider company_name should be "Electricite Martin SPRL"
    And the provider trade_category should be "Electrical"

  Scenario: Create provider fails without authentication
    Given I am not authenticated
    When I create a service provider:
      | company_name   | Test Provider     |
      | trade_category | Plumbing          |
      | bce_number     | 0123.456.789      |
    Then the response status should be 401

  Scenario: Create provider fails without organization
    Given I am authenticated as a user without an organization
    When I create a service provider:
      | company_name   | Test Provider     |
      | trade_category | Plumbing          |
      | bce_number     | 0123.456.789      |
    Then the response status should be 400
    And the error message should contain "Organization ID required"

  Scenario: Create provider fails with invalid trade category
    Given the user is authenticated as syndic "Jean Syndic"
    When I create a service provider:
      | company_name   | Bad Trade Provider |
      | trade_category | InvalidCategory    |
      | bce_number     | 0123.456.789       |
    Then the response status should be 400
    And the error should indicate invalid trade category

  # === CONTRACT EVALUATIONS ANNUAL REPORT ===

  Scenario: Get annual contract evaluations report for a building
    Given the user is authenticated as syndic "Jean Syndic"
    And building "Residence Marketplace" has a known UUID
    When I request GET "/api/v1/buildings/<building_uuid>/reports/contract-evaluations/annual"
    Then the response status should be 200
    And the report should contain "building_id"
    And the report should contain "report_year" with the current year
    And the report should contain "total_evaluations"
    And the report should contain "average_global_score"
    And the report should contain "recommendation_rate"
    And the report should contain "evaluations" array

  Scenario: Get annual report for a specific year
    Given the user is authenticated as syndic "Jean Syndic"
    And building "Residence Marketplace" has a known UUID
    When I request GET "/api/v1/buildings/<building_uuid>/reports/contract-evaluations/annual?year=2025"
    Then the response status should be 200
    And the report_year should be 2025

  Scenario: Get annual report fails without authentication
    Given I am not authenticated
    When I request GET "/api/v1/buildings/<building_uuid>/reports/contract-evaluations/annual"
    Then the response status should be 401
