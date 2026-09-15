Feature: Meeting.mode hybrid + quorum agrégé Decimal (Story 4.1 — Art. 3.87 §1er/§5 CC)
  As a syndic
  I want the AG mode (in_person/remote/hybrid) and the aggregated quorum to be
  computed exactly in Decimal
  So that the legal validity of an AG never depends on a binary rounding
  error, and a hybrid AG is never announced without a working connection link

  # ----------------------------------------------------------------------
  # @happy
  # ----------------------------------------------------------------------

  @happy
  Scenario: Hybrid AG aggregates in-person, remote and proxy quotas
    Given a coproperty management system
    And a meeting mode hybrid
    And un total de 1000 quotités
    And 10 présentiels, 5 distants et 3 procurations
    When le quorum agrégé est calculé
    Then le quorum agrégé est conforme à la somme Decimal exacte
    And le pourcentage de quorum vaut exactement 1.8%

  @happy
  Scenario: Hybrid AG reaches quorum once aggregated quotas pass half
    Given a coproperty management system
    And a meeting mode hybrid
    And un total de 1000 quotités
    And 400 présentiels, 150 distants et 60 procurations
    When le quorum agrégé est calculé
    Then le quorum est atteint

  @happy
  Scenario: Syndic configures hybrid mode with a videoconf URL
    Given a coproperty management system
    And une réunion planifiée "AGO 2026 hybride"
    When le syndic configure le mode hybrid avec l'URL "https://meet.jit.si/koprogo-ago-2026"
    Then la configuration du mode réussit

  # ----------------------------------------------------------------------
  # @edge — les deux côtés du seuil de 50% (Art. 3.87 §5 — au moins la moitié)
  # ----------------------------------------------------------------------

  @edge
  Scenario: Quorum at exactly 50.0% is reached
    Given a coproperty management system
    And a meeting mode hybrid
    And un total de 1000 quotités
    And 300 présentiels, 150 distants et 50 procurations
    When le quorum agrégé est calculé
    Then le pourcentage de quorum vaut exactement 50%
    And le quorum est atteint

  @edge
  Scenario: Quorum at 49.99% is refused
    Given a coproperty management system
    And a meeting mode hybrid
    And un total de 1000 quotités
    And 300 présentiels, 150 distants et 49.9 procurations
    When le quorum agrégé est calculé
    Then le pourcentage de quorum vaut exactement 49.99%
    And le quorum n'est pas atteint

  # ----------------------------------------------------------------------
  # @security
  # ----------------------------------------------------------------------

  @security
  Scenario: Remote mode requires strong authentication for remote quotas
    Given a coproperty management system
    And a meeting mode remote
    And un total de 1000 quotités
    And 0 présentiels, 500 distants et 0 procurations
    And aucune authentification forte distancielle
    When le quorum agrégé est calculé
    Then le calcul est refusé pour authentification distancielle manquante

  # ----------------------------------------------------------------------
  # @negative
  # ----------------------------------------------------------------------

  @negative
  Scenario: Hybrid mode without videoconf configuration is rejected (422)
    Given a coproperty management system
    And une réunion planifiée "AGO 2026 hybride"
    When le syndic configure le mode hybrid sans URL de visioconférence
    Then la configuration du mode échoue avec URL de visioconférence manquante

  @negative
  Scenario: Remote mode without videoconf configuration is rejected (422)
    Given a coproperty management system
    And une réunion planifiée "AGO 2026 distancielle"
    When le syndic configure le mode remote sans URL de visioconférence
    Then la configuration du mode échoue avec URL de visioconférence manquante
