Feature: Vote distant — authentification forte (Story 4.2, #48)

  Art. 3.87 §1er, §4 CC : un copropriétaire peut voter à distance, mais un
  vote à distance n'a de valeur probante que si l'on sait comment le votant
  a été authentifié. `presence` (feuille de présence) n'engage personne à
  distance ; `itsme`/`eid` ou une procuration en bonne et due forme, si.

  Background:
    Given the system is initialized
    And an organization "Vote Distant ASBL" exists with id "org-vote-distant"
    And a building "Residence Distancielle" exists in organization "org-vote-distant"
    And a meeting "AGO distancielle 2026" exists for the building
    And the meeting is held remotely
    And an owner "Alice" with 300 voting power (tantiemes) exists
    And an owner "Bob" with 200 voting power (tantiemes) exists
    And the AG is validly constituted (quorum reached)
    And a pending resolution "Approbation des comptes" exists

  # @happy — Owner vote distant avec itsme → vote enregistré avec auth_method=itsme
  Scenario: Vote distant avec itsme est enregistré
    When "Alice" votes "Pour" using "itsme"
    Then the vote should be recorded with auth method "itsme"

  # @edge — Owner tente vote distant avec proxy (procuration à distance) → autorisé
  # sous conditions Art. 3.87 §4 : la procuration elle-même reste régie par ses
  # propres conditions (limite de trois mandats), pas par cette garde.
  Scenario: Vote distant par procuration en bonne et due forme est autorisé
    When "Alice" votes "Pour" as proxy for "Bob" using "proxy"
    Then the vote should be recorded with auth method "proxy"

  # @security — Owner tente vote distant avec auth_method=presence → 403
  # VoteAuthInsufficient. Déclarer sa présence quand on est absent est
  # exactement la fraude que l'authentification forte doit rendre impossible.
  Scenario: Vote distant avec presence est refusé
    When "Alice" votes "Pour" using "presence"
    Then the vote should be rejected with error "VOTE_AUTH_INSUFFICIENT"

  # @negative — Vote sans auth_method → 422
  Scenario: Vote sans méthode d'authentification est refusé
    When "Alice" votes "Pour" without an authentication method
    Then the vote should be rejected with error "VOTE_AUTH_METHOD_REQUIRED"
