use crate::application::dto::{ConsumptionStatsDto, DailyAggregateDto, MonthlyAggregateDto};
use crate::application::ports::IoTRepository;
use crate::domain::entities::{DeviceType, IoTReading, LinkyDevice, MetricType};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of IoT repository
pub struct PostgresIoTRepository {
    pool: PgPool,
}

impl PostgresIoTRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IoTRepository for PostgresIoTRepository {
    async fn create_reading(&self, reading: &IoTReading) -> Result<IoTReading, String> {
        let record = sqlx::query!(
            r#"
            INSERT INTO iot_readings (
                id, building_id, device_type, metric_type, value,
                unit, timestamp, source, metadata, created_at
            )
            VALUES ($1, $2, $3::TEXT::device_type, $4::TEXT::metric_type, $5, $6, $7, $8, $9, $10)
            RETURNING id, building_id, device_type::text AS "device_type!", metric_type::text AS "metric_type!", value,
                      unit, timestamp, source, metadata, created_at
            "#,
            reading.id,
            reading.building_id,
            reading.device_type.to_string(),
            reading.metric_type.to_string(),
            reading.value,
            reading.unit,
            reading.timestamp,
            reading.source,
            reading.metadata,
            reading.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create IoT reading: {}", e))?;

        Ok(IoTReading {
            id: record.id,
            building_id: record.building_id,
            device_type: record
                .device_type
                .parse()
                .map_err(|e| format!("Invalid device_type: {}", e))?,
            metric_type: record
                .metric_type
                .parse()
                .map_err(|e| format!("Invalid metric_type: {}", e))?,
            value: record.value,
            // value is a method, not a field
            unit: record.unit,
            timestamp: record.timestamp,
            source: record.source,
            metadata: record.metadata,
            created_at: record.created_at,
        })
    }

    async fn create_readings_bulk(&self, readings: &[IoTReading]) -> Result<usize, String> {
        if readings.is_empty() {
            return Ok(0);
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        let mut count = 0;

        for reading in readings {
            sqlx::query!(
                r#"
                INSERT INTO iot_readings (
                    id, building_id, device_type, metric_type, value,
                    unit, timestamp, source, metadata, created_at
                )
                VALUES ($1, $2, $3::TEXT::device_type, $4::TEXT::metric_type, $5, $6, $7, $8, $9, $10)
                "#,
                reading.id,
                reading.building_id,
                reading.device_type.to_string(),
                reading.metric_type.to_string(),
                reading.value,
                reading.unit,
                reading.timestamp,
                reading.source,
                reading.metadata,
                reading.created_at,
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to insert reading: {}", e))?;

            count += 1;
        }

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(count)
    }

    async fn find_readings_by_building(
        &self,
        building_id: Uuid,
        device_type: Option<DeviceType>,
        metric_type: Option<MetricType>,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<IoTReading>, String> {
        let limit = limit.unwrap_or(1000).min(10000) as i64;
        let device_type_str = device_type.as_ref().map(|dt| dt.to_string());
        let metric_type_str = metric_type.as_ref().map(|mt| mt.to_string());

        let records = sqlx::query!(
            r#"
            SELECT id, building_id, device_type::TEXT as device_type, metric_type::TEXT as metric_type, value,
                   unit, timestamp, source, metadata, created_at
            FROM iot_readings
            WHERE building_id = $1
              AND timestamp >= $2
              AND timestamp <= $3
              AND ($4::TEXT IS NULL OR device_type::TEXT = $4)
              AND ($5::TEXT IS NULL OR metric_type::TEXT = $5)
            ORDER BY timestamp DESC
            LIMIT $6
            "#,
            building_id,
            start_date,
            end_date,
            device_type_str,
            metric_type_str,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to query IoT readings: {}", e))?;

        records
            .into_iter()
            .map(|record| {
                Ok(IoTReading {
                    id: record.id,
                    building_id: record.building_id,
                    device_type: record
                        .device_type
                        .ok_or("Device type is required")?
                        .parse()
                        .map_err(|e| format!("Invalid device_type: {}", e))?,
                    metric_type: record
                        .metric_type
                        .ok_or("Metric type is required")?
                        .parse()
                        .map_err(|e| format!("Invalid metric_type: {}", e))?,
                    value: record.value,
                    // value is a method, not a field
                    unit: record.unit,
                    timestamp: record.timestamp,
                    source: record.source,
                    metadata: record.metadata,
                    created_at: record.created_at,
                })
            })
            .collect()
    }

    async fn get_consumption_stats(
        &self,
        building_id: Uuid,
        metric_type: MetricType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<ConsumptionStatsDto, String> {
        let metric_type_str = metric_type.to_string();

        let record = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as "reading_count!",
                SUM(value) as total_consumption,
                AVG(value) as average_daily,
                MIN(value) as min_value,
                MAX(value) as max_value,
                unit,
                source
            FROM iot_readings
            WHERE building_id = $1
              AND metric_type::TEXT = $2
              AND timestamp >= $3
              AND timestamp <= $4
            GROUP BY unit, source
            "#,
            building_id,
            metric_type_str,
            start_date,
            end_date,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get consumption stats: {}", e))?;

        Ok(ConsumptionStatsDto {
            building_id,
            metric_type,
            period_start: start_date,
            period_end: end_date,
            total_consumption: record.total_consumption.unwrap_or(0.0),
            average_daily: record.average_daily.unwrap_or(0.0),
            min_value: record.min_value.unwrap_or(0.0),
            max_value: record.max_value.unwrap_or(0.0),
            reading_count: record.reading_count,
            unit: record.unit,
            source: record.source,
        })
    }

    async fn get_daily_aggregates(
        &self,
        building_id: Uuid,
        device_type: DeviceType,
        metric_type: MetricType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<DailyAggregateDto>, String> {
        let device_type_str = device_type.to_string();
        let metric_type_str = metric_type.to_string();

        let records = sqlx::query!(
            r#"
            SELECT
                DATE(timestamp) as "day!",
                AVG(value) as avg_value,
                MIN(value) as min_value,
                MAX(value) as max_value,
                SUM(value) as total_value,
                COUNT(*) as "reading_count!",
                source
            FROM iot_readings
            WHERE building_id = $1
              AND device_type::TEXT = $2
              AND metric_type::TEXT = $3
              AND timestamp >= $4
              AND timestamp <= $5
            GROUP BY DATE(timestamp), source
            ORDER BY DATE(timestamp) ASC
            "#,
            building_id,
            device_type_str,
            metric_type_str,
            start_date,
            end_date,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get daily aggregates: {}", e))?;

        Ok(records
            .into_iter()
            .map(|record| DailyAggregateDto {
                building_id,
                device_type: device_type.clone(),
                metric_type: metric_type.clone(),
                day: record.day.and_hms_opt(0, 0, 0).unwrap().and_utc(),
                avg_value: record.avg_value.unwrap_or(0.0),
                min_value: record.min_value.unwrap_or(0.0),
                max_value: record.max_value.unwrap_or(0.0),
                total_value: record.total_value.unwrap_or(0.0),
                reading_count: record.reading_count,
                source: record.source,
            })
            .collect())
    }

    async fn get_monthly_aggregates(
        &self,
        building_id: Uuid,
        device_type: DeviceType,
        metric_type: MetricType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<MonthlyAggregateDto>, String> {
        let device_type_str = device_type.to_string();
        let metric_type_str = metric_type.to_string();

        let records = sqlx::query!(
            r#"
            SELECT
                DATE_TRUNC('month', timestamp) as "month!",
                AVG(value) as avg_value,
                MIN(value) as min_value,
                MAX(value) as max_value,
                SUM(value) as total_value,
                COUNT(*) as "reading_count!",
                source
            FROM iot_readings
            WHERE building_id = $1
              AND device_type::TEXT = $2
              AND metric_type::TEXT = $3
              AND timestamp >= $4
              AND timestamp <= $5
            GROUP BY DATE_TRUNC('month', timestamp), source
            ORDER BY DATE_TRUNC('month', timestamp) ASC
            "#,
            building_id,
            device_type_str,
            metric_type_str,
            start_date,
            end_date,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get monthly aggregates: {}", e))?;

        Ok(records
            .into_iter()
            .map(|record| MonthlyAggregateDto {
                building_id,
                device_type: device_type.clone(),
                metric_type: metric_type.clone(),
                month: record.month,
                avg_value: record.avg_value.unwrap_or(0.0),
                min_value: record.min_value.unwrap_or(0.0),
                max_value: record.max_value.unwrap_or(0.0),
                total_value: record.total_value.unwrap_or(0.0),
                reading_count: record.reading_count,
                source: record.source,
            })
            .collect())
    }

    async fn detect_anomalies(
        &self,
        building_id: Uuid,
        metric_type: MetricType,
        threshold_percentage: f64,
        lookback_days: i64,
    ) -> Result<Vec<IoTReading>, String> {
        let metric_type_str = metric_type.to_string();

        // Calculate average value for the lookback period
        let avg_record = sqlx::query!(
            r#"
            SELECT AVG(value) as avg_value
            FROM iot_readings
            WHERE building_id = $1
              AND metric_type::TEXT = $2
              AND timestamp >= NOW() - INTERVAL '1 day' * $3
            "#,
            building_id,
            metric_type_str,
            lookback_days as f64,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to calculate average: {}", e))?;

        let avg_value = avg_record.avg_value.unwrap_or(0.0);
        let threshold = avg_value * (threshold_percentage / 100.0);
        let upper_bound = avg_value + threshold;
        let lower_bound = (avg_value - threshold).max(0.0);

        // Find readings outside the threshold
        let records = sqlx::query!(
            r#"
            SELECT id, building_id, device_type::TEXT as device_type, metric_type::TEXT as metric_type, value,
                   unit, timestamp, source, metadata, created_at
            FROM iot_readings
            WHERE building_id = $1
              AND metric_type::TEXT = $2
              AND timestamp >= NOW() - INTERVAL '1 day' * $3
              AND (value > $4 OR value < $5)
            ORDER BY timestamp DESC
            "#,
            building_id,
            metric_type_str,
            lookback_days as f64,
            upper_bound,
            lower_bound,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to detect anomalies: {}", e))?;

        records
            .into_iter()
            .map(|record| {
                Ok(IoTReading {
                    id: record.id,
                    building_id: record.building_id,
                    device_type: record
                        .device_type
                        .ok_or("Device type is required")?
                        .parse()
                        .map_err(|e| format!("Invalid device_type: {}", e))?,
                    metric_type: record
                        .metric_type
                        .ok_or("Metric type is required")?
                        .parse()
                        .map_err(|e| format!("Invalid metric_type: {}", e))?,
                    value: record.value,
                    // value is a method, not a field
                    unit: record.unit,
                    timestamp: record.timestamp,
                    source: record.source,
                    metadata: record.metadata,
                    created_at: record.created_at,
                })
            })
            .collect()
    }

    async fn create_linky_device(&self, device: &LinkyDevice) -> Result<LinkyDevice, String> {
        let provider_str = device.provider.to_string();

        let record = sqlx::query!(
            r#"
            INSERT INTO linky_devices (
                id, building_id, prm, provider, api_key_encrypted, refresh_token_encrypted,
                token_expires_at, sync_enabled, last_sync_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4::TEXT::linky_provider, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, building_id, prm, provider::TEXT as provider, api_key_encrypted, refresh_token_encrypted,
                      token_expires_at, sync_enabled, last_sync_at, created_at, updated_at
            "#,
            device.id,
            device.building_id,
            device.prm,
            provider_str,
            device.api_key_encrypted,
            device.refresh_token_encrypted,
            device.token_expires_at,
            device.sync_enabled,
            device.last_sync_at,
            device.created_at,
            device.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create Linky device: {}", e))?;

        Ok(LinkyDevice {
            id: record.id,
            building_id: record.building_id,
            prm: record.prm,
            provider: record
                .provider
                .ok_or("Provider is required")?
                .parse()
                .map_err(|e| format!("Invalid provider: {}", e))?,
            api_key_encrypted: record.api_key_encrypted,
            refresh_token_encrypted: record.refresh_token_encrypted,
            token_expires_at: record.token_expires_at,
            sync_enabled: record.sync_enabled,
            last_sync_at: record.last_sync_at,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    async fn find_linky_device_by_id(
        &self,
        device_id: Uuid,
    ) -> Result<Option<LinkyDevice>, String> {
        let record = sqlx::query!(
            r#"
            SELECT id, building_id, prm, provider::TEXT as provider, api_key_encrypted, refresh_token_encrypted,
                   token_expires_at, sync_enabled, last_sync_at, created_at, updated_at
            FROM linky_devices
            WHERE id = $1
            "#,
            device_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find Linky device: {}", e))?;

        record
            .map(|r| -> Result<LinkyDevice, String> {
                Ok(LinkyDevice {
                    id: r.id,
                    building_id: r.building_id,
                    prm: r.prm,
                    provider: r
                        .provider
                        .ok_or("Provider is required")?
                        .parse()
                        .map_err(|e| format!("Invalid provider: {}", e))?,
                    api_key_encrypted: r.api_key_encrypted,
                    refresh_token_encrypted: r.refresh_token_encrypted,
                    token_expires_at: r.token_expires_at,
                    sync_enabled: r.sync_enabled,
                    last_sync_at: r.last_sync_at,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                })
            })
            .transpose()
    }

    async fn find_linky_device_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Option<LinkyDevice>, String> {
        let record = sqlx::query!(
            r#"
            SELECT id, building_id, prm, provider::TEXT as provider, api_key_encrypted, refresh_token_encrypted,
                   token_expires_at, sync_enabled, last_sync_at, created_at, updated_at
            FROM linky_devices
            WHERE building_id = $1
            "#,
            building_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find Linky device by building: {}", e))?;

        record
            .map(|r| -> Result<LinkyDevice, String> {
                Ok(LinkyDevice {
                    id: r.id,
                    building_id: r.building_id,
                    prm: r.prm,
                    provider: r
                        .provider
                        .ok_or("Provider is required")?
                        .parse()
                        .map_err(|e| format!("Invalid provider: {}", e))?,
                    api_key_encrypted: r.api_key_encrypted,
                    refresh_token_encrypted: r.refresh_token_encrypted,
                    token_expires_at: r.token_expires_at,
                    sync_enabled: r.sync_enabled,
                    last_sync_at: r.last_sync_at,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                })
            })
            .transpose()
    }

    async fn find_linky_device_by_prm(
        &self,
        prm: &str,
        provider: &str,
    ) -> Result<Option<LinkyDevice>, String> {
        let record = sqlx::query!(
            r#"
            SELECT id, building_id, prm, provider::TEXT as provider, api_key_encrypted, refresh_token_encrypted,
                   token_expires_at, sync_enabled, last_sync_at, created_at, updated_at
            FROM linky_devices
            WHERE prm = $1 AND provider::TEXT = $2
            "#,
            prm,
            provider,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find Linky device by PRM: {}", e))?;

        record
            .map(|r| -> Result<LinkyDevice, String> {
                Ok(LinkyDevice {
                    id: r.id,
                    building_id: r.building_id,
                    prm: r.prm,
                    provider: r
                        .provider
                        .ok_or("Provider is required")?
                        .parse()
                        .map_err(|e| format!("Invalid provider: {}", e))?,
                    api_key_encrypted: r.api_key_encrypted,
                    refresh_token_encrypted: r.refresh_token_encrypted,
                    token_expires_at: r.token_expires_at,
                    sync_enabled: r.sync_enabled,
                    last_sync_at: r.last_sync_at,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                })
            })
            .transpose()
    }

    async fn update_linky_device(&self, device: &LinkyDevice) -> Result<LinkyDevice, String> {
        let provider_str = device.provider.to_string();

        let record = sqlx::query!(
            r#"
            UPDATE linky_devices
            SET building_id = $2,
                prm = $3,
                provider = $4::TEXT::linky_provider,
                api_key_encrypted = $5,
                refresh_token_encrypted = $6,
                token_expires_at = $7,
                sync_enabled = $8,
                last_sync_at = $9,
                updated_at = $10
            WHERE id = $1
            RETURNING id, building_id, prm, provider::TEXT as provider, api_key_encrypted, refresh_token_encrypted,
                      token_expires_at, sync_enabled, last_sync_at, created_at, updated_at
            "#,
            device.id,
            device.building_id,
            device.prm,
            provider_str,
            device.api_key_encrypted,
            device.refresh_token_encrypted,
            device.token_expires_at,
            device.sync_enabled,
            device.last_sync_at,
            device.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to update Linky device: {}", e))?;

        Ok(LinkyDevice {
            id: record.id,
            building_id: record.building_id,
            prm: record.prm,
            provider: record
                .provider
                .ok_or("Provider is required")?
                .parse()
                .map_err(|e| format!("Invalid provider: {}", e))?,
            api_key_encrypted: record.api_key_encrypted,
            refresh_token_encrypted: record.refresh_token_encrypted,
            token_expires_at: record.token_expires_at,
            sync_enabled: record.sync_enabled,
            last_sync_at: record.last_sync_at,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    async fn delete_linky_device(&self, device_id: Uuid) -> Result<(), String> {
        sqlx::query!(
            r#"
            DELETE FROM linky_devices
            WHERE id = $1
            "#,
            device_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete Linky device: {}", e))?;

        Ok(())
    }

    async fn find_devices_needing_sync(&self) -> Result<Vec<LinkyDevice>, String> {
        let records = sqlx::query!(
            r#"
            SELECT id, building_id, prm, provider::TEXT as provider, api_key_encrypted, refresh_token_encrypted,
                   token_expires_at, sync_enabled, last_sync_at, created_at, updated_at
            FROM linky_devices
            WHERE sync_enabled = true
              AND (last_sync_at IS NULL OR last_sync_at < NOW() - INTERVAL '1 day')
            ORDER BY last_sync_at ASC NULLS FIRST
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find devices needing sync: {}", e))?;

        records
            .into_iter()
            .map(|r| -> Result<LinkyDevice, String> {
                Ok(LinkyDevice {
                    id: r.id,
                    building_id: r.building_id,
                    prm: r.prm,
                    provider: r
                        .provider
                        .ok_or("Provider is required")?
                        .parse()
                        .map_err(|e| format!("Invalid provider: {}", e))?,
                    api_key_encrypted: r.api_key_encrypted,
                    refresh_token_encrypted: r.refresh_token_encrypted,
                    token_expires_at: r.token_expires_at,
                    sync_enabled: r.sync_enabled,
                    last_sync_at: r.last_sync_at,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                })
            })
            .collect()
    }

    async fn find_devices_with_expired_tokens(&self) -> Result<Vec<LinkyDevice>, String> {
        let records = sqlx::query!(
            r#"
            SELECT id, building_id, prm, provider::TEXT as provider, api_key_encrypted, refresh_token_encrypted,
                   token_expires_at, sync_enabled, last_sync_at, created_at, updated_at
            FROM linky_devices
            WHERE token_expires_at < NOW()
            ORDER BY token_expires_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find devices with expired tokens: {}", e))?;

        records
            .into_iter()
            .map(|r| -> Result<LinkyDevice, String> {
                Ok(LinkyDevice {
                    id: r.id,
                    building_id: r.building_id,
                    prm: r.prm,
                    provider: r
                        .provider
                        .ok_or("Provider is required")?
                        .parse()
                        .map_err(|e| format!("Invalid provider: {}", e))?,
                    api_key_encrypted: r.api_key_encrypted,
                    refresh_token_encrypted: r.refresh_token_encrypted,
                    token_expires_at: r.token_expires_at,
                    sync_enabled: r.sync_enabled,
                    last_sync_at: r.last_sync_at,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                })
            })
            .collect()
    }
}
