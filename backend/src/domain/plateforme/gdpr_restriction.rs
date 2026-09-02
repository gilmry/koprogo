use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GDPR Article 18 - Right to Restriction of Processing
///
/// Represents a user's request to temporarily restrict processing of their personal data.
/// During restriction, data can be stored but not processed (except with user consent or for legal claims).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GdprRestrictionRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub requested_at: DateTime<Utc>,
    pub status: RestrictionStatus,
    pub reason: RestrictionReason,
    pub justification: Option<String>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
    pub processed_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RestrictionStatus {
    Pending,
    Active,  // Restriction is in effect
    Lifted,  // Restriction was removed
    Expired, // Restriction period ended
    Rejected,
}

/// Grounds for restriction under Article 18(1)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RestrictionReason {
    /// (a) accuracy of the data is contested by the data subject
    AccuracyContested,
    /// (b) processing is unlawful and data subject opposes erasure
    UnlawfulProcessing,
    /// (c) controller no longer needs data but data subject needs it for legal claims
    LegalClaims,
    /// (d) data subject has objected to processing pending verification
    ObjectionPending,
}

impl GdprRestrictionRequest {
    /// Create a new restriction request
    pub fn new(
        user_id: Uuid,
        organization_id: Option<Uuid>,
        reason: RestrictionReason,
        justification: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            organization_id,
            requested_at: Utc::now(),
            status: RestrictionStatus::Pending,
            reason,
            justification,
            effective_from: None,
            effective_until: None,
            processed_at: None,
            processed_by: None,
        }
    }

    /// Activate the restriction
    pub fn activate(&mut self, admin_id: Uuid, duration_days: Option<u32>) {
        let now = Utc::now();
        self.status = RestrictionStatus::Active;
        self.effective_from = Some(now);
        self.effective_until = duration_days.map(|days| now + chrono::Duration::days(days as i64));
        self.processed_at = Some(now);
        self.processed_by = Some(admin_id);
    }

    /// Lift the restriction
    pub fn lift(&mut self, admin_id: Uuid) {
        self.status = RestrictionStatus::Lifted;
        self.effective_until = Some(Utc::now());
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(admin_id);
    }

    /// Reject the restriction request
    pub fn reject(&mut self, admin_id: Uuid) {
        self.status = RestrictionStatus::Rejected;
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(admin_id);
    }

    /// Check if restriction is currently active
    pub fn is_active(&self) -> bool {
        if self.status != RestrictionStatus::Active {
            return false;
        }

        let now = Utc::now();

        // Check if started
        if let Some(from) = self.effective_from {
            if now < from {
                return false;
            }
        }

        // Check if expired
        if let Some(until) = self.effective_until {
            if now > until {
                return false;
            }
        }

        true
    }

    /// Check if restriction is pending
    pub fn is_pending(&self) -> bool {
        matches!(self.status, RestrictionStatus::Pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_restriction_request() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let request = GdprRestrictionRequest::new(
            user_id,
            Some(org_id),
            RestrictionReason::AccuracyContested,
            Some("My email address is incorrect".to_string()),
        );

        assert_eq!(request.user_id, user_id);
        assert_eq!(request.organization_id, Some(org_id));
        assert!(request.is_pending());
        assert_eq!(request.reason, RestrictionReason::AccuracyContested);
    }

    #[test]
    fn test_activate_restriction_without_duration() {
        let user_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();

        let mut request =
            GdprRestrictionRequest::new(user_id, None, RestrictionReason::ObjectionPending, None);

        request.activate(admin_id, None);

        assert_eq!(request.status, RestrictionStatus::Active);
        assert!(request.effective_from.is_some());
        assert!(request.effective_until.is_none());
        assert!(request.is_active());
    }

    #[test]
    fn test_activate_restriction_with_duration() {
        let user_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();

        let mut request =
            GdprRestrictionRequest::new(user_id, None, RestrictionReason::AccuracyContested, None);

        request.activate(admin_id, Some(30)); // 30 days

        assert_eq!(request.status, RestrictionStatus::Active);
        assert!(request.effective_from.is_some());
        assert!(request.effective_until.is_some());
        assert!(request.is_active());

        // Check duration is approximately 30 days
        let from = request.effective_from.unwrap();
        let until = request.effective_until.unwrap();
        let duration = until - from;
        assert!((duration.num_days() - 30).abs() < 1);
    }

    #[test]
    fn test_lift_restriction() {
        let user_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();

        let mut request =
            GdprRestrictionRequest::new(user_id, None, RestrictionReason::UnlawfulProcessing, None);

        request.activate(admin_id, None);
        assert!(request.is_active());

        request.lift(admin_id);
        assert_eq!(request.status, RestrictionStatus::Lifted);
        assert!(!request.is_active());
    }

    #[test]
    fn test_reject_restriction() {
        let user_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();

        let mut request =
            GdprRestrictionRequest::new(user_id, None, RestrictionReason::LegalClaims, None);

        request.reject(admin_id);

        assert_eq!(request.status, RestrictionStatus::Rejected);
        assert!(!request.is_active());
        assert!(!request.is_pending());
    }
}
