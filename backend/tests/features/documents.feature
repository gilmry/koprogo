Feature: Documents Management
  As a property manager
  I want to upload and manage documents
  So that copro files are centralized

  Scenario: Upload a document
    Given a coproperty management system
    When I upload a document named "Règlement"
    Then the document should be stored

