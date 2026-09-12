# Feature: MCP SSE Server - Model Context Protocol for AI Integrations (Issue #252)
# Protocol: JSON-RPC 2.0 over Server-Sent Events
# Endpoints: GET /mcp/sse, POST /mcp/messages

Feature: MCP SSE Server - Model Context Protocol
  As an AI assistant integrated with KoproGo
  I want to use the MCP protocol to access property management data
  So that I can help syndics manage their co-properties via natural language

  Background:
    Given the system is initialized
    And an organization "MCP Test ASBL" exists with id "org-mcp"
    And a building "Residence MCP" exists in organization "org-mcp"
    And a syndic user "Jean MCP" exists for building "Residence MCP"
    And the user is authenticated as syndic "Jean MCP"

  # === SSE CONNECTION ===

  Scenario: Establish SSE connection and receive endpoint event
    When I connect to GET "/api/v1/mcp/sse" with Accept "text/event-stream"
    Then the response status should be 200
    And the Content-Type should be "text/event-stream"
    And I should receive an "endpoint" SSE event
    And the endpoint event data should contain "/mcp/messages?session_id="

  Scenario: SSE connection fails without authentication
    When I connect to GET "/api/v1/mcp/sse" without authentication
    Then the response status should be 401

  # === INITIALIZE ===

  Scenario: Initialize MCP session successfully
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC request to "/api/v1/mcp/messages":
      | jsonrpc | 2.0        |
      | id      | 1          |
      | method  | initialize |
    Then the response should be a JSON-RPC success response with id 1
    And the result should contain "protocolVersion" equal to "2024-11-05"
    And the result should contain "serverInfo" with name "koprogo-mcp"
    And the result should contain "capabilities" with "tools"
    And the result should contain "instructions" about Belgian co-property management

  Scenario: Initialize with invalid JSON-RPC version
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC request to "/api/v1/mcp/messages":
      | jsonrpc | 1.0        |
      | id      | 1          |
      | method  | initialize |
    Then the response should be a JSON-RPC error response
    And the error code should be -32600
    And the error message should contain "jsonrpc must be '2.0'"

  Scenario: Send initialized notification (no response expected)
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC notification to "/api/v1/mcp/messages":
      | jsonrpc | 2.0                        |
      | method  | notifications/initialized  |
    Then the response should be null (notification acknowledgment)

  # === TOOLS/LIST ===

  Scenario: List all available MCP tools
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC request to "/api/v1/mcp/messages":
      | jsonrpc | 2.0        |
      | id      | 2          |
      | method  | tools/list |
    Then the response should be a JSON-RPC success response with id 2
    And the result should contain a "tools" array
    And the tools array should include "list_buildings"
    And the tools array should include "get_building"
    And the tools array should include "list_owners"
    And the tools array should include "list_meetings"
    And the tools array should include "get_financial_summary"
    And the tools array should include "list_tickets"
    And the tools array should include "legal_search"
    And the tools array should include "majority_calculator"
    And the tools array should include "alertes_list"
    And each tool should have "name", "description" and "inputSchema" fields

  # === TOOLS/CALL ===

  Scenario: Call list_buildings tool successfully
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC tools/call request:
      | tool_name | list_buildings |
      | page      | 1              |
      | per_page  | 10             |
    Then the response should be a JSON-RPC success response
    And the result should contain "content" array with text blocks

  Scenario: Call get_building tool with valid UUID
    Given I have an active SSE connection with a session_id
    And building "Residence MCP" has a known UUID
    When I send a JSON-RPC tools/call request:
      | tool_name   | get_building                           |
      | building_id | <building_uuid>                        |
    Then the response should be a JSON-RPC success response
    And the result content should contain building details

  Scenario: Call get_building tool with non-existent UUID
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC tools/call request:
      | tool_name   | get_building                           |
      | building_id | 00000000-0000-0000-0000-000000000000   |
    Then the response should be a JSON-RPC error response
    And the error message should contain "Building not found"

  Scenario: Call get_building tool with invalid UUID format
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC tools/call request:
      | tool_name   | get_building     |
      | building_id | not-a-valid-uuid |
    Then the response should be a JSON-RPC error response
    And the error code should be -32602
    And the error message should contain "building_id must be a valid UUID"

  Scenario: Call get_building tool without required building_id
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC tools/call request:
      | tool_name | get_building |
    Then the response should be a JSON-RPC error response
    And the error code should be -32602
    And the error message should contain "building_id is required"

  Scenario: Call legal_search tool with keyword "quorum"
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC tools/call request:
      | tool_name | legal_search |
      | query     | quorum       |
    Then the response should be a JSON-RPC success response
    And the result content should contain legal rules about quorum
    And each result should have "code", "article", "title" and "content"

  Scenario: Call majority_calculator tool for ordinary decision
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC tools/call request:
      | tool_name     | majority_calculator |
      | decision_type | ordinary            |
    Then the response should be a JSON-RPC success response
    And the result should indicate "Simple" majority
    And the threshold should be "50%+1 des votes exprimes"
    And the article should reference "Art. 3.88"

  Scenario: Call majority_calculator for heavy works
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC tools/call request:
      | tool_name     | majority_calculator |
      | decision_type | works_heavy         |
    Then the response should be a JSON-RPC success response
    And the result should indicate "Two-thirds" majority
    And the percentage should be 66.7

  Scenario: Call alertes_list tool for compliance alerts
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC tools/call request:
      | tool_name | alertes_list |
    Then the response should be a JSON-RPC success response
    And the result should contain "alert_count" and "alerts" array
    And alerts should include a "LEGAL_REMINDER" type alert

  Scenario: Call non-existent tool returns METHOD_NOT_FOUND
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC tools/call request:
      | tool_name | nonexistent_tool |
    Then the response should be a JSON-RPC error response
    And the error code should be -32601
    And the error message should contain "Unknown tool: nonexistent_tool"
    And the error data should list available tools

  Scenario: Call tools/call without params returns error
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC request to "/api/v1/mcp/messages":
      | jsonrpc | 2.0        |
      | id      | 10         |
      | method  | tools/call |
    Then the response should be a JSON-RPC error response
    And the error code should be -32602
    And the error message should contain "params are required for tools/call"

  Scenario: Call tools/call without tool name returns error
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC request to "/api/v1/mcp/messages":
      | jsonrpc | 2.0                   |
      | id      | 11                    |
      | method  | tools/call            |
      | params  | {"arguments": {}}     |
    Then the response should be a JSON-RPC error response
    And the error code should be -32602
    And the error message should contain "params.name is required"

  # === UNKNOWN METHOD ===

  Scenario: Send unknown JSON-RPC method
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC request to "/api/v1/mcp/messages":
      | jsonrpc | 2.0             |
      | id      | 99              |
      | method  | unknown/method  |
    Then the response should be a JSON-RPC error response
    And the error code should be -32601
    And the error message should contain "Method not found: unknown/method"

  # === PING ===

  Scenario: Send ping request
    Given I have an active SSE connection with a session_id
    When I send a JSON-RPC request to "/api/v1/mcp/messages":
      | jsonrpc | 2.0  |
      | id      | 50   |
      | method  | ping |
    Then the response should be a JSON-RPC success response with id 50
    And the result should be an empty object

  # === USER WITHOUT ORGANIZATION ===

  Scenario: Tool call fails for user without organization
    Given I am authenticated as a user without an organization
    And I have an active SSE connection with a session_id
    When I send a JSON-RPC tools/call request:
      | tool_name | list_buildings |
    Then the response should be a JSON-RPC error response
    And the error code should be -32600
    And the error message should contain "User does not belong to an organization"
