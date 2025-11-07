# Feature: Payment Recovery Workflow (Issue #83)
# Belgian legal compliance - Automated payment reminder system

Feature: Payment Recovery Workflow
  As a Syndic
  I want to automatically manage payment reminders for overdue expenses
  So that I can reduce unpaid charges by 30-50%

  Background:
    Given a test organization "TestOrg"
    And a building "Building A" in organization "TestOrg"
    And an owner "John Doe" with email "john@example.com"
    And an overdue expense of 100 EUR due 20 days ago

  Scenario: Create first reminder after 15 days overdue
    When I create a FirstReminder for the overdue expense
    Then the reminder should be created successfully
    And the reminder level should be "FirstReminder"
    And the penalty amount should be calculated at 8% annual rate
    And the days overdue should be 20
    And the delivery method should be "Email"

  Scenario: Calculate penalties correctly for Belgian legal rate (8% annual)
    Given an overdue expense of 1000 EUR due 365 days ago
    When I calculate the penalty amount
    Then the penalty should be 80 EUR

  Scenario: Prevent creating reminder before minimum days threshold
    Given an overdue expense of 100 EUR due 10 days ago
    When I attempt to create a FirstReminder
    Then the creation should fail
    And the error should mention "Cannot create first reminder before 15 days"

  Scenario: Mark reminder as sent with PDF path
    Given a pending FirstReminder
    When I mark it as sent with PDF path "/reminders/001.pdf"
    Then the reminder status should be "Sent"
    And the sent_date should be set to current timestamp
    And the pdf_path should be "/reminders/001.pdf"

  Scenario: Escalate reminder to next level
    Given a sent FirstReminder from 16 days ago
    When I escalate the reminder
    Then a new SecondReminder should be created
    And the previous reminder status should be "Escalated"
    And the new reminder should have higher penalty amount

  Scenario: Formal notice uses registered letter
    Given an overdue expense of 100 EUR due 65 days ago
    When I create a FormalNotice reminder
    Then the delivery method should be "RegisteredLetter"
    And I should be able to add a tracking number

  Scenario: Bulk create reminders for all overdue expenses
    Given 5 overdue expenses in the organization
    And minimum days overdue threshold is 15
    When I trigger bulk reminder creation
    Then 5 reminders should be created
    And each reminder should have the appropriate level based on days overdue

  Scenario: Get payment recovery statistics
    Given 3 active payment reminders in the organization
    And total owed amount is 500 EUR
    And total penalties amount is 20 EUR
    When I request recovery statistics
    Then the stats should show 500 EUR total owed
    And the stats should show 20 EUR total penalties
    And the stats should show reminder count by level

  Scenario: Find overdue expenses without reminders
    Given 3 overdue expenses without reminders
    And minimum days overdue is 15
    When I search for overdue expenses
    Then 3 expenses should be returned
    And each should have recommended reminder level

  Scenario: Cancel reminder when payment received before sending
    Given a pending FirstReminder
    When the expense is paid before reminder is sent
    And I cancel the reminder with reason "Payment received"
    Then the reminder status should be "Cancelled"
    And the notes should contain "Payment received"

  Scenario: Mark reminder as paid
    Given a sent FirstReminder
    When the owner pays the expense
    And I mark the reminder as paid
    Then the reminder status should be "Paid"

  Scenario: Owner role cannot create reminders
    Given I am logged in as "Owner"
    When I attempt to create a payment reminder
    Then the request should be forbidden
    And the error should mention "Owner role cannot create or modify"

  Scenario: Syndic can create and manage reminders
    Given I am logged in as "Syndic"
    When I create a payment reminder
    Then the reminder should be created successfully
    And an audit log should be created

  Scenario: Recalculate penalties as days increase
    Given a reminder with 20 days overdue
    When 10 more days pass
    And I recalculate the penalties
    Then the days overdue should be updated to 30
    And the penalty amount should increase accordingly
    And the total amount should be updated

  Scenario: Formal notice cannot escalate further
    Given a FormalNotice reminder
    When I attempt to escalate
    Then the escalation should succeed
    But no new reminder level should be created
    And the next step should be bailiff/huissier

  Scenario: Reminder tone matches level
    Given a FirstReminder
    Then the tone should be "aimable"
    When escalated to SecondReminder
    Then the tone should be "ferme"
    When escalated to FormalNotice
    Then the tone should be "juridique"
