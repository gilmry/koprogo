Feature: Validate-before-compute on use-cases (Track H Story H2 FR-H2)
  As a syndic
  I want financial use-cases to refuse computing on a non-conformant building
  So that no charges/calls are emitted with drifting tantièmes (Art. 3.85 CC)

  # Build par défaut : `total_tantiemes=1000` (acte de base standard). Les
  # scénarios @security utilisent un drift de 2.5 sur 1 lot manquant pour
  # déclencher `BuildingNotConformantError` côté domain (Story H1) puis
  # `From<>` for String → handler 422 narratif (Story H2).
  #
  # Note : `charge_distribution` et `etat_date` sont couverts via tests
  # unitaires inline + Playwright API direct (cf. `validate-before-compute.spec.ts`).
  # Ce harness BDD se concentre sur expense + call_for_funds (use-cases legacy).

  Background:
    Given a validate-before-compute system

  # ============================================================================
  # @happy — chemin nominal : building conforme → use-case réussit
  # ============================================================================

  @happy
  Scenario: create_expense succeeds on conformant building (basis 1000)
    Given a conformant building "Conformant Towers" with 2 units summing to 1000
    When syndic submits a new expense on building "Conformant Towers"
    Then the use-case succeeds

  @happy
  Scenario: create_call_for_funds succeeds on conformant building
    Given a conformant building "Conformant Towers" with 2 units summing to 1000
    When syndic creates a call for funds on building "Conformant Towers"
    Then the use-case succeeds

  # ============================================================================
  # @security — bypass tenté sur immeuble non-conforme → ACP_NOT_CONFORMANT
  # ============================================================================

  @security
  Scenario: create_expense blocked on non-conformant building (Drift Manor)
    Given a non-conformant building "Drift Manor" with 1 unit summing to 997.5
    When syndic submits a new expense on building "Drift Manor"
    Then the use-case fails with ACP_NOT_CONFORMANT
    And the error mentions quota_delta "2.5" and quota_basis 1000

  @security
  Scenario: create_call_for_funds blocked on non-conformant building
    Given a non-conformant building "Drift Manor" with 1 unit summing to 997.5
    When syndic creates a call for funds on building "Drift Manor"
    Then the use-case fails with ACP_NOT_CONFORMANT
    And the error mentions quota_delta "2.5" and quota_basis 1000

  # ============================================================================
  # @edge — admin corrige le drift, re-tente : succès
  # ============================================================================

  @edge
  Scenario: building becomes conformant after admin adds missing unit
    Given a non-conformant building "Drift Manor" with 1 unit summing to 997.5
    When admin adds the missing unit on "Drift Manor" with quota 2.5
    Then the building "Drift Manor" is conformant
    And syndic submits a new expense on building "Drift Manor"
    And the use-case succeeds

  # ============================================================================
  # @negative — building inexistant → erreur cohérente (pas de panic)
  # ============================================================================

  @negative
  Scenario: create_expense on non-existent building fails with not-found
    When syndic submits a new expense on a non-existent building
    Then the use-case fails with not-found
