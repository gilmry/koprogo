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
        if let Some(category) = dto.category {
            achievement.category = category;
        }
        if let Some(tier) = dto.tier {
            achievement.tier = tier;
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
        let achievement_points = self
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

        // Calculate rank and total points from leaderboard (includes achievements + challenges)
        let leaderboard = self.get_leaderboard(organization_id, None, 10000).await?;
        let leaderboard_entry = leaderboard
            .entries
            .iter()
            .find(|entry| entry.user_id == user_id);
        let rank = leaderboard_entry.map(|entry| entry.rank);
        let total_points = leaderboard_entry
            .map(|entry| entry.total_points)
            .unwrap_or(achievement_points);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{
        AchievementRepository, ChallengeProgressRepository, ChallengeRepository,
        UserAchievementRepository, UserRepository,
    };
    use crate::domain::entities::{
        Achievement, AchievementCategory, AchievementTier, Challenge, ChallengeProgress,
        ChallengeStatus, ChallengeType, User, UserAchievement, UserRole,
    };
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    // ── Mock AchievementRepository ──────────────────────────────────────────
    struct MockAchievementRepo {
        items: Mutex<HashMap<Uuid, Achievement>>,
    }
    impl MockAchievementRepo {
        fn new() -> Self {
            Self { items: Mutex::new(HashMap::new()) }
        }
    }
    #[async_trait]
    impl AchievementRepository for MockAchievementRepo {
        async fn create(&self, a: &Achievement) -> Result<Achievement, String> {
            self.items.lock().unwrap().insert(a.id, a.clone());
            Ok(a.clone())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Achievement>, String> {
            Ok(self.items.lock().unwrap().get(&id).cloned())
        }
        async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<Achievement>, String> {
            Ok(self.items.lock().unwrap().values().filter(|a| a.organization_id == org_id).cloned().collect())
        }
        async fn find_by_organization_and_category(&self, org_id: Uuid, cat: AchievementCategory) -> Result<Vec<Achievement>, String> {
            Ok(self.items.lock().unwrap().values().filter(|a| a.organization_id == org_id && a.category == cat).cloned().collect())
        }
        async fn find_visible_for_user(&self, org_id: Uuid, _user_id: Uuid) -> Result<Vec<Achievement>, String> {
            Ok(self.items.lock().unwrap().values().filter(|a| a.organization_id == org_id && !a.is_secret).cloned().collect())
        }
        async fn update(&self, a: &Achievement) -> Result<Achievement, String> {
            self.items.lock().unwrap().insert(a.id, a.clone());
            Ok(a.clone())
        }
        async fn delete(&self, id: Uuid) -> Result<(), String> {
            self.items.lock().unwrap().remove(&id);
            Ok(())
        }
        async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, String> {
            Ok(self.items.lock().unwrap().values().filter(|a| a.organization_id == org_id).count() as i64)
        }
    }

    // ── Mock UserAchievementRepository ──────────────────────────────────────
    struct MockUserAchievementRepo {
        items: Mutex<HashMap<Uuid, UserAchievement>>,
    }
    impl MockUserAchievementRepo {
        fn new() -> Self {
            Self { items: Mutex::new(HashMap::new()) }
        }
    }
    #[async_trait]
    impl UserAchievementRepository for MockUserAchievementRepo {
        async fn create(&self, ua: &UserAchievement) -> Result<UserAchievement, String> {
            self.items.lock().unwrap().insert(ua.id, ua.clone());
            Ok(ua.clone())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<UserAchievement>, String> {
            Ok(self.items.lock().unwrap().get(&id).cloned())
        }
        async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<UserAchievement>, String> {
            Ok(self.items.lock().unwrap().values().filter(|u| u.user_id == user_id).cloned().collect())
        }
        async fn find_by_user_and_achievement(&self, user_id: Uuid, achievement_id: Uuid) -> Result<Option<UserAchievement>, String> {
            Ok(self.items.lock().unwrap().values().find(|u| u.user_id == user_id && u.achievement_id == achievement_id).cloned())
        }
        async fn update(&self, ua: &UserAchievement) -> Result<UserAchievement, String> {
            self.items.lock().unwrap().insert(ua.id, ua.clone());
            Ok(ua.clone())
        }
        async fn calculate_total_points(&self, _user_id: Uuid) -> Result<i32, String> {
            Ok(0)
        }
        async fn count_by_user(&self, user_id: Uuid) -> Result<i64, String> {
            Ok(self.items.lock().unwrap().values().filter(|u| u.user_id == user_id).count() as i64)
        }
        async fn find_recent_by_user(&self, user_id: Uuid, limit: i64) -> Result<Vec<UserAchievement>, String> {
            let map = self.items.lock().unwrap();
            let mut v: Vec<_> = map.values().filter(|u| u.user_id == user_id).cloned().collect();
            v.sort_by(|a, b| b.earned_at.cmp(&a.earned_at));
            v.truncate(limit as usize);
            Ok(v)
        }
    }

    // ── Mock ChallengeRepository ────────────────────────────────────────────
    struct MockChallengeRepo {
        items: Mutex<HashMap<Uuid, Challenge>>,
    }
    impl MockChallengeRepo {
        fn new() -> Self {
            Self { items: Mutex::new(HashMap::new()) }
        }
    }
    #[async_trait]
    impl ChallengeRepository for MockChallengeRepo {
        async fn create(&self, c: &Challenge) -> Result<Challenge, String> {
            self.items.lock().unwrap().insert(c.id, c.clone());
            Ok(c.clone())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Challenge>, String> {
            Ok(self.items.lock().unwrap().get(&id).cloned())
        }
        async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<Challenge>, String> {
            Ok(self.items.lock().unwrap().values().filter(|c| c.organization_id == org_id).cloned().collect())
        }
        async fn find_by_organization_and_status(&self, org_id: Uuid, status: ChallengeStatus) -> Result<Vec<Challenge>, String> {
            Ok(self.items.lock().unwrap().values().filter(|c| c.organization_id == org_id && c.status == status).cloned().collect())
        }
        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Challenge>, String> {
            Ok(self.items.lock().unwrap().values().filter(|c| c.building_id == Some(building_id)).cloned().collect())
        }
        async fn find_active(&self, org_id: Uuid) -> Result<Vec<Challenge>, String> {
            let now = Utc::now();
            Ok(self.items.lock().unwrap().values().filter(|c| c.organization_id == org_id && c.status == ChallengeStatus::Active && now >= c.start_date && now < c.end_date).cloned().collect())
        }
        async fn find_ended_not_completed(&self) -> Result<Vec<Challenge>, String> {
            let now = Utc::now();
            Ok(self.items.lock().unwrap().values().filter(|c| c.status == ChallengeStatus::Active && now >= c.end_date).cloned().collect())
        }
        async fn update(&self, c: &Challenge) -> Result<Challenge, String> {
            self.items.lock().unwrap().insert(c.id, c.clone());
            Ok(c.clone())
        }
        async fn delete(&self, id: Uuid) -> Result<(), String> {
            self.items.lock().unwrap().remove(&id);
            Ok(())
        }
        async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, String> {
            Ok(self.items.lock().unwrap().values().filter(|c| c.organization_id == org_id).count() as i64)
        }
    }

    // ── Mock ChallengeProgressRepository ────────────────────────────────────
    struct MockProgressRepo {
        items: Mutex<HashMap<Uuid, ChallengeProgress>>,
    }
    impl MockProgressRepo {
        fn new() -> Self {
            Self { items: Mutex::new(HashMap::new()) }
        }
    }
    #[async_trait]
    impl ChallengeProgressRepository for MockProgressRepo {
        async fn create(&self, p: &ChallengeProgress) -> Result<ChallengeProgress, String> {
            self.items.lock().unwrap().insert(p.id, p.clone());
            Ok(p.clone())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<ChallengeProgress>, String> {
            Ok(self.items.lock().unwrap().get(&id).cloned())
        }
        async fn find_by_user_and_challenge(&self, user_id: Uuid, challenge_id: Uuid) -> Result<Option<ChallengeProgress>, String> {
            Ok(self.items.lock().unwrap().values().find(|p| p.user_id == user_id && p.challenge_id == challenge_id).cloned())
        }
        async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ChallengeProgress>, String> {
            Ok(self.items.lock().unwrap().values().filter(|p| p.user_id == user_id).cloned().collect())
        }
        async fn find_by_challenge(&self, challenge_id: Uuid) -> Result<Vec<ChallengeProgress>, String> {
            Ok(self.items.lock().unwrap().values().filter(|p| p.challenge_id == challenge_id).cloned().collect())
        }
        async fn find_active_by_user(&self, user_id: Uuid) -> Result<Vec<ChallengeProgress>, String> {
            Ok(self.items.lock().unwrap().values().filter(|p| p.user_id == user_id && !p.completed).cloned().collect())
        }
        async fn update(&self, p: &ChallengeProgress) -> Result<ChallengeProgress, String> {
            self.items.lock().unwrap().insert(p.id, p.clone());
            Ok(p.clone())
        }
        async fn count_completed_by_user(&self, user_id: Uuid) -> Result<i64, String> {
            Ok(self.items.lock().unwrap().values().filter(|p| p.user_id == user_id && p.completed).count() as i64)
        }
        async fn get_leaderboard(&self, _org_id: Uuid, _building_id: Option<Uuid>, _limit: i64) -> Result<Vec<(Uuid, i32)>, String> {
            Ok(vec![])
        }
    }

    // ── Mock UserRepository (minimal, needed by AchievementUseCases) ────────
    struct MockUserRepo;
    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn create(&self, _u: &User) -> Result<User, String> { Err("not impl".to_string()) }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<User>, String> { Ok(None) }
        async fn find_by_email(&self, _e: &str) -> Result<Option<User>, String> { Ok(None) }
        async fn find_all(&self) -> Result<Vec<User>, String> { Ok(vec![]) }
        async fn find_by_organization(&self, _o: Uuid) -> Result<Vec<User>, String> { Ok(vec![]) }
        async fn update(&self, _u: &User) -> Result<User, String> { Err("not impl".to_string()) }
        async fn delete(&self, _id: Uuid) -> Result<bool, String> { Ok(false) }
        async fn count_by_organization(&self, _o: Uuid) -> Result<i64, String> { Ok(0) }
    }

    // ── Helpers ─────────────────────────────────────────────────────────────
    fn make_achievement_dto(org_id: Uuid) -> CreateAchievementDto {
        CreateAchievementDto {
            organization_id: org_id,
            category: AchievementCategory::Community,
            tier: AchievementTier::Bronze,
            name: "First Booking".to_string(),
            description: "Made your first resource booking in the system".to_string(),
            icon: "star".to_string(),
            points_value: 10,
            requirements: r#"{"action": "booking_created", "count": 1}"#.to_string(),
            is_secret: false,
            is_repeatable: false,
            display_order: 1,
        }
    }

    fn make_challenge_dto(org_id: Uuid) -> CreateChallengeDto {
        let start = Utc::now() + Duration::days(1);
        let end = start + Duration::days(7);
        CreateChallengeDto {
            organization_id: org_id,
            building_id: None,
            challenge_type: ChallengeType::Individual,
            title: "Booking Week".to_string(),
            description: "Make 5 resource bookings this week to earn points!".to_string(),
            icon: "calendar".to_string(),
            start_date: start,
            end_date: end,
            target_metric: "bookings_created".to_string(),
            target_value: 5,
            reward_points: 50,
        }
    }

    fn setup_achievement_uc() -> (AchievementUseCases, Uuid) {
        let org_id = Uuid::new_v4();
        let uc = AchievementUseCases::new(
            Arc::new(MockAchievementRepo::new()) as Arc<dyn AchievementRepository>,
            Arc::new(MockUserAchievementRepo::new()) as Arc<dyn UserAchievementRepository>,
            Arc::new(MockUserRepo) as Arc<dyn UserRepository>,
        );
        (uc, org_id)
    }

    fn setup_challenge_uc() -> (ChallengeUseCases, Uuid) {
        let org_id = Uuid::new_v4();
        let uc = ChallengeUseCases::new(
            Arc::new(MockChallengeRepo::new()) as Arc<dyn ChallengeRepository>,
            Arc::new(MockProgressRepo::new()) as Arc<dyn ChallengeProgressRepository>,
        );
        (uc, org_id)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  AchievementUseCases Tests
    // ══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_create_achievement_success() {
        let (uc, org_id) = setup_achievement_uc();
        let dto = make_achievement_dto(org_id);
        let result = uc.create_achievement(dto).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.name, "First Booking");
        assert_eq!(resp.points_value, 10);
    }

    #[tokio::test]
    async fn test_get_achievement_success() {
        let (uc, org_id) = setup_achievement_uc();
        let dto = make_achievement_dto(org_id);
        let created = uc.create_achievement(dto).await.unwrap();

        let result = uc.get_achievement(created.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_get_achievement_not_found() {
        let (uc, _) = setup_achievement_uc();
        let result = uc.get_achievement(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Achievement not found");
    }

    #[tokio::test]
    async fn test_award_achievement_success() {
        let (uc, org_id) = setup_achievement_uc();
        let dto = make_achievement_dto(org_id);
        let created = uc.create_achievement(dto).await.unwrap();

        let user_id = Uuid::new_v4();
        let result = uc.award_achievement(user_id, created.id, None).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.user_id, user_id);
        assert_eq!(resp.times_earned, 1);
    }

    #[tokio::test]
    async fn test_award_non_repeatable_twice_fails() {
        let (uc, org_id) = setup_achievement_uc();
        let dto = make_achievement_dto(org_id); // is_repeatable = false
        let created = uc.create_achievement(dto).await.unwrap();

        let user_id = Uuid::new_v4();
        uc.award_achievement(user_id, created.id, None).await.unwrap();

        // Second award should fail
        let result = uc.award_achievement(user_id, created.id, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not repeatable"));
    }

    #[tokio::test]
    async fn test_award_repeatable_increments() {
        let (uc, org_id) = setup_achievement_uc();
        let mut dto = make_achievement_dto(org_id);
        dto.is_repeatable = true;
        let created = uc.create_achievement(dto).await.unwrap();

        let user_id = Uuid::new_v4();
        uc.award_achievement(user_id, created.id, None).await.unwrap();

        // Second award should succeed and increment
        let result = uc.award_achievement(user_id, created.id, None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().times_earned, 2);
    }

    #[tokio::test]
    async fn test_list_achievements() {
        let (uc, org_id) = setup_achievement_uc();
        let dto1 = make_achievement_dto(org_id);
        let mut dto2 = make_achievement_dto(org_id);
        dto2.name = "SEL Pioneer".to_string();
        dto2.description = "Completed your first SEL exchange in the system".to_string();

        uc.create_achievement(dto1).await.unwrap();
        uc.create_achievement(dto2).await.unwrap();

        let result = uc.list_achievements(org_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_delete_achievement() {
        let (uc, org_id) = setup_achievement_uc();
        let dto = make_achievement_dto(org_id);
        let created = uc.create_achievement(dto).await.unwrap();

        let result = uc.delete_achievement(created.id).await;
        assert!(result.is_ok());

        let fetch = uc.get_achievement(created.id).await;
        assert!(fetch.is_err());
    }

    // ══════════════════════════════════════════════════════════════════════
    //  ChallengeUseCases Tests
    // ══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_create_challenge_success() {
        let (uc, org_id) = setup_challenge_uc();
        let dto = make_challenge_dto(org_id);
        let result = uc.create_challenge(dto).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.title, "Booking Week");
        assert_eq!(resp.status, ChallengeStatus::Draft);
    }

    #[tokio::test]
    async fn test_activate_challenge_success() {
        let (uc, org_id) = setup_challenge_uc();
        let dto = make_challenge_dto(org_id);
        let created = uc.create_challenge(dto).await.unwrap();

        let result = uc.activate_challenge(created.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, ChallengeStatus::Active);
    }

    #[tokio::test]
    async fn test_activate_already_active_fails() {
        let (uc, org_id) = setup_challenge_uc();
        let dto = make_challenge_dto(org_id);
        let created = uc.create_challenge(dto).await.unwrap();
        uc.activate_challenge(created.id).await.unwrap();

        let result = uc.activate_challenge(created.id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already active"));
    }

    #[tokio::test]
    async fn test_cancel_challenge_success() {
        let (uc, org_id) = setup_challenge_uc();
        let dto = make_challenge_dto(org_id);
        let created = uc.create_challenge(dto).await.unwrap();

        let result = uc.cancel_challenge(created.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, ChallengeStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_challenge_not_found() {
        let (uc, _) = setup_challenge_uc();
        let result = uc.get_challenge(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Challenge not found");
    }

    #[tokio::test]
    async fn test_increment_progress_creates_and_increments() {
        let (uc, org_id) = setup_challenge_uc();
        let dto = make_challenge_dto(org_id);
        let created = uc.create_challenge(dto).await.unwrap();

        let user_id = Uuid::new_v4();

        // Increment (auto-creates progress record)
        let result = uc.increment_progress(user_id, created.id, 3).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.current_value, 3);
        assert!(!resp.completed);
    }

    #[tokio::test]
    async fn test_increment_progress_auto_completes() {
        let (uc, org_id) = setup_challenge_uc();
        let dto = make_challenge_dto(org_id); // target_value = 5
        let created = uc.create_challenge(dto).await.unwrap();

        let user_id = Uuid::new_v4();

        // Increment to reach target
        uc.increment_progress(user_id, created.id, 3).await.unwrap();
        let result = uc.increment_progress(user_id, created.id, 3).await; // total = 6 >= 5
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(resp.completed);
        assert_eq!(resp.current_value, 6);
    }

    #[tokio::test]
    async fn test_delete_challenge() {
        let (uc, org_id) = setup_challenge_uc();
        let dto = make_challenge_dto(org_id);
        let created = uc.create_challenge(dto).await.unwrap();

        let result = uc.delete_challenge(created.id).await;
        assert!(result.is_ok());

        let fetch = uc.get_challenge(created.id).await;
        assert!(fetch.is_err());
    }
}
