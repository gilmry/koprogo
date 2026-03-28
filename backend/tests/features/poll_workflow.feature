# Feature: Sondage de consultation entre AG — Workflow multi-roles
# Issue #346 — Spec: docs/specs/05-sondage-entre-ag.rst
# Art. 577-8/4 §4 Code Civil Belge: consultations entre assemblees
# Resultats consultatifs uniquement — aucune force de loi

Feature: Sondage de consultation entre AG — Residents vs investisseurs absents
  As Francois Leroy (syndic)
  I want to poll co-owners between general assemblies
  So that I can measure owner sentiment on building decisions (consultative only)

  Background:
    Given the system is initialized
    And the building "Residence du Parc Royal" with 182 lots and 10000 tantiemes
    And Francois Leroy is syndic of the building
    And the following co-owners exist:
      | name                    | tantiemes | role                  | profile                          |
      | Alice Dubois            | 450       | Presidente CdC        | Retraitee 67 ans, impliquee      |
      | Charlie Martin          | 660       | Coproprietaire        | Jeune couple, charges = 48% rev. |
      | Nadia Benali            | 320       | Coproprietaire        | Infirmiere, credit variable      |
      | Marguerite Lemaire      | 380       | Coproprietaire        | Veuve 78 ans, pension survie     |
      | Marcel Dupont           | 450       | Coproprietaire        | Retraite 67 ans, veut renover    |
      | Philippe Vandermeulen   | 1800      | Coproprietaire (inv.) | 3 lots loues, jamais present     |
      | Emmanuel Claes          | 1280      | Coproprietaire (inv.) | Penthouse loue, absent           |

  # ===============================================================
  # WORKFLOW COMPLET : Sondage YesNo "Repeindre le hall"
  # 5 residents votent Oui, 2 investisseurs absents ne repondent pas
  # ===============================================================

  Scenario: Workflow complet — Sondage YesNo consultatif "Repeindre le hall"
    # Etape 1: Francois cree le sondage (Draft)
    When Francois creates a YesNo poll:
      | question    | Faut-il repeindre le hall d'entree ?                                |
      | description | Le hall d'entree montre des signes de degradation — peinture ecaillee, traces d'humidite. Ce sondage mesure l'interet avant la prochaine AG. |
      | is_anonymous | false                                                              |
      | ends_at     | 2026-04-15T23:59:59Z                                               |
    Then the poll should be created successfully
    And the poll status should be "Draft"
    And the poll type should be "YesNo"
    And the poll should have 2 options: "Oui" and "Non"
    # Le sondage n'est pas visible par les coproprietaires tant que Draft

    # Etape 2: Francois publie le sondage (Draft -> Active)
    When Francois publishes the poll
    Then the poll status should change to "Active"
    # Notification envoyee aux coproprietaires

    # Etape 3: Alice vote en premiere (presidente CdC, donne l'exemple)
    When Alice votes "Oui" on the poll
    Then total_votes_cast should be 1
    And the "Oui" option should have vote_count 1
    # Alice traverse le hall degrade chaque jour

    # Etape 4: Charlie vote Oui (hall degrade = mauvaise surprise a l'achat)
    When Charlie votes "Oui" on the poll
    Then total_votes_cast should be 2

    # Etape 5: Nadia vote Oui (un hall propre preserve la valeur de son bien)
    When Nadia votes "Oui" on the poll
    Then total_votes_cast should be 3
    # Nadia espere que le cout sera echelonne

    # Etape 6: Marguerite vote Oui (y passe chaque jour, inquiete du cout)
    When Marguerite votes "Oui" on the poll
    Then total_votes_cast should be 4
    # Son livret de 12.000 EUR est sa seule securite

    # Etape 7: Marcel vote Oui avec enthousiasme
    When Marcel votes "Oui" on the poll
    Then total_votes_cast should be 5
    # Marcel: "Il etait temps de s'en occuper!"

    # Philippe (1800 tantiemes) et Emmanuel (1280 tantiemes) ne repondent pas
    # Philippe ne lit pas l'email — ses 3 appartements sont geres par une agence
    # Emmanuel est en deplacement — la copropriete est un investissement passif

    # Etape 8: Francois cloture le sondage et consulte les resultats
    When Francois closes the poll
    Then the poll status should change to "Closed"
    And the winning option should be "Oui" with 5 votes
    And the "Oui" percentage should be 100%
    And the "Non" percentage should be 0%
    And the participation rate should be 50%
    # 5 votants sur 10 coproprietaires nommes
    # 100% des residents ayant repondu sont favorables
    # Les 2 investisseurs absents (30.8% des tantiemes) n'ont pas participe

  # ===============================================================
  # INDICATEUR SOCIAL : le taux de participation revele le desengagement
  # ===============================================================

  Scenario: Les investisseurs absents ne bloquent pas la consultation
    Given an active YesNo poll "Faut-il repeindre le hall d'entree ?"
    And Alice, Charlie, Nadia, Marguerite, Marcel have voted "Oui"
    And Philippe and Emmanuel have not voted
    When Francois closes the poll
    Then the results are valid despite investor absence
    And the participation rate reflects disengagement at 50%
    # Francois documentera ces resultats dans le PV de la prochaine AG
    # pour illustrer le desengagement des investisseurs

  # ===============================================================
  # PREVENTION DES DOUBLONS
  # ===============================================================

  Scenario: Vote duplique interdit — Alice ne peut pas voter deux fois
    Given an active YesNo poll "Faut-il repeindre le hall d'entree ?"
    And Alice has already voted "Oui"
    When Alice tries to vote "Non" on the same poll
    Then an error "You have already voted on this poll" is returned
    And Alice's original "Oui" vote should remain unchanged

  # ===============================================================
  # REGLES DE LIFECYCLE
  # ===============================================================

  Scenario: Vote impossible sur sondage Draft
    Given a draft poll "Faut-il repeindre le hall d'entree ?"
    When Nadia tries to vote on the draft poll
    Then an error "Poll is not currently accepting votes" is returned

  Scenario: Seul un sondage Draft peut etre publie
    Given an active poll "Faut-il repeindre le hall d'entree ?"
    When Francois tries to publish the already active poll
    Then an error "Only draft polls can be published" is returned

  Scenario: Annulation d'un sondage actif par le syndic
    Given an active poll "Faut-il repeindre le hall d'entree ?"
    When Francois cancels the poll
    Then the poll status should change to "Cancelled"

  Scenario: Impossible d'annuler un sondage cloture
    Given a closed poll "Faut-il repeindre le hall d'entree ?"
    When Francois tries to cancel the closed poll
    Then an error "Cannot cancel a closed poll" is returned

  Scenario: Cloture automatique apres date de fin
    Given an active poll with ends_at in the past
    When the auto-close background job runs
    Then the poll status should change to "Closed" automatically

  # ===============================================================
  # SONDAGE MULTIPLECHOICE — Choix entrepreneur facade
  # ===============================================================

  Scenario: Sondage MultipleChoice — choix entrepreneur pour facade
    When Francois creates a MultipleChoice poll:
      | question    | Quel entrepreneur pour le ravalement de facade ?   |
      | description | Nous avons recu 3 devis. Choisissez le meilleur.  |
      | is_anonymous | false                                             |
      | ends_at     | 2026-04-20T23:59:59Z                              |
    And Francois adds options:
      | option_text               | display_order |
      | Toitures Bruxelles        | 1             |
      | Renov'Art SPRL            | 2             |
      | Lejeune & Fils            | 3             |
    And Francois publishes the poll
    # Votes
    When Alice votes "Toitures Bruxelles" on the poll
    And Marcel votes "Toitures Bruxelles" on the poll
    And Charlie votes "Renov'Art SPRL" on the poll
    And Nadia votes "Renov'Art SPRL" on the poll
    And Marguerite votes "Lejeune & Fils" on the poll
    # Francois closes and views results
    When Francois closes the poll
    Then the winning option should be "Toitures Bruxelles" with 2 votes
    And the participation rate should be 50%
    # Resultats consultatifs — seront presentes a la prochaine AG

  # ===============================================================
  # SONDAGE ANONYME
  # ===============================================================

  Scenario: Sondage anonyme — owner_id non stocke (GDPR)
    When Francois creates an anonymous YesNo poll:
      | question    | Etes-vous satisfait du nettoyage des communs ?  |
      | is_anonymous | true                                           |
      | ends_at     | 2026-04-10T23:59:59Z                           |
    And Francois publishes the poll
    When Alice votes "Oui" on the anonymous poll
    Then the vote should be recorded with owner_id NULL
    And the IP address should be logged for audit
    # GDPR Art. 6: anonymous voting preserves privacy

  # ===============================================================
  # RESULTATS CONSULTATIFS — Pas de force de loi
  # Art. 577-8/4 §4 CC: les resultats doivent etre documentes
  # dans le PV de la prochaine AG
  # ===============================================================

  Scenario: Resultats consultatifs documentes pour la prochaine AG
    Given a closed poll "Faut-il repeindre le hall d'entree ?" with:
      | Oui votes  | 5   |
      | Non votes  | 0   |
      | participation | 50% |
    When Francois retrieves the poll results
    Then the results should include:
      | winning_option | Oui  |
      | winning_pct    | 100% |
      | participation  | 50%  |
    # These results have no legal force
    # Francois must present them at the next AG for formal ratification
    # if a binding decision is needed (Art. 577-8/4 §4 CC)
