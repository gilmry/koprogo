Feature: Building Conformity (Story 1.4 — Refonte UX multi-rôle ACP)
  As an admin / syndic / owner
  I want building conformity (counts of units + sum of quotas) computed and exposed correctly
  So that no operational compute happens on a non-conformant building (#553 Bugs 1/3/4)

  Background:
    Given a building conformity system

  # ==========================================================================
  # @happy — chemin nominal
  # ==========================================================================

  @happy
  Scenario: A building with 2 units summing to 1000 millièmes is conformant
    Given an existing organization "Cabinet Maury" with a building "Conforme Tower" of declared 2 units
    And the building "Conforme Tower" has a unit "A1" with quota 500
    And the building "Conforme Tower" has a unit "A2" with quota 500
    When admin gets building "Conforme Tower" by id
    Then the building units_count should be 2
    And the building quota_sum should be "1000"
    And the building is_conformant should be true
    And the building quota_delta should be "0"

  @happy
  Scenario: Building entity exposes a domain-level is_conformant method
    Given a building entity declared with 3 units
    When the building metrics report 3 units and quota_sum 1000
    Then the entity is_conformant method should return true

  # ==========================================================================
  # @edge — bornes (Decimal strict, pas de tolérance arrondi)
  # ==========================================================================

  @edge
  Scenario: Building short by 1 millième is NOT conformant (no rounding tolerance)
    # Track H Story H1 — convention quota_delta = total_tantiemes - quota_sum
    # (positif = manque). Building basis 1000 sum 999 → delta +1.
    Given an existing organization "Cabinet Maury" with a building "Almost There" of declared 2 units
    And the building "Almost There" has a unit "A1" with quota 500
    And the building "Almost There" has a unit "A2" with quota 499
    When admin gets building "Almost There" by id
    Then the building units_count should be 2
    And the building quota_sum should be "999"
    And the building is_conformant should be false
    And the building quota_delta should be "1"

  @edge
  Scenario: Building with 0 units returns quota_sum 0 (no NaN, no panic)
    # Track H Story H1 — basis 1000, sum 0 → delta +1000 (manque total).
    Given an existing organization "Cabinet Maury" with a building "Empty Shell" of declared 1 units
    When admin gets building "Empty Shell" by id
    Then the building units_count should be 0
    And the building quota_sum should be "0"
    And the building is_conformant should be false
    And the building quota_delta should be "1000"

  @edge
  Scenario: Building with declared_units mismatch is not conformant even with quota_sum 1000
    Given an existing organization "Cabinet Maury" with a building "Mismatch Tower" of declared 3 units
    And the building "Mismatch Tower" has a unit "A1" with quota 500
    And the building "Mismatch Tower" has a unit "A2" with quota 500
    When admin gets building "Mismatch Tower" by id
    Then the building units_count should be 2
    And the building quota_sum should be "1000"
    And the building is_conformant should be false

  # ==========================================================================
  # @security — RBAC & response invariant
  # ==========================================================================

  @security
  Scenario: GET building response always exposes is_conformant flag (never elided)
    Given an existing organization "Cabinet Maury" with a building "Audit Tower" of declared 1 units
    When admin gets building "Audit Tower" by id
    Then the building response should expose the is_conformant boolean field

  @security
  Scenario: Conformity computation is server-authoritative (client cannot forge it)
    Given an existing organization "Cabinet Maury" with a building "Forge Block" of declared 2 units
    And the building "Forge Block" has a unit "A1" with quota 100
    When admin gets building "Forge Block" by id
    Then the building is_conformant should be false
    And the building quota_sum should be "100"

  # ==========================================================================
  # @negative — défaillance correcte (pas de panic, pas de Result<_, String>)
  # ==========================================================================

  @negative
  Scenario: Getting an unknown building id returns a typed not-found error
    When admin gets a building by an unknown id
    Then the building operation should fail with a not found error

  @negative
  Scenario: Building with empty units returns Decimal::ZERO not NaN (Decimal strict)
    Given an existing organization "Cabinet Maury" with a building "Pristine" of declared 5 units
    When admin gets building "Pristine" by id
    Then the building quota_sum should be a valid decimal string
    And the building is_conformant should be false
