# Feature: Work Reports & Warranties (Issue #134)
# Digital Maintenance Logbook

Feature: Work Reports and Warranty Tracking
  As a syndic
  I want to maintain a digital logbook of all building works
  So that I can track warranties and maintenance history

  Background:
    Given the system is initialized
    And an organization "Travaux Copro ASBL" exists with id "org-travaux"
    And a building "Residence Entretien" exists in organization "org-travaux"

  Scenario: Create a work report
    When I create a work report:
      | work_type        | Renovation              |
      | title            | Facade renovation       |
      | description      | Complete facade cleaning and repainting |
      | contractor_name  | Entreprise Martin       |
      | start_date       | 2026-01-15              |
      | end_date         | 2026-02-28              |
      | warranty_years   | 10                      |
    Then the work report should be created

  Scenario: Update a work report
    Given a work report exists
    When I update the contractor name to "Entreprise Dupont"
    Then the work report should be updated

  Scenario: List work reports by building
    Given 3 work reports exist for the building
    When I list work reports for the building
    Then I should get 3 reports

  Scenario: Get active warranties
    Given work reports with 2-year and 10-year warranties exist
    And the 2-year warranty has not expired yet
    When I get active warranties for the building
    Then both warranties should appear

  Scenario: Get expiring warranties
    Given a warranty expiring in 60 days exists
    When I get warranties expiring within 90 days
    Then the expiring warranty should appear

  Scenario: Add photo to work report
    Given a work report exists
    When I add a photo to the work report
    Then the photo should be attached

  Scenario: Add document to work report
    Given a work report exists
    When I add a document to the work report
    Then the document should be attached

  Scenario: Delete a work report
    Given a work report exists
    When I delete the work report
    Then the report should be deleted

  Scenario: List work reports with pagination
    Given 15 work reports exist
    When I list work reports page 1 with 10 per page
    Then I should get 10 reports

  Scenario: Filter work reports by organization
    Given work reports for 2 organizations exist
    When I list work reports for our organization
    Then I should only get our organization's reports
