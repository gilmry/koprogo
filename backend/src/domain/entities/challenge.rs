use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Challenge status lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChallengeStatus {
    Draft,      // Challenge being created
    Active,     // Challenge is live
    Completed,  // Challenge period ended
    Cancelled,  // Challenge cancelled
}

/// Challenge type for different engagement patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChallengeType {
    Individual, // Each user competes individually
    Team,       // Users work together towards shared goal
    Building,   // Entire building works towards goal
}

/// Challenge entity - Time-bound goals to encourage community engagement
///
/// Represents a specific challenge or contest with a defined timeframe.
/// Challenges motivate participation through gamification.
///
/// # Belgian Context
/// - Encourages active copropriété community participation
/// - Promotes use of platform features (SEL, bookings, etc.)
/// - Builds community spirit through shared goals
///
/// # Business Rules
/// - title must be 3-100 characters
/// - description must be 10-1000 characters
/// - start_date must be < end_date
/// - target_value must be > 0
/// - reward_points must be 0-10000
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Option<Uuid>, // None = organization-wide
    pub challenge_type: ChallengeType,
    pub status: ChallengeStatus,
    pub title: String,              // e.g., "Community Booking Week", "SEL Exchange Marathon"
    pub description: String,        // Challenge details and rules
    pub icon: String,               // Emoji or icon URL
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub target_metric: String,      // e.g., "bookings_created", "sel_exchanges_completed"
    pub target_value: i32,          // Target count to achieve
    pub reward_points: i32,         // Points awarded for completion (0-10000)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Challenge {
    /// Minimum title length
    pub const MIN_TITLE_LENGTH: usize = 3;
    /// Maximum title length
    pub const MAX_TITLE_LENGTH: usize = 100;
    /// Minimum description length
    pub const MIN_DESCRIPTION_LENGTH: usize = 10;
    /// Maximum description length
    pub const MAX_DESCRIPTION_LENGTH: usize = 1000;
    /// Maximum reward points
    pub const MAX_REWARD_POINTS: i32 = 10000;

    /// Create a new challenge
    ///
    /// # Validation
    /// - title must be 3-100 characters
    /// - description must be 10-1000 characters
    /// - start_date must be < end_date
    /// - start_date must be in the future
    /// - target_value must be > 0
    /// - reward_points must be 0-10000
    /// - icon must not be empty
    pub fn new(
        organization_id: Uuid,
        building_id: Option<Uuid>,
        challenge_type: ChallengeType,
        title: String,
        description: String,
        icon: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        target_metric: String,
        target_value: i32,
        reward_points: i32,
    ) -> Result<Self, String> {
        // Validate title
        if title.len() < Self::MIN_TITLE_LENGTH || title.len() > Self::MAX_TITLE_LENGTH {
            return Err(format!(
                "Challenge title must be {}-{} characters",
                Self::MIN_TITLE_LENGTH,
                Self::MAX_TITLE_LENGTH
            ));
        }

        // Validate description
        if description.len() < Self::MIN_DESCRIPTION_LENGTH
            || description.len() > Self::MAX_DESCRIPTION_LENGTH
        {
            return Err(format!(
                "Challenge description must be {}-{} characters",
                Self::MIN_DESCRIPTION_LENGTH,
                Self::MAX_DESCRIPTION_LENGTH
            ));
        }

        // Validate icon
        if icon.trim().is_empty() {
            return Err("Challenge icon cannot be empty".to_string());
        }

        // Validate dates
        if start_date >= end_date {
            return Err("Start date must be before end date".to_string());
        }

        let now = Utc::now();
        if start_date <= now {
            return Err("Challenge start date must be in the future".to_string());
        }

        // Validate target
        if target_value <= 0 {
            return Err("Target value must be greater than 0".to_string());
        }

        // Validate reward points
        if reward_points < 0 || reward_points > Self::MAX_REWARD_POINTS {
            return Err(format!(
                "Reward points must be 0-{} points",
                Self::MAX_REWARD_POINTS
            ));
        }

        // Validate metric
        if target_metric.trim().is_empty() {
            return Err("Target metric cannot be empty".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            challenge_type,
            status: ChallengeStatus::Draft,
            title,
            description,
            icon,
            start_date,
            end_date,
            target_metric,
            target_value,
            reward_points,
            created_at: now,
            updated_at: now,
        })
    }

    /// Activate the challenge (Draft → Active)
    pub fn activate(&mut self) -> Result<(), String> {
        match self.status {
            ChallengeStatus::Draft => {
                self.status = ChallengeStatus::Active;
                self.updated_at = Utc::now();
                Ok(())
            }
            ChallengeStatus::Active => Err("Challenge is already active".to_string()),
            ChallengeStatus::Completed => Err("Cannot activate a completed challenge".to_string()),
            ChallengeStatus::Cancelled => {
                Err("Cannot activate a cancelled challenge".to_string())
            }
        }
    }

    /// Complete the challenge (Active → Completed)
    pub fn complete(&mut self) -> Result<(), String> {
        match self.status {
            ChallengeStatus::Active => {
                self.status = ChallengeStatus::Completed;
                self.updated_at = Utc::now();
                Ok(())
            }
            ChallengeStatus::Draft => Err("Cannot complete a draft challenge".to_string()),
            ChallengeStatus::Completed => Err("Challenge is already completed".to_string()),
            ChallengeStatus::Cancelled => {
                Err("Cannot complete a cancelled challenge".to_string())
            }
        }
    }

    /// Cancel the challenge
    pub fn cancel(&mut self) -> Result<(), String> {
        match self.status {
            ChallengeStatus::Draft | ChallengeStatus::Active => {
                self.status = ChallengeStatus::Cancelled;
                self.updated_at = Utc::now();
                Ok(())
            }
            ChallengeStatus::Completed => Err("Cannot cancel a completed challenge".to_string()),
            ChallengeStatus::Cancelled => Err("Challenge is already cancelled".to_string()),
        }
    }

    /// Check if challenge is currently active (now >= start_date AND now < end_date AND status = Active)
    pub fn is_currently_active(&self) -> bool {
        let now = Utc::now();
        self.status == ChallengeStatus::Active && now >= self.start_date && now < self.end_date
    }

    /// Check if challenge has ended (now >= end_date)
    pub fn has_ended(&self) -> bool {
        Utc::now() >= self.end_date
    }

    /// Calculate duration in days
    pub fn duration_days(&self) -> i64 {
        self.end_date.signed_duration_since(self.start_date).num_days()
    }

    /// Update challenge details (only allowed for Draft challenges)
    pub fn update(
        &mut self,
        title: Option<String>,
        description: Option<String>,
        icon: Option<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        target_value: Option<i32>,
        reward_points: Option<i32>,
    ) -> Result<(), String> {
        // Only allow updates for Draft challenges
        if self.status != ChallengeStatus::Draft {
            return Err("Can only update draft challenges".to_string());
        }

        // Update title if provided
        if let Some(t) = title {
            if t.len() < Self::MIN_TITLE_LENGTH || t.len() > Self::MAX_TITLE_LENGTH {
                return Err(format!(
                    "Challenge title must be {}-{} characters",
                    Self::MIN_TITLE_LENGTH,
                    Self::MAX_TITLE_LENGTH
                ));
            }
            self.title = t;
        }

        // Update description if provided
        if let Some(d) = description {
            if d.len() < Self::MIN_DESCRIPTION_LENGTH || d.len() > Self::MAX_DESCRIPTION_LENGTH {
                return Err(format!(
                    "Challenge description must be {}-{} characters",
                    Self::MIN_DESCRIPTION_LENGTH,
                    Self::MAX_DESCRIPTION_LENGTH
                ));
            }
            self.description = d;
        }

        // Update icon if provided
        if let Some(i) = icon {
            if i.trim().is_empty() {
                return Err("Challenge icon cannot be empty".to_string());
            }
            self.icon = i;
        }

        // Update dates if provided (with validation)
        if start_date.is_some() || end_date.is_some() {
            let new_start = start_date.unwrap_or(self.start_date);
            let new_end = end_date.unwrap_or(self.end_date);

            if new_start >= new_end {
                return Err("Start date must be before end date".to_string());
            }

            self.start_date = new_start;
            self.end_date = new_end;
        }

        // Update target value if provided
        if let Some(tv) = target_value {
            if tv <= 0 {
                return Err("Target value must be greater than 0".to_string());
            }
            self.target_value = tv;
        }

        // Update reward points if provided
        if let Some(rp) = reward_points {
            if rp < 0 || rp > Self::MAX_REWARD_POINTS {
                return Err(format!(
                    "Reward points must be 0-{} points",
                    Self::MAX_REWARD_POINTS
                ));
            }
            self.reward_points = rp;
        }

        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Challenge progress tracking for individual users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeProgress {
    pub id: Uuid,
    pub challenge_id: Uuid,
    pub user_id: Uuid,
    pub current_value: i32,     // Current progress (e.g., 5 bookings out of 10)
    pub completed: bool,        // Has user completed the challenge?
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ChallengeProgress {
    /// Start tracking progress for a challenge
    pub fn new(challenge_id: Uuid, user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            challenge_id,
            user_id,
            current_value: 0,
            completed: false,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Increment progress by amount
    pub fn increment(&mut self, amount: i32) -> Result<(), String> {
        if self.completed {
            return Err("Cannot increment progress on completed challenge".to_string());
        }

        self.current_value += amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark challenge as completed
    pub fn mark_completed(&mut self) -> Result<(), String> {
        if self.completed {
            return Err("Challenge is already completed".to_string());
        }

        self.completed = true;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Calculate completion percentage
    pub fn completion_percentage(&self, target_value: i32) -> f64 {
        if target_value <= 0 {
            return 0.0;
        }
        (self.current_value as f64 / target_value as f64 * 100.0).min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_challenge() -> Challenge {
        let organization_id = Uuid::new_v4();
        let start_date = Utc::now() + chrono::Duration::days(1);
        let end_date = start_date + chrono::Duration::days(7);

        Challenge::new(
            organization_id,
            None,
            ChallengeType::Individual,
            "Booking Week".to_string(),
            "Make 5 resource bookings this week to earn points!".to_string(),
            "📅".to_string(),
            start_date,
            end_date,
            "bookings_created".to_string(),
            5,
            50,
        )
        .unwrap()
    }

    #[test]
    fn test_create_challenge_success() {
        let challenge = create_test_challenge();
        assert_eq!(challenge.title, "Booking Week");
        assert_eq!(challenge.challenge_type, ChallengeType::Individual);
        assert_eq!(challenge.status, ChallengeStatus::Draft);
        assert_eq!(challenge.target_value, 5);
        assert_eq!(challenge.reward_points, 50);
    }

    #[test]
    fn test_create_challenge_invalid_title() {
        let organization_id = Uuid::new_v4();
        let start_date = Utc::now() + chrono::Duration::days(1);
        let end_date = start_date + chrono::Duration::days(7);

        let result = Challenge::new(
            organization_id,
            None,
            ChallengeType::Individual,
            "AB".to_string(), // Too short
            "Make 5 resource bookings this week to earn points!".to_string(),
            "📅".to_string(),
            start_date,
            end_date,
            "bookings_created".to_string(),
            5,
            50,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Challenge title must be"));
    }

    #[test]
    fn test_create_challenge_invalid_dates() {
        let organization_id = Uuid::new_v4();
        let start_date = Utc::now() + chrono::Duration::days(7);
        let end_date = start_date - chrono::Duration::days(1); // End before start

        let result = Challenge::new(
            organization_id,
            None,
            ChallengeType::Individual,
            "Booking Week".to_string(),
            "Make 5 resource bookings this week to earn points!".to_string(),
            "📅".to_string(),
            start_date,
            end_date,
            "bookings_created".to_string(),
            5,
            50,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Start date must be before end date"));
    }

    #[test]
    fn test_create_challenge_past_start_date() {
        let organization_id = Uuid::new_v4();
        let start_date = Utc::now() - chrono::Duration::days(1); // Past
        let end_date = start_date + chrono::Duration::days(7);

        let result = Challenge::new(
            organization_id,
            None,
            ChallengeType::Individual,
            "Booking Week".to_string(),
            "Make 5 resource bookings this week to earn points!".to_string(),
            "📅".to_string(),
            start_date,
            end_date,
            "bookings_created".to_string(),
            5,
            50,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Challenge start date must be in the future"));
    }

    #[test]
    fn test_activate_challenge() {
        let mut challenge = create_test_challenge();
        let result = challenge.activate();
        assert!(result.is_ok());
        assert_eq!(challenge.status, ChallengeStatus::Active);
    }

    #[test]
    fn test_complete_challenge() {
        let mut challenge = create_test_challenge();
        challenge.activate().unwrap();
        let result = challenge.complete();
        assert!(result.is_ok());
        assert_eq!(challenge.status, ChallengeStatus::Completed);
    }

    #[test]
    fn test_cancel_challenge() {
        let mut challenge = create_test_challenge();
        let result = challenge.cancel();
        assert!(result.is_ok());
        assert_eq!(challenge.status, ChallengeStatus::Cancelled);
    }

    #[test]
    fn test_duration_days() {
        let challenge = create_test_challenge();
        assert_eq!(challenge.duration_days(), 7);
    }

    #[test]
    fn test_challenge_progress_new() {
        let challenge_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let progress = ChallengeProgress::new(challenge_id, user_id);

        assert_eq!(progress.challenge_id, challenge_id);
        assert_eq!(progress.user_id, user_id);
        assert_eq!(progress.current_value, 0);
        assert!(!progress.completed);
    }

    #[test]
    fn test_challenge_progress_increment() {
        let challenge_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut progress = ChallengeProgress::new(challenge_id, user_id);

        progress.increment(3).unwrap();
        assert_eq!(progress.current_value, 3);

        progress.increment(2).unwrap();
        assert_eq!(progress.current_value, 5);
    }

    #[test]
    fn test_challenge_progress_mark_completed() {
        let challenge_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut progress = ChallengeProgress::new(challenge_id, user_id);

        progress.mark_completed().unwrap();
        assert!(progress.completed);
        assert!(progress.completed_at.is_some());
    }

    #[test]
    fn test_challenge_progress_completion_percentage() {
        let challenge_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut progress = ChallengeProgress::new(challenge_id, user_id);

        progress.increment(3).unwrap();
        assert_eq!(progress.completion_percentage(10), 30.0);

        progress.increment(7).unwrap();
        assert_eq!(progress.completion_percentage(10), 100.0);
    }

    #[test]
    fn test_update_challenge_only_draft() {
        let mut challenge = create_test_challenge();
        challenge.activate().unwrap();

        let result = challenge.update(
            Some("Updated Title".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Can only update draft challenges"));
    }
}
