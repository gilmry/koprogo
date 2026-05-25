Feature: Role-based filtering for list_buildings & list_acps (Story 1.3 — Refonte UX multi-rôle ACP)
  As an authenticated user (admin, syndic, owner)
  I want the building / ACP list to be scoped to my role
  So that I never see (nor can forge access to) data outside my legitimate perimeter

  # Story 1.3 ADR refs : ADR-0010 (ACP racine), ADR-0012 (scope_guard middleware).
  # AppError codes utilisés : AcpNotInScope (403), Unauthorized (401), Validation (400).
  # NOTE: Story 1.2 ajoute `buildings.acp_id` ; tant que la migration n'est pas
  # mergée, ces scénarios exercent uniquement la couche use-case + middleware
  # côté ACPs et l'absence d'accès cross-cabinet côté buildings.

  Background:
    Given a role-based listing system

  # ==========================================================================
  # @happy — chemin nominal pour chaque rôle
  # ==========================================================================

  @happy
  Scenario: Admin lists ACPs across all cabinets
    Given a cabinet "Cabinet A" with 2 ACPs
    And a cabinet "Cabinet B" with 1 ACP
    When admin lists ACPs role-based
    Then the listing should contain 3 ACPs

  @happy
  Scenario: Syndic lists ACPs of his own cabinet only
    Given a cabinet "Cabinet A" with 2 ACPs
    And a cabinet "Cabinet B" with 1 ACP
    When syndic of "Cabinet A" lists ACPs role-based
    Then the listing should contain 2 ACPs
    And the listing should not contain any ACP of "Cabinet B"

  @happy
  Scenario: Owner with no scope sees an empty ACP listing (until Story 3.5)
    Given a cabinet "Cabinet A" with 1 ACP
    When an unmapped owner lists ACPs role-based
    Then the listing should contain 0 ACPs

  # ==========================================================================
  # @edge — bornes, multi-rôles
  # ==========================================================================

  @edge
  Scenario: Multi-role user (admin + syndic) sees as admin (wider scope wins)
    Given a cabinet "Cabinet A" with 1 ACP
    And a cabinet "Cabinet B" with 1 ACP
    When a multi-role user admin-and-syndic-of "Cabinet A" lists ACPs role-based
    Then the listing should contain 2 ACPs

  # ==========================================================================
  # @security — RBAC scope forging
  # ==========================================================================

  @security
  Scenario: Syndic forges scope to access another cabinet's ACP
    Given a cabinet "Cabinet A" with 1 ACP named "Acp Cabinet A"
    And a cabinet "Cabinet B" with 1 ACP
    When syndic of "Cabinet B" tries to access ACP "Acp Cabinet A" by id
    Then the operation should be denied with AcpNotInScope

  @security
  Scenario: Scope guard rejects a forged acp_id query for syndic out of scope
    Given a cabinet "Cabinet A" with 1 ACP named "Acp Cabinet A"
    And a cabinet "Cabinet B" with 1 ACP
    When syndic of "Cabinet B" forges scope_guard with acp of "Acp Cabinet A"
    Then the scope guard should refuse with AcpNotInScope

  # ==========================================================================
  # @negative — défaillances typées
  # ==========================================================================

  @negative
  Scenario: Unauthenticated request to scope guard
    When an unauthenticated request hits the scope guard
    Then the scope guard should refuse with Unauthorized

  @negative
  Scenario: Non-admin caller without scope returns a validation error
    Given a cabinet "Cabinet A" with 1 ACP
    When a syndic with no organization id calls scope guard without scope
    Then the scope guard should refuse with Validation
