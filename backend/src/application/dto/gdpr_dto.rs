use crate::domain::entities::gdpr_export::{
    DocumentData, ExpenseData, GdprExport, MeetingData, OwnerData, UnitOwnershipData, UserData,
};
use serde::{Deserialize, Serialize};

/// Response DTO for GDPR data export (Article 15)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GdprExportResponseDto {
    pub export_date: String, // RFC3339 format
    pub user: UserDataDto,
    pub owners: Vec<OwnerDataDto>,
    pub units: Vec<UnitOwnershipDataDto>,
    pub expenses: Vec<ExpenseDataDto>,
    pub documents: Vec<DocumentDataDto>,
    pub meetings: Vec<MeetingDataDto>,
    pub total_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserDataDto {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub organization_id: Option<String>,
    pub is_active: bool,
    pub is_anonymized: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OwnerDataDto {
    pub id: String,
    pub organization_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub is_anonymized: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnitOwnershipDataDto {
    pub building_name: String,
    pub building_address: String,
    pub unit_number: String,
    pub floor: Option<i32>,
    pub ownership_percentage: f64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub is_primary_contact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpenseDataDto {
    pub description: String,
    pub amount: f64,
    pub due_date: String,
    pub paid: bool,
    pub building_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentDataDto {
    pub title: String,
    pub document_type: String,
    pub uploaded_at: String,
    pub building_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeetingDataDto {
    pub title: String,
    pub meeting_date: String,
    pub agenda: Option<String>,
    pub building_name: String,
}

impl From<GdprExport> for GdprExportResponseDto {
    fn from(export: GdprExport) -> Self {
        let total_items = export.total_items();
        Self {
            export_date: export.export_date.to_rfc3339(),
            user: UserDataDto::from(export.user_data),
            owners: export
                .owner_profiles
                .into_iter()
                .map(OwnerDataDto::from)
                .collect(),
            units: export
                .related_data
                .units
                .into_iter()
                .map(UnitOwnershipDataDto::from)
                .collect(),
            expenses: export
                .related_data
                .expenses
                .into_iter()
                .map(ExpenseDataDto::from)
                .collect(),
            documents: export
                .related_data
                .documents
                .into_iter()
                .map(DocumentDataDto::from)
                .collect(),
            meetings: export
                .related_data
                .meetings
                .into_iter()
                .map(MeetingDataDto::from)
                .collect(),
            total_items,
        }
    }
}

impl From<UserData> for UserDataDto {
    fn from(user: UserData) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            organization_id: user.organization_id.map(|id| id.to_string()),
            is_active: user.is_active,
            is_anonymized: user.is_anonymized,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

impl From<OwnerData> for OwnerDataDto {
    fn from(owner: OwnerData) -> Self {
        Self {
            id: owner.id.to_string(),
            organization_id: owner.organization_id.to_string(),
            first_name: owner.first_name,
            last_name: owner.last_name,
            email: owner.email,
            phone: owner.phone,
            address: owner.address,
            city: owner.city,
            postal_code: owner.postal_code,
            country: owner.country,
            is_anonymized: owner.is_anonymized,
            created_at: owner.created_at.to_rfc3339(),
            updated_at: owner.updated_at.to_rfc3339(),
        }
    }
}

impl From<UnitOwnershipData> for UnitOwnershipDataDto {
    fn from(unit: UnitOwnershipData) -> Self {
        Self {
            building_name: unit.building_name,
            building_address: unit.building_address,
            unit_number: unit.unit_number,
            floor: unit.floor,
            ownership_percentage: unit.ownership_percentage,
            start_date: unit.start_date.to_rfc3339(),
            end_date: unit.end_date.map(|d| d.to_rfc3339()),
            is_primary_contact: unit.is_primary_contact,
        }
    }
}

impl From<ExpenseData> for ExpenseDataDto {
    fn from(expense: ExpenseData) -> Self {
        Self {
            description: expense.description,
            amount: expense.amount,
            due_date: expense.due_date.to_rfc3339(),
            paid: expense.paid,
            building_name: expense.building_name,
        }
    }
}

impl From<DocumentData> for DocumentDataDto {
    fn from(document: DocumentData) -> Self {
        Self {
            title: document.title,
            document_type: document.document_type,
            uploaded_at: document.uploaded_at.to_rfc3339(),
            building_name: document.building_name,
        }
    }
}

impl From<MeetingData> for MeetingDataDto {
    fn from(meeting: MeetingData) -> Self {
        Self {
            title: meeting.title,
            meeting_date: meeting.meeting_date.to_rfc3339(),
            agenda: meeting.agenda,
            building_name: meeting.building_name,
        }
    }
}

/// Request DTO for GDPR data erasure (Article 17)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprEraseRequestDto {
    /// Optional confirmation token for security
    pub confirmation: Option<String>,
}

/// Response DTO for GDPR data erasure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprEraseResponseDto {
    pub success: bool,
    pub message: String,
    pub anonymized_at: String,
    pub user_id: String,
    pub owners_anonymized: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_gdpr_export() -> GdprExport {
        let user_data = UserData {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            organization_id: Some(Uuid::new_v4()),
            is_active: true,
            is_anonymized: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        GdprExport::new(user_data)
    }

    #[test]
    fn test_gdpr_export_response_dto_from_domain() {
        let export = create_test_gdpr_export();
        let user_email = export.user_data.email.clone();

        let dto = GdprExportResponseDto::from(export);

        assert_eq!(dto.user.email, user_email);
        assert_eq!(dto.total_items, 1); // Only user
        assert!(dto.export_date.contains('T')); // RFC3339 format
    }

    #[test]
    fn test_user_data_dto_conversion() {
        let user_data = UserData {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            organization_id: Some(Uuid::new_v4()),
            is_active: true,
            is_anonymized: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let dto = UserDataDto::from(user_data.clone());

        assert_eq!(dto.email, user_data.email);
        assert_eq!(dto.first_name, user_data.first_name);
        assert_eq!(dto.last_name, user_data.last_name);
        assert!(!dto.is_anonymized);
    }

    #[test]
    fn test_owner_data_dto_conversion() {
        let owner_data = OwnerData {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            email: Some("jane@example.com".to_string()),
            phone: Some("+1234567890".to_string()),
            address: Some("123 Main St".to_string()),
            city: Some("Brussels".to_string()),
            postal_code: Some("1000".to_string()),
            country: Some("Belgium".to_string()),
            is_anonymized: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let dto = OwnerDataDto::from(owner_data.clone());

        assert_eq!(dto.first_name, owner_data.first_name);
        assert_eq!(dto.email, owner_data.email);
        assert_eq!(dto.phone, owner_data.phone);
    }

    #[test]
    fn test_json_serialization() {
        let export = create_test_gdpr_export();
        let dto = GdprExportResponseDto::from(export);

        // Test JSON serialization
        let json = serde_json::to_string(&dto).expect("Should serialize to JSON");
        assert!(json.contains("export_date"));
        assert!(json.contains("test@example.com"));
        assert!(json.contains("total_items"));

        // Test JSON deserialization
        let deserialized: GdprExportResponseDto =
            serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(deserialized.user.email, dto.user.email);
    }

    #[test]
    fn test_erase_request_dto() {
        let request = GdprEraseRequestDto {
            confirmation: Some("CONFIRM_DELETE".to_string()),
        };

        let json = serde_json::to_string(&request).expect("Should serialize");
        assert!(json.contains("CONFIRM_DELETE"));
    }

    #[test]
    fn test_erase_response_dto() {
        let response = GdprEraseResponseDto {
            success: true,
            message: "Data successfully anonymized".to_string(),
            anonymized_at: Utc::now().to_rfc3339(),
            user_id: Uuid::new_v4().to_string(),
            owners_anonymized: 2,
        };

        let json = serde_json::to_string(&response).expect("Should serialize");
        assert!(json.contains("success"));
        assert!(json.contains("owners_anonymized"));
    }
}
