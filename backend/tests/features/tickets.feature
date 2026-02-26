# Feature: Maintenance Ticket Management (Issue #85)
# Workflow: Open -> InProgress -> Resolved -> Closed
# Also supports: Cancel, Reopen

Feature: Maintenance Ticket Management
  As a co-owner or syndic
  I want to create and manage maintenance tickets
  So that building issues are tracked and resolved efficiently

  Background:
    Given the system is initialized
    And an organization "Ticket Copro ASBL" exists with id "org-ticket"
    And a building "Residence Maintenance" exists in organization "org-ticket"
    And an owner "Marie Proprietaire" exists in building "Residence Maintenance"
    And an owner "Pierre Contractor" exists in building "Residence Maintenance"
    And the user is authenticated as owner "Marie Proprietaire"

  # === CREATION ===

  Scenario: Owner creates a plumbing ticket with high priority
    When I create a ticket:
      | title       | Leaking pipe in bathroom        |
      | description | Water dripping from ceiling in unit 3B |
      | category    | Plumbing                        |
      | priority    | High                            |
    Then the ticket should be created successfully
    And the ticket status should be "Open"
    And the ticket category should be "Plumbing"
    And the ticket priority should be "High"

  Scenario: Owner creates an electrical ticket with low priority
    When I create a ticket:
      | title       | Flickering light in hallway     |
      | description | The hallway light on floor 2 flickers intermittently |
      | category    | Electrical                      |
      | priority    | Low                             |
    Then the ticket should be created successfully
    And the ticket status should be "Open"
    And the ticket category should be "Electrical"

  Scenario: Ticket creation fails with empty title
    When I create a ticket:
      | title       |                                 |
      | description | Some description                |
      | category    | Other                           |
      | priority    | Medium                          |
    Then the ticket creation should fail
    And the error should contain "title"

  Scenario: Ticket creation fails with empty description
    When I create a ticket:
      | title       | Valid title here                 |
      | description |                                 |
      | category    | Other                           |
      | priority    | Medium                          |
    Then the ticket creation should fail
    And the error should contain "description"

  # === ASSIGNMENT & WORKFLOW ===

  Scenario: Assign ticket to a contractor
    Given an open ticket "Broken elevator door" exists
    When I assign the ticket to "Pierre Contractor"
    Then the ticket status should be "InProgress"
    And the ticket should be assigned to "Pierre Contractor"

  Scenario: Contractor starts work on assigned ticket
    Given an open ticket "Heating issue" exists
    When the contractor starts work on the ticket
    Then the ticket status should be "InProgress"

  Scenario: Resolve ticket with resolution notes
    Given an in-progress ticket "Plumbing fix" exists
    When I resolve the ticket with notes "Replaced the leaking pipe and tested water pressure"
    Then the ticket status should be "Resolved"
    And the resolution notes should contain "Replaced the leaking pipe"

  Scenario: Close a resolved ticket
    Given a resolved ticket "Completed repair" exists
    When I close the ticket
    Then the ticket status should be "Closed"

  # === CANCEL & REOPEN ===

  Scenario: Cancel an open ticket
    Given an open ticket "False alarm" exists
    When I cancel the ticket with reason "Issue resolved by itself"
    Then the ticket status should be "Cancelled"

  Scenario: Reopen a resolved ticket
    Given a resolved ticket "Recurring issue" exists
    When I reopen the ticket with reason "Problem reappeared after 2 days"
    Then the ticket status should be "Open"

  Scenario: Reopen a closed ticket
    Given a closed ticket "Old issue" exists
    When I reopen the ticket with reason "Issue was not properly fixed"
    Then the ticket status should be "Open"

  # === INVALID STATE TRANSITIONS ===

  Scenario: Cannot close an open ticket directly
    Given an open ticket "Not yet resolved" exists
    When I try to close the ticket
    Then the operation should fail
    And the error should contain "Resolved"

  Scenario: Cannot resolve a cancelled ticket
    Given a cancelled ticket "Already cancelled" exists
    When I try to resolve the ticket
    Then the operation should fail

  # === LISTING & FILTERING ===

  Scenario: List tickets by building
    Given an open ticket "Ticket A" exists
    And an open ticket "Ticket B" exists
    When I list tickets for the building
    Then I should get at least 2 tickets

  Scenario: List tickets by status
    Given an open ticket "Open ticket" exists
    And a resolved ticket "Resolved ticket" exists
    When I list tickets with status "Open"
    Then all returned tickets should have status "Open"

  Scenario: List my created tickets
    Given an open ticket "My ticket" exists
    When I list my tickets
    Then I should get at least 1 ticket

  # === STATISTICS & OVERDUE ===

  Scenario: Get ticket statistics for building
    Given an open ticket "Stat ticket 1" exists
    And a resolved ticket "Stat ticket 2" exists
    When I get ticket statistics for the building
    Then the statistics should show at least 1 open ticket
    And the statistics should show at least 1 resolved ticket
