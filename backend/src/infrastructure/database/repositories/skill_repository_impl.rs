use crate::application::ports::SkillRepository;
use crate::domain::entities::{ExpertiseLevel, Skill, SkillCategory};
use crate::infrastructure::database::DbPool;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresSkillRepository {
    pool: DbPool,
}

impl PostgresSkillRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

/// Helper function to map database row to Skill entity
fn map_row_to_skill(row: &sqlx::postgres::PgRow) -> Skill {
    let category_str: String = row.get("skill_category");
    let level_str: String = row.get("expertise_level");

    Skill {
        id: row.get("id"),
        owner_id: row.get("owner_id"),
        building_id: row.get("building_id"),
        skill_category: serde_json::from_str(&format!("\"{}\"", category_str))
            .unwrap_or(SkillCategory::Other),
        skill_name: row.get("skill_name"),
        expertise_level: serde_json::from_str(&format!("\"{}\"", level_str))
            .unwrap_or(ExpertiseLevel::Beginner),
        description: row.get("description"),
        is_available_for_help: row.get("is_available_for_help"),
        hourly_rate_credits: row.get("hourly_rate_credits"),
        years_of_experience: row.get("years_of_experience"),
        certifications: row.get("certifications"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl SkillRepository for PostgresSkillRepository {
    async fn create(&self, skill: &Skill) -> Result<Skill, String> {
        let category_str = serde_json::to_string(&skill.skill_category)
            .map_err(|e| format!("Failed to serialize skill_category: {}", e))?
            .trim_matches('"')
            .to_string();

        let level_str = serde_json::to_string(&skill.expertise_level)
            .map_err(|e| format!("Failed to serialize expertise_level: {}", e))?
            .trim_matches('"')
            .to_string();

        sqlx::query(
            r#"
            INSERT INTO skills (
                id, owner_id, building_id, skill_category, skill_name, expertise_level,
                description, is_available_for_help, hourly_rate_credits,
                years_of_experience, certifications, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4::skill_category, $5, $6::expertise_level, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(skill.id)
        .bind(skill.owner_id)
        .bind(skill.building_id)
        .bind(&category_str)
        .bind(&skill.skill_name)
        .bind(&level_str)
        .bind(&skill.description)
        .bind(skill.is_available_for_help)
        .bind(skill.hourly_rate_credits)
        .bind(skill.years_of_experience)
        .bind(&skill.certifications)
        .bind(skill.created_at)
        .bind(skill.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create skill: {}", e))?;

        Ok(skill.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Skill>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, skill_category::text AS skill_category,
                   skill_name, expertise_level::text AS expertise_level, description,
                   is_available_for_help, hourly_rate_credits, years_of_experience,
                   certifications, created_at, updated_at
            FROM skills
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find skill by ID: {}", e))?;

        Ok(row.as_ref().map(map_row_to_skill))
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Skill>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, skill_category::text AS skill_category,
                   skill_name, expertise_level::text AS expertise_level, description,
                   is_available_for_help, hourly_rate_credits, years_of_experience,
                   certifications, created_at, updated_at
            FROM skills
            WHERE building_id = $1
            ORDER BY
                is_available_for_help DESC,
                expertise_level DESC,
                skill_name ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find skills by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_skill).collect())
    }

    async fn find_available_by_building(&self, building_id: Uuid) -> Result<Vec<Skill>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, skill_category::text AS skill_category,
                   skill_name, expertise_level::text AS expertise_level, description,
                   is_available_for_help, hourly_rate_credits, years_of_experience,
                   certifications, created_at, updated_at
            FROM skills
            WHERE building_id = $1 AND is_available_for_help = TRUE
            ORDER BY
                expertise_level DESC,
                skill_name ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find available skills by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_skill).collect())
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Skill>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, skill_category::text AS skill_category,
                   skill_name, expertise_level::text AS expertise_level, description,
                   is_available_for_help, hourly_rate_credits, years_of_experience,
                   certifications, created_at, updated_at
            FROM skills
            WHERE owner_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find skills by owner: {}", e))?;

        Ok(rows.iter().map(map_row_to_skill).collect())
    }

    async fn find_by_category(
        &self,
        building_id: Uuid,
        category: SkillCategory,
    ) -> Result<Vec<Skill>, String> {
        let category_str = serde_json::to_string(&category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, skill_category::text AS skill_category,
                   skill_name, expertise_level::text AS expertise_level, description,
                   is_available_for_help, hourly_rate_credits, years_of_experience,
                   certifications, created_at, updated_at
            FROM skills
            WHERE building_id = $1 AND skill_category = $2::skill_category
            ORDER BY
                is_available_for_help DESC,
                expertise_level DESC,
                skill_name ASC
            "#,
        )
        .bind(building_id)
        .bind(&category_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find skills by category: {}", e))?;

        Ok(rows.iter().map(map_row_to_skill).collect())
    }

    async fn find_by_expertise(
        &self,
        building_id: Uuid,
        level: ExpertiseLevel,
    ) -> Result<Vec<Skill>, String> {
        let level_str = serde_json::to_string(&level)
            .map_err(|e| format!("Failed to serialize expertise_level: {}", e))?
            .trim_matches('"')
            .to_string();

        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, skill_category::text AS skill_category,
                   skill_name, expertise_level::text AS expertise_level, description,
                   is_available_for_help, hourly_rate_credits, years_of_experience,
                   certifications, created_at, updated_at
            FROM skills
            WHERE building_id = $1 AND expertise_level = $2::expertise_level
            ORDER BY
                is_available_for_help DESC,
                skill_name ASC
            "#,
        )
        .bind(building_id)
        .bind(&level_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find skills by expertise: {}", e))?;

        Ok(rows.iter().map(map_row_to_skill).collect())
    }

    async fn find_free_by_building(&self, building_id: Uuid) -> Result<Vec<Skill>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, skill_category::text AS skill_category,
                   skill_name, expertise_level::text AS expertise_level, description,
                   is_available_for_help, hourly_rate_credits, years_of_experience,
                   certifications, created_at, updated_at
            FROM skills
            WHERE building_id = $1 AND (hourly_rate_credits IS NULL OR hourly_rate_credits = 0)
            ORDER BY
                is_available_for_help DESC,
                expertise_level DESC,
                skill_name ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find free skills by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_skill).collect())
    }

    async fn find_professional_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<Skill>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, building_id, skill_category::text AS skill_category,
                   skill_name, expertise_level::text AS expertise_level, description,
                   is_available_for_help, hourly_rate_credits, years_of_experience,
                   certifications, created_at, updated_at
            FROM skills
            WHERE building_id = $1
              AND (expertise_level = 'Expert' OR certifications IS NOT NULL)
            ORDER BY
                is_available_for_help DESC,
                skill_name ASC
            "#,
        )
        .bind(building_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find professional skills by building: {}", e))?;

        Ok(rows.iter().map(map_row_to_skill).collect())
    }

    async fn update(&self, skill: &Skill) -> Result<Skill, String> {
        let category_str = serde_json::to_string(&skill.skill_category)
            .map_err(|e| format!("Failed to serialize skill_category: {}", e))?
            .trim_matches('"')
            .to_string();

        let level_str = serde_json::to_string(&skill.expertise_level)
            .map_err(|e| format!("Failed to serialize expertise_level: {}", e))?
            .trim_matches('"')
            .to_string();

        sqlx::query(
            r#"
            UPDATE skills
            SET skill_category = $2::skill_category,
                skill_name = $3,
                expertise_level = $4::expertise_level,
                description = $5,
                is_available_for_help = $6,
                hourly_rate_credits = $7,
                years_of_experience = $8,
                certifications = $9,
                updated_at = $10
            WHERE id = $1
            "#,
        )
        .bind(skill.id)
        .bind(&category_str)
        .bind(&skill.skill_name)
        .bind(&level_str)
        .bind(&skill.description)
        .bind(skill.is_available_for_help)
        .bind(skill.hourly_rate_credits)
        .bind(skill.years_of_experience)
        .bind(&skill.certifications)
        .bind(skill.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update skill: {}", e))?;

        Ok(skill.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM skills WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete skill: {}", e))?;

        Ok(())
    }

    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM skills WHERE building_id = $1
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count skills by building: {}", e))?;

        Ok(row.get("count"))
    }

    async fn count_available_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM skills
            WHERE building_id = $1 AND is_available_for_help = TRUE
            "#,
        )
        .bind(building_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count available skills by building: {}", e))?;

        Ok(row.get("count"))
    }

    async fn count_by_category(
        &self,
        building_id: Uuid,
        category: SkillCategory,
    ) -> Result<i64, String> {
        let category_str = serde_json::to_string(&category)
            .map_err(|e| format!("Failed to serialize category: {}", e))?
            .trim_matches('"')
            .to_string();

        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM skills
            WHERE building_id = $1 AND skill_category = $2::skill_category
            "#,
        )
        .bind(building_id)
        .bind(&category_str)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count skills by category: {}", e))?;

        Ok(row.get("count"))
    }

    async fn count_by_expertise(
        &self,
        building_id: Uuid,
        level: ExpertiseLevel,
    ) -> Result<i64, String> {
        let level_str = serde_json::to_string(&level)
            .map_err(|e| format!("Failed to serialize expertise_level: {}", e))?
            .trim_matches('"')
            .to_string();

        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM skills
            WHERE building_id = $1 AND expertise_level = $2::expertise_level
            "#,
        )
        .bind(building_id)
        .bind(&level_str)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count skills by expertise: {}", e))?;

        Ok(row.get("count"))
    }
}
