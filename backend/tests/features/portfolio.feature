Feature: Portfolio Management (Story 2.1 — Slice 2 Refonte UX multi-rôle ACP)
  As a syndic / accountant / authenticated user
  I want to manage portfolios of buildings (favorites + team sharing)
  So that I find my recurring set of buildings quickly across devices and teammates

  Background:
    Given a portfolio management system

  # ==========================================================================
  # @happy — chemin nominal
  # ==========================================================================

  @happy
  Scenario: Syndic creates a portfolio
    Given an existing user "syndic@cabinet.be"
    When that user creates a portfolio named "Mes immeubles favoris"
    Then the portfolio should be persisted successfully
    And the portfolio name should be "Mes immeubles favoris"

  @happy
  Scenario: Syndic adds 3 buildings (1 favorite + 2 normal) and listing returns 3, favorite first
    Given an existing user "syndic@cabinet.be"
    And a portfolio "Mes favoris" owned by "syndic@cabinet.be"
    And an existing building "Residence A"
    And an existing building "Residence B"
    And an existing building "Residence C"
    When that user adds "Residence A" as favorite to portfolio
    And that user adds "Residence B" as normal to portfolio
    And that user adds "Residence C" as normal to portfolio
    And that user lists the buildings of portfolio
    Then the listing should contain 3 buildings
    And the first building of the listing should be favorite

  # ==========================================================================
  # @edge — bornes
  # ==========================================================================

  @edge
  Scenario: Empty portfolio listing returns empty array
    Given an existing user "syndic@cabinet.be"
    And a portfolio "Vide" owned by "syndic@cabinet.be"
    When that user lists the buildings of portfolio
    Then the listing should contain 0 buildings

  @edge
  Scenario: Listing portfolios when user has none returns empty list
    Given an existing user "syndic@cabinet.be"
    When that user lists portfolios
    Then the portfolio list should contain 0 portfolios

  # ==========================================================================
  # @security — RBAC, scope, escalade
  # ==========================================================================

  @security
  Scenario: User of cabinet B cannot read portfolio of user of cabinet A
    Given an existing user "syndic-a@cabinet-a.be"
    And an existing user "syndic-b@cabinet-b.be"
    And a portfolio "Cabinet A Portfolio" owned by "syndic-a@cabinet-a.be"
    When user "syndic-b@cabinet-b.be" tries to get that portfolio
    Then the operation should fail with a forbidden error

  @security
  Scenario: User not shared cannot read portfolio shared with another user
    Given an existing user "owner@cabinet.be"
    And an existing user "teammate@cabinet.be"
    And an existing user "stranger@cabinet.be"
    And a portfolio "Shared" owned by "owner@cabinet.be"
    And the portfolio is shared with "teammate@cabinet.be"
    When user "stranger@cabinet.be" tries to get that portfolio
    Then the operation should fail with a forbidden error

  @security
  Scenario: Shared read-only user cannot add a building
    Given an existing user "owner@cabinet.be"
    And an existing user "teammate@cabinet.be"
    And an existing building "Residence X"
    And a portfolio "Shared RO" owned by "owner@cabinet.be"
    And the portfolio is shared with "teammate@cabinet.be"
    When user "teammate@cabinet.be" tries to add "Residence X" to portfolio
    Then the operation should fail with a forbidden error

  # ==========================================================================
  # @negative — défaillance correcte
  # ==========================================================================

  @negative
  Scenario: Creating a portfolio with empty name is rejected
    Given an existing user "syndic@cabinet.be"
    When that user creates a portfolio named ""
    Then the operation should fail with a validation error

  @negative
  Scenario: Creating a portfolio with a 1-char name is rejected
    Given an existing user "syndic@cabinet.be"
    When that user creates a portfolio named "A"
    Then the operation should fail with a validation error

  @negative
  Scenario: Adding an inexistent building returns a not found error
    Given an existing user "syndic@cabinet.be"
    And a portfolio "Mes favoris" owned by "syndic@cabinet.be"
    When that user adds an unknown building to portfolio
    Then the operation should fail with a not found error

  @negative
  Scenario: Getting an inexistent portfolio returns a not found error
    Given an existing user "syndic@cabinet.be"
    When that user gets a portfolio by an unknown id
    Then the operation should fail with a not found error
