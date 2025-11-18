use crate::application::ports::{
    ConsumptionDataPoint, LinkyApiClient, LinkyApiError, OAuth2TokenResponse, PowerDataPoint,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Production Linky API client implementation
///
/// Implements OAuth2 authorization code flow and data fetching
/// from Enedis Linky API and ORES (Belgium) API
pub struct LinkyApiClientImpl {
    client: reqwest::Client,
    base_url: String,
    client_id: String,
    client_secret: String,
}

impl LinkyApiClientImpl {
    pub fn new(base_url: String, client_id: String, client_secret: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url,
            client_id,
            client_secret,
        }
    }
}

#[derive(Debug, Serialize)]
struct TokenRequest {
    grant_type: String,
    code: Option<String>,
    refresh_token: Option<String>,
    redirect_uri: Option<String>,
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponseBody {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct ConsumptionResponse {
    meter_reading: MeterReading,
}

#[derive(Debug, Deserialize)]
struct MeterReading {
    usage_point_id: String,
    start: String,
    end: String,
    quality: String,
    reading_type: ReadingType,
    interval_reading: Vec<IntervalReading>,
}

#[derive(Debug, Deserialize)]
struct ReadingType {
    unit: String,
    aggregate: String,
    measuring_period: String,
}

#[derive(Debug, Deserialize)]
struct IntervalReading {
    value: String,
    date: String,
    interval_length: Option<String>,
    measure_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PowerResponse {
    meter_reading: PowerMeterReading,
}

#[derive(Debug, Deserialize)]
struct PowerMeterReading {
    usage_point_id: String,
    start: String,
    end: String,
    quality: String,
    reading_type: ReadingType,
    interval_reading: Vec<PowerIntervalReading>,
}

#[derive(Debug, Deserialize)]
struct PowerIntervalReading {
    value: String,
    date: String,
    direction: Option<String>,
}

#[async_trait]
impl LinkyApiClient for LinkyApiClientImpl {
    async fn exchange_authorization_code(
        &self,
        authorization_code: &str,
        redirect_uri: &str,
    ) -> Result<OAuth2TokenResponse, LinkyApiError> {
        let token_url = format!("{}/oauth2/v3/token", self.base_url);

        let request_body = TokenRequest {
            grant_type: "authorization_code".to_string(),
            code: Some(authorization_code.to_string()),
            refresh_token: None,
            redirect_uri: Some(redirect_uri.to_string()),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
        };

        let response = self
            .client
            .post(&token_url)
            .form(&request_body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LinkyApiError::Timeout("Request timed out".to_string())
                } else {
                    LinkyApiError::HttpError(e.to_string())
                }
            })?;

        if response.status() == 401 {
            return Err(LinkyApiError::InvalidAuthorizationCode("Invalid or expired authorization code".to_string()));
        }

        if !response.status().is_success() {
            return Err(LinkyApiError::HttpError(format!(
                "HTTP {} from token endpoint",
                response.status()
            )));
        }

        let token_response: TokenResponseBody = response.json().await.map_err(|e| {
            LinkyApiError::DeserializationError(format!("Failed to parse token response: {}", e))
        })?;

        Ok(OAuth2TokenResponse {
            access_token: token_response.access_token,
            refresh_token: Some(token_response.refresh_token),
            expires_in: token_response.expires_in as i64,
            token_type: token_response.token_type,
        })
    }

    async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<OAuth2TokenResponse, LinkyApiError> {
        let token_url = format!("{}/oauth2/v3/token", self.base_url);

        let request_body = TokenRequest {
            grant_type: "refresh_token".to_string(),
            code: None,
            refresh_token: Some(refresh_token.to_string()),
            redirect_uri: None,
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
        };

        let response = self
            .client
            .post(&token_url)
            .form(&request_body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LinkyApiError::Timeout("Request timed out".to_string())
                } else {
                    LinkyApiError::HttpError(e.to_string())
                }
            })?;

        if response.status() == 401 {
            return Err(LinkyApiError::TokenExpired("Refresh token expired or invalid".to_string()));
        }

        if !response.status().is_success() {
            return Err(LinkyApiError::HttpError(format!(
                "HTTP {} from token endpoint",
                response.status()
            )));
        }

        let token_response: TokenResponseBody = response.json().await.map_err(|e| {
            LinkyApiError::DeserializationError(format!("Failed to parse token response: {}", e))
        })?;

        Ok(OAuth2TokenResponse {
            access_token: token_response.access_token,
            refresh_token: Some(token_response.refresh_token),
            expires_in: token_response.expires_in as i64,
            token_type: token_response.token_type,
        })
    }

    async fn get_daily_consumption(
        &self,
        prm: &str,
        access_token: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ConsumptionDataPoint>, LinkyApiError> {
        if start_date >= end_date {
            return Err(LinkyApiError::InvalidDateRange(
                "start_date must be before end_date".to_string(),
            ));
        }

        let url = format!(
            "{}/metering_data_dc/v5/daily_consumption",
            self.base_url
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .query(&[
                ("usage_point_id", prm),
                ("start", &start_date.format("%Y-%m-%d").to_string()),
                ("end", &end_date.format("%Y-%m-%d").to_string()),
            ])
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LinkyApiError::Timeout("Request timed out".to_string())
                } else {
                    LinkyApiError::HttpError(e.to_string())
                }
            })?;

        if response.status() == 401 {
            return Err(LinkyApiError::TokenExpired("Access token expired".to_string()));
        }

        if response.status() == 403 {
            return Err(LinkyApiError::AuthenticationFailed("Access denied".to_string()));
        }

        if response.status() == 404 {
            return Err(LinkyApiError::InvalidPRM(prm.to_string()));
        }

        if response.status() == 429 {
            return Err(LinkyApiError::RateLimitExceeded("API rate limit exceeded".to_string()));
        }

        if !response.status().is_success() {
            return Err(LinkyApiError::HttpError(format!(
                "HTTP {} from consumption endpoint",
                response.status()
            )));
        }

        let consumption_response: ConsumptionResponse = response.json().await.map_err(|e| {
            LinkyApiError::DeserializationError(format!(
                "Failed to parse consumption response: {}",
                e
            ))
        })?;

        if consumption_response.meter_reading.interval_reading.is_empty() {
            return Err(LinkyApiError::NoDataAvailable("No data available for the specified period".to_string()));
        }

        let data_points: Result<Vec<ConsumptionDataPoint>, LinkyApiError> = consumption_response
            .meter_reading
            .interval_reading
            .into_iter()
            .map(|interval| {
                let value = interval
                    .value
                    .parse::<f64>()
                    .map_err(|e| LinkyApiError::DeserializationError(e.to_string()))?;

                let timestamp = DateTime::parse_from_rfc3339(&interval.date)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| LinkyApiError::DeserializationError(e.to_string()))?;

                Ok(ConsumptionDataPoint {
                    timestamp,
                    value,
                    quality: Some(consumption_response.meter_reading.quality.clone()),
                })
            })
            .collect();

        data_points
    }

    async fn get_monthly_consumption(
        &self,
        prm: &str,
        access_token: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ConsumptionDataPoint>, LinkyApiError> {
        if start_date >= end_date {
            return Err(LinkyApiError::InvalidDateRange(
                "start_date must be before end_date".to_string(),
            ));
        }

        let url = format!(
            "{}/metering_data_dc/v5/monthly_consumption",
            self.base_url
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .query(&[
                ("usage_point_id", prm),
                ("start", &start_date.format("%Y-%m-%d").to_string()),
                ("end", &end_date.format("%Y-%m-%d").to_string()),
            ])
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LinkyApiError::Timeout("Request timed out".to_string())
                } else {
                    LinkyApiError::HttpError(e.to_string())
                }
            })?;

        if response.status() == 401 {
            return Err(LinkyApiError::TokenExpired("Access token expired".to_string()));
        }

        if response.status() == 403 {
            return Err(LinkyApiError::AuthenticationFailed("Access denied".to_string()));
        }

        if response.status() == 404 {
            return Err(LinkyApiError::InvalidPRM(prm.to_string()));
        }

        if response.status() == 429 {
            return Err(LinkyApiError::RateLimitExceeded("API rate limit exceeded".to_string()));
        }

        if !response.status().is_success() {
            return Err(LinkyApiError::HttpError(format!(
                "HTTP {} from consumption endpoint",
                response.status()
            )));
        }

        let consumption_response: ConsumptionResponse = response.json().await.map_err(|e| {
            LinkyApiError::DeserializationError(format!(
                "Failed to parse consumption response: {}",
                e
            ))
        })?;

        if consumption_response.meter_reading.interval_reading.is_empty() {
            return Err(LinkyApiError::NoDataAvailable("No data available for the specified period".to_string()));
        }

        let data_points: Result<Vec<ConsumptionDataPoint>, LinkyApiError> = consumption_response
            .meter_reading
            .interval_reading
            .into_iter()
            .map(|interval| {
                let value = interval
                    .value
                    .parse::<f64>()
                    .map_err(|e| LinkyApiError::DeserializationError(e.to_string()))?;

                let timestamp = DateTime::parse_from_rfc3339(&interval.date)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| LinkyApiError::DeserializationError(e.to_string()))?;

                Ok(ConsumptionDataPoint {
                    timestamp,
                    value,
                    quality: Some(consumption_response.meter_reading.quality.clone()),
                })
            })
            .collect();

        data_points
    }

    async fn get_consumption_load_curve(
        &self,
        prm: &str,
        access_token: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<ConsumptionDataPoint>, LinkyApiError> {
        if start_date >= end_date {
            return Err(LinkyApiError::InvalidDateRange(
                "start_date must be before end_date".to_string(),
            ));
        }

        let url = format!(
            "{}/metering_data_clc/v5/consumption_load_curve",
            self.base_url
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .query(&[
                ("usage_point_id", prm),
                ("start", &start_date.format("%Y-%m-%dT%H:%M:%S").to_string()),
                ("end", &end_date.format("%Y-%m-%dT%H:%M:%S").to_string()),
            ])
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LinkyApiError::Timeout("Request timed out".to_string())
                } else {
                    LinkyApiError::HttpError(e.to_string())
                }
            })?;

        if response.status() == 401 {
            return Err(LinkyApiError::TokenExpired("Access token expired".to_string()));
        }

        if response.status() == 403 {
            return Err(LinkyApiError::AuthenticationFailed("Access denied".to_string()));
        }

        if response.status() == 404 {
            return Err(LinkyApiError::InvalidPRM(prm.to_string()));
        }

        if response.status() == 429 {
            return Err(LinkyApiError::RateLimitExceeded("API rate limit exceeded".to_string()));
        }

        if !response.status().is_success() {
            return Err(LinkyApiError::HttpError(format!(
                "HTTP {} from load curve endpoint",
                response.status()
            )));
        }

        let consumption_response: ConsumptionResponse = response.json().await.map_err(|e| {
            LinkyApiError::DeserializationError(format!("Failed to parse load curve response: {}", e))
        })?;

        if consumption_response.meter_reading.interval_reading.is_empty() {
            return Err(LinkyApiError::NoDataAvailable("No data available for the specified period".to_string()));
        }

        let data_points: Result<Vec<ConsumptionDataPoint>, LinkyApiError> = consumption_response
            .meter_reading
            .interval_reading
            .into_iter()
            .map(|interval| {
                let value = interval
                    .value
                    .parse::<f64>()
                    .map_err(|e| LinkyApiError::DeserializationError(e.to_string()))?;

                let timestamp = DateTime::parse_from_rfc3339(&interval.date)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| LinkyApiError::DeserializationError(e.to_string()))?;

                Ok(ConsumptionDataPoint {
                    timestamp,
                    value,
                    quality: Some(consumption_response.meter_reading.quality.clone()),
                })
            })
            .collect();

        data_points
    }

    async fn get_max_power(
        &self,
        prm: &str,
        access_token: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<PowerDataPoint>, LinkyApiError> {
        if start_date >= end_date {
            return Err(LinkyApiError::InvalidDateRange(
                "start_date must be before end_date".to_string(),
            ));
        }

        let url = format!("{}/metering_data_mp/v5/max_power", self.base_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .query(&[
                ("usage_point_id", prm),
                ("start", &start_date.format("%Y-%m-%d").to_string()),
                ("end", &end_date.format("%Y-%m-%d").to_string()),
            ])
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LinkyApiError::Timeout("Request timed out".to_string())
                } else {
                    LinkyApiError::HttpError(e.to_string())
                }
            })?;

        if response.status() == 401 {
            return Err(LinkyApiError::TokenExpired("Access token expired".to_string()));
        }

        if response.status() == 403 {
            return Err(LinkyApiError::AuthenticationFailed("Access denied".to_string()));
        }

        if response.status() == 404 {
            return Err(LinkyApiError::InvalidPRM(prm.to_string()));
        }

        if response.status() == 429 {
            return Err(LinkyApiError::RateLimitExceeded("API rate limit exceeded".to_string()));
        }

        if !response.status().is_success() {
            return Err(LinkyApiError::HttpError(format!(
                "HTTP {} from max power endpoint",
                response.status()
            )));
        }

        let power_response: PowerResponse = response.json().await.map_err(|e| {
            LinkyApiError::DeserializationError(format!("Failed to parse max power response: {}", e))
        })?;

        if power_response.meter_reading.interval_reading.is_empty() {
            return Err(LinkyApiError::NoDataAvailable("No data available for the specified period".to_string()));
        }

        let data_points: Result<Vec<PowerDataPoint>, LinkyApiError> = power_response
            .meter_reading
            .interval_reading
            .into_iter()
            .map(|interval| {
                let value = interval
                    .value
                    .parse::<f64>()
                    .map_err(|e| LinkyApiError::DeserializationError(e.to_string()))?;

                let timestamp = DateTime::parse_from_rfc3339(&interval.date)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| LinkyApiError::DeserializationError(e.to_string()))?;

                Ok(PowerDataPoint {
                    timestamp,
                    value,
                    direction: interval.direction,
                })
            })
            .collect();

        data_points
    }
}
