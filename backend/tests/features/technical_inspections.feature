# Feature: Technical Inspections (Issue #134)
# Mandatory inspection types: Elevator, Fire, Electrical, Gas, Heating, etc.

Feature: Technical Inspections
  As a syndic
  I want to track mandatory technical inspections
  So that the building stays compliant with safety regulations

  Background:
    Given the system is initialized
    And an organization "Inspect Copro ASBL" exists with id "org-inspect"
    And a building "Residence Securite" exists in organization "org-inspect"

  Scenario: Create an elevator inspection
    When I create a technical inspection:
      | inspection_type   | Elevator                |
      | inspector_name    | Bureau Veritas          |
      | inspection_date   | 2026-02-15              |
      | next_inspection   | 2026-08-15              |
      | result            | Passed                  |
    Then the inspection should be created

  Scenario: Create a fire safety inspection
    When I create a technical inspection:
      | inspection_type   | Fire                    |
      | inspector_name    | Securitas Inspection    |
      | inspection_date   | 2026-01-20              |
      | next_inspection   | 2027-01-20              |
    Then the inspection should be created

  Scenario: Get overdue inspections
    Given an inspection with next_inspection_date in the past exists
    When I list overdue inspections for the building
    Then the overdue inspection should appear

  Scenario: Get upcoming inspections
    Given an inspection with next_inspection_date in 60 days exists
    When I list upcoming inspections within 90 days
    Then the upcoming inspection should appear

  Scenario: Filter inspections by type
    Given Elevator and Fire inspections exist
    When I list inspections of type "Elevator"
    Then all returned inspections should be of type "Elevator"

  Scenario: Update an inspection
    Given an inspection exists
    When I update the inspection result to "Failed"
    Then the inspection should be updated

  Scenario: Add report to inspection
    Given an inspection exists
    When I add a report document to the inspection
    Then the report should be attached

  Scenario: Add photo to inspection
    Given an inspection exists
    When I add a photo to the inspection
    Then the photo should be attached

  Scenario: Add certificate to inspection
    Given an inspection exists
    When I add a certificate to the inspection
    Then the certificate should be attached

  Scenario: List inspections by building
    Given 4 inspections exist for the building
    When I list inspections for the building
    Then I should get 4 inspections

  Scenario: Delete an inspection
    Given an inspection exists
    When I delete the inspection
    Then the inspection should be deleted

  Scenario: List inspections with pagination
    Given 12 inspections exist
    When I list inspections page 1 with 10 per page
    Then I should get 10 inspections
