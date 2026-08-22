# Feature: Governance Decimal exactness (ADR-0008 remediation)
# Belgian copro quorum / voting-power / quota arithmetic MUST be exact (rust_decimal::Decimal).
# Story 521-C1 — closes bug 525 (units.quota DOUBLE PRECISION -> Decimal ColumnDecode panic).
# Art. 3.87 paragraph 5 Code Civil Belge (quorum), Art. 3.87 paragraph 2 (AGE 1/5 threshold).

Feature: Governance Decimal exactness
  As a syndic closing a general assembly
  I want quorum and voting-power arithmetic to be exact decimal
  So that an IEEE754 rounding error never makes a deliberation legally contestable

  Background:
    Given the governance decimal system is initialized

  @negative @adr8 @story521-C1 @bug525
  Scenario: voting power decoded as Decimal does not panic on aggregation
    Given a resolution with votes of power "12.3456" and "7.8900"
    When the syndic aggregates the voting powers
    Then the aggregation succeeds without ColumnDecode panic
    And the sum is exactly "20.2356"

  @security @adr8 @story521-C1
  Scenario: quorum at the exact legal boundary is not skewed by rounding
    Given a building with total quotas "1000.0000"
    And present owners cumulating "500.0001" quotas
    When the syndic checks the quorum at threshold 50 percent
    Then the quorum is REACHED with exact comparison

  @edge @adr8 @story521-C1
  Scenario Outline: unit quotas at PCMN bounds round-trip exact
    Given a unit with quota "<q>" stored in the database
    When the unit is read back from the database
    Then the unit quota is exactly "<q>"

    Examples:
      | q        |
      | 0.0001   |
      | 999.9999 |
      | 333.3333 |

  @happy @adr8 @story521-C1
  Scenario: AGE request shares_pct round-trips exact
    Given an AGE request with a cosignatory shares_pct "0.250000"
    When the AGE request is read back
    Then the cosignatory shares_pct is exactly "0.250000"
