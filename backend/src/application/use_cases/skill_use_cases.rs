use crate::application::dto::{
    CategoryCount, CreateSkillDto, ExpertiseCount, SkillResponseDto, SkillStatisticsDto,
    SkillSummaryDto, UpdateSkillDto,
};
use crate::application::ports::{OwnerRepository, SkillRepository};
use crate::domain::entities::{ExpertiseLevel, Skill, SkillCategory};
use std::sync::Arc;
use uuid::Uuid;

pub struct SkillUseCases {
    skill_repo: Arc<dyn SkillRepository>,
    owner_repo: Arc<dyn OwnerRepository>,
}

impl SkillUseCases {
    pub fn new(skill_repo: Arc<dyn SkillRepository>, owner_repo: Arc<dyn OwnerRepository>) -> Self {
        Self {
            skill_repo,
            owner_repo,
        }
    }

    /// Resolve user_id to owner via organization lookup
    async fn resolve_owner(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<crate::domain::entities::Owner, String> {
        self.owner_repo
            .find_by_user_id_and_organization(user_id, organization_id)
            .await?
            .ok_or_else(|| "Owner not found for this user in the organization".to_string())
    }

    /// Create a new skill
    ///
    /// # Authorization
    /// - Owner must exist in the system
    pub async fn create_skill(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        dto: CreateSkillDto,
    ) -> Result<SkillResponseDto, String> {
        // Resolve user_id to owner
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let owner_id = owner.id;

        // Create skill entity (validates business rules)
        let skill = Skill::new(
            owner_id,
            dto.building_id,
            dto.skill_category,
            dto.skill_name,
            dto.expertise_level,
            dto.description,
            dto.is_available_for_help,
            dto.hourly_rate_credits,
            dto.years_of_experience,
            dto.certifications,
        )?;

        // Persist skill
        let created = self.skill_repo.create(&skill).await?;

        // Return enriched response
        let owner_name = format!("{} {}", owner.first_name, owner.last_name);
        Ok(SkillResponseDto::from_skill(created, owner_name))
    }

    /// Get skill by ID with owner name enrichment
    pub async fn get_skill(&self, skill_id: Uuid) -> Result<SkillResponseDto, String> {
        let skill = self
            .skill_repo
            .find_by_id(skill_id)
            .await?
            .ok_or("Skill not found".to_string())?;

        // Enrich with owner name
        let owner = self
            .owner_repo
            .find_by_id(skill.owner_id)
            .await?
            .ok_or("Owner not found".to_string())?;

        let owner_name = format!("{} {}", owner.first_name, owner.last_name);
        Ok(SkillResponseDto::from_skill(skill, owner_name))
    }

    /// List all skills for a building
    ///
    /// # Returns
    /// - Skills sorted by available (DESC), expertise (DESC), skill_name (ASC)
    pub async fn list_building_skills(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SkillSummaryDto>, String> {
        let skills = self.skill_repo.find_by_building(building_id).await?;
        self.enrich_skills_summary(skills).await
    }

    /// List available skills for a building (marketplace view)
    ///
    /// # Returns
    /// - Only available skills (is_available_for_help = true)
    pub async fn list_available_skills(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SkillSummaryDto>, String> {
        let skills = self
            .skill_repo
            .find_available_by_building(building_id)
            .await?;
        self.enrich_skills_summary(skills).await
    }

    /// List all skills created by an owner
    pub async fn list_owner_skills(&self, owner_id: Uuid) -> Result<Vec<SkillSummaryDto>, String> {
        let skills = self.skill_repo.find_by_owner(owner_id).await?;
        self.enrich_skills_summary(skills).await
    }

    /// List skills by category (HomeRepair, Languages, Technology, etc.)
    pub async fn list_skills_by_category(
        &self,
        building_id: Uuid,
        category: SkillCategory,
    ) -> Result<Vec<SkillSummaryDto>, String> {
        let skills = self
            .skill_repo
            .find_by_category(building_id, category)
            .await?;
        self.enrich_skills_summary(skills).await
    }

    /// List skills by expertise level (Beginner, Intermediate, Advanced, Expert)
    pub async fn list_skills_by_expertise(
        &self,
        building_id: Uuid,
        level: ExpertiseLevel,
    ) -> Result<Vec<SkillSummaryDto>, String> {
        let skills = self
            .skill_repo
            .find_by_expertise(building_id, level)
            .await?;
        self.enrich_skills_summary(skills).await
    }

    /// List free/volunteer skills for a building
    pub async fn list_free_skills(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SkillSummaryDto>, String> {
        let skills = self.skill_repo.find_free_by_building(building_id).await?;
        self.enrich_skills_summary(skills).await
    }

    /// List professional skills for a building (Expert level OR certifications)
    pub async fn list_professional_skills(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<SkillSummaryDto>, String> {
        let skills = self
            .skill_repo
            .find_professional_by_building(building_id)
            .await?;
        self.enrich_skills_summary(skills).await
    }

    /// Update a skill
    ///
    /// # Authorization
    /// - Only owner can update their skill
    pub async fn update_skill(
        &self,
        skill_id: Uuid,
        user_id: Uuid,
        organization_id: Uuid,
        dto: UpdateSkillDto,
    ) -> Result<SkillResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut skill = self
            .skill_repo
            .find_by_id(skill_id)
            .await?
            .ok_or("Skill not found".to_string())?;

        // Authorization: only owner can update
        if skill.owner_id != owner.id {
            return Err("Unauthorized: only owner can update skill".to_string());
        }

        // Update skill (domain validates business rules)
        skill.update(
            dto.skill_name,
            dto.expertise_level,
            dto.description,
            dto.is_available_for_help,
            dto.hourly_rate_credits,
            dto.years_of_experience,
            dto.certifications,
        )?;

        // Persist changes
        let updated = self.skill_repo.update(&skill).await?;

        // Return enriched response
        self.get_skill(updated.id).await
    }

    /// Mark skill as available for help
    ///
    /// # Authorization
    /// - Only owner can mark their skill as available
    pub async fn mark_skill_available(
        &self,
        skill_id: Uuid,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<SkillResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut skill = self
            .skill_repo
            .find_by_id(skill_id)
            .await?
            .ok_or("Skill not found".to_string())?;

        // Authorization: only owner can mark available
        if skill.owner_id != owner.id {
            return Err("Unauthorized: only owner can mark skill as available".to_string());
        }

        // Mark available
        skill.mark_available();

        // Persist changes
        let updated = self.skill_repo.update(&skill).await?;

        // Return enriched response
        self.get_skill(updated.id).await
    }

    /// Mark skill as unavailable for help
    ///
    /// # Authorization
    /// - Only owner can mark their skill as unavailable
    pub async fn mark_skill_unavailable(
        &self,
        skill_id: Uuid,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<SkillResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut skill = self
            .skill_repo
            .find_by_id(skill_id)
            .await?
            .ok_or("Skill not found".to_string())?;

        // Authorization: only owner can mark unavailable
        if skill.owner_id != owner.id {
            return Err("Unauthorized: only owner can mark skill as unavailable".to_string());
        }

        // Mark unavailable
        skill.mark_unavailable();

        // Persist changes
        let updated = self.skill_repo.update(&skill).await?;

        // Return enriched response
        self.get_skill(updated.id).await
    }

    /// Delete a skill
    ///
    /// # Authorization
    /// - Only owner can delete their skill
    pub async fn delete_skill(
        &self,
        skill_id: Uuid,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<(), String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let skill = self
            .skill_repo
            .find_by_id(skill_id)
            .await?
            .ok_or("Skill not found".to_string())?;

        // Authorization: only owner can delete
        if skill.owner_id != owner.id {
            return Err("Unauthorized: only owner can delete skill".to_string());
        }

        // Delete skill
        self.skill_repo.delete(skill_id).await?;

        Ok(())
    }

    /// Get skill statistics for a building
    pub async fn get_skill_statistics(
        &self,
        building_id: Uuid,
    ) -> Result<SkillStatisticsDto, String> {
        let total_skills = self.skill_repo.count_by_building(building_id).await?;
        let available_skills = self
            .skill_repo
            .count_available_by_building(building_id)
            .await?;

        // Calculate free/paid skills
        let skills = self.skill_repo.find_by_building(building_id).await?;
        let free_skills = skills.iter().filter(|s| s.is_free()).count() as i64;
        let paid_skills = total_skills - free_skills;
        let professional_skills = skills.iter().filter(|s| s.is_professional()).count() as i64;

        // Count by category
        let mut skills_by_category = Vec::new();
        for category in [
            SkillCategory::HomeRepair,
            SkillCategory::Languages,
            SkillCategory::Technology,
            SkillCategory::Education,
            SkillCategory::Arts,
            SkillCategory::Sports,
            SkillCategory::Cooking,
            SkillCategory::Gardening,
            SkillCategory::Health,
            SkillCategory::Legal,
            SkillCategory::Financial,
            SkillCategory::PetCare,
            SkillCategory::Other,
        ] {
            let count = self
                .skill_repo
                .count_by_category(building_id, category.clone())
                .await?;
            if count > 0 {
                skills_by_category.push(CategoryCount { category, count });
            }
        }

        // Count by expertise level
        let mut skills_by_expertise = Vec::new();
        for level in [
            ExpertiseLevel::Beginner,
            ExpertiseLevel::Intermediate,
            ExpertiseLevel::Advanced,
            ExpertiseLevel::Expert,
        ] {
            let count = self
                .skill_repo
                .count_by_expertise(building_id, level.clone())
                .await?;
            if count > 0 {
                skills_by_expertise.push(ExpertiseCount { level, count });
            }
        }

        Ok(SkillStatisticsDto {
            total_skills,
            available_skills,
            free_skills,
            paid_skills,
            professional_skills,
            skills_by_category,
            skills_by_expertise,
        })
    }

    /// Helper method to enrich skills with owner names
    async fn enrich_skills_summary(
        &self,
        skills: Vec<Skill>,
    ) -> Result<Vec<SkillSummaryDto>, String> {
        let mut enriched = Vec::new();

        for skill in skills {
            // Get owner name
            let owner = self.owner_repo.find_by_id(skill.owner_id).await?;
            let owner_name = if let Some(owner) = owner {
                format!("{} {}", owner.first_name, owner.last_name)
            } else {
                "Unknown Owner".to_string()
            };

            enriched.push(SkillSummaryDto::from_skill(skill, owner_name));
        }

        Ok(enriched)
    }
}
