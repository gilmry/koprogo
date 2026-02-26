# Feature: Public Syndic Information (Issue #92)
# Belgian legal requirement: Syndic contact must be publicly available
# No authentication required

Feature: Public Syndic Information
  As a member of the public
  I want to access syndic contact information
  So that I can contact the building manager when needed

  Background:
    Given the system is initialized
    And an organization "Public Copro ASBL" exists with id "org-public"
    And a building "Residence Publique" with syndic info exists:
      | syndic_name     | Jean Syndic              |
      | syndic_email    | jean@syndic.be           |
      | syndic_phone    | +32 2 123 4567           |
      | syndic_address  | Rue de la Loi 1, 1000 Bruxelles |

  Scenario: Get public syndic info by building slug
    When I request syndic info for slug "residence-publique-bruxelles"
    Then I should receive the syndic contact information
    And no authentication should be required

  Scenario: Building without syndic info returns empty
    Given a building "Residence Sans Syndic" with no syndic info exists
    When I request syndic info for that building's slug
    Then the response should indicate no syndic info available

  Scenario: Slug generation from building name
    Given a building named "Résidence Château d'Eau" in "Liège"
    Then the slug should be "residence-chateau-d-eau-liege"

  Scenario: Non-existent slug returns 404
    When I request syndic info for slug "non-existent-building"
    Then I should receive a 404 response
