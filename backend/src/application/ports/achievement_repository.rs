use crate::domain::entities::{Achievement, AchievementCategory, UserAchievement};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for Achievement persistence operations
#[async_trait]
pub trait AchievementRepository: Send + Sync {
    /// Create a new achievement
    async fn create(&self, achievement: &Achievement) -> Result<Achievement, String>;

    /// Find achievement by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Achievement>, String>;

    /// Find all achievements for an organization
    async fn find_by_organization(&self, organization_id: Uuid)
        -> Result<Vec<Achievement>, String>;

    /// Find achievements by organization and category
    async fn find_by_organization_and_category(
        &self,
        organization_id: Uuid,
        category: AchievementCategory,
    ) -> Result<Vec<Achievement>, String>;

    /// Find visible achievements (non-secret or user has earned them)
    async fn find_visible_for_user(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Achievement>, String>;

    /// Update achievement
    async fn update(&self, achievement: &Achievement) -> Result<Achievement, String>;

    /// Delete achievement
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Count achievements by organization
    async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, String>;
}

/// Repository trait for UserAchievement persistence operations
#[async_trait]
pub trait UserAchievementRepository: Send + Sync {
    /// Award achievement to user
    async fn create(&self, user_achievement: &UserAchievement) -> Result<UserAchievement, String>;

    /// Find user achievement by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserAchievement>, String>;

    /// Find all achievements earned by a user
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<UserAchievement>, String>;

    /// Find specific user achievement
    async fn find_by_user_and_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<Option<UserAchievement>, String>;

    /// Update user achievement (for repeatable achievements)
    async fn update(&self, user_achievement: &UserAchievement) -> Result<UserAchievement, String>;

    /// Calculate total points for user
    async fn calculate_total_points(&self, user_id: Uuid) -> Result<i32, String>;

    /// Count achievements earned by user
    async fn count_by_user(&self, user_id: Uuid) -> Result<i64, String>;

    /// Get recent achievements for user (last N)
    async fn find_recent_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<UserAchievement>, String>;
}
