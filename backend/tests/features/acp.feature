Feature: ACP Management (Story 1.1 — Refonte UX multi-rôle ACP)
  As an admin / syndic / owner
  I want to manage ACPs (Associations des Copropriétaires) per Art. 3.84 CC belge
  So that the domain model carries the legal copropriété entity correctly

  Background:
    Given an ACP management system

  # ==========================================================================
  # @happy — chemin nominal
  # ==========================================================================

  @happy
  Scenario: Admin creates an ACP attached to a syndic cabinet
    Given an existing organization "Cabinet Maury"
    When admin creates an ACP named "Residence Les Tilleuls" attached to that organization
    Then the ACP should be persisted successfully
    And the ACP slug should be "residence-les-tilleuls"
    And an audit event "AcpCreated" should be logged

  @happy
  Scenario: Admin creates an auto-managed ACP without organization
    When admin creates an ACP named "Copro Autogeree" with no organization
    Then the ACP should be persisted successfully
    And the ACP organization_id should be null

  @happy
  Scenario: Admin lists ACPs sees all of them
    Given an existing organization "Cabinet A"
    And an existing organization "Cabinet B"
    And an ACP "Acp One" attached to "Cabinet A"
    And an ACP "Acp Two" attached to "Cabinet B"
    When admin lists ACPs
    Then the list should contain 2 ACPs

  @happy
  Scenario: Admin retrieves an ACP by id
    Given an existing organization "Cabinet Maury"
    And an ACP "Residence Maury" attached to "Cabinet Maury"
    When admin gets that ACP by id
    Then the ACP returned should have name "Residence Maury"

  @happy
  Scenario: Admin updates an ACP name
    Given an existing organization "Cabinet Maury"
    And an ACP "Old Name" attached to "Cabinet Maury"
    When admin updates that ACP name to "New Name"
    Then the ACP should be persisted successfully
    And the ACP returned should have name "New Name"

  @happy
  Scenario: Admin archives an ACP
    Given an existing organization "Cabinet Maury"
    And an ACP "To Archive" attached to "Cabinet Maury"
    When admin archives that ACP
    Then the ACP archive operation should succeed

  # ==========================================================================
  # @edge — bornes
  # ==========================================================================

  @edge
  Scenario: ACP with zero buildings is allowed at creation time
    Given an existing organization "Cabinet Maury"
    When admin creates an ACP named "Future Building" attached to that organization
    Then the ACP should be persisted successfully
    And the ACP should have 0 buildings linked

  @edge
  Scenario: ACP name with minimum length 2 is accepted
    When admin creates an ACP named "Ab" with no organization
    Then the ACP should be persisted successfully

  @edge
  Scenario: ACP name with surrounding whitespace is trimmed
    When admin creates an ACP named "   Trimmed Acp   " with no organization
    Then the ACP should be persisted successfully
    And the ACP slug should be "trimmed-acp"

  # ==========================================================================
  # @security — RBAC, scope, escalade
  # ==========================================================================

  @security
  Scenario: Syndic of cabinet B cannot read ACPs of cabinet A
    Given an existing organization "Cabinet A"
    And an existing organization "Cabinet B"
    And an ACP "Acp A1" attached to "Cabinet A"
    When syndic of "Cabinet B" lists ACPs
    Then the list should not contain ACP "Acp A1"

  @security
  Scenario: Non-admin user cannot create an ACP via use-case permission check
    Given an existing organization "Cabinet Maury"
    When syndic tries to create an ACP named "Forbidden Acp" attached to that organization
    Then the operation should fail with a forbidden error

  @security
  Scenario: Owner gets only ACPs where they have a role assignment
    Given an existing organization "Cabinet Maury"
    And an ACP "Owner Acp" attached to "Cabinet Maury"
    And an ACP "Other Acp" attached to "Cabinet Maury"
    And an owner user assigned to "Owner Acp"
    When that owner lists ACPs
    Then the list should contain ACP "Owner Acp"
    And the list should not contain ACP "Other Acp"

  # ==========================================================================
  # @negative — défaillance correcte
  # ==========================================================================

  @negative
  Scenario: Creating an ACP with an inexistent organization_id returns a validation error
    When admin creates an ACP named "Phantom" attached to an unknown organization
    Then the operation should fail with a validation error

  @negative
  Scenario: Getting an ACP by an unknown id returns not found
    When admin gets an ACP by an unknown id
    Then the operation should fail with a not found error

  @negative
  Scenario: Updating an inexistent ACP returns a not found error
    When admin updates an inexistent ACP
    Then the operation should fail with a not found error

  @negative
  Scenario: Creating an ACP with an empty name is rejected
    When admin creates an ACP named "" with no organization
    Then the operation should fail with a validation error

  @negative
  Scenario: Creating an ACP with a 1-char name is rejected
    When admin creates an ACP named "A" with no organization
    Then the operation should fail with a validation error
