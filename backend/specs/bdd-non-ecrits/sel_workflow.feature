# Feature: SEL — Echange entre coproprietaires de la Residence du Parc Royal
# Workflow multi-roles avec personas du seed
# Issue #346 — Spec: docs/specs/04-sel-echange.rst
# Legal: SEL is legal in Belgium, non-taxable if non-commercial

Feature: SEL — Echange local entre residents (monnaie temporelle 1h = 1 credit)
  As Alice Dubois (retiree, cooking class provider)
  I want to exchange services with neighbors using time-based credits
  So that community solidarity is built across generations in the Residence du Parc Royal

  Background:
    Given the system is initialized
    And the building "Residence du Parc Royal" with 182 lots and 10000 tantiemes
    And Alice Dubois (retraitee 67 ans, lot 2A, 450/10000) with a credit balance of 0
    And Bob Janssen (comptable 55 ans, lot 2B, 430/10000) with a credit balance of 0
    And Nadia Benali (infirmiere 32 ans, lot 4A, 320/10000) with a credit balance of 0
    And Marguerite Lemaire (veuve 78 ans, lot 1A, 380/10000) with a credit balance of 0
    And Ahmed Mansouri is a tenant of lot 6A (Philippe's apartment)

  # ===============================================================
  # WORKFLOW COMPLET : Alice donne un cours de cuisine a Nadia
  # ===============================================================

  Scenario: Workflow complet — Alice donne un cours de cuisine belge a Nadia
    # Etape 1: Alice publie une offre de cours de cuisine
    When Alice creates a service exchange offer:
      | title       | Cours de cuisine belge — Waterzooi                           |
      | description | Je propose un cours de 2h pour apprendre a preparer un waterzooi. Ingredients fournis, on cuisine ensemble! |
      | credits     | 2                                                            |
    Then the exchange should be created successfully
    And the status should be "Offered"
    And the exchange type should be "Service"
    And the provider should be Alice

    # Etape 2: Nadia demande le cours (elle ne connait personne dans l'immeuble)
    When Nadia requests Alice's exchange
    Then the exchange status should change to "Requested"
    And the requester should be Nadia
    And requested_at should be set

    # Etape 3: Alice accepte et demarre l'echange
    When Alice starts the exchange
    Then the exchange status should change to "InProgress"
    And started_at should be set

    # Etape 4: Alice confirme la completion apres le cours de waterzooi
    When Alice completes the exchange
    Then the exchange status should change to "Completed"
    And completed_at should be set
    # Mise a jour atomique des soldes de credits
    And Alice's credit balance should be +2 (credits_earned=2, balance=2)
    And Nadia's credit balance should be -2 (credits_spent=2, balance=-2)
    And total_exchanges should be 1 for both Alice and Nadia

    # Etape 5: Ratings mutuels
    When Nadia rates Alice with 5 stars
    Then provider_rating should be 5

    When Alice rates Nadia with 5 stars
    Then requester_rating should be 5
    And the exchange should have mutual ratings complete

  # ===============================================================
  # VARIANTE : Bob depanne l'ordinateur de Marguerite (lien social)
  # ===============================================================

  Scenario: Bob depanne l'ordinateur de Marguerite — puis elle offre du repassage
    # Bob (comptable, 55 ans) aide Marguerite (veuve, 78 ans) avec sa tablette
    When Bob creates a service exchange offer:
      | title       | Depannage informatique — tablette et email                  |
      | description | Configuration email, mise a jour, aide imprimante. 1h max. |
      | credits     | 1                                                           |
    And Marguerite requests Bob's exchange
    And Bob starts the exchange
    And Bob completes the exchange
    Then Bob's credit balance should be +1 (credits_earned=1, balance=1)
    And Marguerite's credit balance should be -1 (credits_spent=1, balance=-1)

    # Marguerite offre du repassage en retour (se sentir utile = dignite)
    When Marguerite creates a service exchange offer:
      | title       | Repassage soigne                                            |
      | description | Je propose du repassage. Venez deposer vos vetements au 1A. |
      | credits     | 1                                                           |
    And Bob requests Marguerite's exchange
    And Marguerite starts the exchange
    And Marguerite completes the exchange
    Then Marguerite's credit balance should be 0 (credits_earned=1, credits_spent=1, balance=0)
    And Bob's credit balance should be 0 (credits_earned=1, credits_spent=1, balance=0)
    And Marguerite should have total_exchanges of 2
    # Pour Marguerite, ce n'est pas seulement un echange — c'est son seul lien social

  # ===============================================================
  # VARIANTE : Ahmed (locataire) participe au SEL
  # Le SEL est ouvert a tous les habitants, pas seulement aux proprietaires
  # ===============================================================

  Scenario: Ahmed (locataire de Philippe) participe au SEL
    When Ahmed creates a service exchange offer:
      | title       | Creation site web simple                              |
      | description | Je suis developpeur web freelance, je peux creer un site vitrine en 2h. |
      | credits     | 2                                                     |
    Then the exchange should be created successfully
    And the provider should be Ahmed
    # Ahmed peut participer au SEL meme s'il n'a pas de droit de vote en AG

    When Alice requests Ahmed's exchange
    And Ahmed starts the exchange
    And Ahmed completes the exchange
    Then Ahmed's credit balance should be +2
    And Alice's credit balance should be -2

  # ===============================================================
  # VARIANTE : Marguerite offre du repassage (lien social vital)
  # ===============================================================

  Scenario: Marguerite offre du repassage — son seul lien social dans l'immeuble
    When Marguerite creates a service exchange offer:
      | title       | Repassage soigne — chemises et pantalons               |
      | description | 40 ans d'experience. Deposez vos vetements au 1A, je les rends le lendemain. |
      | credits     | 1                                                      |
    Then the exchange should be created successfully
    And the status should be "Offered"
    # Pour Marguerite (78 ans, veuve), se sentir utile lui redonne de la dignite

  # ===============================================================
  # SOLDES NEGATIFS — Modele de confiance communautaire
  # ===============================================================

  Scenario: Nadia peut continuer a demander malgre un solde negatif
    Given Nadia has a credit balance of -2 after Alice's cooking class
    When a 1-credit service exchange is completed with Nadia as requester
    Then Nadia's credit balance should be -3
    # Pas de blocage — le SEL fonctionne sur la confiance
    # Nadia n'ose pas offrir de service (syndrome imposteur) mais
    # quand elle se sentira prete, elle pourra offrir des conseils sante

  # ===============================================================
  # VALIDATIONS
  # ===============================================================

  Scenario: Interdiction d'auto-echange — Alice ne peut pas demander son propre cours
    When Alice creates a service exchange offer:
      | title       | Cours de cuisine belge    |
      | description | Waterzooi et carbonnade   |
      | credits     | 2                         |
    And Alice tries to request her own exchange
    Then an error "Provider cannot request their own exchange" is returned

  Scenario: Seul le provider peut demarrer l'echange
    Given Alice has an exchange offer and Nadia has requested it
    When Nadia tries to start the exchange
    Then an error "Only the provider can start the exchange" is returned

  Scenario: Annulation avant completion — aucun credit transfere
    Given Alice has an exchange offer and Nadia has requested it
    When Nadia cancels with reason "Garde d'enfant impossible ce samedi"
    Then the exchange status should change to "Cancelled"
    And the cancellation reason should be "Garde d'enfant impossible ce samedi"
    And no credit transfer should occur

  Scenario: Impossible d'annuler un echange complete
    Given a completed exchange between Alice and Nadia
    When Alice tries to cancel the exchange
    Then an error "Cannot cancel a completed exchange" is returned

  Scenario: Validation des ratings — limites 1 a 5 etoiles
    Given a completed exchange between Alice and Nadia
    When Nadia tries to rate Alice with 0 stars
    Then an error "Rating must be between 1 and 5" is returned
    When Nadia tries to rate Alice with 6 stars
    Then an error "Rating must be between 1 and 5" is returned

  Scenario: Credits doivent etre entre 1 et 100
    When Alice tries to create an exchange with 0 credits
    Then the creation should fail
    When Alice tries to create an exchange with 101 credits
    Then the creation should fail

  # ===============================================================
  # LEADERBOARD & STATISTIQUES
  # ===============================================================

  Scenario: Leaderboard de l'immeuble — Alice en tete apres son cours
    Given Alice has a credit balance of +2 (1 exchange completed as provider)
    And Bob has a credit balance of +1 (1 exchange completed as provider)
    And Marguerite has a credit balance of 0 (1 exchange each way)
    And Nadia has a credit balance of -2 (1 exchange completed as requester)
    When I view the building leaderboard
    Then Alice should be ranked #1 with balance 2
    And Bob should be ranked #2 with balance 1

  Scenario: Statistiques SEL de l'immeuble
    Given 3 exchanges have been completed in the building
    When I request SEL statistics for the building
    Then total_exchanges should be 3
    And the most popular exchange type should be "Service"
    And active_participants should be at least 4

  Scenario: Niveaux de participation de Marguerite
    Given Marguerite has completed 2 exchanges total
    When I check Marguerite's participation level
    Then her participation_level should be "Beginner"
    # 1-5 exchanges = Beginner
