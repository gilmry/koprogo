# Feature: Skills Directory (Issue #49 Phase 3)
# Categories: HomeRepair, Languages, Technology, Education, Arts, Sports, etc.
# Expertise levels: Beginner, Intermediate, Advanced, Expert

Feature: Skills Directory
  As a co-owner
  I want to register and discover skills within my building
  So that neighbors can help each other with their expertise

  Background:
    Given the system is initialized
    And an organization "Skills Copro ASBL" exists with id "org-skills"
    And a building "Residence Talents" exists in organization "org-skills"
    And an owner "Marc Plombier" exists in building "Residence Talents"
    And an owner "Sophie Musicienne" exists in building "Residence Talents"

  # === CREATION ===

  Scenario: Register a plumbing skill
    When "Marc Plombier" registers a skill:
      | skill_name      | Emergency plumbing repair       |
      | description     | Fix leaks, unclog drains, replace faucets |
      | skill_category  | HomeRepair                      |
      | expertise_level | Expert                          |
      | hourly_rate     | 2                               |
      | years_experience | 15                             |
    Then the skill offer should be created
    And the category should be "HomeRepair"
    And the expertise level should be "Expert"

  Scenario: Register a free volunteer skill
    When "Sophie Musicienne" registers a skill:
      | skill_name      | Piano lessons for beginners     |
      | description     | Teaching basic piano to children and adults |
      | skill_category  | Arts                            |
      | expertise_level | Advanced                        |
      | hourly_rate     | 0                               |
    Then the skill offer should be created
    And the skill should be marked as free

  Scenario: Skill creation fails with empty name
    When I try to register a skill with empty name
    Then the creation should fail

  # === LISTING & FILTERING ===

  Scenario: List available skills in building
    Given 3 skill offers exist in the building
    When I list skills for the building
    Then I should get 3 skills

  Scenario: Filter skills by category
    Given skills in HomeRepair and Arts categories exist
    When I filter skills by category "HomeRepair"
    Then all returned skills should have category "HomeRepair"

  Scenario: Filter skills by expertise level
    Given skills with Expert and Beginner levels exist
    When I filter skills by expertise "Expert"
    Then all returned skills should have expertise "Expert"

  Scenario: List free skills only
    Given paid and free skills exist
    When I filter for free skills
    Then all returned skills should be free

  Scenario: List owner's skills
    Given "Marc Plombier" has registered 2 skills
    When I list skills for owner "Marc Plombier"
    Then I should get 2 skills

  # === AVAILABILITY ===

  Scenario: Mark skill as unavailable
    Given "Marc Plombier" has an available skill
    When "Marc Plombier" marks the skill as unavailable
    Then the skill should not be available for help

  Scenario: Mark skill as available again
    Given "Marc Plombier" has an unavailable skill
    When "Marc Plombier" marks the skill as available
    Then the skill should be available for help

  # === STATISTICS ===

  Scenario: Get skill statistics for building
    Given multiple skills exist in the building
    When I get skill statistics
    Then the stats should include total skills count
    And the stats should include category breakdown

  # === UPDATE & DELETE ===

  Scenario: Update a skill offer
    Given "Marc Plombier" has a skill offer
    When "Marc Plombier" updates the hourly rate to 3
    Then the hourly rate should be updated

  Scenario: Delete a skill offer
    Given "Marc Plombier" has a skill offer
    When "Marc Plombier" deletes the skill
    Then the skill should be deleted
