use crate::application::dto::{ConsumptionStatsDto, DailyAggregateDto, MonthlyAggregateDto};
use crate::domain::entities::{DeviceType, IoTReading, LinkyDevice, MetricType};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Repository port for IoT readings and Linky devices
#[async_trait]
pub trait IoTRepository: Send + Sync {
    // ========================================
    // IoT Readings
    // ========================================

    /// Create a new IoT reading
    async fn create_reading(&self, reading: &IoTReading) -> Result<IoTReading, String>;

    /// Create multiple IoT readings (bulk insert)
    async fn create_readings_bulk(&self, readings: &[IoTReading]) -> Result<usize, String>;

    /// Find readings by building and time range
    async fn find_readings_by_building(
        &self,
        building_id: Uuid,
        device_type: Option<DeviceType>,
        metric_type: Option<MetricType>,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<IoTReading>, String>;

    /// Get consumption statistics for a building
    async fn get_consumption_stats(
        &self,
        building_id: Uuid,
        metric_type: MetricType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<ConsumptionStatsDto, String>;

    /// Get daily aggregated readings
    async fn get_daily_aggregates(
        &self,
        building_id: Uuid,
        device_type: DeviceType,
        metric_type: MetricType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<DailyAggregateDto>, String>;

    /// Get monthly aggregated readings
    async fn get_monthly_aggregates(
        &self,
        building_id: Uuid,
        device_type: DeviceType,
        metric_type: MetricType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<MonthlyAggregateDto>, String>;

    /// Detect anomalies (readings exceeding average by threshold percentage)
    async fn detect_anomalies(
        &self,
        building_id: Uuid,
        metric_type: MetricType,
        threshold_percentage: f64,
        lookback_days: i64,
    ) -> Result<Vec<IoTReading>, String>;

    // ========================================
    // Linky Devices
    // ========================================

    /// Create a new Linky device configuration
    async fn create_linky_device(&self, device: &LinkyDevice) -> Result<LinkyDevice, String>;

    /// Find Linky device by ID
    async fn find_linky_device_by_id(&self, device_id: Uuid)
        -> Result<Option<LinkyDevice>, String>;

    /// Find Linky device by building ID
    async fn find_linky_device_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Option<LinkyDevice>, String>;

    /// Find Linky device by PRM and provider
    async fn find_linky_device_by_prm(
        &self,
        prm: &str,
        provider: &str,
    ) -> Result<Option<LinkyDevice>, String>;

    /// Update Linky device (tokens, sync status, etc.)
    async fn update_linky_device(&self, device: &LinkyDevice) -> Result<LinkyDevice, String>;

    /// Delete Linky device
    async fn delete_linky_device(&self, device_id: Uuid) -> Result<(), String>;

    /// Find all Linky devices that need sync (sync_enabled=true AND (never synced OR last_sync > 24h))
    async fn find_devices_needing_sync(&self) -> Result<Vec<LinkyDevice>, String>;

    /// Find all Linky devices with expired tokens (token_expires_at <= NOW + 5 minutes)
    async fn find_devices_with_expired_tokens(&self) -> Result<Vec<LinkyDevice>, String>;
}
