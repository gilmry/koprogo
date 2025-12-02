use crate::domain::entities::{DeviceType, IoTReading, LinkyDevice, LinkyProvider, MetricType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ========================================
// IoT Reading DTOs
// ========================================

/// DTO for creating a new IoT reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIoTReadingDto {
    pub building_id: Uuid,
    pub device_type: DeviceType,
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub metadata: Option<serde_json::Value>,
}

/// DTO for IoT reading response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoTReadingResponseDto {
    pub id: Uuid,
    pub building_id: Uuid,
    pub device_type: DeviceType,
    pub metric_type: MetricType,
    pub value: f64,
    pub normalized_value: f64, // Value converted to standard unit
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<IoTReading> for IoTReadingResponseDto {
    fn from(reading: IoTReading) -> Self {
        Self {
            id: reading.id,
            building_id: reading.building_id,
            device_type: reading.device_type,
            metric_type: reading.metric_type,
            normalized_value: reading.normalized_value(),
            value: reading.value,
            unit: reading.unit,
            timestamp: reading.timestamp,
            source: reading.source,
            metadata: reading.metadata,
            created_at: reading.created_at,
        }
    }
}

/// DTO for consumption statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumptionStatsDto {
    pub building_id: Uuid,
    pub metric_type: MetricType,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_consumption: f64,
    pub average_daily: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub reading_count: i64,
    pub unit: String,
    pub source: String,
}

/// DTO for anomaly detection results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionDto {
    pub reading: IoTReadingResponseDto,
    pub is_anomalous: bool,
    pub average_value: f64,
    pub deviation_percentage: f64,
    pub threshold_percentage: f64,
}

// ========================================
// Linky Device DTOs
// ========================================

/// DTO for configuring a new Linky device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureLinkyDeviceDto {
    pub building_id: Uuid,
    pub prm: String,
    pub provider: LinkyProvider,
    pub authorization_code: String, // OAuth2 authorization code (will be exchanged for tokens)
}

/// DTO for Linky device response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkyDeviceResponseDto {
    pub id: Uuid,
    pub building_id: Uuid,
    pub prm: String,
    pub provider: LinkyProvider,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_enabled: bool,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub is_token_expired: bool,
    pub needs_sync: bool,
    pub api_endpoint: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<LinkyDevice> for LinkyDeviceResponseDto {
    fn from(device: LinkyDevice) -> Self {
        Self {
            id: device.id,
            building_id: device.building_id,
            prm: device.prm.clone(),
            provider: device.provider,
            last_sync_at: device.last_sync_at,
            sync_enabled: device.sync_enabled,
            token_expires_at: device.token_expires_at,
            is_token_expired: device.is_token_expired(),
            needs_sync: device.needs_sync(),
            api_endpoint: device.api_endpoint().to_string(),
            created_at: device.created_at,
            updated_at: device.updated_at,
        }
    }
}

/// DTO for triggering Linky data sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLinkyDataDto {
    pub building_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

/// DTO for Linky sync response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkySyncResponseDto {
    pub device_id: Uuid,
    pub sync_started_at: DateTime<Utc>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub readings_fetched: usize,
    pub success: bool,
    pub error_message: Option<String>,
}

// ========================================
// Query DTOs
// ========================================

/// DTO for querying IoT readings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryIoTReadingsDto {
    pub building_id: Uuid,
    pub device_type: Option<DeviceType>,
    pub metric_type: Option<MetricType>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub limit: Option<usize>,
}

impl Default for QueryIoTReadingsDto {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            building_id: Uuid::nil(), // Must be set by caller
            device_type: None,
            metric_type: None,
            start_date: now - chrono::Duration::days(30),
            end_date: now,
            limit: Some(1000),
        }
    }
}

/// DTO for daily aggregated readings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyAggregateDto {
    pub building_id: Uuid,
    pub device_type: DeviceType,
    pub metric_type: MetricType,
    pub day: DateTime<Utc>,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub total_value: f64,
    pub reading_count: i64,
    pub source: String,
}

/// DTO for monthly aggregated readings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyAggregateDto {
    pub building_id: Uuid,
    pub device_type: DeviceType,
    pub metric_type: MetricType,
    pub month: DateTime<Utc>,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub total_value: f64,
    pub reading_count: i64,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iot_reading_response_dto_from_entity() {
        let reading = IoTReading::new(
            Uuid::new_v4(),
            DeviceType::ElectricityMeter,
            MetricType::ElectricityConsumption,
            150.5,
            "kWh".to_string(),
            Utc::now(),
            "linky_ores".to_string(),
        )
        .unwrap();

        let dto: IoTReadingResponseDto = reading.clone().into();

        assert_eq!(dto.id, reading.id);
        assert_eq!(dto.value, 150.5);
        assert_eq!(dto.normalized_value, 150.5);
        assert_eq!(dto.metric_type, MetricType::ElectricityConsumption);
    }

    #[test]
    fn test_linky_device_response_dto_from_entity() {
        let device = LinkyDevice::new(
            Uuid::new_v4(),
            "12345678901234".to_string(),
            LinkyProvider::Enedis,
            "encrypted_token".to_string(),
        )
        .unwrap();

        let dto: LinkyDeviceResponseDto = device.clone().into();

        assert_eq!(dto.id, device.id);
        assert_eq!(dto.prm, "12345678901234");
        assert_eq!(dto.provider, LinkyProvider::Enedis);
        assert_eq!(dto.api_endpoint, "https://ext.hml.myelectricaldata.fr/v1");
        assert!(dto.needs_sync); // Never synced
    }

    #[test]
    fn test_query_iot_readings_dto_default() {
        let query = QueryIoTReadingsDto::default();

        assert_eq!(query.limit, Some(1000));
        assert!(query.device_type.is_none());
        assert!(query.metric_type.is_none());

        let days_diff = (query.end_date - query.start_date).num_days();
        assert_eq!(days_diff, 30);
    }
}
