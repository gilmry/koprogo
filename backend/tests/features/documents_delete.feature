Feature: Documents deletion
  As a property manager
  I want to delete documents
  So that obsolete files are removed

  Scenario: Delete an uploaded document
    Given a coproperty management system
    And I upload a document named "ToDelete"
    When I delete the last document
    And I try to download the last document
    Then the download should fail

