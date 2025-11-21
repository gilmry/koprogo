use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Poll - Sondage pour décisions du conseil de copropriété
///
/// Permet au conseil de consulter rapidement les résidents sur des décisions:
/// - Choix entrepreneur (avec devis attachés)
/// - Couleur de peinture
/// - Horaires de travaux
/// - Décisions mineures entre AG
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Poll {
    pub id: Uuid,
    pub building_id: Uuid,
    pub created_by: Uuid, // Board member or syndic who created the poll

    // Poll details
    pub title: String,
    pub description: Option<String>,
    pub poll_type: PollType,
    pub options: Vec<PollOption>, // Empty for OpenEnded polls
    pub is_anonymous: bool,

    // Voting settings
    pub allow_multiple_votes: bool, // For MultipleChoice polls
    pub require_all_owners: bool,   // Require 100% participation

    // Schedule
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,

    // Status
    pub status: PollStatus,
    pub total_eligible_voters: i32, // Total owners who can vote
    pub total_votes_cast: i32,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PollType {
    YesNo,          // Simple yes/no question
    MultipleChoice, // Choose one or multiple options
    Rating,         // Rate 1-5 stars
    OpenEnded,      // Free text responses
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    Draft,     // Not yet published
    Active,    // Currently accepting votes
    Closed,    // Voting period ended
    Cancelled, // Poll cancelled before completion
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PollOption {
    pub id: Uuid,
    pub option_text: String,
    pub attachment_url: Option<String>, // PDF devis, image, etc.
    pub vote_count: i32,
    pub display_order: i32,
}

impl Poll {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        building_id: Uuid,
        created_by: Uuid,
        title: String,
        description: Option<String>,
        poll_type: PollType,
        options: Vec<PollOption>,
        is_anonymous: bool,
        ends_at: DateTime<Utc>,
        total_eligible_voters: i32,
    ) -> Result<Self, String> {
        // Validation
        if title.trim().is_empty() {
            return Err("Poll title cannot be empty".to_string());
        }

        if ends_at <= Utc::now() {
            return Err("Poll end date must be in the future".to_string());
        }

        if total_eligible_voters <= 0 {
            return Err("Total eligible voters must be positive".to_string());
        }

        // Validate options based on poll type
        match poll_type {
            PollType::YesNo => {
                if options.len() != 2 {
                    return Err("Yes/No polls must have exactly 2 options".to_string());
                }
            }
            PollType::MultipleChoice => {
                if options.len() < 2 {
                    return Err("Multiple choice polls must have at least 2 options".to_string());
                }
            }
            PollType::Rating => {
                if options.len() != 5 {
                    return Err("Rating polls must have exactly 5 options (1-5 stars)".to_string());
                }
            }
            PollType::OpenEnded => {
                if !options.is_empty() {
                    return Err("Open-ended polls cannot have predefined options".to_string());
                }
            }
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            building_id,
            created_by,
            title,
            description,
            poll_type,
            options,
            is_anonymous,
            allow_multiple_votes: false,
            require_all_owners: false,
            starts_at: now,
            ends_at,
            status: PollStatus::Draft,
            total_eligible_voters,
            total_votes_cast: 0,
            created_at: now,
            updated_at: now,
        })
    }

    /// Publish the poll (make it active)
    pub fn publish(&mut self) -> Result<(), String> {
        if self.status != PollStatus::Draft {
            return Err("Only draft polls can be published".to_string());
        }

        self.status = PollStatus::Active;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Close the poll (end voting)
    pub fn close(&mut self) -> Result<(), String> {
        if self.status != PollStatus::Active {
            return Err("Only active polls can be closed".to_string());
        }

        self.status = PollStatus::Closed;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancel the poll
    pub fn cancel(&mut self) -> Result<(), String> {
        if self.status == PollStatus::Closed {
            return Err("Cannot cancel a closed poll".to_string());
        }

        self.status = PollStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if poll is currently active and accepting votes
    pub fn is_active(&self) -> bool {
        self.status == PollStatus::Active
            && Utc::now() >= self.starts_at
            && Utc::now() <= self.ends_at
    }

    /// Check if poll has ended
    pub fn is_ended(&self) -> bool {
        Utc::now() > self.ends_at || self.status == PollStatus::Closed
    }

    /// Calculate participation rate
    pub fn participation_rate(&self) -> f64 {
        if self.total_eligible_voters == 0 {
            return 0.0;
        }
        (self.total_votes_cast as f64 / self.total_eligible_voters as f64) * 100.0
    }

    /// Get winning option (for closed polls)
    pub fn get_winning_option(&self) -> Option<&PollOption> {
        if self.status != PollStatus::Closed {
            return None;
        }

        self.options.iter().max_by_key(|opt| opt.vote_count)
    }

    /// Update vote count for an option
    pub fn record_vote(&mut self, option_id: Uuid) -> Result<(), String> {
        if !self.is_active() {
            return Err("Poll is not currently accepting votes".to_string());
        }

        let option = self
            .options
            .iter_mut()
            .find(|opt| opt.id == option_id)
            .ok_or("Option not found".to_string())?;

        option.vote_count += 1;
        self.total_votes_cast += 1;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Auto-close if past end date
    pub fn auto_close_if_ended(&mut self) -> bool {
        if self.status == PollStatus::Active && Utc::now() > self.ends_at {
            self.status = PollStatus::Closed;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
}

impl PollOption {
    pub fn new(option_text: String, attachment_url: Option<String>, display_order: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            option_text,
            attachment_url,
            vote_count: 0,
            display_order,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_creation_yes_no() {
        let options = vec![
            PollOption::new("Oui".to_string(), None, 1),
            PollOption::new("Non".to_string(), None, 2),
        ];

        let poll = Poll::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Repeindre le hall en vert?".to_string(),
            Some("Vote pour couleur hall".to_string()),
            PollType::YesNo,
            options,
            false,
            Utc::now() + chrono::Duration::days(7),
            60,
        );

        assert!(poll.is_ok());
        let poll = poll.unwrap();
        assert_eq!(poll.status, PollStatus::Draft);
        assert_eq!(poll.options.len(), 2);
        assert!(!poll.is_active());
    }

    #[test]
    fn test_poll_publish_and_close() {
        let options = vec![
            PollOption::new("Oui".to_string(), None, 1),
            PollOption::new("Non".to_string(), None, 2),
        ];

        let mut poll = Poll::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test poll".to_string(),
            None,
            PollType::YesNo,
            options,
            false,
            Utc::now() + chrono::Duration::days(7),
            60,
        )
        .unwrap();

        // Publish poll
        assert!(poll.publish().is_ok());
        assert_eq!(poll.status, PollStatus::Active);
        assert!(poll.is_active());

        // Cannot publish again
        assert!(poll.publish().is_err());

        // Close poll
        assert!(poll.close().is_ok());
        assert_eq!(poll.status, PollStatus::Closed);
        assert!(!poll.is_active());
    }

    #[test]
    fn test_poll_record_vote() {
        let options = vec![
            PollOption::new("Option A".to_string(), None, 1),
            PollOption::new("Option B".to_string(), None, 2),
        ];

        let mut poll = Poll::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Choose contractor".to_string(),
            None,
            PollType::MultipleChoice,
            options.clone(),
            false,
            Utc::now() + chrono::Duration::days(7),
            60,
        )
        .unwrap();

        // Cannot vote on draft poll
        assert!(poll.record_vote(options[0].id).is_err());

        // Publish poll
        poll.publish().unwrap();

        // Record vote
        assert!(poll.record_vote(options[0].id).is_ok());
        assert_eq!(poll.total_votes_cast, 1);
        assert_eq!(poll.options[0].vote_count, 1);
    }

    #[test]
    fn test_poll_participation_rate() {
        let options = vec![
            PollOption::new("Oui".to_string(), None, 1),
            PollOption::new("Non".to_string(), None, 2),
        ];

        let mut poll = Poll::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            None,
            PollType::YesNo,
            options,
            false,
            Utc::now() + chrono::Duration::days(7),
            100,
        )
        .unwrap();

        poll.total_votes_cast = 45;
        assert_eq!(poll.participation_rate(), 45.0);

        poll.total_votes_cast = 100;
        assert_eq!(poll.participation_rate(), 100.0);
    }

    #[test]
    fn test_poll_validation() {
        // Empty title
        let result = Poll::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "".to_string(),
            None,
            PollType::YesNo,
            vec![],
            false,
            Utc::now() + chrono::Duration::days(7),
            60,
        );
        assert!(result.is_err());

        // Past end date
        let result = Poll::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            None,
            PollType::YesNo,
            vec![],
            false,
            Utc::now() - chrono::Duration::days(1),
            60,
        );
        assert!(result.is_err());
    }
}
