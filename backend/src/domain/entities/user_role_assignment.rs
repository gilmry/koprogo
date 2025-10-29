use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::user::UserRole;

/// Represents an assignment of a role to a user within an optional organization scope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserRoleAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: UserRole,
    pub organization_id: Option<Uuid>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserRoleAssignment {
    /// Creates a new role assignment. Primary flag indicates the active role for the user.
    pub fn new(
        user_id: Uuid,
        role: UserRole,
        organization_id: Option<Uuid>,
        is_primary: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            role,
            organization_id,
            is_primary,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set_primary(&mut self, primary: bool) {
        self.is_primary = primary;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_assignment_defaults() {
        let user_id = Uuid::new_v4();
        let assignment = UserRoleAssignment::new(user_id, UserRole::Syndic, None, true);

        assert_eq!(assignment.user_id, user_id);
        assert_eq!(assignment.role, UserRole::Syndic);
        assert!(assignment.is_primary);
        assert!(assignment.organization_id.is_none());
    }

    #[test]
    fn test_set_primary_updates_timestamp() {
        let mut assignment =
            UserRoleAssignment::new(Uuid::new_v4(), UserRole::Accountant, None, false);
        let original_updated_at = assignment.updated_at;

        assignment.set_primary(true);

        assert!(assignment.is_primary);
        assert!(
            assignment.updated_at > original_updated_at,
            "Updated_at should change when toggling primary flag"
        );
    }
}
