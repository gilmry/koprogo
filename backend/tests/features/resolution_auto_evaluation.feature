Feature: Résolution EvaluationContractorsAuto — AGO auto, non retirable (Story 4.6 — #581)
  En tant que copropriétaire
  Je veux que l'évaluation des prestataires figure d'office à l'ordre du jour
  de l'assemblée ordinaire
  Afin que ce point ne puisse pas être retiré par celui-là même dont il évalue
  le travail

  # ----------------------------------------------------------------------
  # @happy
  # ----------------------------------------------------------------------

  @happy
  Scenario: Une AGO génère d'office sa résolution d'évaluation des prestataires
    Given une assemblée générale ordinaire
    When les résolutions automatiques sont générées
    Then une résolution "EvaluationContractorsAuto" est présente
    And elle est marquée comme générée automatiquement

  # ----------------------------------------------------------------------
  # @edge — l'obligation légale (Art. 3.89 § 5, 12° CC) porte sur l'AGO, pas
  # sur l'AGE : l'étendre à l'extraordinaire ajouterait à la loi.
  # ----------------------------------------------------------------------

  @edge
  Scenario: Une AGE ne génère aucune résolution automatique
    Given une assemblée générale extraordinaire
    When les résolutions automatiques sont générées
    Then aucune résolution automatique n'est générée

  # ----------------------------------------------------------------------
  # @security — le cœur de la story : le syndic évalué ne peut pas retirer
  # son propre bulletin de notes de l'ordre du jour.
  # ----------------------------------------------------------------------

  @security
  Scenario: Le syndic tente de supprimer la résolution auto-générée
    Given une assemblée générale ordinaire
    And les résolutions automatiques sont générées
    When le syndic tente de supprimer la résolution auto-générée
    Then la suppression est refusée avec l'erreur "ResolutionAutoNotRemovable"

  # ----------------------------------------------------------------------
  # @negative — immuable, pas seulement non supprimable.
  # ----------------------------------------------------------------------

  @negative
  Scenario: Modification du texte d'une résolution auto-générée refusée
    Given une assemblée générale ordinaire
    And les résolutions automatiques sont générées
    When le syndic modifie le texte de la résolution auto-générée
    Then la modification est refusée avec l'erreur "ResolutionAutoNotRemovable"
