Feature: Pagination and Filtering
  As a user
  I want to paginate and sort lists
  So that I can browse data efficiently

  Scenario: List buildings paginated
    Given a coproperty management system
    When I list buildings page 1 with per_page 10 sorted by created_at desc
    Then I should get at least 1 building

