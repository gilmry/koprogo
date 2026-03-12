use crate::application::ports::{ResolutionRepository, VoteRepository};
use crate::domain::entities::{
    MajorityType, Resolution, ResolutionStatus, ResolutionType, Vote, VoteChoice,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct ResolutionUseCases {
    resolution_repository: Arc<dyn ResolutionRepository>,
    vote_repository: Arc<dyn VoteRepository>,
}

impl ResolutionUseCases {
    pub fn new(
        resolution_repository: Arc<dyn ResolutionRepository>,
        vote_repository: Arc<dyn VoteRepository>,
    ) -> Self {
        Self {
            resolution_repository,
            vote_repository,
        }
    }

    /// Create a new resolution for a meeting
    pub async fn create_resolution(
        &self,
        meeting_id: Uuid,
        title: String,
        description: String,
        resolution_type: ResolutionType,
        majority_required: MajorityType,
    ) -> Result<Resolution, String> {
        let resolution = Resolution::new(
            meeting_id,
            title,
            description,
            resolution_type,
            majority_required,
        )?;

        self.resolution_repository.create(&resolution).await
    }

    /// Get a resolution by ID
    pub async fn get_resolution(&self, id: Uuid) -> Result<Option<Resolution>, String> {
        self.resolution_repository.find_by_id(id).await
    }

    /// Get all resolutions for a meeting
    pub async fn get_meeting_resolutions(
        &self,
        meeting_id: Uuid,
    ) -> Result<Vec<Resolution>, String> {
        self.resolution_repository
            .find_by_meeting_id(meeting_id)
            .await
    }

    /// Get resolutions by status
    pub async fn get_resolutions_by_status(
        &self,
        status: ResolutionStatus,
    ) -> Result<Vec<Resolution>, String> {
        self.resolution_repository.find_by_status(status).await
    }

    /// Update a resolution (only allowed if status is Pending)
    pub async fn update_resolution(&self, resolution: &Resolution) -> Result<Resolution, String> {
        if resolution.status != ResolutionStatus::Pending {
            return Err("Cannot update a resolution that is not pending".to_string());
        }

        self.resolution_repository.update(resolution).await
    }

    /// Delete a resolution (only allowed if no votes have been cast)
    pub async fn delete_resolution(&self, id: Uuid) -> Result<bool, String> {
        // Check if any votes exist
        let votes = self.vote_repository.find_by_resolution_id(id).await?;
        if !votes.is_empty() {
            return Err("Cannot delete a resolution with existing votes".to_string());
        }

        self.resolution_repository.delete(id).await
    }

    /// Cast a vote on a resolution
    pub async fn cast_vote(
        &self,
        resolution_id: Uuid,
        owner_id: Uuid,
        unit_id: Uuid,
        vote_choice: VoteChoice,
        voting_power: f64,
        proxy_owner_id: Option<Uuid>,
    ) -> Result<Vote, String> {
        // Check if resolution exists and is pending
        let resolution = self
            .resolution_repository
            .find_by_id(resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found".to_string())?;

        if resolution.status != ResolutionStatus::Pending {
            return Err("Cannot vote on a resolution that is not pending".to_string());
        }

        // Check if unit has already voted
        if self
            .vote_repository
            .has_voted(resolution_id, unit_id)
            .await?
        {
            return Err("This unit has already voted on this resolution".to_string());
        }

        // Art. 3.87 §7 CC — un mandataire ne peut détenir plus de 3 procurations
        // Exception : si le total des voix de ses procurations < 10% du total général
        if let Some(proxy_id) = proxy_owner_id {
            self.validate_proxy_limit(resolution_id, proxy_id, voting_power)
                .await?;
        }

        // Create and save the vote
        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            vote_choice.clone(),
            voting_power,
            proxy_owner_id,
        )?;

        let created_vote = self.vote_repository.create(&vote).await?;

        // Update vote counts on the resolution
        self.update_resolution_vote_counts(resolution_id).await?;

        Ok(created_vote)
    }

    /// Change a vote (if allowed by business rules)
    pub async fn change_vote(&self, vote_id: Uuid, new_choice: VoteChoice) -> Result<Vote, String> {
        let mut vote = self
            .vote_repository
            .find_by_id(vote_id)
            .await?
            .ok_or_else(|| "Vote not found".to_string())?;

        // Check if resolution is still pending
        let resolution = self
            .resolution_repository
            .find_by_id(vote.resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found".to_string())?;

        if resolution.status != ResolutionStatus::Pending {
            return Err("Cannot change vote on a closed resolution".to_string());
        }

        // Update the vote
        vote.change_vote(new_choice)?;
        let updated_vote = self.vote_repository.update(&vote).await?;

        // Recalculate vote counts
        self.update_resolution_vote_counts(vote.resolution_id)
            .await?;

        Ok(updated_vote)
    }

    /// Get all votes for a resolution
    pub async fn get_resolution_votes(&self, resolution_id: Uuid) -> Result<Vec<Vote>, String> {
        self.vote_repository
            .find_by_resolution_id(resolution_id)
            .await
    }

    /// Get all votes by an owner
    pub async fn get_owner_votes(&self, owner_id: Uuid) -> Result<Vec<Vote>, String> {
        self.vote_repository.find_by_owner_id(owner_id).await
    }

    /// Close voting on a resolution and calculate final result
    pub async fn close_voting(
        &self,
        resolution_id: Uuid,
        total_voting_power: f64,
    ) -> Result<Resolution, String> {
        let mut resolution = self
            .resolution_repository
            .find_by_id(resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found".to_string())?;

        if resolution.status != ResolutionStatus::Pending {
            return Err("Resolution voting is already closed".to_string());
        }

        // Calculate final result
        resolution.close_voting(total_voting_power)?;

        // Update resolution with final status
        self.resolution_repository
            .close_voting(resolution_id, resolution.status.clone())
            .await?;

        // Fetch updated resolution
        self.resolution_repository
            .find_by_id(resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found after closing".to_string())
    }

    /// Get vote summary for a meeting (all resolutions with their results)
    pub async fn get_meeting_vote_summary(
        &self,
        meeting_id: Uuid,
    ) -> Result<Vec<Resolution>, String> {
        self.resolution_repository
            .get_meeting_vote_summary(meeting_id)
            .await
    }

    /// Helper: Update vote counts for a resolution based on actual votes
    async fn update_resolution_vote_counts(&self, resolution_id: Uuid) -> Result<(), String> {
        // Get vote counts
        let (pour_count, contre_count, abstention_count) = self
            .vote_repository
            .count_by_resolution_and_choice(resolution_id)
            .await?;

        // Get voting power totals
        let (pour_power, contre_power, abstention_power) = self
            .vote_repository
            .sum_voting_power_by_resolution(resolution_id)
            .await?;

        // Update resolution
        self.resolution_repository
            .update_vote_counts(
                resolution_id,
                pour_count,
                contre_count,
                abstention_count,
                pour_power,
                contre_power,
                abstention_power,
            )
            .await
    }

    /// Check if a unit has voted on a resolution
    pub async fn has_unit_voted(&self, resolution_id: Uuid, unit_id: Uuid) -> Result<bool, String> {
        self.vote_repository.has_voted(resolution_id, unit_id).await
    }

    /// Valide la limite de procurations par mandataire (Art. 3.87 §7 CC).
    ///
    /// Règle: un mandataire ne peut détenir plus de 3 procurations.
    /// Exception: si le total des voix représentées < 10% du total général,
    /// la limite de 3 ne s'applique pas.
    ///
    /// Le `new_voting_power` est le pouvoir de vote de la nouvelle procuration
    /// envisagée (utilisé pour le calcul de l'exception 10%).
    async fn validate_proxy_limit(
        &self,
        resolution_id: Uuid,
        proxy_owner_id: Uuid,
        new_voting_power: f64,
    ) -> Result<(), String> {
        let (existing_count, existing_power) = self
            .vote_repository
            .count_proxy_votes_for_mandataire(resolution_id, proxy_owner_id)
            .await?;

        // Total du pouvoir de vote représenté après ajout de la nouvelle procuration
        let total_proxy_power = existing_power + new_voting_power;

        // Exception 10% : si le total des procurations < 10% du total AG, pas de limite
        // On récupère le total de la résolution
        let resolution = self
            .resolution_repository
            .find_by_id(resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found".to_string())?;

        // total_voting_power_* contient les votes déjà exprimés
        // Pour l'exception, on compare la puissance des procurations au total millièmes
        // En pratique, le syndic passe le total_quotas du meeting — on utilise une heuristique
        let total_all_votes = resolution.total_voting_power_pour
            + resolution.total_voting_power_contre
            + resolution.total_voting_power_abstention
            + new_voting_power; // inclure ce qu'on est en train d'ajouter

        // Exception: si total procurations < 10% du total général → pas de limite
        if total_all_votes > 0.0 && (total_proxy_power / total_all_votes) < 0.10 {
            return Ok(()); // Exception 10% s'applique
        }

        // Règle générale: max 3 procurations
        if existing_count >= 3 {
            return Err(format!(
                "Le mandataire détient déjà {} procurations. Maximum autorisé : 3 (Art. 3.87 §7 CC). \
                 Exception 10% non applicable (procurations représentent >{:.1}% des votes).",
                existing_count,
                if total_all_votes > 0.0 {
                    (total_proxy_power / total_all_votes) * 100.0
                } else {
                    0.0
                }
            ));
        }

        Ok(())
    }

    /// Get vote statistics for a resolution
    pub async fn get_vote_statistics(&self, resolution_id: Uuid) -> Result<VoteStatistics, String> {
        let resolution = self
            .resolution_repository
            .find_by_id(resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found".to_string())?;

        Ok(VoteStatistics {
            total_votes: resolution.total_votes(),
            vote_count_pour: resolution.vote_count_pour,
            vote_count_contre: resolution.vote_count_contre,
            vote_count_abstention: resolution.vote_count_abstention,
            total_voting_power_pour: resolution.total_voting_power_pour,
            total_voting_power_contre: resolution.total_voting_power_contre,
            total_voting_power_abstention: resolution.total_voting_power_abstention,
            pour_percentage: resolution.pour_percentage(),
            contre_percentage: resolution.contre_percentage(),
            abstention_percentage: resolution.abstention_percentage(),
            status: resolution.status,
        })
    }
}

/// Vote statistics for a resolution
#[derive(Debug, Clone)]
pub struct VoteStatistics {
    pub total_votes: i32,
    pub vote_count_pour: i32,
    pub vote_count_contre: i32,
    pub vote_count_abstention: i32,
    pub total_voting_power_pour: f64,
    pub total_voting_power_contre: f64,
    pub total_voting_power_abstention: f64,
    pub pour_percentage: f64,
    pub contre_percentage: f64,
    pub abstention_percentage: f64,
    pub status: ResolutionStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::ResolutionRepository;
    use crate::application::ports::VoteRepository;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock repositories for testing
    struct MockResolutionRepository {
        resolutions: Mutex<HashMap<Uuid, Resolution>>,
    }

    impl MockResolutionRepository {
        fn new() -> Self {
            Self {
                resolutions: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ResolutionRepository for MockResolutionRepository {
        async fn create(&self, resolution: &Resolution) -> Result<Resolution, String> {
            self.resolutions
                .lock()
                .unwrap()
                .insert(resolution.id, resolution.clone());
            Ok(resolution.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Resolution>, String> {
            Ok(self.resolutions.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_meeting_id(&self, meeting_id: Uuid) -> Result<Vec<Resolution>, String> {
            Ok(self
                .resolutions
                .lock()
                .unwrap()
                .values()
                .filter(|r| r.meeting_id == meeting_id)
                .cloned()
                .collect())
        }

        async fn find_by_status(
            &self,
            status: ResolutionStatus,
        ) -> Result<Vec<Resolution>, String> {
            Ok(self
                .resolutions
                .lock()
                .unwrap()
                .values()
                .filter(|r| r.status == status)
                .cloned()
                .collect())
        }

        async fn update(&self, resolution: &Resolution) -> Result<Resolution, String> {
            self.resolutions
                .lock()
                .unwrap()
                .insert(resolution.id, resolution.clone());
            Ok(resolution.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.resolutions.lock().unwrap().remove(&id).is_some())
        }

        async fn update_vote_counts(
            &self,
            resolution_id: Uuid,
            vote_count_pour: i32,
            vote_count_contre: i32,
            vote_count_abstention: i32,
            total_voting_power_pour: f64,
            total_voting_power_contre: f64,
            total_voting_power_abstention: f64,
        ) -> Result<(), String> {
            if let Some(resolution) = self.resolutions.lock().unwrap().get_mut(&resolution_id) {
                resolution.vote_count_pour = vote_count_pour;
                resolution.vote_count_contre = vote_count_contre;
                resolution.vote_count_abstention = vote_count_abstention;
                resolution.total_voting_power_pour = total_voting_power_pour;
                resolution.total_voting_power_contre = total_voting_power_contre;
                resolution.total_voting_power_abstention = total_voting_power_abstention;
            }
            Ok(())
        }

        async fn close_voting(
            &self,
            resolution_id: Uuid,
            final_status: ResolutionStatus,
        ) -> Result<(), String> {
            if let Some(resolution) = self.resolutions.lock().unwrap().get_mut(&resolution_id) {
                resolution.status = final_status;
                resolution.voted_at = Some(chrono::Utc::now());
            }
            Ok(())
        }

        async fn get_meeting_vote_summary(
            &self,
            meeting_id: Uuid,
        ) -> Result<Vec<Resolution>, String> {
            self.find_by_meeting_id(meeting_id).await
        }
    }

    struct MockVoteRepository {
        votes: Mutex<HashMap<Uuid, Vote>>,
    }

    impl MockVoteRepository {
        fn new() -> Self {
            Self {
                votes: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl VoteRepository for MockVoteRepository {
        async fn create(&self, vote: &Vote) -> Result<Vote, String> {
            self.votes.lock().unwrap().insert(vote.id, vote.clone());
            Ok(vote.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Vote>, String> {
            Ok(self.votes.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_resolution_id(&self, resolution_id: Uuid) -> Result<Vec<Vote>, String> {
            Ok(self
                .votes
                .lock()
                .unwrap()
                .values()
                .filter(|v| v.resolution_id == resolution_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner_id(&self, owner_id: Uuid) -> Result<Vec<Vote>, String> {
            Ok(self
                .votes
                .lock()
                .unwrap()
                .values()
                .filter(|v| v.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn find_by_resolution_and_unit(
            &self,
            resolution_id: Uuid,
            unit_id: Uuid,
        ) -> Result<Option<Vote>, String> {
            Ok(self
                .votes
                .lock()
                .unwrap()
                .values()
                .find(|v| v.resolution_id == resolution_id && v.unit_id == unit_id)
                .cloned())
        }

        async fn has_voted(&self, resolution_id: Uuid, unit_id: Uuid) -> Result<bool, String> {
            Ok(self
                .find_by_resolution_and_unit(resolution_id, unit_id)
                .await?
                .is_some())
        }

        async fn update(&self, vote: &Vote) -> Result<Vote, String> {
            self.votes.lock().unwrap().insert(vote.id, vote.clone());
            Ok(vote.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.votes.lock().unwrap().remove(&id).is_some())
        }

        async fn count_by_resolution_and_choice(
            &self,
            resolution_id: Uuid,
        ) -> Result<(i32, i32, i32), String> {
            let votes = self.find_by_resolution_id(resolution_id).await?;
            let pour = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Pour)
                .count() as i32;
            let contre = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Contre)
                .count() as i32;
            let abstention = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Abstention)
                .count() as i32;
            Ok((pour, contre, abstention))
        }

        async fn sum_voting_power_by_resolution(
            &self,
            resolution_id: Uuid,
        ) -> Result<(f64, f64, f64), String> {
            let votes = self.find_by_resolution_id(resolution_id).await?;
            let pour: f64 = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Pour)
                .map(|v| v.voting_power)
                .sum();
            let contre: f64 = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Contre)
                .map(|v| v.voting_power)
                .sum();
            let abstention: f64 = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Abstention)
                .map(|v| v.voting_power)
                .sum();
            Ok((pour, contre, abstention))
        }

        async fn count_proxy_votes_for_mandataire(
            &self,
            resolution_id: Uuid,
            proxy_owner_id: Uuid,
        ) -> Result<(i64, f64), String> {
            let votes = self.find_by_resolution_id(resolution_id).await?;
            let proxy_votes: Vec<_> = votes
                .iter()
                .filter(|v| v.proxy_owner_id == Some(proxy_owner_id))
                .collect();
            let count = proxy_votes.len() as i64;
            let power: f64 = proxy_votes.iter().map(|v| v.voting_power).sum();
            Ok((count, power))
        }
    }

    #[tokio::test]
    async fn test_create_resolution() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let use_cases = ResolutionUseCases::new(resolution_repo.clone(), vote_repo);

        let meeting_id = Uuid::new_v4();
        let result = use_cases
            .create_resolution(
                meeting_id,
                "Test Resolution".to_string(),
                "Description".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Simple,
            )
            .await;

        assert!(result.is_ok());
        let resolution = result.unwrap();
        assert_eq!(resolution.title, "Test Resolution");
        assert_eq!(resolution.status, ResolutionStatus::Pending);
    }

    #[tokio::test]
    async fn test_cast_vote_updates_counts() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let use_cases = ResolutionUseCases::new(resolution_repo.clone(), vote_repo.clone());

        // Create a resolution
        let meeting_id = Uuid::new_v4();
        let resolution = use_cases
            .create_resolution(
                meeting_id,
                "Test Resolution".to_string(),
                "Description".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Simple,
            )
            .await
            .unwrap();

        // Cast a vote
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let result = use_cases
            .cast_vote(
                resolution.id,
                owner_id,
                unit_id,
                VoteChoice::Pour,
                100.0,
                None,
            )
            .await;

        assert!(result.is_ok());

        // Check that vote counts were updated
        let updated_resolution = use_cases
            .get_resolution(resolution.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_resolution.vote_count_pour, 1);
        assert_eq!(updated_resolution.total_voting_power_pour, 100.0);
    }

    #[tokio::test]
    async fn test_cannot_vote_twice() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let use_cases = ResolutionUseCases::new(resolution_repo.clone(), vote_repo);

        // Create resolution
        let meeting_id = Uuid::new_v4();
        let resolution = use_cases
            .create_resolution(
                meeting_id,
                "Test".to_string(),
                "Desc".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Simple,
            )
            .await
            .unwrap();

        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        // First vote succeeds
        let result1 = use_cases
            .cast_vote(
                resolution.id,
                owner_id,
                unit_id,
                VoteChoice::Pour,
                100.0,
                None,
            )
            .await;
        assert!(result1.is_ok());

        // Second vote from same unit fails
        let result2 = use_cases
            .cast_vote(
                resolution.id,
                owner_id,
                unit_id,
                VoteChoice::Contre,
                100.0,
                None,
            )
            .await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("already voted"));
    }

    /// Art. 3.87 §7 CC — un mandataire ne peut pas détenir plus de 3 procurations
    #[tokio::test]
    async fn test_proxy_limit_max_3_enforced() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let use_cases = ResolutionUseCases::new(resolution_repo.clone(), vote_repo.clone());

        let meeting_id = Uuid::new_v4();
        let resolution = use_cases
            .create_resolution(
                meeting_id,
                "Test procurations".to_string(),
                "Description".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Simple,
            )
            .await
            .unwrap();

        let mandataire_id = Uuid::new_v4();

        // Cast 3 proxy votes for the same mandataire (should all succeed)
        for i in 0..3 {
            let owner_id = Uuid::new_v4();
            let unit_id = Uuid::new_v4();
            let result = use_cases
                .cast_vote(
                    resolution.id,
                    owner_id,
                    unit_id,
                    VoteChoice::Pour,
                    100.0, // 100 millièmes chacun = 300 total
                    Some(mandataire_id),
                )
                .await;
            assert!(
                result.is_ok(),
                "Proxy vote {} should succeed, got: {:?}",
                i + 1,
                result.err()
            );
        }

        // 4th proxy vote for the same mandataire should fail (>3 proxies AND >10% of votes)
        let owner_id_4 = Uuid::new_v4();
        let unit_id_4 = Uuid::new_v4();
        let result4 = use_cases
            .cast_vote(
                resolution.id,
                owner_id_4,
                unit_id_4,
                VoteChoice::Pour,
                100.0,
                Some(mandataire_id),
            )
            .await;
        assert!(result4.is_err(), "4th proxy vote should be rejected");
        assert!(
            result4.unwrap_err().contains("3"),
            "Error should mention the 3-proxy limit"
        );
    }

    /// Art. 3.87 §7 CC — exception 10% : si total procurations < 10% → pas de limite
    #[tokio::test]
    async fn test_proxy_limit_10_percent_exception_allows_more() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let use_cases = ResolutionUseCases::new(resolution_repo.clone(), vote_repo.clone());

        let meeting_id = Uuid::new_v4();
        let resolution = use_cases
            .create_resolution(
                meeting_id,
                "Test exception 10%".to_string(),
                "Description".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Simple,
            )
            .await
            .unwrap();

        let mandataire_id = Uuid::new_v4();

        // First, add many direct votes to make the total large (900 millièmes direct)
        for _ in 0..9 {
            let _ = use_cases
                .cast_vote(
                    resolution.id,
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    VoteChoice::Pour,
                    100.0, // 100 millièmes each, 9 × 100 = 900 total direct
                    None,
                )
                .await;
        }

        // Now add 4 proxy votes of 5 millièmes each = 20 millièmes proxy
        // Total votes = 900 + 20 = 920 → 20/920 = 2.2% < 10% → exception applies
        for i in 0..4 {
            let result = use_cases
                .cast_vote(
                    resolution.id,
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    VoteChoice::Pour,
                    5.0, // 5 millièmes chacun
                    Some(mandataire_id),
                )
                .await;
            assert!(
                result.is_ok(),
                "Proxy vote {} (10% exception) should succeed, got: {:?}",
                i + 1,
                result.err()
            );
        }
    }
}
