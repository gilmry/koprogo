use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Linky API client port for fetching electricity consumption data
#[async_trait]
pub trait LinkyApiClient: Send + Sync {
    /// Exchange OAuth2 authorization code for access token + refresh token
    async fn exchange_authorization_code(
        &self,
        authorization_code: &str,
        redirect_uri: &str,
    ) -> Result<OAuth2TokenResponse, LinkyApiError>;

    /// Refresh OAuth2 access token using refresh token
    async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<OAuth2TokenResponse, LinkyApiError>;

    /// Get daily electricity consumption (30-minute granularity)
    /// Returns consumption data for each day in the range
    async fn get_daily_consumption(
        &self,
        prm: &str,
        access_token: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ConsumptionDataPoint>, LinkyApiError>;

    /// Get monthly electricity consumption (aggregated by month)
    async fn get_monthly_consumption(
        &self,
        prm: &str,
        access_token: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ConsumptionDataPoint>, LinkyApiError>;

    /// Get consumption load curve (granularity: 30 minutes)
    /// High-frequency data for detailed analysis
    async fn get_consumption_load_curve(
        &self,
        prm: &str,
        access_token: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ConsumptionDataPoint>, LinkyApiError>;

    /// Get maximum power draw (kW) over a period
    async fn get_max_power(
        &self,
        prm: &str,
        access_token: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<PowerDataPoint>, LinkyApiError>;
}

/// OAuth2 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64, // Seconds
    pub token_type: String, // Usually "Bearer"
}

/// Consumption data point (single reading)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumptionDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64, // kWh
    pub quality: Option<String>, // Data quality indicator (e.g., "good", "estimated")
}

/// Power data point (instantaneous power draw)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64, // kW
    pub direction: Option<String>, // "consumption" or "production" (for solar panels)
}

/// Linky API errors
#[derive(Debug, thiserror::Error)]
pub enum LinkyApiError {
    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid authorization code: {0}")]
    InvalidAuthorizationCode(String),

    #[error("Token expired or invalid: {0}")]
    TokenExpired(String),

    #[error("Invalid PRM: {0}")]
    InvalidPRM(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Invalid date range: {0}")]
    InvalidDateRange(String),

    #[error("No data available for period: {0}")]
    NoDataAvailable(String),

    #[error("API timeout: {0}")]
    Timeout(String),

    #[error("Internal API error: {0}")]
    InternalError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}
