use crate::application::dto::{
    ConfigureLinkyDeviceDto, ConsumptionStatsDto, CreateIoTReadingDto, DailyAggregateDto,
    IoTReadingResponseDto, LinkySyncResponseDto, LinkyDeviceResponseDto, MonthlyAggregateDto,
    QueryIoTReadingsDto, SyncLinkyDataDto,
};
use crate::application::ports::{IoTRepository, LinkyApiClient};
use crate::domain::entities::{DeviceType, IoTReading, LinkyDevice, MetricType};
use crate::infrastructure::audit::{log_audit_event, AuditEventType};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Use cases for managing IoT readings and consumption data
pub struct IoTUseCases {
    iot_repo: Arc<dyn IoTRepository>,
}

impl IoTUseCases {
    pub fn new(iot_repo: Arc<dyn IoTRepository>) -> Self {
        Self { iot_repo }
    }

    /// Create a single IoT reading
    pub async fn create_reading(
        &self,
        dto: CreateIoTReadingDto,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<IoTReadingResponseDto, String> {
        // Validate device type and metric type compatibility
        Self::validate_metric_for_device(&dto.device_type, &dto.metric_type)?;

        let mut reading = IoTReading::new(
            dto.building_id,
            dto.device_type,
            dto.metric_type,
            dto.value,
            dto.unit,
            dto.timestamp,
            dto.source,
        )?;

        if let Some(metadata) = dto.metadata {
            reading = reading.with_metadata(metadata);
        }

        let created = self.iot_repo.create_reading(&reading).await?;

        // Audit log (async, non-blocking)
        tokio::spawn(async move {
            log_audit_event(
                AuditEventType::IoTReadingCreated,
                Some(user_id),
                Some(organization_id),
                Some(format!(
                    "IoT reading created: {:?} {} {}",
                    dto.device_type, dto.metric_type, dto.value
                )),
                None,
            )
            .await;
        });

        Ok(IoTReadingResponseDto::from(created))
    }

    /// Create multiple IoT readings in bulk
    pub async fn create_readings_bulk(
        &self,
        dtos: Vec<CreateIoTReadingDto>,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<usize, String> {
        if dtos.is_empty() {
            return Err("No readings provided".to_string());
        }

        let readings: Result<Vec<IoTReading>, String> = dtos
            .iter()
            .map(|dto| {
                Self::validate_metric_for_device(&dto.device_type, &dto.metric_type)?;
                let mut reading = IoTReading::new(
                    dto.building_id,
                    dto.device_type.clone(),
                    dto.metric_type.clone(),
                    dto.value,
                    dto.unit.clone(),
                    dto.timestamp,
                    dto.source.clone(),
                )?;

                if let Some(ref metadata) = dto.metadata {
                    reading = reading.with_metadata(metadata.clone());
                }

                Ok(reading)
            })
            .collect();

        let readings = readings?;
        let count = self.iot_repo.create_readings_bulk(&readings).await?;

        // Audit log (async, non-blocking)
        tokio::spawn(async move {
            log_audit_event(
                AuditEventType::IoTReadingsBulkCreated,
                Some(user_id),
                Some(organization_id),
                Some(format!("Bulk IoT readings created: {} records", count)),
                None,
            )
            .await;
        });

        Ok(count)
    }

    /// Query IoT readings with filters
    pub async fn query_readings(
        &self,
        query: QueryIoTReadingsDto,
    ) -> Result<Vec<IoTReadingResponseDto>, String> {
        let readings = self
            .iot_repo
            .find_readings_by_building(
                query.building_id,
                query.device_type,
                query.metric_type,
                query.start_date,
                query.end_date,
                query.limit,
            )
            .await?;

        Ok(readings.into_iter().map(IoTReadingResponseDto::from).collect())
    }

    /// Get consumption statistics for a building and metric type
    pub async fn get_consumption_stats(
        &self,
        building_id: Uuid,
        metric_type: MetricType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<ConsumptionStatsDto, String> {
        if start_date >= end_date {
            return Err("start_date must be before end_date".to_string());
        }

        self.iot_repo
            .get_consumption_stats(building_id, metric_type, start_date, end_date)
            .await
    }

    /// Get daily aggregates for a device type and metric
    pub async fn get_daily_aggregates(
        &self,
        building_id: Uuid,
        device_type: DeviceType,
        metric_type: MetricType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<DailyAggregateDto>, String> {
        Self::validate_metric_for_device(&device_type, &metric_type)?;

        if start_date >= end_date {
            return Err("start_date must be before end_date".to_string());
        }

        self.iot_repo
            .get_daily_aggregates(building_id, device_type, metric_type, start_date, end_date)
            .await
    }

    /// Get monthly aggregates for a device type and metric
    pub async fn get_monthly_aggregates(
        &self,
        building_id: Uuid,
        device_type: DeviceType,
        metric_type: MetricType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<MonthlyAggregateDto>, String> {
        Self::validate_metric_for_device(&device_type, &metric_type)?;

        if start_date >= end_date {
            return Err("start_date must be before end_date".to_string());
        }

        self.iot_repo
            .get_monthly_aggregates(building_id, device_type, metric_type, start_date, end_date)
            .await
    }

    /// Detect consumption anomalies (values deviating significantly from average)
    pub async fn detect_anomalies(
        &self,
        building_id: Uuid,
        metric_type: MetricType,
        threshold_percentage: f64,
        lookback_days: i64,
    ) -> Result<Vec<IoTReadingResponseDto>, String> {
        if threshold_percentage <= 0.0 || threshold_percentage > 100.0 {
            return Err("threshold_percentage must be between 0 and 100".to_string());
        }

        if lookback_days <= 0 {
            return Err("lookback_days must be positive".to_string());
        }

        let anomalies = self
            .iot_repo
            .detect_anomalies(building_id, metric_type, threshold_percentage, lookback_days)
            .await?;

        Ok(anomalies.into_iter().map(IoTReadingResponseDto::from).collect())
    }

    /// Validate that a metric type is compatible with a device type
    fn validate_metric_for_device(
        device_type: &DeviceType,
        metric_type: &MetricType,
    ) -> Result<(), String> {
        match device_type {
            DeviceType::ElectricityMeter => match metric_type {
                MetricType::ElectricityConsumption
                | MetricType::Power
                | MetricType::Voltage => Ok(()),
                _ => Err(format!(
                    "Metric type {:?} is not compatible with Linky device",
                    metric_type
                )),
            },
            DeviceType::GasMeter => match metric_type {
                MetricType::GasConsumption => Ok(()),
                _ => Err(format!(
                    "Metric type {:?} is not compatible with GasMeter device",
                    metric_type
                )),
            },
            DeviceType::PowerMeter => match metric_type {
                MetricType::Power => Ok(()),
                _ => Err(format!(
                    "Metric type {:?} is not compatible with PowerMeter device",
                    metric_type
                )),
            },
            DeviceType::WaterMeter => match metric_type {
                MetricType::WaterConsumption => Ok(()),
                _ => Err(format!(
                    "Metric type {:?} is not compatible with WaterMeter device",
                    metric_type
                )),
            },
            DeviceType::TemperatureSensor => match metric_type {
                MetricType::Temperature => Ok(()),
                _ => Err(format!(
                    "Metric type {:?} is not compatible with TemperatureSensor device",
                    metric_type
                )),
            },
            DeviceType::HumiditySensor => match metric_type {
                MetricType::Humidity => Ok(()),
                _ => Err(format!(
                    "Metric type {:?} is not compatible with HumiditySensor device",
                    metric_type
                )),
            },
        }
    }
}

/// Use cases for managing Linky devices and syncing data
pub struct LinkyUseCases {
    iot_repo: Arc<dyn IoTRepository>,
    linky_client: Arc<dyn LinkyApiClient>,
}

impl LinkyUseCases {
    pub fn new(iot_repo: Arc<dyn IoTRepository>, linky_client: Arc<dyn LinkyApiClient>) -> Self {
        Self {
            iot_repo,
            linky_client,
        }
    }

    /// Configure a new Linky device for a building
    pub async fn configure_linky_device(
        &self,
        dto: ConfigureLinkyDeviceDto,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<LinkyDeviceResponseDto, String> {
        // Check if device already exists for this building
        if let Some(_existing) = self
            .iot_repo
            .find_linky_device_by_building(dto.building_id)
            .await?
        {
            return Err("Linky device already configured for this building".to_string());
        }

        // Check if PRM already registered with this provider
        if let Some(_existing) = self
            .iot_repo
            .find_linky_device_by_prm(&dto.prm, &dto.provider.to_string())
            .await?
        {
            return Err("This PRM is already registered with this provider".to_string());
        }

        // Exchange authorization code for access token
        let redirect_uri = format!("https://koprogo.be/oauth/linky/callback"); // TODO: Make configurable
        let token_response = self
            .linky_client
            .exchange_authorization_code(&dto.authorization_code, &redirect_uri)
            .await
            .map_err(|e| format!("Failed to exchange authorization code: {:?}", e))?;

        // Create Linky device
        let mut device = LinkyDevice::new(
            dto.building_id,
            dto.prm,
            dto.provider,
            token_response.access_token,
        )?;

        // Add refresh token if present
        if let Some(refresh_token) = token_response.refresh_token {
            let expires_at = chrono::Utc::now() + chrono::Duration::seconds(token_response.expires_in);
            device = device.with_refresh_token(refresh_token, expires_at);
        }

        let created = self.iot_repo.create_linky_device(&device).await?;

        // Audit log (async, non-blocking)
        let prm = created.prm.clone();
        let provider = created.provider.clone();
        tokio::spawn(async move {
            log_audit_event(
                AuditEventType::LinkyDeviceConfigured,
                Some(user_id),
                Some(organization_id),
                Some(format!(
                    "Linky device configured: PRM={}, Provider={}",
                    prm, provider
                )),
                None,
            )
            .await;
        });

        Ok(LinkyDeviceResponseDto::from(created))
    }

    /// Sync data from Linky API for a building
    pub async fn sync_linky_data(
        &self,
        dto: SyncLinkyDataDto,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<LinkySyncResponseDto, String> {
        // Find Linky device for this building
        let mut device = self
            .iot_repo
            .find_linky_device_by_building(dto.building_id)
            .await?
            .ok_or("No Linky device configured for this building".to_string())?;

        if !device.sync_enabled {
            return Err("Sync is disabled for this device".to_string());
        }

        // Refresh token if expired
        if device.is_token_expired() {
            // Unwrap refresh token (must exist if token is expired)
            let refresh_token = device.refresh_token_encrypted
                .as_ref()
                .ok_or("Refresh token not available")?;

            let token_response = self
                .linky_client
                .refresh_access_token(refresh_token)
                .await
                .map_err(|e| format!("Failed to refresh access token: {:?}", e))?;

            // Convert expires_in to DateTime
            let expires_at = chrono::Utc::now() + chrono::Duration::seconds(token_response.expires_in);

            device.update_tokens(
                token_response.access_token,
                token_response.refresh_token,
                Some(expires_at),
            )?;

            self.iot_repo.update_linky_device(&device).await?;
        }

        let sync_started_at = Utc::now();

        // Fetch daily consumption data
        let consumption_data = self
            .linky_client
            .get_daily_consumption(
                &device.prm,
                &device.api_key_encrypted,
                dto.start_date,
                dto.end_date,
            )
            .await
            .map_err(|e| format!("Failed to fetch consumption data: {:?}", e))?;

        // Convert consumption data to IoT readings
        let readings: Vec<IoTReading> = consumption_data
            .iter()
            .map(|data_point| -> Result<IoTReading, String> {
                let reading = IoTReading::new(
                    dto.building_id,
                    DeviceType::ElectricityMeter,
                    MetricType::ElectricityConsumption,
                    data_point.value,
                    "kWh".to_string(),
                    data_point.timestamp,
                    device.provider.to_string(),
                )?;

                Ok(reading.with_metadata(serde_json::json!({
                    "prm": device.prm,
                    "quality": data_point.quality,
                    "sync_id": Uuid::new_v4().to_string(),
                })))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let readings_count = readings.len();

        // Bulk insert readings
        if readings_count > 0 {
            self.iot_repo.create_readings_bulk(&readings).await?;
        }

        // Update device last_sync_at
        device.mark_synced();
        self.iot_repo.update_linky_device(&device).await?;

        // Audit log (async, non-blocking)
        let device_id = device.id;
        tokio::spawn(async move {
            log_audit_event(
                AuditEventType::LinkyDataSynced,
                Some(user_id),
                Some(organization_id),
                Some(format!(
                    "Linky data synced: {} readings fetched",
                    readings_count
                )),
                None,
            )
            .await;
        });

        Ok(LinkySyncResponseDto {
            device_id,
            sync_started_at,
            start_date: dto.start_date,
            end_date: dto.end_date,
            readings_fetched: readings_count,
            success: true,
            error_message: None,
        })
    }

    /// Get Linky device information for a building
    pub async fn get_linky_device(
        &self,
        building_id: Uuid,
    ) -> Result<LinkyDeviceResponseDto, String> {
        let device = self
            .iot_repo
            .find_linky_device_by_building(building_id)
            .await?
            .ok_or("No Linky device configured for this building".to_string())?;

        Ok(LinkyDeviceResponseDto::from(device))
    }

    /// Delete Linky device configuration
    pub async fn delete_linky_device(
        &self,
        building_id: Uuid,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<(), String> {
        let device = self
            .iot_repo
            .find_linky_device_by_building(building_id)
            .await?
            .ok_or("No Linky device configured for this building".to_string())?;

        let device_id = device.id;
        self.iot_repo.delete_linky_device(device_id).await?;

        // Audit log (async, non-blocking)
        tokio::spawn(async move {
            log_audit_event(
                AuditEventType::LinkyDeviceDeleted,
                Some(user_id),
                Some(organization_id),
                Some(format!("Linky device deleted: {}", device_id)),
                None,
            )
            .await;
        });

        Ok(())
    }

    /// Enable/disable sync for a Linky device
    pub async fn toggle_sync(
        &self,
        building_id: Uuid,
        enabled: bool,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<LinkyDeviceResponseDto, String> {
        let mut device = self
            .iot_repo
            .find_linky_device_by_building(building_id)
            .await?
            .ok_or("No Linky device configured for this building".to_string())?;

        if enabled {
            device.enable_sync();
        } else {
            device.disable_sync();
        }

        let updated = self.iot_repo.update_linky_device(&device).await?;

        // Audit log (async, non-blocking)
        tokio::spawn(async move {
            log_audit_event(
                AuditEventType::LinkySyncToggled,
                Some(user_id),
                Some(organization_id),
                Some(format!("Linky sync {}", if enabled { "enabled" } else { "disabled" })),
                None,
            )
            .await;
        });

        Ok(LinkyDeviceResponseDto::from(updated))
    }

    /// Find devices that need syncing (enabled, not synced recently)
    pub async fn find_devices_needing_sync(&self) -> Result<Vec<LinkyDeviceResponseDto>, String> {
        let devices = self.iot_repo.find_devices_needing_sync().await?;
        Ok(devices.into_iter().map(LinkyDeviceResponseDto::from).collect())
    }

    /// Find devices with expired tokens
    pub async fn find_devices_with_expired_tokens(
        &self,
    ) -> Result<Vec<LinkyDeviceResponseDto>, String> {
        let devices = self.iot_repo.find_devices_with_expired_tokens().await?;
        Ok(devices.into_iter().map(LinkyDeviceResponseDto::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_metric_for_device_linky_valid() {
        assert!(IoTUseCases::validate_metric_for_device(
            &DeviceType::ElectricityMeter,
            &MetricType::ElectricityConsumption
        )
        .is_ok());
        assert!(IoTUseCases::validate_metric_for_device(
            &DeviceType::ElectricityMeter,
            &MetricType::Power
        )
        .is_ok());
        assert!(IoTUseCases::validate_metric_for_device(
            &DeviceType::ElectricityMeter,
            &MetricType::Voltage
        )
        .is_ok());
    }

    #[test]
    fn test_validate_metric_for_device_linky_invalid() {
        assert!(IoTUseCases::validate_metric_for_device(
            &DeviceType::ElectricityMeter,
            &MetricType::GasConsumption
        )
        .is_err());
        assert!(IoTUseCases::validate_metric_for_device(
            &DeviceType::ElectricityMeter,
            &MetricType::WaterConsumption
        )
        .is_err());
    }

    #[test]
    fn test_validate_metric_for_device_ores_valid() {
        assert!(IoTUseCases::validate_metric_for_device(
            &DeviceType::ElectricityMeter,
            &MetricType::GasConsumption
        )
        .is_ok());
    }

    #[test]
    fn test_validate_metric_for_device_ores_invalid() {
        assert!(IoTUseCases::validate_metric_for_device(
            &DeviceType::ElectricityMeter,
            &MetricType::ElectricityConsumption
        )
        .is_err());
    }

    #[test]
    fn test_validate_metric_for_device_water_meter_valid() {
        assert!(IoTUseCases::validate_metric_for_device(
            &DeviceType::WaterMeter,
            &MetricType::WaterConsumption
        )
        .is_ok());
    }

    #[test]
    fn test_validate_metric_for_device_temperature_sensor_valid() {
        assert!(IoTUseCases::validate_metric_for_device(
            &DeviceType::TemperatureSensor,
            &MetricType::Temperature
        )
        .is_ok());
    }

    #[test]
    fn test_validate_metric_for_device_humidity_sensor_valid() {
        assert!(IoTUseCases::validate_metric_for_device(
            &DeviceType::HumiditySensor,
            &MetricType::Humidity
        )
        .is_ok());
    }
}
