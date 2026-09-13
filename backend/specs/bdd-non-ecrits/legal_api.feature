# Feature: Legal API - Belgian Law Reference (Issue #277)
# Static legal index embedded in binary
# Endpoints: GET /legal/rules, GET /legal/rules/:code, GET /legal/ag-sequence, GET /legal/majority-for/:decision_type

Feature: Legal API - Belgian Law Reference
  As a syndic or board member
  I want to query Belgian property law articles
  So that I can ensure legal compliance in co-property management

  Background:
    Given the system is initialized
    And the legal index is loaded with Belgian property law rules

  # === LIST / FILTER RULES ===

  Scenario: List all legal rules without filters
    When I request GET "/api/v1/legal/rules"
    Then the response status should be 200
    And the response should be an array of legal rules
    And each rule should have fields "code", "article", "title", "content", "keywords", "roles", "category"

  Scenario: Filter rules by role "syndic"
    When I request GET "/api/v1/legal/rules?role=syndic"
    Then the response status should be 200
    And every returned rule should have "syndic" in its "roles" array

  Scenario: Filter rules by role "coproprietaire"
    When I request GET "/api/v1/legal/rules?role=coproprietaire"
    Then the response status should be 200
    And every returned rule should have "coproprietaire" in its "roles" array

  Scenario: Filter rules by category "assemblee-generale"
    When I request GET "/api/v1/legal/rules?category=assemblee-generale"
    Then the response status should be 200
    And every returned rule should have category "assemblee-generale"
    And the results should include AG-related rules

  Scenario: Filter rules by category "travaux"
    When I request GET "/api/v1/legal/rules?category=travaux"
    Then the response status should be 200
    And every returned rule should have category "travaux"

  Scenario: Filter rules by both role and category
    When I request GET "/api/v1/legal/rules?role=syndic&category=travaux"
    Then the response status should be 200
    And every returned rule should have "syndic" in its "roles" array
    And every returned rule should have category "travaux"

  Scenario: Filter by non-existent role returns empty array
    When I request GET "/api/v1/legal/rules?role=nonexistent_role"
    Then the response status should be 200
    And the response should be an empty array

  Scenario: Filter by non-existent category returns empty array
    When I request GET "/api/v1/legal/rules?category=nonexistent_category"
    Then the response status should be 200
    And the response should be an empty array

  # === GET RULE BY CODE ===

  Scenario: Get a specific rule by code AG01
    When I request GET "/api/v1/legal/rules/AG01"
    Then the response status should be 200
    And the rule code should be "AG01"
    And the rule should have an article reference
    And the rule should have a title and content

  Scenario: Get a specific rule by code T01
    When I request GET "/api/v1/legal/rules/T01"
    Then the response status should be 200
    And the rule code should be "T01"
    And the rule category should be "travaux"

  Scenario: Get rule with non-existent code returns 404
    When I request GET "/api/v1/legal/rules/NONEXISTENT99"
    Then the response status should be 404
    And the error message should contain "Legal rule not found: NONEXISTENT99"

  Scenario: Get rule with empty code returns 404
    When I request GET "/api/v1/legal/rules/"
    Then the response status should be 404

  # === AG SEQUENCE ===

  Scenario: Get mandatory AG agenda sequence
    When I request GET "/api/v1/legal/ag-sequence"
    Then the response status should be 200
    And the response should be an array of AG sequence steps
    And each step should have fields "step", "point_odj", "mandatory"
    And the first step should be about opening and bureau constitution
    And the steps should be ordered by step number

  Scenario: AG sequence includes quorum verification step
    When I request GET "/api/v1/legal/ag-sequence"
    Then the response status should be 200
    And one step should mention quorum verification

  # === MAJORITY TYPES ===

  Scenario: Get majority rules for ordinary decisions
    When I request GET "/api/v1/legal/majority-for/ordinary"
    Then the response status should be 200
    And the decision_type should be "ordinary"
    And the response should include a threshold description
    And the response should reference an article of Belgian Civil Code

  Scenario: Get majority rules for qualified two-thirds decisions
    When I request GET "/api/v1/legal/majority-for/qualified_two_thirds"
    Then the response status should be 200
    And the decision_type should be "qualified_two_thirds"
    And the percentage should be approximately 66.67
    And the response should include examples of such decisions

  Scenario: Get majority rules for qualified four-fifths decisions
    When I request GET "/api/v1/legal/majority-for/qualified_four_fifths"
    Then the response status should be 200
    And the decision_type should be "qualified_four_fifths"
    And the percentage should be approximately 80.0

  Scenario: Get majority rules for unanimity
    When I request GET "/api/v1/legal/majority-for/unanimity"
    Then the response status should be 200
    And the decision_type should be "unanimity"
    And the percentage should be 100.0

  Scenario: Get majority for non-existent decision type returns 404
    When I request GET "/api/v1/legal/majority-for/nonexistent_type"
    Then the response status should be 404
    And the error message should contain "Decision type not found: nonexistent_type"
    And the response should list valid decision types

  Scenario: Get majority for proxy limit
    When I request GET "/api/v1/legal/majority-for/proxy_limit"
    Then the response status should be 200
    And the decision_type should be "proxy_limit"
