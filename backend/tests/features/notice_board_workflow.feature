# Feature: Tableau d'affichage communautaire — Workflow multi-roles
# Issue #346 — Spec: docs/specs/08-notice-board.rst
# No specific CC article — community feature, not a legal obligation
# The notice board is the digital social link for isolated residents

Feature: Tableau d'affichage communautaire — Multi-auteurs et lecteurs
  As a resident of the Residence du Parc Royal
  I want to post and read notices on the community board
  So that building information flows and social isolation is reduced

  Background:
    Given the system is initialized
    And the building "Residence du Parc Royal" with 182 lots and 10000 tantiemes
    And Francois Leroy is syndic of the building
    And Alice Dubois is presidente of the CdC (lot 2A, 450/10000)
    And Charlie Martin is co-owner (lot 3B, 660/10000, charges = 48% du revenu)
    And Marguerite Lemaire is co-owner (lot 1A, 380/10000, veuve 78 ans)
    And Ahmed Mansouri is a tenant of lot 6A (Philippe's apartment)

  # ===============================================================
  # ETAPE 1 : Francois publie une annonce travaux parking (Announcement)
  # ===============================================================

  Scenario: Francois publie et epingle une annonce travaux parking
    Given Francois is authenticated as syndic
    When Francois creates a notice:
      | notice_type | Announcement                                            |
      | category    | Parking                                                 |
      | title       | Fermeture du parking souterrain pour travaux             |
      | content     | Le parking souterrain sera ferme du 15 au 30 avril pour travaux de refection du sol et mise aux normes de l'eclairage. Veuillez utiliser le parking exterieur pendant cette periode. |
      | expires_at  | 2026-05-01T00:00:00Z                                    |
    Then the notice should be created successfully
    And the notice status should be "Draft"
    And is_pinned should be false

    When Francois publishes the notice
    Then the notice status should be "Published"
    And published_at should be set

    When Francois pins the notice
    Then is_pinned should be true
    # L'annonce epinglee apparaitra en tete de la liste pendant les travaux

  # ===============================================================
  # ETAPE 2 : Alice publie un evenement SEL (Event)
  # ===============================================================

  Scenario: Alice publie un evenement cours de cuisine SEL
    Given Alice is authenticated as co-owner
    When Alice creates a notice:
      | notice_type    | Event                                                          |
      | category       | Social                                                         |
      | title          | Cours de cuisine — Atelier pain maison (SEL)                   |
      | content        | Rejoignez-moi pour apprendre a faire du pain au levain ! Atelier ouvert a tous les residents. Ingredients fournis. Valeur SEL : 2 credits. Places : 6. |
      | event_date     | 2026-04-05T10:00:00Z                                           |
      | event_location | Salle commune, rez-de-chaussee                                 |
      | contact_info   | alice@residence-parc.be ou apt 2A                              |
      | expires_at     | 2026-04-05T23:59:59Z                                           |
    Then the notice should be created successfully
    And the notice type should be "Event"
    And the event_date should be set
    And the event_location should be "Salle commune, rez-de-chaussee"

    When Alice publishes the notice
    Then the notice status should be "Published"
    # L'annonce expire automatiquement apres la date de l'evenement

  Scenario: Alice cree un evenement sans date echoue
    Given Alice is authenticated as co-owner
    When Alice creates a notice of type "Event" without event_date
    Then an error "Event notices must have an event_date" is returned

  Scenario: Alice cree un evenement sans lieu echoue
    Given Alice is authenticated as co-owner
    When Alice creates a notice of type "Event" with event_date but without event_location
    Then an error "Event notices must have an event_location" is returned

  # ===============================================================
  # ETAPE 3 : Charlie publie une petite annonce (ClassifiedAd)
  # Pour Charlie (charges = 48% du revenu), chaque euro compte
  # ===============================================================

  Scenario: Charlie publie une petite annonce — vends poussette
    Given Charlie is authenticated as co-owner
    When Charlie creates a notice:
      | notice_type  | ClassifiedAd                                                      |
      | category     | General                                                           |
      | title        | Vends poussette Bugaboo Fox 3 — bon etat                          |
      | content      | Poussette Bugaboo Fox 3, utilisee 2 ans, tres bon etat. Nacelle + hamac + habillage pluie inclus. Prix : 250 EUR (neuf : 1100 EUR). Priorite aux residents. |
      | contact_info | charlie@residence-parc.be ou apt 3B                               |
      | expires_at   | 2026-04-30T00:00:00Z                                              |
    And Charlie publishes the notice
    Then the notice status should be "Published"
    # 250 EUR recuperes allegent un peu le budget de Charlie

  Scenario: Charlie supprime sa petite annonce une fois la poussette vendue
    Given Charlie's published notice "Vends poussette Bugaboo Fox 3" exists
    When Charlie deletes the notice
    Then the notice should be deleted

  # ===============================================================
  # ETAPE 4 : Marguerite lit les annonces (seul lien social numerique)
  # ===============================================================

  Scenario: Marguerite consulte les annonces — son lien social numerique quotidien
    Given 3 published notices exist:
      | author    | type          | title                                     | pinned |
      | Francois  | Announcement  | Fermeture du parking souterrain pour travaux | true   |
      | Alice     | Event         | Cours de cuisine — Atelier pain maison     | false  |
      | Charlie   | ClassifiedAd  | Vends poussette Bugaboo Fox 3              | false  |
    When Marguerite browses the building notices
    Then the pinned notice from Francois (travaux parking) should appear first
    And Marguerite should see 3 notices total
    # Marguerite (78 ans, veuve) lit les annonces chaque matin sur sa tablette
    # C'est son SEUL lien social numerique avec l'immeuble
    # Elle ira peut-etre au cours de cuisine d'Alice
    # Elle parlera de la poussette de Charlie a sa petite-fille

  # ===============================================================
  # ETAPE 5 : Ahmed (locataire) consulte les annonces
  # Le tableau d'affichage est inclusif — meme sans droit de vote
  # ===============================================================

  Scenario: Ahmed (locataire) consulte les annonces — acces en lecture
    Given 3 published notices exist for the building
    When Ahmed browses the building notices
    Then Ahmed should see all 3 notices
    # Ahmed decouvre la fermeture du parking pour 2 semaines
    # Information vitale pour son quotidien de freelance
    # Le tableau d'affichage l'integre a la vie de l'immeuble
    # la ou l'AG l'exclut (pas de droit de vote)

  # ===============================================================
  # FILTRAGE ET LISTING
  # ===============================================================

  Scenario: Filtrage par type Event
    Given 1 Announcement (travaux) and 1 Event (cuisine) are published
    When one filters notices by type "Event"
    Then 1 notice of type "Event" should be returned (Alice's cooking class)

  Scenario: Annonces epinglees en tete de liste pour tous
    Given Francois's pinned notice and 2 unpinned notices
    When any resident browses the building notices
    Then the pinned notice should appear first

  # ===============================================================
  # VALIDATIONS
  # ===============================================================

  Scenario: Titre trop court echoue (minimum 5 caracteres)
    When one creates a notice with title "Sale"
    Then an error containing "at least 5 characters" is returned

  Scenario: Contenu vide echoue
    When one creates a notice with empty content
    Then an error containing "cannot be empty" is returned

  Scenario: Epingler un brouillon echoue
    Given a notice in status "Draft"
    When one tries to pin the notice
    Then an error "Only Published notices can be pinned" is returned

  Scenario: Modification d'une annonce publiee echoue
    Given a notice in status "Published"
    When one tries to update the title
    Then an error "Only Draft notices can be updated" is returned

  # ===============================================================
  # EXPIRATION ET ARCHIVAGE
  # ===============================================================

  Scenario: Annonce cours de cuisine expire automatiquement apres l'evenement
    Given Alice's notice with expires_at on April 5th 23:59
    When the date of April 6th is reached
    Then the notice status should be "Expired"
    And is_pinned should be false

  Scenario: Francois archive l'annonce travaux apres la fin des travaux
    Given Francois's published notice about parking closure
    When Francois archives the notice
    Then the notice status should be "Archived"
    And archived_at should be set
    And is_pinned should be false
    # L'annonce n'apparait plus dans la liste active mais reste dans l'historique

  Scenario: Archiver un brouillon echoue
    Given a notice in status "Draft"
    When one tries to archive the notice
    Then an error "Only Published or Expired notices can be archived" is returned

  # ===============================================================
  # CHARLIE MODIFIE SA PETITE ANNONCE (brouillon uniquement)
  # ===============================================================

  Scenario: Charlie modifie le prix de sa poussette avant publication
    Given Charlie's draft notice "Vends poussette Bugaboo Fox 3"
    When Charlie updates the title to "Vends poussette Bugaboo Fox 3 — PRIX REDUIT 200 EUR"
    Then the title should be updated
