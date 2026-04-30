use crate::application::dto::{
    ConvocationRecipientResponse, ConvocationResponse, CreateConvocationRequest,
    RecipientTrackingSummaryResponse, ScheduleConvocationRequest, SendConvocationRequest,
};
use crate::application::ports::{
    BuildingRepository, ConvocationRecipientRepository, ConvocationRepository, MeetingRepository,
    OwnerRepository,
};
use crate::domain::entities::{AttendanceStatus, Convocation, ConvocationRecipient};
use crate::domain::services::ConvocationExporter;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ConvocationUseCases {
    convocation_repository: Arc<dyn ConvocationRepository>,
    recipient_repository: Arc<dyn ConvocationRecipientRepository>,
    owner_repository: Arc<dyn OwnerRepository>,
    building_repository: Arc<dyn BuildingRepository>,
    meeting_repository: Arc<dyn MeetingRepository>,
}

impl ConvocationUseCases {
    pub fn new(
        convocation_repository: Arc<dyn ConvocationRepository>,
        recipient_repository: Arc<dyn ConvocationRecipientRepository>,
        owner_repository: Arc<dyn OwnerRepository>,
        building_repository: Arc<dyn BuildingRepository>,
        meeting_repository: Arc<dyn MeetingRepository>,
    ) -> Self {
        Self {
            convocation_repository,
            recipient_repository,
            owner_repository,
            building_repository,
            meeting_repository,
        }
    }

    /// Create a new convocation
    pub async fn create_convocation(
        &self,
        organization_id: Uuid,
        request: CreateConvocationRequest,
        created_by: Uuid,
    ) -> Result<ConvocationResponse, String> {
        // Create domain entity (validates legal deadline)
        let convocation = Convocation::new(
            organization_id,
            request.building_id,
            request.meeting_id,
            request.meeting_type,
            request.meeting_date,
            request.language,
            created_by,
        )?;

        let created = self.convocation_repository.create(&convocation).await?;

        Ok(ConvocationResponse::from(created))
    }

    /// Get convocation by ID
    pub async fn get_convocation(&self, id: Uuid) -> Result<ConvocationResponse, String> {
        let convocation = self
            .convocation_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Convocation not found: {}", id))?;

        Ok(ConvocationResponse::from(convocation))
    }

    /// Get convocation by meeting ID
    pub async fn get_convocation_by_meeting(
        &self,
        meeting_id: Uuid,
    ) -> Result<Option<ConvocationResponse>, String> {
        let convocation = self
            .convocation_repository
            .find_by_meeting_id(meeting_id)
            .await?;

        Ok(convocation.map(ConvocationResponse::from))
    }

    /// List convocations for a building
    pub async fn list_building_convocations(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<ConvocationResponse>, String> {
        let convocations = self
            .convocation_repository
            .find_by_building(building_id)
            .await?;

        Ok(convocations
            .into_iter()
            .map(ConvocationResponse::from)
            .collect())
    }

    /// List convocations for an organization
    pub async fn list_organization_convocations(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<ConvocationResponse>, String> {
        let convocations = self
            .convocation_repository
            .find_by_organization(organization_id)
            .await?;

        Ok(convocations
            .into_iter()
            .map(ConvocationResponse::from)
            .collect())
    }

    /// Schedule convocation to be sent at specific date
    pub async fn schedule_convocation(
        &self,
        id: Uuid,
        request: ScheduleConvocationRequest,
    ) -> Result<ConvocationResponse, String> {
        let mut convocation = self
            .convocation_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Convocation not found: {}", id))?;

        convocation.schedule(request.send_date)?;

        let updated = self.convocation_repository.update(&convocation).await?;

        Ok(ConvocationResponse::from(updated))
    }

    /// Send convocation to owners (generates PDF, creates recipients, sends emails)
    /// This would typically be called by a background job or email service
    pub async fn send_convocation(
        &self,
        id: Uuid,
        request: SendConvocationRequest,
    ) -> Result<ConvocationResponse, String> {
        let mut convocation = self
            .convocation_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Convocation not found: {}", id))?;

        // Fetch building for PDF generation
        let building = self
            .building_repository
            .find_by_id(convocation.building_id)
            .await?
            .ok_or_else(|| format!("Building not found: {}", convocation.building_id))?;

        // Fetch meeting for PDF generation
        let meeting = self
            .meeting_repository
            .find_by_id(convocation.meeting_id)
            .await?
            .ok_or_else(|| format!("Meeting not found: {}", convocation.meeting_id))?;

        // Generate PDF
        let pdf_bytes = ConvocationExporter::export_to_pdf(&building, &meeting, &convocation)
            .map_err(|e| format!("Failed to generate PDF: {}", e))?;

        // Save PDF to file
        let upload_dir =
            std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "/tmp/koprogo-uploads".to_string());
        let pdf_file_path = format!("{}/convocations/conv-{}.pdf", upload_dir, id);
        ConvocationExporter::save_to_file(&pdf_bytes, &pdf_file_path)
            .map_err(|e| format!("Failed to save PDF: {}", e))?;

        // Fetch owner emails
        let mut recipients = Vec::new();
        for owner_id in &request.recipient_owner_ids {
            let owner = self
                .owner_repository
                .find_by_id(*owner_id)
                .await?
                .ok_or_else(|| format!("Owner not found: {}", owner_id))?;

            let mut recipient = ConvocationRecipient::new(id, *owner_id, owner.email)?;
            recipient.mark_email_sent();
            recipients.push(recipient);
        }

        // Create recipients in database (bulk insert)
        let created_recipients = self.recipient_repository.create_many(&recipients).await?;

        // Mark convocation as sent
        convocation.mark_sent(pdf_file_path, created_recipients.len() as i32)?;

        let updated = self.convocation_repository.update(&convocation).await?;

        Ok(ConvocationResponse::from(updated))
    }

    /// Mark recipient email as sent
    pub async fn mark_recipient_email_sent(
        &self,
        recipient_id: Uuid,
    ) -> Result<ConvocationRecipientResponse, String> {
        let mut recipient = self
            .recipient_repository
            .find_by_id(recipient_id)
            .await?
            .ok_or_else(|| format!("Recipient not found: {}", recipient_id))?;

        recipient.mark_email_sent();

        let updated = self.recipient_repository.update(&recipient).await?;

        Ok(ConvocationRecipientResponse::from(updated))
    }

    /// Mark recipient email as opened (tracking pixel or link click)
    pub async fn mark_recipient_email_opened(
        &self,
        recipient_id: Uuid,
    ) -> Result<ConvocationRecipientResponse, String> {
        let mut recipient = self
            .recipient_repository
            .find_by_id(recipient_id)
            .await?
            .ok_or_else(|| format!("Recipient not found: {}", recipient_id))?;

        recipient.mark_email_opened()?;

        let updated = self.recipient_repository.update(&recipient).await?;

        // Update convocation tracking counts
        self.update_convocation_tracking(recipient.convocation_id)
            .await?;

        Ok(ConvocationRecipientResponse::from(updated))
    }

    /// Update recipient attendance status
    pub async fn update_recipient_attendance(
        &self,
        recipient_id: Uuid,
        status: AttendanceStatus,
    ) -> Result<ConvocationRecipientResponse, String> {
        let mut recipient = self
            .recipient_repository
            .find_by_id(recipient_id)
            .await?
            .ok_or_else(|| format!("Recipient not found: {}", recipient_id))?;

        recipient.update_attendance_status(status)?;

        let updated = self.recipient_repository.update(&recipient).await?;

        // Update convocation tracking counts
        self.update_convocation_tracking(recipient.convocation_id)
            .await?;

        Ok(ConvocationRecipientResponse::from(updated))
    }

    /// Set proxy delegation for recipient
    pub async fn set_recipient_proxy(
        &self,
        recipient_id: Uuid,
        proxy_owner_id: Uuid,
    ) -> Result<ConvocationRecipientResponse, String> {
        let mut recipient = self
            .recipient_repository
            .find_by_id(recipient_id)
            .await?
            .ok_or_else(|| format!("Recipient not found: {}", recipient_id))?;

        recipient.set_proxy(proxy_owner_id)?;

        let updated = self.recipient_repository.update(&recipient).await?;

        Ok(ConvocationRecipientResponse::from(updated))
    }

    /// Send reminders to recipients who haven't opened the convocation (J-3)
    /// This would typically be called by a background job
    pub async fn send_reminders(
        &self,
        convocation_id: Uuid,
    ) -> Result<Vec<ConvocationRecipientResponse>, String> {
        // Get recipients who need reminder
        let recipients = self
            .recipient_repository
            .find_needing_reminder(convocation_id)
            .await?;

        let mut updated_recipients = Vec::new();

        for mut recipient in recipients {
            recipient.mark_reminder_sent()?;
            let updated = self.recipient_repository.update(&recipient).await?;
            updated_recipients.push(ConvocationRecipientResponse::from(updated));
        }

        // Mark convocation as reminder sent
        if !updated_recipients.is_empty() {
            let mut convocation = self
                .convocation_repository
                .find_by_id(convocation_id)
                .await?
                .ok_or_else(|| format!("Convocation not found: {}", convocation_id))?;

            convocation.mark_reminder_sent()?;
            self.convocation_repository.update(&convocation).await?;
        }

        Ok(updated_recipients)
    }

    /// Get tracking summary for convocation
    pub async fn get_tracking_summary(
        &self,
        convocation_id: Uuid,
    ) -> Result<RecipientTrackingSummaryResponse, String> {
        let summary = self
            .recipient_repository
            .get_tracking_summary(convocation_id)
            .await?;

        Ok(RecipientTrackingSummaryResponse::new(
            summary.total_count,
            summary.opened_count,
            summary.will_attend_count,
            summary.will_not_attend_count,
            summary.attended_count,
            summary.did_not_attend_count,
            summary.pending_count,
            summary.failed_email_count,
        ))
    }

    /// Get all recipients for a convocation
    pub async fn list_convocation_recipients(
        &self,
        convocation_id: Uuid,
    ) -> Result<Vec<ConvocationRecipientResponse>, String> {
        let recipients = self
            .recipient_repository
            .find_by_convocation(convocation_id)
            .await?;

        Ok(recipients
            .into_iter()
            .map(ConvocationRecipientResponse::from)
            .collect())
    }

    /// Cancel convocation
    pub async fn cancel_convocation(&self, id: Uuid) -> Result<ConvocationResponse, String> {
        let mut convocation = self
            .convocation_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Convocation not found: {}", id))?;

        convocation.cancel()?;

        let updated = self.convocation_repository.update(&convocation).await?;

        Ok(ConvocationResponse::from(updated))
    }

    /// Delete convocation (and all recipients via CASCADE)
    pub async fn delete_convocation(&self, id: Uuid) -> Result<bool, String> {
        self.convocation_repository.delete(id).await
    }

    /// Process scheduled convocations (called by background job)
    /// Returns list of convocations that were sent
    pub async fn process_scheduled_convocations(&self) -> Result<Vec<ConvocationResponse>, String> {
        let now = Utc::now();
        let scheduled = self
            .convocation_repository
            .find_pending_scheduled(now)
            .await?;

        let mut sent = Vec::new();

        for convocation in scheduled {
            // This would trigger PDF generation and email sending
            // For now, we just return the list that needs processing
            sent.push(ConvocationResponse::from(convocation));
        }

        Ok(sent)
    }

    /// Process reminder sending (called by background job)
    /// Returns list of convocations that had reminders sent
    pub async fn process_reminder_sending(&self) -> Result<Vec<ConvocationResponse>, String> {
        let now = Utc::now();
        let needing_reminder = self
            .convocation_repository
            .find_needing_reminder(now)
            .await?;

        let mut processed = Vec::new();

        for convocation in needing_reminder {
            // Send reminders to recipients
            self.send_reminders(convocation.id).await?;
            processed.push(ConvocationResponse::from(convocation));
        }

        Ok(processed)
    }

    /// Schedule a second convocation after quorum not reached
    /// Art. 3.87 §5 CC: "La deuxième assemblée délibère valablement quel que soit le nombre de présents."
    ///
    /// # Arguments
    /// * `first_meeting_id` - ID of the first meeting where quorum was not reached
    /// * `new_meeting_id` - ID of the new meeting scheduled for the second convocation
    /// * `new_meeting_date` - Date of the second meeting (must be ≥15 days after first meeting)
    /// * `language` - Language for the convocation (FR/NL/DE/EN)
    /// * `created_by` - User ID creating the second convocation
    ///
    /// # Returns
    /// Result with the created second convocation
    pub async fn schedule_second_convocation(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
        first_meeting_id: Uuid,
        new_meeting_id: Uuid,
        new_meeting_date: chrono::DateTime<chrono::Utc>,
        language: String,
        created_by: Uuid,
    ) -> Result<ConvocationResponse, String> {
        // Fetch the first meeting to get its meeting date
        let first_meeting = self
            .meeting_repository
            .find_by_id(first_meeting_id)
            .await?
            .ok_or_else(|| format!("First meeting not found: {}", first_meeting_id))?;

        // Create the second convocation using the domain entity constructor
        // This validates that the second meeting is at least 15 days after the first
        let second_convocation = Convocation::new_second_convocation(
            organization_id,
            building_id,
            new_meeting_id,
            first_meeting_id,
            first_meeting.scheduled_date,
            new_meeting_date,
            language,
            created_by,
        )?;

        let created = self
            .convocation_repository
            .create(&second_convocation)
            .await?;

        Ok(ConvocationResponse::from(created))
    }

    /// Internal helper: Update convocation tracking counts from recipients
    async fn update_convocation_tracking(&self, convocation_id: Uuid) -> Result<(), String> {
        let summary = self
            .recipient_repository
            .get_tracking_summary(convocation_id)
            .await?;

        let mut convocation = self
            .convocation_repository
            .find_by_id(convocation_id)
            .await?
            .ok_or_else(|| format!("Convocation not found: {}", convocation_id))?;

        convocation.update_tracking_counts(
            summary.opened_count as i32,
            summary.will_attend_count as i32,
            summary.will_not_attend_count as i32,
        );

        self.convocation_repository.update(&convocation).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{BuildingFilters, OwnerFilters, PageRequest};
    use crate::application::ports::{
        ConvocationRecipientRepository, ConvocationRepository, RecipientTrackingSummary,
    };
    use crate::domain::entities::{
        AttendanceStatus, Building, Convocation, ConvocationRecipient, ConvocationStatus,
        ConvocationType, Meeting, Owner,
    };
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use mockall::mock;
    use std::sync::Arc;
    use uuid::Uuid;

    // ---------------------------------------------------------------------------
    // Mock definitions using mockall::mock!
    // ---------------------------------------------------------------------------

    mock! {
        ConvRepo {}

        #[async_trait]
        impl ConvocationRepository for ConvRepo {
            async fn create(&self, convocation: &Convocation) -> Result<Convocation, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Convocation>, String>;
            async fn find_by_meeting_id(&self, meeting_id: Uuid) -> Result<Option<Convocation>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Convocation>, String>;
            async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<Convocation>, String>;
            async fn find_by_status(&self, organization_id: Uuid, status: ConvocationStatus) -> Result<Vec<Convocation>, String>;
            async fn find_pending_scheduled(&self, now: chrono::DateTime<Utc>) -> Result<Vec<Convocation>, String>;
            async fn find_needing_reminder(&self, now: chrono::DateTime<Utc>) -> Result<Vec<Convocation>, String>;
            async fn update(&self, convocation: &Convocation) -> Result<Convocation, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String>;
            async fn count_by_status(&self, organization_id: Uuid, status: ConvocationStatus) -> Result<i64, String>;
        }
    }

    mock! {
        RecipientRepo {}

        #[async_trait]
        impl ConvocationRecipientRepository for RecipientRepo {
            async fn create(&self, recipient: &ConvocationRecipient) -> Result<ConvocationRecipient, String>;
            async fn create_many(&self, recipients: &[ConvocationRecipient]) -> Result<Vec<ConvocationRecipient>, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<ConvocationRecipient>, String>;
            async fn find_by_convocation(&self, convocation_id: Uuid) -> Result<Vec<ConvocationRecipient>, String>;
            async fn find_by_convocation_and_owner(&self, convocation_id: Uuid, owner_id: Uuid) -> Result<Option<ConvocationRecipient>, String>;
            async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<ConvocationRecipient>, String>;
            async fn find_by_attendance_status(&self, convocation_id: Uuid, status: AttendanceStatus) -> Result<Vec<ConvocationRecipient>, String>;
            async fn find_needing_reminder(&self, convocation_id: Uuid) -> Result<Vec<ConvocationRecipient>, String>;
            async fn find_failed_emails(&self, convocation_id: Uuid) -> Result<Vec<ConvocationRecipient>, String>;
            async fn update(&self, recipient: &ConvocationRecipient) -> Result<ConvocationRecipient, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn count_by_convocation(&self, convocation_id: Uuid) -> Result<i64, String>;
            async fn count_opened(&self, convocation_id: Uuid) -> Result<i64, String>;
            async fn count_by_attendance_status(&self, convocation_id: Uuid, status: AttendanceStatus) -> Result<i64, String>;
            async fn get_tracking_summary(&self, convocation_id: Uuid) -> Result<RecipientTrackingSummary, String>;
        }
    }

    mock! {
        OwnerRepo {}

        #[async_trait]
        impl OwnerRepository for OwnerRepo {
            async fn create(&self, owner: &Owner) -> Result<Owner, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, String>;
            async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Owner>, String>;
            async fn find_by_user_id_and_organization(&self, user_id: Uuid, organization_id: Uuid) -> Result<Option<Owner>, String>;
            async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, String>;
            async fn find_all(&self) -> Result<Vec<Owner>, String>;
            async fn find_all_paginated(&self, page_request: &PageRequest, filters: &OwnerFilters) -> Result<(Vec<Owner>, i64), String>;
            async fn update(&self, owner: &Owner) -> Result<Owner, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn set_user_link(&self, owner_id: Uuid, user_id: Option<Uuid>) -> Result<bool, String>;
        }
    }

    mock! {
        BuildingRepo {}

        #[async_trait]
        impl BuildingRepository for BuildingRepo {
            async fn create(&self, building: &Building) -> Result<Building, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
            async fn find_all(&self) -> Result<Vec<Building>, String>;
            async fn find_all_paginated(&self, page_request: &PageRequest, filters: &BuildingFilters) -> Result<(Vec<Building>, i64), String>;
            async fn update(&self, building: &Building) -> Result<Building, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn find_by_slug(&self, slug: &str) -> Result<Option<Building>, String>;
        }
    }

    mock! {
        MeetingRepo {}

        #[async_trait]
        impl MeetingRepository for MeetingRepo {
            async fn create(&self, meeting: &Meeting) -> Result<Meeting, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Meeting>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Meeting>, String>;
            async fn update(&self, meeting: &Meeting) -> Result<Meeting, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn find_all_paginated(&self, page_request: &PageRequest, organization_id: Option<Uuid>) -> Result<(Vec<Meeting>, i64), String>;
        }
    }

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    /// Build a ConvocationUseCases with the given mocks, using defaults (no-op) for the rest.
    fn make_use_cases(
        conv_repo: MockConvRepo,
        recip_repo: MockRecipientRepo,
        owner_repo: MockOwnerRepo,
        building_repo: MockBuildingRepo,
        meeting_repo: MockMeetingRepo,
    ) -> ConvocationUseCases {
        ConvocationUseCases::new(
            Arc::new(conv_repo),
            Arc::new(recip_repo),
            Arc::new(owner_repo),
            Arc::new(building_repo),
            Arc::new(meeting_repo),
        )
    }

    /// Create a valid Convocation domain entity (meeting in 20 days, Ordinary type).
    fn make_convocation(org_id: Uuid, building_id: Uuid, meeting_id: Uuid) -> Convocation {
        let meeting_date = Utc::now() + Duration::days(20);
        Convocation::new(
            org_id,
            building_id,
            meeting_id,
            ConvocationType::Ordinary,
            meeting_date,
            "FR".to_string(),
            Uuid::new_v4(),
        )
        .expect("helper should produce a valid convocation")
    }

    /// Create a valid Convocation that is already Sent (status=Sent, has recipients, etc.).
    fn make_sent_convocation(org_id: Uuid, building_id: Uuid, meeting_id: Uuid) -> Convocation {
        let mut conv = make_convocation(org_id, building_id, meeting_id);
        conv.mark_sent("/tmp/conv.pdf".to_string(), 5).unwrap();
        conv
    }

    /// Create a valid ConvocationRecipient (email already sent).
    fn make_recipient(convocation_id: Uuid, owner_id: Uuid) -> ConvocationRecipient {
        let mut r =
            ConvocationRecipient::new(convocation_id, owner_id, "owner@example.com".to_string())
                .unwrap();
        r.mark_email_sent();
        r
    }

    // ---------------------------------------------------------------------------
    // Test 1: Create convocation with valid legal deadline (ordinary, 20 days)
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_create_convocation_ordinary_valid_deadline() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let meeting_date = Utc::now() + Duration::days(20);

        let mut conv_repo = MockConvRepo::new();
        conv_repo.expect_create().returning(|conv| Ok(conv.clone()));

        let uc = make_use_cases(
            conv_repo,
            MockRecipientRepo::new(),
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        let request = CreateConvocationRequest {
            building_id,
            meeting_id,
            meeting_type: ConvocationType::Ordinary,
            meeting_date,
            language: "FR".to_string(),
        };

        let result = uc.create_convocation(org_id, request, Uuid::new_v4()).await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let resp = result.unwrap();
        assert_eq!(resp.status, ConvocationStatus::Draft);
        assert_eq!(resp.language, "FR");
        assert!(resp.respects_legal_deadline);
    }

    // ---------------------------------------------------------------------------
    // Test 2: Create convocation violating legal deadline (only 5 days notice)
    // Art. 3.87 §3 CC requires 15 days for Ordinary
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_create_convocation_violating_legal_deadline() {
        let org_id = Uuid::new_v4();
        let meeting_date = Utc::now() + Duration::days(5); // Only 5 days — too soon

        let uc = make_use_cases(
            MockConvRepo::new(),
            MockRecipientRepo::new(),
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        let request = CreateConvocationRequest {
            building_id: Uuid::new_v4(),
            meeting_id: Uuid::new_v4(),
            meeting_type: ConvocationType::Ordinary,
            meeting_date,
            language: "FR".to_string(),
        };

        let result = uc.create_convocation(org_id, request, Uuid::new_v4()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Meeting date too soon"),
            "Expected 'Meeting date too soon' error, got: {}",
            err
        );
    }

    // ---------------------------------------------------------------------------
    // Test 3: Create extraordinary convocation with valid deadline (15 days)
    // Art. 3.87 §3 CC: extraordinary also requires 15 days
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_create_convocation_extraordinary_valid_deadline() {
        let org_id = Uuid::new_v4();
        let meeting_date = Utc::now() + Duration::days(16); // 16 days — enough for extraordinary

        let mut conv_repo = MockConvRepo::new();
        conv_repo.expect_create().returning(|conv| Ok(conv.clone()));

        let uc = make_use_cases(
            conv_repo,
            MockRecipientRepo::new(),
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        let request = CreateConvocationRequest {
            building_id: Uuid::new_v4(),
            meeting_id: Uuid::new_v4(),
            meeting_type: ConvocationType::Extraordinary,
            meeting_date,
            language: "NL".to_string(),
        };

        let result = uc.create_convocation(org_id, request, Uuid::new_v4()).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.language, "NL");
    }

    // ---------------------------------------------------------------------------
    // Test 4: Schedule convocation (Draft -> Scheduled)
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_schedule_convocation_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let conv = make_convocation(org_id, building_id, meeting_id);
        let conv_id = conv.id;
        let min_send_date = conv.minimum_send_date;

        let mut conv_repo = MockConvRepo::new();
        let conv_clone = conv.clone();
        conv_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(conv_clone.clone())));
        conv_repo.expect_update().returning(|conv| Ok(conv.clone()));

        let uc = make_use_cases(
            conv_repo,
            MockRecipientRepo::new(),
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        // Schedule to send before the minimum_send_date (valid)
        let send_date = min_send_date - Duration::days(1);
        let request = ScheduleConvocationRequest { send_date };

        let result = uc.schedule_convocation(conv_id, request).await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let resp = result.unwrap();
        assert_eq!(resp.status, ConvocationStatus::Scheduled);
        assert!(resp.scheduled_send_date.is_some());
    }

    // ---------------------------------------------------------------------------
    // Test 5: Cancel convocation (Draft -> Cancelled)
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_cancel_convocation_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let conv = make_convocation(org_id, building_id, meeting_id);
        let conv_id = conv.id;

        let mut conv_repo = MockConvRepo::new();
        let conv_clone = conv.clone();
        conv_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(conv_clone.clone())));
        conv_repo.expect_update().returning(|conv| Ok(conv.clone()));

        let uc = make_use_cases(
            conv_repo,
            MockRecipientRepo::new(),
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        let result = uc.cancel_convocation(conv_id).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, ConvocationStatus::Cancelled);
    }

    // ---------------------------------------------------------------------------
    // Test 6: Cancel already-cancelled convocation -> error
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_cancel_convocation_already_cancelled_error() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let mut conv = make_convocation(org_id, building_id, meeting_id);
        conv.cancel().unwrap(); // Already cancelled
        let conv_id = conv.id;

        let mut conv_repo = MockConvRepo::new();
        let conv_clone = conv.clone();
        conv_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(conv_clone.clone())));

        let uc = make_use_cases(
            conv_repo,
            MockRecipientRepo::new(),
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        let result = uc.cancel_convocation(conv_id).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already cancelled"));
    }

    // ---------------------------------------------------------------------------
    // Test 7: Send reminders (J-3) marks recipients as reminder_sent
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_send_reminders_marks_recipients() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let conv = make_sent_convocation(org_id, building_id, meeting_id);
        let conv_id = conv.id;

        let owner1_id = Uuid::new_v4();
        let owner2_id = Uuid::new_v4();
        let r1 = make_recipient(conv_id, owner1_id);
        let r2 = make_recipient(conv_id, owner2_id);

        let mut recip_repo = MockRecipientRepo::new();
        let r1_clone = r1.clone();
        let r2_clone = r2.clone();
        recip_repo
            .expect_find_needing_reminder()
            .returning(move |_| Ok(vec![r1_clone.clone(), r2_clone.clone()]));
        recip_repo.expect_update().returning(|r| Ok(r.clone()));

        let mut conv_repo = MockConvRepo::new();
        let conv_clone = conv.clone();
        conv_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(conv_clone.clone())));
        conv_repo.expect_update().returning(|conv| Ok(conv.clone()));

        let uc = make_use_cases(
            conv_repo,
            recip_repo,
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        let result = uc.send_reminders(conv_id).await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let recipients = result.unwrap();
        assert_eq!(recipients.len(), 2);
        // After mark_reminder_sent, reminder_sent_at should be set
        assert!(recipients[0].reminder_sent_at.is_some());
        assert!(recipients[1].reminder_sent_at.is_some());
    }

    // ---------------------------------------------------------------------------
    // Test 8: Track email opened updates convocation tracking counts
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_mark_email_opened_updates_tracking() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let conv = make_sent_convocation(org_id, building_id, meeting_id);
        let conv_id = conv.id;

        let owner_id = Uuid::new_v4();
        let recipient = make_recipient(conv_id, owner_id);
        let recipient_id = recipient.id;

        let mut recip_repo = MockRecipientRepo::new();
        let recip_clone = recipient.clone();
        recip_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(recip_clone.clone())));
        recip_repo.expect_update().returning(|r| Ok(r.clone()));
        recip_repo
            .expect_get_tracking_summary()
            .returning(move |_| {
                Ok(RecipientTrackingSummary {
                    total_count: 5,
                    opened_count: 3,
                    will_attend_count: 2,
                    will_not_attend_count: 1,
                    attended_count: 0,
                    did_not_attend_count: 0,
                    pending_count: 2,
                    failed_email_count: 0,
                })
            });

        let mut conv_repo = MockConvRepo::new();
        let conv_clone = conv.clone();
        conv_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(conv_clone.clone())));
        conv_repo.expect_update().returning(|conv| Ok(conv.clone()));

        let uc = make_use_cases(
            conv_repo,
            recip_repo,
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        let result = uc.mark_recipient_email_opened(recipient_id).await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let resp = result.unwrap();
        assert!(resp.has_opened_email);
    }

    // ---------------------------------------------------------------------------
    // Test 9: Update attendance status (Pending -> WillAttend)
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_update_attendance_will_attend() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let conv = make_sent_convocation(org_id, building_id, meeting_id);
        let conv_id = conv.id;

        let owner_id = Uuid::new_v4();
        let recipient = make_recipient(conv_id, owner_id);
        let recipient_id = recipient.id;

        let mut recip_repo = MockRecipientRepo::new();
        let recip_clone = recipient.clone();
        recip_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(recip_clone.clone())));
        recip_repo.expect_update().returning(|r| Ok(r.clone()));
        recip_repo
            .expect_get_tracking_summary()
            .returning(move |_| {
                Ok(RecipientTrackingSummary {
                    total_count: 5,
                    opened_count: 1,
                    will_attend_count: 1,
                    will_not_attend_count: 0,
                    attended_count: 0,
                    did_not_attend_count: 0,
                    pending_count: 4,
                    failed_email_count: 0,
                })
            });

        let mut conv_repo = MockConvRepo::new();
        let conv_clone = conv.clone();
        conv_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(conv_clone.clone())));
        conv_repo.expect_update().returning(|conv| Ok(conv.clone()));

        let uc = make_use_cases(
            conv_repo,
            recip_repo,
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        let result = uc
            .update_recipient_attendance(recipient_id, AttendanceStatus::WillAttend)
            .await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let resp = result.unwrap();
        assert_eq!(resp.attendance_status, AttendanceStatus::WillAttend);
        assert!(resp.has_confirmed_attendance);
    }

    // ---------------------------------------------------------------------------
    // Test 10: Set proxy delegation (Belgian "procuration")
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_set_proxy_delegation() {
        let conv_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let proxy_owner_id = Uuid::new_v4();
        let recipient = make_recipient(conv_id, owner_id);
        let recipient_id = recipient.id;

        let mut recip_repo = MockRecipientRepo::new();
        let recip_clone = recipient.clone();
        recip_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(recip_clone.clone())));
        recip_repo.expect_update().returning(|r| Ok(r.clone()));

        let uc = make_use_cases(
            MockConvRepo::new(),
            recip_repo,
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        let result = uc.set_recipient_proxy(recipient_id, proxy_owner_id).await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let resp = result.unwrap();
        assert_eq!(resp.proxy_owner_id, Some(proxy_owner_id));
    }

    // ---------------------------------------------------------------------------
    // Test 11: Set proxy to self -> error ("Cannot delegate to self")
    // ---------------------------------------------------------------------------
    #[tokio::test]
    async fn test_set_proxy_to_self_error() {
        let conv_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let recipient = make_recipient(conv_id, owner_id);
        let recipient_id = recipient.id;
        let self_owner_id = recipient.owner_id;

        let mut recip_repo = MockRecipientRepo::new();
        let recip_clone = recipient.clone();
        recip_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(recip_clone.clone())));

        let uc = make_use_cases(
            MockConvRepo::new(),
            recip_repo,
            MockOwnerRepo::new(),
            MockBuildingRepo::new(),
            MockMeetingRepo::new(),
        );

        let result = uc.set_recipient_proxy(recipient_id, self_owner_id).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot delegate to self"));
    }
}
