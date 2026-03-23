# Feature: Resolution linked to Agenda Item (Issue #310)
# Art. 3.87 CC: Only items on the agenda can be voted on in a general assembly
# agenda_item_index references meeting.agenda Vec index

Feature: Resolution Agenda Item Linking
  As a syndic managing a general assembly
  I want to link resolutions to specific agenda items
  So that voting respects Belgian copropriete law (Art. 3.87 CC)

  Background:
    Given the system is initialized
    And an organization "Agenda Copro ASBL" exists with id "org-agenda"
    And a building "Residence Agenda" exists in organization "org-agenda"
    And a meeting "AGO 2025" exists for building "Residence Agenda"
    And the meeting has the following agenda items:
      | index | item                                        |
      | 0     | Approbation des comptes 2024                |
      | 1     | Budget previsionnel 2025                    |
      | 2     | Travaux de renovation facade                |
      | 3     | Election du conseil de coproprietaires      |
    And the meeting quorum is validated with 600 out of 1000 quotas
    And the user is authenticated as syndic "Jean Syndic"

  # === SUCCESSFUL CREATION WITH AGENDA LINK ===

  Scenario: Create resolution linked to first agenda item
    When I create a resolution for the meeting:
      | title              | Approbation des comptes annuels  |
      | description        | Vote sur les comptes 2024        |
      | resolution_type    | Ordinary                         |
      | majority_required  | Simple                           |
      | agenda_item_index  | 0                                |
    Then the resolution should be created successfully
    And the resolution agenda_item_index should be 0

  Scenario: Create resolution linked to last agenda item
    When I create a resolution for the meeting:
      | title              | Election conseil                 |
      | description        | Vote pour nouveaux membres       |
      | resolution_type    | Ordinary                         |
      | majority_required  | Absolute                         |
      | agenda_item_index  | 3                                |
    Then the resolution should be created successfully
    And the resolution agenda_item_index should be 3

  Scenario: Create resolution linked to works agenda item with qualified majority
    When I create a resolution for the meeting:
      | title              | Renovation facade sud            |
      | description        | Approbation devis renovation     |
      | resolution_type    | Extraordinary                    |
      | majority_required  | Qualified                        |
      | agenda_item_index  | 2                                |
    Then the resolution should be created successfully
    And the resolution agenda_item_index should be 2

  # === CREATION WITHOUT AGENDA LINK ===

  Scenario: Create resolution without agenda item link
    When I create a resolution for the meeting:
      | title              | Motion diverse                   |
      | description        | Discussion informelle            |
      | resolution_type    | Ordinary                         |
      | majority_required  | Simple                           |
    Then the resolution should be created successfully
    And the resolution agenda_item_index should be null

  # === VALIDATION ERRORS ===

  Scenario: Reject resolution with agenda index beyond agenda length
    When I create a resolution for the meeting:
      | title              | Invalid agenda reference          |
      | description        | This should not be allowed        |
      | resolution_type    | Ordinary                          |
      | majority_required  | Simple                            |
      | agenda_item_index  | 10                                |
    Then the resolution creation should fail
    And the error should contain "valid agenda item"
    And the error should contain "Art. 3.87 CC"

  Scenario: Reject resolution with agenda index equal to agenda length
    When I create a resolution for the meeting:
      | title              | Off-by-one agenda reference       |
      | description        | Index 4 on 4-item agenda          |
      | resolution_type    | Ordinary                          |
      | majority_required  | Simple                            |
      | agenda_item_index  | 4                                 |
    Then the resolution creation should fail
    And the error should contain "valid agenda item"

  Scenario: Reject resolution referencing empty agenda item
    Given the meeting has an empty agenda item at index 2
    When I create a resolution for the meeting:
      | title              | References empty item             |
      | description        | Agenda item is blank              |
      | resolution_type    | Ordinary                          |
      | majority_required  | Simple                            |
      | agenda_item_index  | 2                                 |
    Then the resolution creation should fail
    And the error should contain "Agenda item cannot be empty"

  # === QUORUM ENFORCEMENT ===

  Scenario: Reject resolution creation when quorum not validated
    Given a meeting "AGO sans quorum" exists for building "Residence Agenda"
    And the meeting quorum has not been validated
    When I create a resolution for the meeting:
      | title              | Resolution sans quorum            |
      | description        | Should not be allowed             |
      | resolution_type    | Ordinary                          |
      | majority_required  | Simple                            |
      | agenda_item_index  | 0                                 |
    Then the resolution creation should fail
    And the error should contain "quorum"

  Scenario: Allow resolution creation on second convocation without quorum
    Given a meeting "2e Convocation" exists for building "Residence Agenda"
    And the meeting is marked as second convocation
    And the meeting has agenda item "Budget 2025" at index 0
    When I create a resolution for the meeting:
      | title              | Budget sans quorum                |
      | description        | 2e convocation - pas de quorum    |
      | resolution_type    | Ordinary                          |
      | majority_required  | Simple                            |
      | agenda_item_index  | 0                                 |
    Then the resolution should be created successfully
    And the resolution agenda_item_index should be 0

  # === MEETING NOT FOUND ===

  Scenario: Reject resolution for non-existent meeting
    When I create a resolution for a non-existent meeting:
      | title              | Orphan resolution                 |
      | description        | Meeting does not exist            |
      | resolution_type    | Ordinary                          |
      | majority_required  | Simple                            |
    Then the resolution creation should fail
    And the error should contain "Meeting not found"
