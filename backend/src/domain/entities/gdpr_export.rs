use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Complete GDPR data export for a user
/// Aggregates all personal data for GDPR Article 15 (Right to Access) compliance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GdprExport {
    pub export_date: DateTime<Utc>,
    pub user_data: UserData,
    pub owner_profiles: Vec<OwnerData>,
    pub related_data: RelatedData,
}

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserData {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub organization_id: Option<Uuid>,
    pub is_active: bool,
    pub is_anonymized: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Owner profile information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OwnerData {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub is_anonymized: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Related data (units, expenses, documents, meetings)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RelatedData {
    pub units: Vec<UnitOwnershipData>,
    pub expenses: Vec<ExpenseData>,
    pub documents: Vec<DocumentData>,
    pub meetings: Vec<MeetingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnitOwnershipData {
    pub building_name: String,
    pub building_address: String,
    pub unit_number: String,
    pub floor: Option<i32>,
    pub ownership_percentage: f64,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub is_primary_contact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpenseData {
    pub description: String,
    pub amount: f64,
    pub due_date: DateTime<Utc>,
    pub paid: bool,
    pub building_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentData {
    pub title: String,
    pub document_type: String,
    pub uploaded_at: DateTime<Utc>,
    pub building_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeetingData {
    pub title: String,
    pub meeting_date: DateTime<Utc>,
    pub agenda: Option<String>,
    pub building_name: String,
}

impl GdprExport {
    /// Create a new GDPR export
    pub fn new(user_data: UserData) -> Self {
        Self {
            export_date: Utc::now(),
            user_data,
            owner_profiles: Vec::new(),
            related_data: RelatedData::default(),
        }
    }

    /// Add owner profile to export
    pub fn add_owner_profile(&mut self, owner: OwnerData) {
        self.owner_profiles.push(owner);
    }

    /// Add unit ownership data
    pub fn add_unit_ownership(&mut self, unit: UnitOwnershipData) {
        self.related_data.units.push(unit);
    }

    /// Add expense data
    pub fn add_expense(&mut self, expense: ExpenseData) {
        self.related_data.expenses.push(expense);
    }

    /// Add document data
    pub fn add_document(&mut self, document: DocumentData) {
        self.related_data.documents.push(document);
    }

    /// Add meeting data
    pub fn add_meeting(&mut self, meeting: MeetingData) {
        self.related_data.meetings.push(meeting);
    }

    /// Check if user data is anonymized
    pub fn is_anonymized(&self) -> bool {
        self.user_data.is_anonymized
    }

    /// Get total number of data items
    pub fn total_items(&self) -> usize {
        1 // user
            + self.owner_profiles.len()
            + self.related_data.units.len()
            + self.related_data.expenses.len()
            + self.related_data.documents.len()
            + self.related_data.meetings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user_data() -> UserData {
        UserData {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            organization_id: Some(Uuid::new_v4()),
            is_active: true,
            is_anonymized: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_owner_data() -> OwnerData {
        OwnerData {
            id: Uuid::new_v4(),
            organization_id: Some(Uuid::new_v4()),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: Some("john.doe@example.com".to_string()),
            phone: Some("+1234567890".to_string()),
            address: Some("123 Main St".to_string()),
            city: Some("Brussels".to_string()),
            postal_code: Some("1000".to_string()),
            country: Some("Belgium".to_string()),
            is_anonymized: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_create_gdpr_export() {
        let user_data = create_test_user_data();
        let export = GdprExport::new(user_data.clone());

        assert_eq!(export.user_data, user_data);
        assert_eq!(export.owner_profiles.len(), 0);
        assert_eq!(export.related_data.units.len(), 0);
        assert!(!export.is_anonymized());
    }

    #[test]
    fn test_add_owner_profile() {
        let user_data = create_test_user_data();
        let mut export = GdprExport::new(user_data);
        let owner = create_test_owner_data();

        export.add_owner_profile(owner.clone());

        assert_eq!(export.owner_profiles.len(), 1);
        assert_eq!(export.owner_profiles[0], owner);
    }

    #[test]
    fn test_add_unit_ownership() {
        let user_data = create_test_user_data();
        let mut export = GdprExport::new(user_data);
        let unit = UnitOwnershipData {
            building_name: "Building A".to_string(),
            building_address: "123 Main St".to_string(),
            unit_number: "101".to_string(),
            floor: Some(1),
            ownership_percentage: 50.0,
            start_date: Utc::now(),
            end_date: None,
            is_primary_contact: true,
        };

        export.add_unit_ownership(unit.clone());

        assert_eq!(export.related_data.units.len(), 1);
        assert_eq!(export.related_data.units[0], unit);
    }

    #[test]
    fn test_add_expense() {
        let user_data = create_test_user_data();
        let mut export = GdprExport::new(user_data);
        let expense = ExpenseData {
            description: "Monthly maintenance".to_string(),
            amount: 100.0,
            due_date: Utc::now(),
            paid: true,
            building_name: "Building A".to_string(),
        };

        export.add_expense(expense.clone());

        assert_eq!(export.related_data.expenses.len(), 1);
        assert_eq!(export.related_data.expenses[0], expense);
    }

    #[test]
    fn test_add_document() {
        let user_data = create_test_user_data();
        let mut export = GdprExport::new(user_data);
        let document = DocumentData {
            title: "Meeting Minutes".to_string(),
            document_type: "PDF".to_string(),
            uploaded_at: Utc::now(),
            building_name: Some("Building A".to_string()),
        };

        export.add_document(document.clone());

        assert_eq!(export.related_data.documents.len(), 1);
        assert_eq!(export.related_data.documents[0], document);
    }

    #[test]
    fn test_add_meeting() {
        let user_data = create_test_user_data();
        let mut export = GdprExport::new(user_data);
        let meeting = MeetingData {
            title: "Annual General Meeting".to_string(),
            meeting_date: Utc::now(),
            agenda: Some("Budget approval".to_string()),
            building_name: "Building A".to_string(),
        };

        export.add_meeting(meeting.clone());

        assert_eq!(export.related_data.meetings.len(), 1);
        assert_eq!(export.related_data.meetings[0], meeting);
    }

    #[test]
    fn test_is_anonymized() {
        let mut user_data = create_test_user_data();
        user_data.is_anonymized = true;
        let export = GdprExport::new(user_data);

        assert!(export.is_anonymized());
    }

    #[test]
    fn test_total_items() {
        let user_data = create_test_user_data();
        let mut export = GdprExport::new(user_data);

        // Initially 1 (user only)
        assert_eq!(export.total_items(), 1);

        export.add_owner_profile(create_test_owner_data());
        assert_eq!(export.total_items(), 2);

        export.add_unit_ownership(UnitOwnershipData {
            building_name: "Building A".to_string(),
            building_address: "123 Main St".to_string(),
            unit_number: "101".to_string(),
            floor: Some(1),
            ownership_percentage: 50.0,
            start_date: Utc::now(),
            end_date: None,
            is_primary_contact: true,
        });
        assert_eq!(export.total_items(), 3);
    }

    #[test]
    fn test_serialization() {
        let user_data = create_test_user_data();
        let export = GdprExport::new(user_data);

        // Test JSON serialization
        let json = serde_json::to_string(&export).expect("Should serialize to JSON");
        assert!(json.contains("export_date"));
        assert!(json.contains("user_data"));
        assert!(json.contains("test@example.com"));

        // Test JSON deserialization
        let deserialized: GdprExport =
            serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(deserialized.user_data.email, export.user_data.email);
    }
}
