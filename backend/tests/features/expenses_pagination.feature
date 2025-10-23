Feature: Expenses pagination
  As an accountant
  I want to paginate expenses
  So that I can browse expenses efficiently

  Scenario: List expenses paginated
    Given a coproperty management system
    And I create an expense of amount 50.0
    When I list expenses page 1 with per_page 10
    Then I should get at least 1 expense

