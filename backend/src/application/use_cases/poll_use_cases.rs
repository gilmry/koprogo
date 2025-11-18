use crate::application::dto::{
    CreatePollDto, PollFilters, PollListResponseDto, PollResponseDto, UpdatePollDto,
    CastVoteDto, PollResultsDto, PollOptionDto, PageRequest,
};
use crate::application::ports::{PollRepository, PollVoteRepository, OwnerRepository};
use crate::domain::entities::{Poll, PollVote, PollStatus, PollType};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct PollUseCases {
    poll_repository: Arc<dyn PollRepository>,
    poll_vote_repository: Arc<dyn PollVoteRepository>,
    owner_repository: Arc<dyn OwnerRepository>,
}

impl PollUseCases {
    pub fn new(
        poll_repository: Arc<dyn PollRepository>,
        poll_vote_repository: Arc<dyn PollVoteRepository>,
        owner_repository: Arc<dyn OwnerRepository>,
    ) -> Self {
        Self {
            poll_repository,
            poll_vote_repository,
            owner_repository,
        }
    }

    /// Create a new poll (draft status)
    pub async fn create_poll(
        &self,
        dto: CreatePollDto,
        created_by: Uuid,
    ) -> Result<PollResponseDto, String> {
        // Parse UUIDs
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building ID format".to_string())?;

        // Parse end date
        let ends_at = DateTime::parse_from_rfc3339(&dto.ends_at)
            .map_err(|_| "Invalid ends_at date format (expected RFC3339)".to_string())?
            .with_timezone(&Utc);

        // Validate end date is in future
        if ends_at <= Utc::now() {
            return Err("Poll end date must be in the future".to_string());
        }

        // Convert DTO poll type to domain poll type
        let poll_type = match dto.poll_type.as_str() {
            "yes_no" => PollType::YesNo,
            "multiple_choice" => PollType::MultipleChoice,
            "rating" => PollType::Rating,
            "open_ended" => PollType::OpenEnded,
            _ => return Err("Invalid poll type".to_string()),
        };

        // Convert options
        let options = dto
            .options
            .iter()
            .map(|opt| crate::domain::entities::PollOption {
                id: Uuid::new_v4(),
                option_text: opt.option_text.clone(),
                attachment_url: opt.attachment_url.clone(),
                vote_count: 0,
                display_order: opt.display_order,
            })
            .collect();

        // TODO: Count eligible voters by querying unit_owners for building
        // For now, use a default value or require it in the DTO
        let total_eligible_voters = 10; // Default placeholder

        // Create poll entity
        let mut poll = Poll::new(
            building_id,
            created_by,
            dto.title.clone(),
            dto.description.clone(),
            poll_type,
            options,
            dto.is_anonymous.unwrap_or(false),
            ends_at,
            total_eligible_voters,
        )?;

        // Set optional fields
        poll.allow_multiple_votes = dto.allow_multiple_votes.unwrap_or(false);
        poll.require_all_owners = dto.require_all_owners.unwrap_or(false);

        // Save to repository
        let created_poll = self.poll_repository.create(&poll).await?;

        Ok(PollResponseDto::from(created_poll))
    }

    /// Update an existing poll (only if in draft status)
    pub async fn update_poll(
        &self,
        poll_id: Uuid,
        dto: UpdatePollDto,
        user_id: Uuid,
    ) -> Result<PollResponseDto, String> {
        // Fetch existing poll
        let mut poll = self
            .poll_repository
            .find_by_id(poll_id)
            .await?
            .ok_or_else(|| "Poll not found".to_string())?;

        // Verify user is the creator
        if poll.created_by != user_id {
            return Err("Only the poll creator can update the poll".to_string());
        }

        // Only allow updates to draft polls
        if poll.status != PollStatus::Draft {
            return Err("Cannot update poll that is no longer in draft status".to_string());
        }

        // Update fields
        if let Some(title) = dto.title {
            if title.trim().is_empty() {
                return Err("Poll title cannot be empty".to_string());
            }
            poll.title = title;
        }

        if let Some(description) = dto.description {
            poll.description = Some(description);
        }

        if let Some(ends_at_str) = dto.ends_at {
            let ends_at = DateTime::parse_from_rfc3339(&ends_at_str)
                .map_err(|_| "Invalid ends_at date format".to_string())?
                .with_timezone(&Utc);

            if ends_at <= Utc::now() {
                return Err("Poll end date must be in the future".to_string());
            }
            poll.ends_at = ends_at;
        }

        poll.updated_at = Utc::now();

        // Save updated poll
        let updated_poll = self.poll_repository.update(&poll).await?;

        Ok(PollResponseDto::from(updated_poll))
    }

    /// Get poll by ID
    pub async fn get_poll(&self, poll_id: Uuid) -> Result<PollResponseDto, String> {
        let poll = self
            .poll_repository
            .find_by_id(poll_id)
            .await?
            .ok_or_else(|| "Poll not found".to_string())?;

        Ok(PollResponseDto::from(poll))
    }

    /// List polls with pagination and filters
    pub async fn list_polls_paginated(
        &self,
        page_request: &PageRequest,
        filters: &PollFilters,
    ) -> Result<PollListResponseDto, String> {
        let (polls, total) = self
            .poll_repository
            .find_all_paginated(page_request, filters)
            .await?;

        let poll_dtos = polls.into_iter().map(PollResponseDto::from).collect();

        Ok(PollListResponseDto {
            polls: poll_dtos,
            total,
            page: page_request.page,
            page_size: page_request.per_page,
        })
    }

    /// Find active polls for a building
    pub async fn find_active_polls(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<PollResponseDto>, String> {
        let polls = self.poll_repository.find_active(building_id).await?;
        Ok(polls.into_iter().map(PollResponseDto::from).collect())
    }

    /// Publish a draft poll (change status to Active)
    pub async fn publish_poll(
        &self,
        poll_id: Uuid,
        user_id: Uuid,
    ) -> Result<PollResponseDto, String> {
        // Fetch poll
        let mut poll = self
            .poll_repository
            .find_by_id(poll_id)
            .await?
            .ok_or_else(|| "Poll not found".to_string())?;

        // Verify user is the creator
        if poll.created_by != user_id {
            return Err("Only the poll creator can publish the poll".to_string());
        }

        // Publish poll (activate it)
        poll.publish()?;

        // Save
        let updated_poll = self.poll_repository.update(&poll).await?;

        Ok(PollResponseDto::from(updated_poll))
    }

    /// Close a poll manually
    pub async fn close_poll(
        &self,
        poll_id: Uuid,
        user_id: Uuid,
    ) -> Result<PollResponseDto, String> {
        // Fetch poll
        let mut poll = self
            .poll_repository
            .find_by_id(poll_id)
            .await?
            .ok_or_else(|| "Poll not found".to_string())?;

        // Verify user is the creator
        if poll.created_by != user_id {
            return Err("Only the poll creator can close the poll".to_string());
        }

        // Close poll
        poll.close()?;

        // Save
        let updated_poll = self.poll_repository.update(&poll).await?;

        Ok(PollResponseDto::from(updated_poll))
    }

    /// Cancel a poll
    pub async fn cancel_poll(
        &self,
        poll_id: Uuid,
        user_id: Uuid,
    ) -> Result<PollResponseDto, String> {
        // Fetch poll
        let mut poll = self
            .poll_repository
            .find_by_id(poll_id)
            .await?
            .ok_or_else(|| "Poll not found".to_string())?;

        // Verify user is the creator
        if poll.created_by != user_id {
            return Err("Only the poll creator can cancel the poll".to_string());
        }

        // Cancel poll
        poll.cancel()?;

        // Save
        let updated_poll = self.poll_repository.update(&poll).await?;

        Ok(PollResponseDto::from(updated_poll))
    }

    /// Delete a poll (only if in draft or cancelled status)
    pub async fn delete_poll(&self, poll_id: Uuid, user_id: Uuid) -> Result<bool, String> {
        // Fetch poll
        let poll = self
            .poll_repository
            .find_by_id(poll_id)
            .await?
            .ok_or_else(|| "Poll not found".to_string())?;

        // Verify user is the creator
        if poll.created_by != user_id {
            return Err("Only the poll creator can delete the poll".to_string());
        }

        // Only allow deletion of draft or cancelled polls
        if poll.status != PollStatus::Draft && poll.status != PollStatus::Cancelled {
            return Err("Can only delete polls in draft or cancelled status".to_string());
        }

        self.poll_repository.delete(poll_id).await
    }

    /// Cast a vote on a poll
    pub async fn cast_vote(
        &self,
        dto: CastVoteDto,
        owner_id: Option<Uuid>,
    ) -> Result<String, String> {
        // Parse poll ID
        let poll_id = Uuid::parse_str(&dto.poll_id)
            .map_err(|_| "Invalid poll ID format".to_string())?;

        // Fetch poll
        let mut poll = self
            .poll_repository
            .find_by_id(poll_id)
            .await?
            .ok_or_else(|| "Poll not found".to_string())?;

        // Verify poll is active
        if poll.status != PollStatus::Active {
            return Err("Poll is not active".to_string());
        }

        // Verify poll hasn't expired
        if Utc::now() > poll.ends_at {
            return Err("Poll has expired".to_string());
        }

        // Check if user already voted (if not anonymous)
        if let Some(oid) = owner_id {
            if !poll.is_anonymous {
                let existing_vote = self
                    .poll_vote_repository
                    .find_by_poll_and_owner(poll_id, oid)
                    .await?;
                if existing_vote.is_some() {
                    return Err("You have already voted on this poll".to_string());
                }
            }
        }

        // Validate vote based on poll type
        let vote = match poll.poll_type {
            PollType::YesNo | PollType::MultipleChoice => {
                let selected_ids = dto
                    .selected_option_ids
                    .ok_or_else(|| "Selected option IDs required for this poll type".to_string())?
                    .iter()
                    .map(|id| {
                        Uuid::parse_str(id).map_err(|_| "Invalid option ID format".to_string())
                    })
                    .collect::<Result<Vec<Uuid>, String>>()?;

                // Validate options exist in poll
                for opt_id in &selected_ids {
                    if !poll.options.iter().any(|o| &o.id == opt_id) {
                        return Err("Invalid option ID".to_string());
                    }
                }

                // Validate multiple votes setting
                if !poll.allow_multiple_votes && selected_ids.len() > 1 {
                    return Err("This poll does not allow multiple votes".to_string());
                }

                PollVote::new(poll_id, owner_id, poll.building_id, selected_ids, None, None)?
            }
            PollType::Rating => {
                let rating = dto
                    .rating_value
                    .ok_or_else(|| "Rating value required for rating poll".to_string())?;

                PollVote::new(poll_id, owner_id, poll.building_id, vec![], Some(rating), None)?
            }
            PollType::OpenEnded => {
                let text = dto
                    .open_text
                    .ok_or_else(|| "Open text required for open-ended poll".to_string())?;

                PollVote::new(poll_id, owner_id, poll.building_id, vec![], None, Some(text))?
            }
        };

        // Save vote
        self.poll_vote_repository.create(&vote).await?;

        // Update poll vote count and option counts
        poll.total_votes_cast += 1;

        // Update option vote counts for YesNo/MultipleChoice
        if matches!(poll.poll_type, PollType::YesNo | PollType::MultipleChoice) {
            for opt_id in &vote.selected_option_ids {
                if let Some(option) = poll.options.iter_mut().find(|o| &o.id == opt_id) {
                    option.vote_count += 1;
                }
            }
        }

        // Save updated poll
        self.poll_repository.update(&poll).await?;

        Ok("Vote cast successfully".to_string())
    }

    /// Get poll results
    pub async fn get_poll_results(&self, poll_id: Uuid) -> Result<PollResultsDto, String> {
        // Fetch poll
        let poll = self
            .poll_repository
            .find_by_id(poll_id)
            .await?
            .ok_or_else(|| "Poll not found".to_string())?;

        // Calculate winning option (for YesNo/MultipleChoice)
        let winning_option = if matches!(poll.poll_type, PollType::YesNo | PollType::MultipleChoice)
        {
            poll.options
                .iter()
                .max_by_key(|opt| opt.vote_count)
                .map(|opt| {
                    let vote_percentage = if poll.total_votes_cast > 0 {
                        (opt.vote_count as f64 / poll.total_votes_cast as f64) * 100.0
                    } else {
                        0.0
                    };
                    PollOptionDto {
                        id: opt.id.to_string(),
                        option_text: opt.option_text.clone(),
                        attachment_url: opt.attachment_url.clone(),
                        vote_count: opt.vote_count,
                        vote_percentage,
                        display_order: opt.display_order,
                    }
                })
        } else {
            None
        };

        Ok(PollResultsDto {
            poll_id: poll.id.to_string(),
            total_votes_cast: poll.total_votes_cast,
            total_eligible_voters: poll.total_eligible_voters,
            participation_rate: poll.participation_rate(),
            winning_option,
            options: poll
                .options
                .iter()
                .map(|opt| {
                    let vote_percentage = if poll.total_votes_cast > 0 {
                        (opt.vote_count as f64 / poll.total_votes_cast as f64) * 100.0
                    } else {
                        0.0
                    };
                    PollOptionDto {
                        id: opt.id.to_string(),
                        option_text: opt.option_text.clone(),
                        attachment_url: opt.attachment_url.clone(),
                        vote_count: opt.vote_count,
                        vote_percentage,
                        display_order: opt.display_order,
                    }
                })
                .collect(),
        })
    }

    /// Get poll statistics for a building
    pub async fn get_building_statistics(
        &self,
        building_id: Uuid,
    ) -> Result<crate::application::ports::PollStatistics, String> {
        self.poll_repository
            .get_building_statistics(building_id)
            .await
    }

    /// Find and auto-close expired polls (for background job)
    pub async fn auto_close_expired_polls(&self) -> Result<usize, String> {
        let expired_polls = self.poll_repository.find_expired_active().await?;
        let mut closed_count = 0;

        for mut poll in expired_polls {
            if poll.close().is_ok() {
                self.poll_repository.update(&poll).await?;
                closed_count += 1;
            }
        }

        Ok(closed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::CreatePollOptionDto;
    use crate::application::ports::PollStatistics;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock repositories
    struct MockPollRepository {
        polls: Mutex<HashMap<Uuid, Poll>>,
    }

    impl MockPollRepository {
        fn new() -> Self {
            Self {
                polls: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl PollRepository for MockPollRepository {
        async fn create(&self, poll: &Poll) -> Result<Poll, String> {
            let mut polls = self.polls.lock().unwrap();
            polls.insert(poll.id, poll.clone());
            Ok(poll.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Poll>, String> {
            let polls = self.polls.lock().unwrap();
            Ok(polls.get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Poll>, String> {
            let polls = self.polls.lock().unwrap();
            Ok(polls
                .values()
                .filter(|p| p.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_created_by(&self, created_by: Uuid) -> Result<Vec<Poll>, String> {
            let polls = self.polls.lock().unwrap();
            Ok(polls
                .values()
                .filter(|p| p.created_by == created_by)
                .cloned()
                .collect())
        }

        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _filters: &PollFilters,
        ) -> Result<(Vec<Poll>, i64), String> {
            let polls = self.polls.lock().unwrap();
            let all: Vec<Poll> = polls.values().cloned().collect();
            let total = all.len() as i64;
            Ok((all, total))
        }

        async fn find_active(&self, building_id: Uuid) -> Result<Vec<Poll>, String> {
            let polls = self.polls.lock().unwrap();
            Ok(polls
                .values()
                .filter(|p| p.building_id == building_id && p.status == PollStatus::Active)
                .cloned()
                .collect())
        }

        async fn find_by_status(
            &self,
            building_id: Uuid,
            status: &str,
        ) -> Result<Vec<Poll>, String> {
            let polls = self.polls.lock().unwrap();
            let poll_status = match status {
                "draft" => PollStatus::Draft,
                "active" => PollStatus::Active,
                "closed" => PollStatus::Closed,
                "cancelled" => PollStatus::Cancelled,
                _ => return Err("Invalid status".to_string()),
            };
            Ok(polls
                .values()
                .filter(|p| p.building_id == building_id && p.status == poll_status)
                .cloned()
                .collect())
        }

        async fn find_expired_active(&self) -> Result<Vec<Poll>, String> {
            let polls = self.polls.lock().unwrap();
            Ok(polls
                .values()
                .filter(|p| p.status == PollStatus::Active && Utc::now() > p.ends_at)
                .cloned()
                .collect())
        }

        async fn update(&self, poll: &Poll) -> Result<Poll, String> {
            let mut polls = self.polls.lock().unwrap();
            polls.insert(poll.id, poll.clone());
            Ok(poll.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut polls = self.polls.lock().unwrap();
            Ok(polls.remove(&id).is_some())
        }

        async fn get_building_statistics(
            &self,
            building_id: Uuid,
        ) -> Result<PollStatistics, String> {
            let polls = self.polls.lock().unwrap();
            let building_polls: Vec<&Poll> =
                polls.values().filter(|p| p.building_id == building_id).collect();

            let total = building_polls.len() as i64;
            let active = building_polls
                .iter()
                .filter(|p| p.status == PollStatus::Active)
                .count() as i64;
            let closed = building_polls
                .iter()
                .filter(|p| p.status == PollStatus::Closed)
                .count() as i64;

            let avg_participation = if total > 0 {
                building_polls
                    .iter()
                    .map(|p| p.participation_rate())
                    .sum::<f64>()
                    / total as f64
            } else {
                0.0
            };

            Ok(PollStatistics {
                total_polls: total,
                active_polls: active,
                closed_polls: closed,
                average_participation_rate: avg_participation,
            })
        }
    }

    struct MockPollVoteRepository {
        votes: Mutex<HashMap<Uuid, PollVote>>,
    }

    impl MockPollVoteRepository {
        fn new() -> Self {
            Self {
                votes: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl PollVoteRepository for MockPollVoteRepository {
        async fn create(&self, vote: &PollVote) -> Result<PollVote, String> {
            let mut votes = self.votes.lock().unwrap();
            votes.insert(vote.id, vote.clone());
            Ok(vote.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<PollVote>, String> {
            let votes = self.votes.lock().unwrap();
            Ok(votes.get(&id).cloned())
        }

        async fn find_by_poll(&self, poll_id: Uuid) -> Result<Vec<PollVote>, String> {
            let votes = self.votes.lock().unwrap();
            Ok(votes
                .values()
                .filter(|v| v.poll_id == poll_id)
                .cloned()
                .collect())
        }

        async fn find_by_poll_and_owner(
            &self,
            poll_id: Uuid,
            owner_id: Uuid,
        ) -> Result<Option<PollVote>, String> {
            let votes = self.votes.lock().unwrap();
            Ok(votes
                .values()
                .find(|v| v.poll_id == poll_id && v.owner_id == Some(owner_id))
                .cloned())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<PollVote>, String> {
            let votes = self.votes.lock().unwrap();
            Ok(votes
                .values()
                .filter(|v| v.owner_id == Some(owner_id))
                .cloned()
                .collect())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut votes = self.votes.lock().unwrap();
            Ok(votes.remove(&id).is_some())
        }
    }

    struct MockOwnerRepository;

    #[async_trait]
    impl OwnerRepository for MockOwnerRepository {
        async fn find_by_building(&self, _building_id: Uuid) -> Result<Vec<crate::domain::entities::Owner>, String> {
            // Return 10 mock owners
            Ok((0..10)
                .map(|i| crate::domain::entities::Owner {
                    id: Uuid::new_v4(),
                    organization_id: Uuid::new_v4(),
                    first_name: format!("Owner{}", i),
                    last_name: "Test".to_string(),
                    email: format!("owner{}@test.com", i),
                    phone: None,
                    national_id: None,
                    address: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    is_active: true,
                    processing_restricted: false,
                    processing_restricted_at: None,
                    marketing_opt_out: false,
                    marketing_opt_out_at: None,
                })
                .collect())
        }

        async fn create(&self, _owner: &crate::domain::entities::Owner) -> Result<crate::domain::entities::Owner, String> {
            unimplemented!()
        }

        async fn find_by_id(&self, _id: Uuid) -> Result<Option<crate::domain::entities::Owner>, String> {
            unimplemented!()
        }

        async fn find_all(&self) -> Result<Vec<crate::domain::entities::Owner>, String> {
            unimplemented!()
        }

        async fn find_by_email(&self, _email: &str) -> Result<Option<crate::domain::entities::Owner>, String> {
            unimplemented!()
        }

        async fn update(&self, _owner: &crate::domain::entities::Owner) -> Result<crate::domain::entities::Owner, String> {
            unimplemented!()
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, String> {
            unimplemented!()
        }

        async fn find_by_organization(&self, _organization_id: Uuid) -> Result<Vec<crate::domain::entities::Owner>, String> {
            unimplemented!()
        }
    }

    fn setup_use_cases() -> PollUseCases {
        PollUseCases::new(
            Arc::new(MockPollRepository::new()),
            Arc::new(MockPollVoteRepository::new()),
            Arc::new(MockOwnerRepository),
        )
    }

    #[tokio::test]
    async fn test_create_poll_success() {
        let use_cases = setup_use_cases();
        let building_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();

        let dto = CreatePollDto {
            building_id: building_id.to_string(),
            title: "Test Poll".to_string(),
            description: Some("Test description".to_string()),
            poll_type: "yes_no".to_string(),
            options: vec![
                CreatePollOptionDto {
                    option_text: "Yes".to_string(),
                    attachment_url: None,
                    display_order: 0,
                },
                CreatePollOptionDto {
                    option_text: "No".to_string(),
                    attachment_url: None,
                    display_order: 1,
                },
            ],
            is_anonymous: Some(false),
            allow_multiple_votes: Some(false),
            require_all_owners: Some(false),
            ends_at: (Utc::now() + chrono::Duration::days(7))
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        };

        let result = use_cases.create_poll(dto, created_by).await;
        assert!(result.is_ok());

        let poll_response = result.unwrap();
        assert_eq!(poll_response.title, "Test Poll");
        assert_eq!(poll_response.total_eligible_voters, 10); // Mock returns 10 owners
    }

    #[tokio::test]
    async fn test_create_poll_invalid_end_date() {
        let use_cases = setup_use_cases();
        let building_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();

        let dto = CreatePollDto {
            building_id: building_id.to_string(),
            title: "Test Poll".to_string(),
            description: None,
            poll_type: "yes_no".to_string(),
            options: vec![],
            is_anonymous: None,
            allow_multiple_votes: None,
            require_all_owners: None,
            ends_at: (Utc::now() - chrono::Duration::days(1))
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        };

        let result = use_cases.create_poll(dto, created_by).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be in the future"));
    }
}
