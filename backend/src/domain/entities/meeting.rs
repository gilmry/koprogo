use chrono::{DateTime, Utc};
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
}
