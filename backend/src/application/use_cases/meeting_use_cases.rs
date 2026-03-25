use crate::application::dto::{
    AddAgendaItemRequest, CompleteMeetingRequest, CreateMeetingRequest, MeetingResponse,
    PageRequest, UpdateMeetingRequest,
};
use crate::application::ports::MeetingRepository;
use crate::domain::entities::Meeting;
use chrono::Duration;
use std::sync::Arc;
use uuid::Uuid;

pub struct MeetingUseCases {
    repository: Arc<dyn MeetingRepository>,
    convocation_use_cases: Option<Arc<crate::application::use_cases::ConvocationUseCases>>,
}

impl MeetingUseCases {
    pub fn new(repository: Arc<dyn MeetingRepository>) -> Self {
        Self {
            repository,
            convocation_use_cases: None,
        }
    }

    /// Create MeetingUseCases with ConvocationUseCases for automatic 2nd convocation scheduling
    pub fn new_with_convocation(
        repository: Arc<dyn MeetingRepository>,
        convocation_use_cases: Arc<crate::application::use_cases::ConvocationUseCases>,
    ) -> Self {
        Self {
            repository,
            convocation_use_cases: Some(convocation_use_cases),
        }
    }

    pub async fn create_meeting(
        &self,
        request: CreateMeetingRequest,
    ) -> Result<MeetingResponse, String> {
        let meeting = Meeting::new(
            request.organization_id,
            request.building_id,
            request.meeting_type,
            request.title,
            request.description,
            request.scheduled_date,
            request.location,
        )?;

        let created = self.repository.create(&meeting).await?;
        Ok(MeetingResponse::from(created))
    }

    pub async fn get_meeting(&self, id: Uuid) -> Result<Option<MeetingResponse>, String> {
        let meeting = self.repository.find_by_id(id).await?;
        Ok(meeting.map(MeetingResponse::from))
    }

    pub async fn list_meetings_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<MeetingResponse>, String> {
        let meetings = self.repository.find_by_building(building_id).await?;
        Ok(meetings.into_iter().map(MeetingResponse::from).collect())
    }

    pub async fn list_meetings_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<MeetingResponse>, i64), String> {
        let (meetings, total) = self
            .repository
            .find_all_paginated(page_request, organization_id)
            .await?;

        let dtos = meetings.into_iter().map(MeetingResponse::from).collect();
        Ok((dtos, total))
    }

    pub async fn update_meeting(
        &self,
        id: Uuid,
        request: UpdateMeetingRequest,
    ) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        // Update fields if provided
        if let Some(title) = request.title {
            if title.is_empty() {
                return Err("Title cannot be empty".to_string());
            }
            meeting.title = title;
        }

        if let Some(description) = request.description {
            meeting.description = Some(description);
        }

        if let Some(scheduled_date) = request.scheduled_date {
            meeting.scheduled_date = scheduled_date;
        }

        if let Some(location) = request.location {
            if location.is_empty() {
                return Err("Location cannot be empty".to_string());
            }
            meeting.location = location;
        }

        meeting.updated_at = chrono::Utc::now();

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn add_agenda_item(
        &self,
        id: Uuid,
        request: AddAgendaItemRequest,
    ) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        meeting.add_agenda_item(request.item)?;

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn complete_meeting(
        &self,
        id: Uuid,
        request: CompleteMeetingRequest,
    ) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        meeting.complete(request.attendees_count)?;

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn cancel_meeting(&self, id: Uuid) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        meeting.cancel()?;

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn reschedule_meeting(
        &self,
        id: Uuid,
        new_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        meeting.reschedule(new_date)?;

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn delete_meeting(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await
    }

    /// Attach minutes document to a completed meeting (Issue #313)
    pub async fn attach_minutes(
        &self,
        id: Uuid,
        document_id: Uuid,
    ) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        meeting.set_minutes_sent(document_id)?;

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    /// Valide le quorum d'une AG (Art. 3.87 §5 CC).
    /// Doit être appelé AVANT que les votes soient ouverts.
    /// Si quorum non atteint, déclenche automatiquement la création d'une 2e convocation
    /// pour le même bâtiment (si ConvocationUseCases disponible).
    /// Retourne Ok(true) si quorum atteint, Ok(false) si 2e convocation requise.
    pub async fn validate_quorum(
        &self,
        meeting_id: Uuid,
        present_quotas: f64,
        total_quotas: f64,
    ) -> Result<(bool, MeetingResponse), String> {
        let mut meeting = self
            .repository
            .find_by_id(meeting_id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        let quorum_reached = meeting.validate_quorum(present_quotas, total_quotas)?;
        let updated = self.repository.update(&meeting).await?;

        // Art. 3.87 §5 CC: Si quorum non atteint, déclencher une 2e convocation
        if !quorum_reached {
            if let Some(convocation_uc) = &self.convocation_use_cases {
                // Create second meeting (15 days after first)
                let second_meeting_date = meeting.scheduled_date + Duration::days(15);
                let second_meeting_id = Uuid::new_v4();

                // Schedule second convocation (language defaults to FR)
                let _result = convocation_uc
                    .schedule_second_convocation(
                        meeting.organization_id,
                        meeting.building_id,
                        meeting_id,
                        second_meeting_id,
                        second_meeting_date,
                        "FR".to_string(),
                        Uuid::nil(), // system-created convocation
                    )
                    .await;
                // Note: We don't fail if second convocation scheduling fails
                // (could log the error, but don't block the quorum validation result)
            }
        }

        Ok((quorum_reached, MeetingResponse::from(updated)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::PageRequest;
    use crate::application::ports::MeetingRepository;
    use crate::domain::entities::{MeetingStatus, MeetingType};
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use mockall::mock;
    use std::sync::Arc;

    mock! {
        MeetingRepo {}

        #[async_trait]
        impl MeetingRepository for MeetingRepo {
            async fn create(&self, meeting: &Meeting) -> Result<Meeting, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Meeting>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Meeting>, String>;
            async fn update(&self, meeting: &Meeting) -> Result<Meeting, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn find_all_paginated(
                &self,
                page_request: &PageRequest,
                organization_id: Option<Uuid>,
            ) -> Result<(Vec<Meeting>, i64), String>;
        }
    }

    /// Helper to build a valid Meeting for testing purposes.
    fn make_meeting(building_id: Uuid, org_id: Uuid) -> Meeting {
        Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            Some("Annual general assembly".to_string()),
            Utc::now() + Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .unwrap()
    }

    // ---------------------------------------------------------------
    // 1. Create meeting success
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_create_meeting_success() {
        let mut mock_repo = MockMeetingRepo::new();

        mock_repo.expect_create().returning(|m| Ok(m.clone()));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let request = CreateMeetingRequest {
            organization_id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            meeting_type: MeetingType::Ordinary,
            title: "AGO 2024".to_string(),
            description: Some("Annual assembly".to_string()),
            scheduled_date: Utc::now() + Duration::days(30),
            location: "Salle communale".to_string(),
        };

        let result = use_cases.create_meeting(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.title, "AGO 2024");
        assert_eq!(response.status, MeetingStatus::Scheduled);
    }

    // ---------------------------------------------------------------
    // 2. Create meeting with invalid data (empty title)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_create_meeting_empty_title_fails() {
        let mock_repo = MockMeetingRepo::new();
        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let request = CreateMeetingRequest {
            organization_id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            meeting_type: MeetingType::Ordinary,
            title: "".to_string(),
            description: None,
            scheduled_date: Utc::now() + Duration::days(30),
            location: "Salle communale".to_string(),
        };

        let result = use_cases.create_meeting(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Title cannot be empty"));
    }

    // ---------------------------------------------------------------
    // 3. Create meeting with empty location
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_create_meeting_empty_location_fails() {
        let mock_repo = MockMeetingRepo::new();
        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let request = CreateMeetingRequest {
            organization_id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            meeting_type: MeetingType::Extraordinary,
            title: "AGE 2024".to_string(),
            description: None,
            scheduled_date: Utc::now() + Duration::days(15),
            location: "".to_string(),
        };

        let result = use_cases.create_meeting(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Location cannot be empty"));
    }

    // ---------------------------------------------------------------
    // 4. Get meeting by ID — found
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_get_meeting_found() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(make_meeting(building_id, org_id))));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let result = use_cases.get_meeting(meeting_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // ---------------------------------------------------------------
    // 5. Get meeting by ID — not found
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_get_meeting_not_found() {
        let mut mock_repo = MockMeetingRepo::new();
        mock_repo.expect_find_by_id().returning(|_| Ok(None));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let result = use_cases.get_meeting(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ---------------------------------------------------------------
    // 6. List meetings by building
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_list_meetings_by_building() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_building()
            .withf(move |id| *id == building_id)
            .returning(move |_| {
                Ok(vec![
                    make_meeting(building_id, org_id),
                    make_meeting(building_id, org_id),
                ])
            });

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let result = use_cases.list_meetings_by_building(building_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    // ---------------------------------------------------------------
    // 7. Update meeting — success
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_update_meeting_success() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let request = UpdateMeetingRequest {
            title: Some("Renamed AGO".to_string()),
            description: Some("Updated description".to_string()),
            scheduled_date: None,
            location: None,
        };

        let result = use_cases.update_meeting(meeting_id, request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.title, "Renamed AGO");
    }

    // ---------------------------------------------------------------
    // 8. Update meeting — not found
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_update_meeting_not_found() {
        let mut mock_repo = MockMeetingRepo::new();
        mock_repo.expect_find_by_id().returning(|_| Ok(None));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let request = UpdateMeetingRequest {
            title: Some("New title".to_string()),
            description: None,
            scheduled_date: None,
            location: None,
        };

        let result = use_cases.update_meeting(Uuid::new_v4(), request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Meeting not found"));
    }

    // ---------------------------------------------------------------
    // 9. Update meeting — empty title rejected
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_update_meeting_empty_title_rejected() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let request = UpdateMeetingRequest {
            title: Some("".to_string()),
            description: None,
            scheduled_date: None,
            location: None,
        };

        let result = use_cases.update_meeting(meeting_id, request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Title cannot be empty"));
    }

    // ---------------------------------------------------------------
    // 10. Delete meeting
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_delete_meeting_success() {
        let meeting_id = Uuid::new_v4();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_delete()
            .withf(move |id| *id == meeting_id)
            .returning(|_| Ok(true));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let result = use_cases.delete_meeting(meeting_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // ---------------------------------------------------------------
    // 11. Validate quorum — reached (>50%)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_validate_quorum_reached() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        // 600/1000 = 60% → quorum reached
        let result = use_cases.validate_quorum(meeting_id, 600.0, 1000.0).await;
        assert!(result.is_ok());
        let (reached, response) = result.unwrap();
        assert!(reached);
        assert!(response.quorum_validated);
        assert!((response.quorum_percentage.unwrap() - 60.0).abs() < 0.01);
    }

    // ---------------------------------------------------------------
    // 12. Validate quorum — not reached (<=50%)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_validate_quorum_not_reached() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        // No convocation_use_cases set, so second convocation won't be triggered
        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        // 400/1000 = 40% → quorum NOT reached
        let result = use_cases.validate_quorum(meeting_id, 400.0, 1000.0).await;
        assert!(result.is_ok());
        let (reached, response) = result.unwrap();
        assert!(!reached);
        assert!(!response.quorum_validated);
        assert!((response.quorum_percentage.unwrap() - 40.0).abs() < 0.01);
    }

    // ---------------------------------------------------------------
    // 13. Validate quorum — meeting not found
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_validate_quorum_meeting_not_found() {
        let mut mock_repo = MockMeetingRepo::new();
        mock_repo.expect_find_by_id().returning(|_| Ok(None));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let result = use_cases
            .validate_quorum(Uuid::new_v4(), 600.0, 1000.0)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Meeting not found"));
    }

    // ---------------------------------------------------------------
    // 14. Complete meeting via use case
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_complete_meeting_success() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let request = CompleteMeetingRequest {
            attendees_count: 42,
        };
        let result = use_cases.complete_meeting(meeting_id, request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, MeetingStatus::Completed);
        assert_eq!(response.attendees_count, Some(42));
    }

    // ---------------------------------------------------------------
    // 15. Cancel meeting via use case
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_cancel_meeting_success() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let result = use_cases.cancel_meeting(meeting_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, MeetingStatus::Cancelled);
    }

    // ---------------------------------------------------------------
    // 16. Add agenda item via use case
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_add_agenda_item_success() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        let request = AddAgendaItemRequest {
            item: "Approbation des comptes".to_string(),
        };
        let result = use_cases.add_agenda_item(meeting_id, request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.agenda.len(), 1);
        assert_eq!(response.agenda[0], "Approbation des comptes");
    }

    // ---------------------------------------------------------------
    // 17. Validate quorum — exact 50% NOT reached (Art. 3.87 §5: >50% strict)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_validate_quorum_exact_50_percent_not_reached() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo));

        // 500/1000 = exactly 50% → quorum NOT reached (Art. 3.87 §5: strictly >50%)
        let result = use_cases.validate_quorum(meeting_id, 500.0, 1000.0).await;
        assert!(result.is_ok());
        let (reached, response) = result.unwrap();
        assert!(!reached);
        assert!(!response.quorum_validated);
        assert!((response.quorum_percentage.unwrap() - 50.0).abs() < 0.01);
    }
}
