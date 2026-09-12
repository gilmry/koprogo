use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GDPR Article 16 - Right to Rectification
///
/// Represents a user's request to correct inaccurate personal data.
/// This entity tracks which fields need correction and their new values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GdprRectificationRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub requested_at: DateTime<Utc>,
    pub status: RectificationStatus,
    pub changes: Vec<FieldChange>,
    pub reason: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub processed_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RectificationStatus {
    Pending,
    Approved,
    Rejected,
    Applied,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldChange {
    pub entity_type: String, // "User", "Owner", etc.
    pub entity_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: String,
}

impl GdprRectificationRequest {
    /// Create a new rectification request
    pub fn new(
        user_id: Uuid,
        organization_id: Option<Uuid>,
        changes: Vec<FieldChange>,
        reason: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            organization_id,
            requested_at: Utc::now(),
            status: RectificationStatus::Pending,
            changes,
            reason,
            processed_at: None,
            processed_by: None,
        }
    }

    /// Approve the rectification request
    pub fn approve(&mut self, admin_id: Uuid) {
        self.status = RectificationStatus::Approved;
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(admin_id);
    }

    /// Reject the rectification request
    pub fn reject(&mut self, admin_id: Uuid) {
        self.status = RectificationStatus::Rejected;
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(admin_id);
    }

    /// Mark as applied after changes are made
    pub fn mark_applied(&mut self) {
        self.status = RectificationStatus::Applied;
    }

    /// Check if request is still pending
    pub fn is_pending(&self) -> bool {
        matches!(self.status, RectificationStatus::Pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rectification_request() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let changes = vec![FieldChange {
            entity_type: "User".to_string(),
            entity_id: user_id,
            field_name: "email".to_string(),
            old_value: Some("old@example.com".to_string()),
            new_value: "new@example.com".to_string(),
        }];

        let request = GdprRectificationRequest::new(
            user_id,
            Some(org_id),
            changes.clone(),
            Some("Email address was incorrect".to_string()),
        );

        assert_eq!(request.user_id, user_id);
        assert_eq!(request.organization_id, Some(org_id));
        assert!(request.is_pending());
        assert_eq!(request.changes.len(), 1);
        assert_eq!(request.changes[0].new_value, "new@example.com");
    }

    #[test]
    fn test_approve_request() {
        let user_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();
        let changes = vec![FieldChange {
            entity_type: "User".to_string(),
            entity_id: user_id,
            field_name: "first_name".to_string(),
            old_value: Some("Jon".to_string()),
            new_value: "John".to_string(),
        }];

        let mut request = GdprRectificationRequest::new(user_id, None, changes, None);
        request.approve(admin_id);

        assert_eq!(request.status, RectificationStatus::Approved);
        assert!(request.processed_at.is_some());
        assert_eq!(request.processed_by, Some(admin_id));
        assert!(!request.is_pending());
    }

    #[test]
    fn test_reject_request() {
        let user_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();
        let changes = vec![FieldChange {
            entity_type: "User".to_string(),
            entity_id: user_id,
            field_name: "email".to_string(),
            old_value: Some("old@example.com".to_string()),
            new_value: "invalid-email".to_string(),
        }];

        let mut request = GdprRectificationRequest::new(user_id, None, changes, None);
        request.reject(admin_id);

        assert_eq!(request.status, RectificationStatus::Rejected);
        assert!(request.processed_at.is_some());
        assert_eq!(request.processed_by, Some(admin_id));
    }

    #[test]
    fn test_mark_applied() {
        let user_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();
        let changes = vec![FieldChange {
            entity_type: "User".to_string(),
            entity_id: user_id,
            field_name: "last_name".to_string(),
            old_value: Some("Smith".to_string()),
            new_value: "Smyth".to_string(),
        }];

        let mut request = GdprRectificationRequest::new(user_id, None, changes, None);
        request.approve(admin_id);
        request.mark_applied();

        assert_eq!(request.status, RectificationStatus::Applied);
    }
}
