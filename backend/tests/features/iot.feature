# Feature: IoT Smart Meters & Linky (Issue #133)
# Device types: Linky, Ores, SmartMeter, OtherIoT
# Metric types: electricity_kwh, gas_m3, water_m3, temperature_c

Feature: IoT Smart Meters & Linky Integration
  As a syndic
  I want to collect and analyze smart meter data
  So that I can monitor building energy consumption

  Background:
    Given the system is initialized
    And an organization "IoT Copro ASBL" exists with id "org-iot"
    And a building "Residence Connectee" exists in organization "org-iot"

  # === READINGS ===

  Scenario: Create a single IoT reading
    When I create an IoT reading:
      | device_type | Linky             |
      | metric_type | electricity_kwh   |
      | value       | 42.5              |
      | unit        | kWh               |
      | source      | automatic         |
    Then the reading should be created

  Scenario: Bulk create IoT readings
    When I create 10 IoT readings in bulk
    Then all 10 readings should be created

  Scenario: Query readings with filters
    Given 20 readings exist for the building
    When I query readings with:
      | device_type | Linky             |
      | metric_type | electricity_kwh   |
      | limit       | 5                 |
    Then I should get 5 readings

  Scenario: Query readings with date range
    Given readings exist from January to March 2026
    When I query readings from February 1 to February 28
    Then I should only get February readings

  # === CONSUMPTION ANALYTICS ===

  Scenario: Get consumption statistics for building
    Given 100 electricity readings exist
    When I get consumption stats for the building
    Then the stats should include total, average, min, max values

  Scenario: Get daily consumption aggregates
    Given daily readings exist for the past 30 days
    When I get daily aggregates
    Then I should get 30 daily data points

  Scenario: Get monthly consumption aggregates
    Given readings exist for the past 6 months
    When I get monthly aggregates
    Then I should get aggregated monthly totals

  Scenario: Detect consumption anomalies
    Given normal readings with one spike exist
    When I check for consumption anomalies
    Then the spike should be flagged as anomalous

  # === LINKY DEVICE MANAGEMENT ===

  Scenario: Configure a Linky device
    When I configure a Linky device:
      | prm                | 12345678901234        |
      | provider           | Enedis                |
      | authorization_code | AUTH-CODE-XYZ         |
    Then the Linky device should be configured
    And sync should be enabled by default

  Scenario: Get Linky device for building
    Given a Linky device is configured
    When I get the Linky device for the building
    Then I should receive the device configuration

  Scenario: Toggle Linky sync on/off
    Given a Linky device with sync enabled
    When I toggle sync off
    Then sync should be disabled

  Scenario: Find devices needing sync
    Given devices that haven't synced in 24 hours
    When I find devices needing sync
    Then the stale devices should be returned
