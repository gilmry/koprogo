use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Sévérité d'un incident de sécurité (GDPR Art. 33)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for IncidentSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IncidentSeverity::Critical => write!(f, "critical"),
            IncidentSeverity::High => write!(f, "high"),
            IncidentSeverity::Medium => write!(f, "medium"),
            IncidentSeverity::Low => write!(f, "low"),
        }
    }
}

impl std::str::FromStr for IncidentSeverity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "critical" => Ok(IncidentSeverity::Critical),
            "high" => Ok(IncidentSeverity::High),
            "medium" => Ok(IncidentSeverity::Medium),
            "low" => Ok(IncidentSeverity::Low),
            _ => Err(format!("Invalid severity: {}", s)),
        }
    }
}

/// Statut d'un incident de sécurité
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentStatus {
    Detected,
    Investigating,
    Contained,
    Reported,  // Notifié à l'APD (Art. 33 GDPR — délai 72h)
    Closed,
}

impl std::fmt::Display for IncidentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IncidentStatus::Detected => write!(f, "detected"),
            IncidentStatus::Investigating => write!(f, "investigating"),
            IncidentStatus::Contained => write!(f, "contained"),
            IncidentStatus::Reported => write!(f, "reported"),
            IncidentStatus::Closed => write!(f, "closed"),
        }
    }
}

impl std::str::FromStr for IncidentStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "detected" => Ok(IncidentStatus::Detected),
            "investigating" => Ok(IncidentStatus::Investigating),
            "contained" => Ok(IncidentStatus::Contained),
            "reported" => Ok(IncidentStatus::Reported),
            "closed" => Ok(IncidentStatus::Closed),
            _ => Err(format!("Invalid incident status: {}", s)),
        }
    }
}

/// Incident de sécurité (GDPR Art. 33 — notification APD dans les 72h)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub severity: String,
    pub incident_type: String,
    pub title: String,
    pub description: String,
    pub data_categories_affected: Vec<String>,
    pub affected_subjects_count: Option<i32>,
    pub discovery_at: DateTime<Utc>,
    pub notification_at: Option<DateTime<Utc>>,
    pub apd_reference_number: Option<String>,
    pub status: String,
    pub reported_by: Uuid,
    pub investigation_notes: Option<String>,
    pub root_cause: Option<String>,
    pub remediation_steps: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SecurityIncident {
    pub fn new(
        organization_id: Option<Uuid>,
        reported_by: Uuid,
        severity: String,
        incident_type: String,
        title: String,
        description: String,
        data_categories_affected: Vec<String>,
        affected_subjects_count: Option<i32>,
    ) -> Result<Self, String> {
        if title.is_empty() {
            return Err("title is required".to_string());
        }
        if description.is_empty() {
            return Err("description is required".to_string());
        }
        severity.parse::<IncidentSeverity>()?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            severity,
            incident_type,
            title,
            description,
            data_categories_affected,
            affected_subjects_count,
            discovery_at: now,
            notification_at: None,
            apd_reference_number: None,
            status: IncidentStatus::Detected.to_string(),
            reported_by,
            investigation_notes: None,
            root_cause: None,
            remediation_steps: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Heures depuis la découverte (délai APD Art. 33 GDPR = 72h)
    pub fn hours_since_discovery(&self) -> f64 {
        let duration = Utc::now().signed_duration_since(self.discovery_at);
        duration.num_seconds() as f64 / 3600.0
    }

    /// Vrai si l'incident dépasse 72h sans notification APD
    pub fn is_overdue_for_apd(&self) -> bool {
        self.notification_at.is_none() && self.hours_since_discovery() > 72.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_incident_valid() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let incident = SecurityIncident::new(
            Some(org_id),
            user_id,
            "high".to_string(),
            "data_breach".to_string(),
            "Test incident".to_string(),
            "Description".to_string(),
            vec!["email".to_string()],
            Some(10),
        );
        assert!(incident.is_ok());
        let inc = incident.unwrap();
        assert_eq!(inc.status, "detected");
        assert!(inc.notification_at.is_none());
    }

    #[test]
    fn test_new_incident_empty_title() {
        let result = SecurityIncident::new(
            None,
            Uuid::new_v4(),
            "low".to_string(),
            "unauthorized_access".to_string(),
            "".to_string(),
            "desc".to_string(),
            vec![],
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("title"));
    }

    #[test]
    fn test_new_incident_invalid_severity() {
        let result = SecurityIncident::new(
            None,
            Uuid::new_v4(),
            "extreme".to_string(),
            "malware".to_string(),
            "title".to_string(),
            "desc".to_string(),
            vec![],
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_hours_since_discovery() {
        let org_id = Uuid::new_v4();
        let incident = SecurityIncident::new(
            Some(org_id),
            Uuid::new_v4(),
            "critical".to_string(),
            "data_breach".to_string(),
            "Test".to_string(),
            "Desc".to_string(),
            vec![],
            None,
        )
        .unwrap();
        let hours = incident.hours_since_discovery();
        assert!(hours >= 0.0 && hours < 0.1); // just created
    }

    #[test]
    fn test_is_overdue_for_apd_new_incident() {
        let incident = SecurityIncident::new(
            None,
            Uuid::new_v4(),
            "high".to_string(),
            "breach".to_string(),
            "title".to_string(),
            "desc".to_string(),
            vec![],
            None,
        )
        .unwrap();
        // New incident is not overdue yet
        assert!(!incident.is_overdue_for_apd());
    }

    #[test]
    fn test_severity_parse() {
        assert!("critical".parse::<IncidentSeverity>().is_ok());
        assert!("high".parse::<IncidentSeverity>().is_ok());
        assert!("invalid".parse::<IncidentSeverity>().is_err());
    }

    #[test]
    fn test_status_display() {
        assert_eq!(IncidentStatus::Detected.to_string(), "detected");
        assert_eq!(IncidentStatus::Reported.to_string(), "reported");
    }
}
