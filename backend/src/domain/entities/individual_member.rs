use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Individual Member (non-copropriétaire)
/// Issue #280: Energy group buying extensions — allows individuals to join energy campaigns
/// Art. 22 RED II (Renewable Energy Directive II)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndividualMember {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub email: String,
    pub postal_code: String,
    pub has_gdpr_consent: bool,
    pub consent_at: Option<DateTime<Utc>>,
    pub annual_consumption_kwh: Option<f64>,
    pub current_provider: Option<String>,
    pub ean_code: Option<String>, // Belgian EAN identifier
    pub unsubscribed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl IndividualMember {
    pub fn new(campaign_id: Uuid, email: String, postal_code: String) -> Result<Self, String> {
        if email.is_empty() || !email.contains('@') {
            return Err("Invalid email address".to_string());
        }
        if postal_code.is_empty() {
            return Err("Postal code cannot be empty".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            campaign_id,
            email,
            postal_code,
            has_gdpr_consent: false,
            consent_at: None,
            annual_consumption_kwh: None,
            current_provider: None,
            ean_code: None,
            unsubscribed_at: None,
            created_at: Utc::now(),
        })
    }

    /// Grant GDPR consent for campaign participation
    pub fn grant_consent(&mut self) -> Result<(), String> {
        if self.unsubscribed_at.is_some() {
            return Err("Cannot grant consent to unsubscribed member".to_string());
        }
        self.has_gdpr_consent = true;
        self.consent_at = Some(Utc::now());
        Ok(())
    }

    /// Update consumption data
    pub fn update_consumption(
        &mut self,
        kwh: f64,
        provider: Option<String>,
        ean: Option<String>,
    ) -> Result<(), String> {
        if kwh < 0.0 {
            return Err("Annual consumption cannot be negative".to_string());
        }
        self.annual_consumption_kwh = Some(kwh);
        self.current_provider = provider;
        self.ean_code = ean;
        Ok(())
    }

    /// Unsubscribe member from campaign (GDPR right to erasure prep)
    pub fn unsubscribe(&mut self) -> Result<(), String> {
        if self.unsubscribed_at.is_some() {
            return Err("Member already unsubscribed".to_string());
        }
        self.unsubscribed_at = Some(Utc::now());
        Ok(())
    }

    /// Check if member is active (not unsubscribed)
    pub fn is_active(&self) -> bool {
        self.unsubscribed_at.is_none()
    }

    /// Check if member has valid consent
    pub fn has_valid_consent(&self) -> bool {
        self.has_gdpr_consent && self.is_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_individual_member_new_success() {
        let campaign_id = Uuid::new_v4();
        let member = IndividualMember::new(
            campaign_id,
            "user@example.com".to_string(),
            "1000".to_string(),
        );

        assert!(member.is_ok());
        let m = member.unwrap();
        assert_eq!(m.email, "user@example.com");
        assert_eq!(m.postal_code, "1000");
        assert!(!m.has_gdpr_consent);
        assert!(m.is_active());
    }

    #[test]
    fn test_individual_member_invalid_email() {
        let campaign_id = Uuid::new_v4();
        let result = IndividualMember::new(campaign_id, "invalid".to_string(), "1000".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_individual_member_empty_postal_code() {
        let campaign_id = Uuid::new_v4();
        let result =
            IndividualMember::new(campaign_id, "user@example.com".to_string(), "".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_grant_consent() {
        let campaign_id = Uuid::new_v4();
        let mut member = IndividualMember::new(
            campaign_id,
            "user@example.com".to_string(),
            "1000".to_string(),
        )
        .unwrap();

        assert!(!member.has_gdpr_consent);
        let _ = member.grant_consent();
        assert!(member.has_gdpr_consent);
        assert!(member.consent_at.is_some());
    }

    #[test]
    fn test_unsubscribe() {
        let campaign_id = Uuid::new_v4();
        let mut member = IndividualMember::new(
            campaign_id,
            "user@example.com".to_string(),
            "1000".to_string(),
        )
        .unwrap();

        assert!(member.is_active());
        let _ = member.unsubscribe();
        assert!(!member.is_active());
        assert!(member.unsubscribed_at.is_some());
    }

    #[test]
    fn test_has_valid_consent() {
        let campaign_id = Uuid::new_v4();
        let mut member = IndividualMember::new(
            campaign_id,
            "user@example.com".to_string(),
            "1000".to_string(),
        )
        .unwrap();

        assert!(!member.has_valid_consent());
        let _ = member.grant_consent();
        assert!(member.has_valid_consent());
        let _ = member.unsubscribe();
        assert!(!member.has_valid_consent());
    }

    #[test]
    fn test_update_consumption() {
        let campaign_id = Uuid::new_v4();
        let mut member = IndividualMember::new(
            campaign_id,
            "user@example.com".to_string(),
            "1000".to_string(),
        )
        .unwrap();

        let result = member.update_consumption(
            5000.0,
            Some("Engie".to_string()),
            Some("501234567890".to_string()),
        );
        assert!(result.is_ok());
        assert_eq!(member.annual_consumption_kwh, Some(5000.0));
        assert_eq!(member.current_provider, Some("Engie".to_string()));
    }

    #[test]
    fn test_update_consumption_negative() {
        let campaign_id = Uuid::new_v4();
        let mut member = IndividualMember::new(
            campaign_id,
            "user@example.com".to_string(),
            "1000".to_string(),
        )
        .unwrap();

        let result = member.update_consumption(-100.0, None, None);
        assert!(result.is_err());
    }
}
