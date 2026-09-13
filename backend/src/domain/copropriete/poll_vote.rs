use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// PollVote - Individual vote cast on a poll
///
/// Tracks votes from owners on poll questions.
/// Can be anonymous or linked to specific owner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PollVote {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub owner_id: Option<Uuid>, // None if anonymous voting
    pub building_id: Uuid,

    // Vote details
    pub selected_option_ids: Vec<Uuid>, // Multiple for MultipleChoice with allow_multiple_votes
    pub rating_value: Option<i32>,      // For Rating polls (1-5)
    pub open_text: Option<String>,      // For OpenEnded polls

    // Metadata
    pub voted_at: DateTime<Utc>,
    pub ip_address: Option<String>, // For audit trail
}

impl PollVote {
    pub fn new(
        poll_id: Uuid,
        owner_id: Option<Uuid>,
        building_id: Uuid,
        selected_option_ids: Vec<Uuid>,
        rating_value: Option<i32>,
        open_text: Option<String>,
    ) -> Result<Self, String> {
        // Validation
        if selected_option_ids.is_empty() && rating_value.is_none() && open_text.is_none() {
            return Err("Vote must have at least one selection".to_string());
        }

        if let Some(rating) = rating_value {
            if !(1..=5).contains(&rating) {
                return Err("Rating must be between 1 and 5".to_string());
            }
        }

        Ok(Self {
            id: Uuid::new_v4(),
            poll_id,
            owner_id,
            building_id,
            selected_option_ids,
            rating_value,
            open_text,
            voted_at: Utc::now(),
            ip_address: None,
        })
    }

    /// Check if this is an anonymous vote
    pub fn is_anonymous(&self) -> bool {
        self.owner_id.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_vote_creation() {
        let vote = PollVote::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            Uuid::new_v4(),
            vec![Uuid::new_v4()],
            None,
            None,
        );

        assert!(vote.is_ok());
        let vote = vote.unwrap();
        assert!(!vote.is_anonymous());
        assert_eq!(vote.selected_option_ids.len(), 1);
    }

    #[test]
    fn test_anonymous_vote() {
        let vote = PollVote::new(
            Uuid::new_v4(),
            None, // Anonymous
            Uuid::new_v4(),
            vec![Uuid::new_v4()],
            None,
            None,
        );

        assert!(vote.is_ok());
        let vote = vote.unwrap();
        assert!(vote.is_anonymous());
    }

    #[test]
    fn test_rating_vote() {
        let vote = PollVote::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            Uuid::new_v4(),
            vec![],
            Some(4),
            None,
        );

        assert!(vote.is_ok());
        let vote = vote.unwrap();
        assert_eq!(vote.rating_value, Some(4));
    }

    #[test]
    fn test_rating_validation() {
        // Rating too low
        let result = PollVote::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            Uuid::new_v4(),
            vec![],
            Some(0),
            None,
        );
        assert!(result.is_err());

        // Rating too high
        let result = PollVote::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            Uuid::new_v4(),
            vec![],
            Some(6),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_open_ended_vote() {
        let vote = PollVote::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            Uuid::new_v4(),
            vec![],
            None,
            Some("Je préfère le bleu pour le hall".to_string()),
        );

        assert!(vote.is_ok());
        let vote = vote.unwrap();
        assert!(vote.open_text.is_some());
    }

    #[test]
    fn test_empty_vote() {
        let result = PollVote::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            Uuid::new_v4(),
            vec![],
            None,
            None,
        );

        assert!(result.is_err());
    }
}
