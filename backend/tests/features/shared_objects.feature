# Feature: Object Sharing Library (Issue #49 Phase 4)
# Categories: Tools, Kitchen, Electronics, Sports, Garden, Books, Games, Other
# Statuses: Available -> Borrowed -> Returned, Unavailable

Feature: Object Sharing Library
  As a co-owner
  I want to share and borrow objects with neighbors
  So that we reduce waste and build community solidarity

  Background:
    Given the system is initialized
    And an organization "Partage Copro ASBL" exists with id "org-partage"
    And a building "Residence Partage" exists in organization "org-partage"
    And an owner "Alice Preteuse" exists in building "Residence Partage"
    And an owner "Bob Emprunteur" exists in building "Residence Partage"

  # === CREATION ===

  Scenario: Share a tool
    When "Alice Preteuse" shares an object:
      | name            | Electric drill                  |
      | description     | Makita 18V cordless drill       |
      | category        | Tools                           |
      | deposit_credits | 1                               |
      | max_loan_days   | 7                               |
    Then the shared object should be created
    And the status should be "Available"

  Scenario: Share a free object (no deposit)
    When "Alice Preteuse" shares an object:
      | name            | French novel collection         |
      | description     | 10 classic French novels        |
      | category        | Books                           |
      | deposit_credits | 0                               |
      | max_loan_days   | 14                              |
    Then the shared object should be created
    And it should be free to borrow

  # === BORROWING ===

  Scenario: Borrow an available object
    Given "Alice Preteuse" has shared an "Electric drill"
    When "Bob Emprunteur" borrows the "Electric drill"
    Then the loan should be created
    And the object status should be "Borrowed"
    And the due date should be set based on max_loan_days

  Scenario: Cannot borrow an already borrowed object
    Given the "Electric drill" is currently borrowed
    When "Bob Emprunteur" tries to borrow it
    Then the borrowing should fail

  Scenario: Return a borrowed object
    Given "Bob Emprunteur" has borrowed the "Electric drill"
    When "Bob Emprunteur" returns the object
    Then the object status should be "Available"
    And the return date should be recorded

  # === LISTING ===

  Scenario: List available objects in building
    Given 3 available and 1 borrowed objects exist
    When I list available objects
    Then I should get 3 objects

  Scenario: List borrowed objects
    Given 2 borrowed objects exist
    When I list borrowed objects
    Then I should get 2 objects

  Scenario: List my borrowed objects
    Given "Bob Emprunteur" has 2 active loans
    When "Bob Emprunteur" lists their borrowed objects
    Then they should get 2 objects

  Scenario: List overdue objects
    Given an object borrowed 10 days ago with max_loan_days of 7
    When I list overdue objects
    Then the overdue object should appear

  Scenario: List objects by category
    Given objects in Tools and Books categories exist
    When I list objects with category "Tools"
    Then all returned objects should have category "Tools"

  Scenario: List free objects
    Given paid and free objects exist
    When I list free objects
    Then all returned objects should have deposit 0

  Scenario: List owner's shared objects
    Given "Alice Preteuse" has shared 3 objects
    When I list objects owned by "Alice Preteuse"
    Then I should get 3 objects

  # === AVAILABILITY ===

  Scenario: Mark object as unavailable
    Given "Alice Preteuse" has an available object
    When "Alice Preteuse" marks it as unavailable
    Then the object should not appear in available listings

  # === STATISTICS ===

  Scenario: Get sharing statistics for building
    Given multiple shared objects and loans exist
    When I get sharing statistics
    Then the stats should include total objects count
    And the stats should include active loans count

  # === DELETE ===

  Scenario: Delete a shared object
    Given "Alice Preteuse" has a shared object that is not borrowed
    When "Alice Preteuse" deletes the object
    Then the object should be deleted
