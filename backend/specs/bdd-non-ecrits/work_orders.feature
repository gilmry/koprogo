# Feature: Work Order on Ticket (Issue #309)
# Extension to Ticket entity: send_work_order_to_contractor method
# Sends magic link PWA to assigned contractor for InProgress tickets

Feature: Work Order Management
  As a syndic
  I want to send work orders to contractors
  So that maintenance work can be tracked from ticket to completion

  Background:
    Given the system is initialized
    And an organization "WorkOrder Copro ASBL" exists with id "org-workorder"
    And a building "Residence Work Orders" exists in organization "org-workorder"
    And an owner "Marie Proprietaire" exists in building "Residence Work Orders"
    And a contractor "Pierre Artisan" exists in organization "org-workorder"
    And the user is authenticated as syndic "Jean Syndic"

  # === SUCCESSFUL WORK ORDER ===

  Scenario: Successfully send work order for assigned InProgress ticket
    Given a ticket exists:
      | title       | Leaking pipe in bathroom             |
      | description | Water dripping from ceiling in unit 3B |
      | category    | Plumbing                             |
      | priority    | High                                 |
    And the ticket is assigned to contractor "Pierre Artisan"
    And the ticket status is "InProgress"
    When I send a work order for the ticket
    Then the work order should be sent successfully
    And the ticket should have a work_order_sent_at timestamp
    And the ticket updated_at should be refreshed

  Scenario: Send work order after explicit start_work transition
    Given a ticket exists with status "Open"
    And the ticket is assigned to contractor "Pierre Artisan"
    When I send a work order for the ticket
    Then the work order should be sent successfully
    And the ticket should have a work_order_sent_at timestamp

  # === STATUS VALIDATION ===

  Scenario: Reject work order for Open ticket without assignment
    Given a ticket exists with status "Open"
    And the ticket has no assigned contractor
    When I send a work order for the ticket
    Then the work order should fail
    And the error should contain "InProgress status"

  Scenario: Reject work order for Resolved ticket
    Given a ticket exists with status "Open"
    And the ticket is assigned to contractor "Pierre Artisan"
    And the ticket is resolved with notes "Fixed the pipe"
    When I send a work order for the ticket
    Then the work order should fail
    And the error should contain "InProgress status"

  Scenario: Reject work order for Closed ticket
    Given a ticket exists with status "Open"
    And the ticket is assigned to contractor "Pierre Artisan"
    And the ticket is resolved with notes "Fixed"
    And the ticket is closed
    When I send a work order for the ticket
    Then the work order should fail
    And the error should contain "InProgress status"

  Scenario: Reject work order for Cancelled ticket
    Given a ticket exists with status "Open"
    And the ticket is cancelled with reason "Duplicate report"
    When I send a work order for the ticket
    Then the work order should fail
    And the error should contain "InProgress status"

  # === ASSIGNMENT VALIDATION ===

  Scenario: Reject work order for InProgress ticket without contractor assignment
    Given a ticket exists with status "Open"
    And the ticket status is manually set to "InProgress"
    And the ticket has no assigned contractor
    When I send a work order for the ticket
    Then the work order should fail
    And the error should contain "must be assigned to a contractor"

  # === TIMESTAMP TRACKING ===

  Scenario: Work order timestamp is initially null on new ticket
    Given a ticket exists:
      | title       | New maintenance request |
      | description | Light bulb replacement  |
      | category    | Electrical             |
      | priority    | Low                    |
    Then the ticket work_order_sent_at should be null

  Scenario: Sending work order twice updates the timestamp
    Given a ticket exists with status "Open"
    And the ticket is assigned to contractor "Pierre Artisan"
    When I send a work order for the ticket
    Then the ticket should have a work_order_sent_at timestamp
    When I send a work order for the ticket again
    Then the work order should be sent successfully
    And the ticket work_order_sent_at should be updated

  # === INTEGRATION WITH TICKET WORKFLOW ===

  Scenario: Full workflow from ticket creation to work order
    Given a ticket exists:
      | title       | Broken elevator door     |
      | description | Door does not close properly |
      | category    | Elevator                 |
      | priority    | Critical                 |
    And the ticket status should be "Open"
    When the ticket is assigned to contractor "Pierre Artisan"
    Then the ticket status should be "InProgress"
    When I send a work order for the ticket
    Then the work order should be sent successfully
    And the ticket should have a work_order_sent_at timestamp
    When the ticket is resolved with notes "Replaced door mechanism"
    Then the ticket status should be "Resolved"
