Feature: Documents link to expenses and access control
  As a property manager
  I want to link documents to expenses and enforce org access control
  So that documents are properly associated and isolated

  Scenario: Link a document to an expense
    Given a coproperty management system
    And I upload a document named "Invoice"
    And I create an expense of amount 123.45
    When I link the document to the expense
    Then the document should be stored

  Scenario: Documents are filtered by organization
    Given a coproperty management system with two organizations
    When I upload a document named "Doc Org A"
    And I list documents for the second organization
    Then I should get 0 documents

