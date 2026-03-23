use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type d'assemblée générale
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MeetingType {
    Ordinary,      // Assemblée Générale Ordinaire (AGO)
    Extraordinary, // Assemblée Générale Extraordinaire (AGE)
}

/// Statut de l'assemblée
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MeetingStatus {
    Scheduled,
    Completed,
    Cancelled,
}

/// Représente une assemblée générale de copropriétaires
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Meeting {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub meeting_type: MeetingType,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_date: DateTime<Utc>,
    pub location: String,
    pub status: MeetingStatus,
    pub agenda: Vec<String>,
    pub attendees_count: Option<i32>,
    // Quorum — Art. 3.87 §5 CC : AG valide si >50% des quotes-parts présentes/représentées
    pub quorum_validated: bool,
    pub quorum_percentage: Option<f64>, // % des quotes-parts présentes/représentées (0.0-100.0)
    pub total_quotas: Option<f64>,      // Total millièmes du bâtiment (généralement 1000)
    pub present_quotas: Option<f64>,    // Millièmes présents + représentés par procuration
    // Second Convocation — Issue #311 (Art. 3.87 §5 CC: No quorum required for 2nd convocation)
    pub is_second_convocation: bool,  // true = 2e convocation (no quorum check needed)
    // PV Distribution — Issue #313: Track when AG minutes are sent to owners
    pub minutes_document_id: Option<Uuid>,  // FK to Document
    pub minutes_sent_at: Option<DateTime<Utc>>,  // When PV was distributed
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Meeting {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        meeting_type: MeetingType,
        title: String,
        description: Option<String>,
        scheduled_date: DateTime<Utc>,
        location: String,
    ) -> Result<Self, String> {
        if title.is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if location.is_empty() {
            return Err("Location cannot be empty".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            meeting_type,
            title,
            description,
            scheduled_date,
            location,
            status: MeetingStatus::Scheduled,
            agenda: Vec::new(),
            attendees_count: None,
            quorum_validated: false,
            quorum_percentage: None,
            total_quotas: None,
            present_quotas: None,
            is_second_convocation: false,  // Default: first convocation
            minutes_document_id: None,
            minutes_sent_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn add_agenda_item(&mut self, item: String) -> Result<(), String> {
        if item.is_empty() {
            return Err("Agenda item cannot be empty".to_string());
        }
        self.agenda.push(item);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn complete(&mut self, attendees_count: i32) -> Result<(), String> {
        match self.status {
            MeetingStatus::Scheduled => {
                self.status = MeetingStatus::Completed;
                self.attendees_count = Some(attendees_count);
                self.updated_at = Utc::now();
                Ok(())
            }
            MeetingStatus::Completed => Err("Meeting is already completed".to_string()),
            MeetingStatus::Cancelled => Err("Cannot complete a cancelled meeting".to_string()),
        }
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        match self.status {
            MeetingStatus::Scheduled => {
                self.status = MeetingStatus::Cancelled;
                self.updated_at = Utc::now();
                Ok(())
            }
            MeetingStatus::Completed => Err("Cannot cancel a completed meeting".to_string()),
            MeetingStatus::Cancelled => Err("Meeting is already cancelled".to_string()),
        }
    }

    pub fn reschedule(&mut self, new_date: DateTime<Utc>) -> Result<(), String> {
        match self.status {
            MeetingStatus::Scheduled | MeetingStatus::Cancelled => {
                self.scheduled_date = new_date;
                self.status = MeetingStatus::Scheduled;
                self.updated_at = Utc::now();
                Ok(())
            }
            MeetingStatus::Completed => Err("Cannot reschedule a completed meeting".to_string()),
        }
    }

    pub fn is_upcoming(&self) -> bool {
        self.status == MeetingStatus::Scheduled && self.scheduled_date > Utc::now()
    }

    /// Valide le quorum de l'AG (Art. 3.87 §5 CC).
    /// Quorum atteint si les quotes-parts présentes/représentées dépassent 50% du total.
    /// Retourne Ok(true) si quorum atteint, Ok(false) si insuffisant (2e convocation requise).
    pub fn validate_quorum(
        &mut self,
        present_quotas: f64,
        total_quotas: f64,
    ) -> Result<bool, String> {
        if total_quotas <= 0.0 {
            return Err("Total quotas must be positive".to_string());
        }
        if present_quotas < 0.0 {
            return Err("Present quotas cannot be negative".to_string());
        }
        if present_quotas > total_quotas {
            return Err("Present quotas cannot exceed total quotas".to_string());
        }

        let percentage = (present_quotas / total_quotas) * 100.0;
        // Quorum : >50% des quotes-parts (Art. 3.87 §5 — majorité stricte)
        let quorum_reached = percentage > 50.0;

        self.present_quotas = Some(present_quotas);
        self.total_quotas = Some(total_quotas);
        self.quorum_percentage = Some(percentage);
        self.quorum_validated = quorum_reached;
        self.updated_at = Utc::now();

        Ok(quorum_reached)
    }

    /// Vérifie si le quorum est atteint avant d'autoriser un vote.
    /// Retourne Err si le quorum n'a pas encore été validé ou n'est pas atteint.
    ///
    /// EXCEPTION (Art. 3.87 §5 CC): No quorum check required for second convocation (is_second_convocation = true).
    /// Belgian law: 2e convocation = voting allowed without quorum requirement.
    pub fn check_quorum_for_voting(&self) -> Result<(), String> {
        // Art. 3.87 §5 CC: No quorum check needed for 2nd convocation
        if self.is_second_convocation {
            return Ok(());
        }

        if self.quorum_percentage.is_none() {
            return Err("Quorum has not been validated yet (Art. 3.87 §5 CC)".to_string());
        }
        if !self.quorum_validated {
            let pct = self.quorum_percentage.unwrap_or(0.0);
            return Err(format!(
                "Quorum not reached: {:.1}% present (>50% required, Art. 3.87 §5 CC). \
                 A second convocation is required.",
                pct
            ));
        }
        Ok(())
    }

    /// Sets minutes as sent (Issue #313: PV distribution tracking).
    /// Can only be called once meeting is Completed.
    pub fn set_minutes_sent(&mut self, document_id: Uuid) -> Result<(), String> {
        if self.status != MeetingStatus::Completed {
            return Err("Minutes can only be sent after meeting is completed".to_string());
        }
        self.minutes_document_id = Some(document_id);
        self.minutes_sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Checks if minutes are overdue (Issue #313: 30 days after meeting completion).
    /// Returns true if meeting is Completed, minutes not yet sent, and >30 days have passed.
    pub fn is_minutes_overdue(&self) -> bool {
        if self.status != MeetingStatus::Completed {
            return false;
        }
        if self.minutes_sent_at.is_some() {
            return false;
        }
        // Minutes are overdue if more than 30 days have passed since completion/update
        let deadline = self.updated_at + Duration::days(30);
        Utc::now() > deadline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_create_meeting_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            Some("Assemblée générale ordinaire annuelle".to_string()),
            future_date,
            "Salle des fêtes".to_string(),
        );

        assert!(meeting.is_ok());
        let meeting = meeting.unwrap();
        assert_eq!(meeting.organization_id, org_id);
        assert_eq!(meeting.status, MeetingStatus::Scheduled);
        assert!(meeting.is_upcoming());
    }

    #[test]
    fn test_add_agenda_item() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let result = meeting.add_agenda_item("Approbation des comptes".to_string());
        assert!(result.is_ok());
        assert_eq!(meeting.agenda.len(), 1);
    }

    #[test]
    fn test_complete_meeting() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let result = meeting.complete(45);
        assert!(result.is_ok());
        assert_eq!(meeting.status, MeetingStatus::Completed);
        assert_eq!(meeting.attendees_count, Some(45));
        assert!(!meeting.is_upcoming());
    }

    #[test]
    fn test_complete_already_completed_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        meeting.complete(45).unwrap();
        let result = meeting.complete(50);
        assert!(result.is_err());
        assert_eq!(meeting.attendees_count, Some(45)); // Should not change
    }

    #[test]
    fn test_cancel_meeting() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let result = meeting.cancel();
        assert!(result.is_ok());
        assert_eq!(meeting.status, MeetingStatus::Cancelled);
    }

    #[test]
    fn test_quorum_reached_above_50_percent() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // 600 millièmes présents sur 1000 = 60% → quorum atteint
        let result = meeting.validate_quorum(600.0, 1000.0);
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(meeting.quorum_validated);
        assert!((meeting.quorum_percentage.unwrap() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_quorum_not_reached_at_50_percent_exact() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // 500 millièmes sur 1000 = exactement 50% → quorum NON atteint (Art. 3.87 §5 : >50% requis)
        let result = meeting.validate_quorum(500.0, 1000.0);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert!(!meeting.quorum_validated);
    }

    #[test]
    fn test_quorum_not_reached_below_50_percent() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // 400 millièmes sur 1000 = 40% → quorum non atteint
        let result = meeting.validate_quorum(400.0, 1000.0);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert!(!meeting.quorum_validated);
    }

    #[test]
    fn test_check_quorum_blocks_vote_when_not_validated() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let result = meeting.check_quorum_for_voting();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not been validated yet"));
    }

    #[test]
    fn test_check_quorum_skipped_for_second_convocation() {
        // Art. 3.87 §5 CC: No quorum check for 2nd convocation
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Extraordinary,
            "2e Convocation AGE".to_string(),
            Some("Deuxième convocation - sans quorum".to_string()),
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Mark as second convocation
        meeting.is_second_convocation = true;

        // Should allow voting even without quorum validation
        let result = meeting.check_quorum_for_voting();
        assert!(result.is_ok(), "2nd convocation should skip quorum check");
    }

    #[test]
    fn test_check_quorum_blocks_vote_when_quorum_not_reached() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        meeting.validate_quorum(400.0, 1000.0).unwrap();
        let result = meeting.check_quorum_for_voting();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("second convocation"));
    }

    #[test]
    fn test_quorum_invalid_total_quotas() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let result = meeting.validate_quorum(100.0, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_reschedule_meeting() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let new_date = Utc::now() + Duration::days(60);
        let result = meeting.reschedule(new_date);
        assert!(result.is_ok());
        assert_eq!(meeting.scheduled_date, new_date);
    }

    #[test]
    fn test_set_minutes_sent_success() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);
        let doc_id = Uuid::new_v4();

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act: Complete the meeting first
        meeting.complete(45).unwrap();
        let result = meeting.set_minutes_sent(doc_id);

        // Assert
        assert!(result.is_ok());
        assert_eq!(meeting.minutes_document_id, Some(doc_id));
        assert!(meeting.minutes_sent_at.is_some());
    }

    #[test]
    fn test_set_minutes_sent_before_completion_fails() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);
        let doc_id = Uuid::new_v4();

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act: Try to send minutes while meeting is still Scheduled
        let result = meeting.set_minutes_sent(doc_id);

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Minutes can only be sent after meeting is completed"
        );
    }

    #[test]
    fn test_is_minutes_overdue_not_completed() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act & Assert
        assert!(!meeting.is_minutes_overdue()); // Not completed yet
    }

    #[test]
    fn test_is_minutes_overdue_sent() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);
        let doc_id = Uuid::new_v4();

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act
        meeting.complete(45).unwrap();
        meeting.set_minutes_sent(doc_id).unwrap();

        // Assert
        assert!(!meeting.is_minutes_overdue()); // Minutes sent
    }

    #[test]
    fn test_is_minutes_overdue_past_30_days() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act: Complete the meeting and manually set updated_at to >30 days ago
        meeting.complete(45).unwrap();
        meeting.updated_at = Utc::now() - Duration::days(31);

        // Assert
        assert!(meeting.is_minutes_overdue()); // >30 days without sending minutes
    }

    #[test]
    fn test_is_minutes_overdue_within_30_days() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act: Complete the meeting
        meeting.complete(45).unwrap();

        // Assert
        assert!(!meeting.is_minutes_overdue()); // Within 30 days
    }
}
