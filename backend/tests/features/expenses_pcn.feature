Feature: PCN Reporting
  As an accountant
  I want to generate PCN reports
  So that expenses are aggregated per account

  Scenario: Generate PCN report
    Given a coproperty management system
    When I generate a PCN report for the building
    Then the PCN report should be generated

