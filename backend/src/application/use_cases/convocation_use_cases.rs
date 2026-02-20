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
