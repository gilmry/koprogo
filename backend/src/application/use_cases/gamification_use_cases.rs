use crate::application::dto::{
    AchievementResponseDto, ChallengeProgressResponseDto, ChallengeResponseDto,
    CreateAchievementDto, CreateChallengeDto, LeaderboardEntryDto, LeaderboardResponseDto,
    UpdateAchievementDto, UpdateChallengeDto, UserAchievementResponseDto, UserGamificationStatsDto,
};
use crate::application::ports::{
    AchievementRepository, ChallengeProgressRepository, ChallengeRepository,
    UserAchievementRepository, UserRepository,
};
use crate::domain::entities::{
    Achievement, AchievementCategory, Challenge, ChallengeProgress, ChallengeStatus,
    UserAchievement,
};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Achievement Use Cases
// ============================================================================

/// Use cases for achievement operations
///
/// Orchestrates business logic for achievement CRUD, awarding achievements to users,
/// and calculating user achievement statistics.
pub struct AchievementUseCases {
    achievement_repo: Arc<dyn AchievementRepository>,
    user_achievement_repo: Arc<dyn UserAchievementRepository>,
    #[allow(dead_code)]
    user_repo: Arc<dyn UserRepository>,
}

impl AchievementUseCases {
    pub fn new(
        achievement_repo: Arc<dyn AchievementRepository>,
        user_achievement_repo: Arc<dyn UserAchievementRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            achievement_repo,
            user_achievement_repo,
            user_repo,
        }
    }

    /// Create a new achievement
    ///
    /// # Authorization
    /// - Only organization admins can create achievements
    pub async fn create_achievement(
        &self,
        dto: CreateAchievementDto,
    ) -> Result<AchievementResponseDto, String> {
        let achievement = Achievement::new(
            dto.organization_id,
            dto.category,
            dto.tier,
            dto.name,
            dto.description,
            dto.icon,
            dto.points_value,
            dto.requirements,
            dto.is_secret,
            dto.is_repeatable,
            dto.display_order,
        )?;

        let created = self.achievement_repo.create(&achievement).await?;
        Ok(AchievementResponseDto::from(created))
    }

    /// Get achievement by ID
    pub async fn get_achievement(
        &self,
        achievement_id: Uuid,
    ) -> Result<AchievementResponseDto, String> {
        let achievement = self
            .achievement_repo
            .find_by_id(achievement_id)
            .await?
            .ok_or("Achievement not found".to_string())?;

        Ok(AchievementResponseDto::from(achievement))
    }

    /// List all achievements for an organization
    pub async fn list_achievements(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<AchievementResponseDto>, String> {
        let achievements = self
            .achievement_repo
            .find_by_organization(organization_id)
            .await?;
        Ok(achievements
            .into_iter()
            .map(AchievementResponseDto::from)
            .collect())
    }

    /// List achievements by category
    pub async fn list_achievements_by_category(
        &self,
        organization_id: Uuid,
        category: AchievementCategory,
    ) -> Result<Vec<AchievementResponseDto>, String> {
        let achievements = self
            .achievement_repo
            .find_by_organization_and_category(organization_id, category)
            .await?;
        Ok(achievements
            .into_iter()
            .map(AchievementResponseDto::from)
            .collect())
    }

    /// List visible achievements for a user (non-secret or already earned)
    pub async fn list_visible_achievements(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AchievementResponseDto>, String> {
        let achievements = self
            .achievement_repo
            .find_visible_for_user(organization_id, user_id)
            .await?;
        Ok(achievements
            .into_iter()
            .map(AchievementResponseDto::from)
            .collect())
    }

    /// Update achievement (admin only)
    pub async fn update_achievement(
        &self,
        achievement_id: Uuid,
        dto: UpdateAchievementDto,
    ) -> Result<AchievementResponseDto, String> {
        let mut achievement = self
            .achievement_repo
            .find_by_id(achievement_id)
            .await?
            .ok_or("Achievement not found".to_string())?;

        // Apply updates
        if let Some(name) = dto.name {
            achievement.update_name(name)?;
        }
        if let Some(description) = dto.description {
            achievement.update_description(description)?;
        }
        if let Some(icon) = dto.icon {
            achievement.update_icon(icon)?;
        }
        if let Some(points_value) = dto.points_value {
            achievement.update_points_value(points_value)?;
        }
        if let Some(requirements) = dto.requirements {
            achievement.update_requirements(requirements)?;
        }
        if let Some(is_secret) = dto.is_secret {
            achievement.is_secret = is_secret;
        }
        if let Some(is_repeatable) = dto.is_repeatable {
            achievement.is_repeatable = is_repeatable;
        }
        if let Some(display_order) = dto.display_order {
            achievement.display_order = display_order;
        }

        let updated = self.achievement_repo.update(&achievement).await?;
        Ok(AchievementResponseDto::from(updated))
    }

    /// Delete achievement (admin only)
    pub async fn delete_achievement(&self, achievement_id: Uuid) -> Result<(), String> {
        self.achievement_repo.delete(achievement_id).await
    }

    /// Award achievement to user
    ///
    /// For repeatable achievements, increments times_earned counter.
    /// For non-repeatable, returns error if already earned.
    pub async fn award_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
        progress_data: Option<String>,
    ) -> Result<UserAchievementResponseDto, String> {
        // Fetch achievement
        let achievement = self
            .achievement_repo
            .find_by_id(achievement_id)
            .await?
            .ok_or("Achievement not found".to_string())?;

        // Check if already earned
        if let Some(mut existing) = self
            .user_achievement_repo
            .find_by_user_and_achievement(user_id, achievement_id)
            .await?
        {
            if !achievement.is_repeatable {
                return Err("Achievement already earned and not repeatable".to_string());
            }

            // Increment times_earned for repeatable achievements
            existing.repeat_earn()?;
            let updated = self.user_achievement_repo.update(&existing).await?;
            return Ok(UserAchievementResponseDto::from_entities(
                updated,
                achievement,
            ));
        }

        // Award new achievement
        let user_achievement = UserAchievement::new(user_id, achievement_id, progress_data);
        let created = self.user_achievement_repo.create(&user_achievement).await?;

        Ok(UserAchievementResponseDto::from_entities(
            created,
            achievement,
        ))
    }

    /// Get all achievements earned by a user (enriched with achievement data)
    pub async fn get_user_achievements(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserAchievementResponseDto>, String> {
        let user_achievements = self.user_achievement_repo.find_by_user(user_id).await?;

        // Enrich with achievement data
        let mut enriched = Vec::new();
        for ua in user_achievements {
            if let Some(achievement) = self.achievement_repo.find_by_id(ua.achievement_id).await? {
                enriched.push(UserAchievementResponseDto::from_entities(ua, achievement));
            }
        }

        Ok(enriched)
    }

    /// Get recent achievements for a user (last N)
    pub async fn get_recent_achievements(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<UserAchievementResponseDto>, String> {
        let user_achievements = self
            .user_achievement_repo
            .find_recent_by_user(user_id, limit)
            .await?;

        // Enrich with achievement data
        let mut enriched = Vec::new();
        for ua in user_achievements {
            if let Some(achievement) = self.achievement_repo.find_by_id(ua.achievement_id).await? {
                enriched.push(UserAchievementResponseDto::from_entities(ua, achievement));
            }
        }

        Ok(enriched)
    }
}

// ============================================================================
// Challenge Use Cases
// ============================================================================

/// Use cases for challenge operations
///
/// Orchestrates business logic for challenge CRUD, activation/completion,
/// and progress tracking.
pub struct ChallengeUseCases {
    challenge_repo: Arc<dyn ChallengeRepository>,
    progress_repo: Arc<dyn ChallengeProgressRepository>,
}

impl ChallengeUseCases {
    pub fn new(
        challenge_repo: Arc<dyn ChallengeRepository>,
        progress_repo: Arc<dyn ChallengeProgressRepository>,
    ) -> Self {
        Self {
            challenge_repo,
            progress_repo,
        }
    }

    /// Create a new challenge (Draft status)
    ///
    /// # Authorization
    /// - Only organization admins can create challenges
    pub async fn create_challenge(
        &self,
        dto: CreateChallengeDto,
    ) -> Result<ChallengeResponseDto, String> {
        let challenge = Challenge::new(
            dto.organization_id,
            dto.building_id,
            dto.challenge_type,
            dto.title,
            dto.description,
            dto.icon,
            dto.start_date,
            dto.end_date,
            dto.target_metric,
            dto.target_value,
            dto.reward_points,
        )?;

        let created = self.challenge_repo.create(&challenge).await?;
        Ok(ChallengeResponseDto::from(created))
    }

    /// Get challenge by ID
    pub async fn get_challenge(&self, challenge_id: Uuid) -> Result<ChallengeResponseDto, String> {
        let challenge = self
            .challenge_repo
            .find_by_id(challenge_id)
            .await?
            .ok_or("Challenge not found".to_string())?;

        Ok(ChallengeResponseDto::from(challenge))
    }

    /// List all challenges for an organization
    pub async fn list_challenges(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<ChallengeResponseDto>, String> {
        let challenges = self
            .challenge_repo
            .find_by_organization(organization_id)
            .await?;
        Ok(challenges
            .into_iter()
            .map(ChallengeResponseDto::from)
            .collect())
    }

    /// List challenges by status
    pub async fn list_challenges_by_status(
        &self,
        organization_id: Uuid,
        status: ChallengeStatus,
    ) -> Result<Vec<ChallengeResponseDto>, String> {
        let challenges = self
            .challenge_repo
            .find_by_organization_and_status(organization_id, status)
            .await?;
        Ok(challenges
            .into_iter()
            .map(ChallengeResponseDto::from)
            .collect())
    }

    /// List challenges for a building
    pub async fn list_building_challenges(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<ChallengeResponseDto>, String> {
        let challenges = self.challenge_repo.find_by_building(building_id).await?;
        Ok(challenges
            .into_iter()
            .map(ChallengeResponseDto::from)
            .collect())
    }

    /// List active challenges (Active status + date range)
    pub async fn list_active_challenges(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<ChallengeResponseDto>, String> {
        let challenges = self.challenge_repo.find_active(organization_id).await?;
        Ok(challenges
            .into_iter()
            .map(ChallengeResponseDto::from)
            .collect())
    }

    /// Update challenge (Draft only)
    ///
    /// # Authorization
    /// - Only organization admins can update challenges
    /// - Can only update Draft challenges
    pub async fn update_challenge(
        &self,
        challenge_id: Uuid,
        dto: UpdateChallengeDto,
    ) -> Result<ChallengeResponseDto, String> {
        let mut challenge = self
            .challenge_repo
            .find_by_id(challenge_id)
            .await?
            .ok_or("Challenge not found".to_string())?;

        // Only Draft challenges can be updated
        if challenge.status != ChallengeStatus::Draft {
            return Err("Can only update Draft challenges".to_string());
        }

        // Apply updates
        if let Some(title) = dto.title {
            challenge.update_title(title)?;
        }
        if let Some(description) = dto.description {
            challenge.update_description(description)?;
        }
        if let Some(icon) = dto.icon {
            challenge.update_icon(icon)?;
        }
        if let Some(start_date) = dto.start_date {
            challenge.update_start_date(start_date)?;
        }
        if let Some(end_date) = dto.end_date {
            challenge.update_end_date(end_date)?;
        }
        if let Some(target_value) = dto.target_value {
            challenge.update_target_value(target_value)?;
        }
        if let Some(reward_points) = dto.reward_points {
            challenge.update_reward_points(reward_points)?;
        }

        let updated = self.challenge_repo.update(&challenge).await?;
        Ok(ChallengeResponseDto::from(updated))
    }

    /// Activate challenge (Draft → Active)
    ///
    /// # Authorization
    /// - Only organization admins can activate challenges
    pub async fn activate_challenge(
        &self,
        challenge_id: Uuid,
    ) -> Result<ChallengeResponseDto, String> {
        let mut challenge = self
            .challenge_repo
            .find_by_id(challenge_id)
            .await?
            .ok_or("Challenge not found".to_string())?;

        challenge.activate()?;
        let updated = self.challenge_repo.update(&challenge).await?;
        Ok(ChallengeResponseDto::from(updated))
    }

    /// Complete challenge (Active → Completed)
    ///
    /// # Authorization
    /// - Only organization admins can complete challenges
    pub async fn complete_challenge(
        &self,
        challenge_id: Uuid,
    ) -> Result<ChallengeResponseDto, String> {
        let mut challenge = self
            .challenge_repo
            .find_by_id(challenge_id)
            .await?
            .ok_or("Challenge not found".to_string())?;

        challenge.complete()?;
        let updated = self.challenge_repo.update(&challenge).await?;
        Ok(ChallengeResponseDto::from(updated))
    }

    /// Cancel challenge (Draft/Active → Cancelled)
    ///
    /// # Authorization
    /// - Only organization admins can cancel challenges
    pub async fn cancel_challenge(
        &self,
        challenge_id: Uuid,
    ) -> Result<ChallengeResponseDto, String> {
        let mut challenge = self
            .challenge_repo
            .find_by_id(challenge_id)
            .await?
            .ok_or("Challenge not found".to_string())?;

        challenge.cancel()?;
        let updated = self.challenge_repo.update(&challenge).await?;
        Ok(ChallengeResponseDto::from(updated))
    }

    /// Delete challenge (admin only)
    pub async fn delete_challenge(&self, challenge_id: Uuid) -> Result<(), String> {
        self.challenge_repo.delete(challenge_id).await
    }

    /// Get user progress for a challenge
    pub async fn get_challenge_progress(
        &self,
        user_id: Uuid,
        challenge_id: Uuid,
    ) -> Result<ChallengeProgressResponseDto, String> {
        let progress = self
            .progress_repo
            .find_by_user_and_challenge(user_id, challenge_id)
            .await?
            .ok_or("Progress not found".to_string())?;

        let challenge = self
            .challenge_repo
            .find_by_id(challenge_id)
            .await?
            .ok_or("Challenge not found".to_string())?;

        Ok(ChallengeProgressResponseDto::from_entities(
            progress, challenge,
        ))
    }

    /// List all progress for a challenge
    pub async fn list_challenge_progress(
        &self,
        challenge_id: Uuid,
    ) -> Result<Vec<ChallengeProgressResponseDto>, String> {
        let progress_list = self.progress_repo.find_by_challenge(challenge_id).await?;

        let challenge = self
            .challenge_repo
            .find_by_id(challenge_id)
            .await?
            .ok_or("Challenge not found".to_string())?;

        Ok(progress_list
            .into_iter()
            .map(|p| ChallengeProgressResponseDto::from_entities(p, challenge.clone()))
            .collect())
    }

    /// List active challenges for a user with progress
    pub async fn list_user_active_progress(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ChallengeProgressResponseDto>, String> {
        let progress_list = self.progress_repo.find_active_by_user(user_id).await?;

        // Enrich with challenge data
        let mut enriched = Vec::new();
        for progress in progress_list {
            if let Some(challenge) = self
                .challenge_repo
                .find_by_id(progress.challenge_id)
                .await?
            {
                enriched.push(ChallengeProgressResponseDto::from_entities(
                    progress, challenge,
                ));
            }
        }

        Ok(enriched)
    }

    /// Increment user progress for a challenge
    ///
    /// Creates progress if doesn't exist, increments if exists.
    /// Automatically completes challenge if target reached.
    pub async fn increment_progress(
        &self,
        user_id: Uuid,
        challenge_id: Uuid,
        increment: i32,
    ) -> Result<ChallengeProgressResponseDto, String> {
        let challenge = self
            .challenge_repo
            .find_by_id(challenge_id)
            .await?
            .ok_or("Challenge not found".to_string())?;

        // Get or create progress
        let mut progress = match self
            .progress_repo
            .find_by_user_and_challenge(user_id, challenge_id)
            .await?
        {
            Some(p) => p,
            None => {
                let new_progress = ChallengeProgress::new(challenge_id, user_id);
                self.progress_repo.create(&new_progress).await?
            }
        };

        // Increment progress
        progress.increment(increment)?;

        // Check if completed
        if progress.current_value >= challenge.target_value && !progress.completed {
            progress.mark_completed()?;
        }

        let updated = self.progress_repo.update(&progress).await?;
        Ok(ChallengeProgressResponseDto::from_entities(
            updated, challenge,
        ))
    }
}

// ============================================================================
// Gamification Stats Use Cases
// ============================================================================

/// Use cases for gamification statistics and leaderboards
///
/// Orchestrates complex queries across achievements, challenges, and user data.
pub struct GamificationStatsUseCases {
    achievement_repo: Arc<dyn AchievementRepository>,
    user_achievement_repo: Arc<dyn UserAchievementRepository>,
    #[allow(dead_code)]
    challenge_repo: Arc<dyn ChallengeRepository>,
    progress_repo: Arc<dyn ChallengeProgressRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl GamificationStatsUseCases {
    pub fn new(
        achievement_repo: Arc<dyn AchievementRepository>,
        user_achievement_repo: Arc<dyn UserAchievementRepository>,
        challenge_repo: Arc<dyn ChallengeRepository>,
        progress_repo: Arc<dyn ChallengeProgressRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            achievement_repo,
            user_achievement_repo,
            challenge_repo,
            progress_repo,
            user_repo,
        }
    }

    /// Get comprehensive gamification stats for a user
    pub async fn get_user_stats(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<UserGamificationStatsDto, String> {
        // Calculate achievement points
        let total_points = self
            .user_achievement_repo
            .calculate_total_points(user_id)
            .await?;

        // Count achievements
        let achievements_earned = self.user_achievement_repo.count_by_user(user_id).await? as i32;
        let achievements_available = self
            .achievement_repo
            .count_by_organization(organization_id)
            .await? as i32;

        // Count challenges
        let challenges_completed =
            self.progress_repo.count_completed_by_user(user_id).await? as i32;
        let challenges_active = self.progress_repo.find_active_by_user(user_id).await?.len() as i32;

        // Get recent achievements (last 5)
        let recent = self
            .user_achievement_repo
            .find_recent_by_user(user_id, 5)
            .await?;
        let mut recent_achievements = Vec::new();
        for ua in recent {
            if let Some(achievement) = self.achievement_repo.find_by_id(ua.achievement_id).await? {
                recent_achievements
                    .push(UserAchievementResponseDto::from_entities(ua, achievement));
            }
        }

        // Calculate rank from leaderboard
        // Get leaderboard with large limit to find user's rank
        let leaderboard = self.get_leaderboard(organization_id, None, 10000).await?;
        let rank = leaderboard
            .entries
            .iter()
            .find(|entry| entry.user_id == user_id)
            .map(|entry| entry.rank);

        Ok(UserGamificationStatsDto {
            user_id,
            total_points,
            achievements_earned,
            achievements_available,
            challenges_completed,
            challenges_active,
            rank,
            recent_achievements,
        })
    }

    /// Get leaderboard for an organization or building
    pub async fn get_leaderboard(
        &self,
        organization_id: Uuid,
        building_id: Option<Uuid>,
        limit: i64,
    ) -> Result<LeaderboardResponseDto, String> {
        // Get leaderboard data from challenge progress (points from completed challenges)
        let leaderboard_data = self
            .progress_repo
            .get_leaderboard(organization_id, building_id, limit)
            .await?;

        // Enrich with user data
        let mut entries = Vec::new();
        let mut rank = 1;
        for (user_id, challenge_points) in leaderboard_data {
            // Get achievement points
            let achievement_points = self
                .user_achievement_repo
                .calculate_total_points(user_id)
                .await?;

            // Get counts
            let achievements_count =
                self.user_achievement_repo.count_by_user(user_id).await? as i32;
            let challenges_completed =
                self.progress_repo.count_completed_by_user(user_id).await? as i32;

            // Get username
            let username = if let Some(user) = self.user_repo.find_by_id(user_id).await? {
                format!("{} {}", user.first_name, user.last_name)
            } else {
                "Unknown User".to_string()
            };

            entries.push(LeaderboardEntryDto {
                user_id,
                username,
                total_points: achievement_points + challenge_points,
                achievements_count,
                challenges_completed,
                rank,
            });

            rank += 1;
        }

        Ok(LeaderboardResponseDto {
            organization_id,
            building_id,
            entries,
            total_users: rank - 1,
        })
    }
}
