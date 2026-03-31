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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{OwnerFilters, PageRequest};
    use crate::application::ports::{OwnerRepository, SkillRepository};
    use crate::domain::entities::{ExpertiseLevel, Owner, Skill, SkillCategory};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    // ── Mock SkillRepository ────────────────────────────────────────────────
    struct MockSkillRepo {
        skills: Mutex<HashMap<Uuid, Skill>>,
    }

    impl MockSkillRepo {
        fn new() -> Self {
            Self {
                skills: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl SkillRepository for MockSkillRepo {
        async fn create(&self, skill: &Skill) -> Result<Skill, String> {
            let mut map = self.skills.lock().unwrap();
            map.insert(skill.id, skill.clone());
            Ok(skill.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Skill>, String> {
            Ok(self.skills.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Skill>, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_available_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<Skill>, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.building_id == building_id && s.is_available_for_help)
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Skill>, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn find_by_category(
            &self,
            building_id: Uuid,
            category: SkillCategory,
        ) -> Result<Vec<Skill>, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.building_id == building_id && s.skill_category == category)
                .cloned()
                .collect())
        }

        async fn find_by_expertise(
            &self,
            building_id: Uuid,
            level: ExpertiseLevel,
        ) -> Result<Vec<Skill>, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.building_id == building_id && s.expertise_level == level)
                .cloned()
                .collect())
        }

        async fn find_free_by_building(&self, building_id: Uuid) -> Result<Vec<Skill>, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.building_id == building_id && s.is_free())
                .cloned()
                .collect())
        }

        async fn find_professional_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<Skill>, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.building_id == building_id && s.is_professional())
                .cloned()
                .collect())
        }

        async fn update(&self, skill: &Skill) -> Result<Skill, String> {
            let mut map = self.skills.lock().unwrap();
            map.insert(skill.id, skill.clone());
            Ok(skill.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<(), String> {
            self.skills.lock().unwrap().remove(&id);
            Ok(())
        }

        async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.building_id == building_id)
                .count() as i64)
        }

        async fn count_available_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.building_id == building_id && s.is_available_for_help)
                .count() as i64)
        }

        async fn count_by_category(
            &self,
            building_id: Uuid,
            category: SkillCategory,
        ) -> Result<i64, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.building_id == building_id && s.skill_category == category)
                .count() as i64)
        }

        async fn count_by_expertise(
            &self,
            building_id: Uuid,
            level: ExpertiseLevel,
        ) -> Result<i64, String> {
            Ok(self
                .skills
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.building_id == building_id && s.expertise_level == level)
                .count() as i64)
        }
    }

    // ── Mock OwnerRepository ────────────────────────────────────────────────
    struct MockOwnerRepo {
        owners: Mutex<HashMap<Uuid, Owner>>,
    }

    impl MockOwnerRepo {
        fn new() -> Self {
            Self {
                owners: Mutex::new(HashMap::new()),
            }
        }
        fn add_owner(&self, owner: Owner) {
            self.owners.lock().unwrap().insert(owner.id, owner);
        }
    }

    #[async_trait]
    impl OwnerRepository for MockOwnerRepo {
        async fn create(&self, owner: &Owner) -> Result<Owner, String> {
            self.owners.lock().unwrap().insert(owner.id, owner.clone());
            Ok(owner.clone())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, String> {
            Ok(self.owners.lock().unwrap().get(&id).cloned())
        }
        async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Owner>, String> {
            Ok(self
                .owners
                .lock()
                .unwrap()
                .values()
                .find(|o| o.user_id == Some(user_id))
                .cloned())
        }
        async fn find_by_user_id_and_organization(
            &self,
            user_id: Uuid,
            org_id: Uuid,
        ) -> Result<Option<Owner>, String> {
            Ok(self
                .owners
                .lock()
                .unwrap()
                .values()
                .find(|o| o.user_id == Some(user_id) && o.organization_id == org_id)
                .cloned())
        }
        async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, String> {
            Ok(self
                .owners
                .lock()
                .unwrap()
                .values()
                .find(|o| o.email == email)
                .cloned())
        }
        async fn find_all(&self) -> Result<Vec<Owner>, String> {
            Ok(self.owners.lock().unwrap().values().cloned().collect())
        }
        async fn find_all_paginated(
            &self,
            _p: &PageRequest,
            _f: &OwnerFilters,
        ) -> Result<(Vec<Owner>, i64), String> {
            let v: Vec<_> = self.owners.lock().unwrap().values().cloned().collect();
            let c = v.len() as i64;
            Ok((v, c))
        }
        async fn update(&self, owner: &Owner) -> Result<Owner, String> {
            self.owners.lock().unwrap().insert(owner.id, owner.clone());
            Ok(owner.clone())
        }
        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.owners.lock().unwrap().remove(&id).is_some())
        }
        async fn set_user_link(
            &self,
            owner_id: Uuid,
            user_id: Option<Uuid>,
        ) -> Result<bool, String> {
            let mut map = self.owners.lock().unwrap();
            if let Some(o) = map.get_mut(&owner_id) {
                o.user_id = user_id;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    // ── Helpers ─────────────────────────────────────────────────────────────
    fn create_test_owner(user_id: Uuid, org_id: Uuid) -> Owner {
        let mut owner = Owner::new(
            org_id,
            "Marie".to_string(),
            "Lefevre".to_string(),
            "marie@test.com".to_string(),
            None,
            "Rue Haute 5".to_string(),
            "Brussels".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
        )
        .unwrap();
        owner.user_id = Some(user_id);
        owner
    }

    fn setup() -> (SkillUseCases, Uuid, Uuid, Uuid) {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let skill_repo = Arc::new(MockSkillRepo::new());
        let owner_repo = Arc::new(MockOwnerRepo::new());

        let owner = create_test_owner(user_id, org_id);
        owner_repo.add_owner(owner);

        let uc = SkillUseCases::new(
            skill_repo as Arc<dyn SkillRepository>,
            owner_repo as Arc<dyn OwnerRepository>,
        );

        (uc, user_id, org_id, building_id)
    }

    fn make_create_dto(building_id: Uuid) -> CreateSkillDto {
        CreateSkillDto {
            building_id,
            skill_category: SkillCategory::Technology,
            skill_name: "Web Development".to_string(),
            expertise_level: ExpertiseLevel::Advanced,
            description: "Full-stack web development with React and Node".to_string(),
            is_available_for_help: true,
            hourly_rate_credits: Some(10),
            years_of_experience: Some(5),
            certifications: Some("AWS Certified".to_string()),
        }
    }

    // ── Tests ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_skill_success() {
        let (uc, user_id, org_id, building_id) = setup();
        let dto = make_create_dto(building_id);
        let result = uc.create_skill(user_id, org_id, dto).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.skill_name, "Web Development");
        assert_eq!(resp.owner_name, "Marie Lefevre");
    }

    #[tokio::test]
    async fn test_get_skill_success() {
        let (uc, user_id, org_id, building_id) = setup();
        let dto = make_create_dto(building_id);
        let created = uc.create_skill(user_id, org_id, dto).await.unwrap();

        let result = uc.get_skill(created.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_get_skill_not_found() {
        let (uc, _, _, _) = setup();
        let result = uc.get_skill(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Skill not found");
    }

    #[tokio::test]
    async fn test_delete_skill_success() {
        let (uc, user_id, org_id, building_id) = setup();
        let dto = make_create_dto(building_id);
        let created = uc.create_skill(user_id, org_id, dto).await.unwrap();

        let result = uc.delete_skill(created.id, user_id, org_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_skill_wrong_owner() {
        let (uc, user_id, org_id, building_id) = setup();
        let dto = make_create_dto(building_id);
        let created = uc.create_skill(user_id, org_id, dto).await.unwrap();

        // Unknown user
        let other = Uuid::new_v4();
        let result = uc.delete_skill(created.id, other, org_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Owner not found"));
    }

    #[tokio::test]
    async fn test_list_building_skills() {
        let (uc, user_id, org_id, building_id) = setup();

        let dto1 = make_create_dto(building_id);
        let mut dto2 = make_create_dto(building_id);
        dto2.skill_name = "Plumbing".to_string();
        dto2.skill_category = SkillCategory::HomeRepair;
        dto2.description = "Residential plumbing repair and installation".to_string();

        uc.create_skill(user_id, org_id, dto1).await.unwrap();
        uc.create_skill(user_id, org_id, dto2).await.unwrap();

        let result = uc.list_building_skills(building_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_owner_not_found() {
        let skill_repo = Arc::new(MockSkillRepo::new());
        let owner_repo = Arc::new(MockOwnerRepo::new());
        // No owner added

        let uc = SkillUseCases::new(
            skill_repo as Arc<dyn SkillRepository>,
            owner_repo as Arc<dyn OwnerRepository>,
        );

        let dto = make_create_dto(Uuid::new_v4());
        let result = uc.create_skill(Uuid::new_v4(), Uuid::new_v4(), dto).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Owner not found"));
    }
}
