use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Linky smart meter device configuration
/// Stores configuration for Linky (Enedis France) or Ores (Belgium) smart meters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkyDevice {
    pub id: Uuid,
    pub building_id: Uuid,
    pub prm: String, // Point Reference Measure (unique meter identifier)
    pub provider: LinkyProvider,
    pub api_key_encrypted: String, // OAuth2 access token (encrypted at rest)
    pub refresh_token_encrypted: Option<String>, // OAuth2 refresh token (encrypted)
    pub token_expires_at: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LinkyDevice {
    /// Create a new Linky device configuration
    pub fn new(
        building_id: Uuid,
        prm: String,
        provider: LinkyProvider,
        api_key_encrypted: String,
    ) -> Result<Self, String> {
        // Validate PRM (Point Reference Measure)
        Self::validate_prm(&prm)?;

        // Validate API key is non-empty
        if api_key_encrypted.trim().is_empty() {
            return Err("API key cannot be empty".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            building_id,
            prm,
            provider,
            api_key_encrypted,
            refresh_token_encrypted: None,
            token_expires_at: None,
            last_sync_at: None,
            sync_enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Set refresh token (for OAuth2 token rotation)
    pub fn with_refresh_token(
        mut self,
        refresh_token_encrypted: String,
        expires_at: DateTime<Utc>,
    ) -> Self {
        self.refresh_token_encrypted = Some(refresh_token_encrypted);
        self.token_expires_at = Some(expires_at);
        self
    }

    /// Enable/disable automatic sync
    pub fn set_sync_enabled(&mut self, enabled: bool) {
        self.sync_enabled = enabled;
        self.updated_at = Utc::now();
    }

    /// Enable automatic sync (convenience method)
    pub fn enable_sync(&mut self) {
        self.set_sync_enabled(true);
    }

    /// Disable automatic sync (convenience method)
    pub fn disable_sync(&mut self) {
        self.set_sync_enabled(false);
    }

    /// Update last sync timestamp
    pub fn mark_synced(&mut self) {
        self.last_sync_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Update OAuth2 tokens (access + refresh)
    pub fn update_tokens(
        &mut self,
        api_key_encrypted: String,
        refresh_token_encrypted: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), String> {
        if api_key_encrypted.trim().is_empty() {
            return Err("API key cannot be empty".to_string());
        }

        self.api_key_encrypted = api_key_encrypted;
        self.refresh_token_encrypted = refresh_token_encrypted;
        self.token_expires_at = expires_at;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Check if OAuth2 token is expired or about to expire (within 5 minutes)
    pub fn is_token_expired(&self) -> bool {
        match self.token_expires_at {
            Some(expires_at) => expires_at <= Utc::now() + chrono::Duration::minutes(5),
            None => false, // No expiration set, assume valid
        }
    }

    /// Check if device needs sync (never synced or last sync > 24h ago)
    pub fn needs_sync(&self) -> bool {
        if !self.sync_enabled {
            return false;
        }

        match self.last_sync_at {
            Some(last_sync) => {
                let hours_since_sync = (Utc::now() - last_sync).num_hours();
                hours_since_sync >= 24
            }
            None => true, // Never synced
        }
    }

    /// Validate PRM format
    /// - France (Enedis): 14 digits
    /// - Belgium (Ores): 18 digits (EAN code)
    fn validate_prm(prm: &str) -> Result<(), String> {
        let prm = prm.trim();

        if prm.is_empty() {
            return Err("PRM cannot be empty".to_string());
        }

        // Check if all characters are digits
        if !prm.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("PRM must contain only digits: {}", prm));
        }

        // Validate length (14 for France, 18 for Belgium)
        let len = prm.len();
        if len != 14 && len != 18 {
            return Err(format!(
                "PRM must be 14 digits (France) or 18 digits (Belgium), got {}: {}",
                len, prm
            ));
        }

        Ok(())
    }

    /// Get API endpoint for this provider
    pub fn api_endpoint(&self) -> &'static str {
        match self.provider {
            LinkyProvider::Ores => "https://ext.prod-eu.oresnet.be/v1",
            LinkyProvider::Enedis => "https://ext.hml.myelectricaldata.fr/v1",
        }
    }
}

/// Linky smart meter provider
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LinkyProvider {
    Ores,   // Belgium (Ores network)
    Enedis, // France (Enedis Linky)
}

impl std::fmt::Display for LinkyProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkyProvider::Ores => write!(f, "Ores"),
            LinkyProvider::Enedis => write!(f, "Enedis"),
        }
    }
}

impl std::str::FromStr for LinkyProvider {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Ores" => Ok(LinkyProvider::Ores),
            "Enedis" => Ok(LinkyProvider::Enedis),
            _ => Err(format!("Invalid LinkyProvider: {}", s)),
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
    fn test_create_linky_device_success() {
        let device = LinkyDevice::new(
            sample_building_id(),
            "12345678901234".to_string(), // 14 digits (France)
            LinkyProvider::Enedis,
            "encrypted_access_token".to_string(),
        );

        assert!(device.is_ok());
        let d = device.unwrap();
        assert_eq!(d.building_id, sample_building_id());
        assert_eq!(d.prm, "12345678901234");
        assert_eq!(d.provider, LinkyProvider::Enedis);
        assert!(d.sync_enabled);
        assert!(d.last_sync_at.is_none());
    }

    #[test]
    fn test_create_linky_device_belgium() {
        let device = LinkyDevice::new(
            sample_building_id(),
            "541448030003312345".to_string(), // 18 digits (Belgium EAN)
            LinkyProvider::Ores,
            "encrypted_access_token".to_string(),
        );

        assert!(device.is_ok());
        let d = device.unwrap();
        assert_eq!(d.prm, "541448030003312345");
        assert_eq!(d.provider, LinkyProvider::Ores);
    }

    #[test]
    fn test_validate_prm_empty() {
        let device = LinkyDevice::new(
            sample_building_id(),
            "".to_string(),
            LinkyProvider::Enedis,
            "encrypted_access_token".to_string(),
        );

        assert!(device.is_err());
        assert!(device.unwrap_err().contains("PRM cannot be empty"));
    }

    #[test]
    fn test_validate_prm_invalid_length() {
        let device = LinkyDevice::new(
            sample_building_id(),
            "12345".to_string(), // Too short
            LinkyProvider::Enedis,
            "encrypted_access_token".to_string(),
        );

        assert!(device.is_err());
        assert!(device.unwrap_err().contains("must be 14 digits"));
    }

    #[test]
    fn test_validate_prm_non_digits() {
        let device = LinkyDevice::new(
            sample_building_id(),
            "1234567890ABCD".to_string(), // Contains letters
            LinkyProvider::Enedis,
            "encrypted_access_token".to_string(),
        );

        assert!(device.is_err());
        assert!(device.unwrap_err().contains("must contain only digits"));
    }

    #[test]
    fn test_validate_api_key_empty() {
        let device = LinkyDevice::new(
            sample_building_id(),
            "12345678901234".to_string(),
            LinkyProvider::Enedis,
            "".to_string(),
        );

        assert!(device.is_err());
        assert!(device.unwrap_err().contains("API key cannot be empty"));
    }

    #[test]
    fn test_with_refresh_token() {
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        let device = LinkyDevice::new(
            sample_building_id(),
            "12345678901234".to_string(),
            LinkyProvider::Enedis,
            "encrypted_access_token".to_string(),
        )
        .unwrap()
        .with_refresh_token("encrypted_refresh_token".to_string(), expires_at);

        assert!(device.refresh_token_encrypted.is_some());
        assert_eq!(
            device.refresh_token_encrypted.unwrap(),
            "encrypted_refresh_token"
        );
        assert_eq!(device.token_expires_at.unwrap(), expires_at);
    }

    #[test]
    fn test_set_sync_enabled() {
        let mut device = LinkyDevice::new(
            sample_building_id(),
            "12345678901234".to_string(),
            LinkyProvider::Enedis,
            "encrypted_access_token".to_string(),
        )
        .unwrap();

        assert!(device.sync_enabled);

        device.set_sync_enabled(false);
        assert!(!device.sync_enabled);
    }

    #[test]
    fn test_mark_synced() {
        let mut device = LinkyDevice::new(
            sample_building_id(),
            "12345678901234".to_string(),
            LinkyProvider::Enedis,
            "encrypted_access_token".to_string(),
        )
        .unwrap();

        assert!(device.last_sync_at.is_none());

        device.mark_synced();
        assert!(device.last_sync_at.is_some());
        assert!(device.last_sync_at.unwrap() <= Utc::now());
    }

    #[test]
    fn test_update_tokens() {
        let mut device = LinkyDevice::new(
            sample_building_id(),
            "12345678901234".to_string(),
            LinkyProvider::Enedis,
            "old_token".to_string(),
        )
        .unwrap();

        let expires_at = Utc::now() + chrono::Duration::hours(2);
        let result = device.update_tokens(
            "new_access_token".to_string(),
            Some("new_refresh_token".to_string()),
            Some(expires_at),
        );

        assert!(result.is_ok());
        assert_eq!(device.api_key_encrypted, "new_access_token");
        assert_eq!(device.refresh_token_encrypted.unwrap(), "new_refresh_token");
        assert_eq!(device.token_expires_at.unwrap(), expires_at);
    }

    #[test]
    fn test_update_tokens_empty() {
        let mut device = LinkyDevice::new(
            sample_building_id(),
            "12345678901234".to_string(),
            LinkyProvider::Enedis,
            "old_token".to_string(),
        )
        .unwrap();

        let result = device.update_tokens("".to_string(), None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key cannot be empty"));
    }

    #[test]
    fn test_is_token_expired() {
        let mut device = LinkyDevice::new(
            sample_building_id(),
            "12345678901234".to_string(),
            LinkyProvider::Enedis,
            "encrypted_access_token".to_string(),
        )
        .unwrap();

        // No expiration set
        assert!(!device.is_token_expired());

        // Token expires in 10 minutes (not expired)
        device.token_expires_at = Some(Utc::now() + chrono::Duration::minutes(10));
        assert!(!device.is_token_expired());

        // Token expires in 3 minutes (considered expired - within 5 min buffer)
        device.token_expires_at = Some(Utc::now() + chrono::Duration::minutes(3));
        assert!(device.is_token_expired());

        // Token already expired
        device.token_expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(device.is_token_expired());
    }

    #[test]
    fn test_needs_sync() {
        let mut device = LinkyDevice::new(
            sample_building_id(),
            "12345678901234".to_string(),
            LinkyProvider::Enedis,
            "encrypted_access_token".to_string(),
        )
        .unwrap();

        // Never synced
        assert!(device.needs_sync());

        // Synced recently (1 hour ago)
        device.last_sync_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(!device.needs_sync());

        // Synced long ago (25 hours ago)
        device.last_sync_at = Some(Utc::now() - chrono::Duration::hours(25));
        assert!(device.needs_sync());

        // Sync disabled
        device.set_sync_enabled(false);
        assert!(!device.needs_sync());
    }

    #[test]
    fn test_api_endpoint() {
        let device_ores = LinkyDevice::new(
            sample_building_id(),
            "541448030003312345".to_string(),
            LinkyProvider::Ores,
            "encrypted_access_token".to_string(),
        )
        .unwrap();

        assert_eq!(
            device_ores.api_endpoint(),
            "https://ext.prod-eu.oresnet.be/v1"
        );

        let device_enedis = LinkyDevice::new(
            sample_building_id(),
            "12345678901234".to_string(),
            LinkyProvider::Enedis,
            "encrypted_access_token".to_string(),
        )
        .unwrap();

        assert_eq!(
            device_enedis.api_endpoint(),
            "https://ext.hml.myelectricaldata.fr/v1"
        );
    }
}
