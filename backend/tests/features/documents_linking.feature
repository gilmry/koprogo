Feature: Documents linking and download
  As a property manager
  I want to link documents to meetings and expenses and download them
  So that related records are connected and files retrievable

  Scenario: Link a document to a meeting and download it
    Given a coproperty management system
    When I create a meeting titled "AG Test"
    And I upload a document named "PV"
    And I link the document to the meeting
    And I download the last document
    Then the downloaded content should not be empty

