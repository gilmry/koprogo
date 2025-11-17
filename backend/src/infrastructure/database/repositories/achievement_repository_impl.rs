use crate::application::ports::{AchievementRepository, UserAchievementRepository};
use crate::domain::entities::{Achievement, AchievementCategory, AchievementTier, UserAchievement};
use crate::infrastructure::pool::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresAchievementRepository {
    pool: DbPool,
}

impl PostgresAchievementRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Helper to convert database row to Achievement entity
    fn row_to_achievement(row: &sqlx::postgres::PgRow) -> Result<Achievement, String> {
        // Parse ENUMs from database strings
        let category_str: String = row
            .try_get("category")
            .map_err(|e| format!("Failed to get category: {}", e))?;
        let category: AchievementCategory = serde_json::from_str(&format!("\"{}\"", category_str))
            .map_err(|e| format!("Failed to parse category: {}", e))?;

        let tier_str: String = row
            .try_get("tier")
            .map_err(|e| format!("Failed to get tier: {}", e))?;
        let tier: AchievementTier = serde_json::from_str(&format!("\"{}\"", tier_str))
            .map_err(|e| format!("Failed to parse tier: {}", e))?;

        Ok(Achievement {
            id: row
                .try_get("id")
                .map_err(|e| format!("Failed to get id: {}", e))?,
            organization_id: row
                .try_get("organization_id")
                .map_err(|e| format!("Failed to get organization_id: {}", e))?,
            category,
            tier,
            name: row
                .try_get("name")
                .map_err(|e| format!("Failed to get name: {}", e))?,
            description: row
                .try_get("description")
                .map_err(|e| format!("Failed to get description: {}", e))?,
            icon: row
                .try_get("icon")
                .map_err(|e| format!("Failed to get icon: {}", e))?,
            points_value: row
                .try_get("points_value")
                .map_err(|e| format!("Failed to get points_value: {}", e))?,
            requirements: row
                .try_get("requirements")
                .map_err(|e| format!("Failed to get requirements: {}", e))?,
            is_secret: row
                .try_get("is_secret")
                .map_err(|e| format!("Failed to get is_secret: {}", e))?,
            is_repeatable: row
                .try_get("is_repeatable")
                .map_err(|e| format!("Failed to get is_repeatable: {}", e))?,
            display_order: row
                .try_get("display_order")
                .map_err(|e| format!("Failed to get display_order: {}", e))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| format!("Failed to get created_at: {}", e))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|e| format!("Failed to get updated_at: {}", e))?,
        })
    }
}

#[async_trait]
impl AchievementRepository for PostgresAchievementRepository {
    async fn create(&self, achievement: &Achievement) -> Result<Achievement, String> {
        // Serialize ENUMs to strings for database
        let category_str = serde_json::to_string(&achievement.category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?
            .trim_matches('"')
            .to_string();

        let tier_str = serde_json::to_string(&achievement.tier)
            .map_err(|e| format!("Failed to serialize tier: {}", e))?
            .trim_matches('"')
            .to_string();

        sqlx::query(
            r#"
            INSERT INTO achievements (
                id, organization_id, category, tier, name, description, icon,
                points_value, requirements, is_secret, is_repeatable, display_order,
                created_at, updated_at
            )
            VALUES ($1, $2, $3::achievement_category, $4::achievement_tier, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(&achievement.id)
        .bind(&achievement.organization_id)
        .bind(&category_str)
        .bind(&tier_str)
        .bind(&achievement.name)
        .bind(&achievement.description)
        .bind(&achievement.icon)
        .bind(&achievement.points_value)
        .bind(&achievement.requirements)
        .bind(&achievement.is_secret)
        .bind(&achievement.is_repeatable)
        .bind(&achievement.display_order)
        .bind(&achievement.created_at)
        .bind(&achievement.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create achievement: {}", e))?;

        Ok(achievement.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Achievement>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, category, tier, name, description, icon,
                   points_value, requirements, is_secret, is_repeatable, display_order,
                   created_at, updated_at
            FROM achievements
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find achievement by id: {}", e))?;

        row.as_ref().map(Self::row_to_achievement).transpose()
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<Achievement>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, category, tier, name, description, icon,
                   points_value, requirements, is_secret, is_repeatable, display_order,
                   created_at, updated_at
            FROM achievements
            WHERE organization_id = $1
            ORDER BY display_order ASC, tier ASC, name ASC
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find achievements by organization: {}", e))?;

        rows.iter().map(Self::row_to_achievement).collect()
    }

    async fn find_by_organization_and_category(
        &self,
        organization_id: Uuid,
        category: AchievementCategory,
    ) -> Result<Vec<Achievement>, String> {
        let category_str = serde_json::to_string(&category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, category, tier, name, description, icon,
                   points_value, requirements, is_secret, is_repeatable, display_order,
                   created_at, updated_at
            FROM achievements
            WHERE organization_id = $1
              AND category = $2::achievement_category
            ORDER BY display_order ASC, tier ASC, name ASC
            "#,
        )
        .bind(organization_id)
        .bind(&category_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find achievements by category: {}", e))?;

        rows.iter().map(Self::row_to_achievement).collect()
    }

    async fn find_visible_for_user(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Achievement>, String> {
        // Return all non-secret achievements OR secret achievements the user has earned
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT a.id, a.organization_id, a.category, a.tier, a.name,
                   a.description, a.icon, a.points_value, a.requirements,
                   a.is_secret, a.is_repeatable, a.display_order,
                   a.created_at, a.updated_at
            FROM achievements a
            LEFT JOIN user_achievements ua ON ua.achievement_id = a.id AND ua.user_id = $2
            WHERE a.organization_id = $1
              AND (a.is_secret = FALSE OR ua.id IS NOT NULL)
            ORDER BY a.display_order ASC, a.tier ASC, a.name ASC
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find visible achievements: {}", e))?;

        rows.iter().map(Self::row_to_achievement).collect()
    }

    async fn update(&self, achievement: &Achievement) -> Result<Achievement, String> {
        let category_str = serde_json::to_string(&achievement.category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?
            .trim_matches('"')
            .to_string();

        let tier_str = serde_json::to_string(&achievement.tier)
            .map_err(|e| format!("Failed to serialize tier: {}", e))?
            .trim_matches('"')
            .to_string();

        let result = sqlx::query(
            r#"
            UPDATE achievements
            SET category = $2::achievement_category,
                tier = $3::achievement_tier,
                name = $4,
                description = $5,
                icon = $6,
                points_value = $7,
                requirements = $8,
                is_secret = $9,
                is_repeatable = $10,
                display_order = $11,
                updated_at = $12
            WHERE id = $1
            "#,
        )
        .bind(&achievement.id)
        .bind(&category_str)
        .bind(&tier_str)
        .bind(&achievement.name)
        .bind(&achievement.description)
        .bind(&achievement.icon)
        .bind(&achievement.points_value)
        .bind(&achievement.requirements)
        .bind(&achievement.is_secret)
        .bind(&achievement.is_repeatable)
        .bind(&achievement.display_order)
        .bind(&achievement.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update achievement: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Achievement not found".to_string());
        }

        Ok(achievement.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        let result = sqlx::query(
            r#"
            DELETE FROM achievements
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete achievement: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Achievement not found".to_string());
        }

        Ok(())
    }

    async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM achievements
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count achievements: {}", e))?;

        let count: i64 = row
            .try_get("count")
            .map_err(|e| format!("Failed to get count: {}", e))?;

        Ok(count)
    }
}

// ============================================================================
// UserAchievementRepository Implementation
// ============================================================================

pub struct PostgresUserAchievementRepository {
    pool: DbPool,
}

impl PostgresUserAchievementRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Helper to convert database row to UserAchievement entity
    fn row_to_user_achievement(row: &sqlx::postgres::PgRow) -> Result<UserAchievement, String> {
        Ok(UserAchievement {
            id: row
                .try_get("id")
                .map_err(|e| format!("Failed to get id: {}", e))?,
            user_id: row
                .try_get("user_id")
                .map_err(|e| format!("Failed to get user_id: {}", e))?,
            achievement_id: row
                .try_get("achievement_id")
                .map_err(|e| format!("Failed to get achievement_id: {}", e))?,
            earned_at: row
                .try_get("earned_at")
                .map_err(|e| format!("Failed to get earned_at: {}", e))?,
            progress_data: row
                .try_get("progress_data")
                .map_err(|e| format!("Failed to get progress_data: {}", e))?,
            times_earned: row
                .try_get("times_earned")
                .map_err(|e| format!("Failed to get times_earned: {}", e))?,
        })
    }
}

#[async_trait]
impl UserAchievementRepository for PostgresUserAchievementRepository {
    async fn create(
        &self,
        user_achievement: &UserAchievement,
    ) -> Result<UserAchievement, String> {
        sqlx::query(
            r#"
            INSERT INTO user_achievements (
                id, user_id, achievement_id, earned_at, progress_data, times_earned
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&user_achievement.id)
        .bind(&user_achievement.user_id)
        .bind(&user_achievement.achievement_id)
        .bind(&user_achievement.earned_at)
        .bind(&user_achievement.progress_data)
        .bind(&user_achievement.times_earned)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create user achievement: {}", e))?;

        Ok(user_achievement.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserAchievement>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, achievement_id, earned_at, progress_data, times_earned
            FROM user_achievements
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find user achievement by id: {}", e))?;

        row.as_ref()
            .map(Self::row_to_user_achievement)
            .transpose()
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<UserAchievement>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, achievement_id, earned_at, progress_data, times_earned
            FROM user_achievements
            WHERE user_id = $1
            ORDER BY earned_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find user achievements: {}", e))?;

        rows.iter().map(Self::row_to_user_achievement).collect()
    }

    async fn find_by_user_and_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<Option<UserAchievement>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, achievement_id, earned_at, progress_data, times_earned
            FROM user_achievements
            WHERE user_id = $1 AND achievement_id = $2
            "#,
        )
        .bind(user_id)
        .bind(achievement_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find user achievement: {}", e))?;

        row.as_ref()
            .map(Self::row_to_user_achievement)
            .transpose()
    }

    async fn update(
        &self,
        user_achievement: &UserAchievement,
    ) -> Result<UserAchievement, String> {
        let result = sqlx::query(
            r#"
            UPDATE user_achievements
            SET progress_data = $2,
                times_earned = $3
            WHERE id = $1
            "#,
        )
        .bind(&user_achievement.id)
        .bind(&user_achievement.progress_data)
        .bind(&user_achievement.times_earned)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update user achievement: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("User achievement not found".to_string());
        }

        Ok(user_achievement.clone())
    }

    async fn calculate_total_points(&self, user_id: Uuid) -> Result<i32, String> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(a.points_value * ua.times_earned), 0) as total_points
            FROM user_achievements ua
            JOIN achievements a ON a.id = ua.achievement_id
            WHERE ua.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to calculate total points: {}", e))?;

        let total_points: i64 = row
            .try_get("total_points")
            .map_err(|e| format!("Failed to get total_points: {}", e))?;

        Ok(total_points as i32)
    }

    async fn count_by_user(&self, user_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM user_achievements
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count user achievements: {}", e))?;

        let count: i64 = row
            .try_get("count")
            .map_err(|e| format!("Failed to get count: {}", e))?;

        Ok(count)
    }

    async fn find_recent_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<UserAchievement>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, achievement_id, earned_at, progress_data, times_earned
            FROM user_achievements
            WHERE user_id = $1
            ORDER BY earned_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find recent user achievements: {}", e))?;

        rows.iter().map(Self::row_to_user_achievement).collect()
    }
}
