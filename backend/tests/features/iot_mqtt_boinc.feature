Feature: IoT MQTT Home Assistant + BOINC Grid Computing
  In order to integrate real-time energy sensor data and distributed computing
  As a syndic or system administrator
  I want to receive MQTT readings from Home Assistant and submit BOINC optimisation tasks

  # ─────────────────────────────────────────────────────────────────────────────
  # MQTT Topic Parsing
  # ─────────────────────────────────────────────────────────────────────────────

  Scenario: Parse valid MQTT topic
    Given a MQTT topic "koprogo/550e8400-e29b-41d4-a716-446655440000/energy/aaaa0000-bbbb-cccc-dddd-eeeeeeeeeeee/electricity"
    When I parse the topic
    Then the copropriete_id is "550e8400-e29b-41d4-a716-446655440000"
    And the unit_id is "aaaa0000-bbbb-cccc-dddd-eeeeeeeeeeee"

  Scenario: Reject MQTT topic with wrong prefix
    Given a MQTT topic "other/abc/energy/def/electricity"
    When I parse the topic
    Then topic parsing fails with "InvalidTopic"

  Scenario: Reject MQTT topic with missing energy segment
    Given a MQTT topic "koprogo/abc/readings/def/power"
    When I parse the topic
    Then topic parsing fails with "InvalidTopic"

  Scenario: Reject MQTT topic with invalid UUIDs
    Given a MQTT topic "koprogo/not-a-uuid/energy/also-bad/electricity"
    When I parse the topic
    Then topic parsing fails with "InvalidTopic"

  # ─────────────────────────────────────────────────────────────────────────────
  # MQTT Incoming Reading
  # ─────────────────────────────────────────────────────────────────────────────

  Scenario: Process valid MQTT electricity reading
    Given a valid organization exists
    And a building exists in the organization
    When a MQTT message arrives on topic "koprogo/{building_id}/energy/{unit_id}/electricity"
    And the payload contains value 12.47 unit "kWh" metric "electricity_consumption" source "mqtt_home_assistant"
    Then the IoT reading is persisted in the database
    And the reading has value 12.47 and unit "kWh"
    And the reading source is "mqtt_home_assistant"

  Scenario: Reject MQTT reading with negative electricity value
    Given a valid organization and building exist
    When a MQTT message arrives with value -5.0 and unit "kWh" for metric "electricity_consumption"
    Then the reading is rejected with domain error "cannot be negative"
    And no reading is persisted

  Scenario: Reject MQTT reading with future timestamp
    Given a valid organization and building exist
    When a MQTT message arrives with a timestamp 1 hour in the future
    Then the reading is rejected with domain error "cannot be in the future"

  Scenario: Reject MQTT reading with invalid unit for metric
    Given a valid organization and building exist
    When a MQTT message arrives with value 100.0, unit "gallons", metric "electricity_consumption"
    Then the reading is rejected with domain error "Invalid unit"

  # ─────────────────────────────────────────────────────────────────────────────
  # BOINC Consent Management (GDPR)
  # ─────────────────────────────────────────────────────────────────────────────

  Scenario: Grant BOINC consent for a building owner
    Given a building owner exists
    When the owner grants BOINC consent from IP "192.168.1.100"
    Then the consent is stored with granted = true
    And granted_at is set
    And consent_ip is "192.168.1.100"
    And consent_version is "v1.0"

  Scenario: Revoke BOINC consent (GDPR Art. 7.3)
    Given a building owner has previously granted BOINC consent
    When the owner revokes their consent
    Then the consent is updated with granted = false
    And revoked_at is set
    And check_consent returns false for this owner

  Scenario: Check consent returns false when no record exists
    Given a building owner without any BOINC consent record
    When I check their BOINC consent
    Then check_consent returns false

  Scenario: Re-grant consent after revocation
    Given a building owner who previously revoked consent
    When the owner grants consent again
    Then granted = true
    And revoked_at is NULL

  # ─────────────────────────────────────────────────────────────────────────────
  # BOINC Task Submission
  # ─────────────────────────────────────────────────────────────────────────────

  Scenario: Submit optimisation task with valid consent
    Given a building owner has granted BOINC consent
    And IoT readings exist for their building
    When the owner submits an energy optimisation task for 12 months
    Then the task is stored in grid_tasks with status "queued"
    And a task_id is returned

  Scenario: Reject task submission without consent
    Given a building owner has NOT granted BOINC consent
    When they attempt to submit an energy optimisation task
    Then the submission fails with error containing "not consented"
    And no task is created

  Scenario: Cancel a queued BOINC task
    Given a BOINC task exists with status "queued"
    When I cancel the task
    Then the task status is updated to "cancelled"

  Scenario: Poll status of a completed BOINC task
    Given a BOINC task exists with status "completed" and a result JSON
    When I poll the task status
    Then the response contains status "Completed"
    And result_json is not empty

  Scenario: Poll status of a non-existent task
    Given a task_id that does not exist in grid_tasks
    When I poll the task status
    Then the poll fails with "TaskNotFound"

  # ─────────────────────────────────────────────────────────────────────────────
  # REST API Endpoints
  # ─────────────────────────────────────────────────────────────────────────────

  Scenario: GET /iot/mqtt/status returns running state
    Given the MQTT listener is stopped
    When I call GET /api/v1/iot/mqtt/status
    Then the response is 200 OK
    And the response contains "running": false

  Scenario: POST /iot/mqtt/start starts the listener
    Given the MQTT listener is not running
    When I call POST /api/v1/iot/mqtt/start
    Then the response is 200 OK
    And the response contains "status": "started"

  Scenario: POST /iot/mqtt/start returns error if already running
    Given the MQTT listener is already running
    When I call POST /api/v1/iot/mqtt/start again
    Then the response is 400 Bad Request
    And the response contains "Already running"

  Scenario: POST /iot/grid/consent grants consent via REST
    Given an authenticated user with a valid owner_id
    When I POST /api/v1/iot/grid/consent with granted=true
    Then the response is 200 OK
    And the response contains owner_id and granted=true

  Scenario: POST /iot/grid/tasks requires prior consent
    Given an owner without BOINC consent
    When I POST /api/v1/iot/grid/tasks
    Then the response is 403 Forbidden
    And the response contains "not consented"
