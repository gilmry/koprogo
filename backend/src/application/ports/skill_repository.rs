use crate::domain::entities::{ExpertiseLevel, Skill, SkillCategory};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository port for Skill aggregate
#[async_trait]
pub trait SkillRepository: Send + Sync {
    /// Create a new skill
    async fn create(&self, skill: &Skill) -> Result<Skill, String>;

    /// Find skill by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Skill>, String>;

    /// Find all skills for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Skill>, String>;

    /// Find all available skills for a building (is_available_for_help = true)
    async fn find_available_by_building(&self, building_id: Uuid) -> Result<Vec<Skill>, String>;

    /// Find all skills by owner
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Skill>, String>;

    /// Find skills by category (HomeRepair, Languages, Technology, etc.)
    async fn find_by_category(
        &self,
        building_id: Uuid,
        category: SkillCategory,
    ) -> Result<Vec<Skill>, String>;

    /// Find skills by expertise level (Beginner, Intermediate, Advanced, Expert)
    async fn find_by_expertise(
        &self,
        building_id: Uuid,
        level: ExpertiseLevel,
    ) -> Result<Vec<Skill>, String>;

    /// Find free/volunteer skills for a building (hourly_rate_credits IS NULL OR = 0)
    async fn find_free_by_building(&self, building_id: Uuid) -> Result<Vec<Skill>, String>;

    /// Find professional skills for a building (Expert level OR has certifications)
    async fn find_professional_by_building(&self, building_id: Uuid)
        -> Result<Vec<Skill>, String>;

    /// Update an existing skill
    async fn update(&self, skill: &Skill) -> Result<Skill, String>;

    /// Delete a skill
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Count total skills for a building
    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count available skills for a building
    async fn count_available_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count skills by category (for statistics)
    async fn count_by_category(
        &self,
        building_id: Uuid,
        category: SkillCategory,
    ) -> Result<i64, String>;

    /// Count skills by expertise level (for statistics)
    async fn count_by_expertise(
        &self,
        building_id: Uuid,
        level: ExpertiseLevel,
    ) -> Result<i64, String>;
}
