use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// IoT sensor reading entity
/// Stores time-series data from various IoT devices (Linky smart meters, temperature sensors, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IoTReading {
    pub id: Uuid,
    pub building_id: Uuid,
    pub device_type: DeviceType,
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub source: String, // e.g., "linky_ores", "linky_enedis", "netatmo"
    pub metadata: Option<serde_json::Value>, // Additional context (granularity, quality, etc.)
    pub created_at: DateTime<Utc>,
}

impl IoTReading {
    /// Create a new IoT reading with validation
    pub fn new(
        building_id: Uuid,
        device_type: DeviceType,
        metric_type: MetricType,
        value: f64,
        unit: String,
        timestamp: DateTime<Utc>,
        source: String,
    ) -> Result<Self, String> {
        // Validate value based on metric type
        Self::validate_value(&metric_type, value)?;

        // Validate unit matches metric type
        Self::validate_unit(&metric_type, &unit)?;

        // Validate source is non-empty
        if source.trim().is_empty() {
            return Err("Source cannot be empty".to_string());
        }

        // Validate timestamp is not in future
        if timestamp > Utc::now() {
            return Err("Timestamp cannot be in the future".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            building_id,
            device_type,
            metric_type,
            value,
            unit,
            timestamp,
            source,
            metadata: None,
            created_at: Utc::now(),
        })
    }

    /// Set metadata (optional context)
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Validate value based on metric type
    fn validate_value(metric_type: &MetricType, value: f64) -> Result<(), String> {
        match metric_type {
            MetricType::ElectricityConsumption | MetricType::WaterConsumption | MetricType::GasConsumption => {
                if value < 0.0 {
                    return Err(format!("Consumption value cannot be negative: {}", value));
                }
                if value > 1_000_000.0 {
                    return Err(format!("Consumption value too large (max 1M): {}", value));
                }
            }
            MetricType::Temperature => {
                if value < -40.0 || value > 80.0 {
                    return Err(format!("Temperature value out of range (-40 to +80°C): {}", value));
                }
            }
            MetricType::Humidity => {
                if value < 0.0 || value > 100.0 {
                    return Err(format!("Humidity value out of range (0-100%): {}", value));
                }
            }
            MetricType::Power => {
                if value < 0.0 {
                    return Err(format!("Power value cannot be negative: {}", value));
                }
                if value > 100_000.0 {
                    return Err(format!("Power value too large (max 100kW): {}", value));
                }
            }
            MetricType::Voltage => {
                if value < 0.0 || value > 500.0 {
                    return Err(format!("Voltage value out of range (0-500V): {}", value));
                }
            }
        }
        Ok(())
    }

    /// Validate unit matches metric type
    fn validate_unit(metric_type: &MetricType, unit: &str) -> Result<(), String> {
        let valid_units = match metric_type {
            MetricType::ElectricityConsumption => vec!["kWh", "Wh", "MWh"],
            MetricType::WaterConsumption => vec!["m3", "L", "gal"],
            MetricType::GasConsumption => vec!["m3", "kWh"],
            MetricType::Temperature => vec!["C", "°C", "F", "°F", "K"],
            MetricType::Humidity => vec!["%", "percent"],
            MetricType::Power => vec!["W", "kW", "MW"],
            MetricType::Voltage => vec!["V", "kV"],
        };

        if !valid_units.contains(&unit) {
            return Err(format!(
                "Invalid unit '{}' for metric type '{:?}'. Valid units: {:?}",
                unit, metric_type, valid_units
            ));
        }

        Ok(())
    }

    /// Check if reading is anomalous compared to average
    /// Returns true if value exceeds average by more than threshold percentage
    pub fn is_anomalous(&self, average_value: f64, threshold_percentage: f64) -> bool {
        if average_value == 0.0 {
            return false;
        }

        let deviation_percentage = ((self.value - average_value) / average_value).abs() * 100.0;
        deviation_percentage > threshold_percentage
    }

    /// Convert value to standard unit (kWh for electricity, m3 for water/gas, °C for temperature)
    pub fn normalized_value(&self) -> f64 {
        match &self.metric_type {
            MetricType::ElectricityConsumption => match self.unit.as_str() {
                "Wh" => self.value / 1000.0,
                "MWh" => self.value * 1000.0,
                _ => self.value, // Already kWh
            },
            MetricType::Temperature => match self.unit.as_str() {
                "F" | "°F" => (self.value - 32.0) * 5.0 / 9.0, // Convert to °C
                "K" => self.value - 273.15, // Convert to °C
                _ => self.value, // Already °C
            },
            _ => self.value, // No conversion needed
        }
    }
}

/// Type of IoT device
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    ElectricityMeter,  // Linky smart meter
    WaterMeter,        // Water consumption meter
    GasMeter,          // Gas consumption meter
    TemperatureSensor, // Temperature sensor (Netatmo, etc.)
    HumiditySensor,    // Humidity sensor
    PowerMeter,        // Real-time power meter
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::ElectricityMeter => write!(f, "ElectricityMeter"),
            DeviceType::WaterMeter => write!(f, "WaterMeter"),
            DeviceType::GasMeter => write!(f, "GasMeter"),
            DeviceType::TemperatureSensor => write!(f, "TemperatureSensor"),
            DeviceType::HumiditySensor => write!(f, "HumiditySensor"),
            DeviceType::PowerMeter => write!(f, "PowerMeter"),
        }
    }
}

impl std::str::FromStr for DeviceType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ElectricityMeter" => Ok(DeviceType::ElectricityMeter),
            "WaterMeter" => Ok(DeviceType::WaterMeter),
            "GasMeter" => Ok(DeviceType::GasMeter),
            "TemperatureSensor" => Ok(DeviceType::TemperatureSensor),
            "HumiditySensor" => Ok(DeviceType::HumiditySensor),
            "PowerMeter" => Ok(DeviceType::PowerMeter),
            _ => Err(format!("Invalid DeviceType: {}", s)),
        }
    }
}

/// Type of metric being measured
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    ElectricityConsumption, // kWh
    WaterConsumption,       // m3
    GasConsumption,         // m3
    Temperature,            // °C
    Humidity,               // %
    Power,                  // W (real-time power draw)
    Voltage,                // V
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::ElectricityConsumption => write!(f, "ElectricityConsumption"),
            MetricType::WaterConsumption => write!(f, "WaterConsumption"),
            MetricType::GasConsumption => write!(f, "GasConsumption"),
            MetricType::Temperature => write!(f, "Temperature"),
            MetricType::Humidity => write!(f, "Humidity"),
            MetricType::Power => write!(f, "Power"),
            MetricType::Voltage => write!(f, "Voltage"),
        }
    }
}

impl std::str::FromStr for MetricType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ElectricityConsumption" => Ok(MetricType::ElectricityConsumption),
            "WaterConsumption" => Ok(MetricType::WaterConsumption),
            "GasConsumption" => Ok(MetricType::GasConsumption),
            "Temperature" => Ok(MetricType::Temperature),
            "Humidity" => Ok(MetricType::Humidity),
            "Power" => Ok(MetricType::Power),
            "Voltage" => Ok(MetricType::Voltage),
            _ => Err(format!("Invalid MetricType: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_building_id() -> Uuid {
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
    }

    #[test]
    fn test_create_iot_reading_success() {
        let reading = IoTReading::new(
            sample_building_id(),
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            123.45,
            "kWh".to_string(),
            Utc::now() - chrono::Duration::hours(1),
            "linky_ores".to_string(),
        );

        assert!(reading.is_ok());
        let r = reading.unwrap();
        assert_eq!(r.building_id, sample_building_id());
        assert_eq!(r.device_type, DeviceType::ElectricityMeter);
        assert_eq!(r.metric_type, MetricType::ElectricityConsumption);
        assert_eq!(r.value, 123.45);
        assert_eq!(r.unit, "kWh");
        assert_eq!(r.source, "linky_ores");
    }

    #[test]
    fn test_validate_electricity_consumption_negative() {
        let reading = IoTReading::new(
            sample_building_id(),
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            -10.0,
            "kWh".to_string(),
            Utc::now(),
            "linky_ores".to_string(),
        );

        assert!(reading.is_err());
        assert!(reading.unwrap_err().contains("cannot be negative"));
    }

    #[test]
    fn test_validate_temperature_out_of_range() {
        let reading = IoTReading::new(
            sample_building_id(),
            DeviceType::TemperatureSensor,
            MetricType::Temperature,
            -50.0,
            "°C".to_string(),
            Utc::now(),
            "netatmo".to_string(),
        );

        assert!(reading.is_err());
        assert!(reading.unwrap_err().contains("out of range"));

        let reading2 = IoTReading::new(
            sample_building_id(),
            DeviceType::TemperatureSensor,
            MetricType::Temperature,
            100.0,
            "°C".to_string(),
            Utc::now(),
            "netatmo".to_string(),
        );

        assert!(reading2.is_err());
    }

    #[test]
    fn test_validate_humidity_range() {
        let reading = IoTReading::new(
            sample_building_id(),
            DeviceType::HumiditySensor,
            MetricType::Humidity,
            50.0,
            "%".to_string(),
            Utc::now(),
            "netatmo".to_string(),
        );

        assert!(reading.is_ok());

        let reading2 = IoTReading::new(
            sample_building_id(),
            DeviceType::HumiditySensor,
            MetricType::Humidity,
            150.0,
            "%".to_string(),
            Utc::now(),
            "netatmo".to_string(),
        );

        assert!(reading2.is_err());
    }

    #[test]
    fn test_validate_invalid_unit() {
        let reading = IoTReading::new(
            sample_building_id(),
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            100.0,
            "gallons".to_string(), // Invalid unit for electricity
            Utc::now(),
            "linky_ores".to_string(),
        );

        assert!(reading.is_err());
        assert!(reading.unwrap_err().contains("Invalid unit"));
    }

    #[test]
    fn test_validate_future_timestamp() {
        let reading = IoTReading::new(
            sample_building_id(),
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            100.0,
            "kWh".to_string(),
            Utc::now() + chrono::Duration::hours(1),
            "linky_ores".to_string(),
        );

        assert!(reading.is_err());
        assert!(reading.unwrap_err().contains("cannot be in the future"));
    }

    #[test]
    fn test_validate_empty_source() {
        let reading = IoTReading::new(
            sample_building_id(),
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            100.0,
            "kWh".to_string(),
            Utc::now(),
            "".to_string(),
        );

        assert!(reading.is_err());
        assert!(reading.unwrap_err().contains("Source cannot be empty"));
    }

    #[test]
    fn test_with_metadata() {
        let reading = IoTReading::new(
            sample_building_id(),
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            100.0,
            "kWh".to_string(),
            Utc::now(),
            "linky_ores".to_string(),
        )
        .unwrap()
        .with_metadata(serde_json::json!({"granularity": "30min", "quality": "good"}));

        assert!(reading.metadata.is_some());
        assert_eq!(reading.metadata.unwrap()["granularity"], "30min");
    }

    #[test]
    fn test_is_anomalous() {
        let reading = IoTReading::new(
            sample_building_id(),
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            150.0,
            "kWh".to_string(),
            Utc::now(),
            "linky_ores".to_string(),
        )
        .unwrap();

        // Average: 100 kWh, reading: 150 kWh = 50% deviation
        assert!(reading.is_anomalous(100.0, 40.0)); // Threshold 40% -> anomalous
        assert!(!reading.is_anomalous(100.0, 60.0)); // Threshold 60% -> not anomalous
    }

    #[test]
    fn test_normalized_value_electricity() {
        let reading_wh = IoTReading::new(
            sample_building_id(),
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            5000.0,
            "Wh".to_string(),
            Utc::now(),
            "linky_ores".to_string(),
        )
        .unwrap();

        assert_eq!(reading_wh.normalized_value(), 5.0); // 5000 Wh = 5 kWh

        let reading_mwh = IoTReading::new(
            sample_building_id(),
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            0.5,
            "MWh".to_string(),
            Utc::now(),
            "linky_ores".to_string(),
        )
        .unwrap();

        assert_eq!(reading_mwh.normalized_value(), 500.0); // 0.5 MWh = 500 kWh
    }

    #[test]
    fn test_normalized_value_temperature() {
        let reading_f = IoTReading::new(
            sample_building_id(),
            DeviceType::TemperatureSensor,
            MetricType::Temperature,
            32.0,
            "°F".to_string(),
            Utc::now(),
            "netatmo".to_string(),
        )
        .unwrap();

        // 32°F = 0°C
        assert!((reading_f.normalized_value() - 0.0).abs() < 0.01);

        let reading_k = IoTReading::new(
            sample_building_id(),
            DeviceType::TemperatureSensor,
            MetricType::Temperature,
            273.15,
            "K".to_string(),
            Utc::now(),
            "sensor".to_string(),
        )
        .unwrap();

        // 273.15 K = 0°C
        assert!((reading_k.normalized_value() - 0.0).abs() < 0.01);
    }
}
