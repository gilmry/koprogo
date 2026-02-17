use crate::domain::entities::{
    Achievement, AchievementCategory, AchievementTier, ChallengeProgress, ChallengeStatus,
    ChallengeType, UserAchievement,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Achievement DTOs
// ============================================================================

/// DTO for creating a new achievement
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAchievementDto {
    pub organization_id: Uuid,
    pub category: AchievementCategory,
    pub tier: AchievementTier,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub points_value: i32,
    pub requirements: String, // JSON criteria
    pub is_secret: bool,
    pub is_repeatable: bool,
    pub display_order: i32,
}

/// DTO for updating an achievement
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAchievementDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<AchievementCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<AchievementTier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points_value: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_secret: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_repeatable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_order: Option<i32>,
}

/// Response DTO for Achievement
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AchievementResponseDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub category: AchievementCategory,
    pub tier: AchievementTier,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub points_value: i32,
    pub requirements: String,
    pub is_secret: bool,
    pub is_repeatable: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Achievement> for AchievementResponseDto {
    fn from(achievement: Achievement) -> Self {
        Self {
            id: achievement.id,
            organization_id: achievement.organization_id,
            category: achievement.category,
            tier: achievement.tier,
            name: achievement.name,
            description: achievement.description,
            icon: achievement.icon,
            points_value: achievement.points_value,
            requirements: achievement.requirements,
            is_secret: achievement.is_secret,
            is_repeatable: achievement.is_repeatable,
            display_order: achievement.display_order,
            created_at: achievement.created_at,
            updated_at: achievement.updated_at,
        }
    }
}

// ============================================================================
// UserAchievement DTOs
// ============================================================================

/// Response DTO for UserAchievement with enriched achievement data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAchievementResponseDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub achievement_id: Uuid,
    pub earned_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_data: Option<String>,
    pub times_earned: i32,
    // Enriched data
    pub achievement: AchievementResponseDto,
}

impl UserAchievementResponseDto {
    pub fn from_entities(user_achievement: UserAchievement, achievement: Achievement) -> Self {
        Self {
            id: user_achievement.id,
            user_id: user_achievement.user_id,
            achievement_id: user_achievement.achievement_id,
            earned_at: user_achievement.earned_at,
            progress_data: user_achievement.progress_data,
            times_earned: user_achievement.times_earned,
            achievement: AchievementResponseDto::from(achievement),
        }
    }
}

// ============================================================================
// Challenge DTOs
// ============================================================================

/// DTO for creating a new challenge
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateChallengeDto {
    pub organization_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_id: Option<Uuid>,
    pub challenge_type: ChallengeType,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub target_metric: String,
    pub target_value: i32,
    pub reward_points: i32,
}

/// DTO for updating a challenge (Draft only)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateChallengeDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_value: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward_points: Option<i32>,
}

/// Response DTO for Challenge
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChallengeResponseDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_id: Option<Uuid>,
    pub challenge_type: ChallengeType,
    pub status: ChallengeStatus,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub target_metric: String,
    pub target_value: i32,
    pub reward_points: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Computed fields
    pub duration_days: i64,
    pub is_currently_active: bool,
    pub has_ended: bool,
}

impl From<crate::domain::entities::Challenge> for ChallengeResponseDto {
    fn from(challenge: crate::domain::entities::Challenge) -> Self {
        Self {
            id: challenge.id,
            organization_id: challenge.organization_id,
            building_id: challenge.building_id,
            challenge_type: challenge.challenge_type.clone(),
            status: challenge.status.clone(),
            title: challenge.title.clone(),
            description: challenge.description.clone(),
            icon: challenge.icon.clone(),
            start_date: challenge.start_date,
            end_date: challenge.end_date,
            target_metric: challenge.target_metric.clone(),
            target_value: challenge.target_value,
            reward_points: challenge.reward_points,
            created_at: challenge.created_at,
            updated_at: challenge.updated_at,
            // Computed fields
            duration_days: challenge.duration_days(),
            is_currently_active: challenge.is_currently_active(),
            has_ended: challenge.has_ended(),
        }
    }
}

// ============================================================================
// ChallengeProgress DTOs
// ============================================================================

/// Response DTO for ChallengeProgress with enriched challenge data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChallengeProgressResponseDto {
    pub id: Uuid,
    pub challenge_id: Uuid,
    pub user_id: Uuid,
    pub current_value: i32,
    pub completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Enriched data
    pub challenge: ChallengeResponseDto,
    // Computed fields
    pub completion_percentage: f64,
}

impl ChallengeProgressResponseDto {
    pub fn from_entities(
        progress: ChallengeProgress,
        challenge: crate::domain::entities::Challenge,
    ) -> Self {
        let completion_percentage = progress.completion_percentage(challenge.target_value);
        Self {
            id: progress.id,
            challenge_id: progress.challenge_id,
            user_id: progress.user_id,
            current_value: progress.current_value,
            completed: progress.completed,
            completed_at: progress.completed_at,
            created_at: progress.created_at,
            updated_at: progress.updated_at,
            challenge: ChallengeResponseDto::from(challenge),
            completion_percentage,
        }
    }
}

// ============================================================================
// Leaderboard DTOs
// ============================================================================

/// Leaderboard entry for user ranking
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaderboardEntryDto {
    pub user_id: Uuid,
    pub username: String, // Enriched from User
    pub total_points: i32,
    pub achievements_count: i32,
    pub challenges_completed: i32,
    pub rank: i32,
}

/// Leaderboard response with top users
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaderboardResponseDto {
    pub organization_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_id: Option<Uuid>,
    pub entries: Vec<LeaderboardEntryDto>,
    pub total_users: i32,
}

/// User gamification stats
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserGamificationStatsDto {
    pub user_id: Uuid,
    pub total_points: i32,
    pub achievements_earned: i32,
    pub achievements_available: i32,
    pub challenges_completed: i32,
    pub challenges_active: i32,
    pub rank: Option<i32>,
    pub recent_achievements: Vec<UserAchievementResponseDto>, // Last 5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_response_dto_from_entity() {
        let organization_id = Uuid::new_v4();
        let achievement = Achievement::new(
            organization_id,
            AchievementCategory::Community,
            AchievementTier::Bronze,
            "First Booking".to_string(),
            "Made your first resource booking".to_string(),
            "🎉".to_string(),
            10,
            r#"{"action": "booking_created", "count": 1}"#.to_string(),
            false,
            false,
            1,
        )
        .unwrap();

        let dto = AchievementResponseDto::from(achievement.clone());

        assert_eq!(dto.id, achievement.id);
        assert_eq!(dto.name, "First Booking");
        assert_eq!(dto.category, AchievementCategory::Community);
        assert_eq!(dto.tier, AchievementTier::Bronze);
        assert_eq!(dto.points_value, 10);
    }

    #[test]
    fn test_challenge_response_dto_computed_fields() {
        let organization_id = Uuid::new_v4();
        let start_date = Utc::now() + chrono::Duration::days(1);
        let end_date = start_date + chrono::Duration::days(7);

        let challenge = crate::domain::entities::Challenge::new(
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
        .unwrap();

        let dto = ChallengeResponseDto::from(challenge);

        assert_eq!(dto.duration_days, 7);
        assert!(!dto.is_currently_active); // Not started yet
        assert!(!dto.has_ended);
    }

    #[test]
    fn test_challenge_progress_completion_percentage() {
        let organization_id = Uuid::new_v4();
        let start_date = Utc::now() + chrono::Duration::days(1);
        let end_date = start_date + chrono::Duration::days(7);

        let challenge = crate::domain::entities::Challenge::new(
            organization_id,
            None,
            ChallengeType::Individual,
            "Booking Week".to_string(),
            "Make 5 resource bookings this week to earn points!".to_string(),
            "📅".to_string(),
            start_date,
            end_date,
            "bookings_created".to_string(),
            10,
            50,
        )
        .unwrap();

        let challenge_id = challenge.id;
        let user_id = Uuid::new_v4();
        let mut progress = ChallengeProgress::new(challenge_id, user_id);
        progress.increment(3).unwrap();

        let dto = ChallengeProgressResponseDto::from_entities(progress, challenge);

        assert_eq!(dto.completion_percentage, 30.0);
        assert!(!dto.completed);
    }
}
