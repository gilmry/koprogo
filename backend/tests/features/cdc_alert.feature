Feature: CdC élu et alerte à la prochaine AG (Story 4.7 / #582)
  As a board member (conseil de copropriété)
  I want to alert the next general assembly
  So that the control mission of Art. 3.90 §1er CC has an instrument, not just a title

  Background:
    Given a coproperty management system

  # ==========================================================================
  # @happy — chemin nominal
  # ==========================================================================

  @happy
  Scenario: Le conseil élu en AG peut alerter la prochaine assemblée
    Given a building "Résidence Test"
    And a completed meeting "AG 2026" for that building
    And a scheduled meeting "AG 2027" for that building
    When the CdC is elected with 3 members from "AG 2026"
    Then the election should succeed with 3 members
    When the first elected member creates an alert "Toiture en mauvais état" with severity "warning" targeting "AG 2027"
    Then the alert operation should succeed
    And the alert should be visible at "AG 2027"

  # ==========================================================================
  # @negative — défaillance correcte
  # ==========================================================================

  @negative
  Scenario: Élire le conseil sur une AG non clôturée est refusé (quorum non prouvé)
    Given a building "Résidence Sans Quorum"
    And a scheduled meeting "AG 2026" for that building
    When the CdC is elected with 1 member from "AG 2026"
    Then the election should fail with a quorum not reached error

  # ==========================================================================
  # @edge — bornes
  # ==========================================================================

  @edge
  Scenario: Une démission fait perdre les droits immédiatement
    Given a building "Résidence Démission"
    And a completed meeting "AG 2026" for that building
    And a scheduled meeting "AG 2027" for that building
    And the CdC is elected with 1 member from "AG 2026"
    When the elected member resigns
    And the first elected member creates an alert "Fuite d'eau" with severity "critical" targeting "AG 2027"
    Then the alert operation should fail with a forbidden error

  # ==========================================================================
  # @security — RBAC, mandat
  # ==========================================================================

  @security
  Scenario: Un copropriétaire non élu ne peut pas alerter
    Given a building "Résidence Sécurité"
    And a scheduled meeting "AG 2027" for that building
    When a non-elected owner creates an alert "Suspicion de malversation" with severity "critical" targeting "AG 2027"
    Then the alert operation should fail with a forbidden error

  @security
  Scenario: Un ancien membre après la fin de son mandat ne peut plus alerter
    Given a building "Résidence Mandat Échu"
    And a completed meeting "AG 2026" for that building
    And a scheduled meeting "AG 2027" for that building
    And the CdC is elected with 1 member from "AG 2026"
    And the elected member's mandate has already ended
    When the first elected member creates an alert "Chaudière défaillante" with severity "warning" targeting "AG 2027"
    Then the alert operation should fail with a forbidden error
