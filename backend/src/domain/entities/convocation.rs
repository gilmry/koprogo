use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Convocation type according to Belgian copropriété law
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConvocationType {
    /// Ordinary General Assembly (15 days minimum notice)
    Ordinary,
    /// Extraordinary General Assembly (8 days minimum notice)
    Extraordinary,
    /// Second convocation after quorum not reached (8 days minimum notice)
    SecondConvocation,
}

impl ConvocationType {
    /// Get minimum notice period in days according to Belgian law
    pub fn minimum_notice_days(&self) -> i64 {
        match self {
            ConvocationType::Ordinary => 15,
            ConvocationType::Extraordinary | ConvocationType::SecondConvocation => 8,
        }
    }

    /// Convert to database string
    pub fn to_db_string(&self) -> &'static str {
        match self {
            ConvocationType::Ordinary => "ordinary",
            ConvocationType::Extraordinary => "extraordinary",
            ConvocationType::SecondConvocation => "second_convocation",
        }
    }

    /// Parse from database string
    pub fn from_db_string(s: &str) -> Result<Self, String> {
        match s {
            "ordinary" => Ok(ConvocationType::Ordinary),
            "extraordinary" => Ok(ConvocationType::Extraordinary),
            "second_convocation" => Ok(ConvocationType::SecondConvocation),
            _ => Err(format!("Invalid meeting type: {}", s)),
        }
    }
}

/// Convocation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConvocationStatus {
    /// Draft (not yet sent)
    Draft,
    /// Scheduled (will be sent at scheduled time)
    Scheduled,
    /// Sent (emails dispatched)
    Sent,
    /// Cancelled (meeting cancelled)
    Cancelled,
}

impl ConvocationStatus {
    pub fn to_db_string(&self) -> &'static str {
        match self {
            ConvocationStatus::Draft => "draft",
            ConvocationStatus::Scheduled => "scheduled",
            ConvocationStatus::Sent => "sent",
            ConvocationStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_db_string(s: &str) -> Result<Self, String> {
        match s {
            "draft" => Ok(ConvocationStatus::Draft),
            "scheduled" => Ok(ConvocationStatus::Scheduled),
            "sent" => Ok(ConvocationStatus::Sent),
            "cancelled" => Ok(ConvocationStatus::Cancelled),
            _ => Err(format!("Invalid convocation status: {}", s)),
        }
    }
}

/// Convocation entity - Automatic meeting invitations with legal compliance
///
/// Implements Belgian copropriété legal requirements for meeting convocations:
/// - Ordinary AG: 15 days minimum notice
/// - Extraordinary AG: 8 days minimum notice
/// - Second convocation: 8 days minimum notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Convocation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub meeting_id: Uuid,
    pub meeting_type: ConvocationType,
    pub meeting_date: DateTime<Utc>,
    pub status: ConvocationStatus,

    // Legal deadline tracking
    pub minimum_send_date: DateTime<Utc>, // Latest date to send (meeting_date - minimum_notice_days)
    pub actual_send_date: Option<DateTime<Utc>>, // When actually sent
    pub scheduled_send_date: Option<DateTime<Utc>>, // When scheduled to be sent

    // PDF generation
    pub pdf_file_path: Option<String>, // Path to generated PDF
    pub language: String,              // FR, NL, DE, EN

    // Tracking
    pub total_recipients: i32,
    pub opened_count: i32,
    pub will_attend_count: i32,
    pub will_not_attend_count: i32,

    // Reminders
    pub reminder_sent_at: Option<DateTime<Utc>>, // J-3 reminder

    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl Convocation {
    /// Create a new convocation
    ///
    /// # Arguments
    /// * `organization_id` - Organization ID
    /// * `building_id` - Building ID
    /// * `meeting_id` - Meeting ID
    /// * `meeting_type` - Type of meeting (Ordinary/Extraordinary/Second)
    /// * `meeting_date` - Scheduled meeting date
    /// * `language` - Convocation language (FR/NL/DE/EN)
    /// * `created_by` - User creating the convocation
    ///
    /// # Returns
    /// Result with Convocation or error if meeting date is too soon
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        meeting_id: Uuid,
        meeting_type: ConvocationType,
        meeting_date: DateTime<Utc>,
        language: String,
        created_by: Uuid,
    ) -> Result<Self, String> {
        // Validate language
        if !["FR", "NL", "DE", "EN"].contains(&language.to_uppercase().as_str()) {
            return Err(format!(
                "Invalid language '{}'. Must be FR, NL, DE, or EN",
                language
            ));
        }

        // Calculate minimum send date (meeting_date - minimum_notice_days)
        let minimum_notice_days = meeting_type.minimum_notice_days();
        let minimum_send_date = meeting_date - Duration::days(minimum_notice_days);

        // Check if meeting date allows for legal notice period
        let now = Utc::now();
        if minimum_send_date < now {
            return Err(format!(
                "Meeting date too soon. {} meeting requires {} days notice. Minimum send date would be {}",
                match meeting_type {
                    ConvocationType::Ordinary => "Ordinary",
                    ConvocationType::Extraordinary => "Extraordinary",
                    ConvocationType::SecondConvocation => "Second convocation",
                },
                minimum_notice_days,
                minimum_send_date.format("%Y-%m-%d %H:%M")
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            meeting_id,
            meeting_type,
            meeting_date,
            status: ConvocationStatus::Draft,
            minimum_send_date,
            actual_send_date: None,
            scheduled_send_date: None,
            pdf_file_path: None,
            language: language.to_uppercase(),
            total_recipients: 0,
            opened_count: 0,
            will_attend_count: 0,
            will_not_attend_count: 0,
            reminder_sent_at: None,
            created_at: now,
            updated_at: now,
            created_by,
        })
    }

    /// Schedule convocation to be sent at specific date
    pub fn schedule(&mut self, send_date: DateTime<Utc>) -> Result<(), String> {
        if self.status != ConvocationStatus::Draft {
            return Err(format!(
                "Cannot schedule convocation in status '{:?}'. Must be Draft",
                self.status
            ));
        }

        // Verify send_date is before meeting_date - minimum_notice_days
        if send_date > self.minimum_send_date {
            return Err(format!(
                "Scheduled send date {} is after minimum send date {}. Meeting would not have required notice period",
                send_date.format("%Y-%m-%d %H:%M"),
                self.minimum_send_date.format("%Y-%m-%d %H:%M")
            ));
        }

        self.scheduled_send_date = Some(send_date);
        self.status = ConvocationStatus::Scheduled;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark convocation as sent
    pub fn mark_sent(
        &mut self,
        pdf_file_path: String,
        total_recipients: i32,
    ) -> Result<(), String> {
        if self.status != ConvocationStatus::Draft && self.status != ConvocationStatus::Scheduled {
            return Err(format!(
                "Cannot send convocation in status '{:?}'",
                self.status
            ));
        }

        if total_recipients <= 0 {
            return Err("Total recipients must be greater than 0".to_string());
        }

        self.status = ConvocationStatus::Sent;
        self.actual_send_date = Some(Utc::now());
        self.pdf_file_path = Some(pdf_file_path);
        self.total_recipients = total_recipients;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancel convocation
    pub fn cancel(&mut self) -> Result<(), String> {
        if self.status == ConvocationStatus::Cancelled {
            return Err("Convocation is already cancelled".to_string());
        }

        self.status = ConvocationStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark reminder as sent (J-3)
    pub fn mark_reminder_sent(&mut self) -> Result<(), String> {
        if self.status != ConvocationStatus::Sent {
            return Err("Cannot send reminder for unsent convocation".to_string());
        }

        self.reminder_sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update tracking counts from recipients
    pub fn update_tracking_counts(
        &mut self,
        opened_count: i32,
        will_attend_count: i32,
        will_not_attend_count: i32,
    ) {
        self.opened_count = opened_count;
        self.will_attend_count = will_attend_count;
        self.will_not_attend_count = will_not_attend_count;
        self.updated_at = Utc::now();
    }

    /// Check if convocation respects legal deadline
    pub fn respects_legal_deadline(&self) -> bool {
        match &self.actual_send_date {
            Some(sent_at) => *sent_at <= self.minimum_send_date,
            None => false, // Not sent yet
        }
    }

    /// Get days until meeting
    pub fn days_until_meeting(&self) -> i64 {
        let now = Utc::now();
        let duration = self.meeting_date.signed_duration_since(now);
        duration.num_days()
    }

    /// Check if reminder should be sent (3 days before meeting)
    pub fn should_send_reminder(&self) -> bool {
        if self.status != ConvocationStatus::Sent {
            return false;
        }

        if self.reminder_sent_at.is_some() {
            return false; // Already sent
        }

        let days_until = self.days_until_meeting();
        days_until <= 3 && days_until >= 0
    }

    /// Get opening rate (percentage of recipients who opened)
    pub fn opening_rate(&self) -> f64 {
        if self.total_recipients == 0 {
            return 0.0;
        }
        (self.opened_count as f64 / self.total_recipients as f64) * 100.0
    }

    /// Get attendance rate (percentage confirmed attending)
    pub fn attendance_rate(&self) -> f64 {
        if self.total_recipients == 0 {
            return 0.0;
        }
        (self.will_attend_count as f64 / self.total_recipients as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meeting_type_minimum_notice_days() {
        assert_eq!(ConvocationType::Ordinary.minimum_notice_days(), 15);
        assert_eq!(ConvocationType::Extraordinary.minimum_notice_days(), 8);
        assert_eq!(ConvocationType::SecondConvocation.minimum_notice_days(), 8);
    }

    #[test]
    fn test_create_convocation_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();
        let meeting_date = Utc::now() + Duration::days(20);

        let convocation = Convocation::new(
            org_id,
            building_id,
            meeting_id,
            ConvocationType::Ordinary,
            meeting_date,
            "FR".to_string(),
            creator_id,
        );

        assert!(convocation.is_ok());
        let conv = convocation.unwrap();
        assert_eq!(conv.meeting_type, ConvocationType::Ordinary);
        assert_eq!(conv.language, "FR");
        assert_eq!(conv.status, ConvocationStatus::Draft);
        assert_eq!(conv.total_recipients, 0);
    }

    #[test]
    fn test_create_convocation_meeting_too_soon() {
        let meeting_date = Utc::now() + Duration::days(5); // Only 5 days notice for ordinary meeting

        let result = Convocation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ConvocationType::Ordinary, // Requires 15 days
            meeting_date,
            "FR".to_string(),
            Uuid::new_v4(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Meeting date too soon"));
    }

    #[test]
    fn test_create_convocation_invalid_language() {
        let meeting_date = Utc::now() + Duration::days(20);

        let result = Convocation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ConvocationType::Ordinary,
            meeting_date,
            "ES".to_string(), // Spanish not supported
            Uuid::new_v4(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid language"));
    }

    #[test]
    fn test_schedule_convocation() {
        let meeting_date = Utc::now() + Duration::days(20);
        let mut convocation = Convocation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ConvocationType::Ordinary,
            meeting_date,
            "FR".to_string(),
            Uuid::new_v4(),
        )
        .unwrap();

        let send_date = Utc::now() + Duration::days(3); // Send in 3 days
        let result = convocation.schedule(send_date);

        assert!(result.is_ok());
        assert_eq!(convocation.status, ConvocationStatus::Scheduled);
        assert_eq!(convocation.scheduled_send_date, Some(send_date));
    }

    #[test]
    fn test_schedule_convocation_too_late() {
        let meeting_date = Utc::now() + Duration::days(20);
        let mut convocation = Convocation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ConvocationType::Ordinary,
            meeting_date,
            "FR".to_string(),
            Uuid::new_v4(),
        )
        .unwrap();

        // Try to schedule send date after minimum_send_date
        let send_date = meeting_date - Duration::days(10); // Only 10 days before (needs 15)
        let result = convocation.schedule(send_date);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("after minimum send date"));
    }

    #[test]
    fn test_mark_sent() {
        let meeting_date = Utc::now() + Duration::days(20);
        let mut convocation = Convocation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ConvocationType::Ordinary,
            meeting_date,
            "FR".to_string(),
            Uuid::new_v4(),
        )
        .unwrap();

        let result = convocation.mark_sent("/uploads/convocations/conv-123.pdf".to_string(), 50);

        assert!(result.is_ok());
        assert_eq!(convocation.status, ConvocationStatus::Sent);
        assert!(convocation.actual_send_date.is_some());
        assert_eq!(convocation.total_recipients, 50);
        assert_eq!(
            convocation.pdf_file_path,
            Some("/uploads/convocations/conv-123.pdf".to_string())
        );
    }

    #[test]
    fn test_should_send_reminder() {
        let meeting_date = Utc::now() + Duration::days(3); // 3 days until meeting (exactly on threshold)
        let mut convocation = Convocation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ConvocationType::Extraordinary, // 8 days notice
            meeting_date,
            "FR".to_string(),
            Uuid::new_v4(),
        )
        .unwrap();

        convocation
            .mark_sent("/uploads/conv.pdf".to_string(), 30)
            .unwrap();

        assert!(convocation.should_send_reminder());

        // After marking reminder sent
        convocation.mark_reminder_sent().unwrap();
        assert!(!convocation.should_send_reminder());
    }

    #[test]
    fn test_opening_rate() {
        let meeting_date = Utc::now() + Duration::days(20);
        let mut convocation = Convocation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ConvocationType::Ordinary,
            meeting_date,
            "FR".to_string(),
            Uuid::new_v4(),
        )
        .unwrap();

        convocation
            .mark_sent("/uploads/conv.pdf".to_string(), 100)
            .unwrap();
        convocation.update_tracking_counts(75, 50, 10);

        assert_eq!(convocation.opening_rate(), 75.0);
        assert_eq!(convocation.attendance_rate(), 50.0);
    }

    #[test]
    fn test_respects_legal_deadline() {
        let meeting_date = Utc::now() + Duration::days(20);
        let mut convocation = Convocation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ConvocationType::Ordinary,
            meeting_date,
            "FR".to_string(),
            Uuid::new_v4(),
        )
        .unwrap();

        // Before sending
        assert!(!convocation.respects_legal_deadline());

        // After sending (now is definitely before minimum_send_date)
        convocation
            .mark_sent("/uploads/conv.pdf".to_string(), 30)
            .unwrap();
        assert!(convocation.respects_legal_deadline());
    }
}
