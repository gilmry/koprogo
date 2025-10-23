use crate::application::dto::{
    AddAgendaItemRequest, CompleteMeetingRequest, CreateMeetingRequest, MeetingResponse,
    UpdateMeetingRequest,
};
use crate::application::ports::MeetingRepository;
use crate::domain::entities::Meeting;
use std::sync::Arc;
use uuid::Uuid;

pub struct MeetingUseCases {
    repository: Arc<dyn MeetingRepository>,
}

impl MeetingUseCases {
    pub fn new(repository: Arc<dyn MeetingRepository>) -> Self {
        Self { repository }
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

        meeting.complete(request.attendees_count);

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn cancel_meeting(&self, id: Uuid) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        meeting.cancel();

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn delete_meeting(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await
    }
}
