use crate::application::dto::{
    ConfigureLinkyDeviceDto, CreateIoTReadingDto, QueryIoTReadingsDto, SyncLinkyDataDto,
};
use crate::application::use_cases::{IoTUseCases, LinkyUseCases};
use crate::domain::entities::{DeviceType, MetricType};
use crate::infrastructure::web::middleware::AuthenticatedUser;
use actix_web::{error::ErrorBadRequest, web, HttpResponse, Result};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// IoT Readings Handlers
// ============================================================================

/// Create a single IoT reading
///
/// POST /api/v1/iot/readings
///
/// Request body:
/// ```json
/// {
///   "building_id": "uuid",
///   "device_type": "Linky",
///   "metric_type": "ElectricityConsumption",
///   "value": 15.5,
///   "unit": "kWh",
///   "timestamp": "2024-01-01T00:00:00Z",
///   "source": "Enedis",
///   "metadata": {"prm": "12345678901234"}
/// }
/// ```
pub async fn create_iot_reading(
    auth: AuthenticatedUser,
    dto: web::Json<CreateIoTReadingDto>,
    iot_use_cases: web::Data<Arc<IoTUseCases>>,
) -> Result<HttpResponse> {
    let organization_id = auth.organization_id.ok_or_else(|| ErrorBadRequest("Organization ID is required"))?;

    match iot_use_cases
        .create_reading(dto.into_inner(), auth.user_id, organization_id)
        .await
    {
        Ok(reading) => Ok(HttpResponse::Created().json(reading)),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Create multiple IoT readings in bulk
///
/// POST /api/v1/iot/readings/bulk
///
/// Request body:
/// ```json
/// [
///   {
///     "building_id": "uuid",
///     "device_type": "Linky",
///     "metric_type": "ElectricityConsumption",
///     "value": 15.5,
///     "unit": "kWh",
///     "timestamp": "2024-01-01T00:00:00Z",
///     "source": "Enedis",
///     "metadata": null
///   },
///   ...
/// ]
/// ```
pub async fn create_iot_readings_bulk(
    auth: AuthenticatedUser,
    dtos: web::Json<Vec<CreateIoTReadingDto>>,
    iot_use_cases: web::Data<Arc<IoTUseCases>>,
) -> Result<HttpResponse> {
    let organization_id = auth.organization_id.ok_or_else(|| ErrorBadRequest("Organization ID is required"))?;

    match iot_use_cases
        .create_readings_bulk(dtos.into_inner(), auth.user_id, organization_id)
        .await
    {
        Ok(count) => Ok(HttpResponse::Created().json(serde_json::json!({
            "count": count,
            "message": format!("{} IoT readings created", count)
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Query IoT readings with filters
///
/// GET /api/v1/iot/readings?building_id=uuid&device_type=Linky&metric_type=ElectricityConsumption&start_date=2024-01-01&end_date=2024-01-31&limit=100
pub async fn query_iot_readings(
    auth: AuthenticatedUser,
    query: web::Query<QueryIoTReadingsDto>,
    iot_use_cases: web::Data<Arc<IoTUseCases>>,
) -> Result<HttpResponse> {
    let _ = auth; // Authentication required
    match iot_use_cases.query_readings(query.into_inner()).await {
        Ok(readings) => Ok(HttpResponse::Ok().json(readings)),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Get consumption statistics for a building
///
/// GET /api/v1/iot/buildings/{building_id}/consumption/stats?metric_type=ElectricityConsumption&start_date=2024-01-01&end_date=2024-01-31
pub async fn get_consumption_stats(
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    query: web::Query<serde_json::Value>,
    iot_use_cases: web::Data<Arc<IoTUseCases>>,
) -> Result<HttpResponse> {
    let _ = auth; // Authentication required
    let building_id = path.into_inner();

    let metric_type_str = query
        .get("metric_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("metric_type query param required"))?;
    let metric_type: MetricType = metric_type_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid metric_type"))?;

    let start_date_str = query
        .get("start_date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("start_date query param required"))?;
    let start_date: DateTime<Utc> = start_date_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid start_date format (use ISO 8601)"))?;

    let end_date_str = query
        .get("end_date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("end_date query param required"))?;
    let end_date: DateTime<Utc> = end_date_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid end_date format (use ISO 8601)"))?;

    match iot_use_cases
        .get_consumption_stats(building_id, metric_type, start_date, end_date)
        .await
    {
        Ok(stats) => Ok(HttpResponse::Ok().json(stats)),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Get daily aggregates for a building
///
/// GET /api/v1/iot/buildings/{building_id}/consumption/daily?device_type=Linky&metric_type=ElectricityConsumption&start_date=2024-01-01&end_date=2024-01-31
pub async fn get_daily_aggregates(
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    query: web::Query<serde_json::Value>,
    iot_use_cases: web::Data<Arc<IoTUseCases>>,
) -> Result<HttpResponse> {
    let _ = auth; // Authentication required
    let building_id = path.into_inner();

    let device_type_str = query
        .get("device_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("device_type query param required"))?;
    let device_type: DeviceType = device_type_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid device_type"))?;

    let metric_type_str = query
        .get("metric_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("metric_type query param required"))?;
    let metric_type: MetricType = metric_type_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid metric_type"))?;

    let start_date_str = query
        .get("start_date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("start_date query param required"))?;
    let start_date: DateTime<Utc> = start_date_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid start_date format (use ISO 8601)"))?;

    let end_date_str = query
        .get("end_date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("end_date query param required"))?;
    let end_date: DateTime<Utc> = end_date_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid end_date format (use ISO 8601)"))?;

    match iot_use_cases
        .get_daily_aggregates(building_id, device_type, metric_type, start_date, end_date)
        .await
    {
        Ok(aggregates) => Ok(HttpResponse::Ok().json(aggregates)),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Get monthly aggregates for a building
///
/// GET /api/v1/iot/buildings/{building_id}/consumption/monthly?device_type=Linky&metric_type=ElectricityConsumption&start_date=2024-01-01&end_date=2024-12-31
pub async fn get_monthly_aggregates(
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    query: web::Query<serde_json::Value>,
    iot_use_cases: web::Data<Arc<IoTUseCases>>,
) -> Result<HttpResponse> {
    let _ = auth; // Authentication required
    let building_id = path.into_inner();

    let device_type_str = query
        .get("device_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("device_type query param required"))?;
    let device_type: DeviceType = device_type_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid device_type"))?;

    let metric_type_str = query
        .get("metric_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("metric_type query param required"))?;
    let metric_type: MetricType = metric_type_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid metric_type"))?;

    let start_date_str = query
        .get("start_date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("start_date query param required"))?;
    let start_date: DateTime<Utc> = start_date_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid start_date format (use ISO 8601)"))?;

    let end_date_str = query
        .get("end_date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("end_date query param required"))?;
    let end_date: DateTime<Utc> = end_date_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid end_date format (use ISO 8601)"))?;

    match iot_use_cases
        .get_monthly_aggregates(building_id, device_type, metric_type, start_date, end_date)
        .await
    {
        Ok(aggregates) => Ok(HttpResponse::Ok().json(aggregates)),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Detect consumption anomalies for a building
///
/// GET /api/v1/iot/buildings/{building_id}/consumption/anomalies?metric_type=ElectricityConsumption&threshold_percentage=30&lookback_days=30
pub async fn detect_anomalies(
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    query: web::Query<serde_json::Value>,
    iot_use_cases: web::Data<Arc<IoTUseCases>>,
) -> Result<HttpResponse> {
    let _ = auth; // Authentication required
    let building_id = path.into_inner();

    let metric_type_str = query
        .get("metric_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorBadRequest("metric_type query param required"))?;
    let metric_type: MetricType = metric_type_str
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid metric_type"))?;

    let threshold_percentage = query
        .get("threshold_percentage")
        .and_then(|v| v.as_f64())
        .unwrap_or(30.0);

    let lookback_days = query
        .get("lookback_days")
        .and_then(|v| v.as_i64())
        .unwrap_or(30);

    match iot_use_cases
        .detect_anomalies(building_id, metric_type, threshold_percentage, lookback_days)
        .await
    {
        Ok(anomalies) => Ok(HttpResponse::Ok().json(anomalies)),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

// ============================================================================
// Linky Device Handlers
// ============================================================================

/// Configure a Linky device for a building
///
/// POST /api/v1/iot/linky/devices
///
/// Request body:
/// ```json
/// {
///   "building_id": "uuid",
///   "prm": "12345678901234",
///   "provider": "Enedis",
///   "authorization_code": "abc123..."
/// }
/// ```
pub async fn configure_linky_device(
    auth: AuthenticatedUser,
    dto: web::Json<ConfigureLinkyDeviceDto>,
    linky_use_cases: web::Data<Arc<LinkyUseCases>>,
) -> Result<HttpResponse> {
    let organization_id = auth.organization_id.ok_or_else(|| ErrorBadRequest("Organization ID is required"))?;

    match linky_use_cases
        .configure_linky_device(dto.into_inner(), auth.user_id, organization_id)
        .await
    {
        Ok(device) => Ok(HttpResponse::Created().json(device)),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Get Linky device for a building
///
/// GET /api/v1/iot/linky/buildings/{building_id}/device
pub async fn get_linky_device(
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    linky_use_cases: web::Data<Arc<LinkyUseCases>>,
) -> Result<HttpResponse> {
    let _ = auth; // Authentication required
    let building_id = path.into_inner();

    match linky_use_cases.get_linky_device(building_id).await {
        Ok(device) => Ok(HttpResponse::Ok().json(device)),
        Err(e) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Delete Linky device for a building
///
/// DELETE /api/v1/iot/linky/buildings/{building_id}/device
pub async fn delete_linky_device(
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    linky_use_cases: web::Data<Arc<LinkyUseCases>>,
) -> Result<HttpResponse> {
    let building_id = path.into_inner();
    let organization_id = auth.organization_id.ok_or_else(|| ErrorBadRequest("Organization ID is required"))?;

    match linky_use_cases
        .delete_linky_device(building_id, auth.user_id, organization_id)
        .await
    {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Sync Linky data for a building
///
/// POST /api/v1/iot/linky/buildings/{building_id}/sync
///
/// Request body:
/// ```json
/// {
///   "building_id": "uuid",
///   "start_date": "2024-01-01T00:00:00Z",
///   "end_date": "2024-01-31T23:59:59Z"
/// }
/// ```
pub async fn sync_linky_data(
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    dto: web::Json<SyncLinkyDataDto>,
    linky_use_cases: web::Data<Arc<LinkyUseCases>>,
) -> Result<HttpResponse> {
    let _building_id = path.into_inner();
    let organization_id = auth.organization_id.ok_or_else(|| ErrorBadRequest("Organization ID is required"))?;

    match linky_use_cases
        .sync_linky_data(dto.into_inner(), auth.user_id, organization_id)
        .await
    {
        Ok(sync_result) => Ok(HttpResponse::Ok().json(sync_result)),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Toggle sync for a Linky device
///
/// PUT /api/v1/iot/linky/buildings/{building_id}/sync/toggle
///
/// Request body:
/// ```json
/// {
///   "enabled": true
/// }
/// ```
pub async fn toggle_linky_sync(
    auth: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
    linky_use_cases: web::Data<Arc<LinkyUseCases>>,
) -> Result<HttpResponse> {
    let building_id = path.into_inner();
    let enabled = body
        .get("enabled")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| ErrorBadRequest("enabled field required (boolean)"))?;
    let organization_id = auth.organization_id.ok_or_else(|| ErrorBadRequest("Organization ID is required"))?;

    match linky_use_cases
        .toggle_sync(building_id, enabled, auth.user_id, organization_id)
        .await
    {
        Ok(device) => Ok(HttpResponse::Ok().json(device)),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Find Linky devices needing sync
///
/// GET /api/v1/iot/linky/devices/needing-sync
pub async fn find_devices_needing_sync(
    auth: AuthenticatedUser,
    linky_use_cases: web::Data<Arc<LinkyUseCases>>,
) -> Result<HttpResponse> {
    let _ = auth; // Authentication required

    match linky_use_cases.find_devices_needing_sync().await {
        Ok(devices) => Ok(HttpResponse::Ok().json(devices)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Find Linky devices with expired tokens
///
/// GET /api/v1/iot/linky/devices/expired-tokens
pub async fn find_devices_with_expired_tokens(
    auth: AuthenticatedUser,
    linky_use_cases: web::Data<Arc<LinkyUseCases>>,
) -> Result<HttpResponse> {
    let _ = auth; // Authentication required

    match linky_use_cases.find_devices_with_expired_tokens().await {
        Ok(devices) => Ok(HttpResponse::Ok().json(devices)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        }))),
    }
}

/// Configure IoT routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/iot")
            // IoT Readings
            .route("/readings", web::post().to(create_iot_reading))
            .route("/readings/bulk", web::post().to(create_iot_readings_bulk))
            .route("/readings", web::get().to(query_iot_readings))
            .route(
                "/buildings/{building_id}/consumption/stats",
                web::get().to(get_consumption_stats),
            )
            .route(
                "/buildings/{building_id}/consumption/daily",
                web::get().to(get_daily_aggregates),
            )
            .route(
                "/buildings/{building_id}/consumption/monthly",
                web::get().to(get_monthly_aggregates),
            )
            .route(
                "/buildings/{building_id}/consumption/anomalies",
                web::get().to(detect_anomalies),
            )
            // Linky Devices
            .route("/linky/devices", web::post().to(configure_linky_device))
            .route(
                "/linky/buildings/{building_id}/device",
                web::get().to(get_linky_device),
            )
            .route(
                "/linky/buildings/{building_id}/device",
                web::delete().to(delete_linky_device),
            )
            .route(
                "/linky/buildings/{building_id}/sync",
                web::post().to(sync_linky_data),
            )
            .route(
                "/linky/buildings/{building_id}/sync/toggle",
                web::put().to(toggle_linky_sync),
            )
            .route(
                "/linky/devices/needing-sync",
                web::get().to(find_devices_needing_sync),
            )
            .route(
                "/linky/devices/expired-tokens",
                web::get().to(find_devices_with_expired_tokens),
            ),
    );
}
