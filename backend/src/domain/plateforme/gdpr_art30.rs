use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Registre des activités de traitement — GDPR Art. 30 §1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingActivity {
    pub id: Uuid,
    pub activity_name: String,
    pub controller_name: String,
    pub purpose: String,
    pub legal_basis: String,
    pub data_categories: Vec<String>,
    pub data_subjects: Vec<String>,
    pub recipients: Vec<String>,
    pub retention_period: String,
    pub security_measures: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Accord sous-traitant (DPA) — GDPR Art. 28 + Art. 30 §2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorAgreement {
    pub id: Uuid,
    pub processor_name: String,
    pub service_description: String,
    pub dpa_signed_at: Option<DateTime<Utc>>,
    pub dpa_url: Option<String>,
    pub transfer_mechanism: Option<String>,
    pub data_categories: Vec<String>,
    pub certifications: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProcessorAgreement {
    pub fn has_signed_dpa(&self) -> bool {
        self.dpa_signed_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agreement(signed: bool) -> ProcessorAgreement {
        let now = Utc::now();
        ProcessorAgreement {
            id: Uuid::new_v4(),
            processor_name: "AWS".to_string(),
            service_description: "Cloud hosting".to_string(),
            dpa_signed_at: if signed { Some(now) } else { None },
            dpa_url: Some("https://aws.amazon.com/dpa".to_string()),
            transfer_mechanism: Some("SCCs".to_string()),
            data_categories: vec!["personal_data".to_string()],
            certifications: Some(vec!["ISO27001".to_string()]),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_has_signed_dpa_true() {
        assert!(make_agreement(true).has_signed_dpa());
    }

    #[test]
    fn test_has_signed_dpa_false() {
        assert!(!make_agreement(false).has_signed_dpa());
    }

    #[test]
    fn test_processing_activity_fields() {
        let now = Utc::now();
        let act = ProcessingActivity {
            id: Uuid::new_v4(),
            activity_name: "User management".to_string(),
            controller_name: "KoproGo ASBL".to_string(),
            purpose: "Contract performance".to_string(),
            legal_basis: "Art. 6(1)(b) GDPR".to_string(),
            data_categories: vec!["contact".to_string(), "financial".to_string()],
            data_subjects: vec!["owners".to_string()],
            recipients: vec!["syndic".to_string()],
            retention_period: "7 years".to_string(),
            security_measures: "Encryption at rest, TLS".to_string(),
            created_at: now,
            updated_at: now,
        };
        assert_eq!(act.data_categories.len(), 2);
        assert_eq!(act.legal_basis, "Art. 6(1)(b) GDPR");
    }
}
