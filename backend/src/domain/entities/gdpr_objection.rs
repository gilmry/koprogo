use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GDPR Article 21 - Right to Object
///
/// Represents a user's objection to processing of their personal data
/// for specific purposes (marketing, profiling, legitimate interests).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GdprObjectionRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub requested_at: DateTime<Utc>,
    pub status: ObjectionStatus,
    pub objection_type: ObjectionType,
    pub processing_purposes: Vec<ProcessingPurpose>,
    pub justification: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub processed_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObjectionStatus {
    Pending,
    Accepted, // Objection upheld, processing stopped
    Rejected, // Objection rejected (compelling legitimate grounds)
    Partial,  // Some purposes accepted, others rejected
}

/// Types of objection under Article 21
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObjectionType {
    /// Article 21(1) - objection to processing based on legitimate interests
    LegitimateInterests,
    /// Article 21(2) - objection to direct marketing (absolute right)
    DirectMarketing,
    /// Article 21(3) - objection to profiling
    Profiling,
    /// Article 21(4) - objection to automated decision-making
    AutomatedDecisionMaking,
    /// Article 21(6) - objection to scientific/historical research
    ScientificResearch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessingPurpose {
    pub purpose: String,
    pub accepted: Option<bool>, // None = pending, Some(true) = accepted, Some(false) = rejected
}

impl GdprObjectionRequest {
    /// Create a new objection request
    pub fn new(
        user_id: Uuid,
        organization_id: Option<Uuid>,
        objection_type: ObjectionType,
        processing_purposes: Vec<String>,
        justification: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            organization_id,
            requested_at: Utc::now(),
            status: ObjectionStatus::Pending,
            objection_type,
            processing_purposes: processing_purposes
                .into_iter()
                .map(|purpose| ProcessingPurpose {
                    purpose,
                    accepted: None,
                })
                .collect(),
            justification,
            processed_at: None,
            processed_by: None,
        }
    }

    /// Accept the objection (stop all processing)
    pub fn accept(&mut self, admin_id: Uuid) {
        self.status = ObjectionStatus::Accepted;
        for purpose in &mut self.processing_purposes {
            purpose.accepted = Some(true);
        }
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(admin_id);
    }

    /// Reject the objection (continue processing)
    pub fn reject(&mut self, admin_id: Uuid) {
        self.status = ObjectionStatus::Rejected;
        for purpose in &mut self.processing_purposes {
            purpose.accepted = Some(false);
        }
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(admin_id);
    }

    /// Accept some purposes, reject others
    pub fn partial_accept(&mut self, admin_id: Uuid, accepted_purposes: Vec<String>) {
        self.status = ObjectionStatus::Partial;
        for purpose in &mut self.processing_purposes {
            purpose.accepted = Some(accepted_purposes.contains(&purpose.purpose));
        }
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(admin_id);
    }

    /// Check if this is a direct marketing objection (absolute right)
    pub fn is_marketing_objection(&self) -> bool {
        matches!(self.objection_type, ObjectionType::DirectMarketing)
    }

    /// Check if objection is pending
    pub fn is_pending(&self) -> bool {
        matches!(self.status, ObjectionStatus::Pending)
    }

    /// Get list of accepted objections
    pub fn get_accepted_purposes(&self) -> Vec<String> {
        self.processing_purposes
            .iter()
            .filter(|p| p.accepted == Some(true))
            .map(|p| p.purpose.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_objection_request() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let purposes = vec!["email_marketing".to_string(), "sms_marketing".to_string()];

        let request = GdprObjectionRequest::new(
            user_id,
            Some(org_id),
            ObjectionType::DirectMarketing,
            purposes.clone(),
            Some("I don't want to receive marketing emails".to_string()),
        );

        assert_eq!(request.user_id, user_id);
        assert_eq!(request.organization_id, Some(org_id));
        assert!(request.is_pending());
        assert!(request.is_marketing_objection());
        assert_eq!(request.processing_purposes.len(), 2);
    }

    #[test]
    fn test_accept_objection() {
        let user_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();
        let purposes = vec!["profiling".to_string()];

        let mut request =
            GdprObjectionRequest::new(user_id, None, ObjectionType::Profiling, purposes, None);

        request.accept(admin_id);

        assert_eq!(request.status, ObjectionStatus::Accepted);
        assert!(request
            .processing_purposes
            .iter()
            .all(|p| p.accepted == Some(true)));
        assert_eq!(request.get_accepted_purposes().len(), 1);
    }

    #[test]
    fn test_reject_objection() {
        let user_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();
        let purposes = vec!["legitimate_interest".to_string()];

        let mut request = GdprObjectionRequest::new(
            user_id,
            None,
            ObjectionType::LegitimateInterests,
            purposes,
            Some("Compelling legitimate grounds".to_string()),
        );

        request.reject(admin_id);

        assert_eq!(request.status, ObjectionStatus::Rejected);
        assert!(request
            .processing_purposes
            .iter()
            .all(|p| p.accepted == Some(false)));
        assert_eq!(request.get_accepted_purposes().len(), 0);
    }

    #[test]
    fn test_partial_accept() {
        let user_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();
        let purposes = vec![
            "email_marketing".to_string(),
            "analytics".to_string(),
            "sms_marketing".to_string(),
        ];

        let mut request = GdprObjectionRequest::new(
            user_id,
            None,
            ObjectionType::DirectMarketing,
            purposes,
            None,
        );

        request.partial_accept(
            admin_id,
            vec!["email_marketing".to_string(), "sms_marketing".to_string()],
        );

        assert_eq!(request.status, ObjectionStatus::Partial);
        assert_eq!(request.get_accepted_purposes().len(), 2);
        assert!(request
            .get_accepted_purposes()
            .contains(&"email_marketing".to_string()));
        assert!(!request
            .get_accepted_purposes()
            .contains(&"analytics".to_string()));
    }

    #[test]
    fn test_marketing_objection_detection() {
        let user_id = Uuid::new_v4();
        let purposes = vec!["marketing".to_string()];

        let marketing_request = GdprObjectionRequest::new(
            user_id,
            None,
            ObjectionType::DirectMarketing,
            purposes.clone(),
            None,
        );

        let profiling_request =
            GdprObjectionRequest::new(user_id, None, ObjectionType::Profiling, purposes, None);

        assert!(marketing_request.is_marketing_objection());
        assert!(!profiling_request.is_marketing_objection());
    }
}
