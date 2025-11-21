use crate::domain::entities::{ExpertiseLevel, Skill, SkillCategory};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DTO for creating a new skill
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSkillDto {
    pub building_id: Uuid,
    pub skill_category: SkillCategory,
    pub skill_name: String,
    pub expertise_level: ExpertiseLevel,
    pub description: String,
    pub is_available_for_help: bool,
    pub hourly_rate_credits: Option<i32>, // 0-100 (SEL integration)
    pub years_of_experience: Option<i32>,
    pub certifications: Option<String>,
}

/// DTO for updating a skill
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSkillDto {
    pub skill_name: Option<String>,
    pub expertise_level: Option<ExpertiseLevel>,
    pub description: Option<String>,
    pub is_available_for_help: Option<bool>,
    pub hourly_rate_credits: Option<Option<i32>>,
    pub years_of_experience: Option<Option<i32>>,
    pub certifications: Option<Option<String>>,
}

/// Complete skill response with owner information
#[derive(Debug, Serialize, Clone)]
pub struct SkillResponseDto {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub owner_name: String, // Enriched from Owner
    pub building_id: Uuid,
    pub skill_category: SkillCategory,
    pub skill_name: String,
    pub expertise_level: ExpertiseLevel,
    pub description: String,
    pub is_available_for_help: bool,
    pub hourly_rate_credits: Option<i32>,
    pub years_of_experience: Option<i32>,
    pub certifications: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // Computed fields
    pub is_free: bool,
    pub is_professional: bool,
}

impl SkillResponseDto {
    /// Create from Skill with owner name enrichment
    pub fn from_skill(skill: Skill, owner_name: String) -> Self {
        let is_free = skill.is_free();
        let is_professional = skill.is_professional();

        Self {
            id: skill.id,
            owner_id: skill.owner_id,
            owner_name,
            building_id: skill.building_id,
            skill_category: skill.skill_category,
            skill_name: skill.skill_name,
            expertise_level: skill.expertise_level,
            description: skill.description,
            is_available_for_help: skill.is_available_for_help,
            hourly_rate_credits: skill.hourly_rate_credits,
            years_of_experience: skill.years_of_experience,
            certifications: skill.certifications,
            created_at: skill.created_at,
            updated_at: skill.updated_at,
            is_free,
            is_professional,
        }
    }
}

/// Summary skill view for lists
#[derive(Debug, Serialize, Clone)]
pub struct SkillSummaryDto {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub owner_name: String, // Enriched from Owner
    pub building_id: Uuid,
    pub skill_category: SkillCategory,
    pub skill_name: String,
    pub expertise_level: ExpertiseLevel,
    pub is_available_for_help: bool,
    pub hourly_rate_credits: Option<i32>,
    pub is_free: bool,
    pub is_professional: bool,
}

impl SkillSummaryDto {
    /// Create from Skill with owner name enrichment
    pub fn from_skill(skill: Skill, owner_name: String) -> Self {
        let is_free = skill.is_free();
        let is_professional = skill.is_professional();

        Self {
            id: skill.id,
            owner_id: skill.owner_id,
            owner_name,
            building_id: skill.building_id,
            skill_category: skill.skill_category,
            skill_name: skill.skill_name,
            expertise_level: skill.expertise_level,
            is_available_for_help: skill.is_available_for_help,
            hourly_rate_credits: skill.hourly_rate_credits,
            is_free,
            is_professional,
        }
    }
}

/// Statistics for building skills
#[derive(Debug, Serialize)]
pub struct SkillStatisticsDto {
    pub total_skills: i64,
    pub available_skills: i64,
    pub free_skills: i64,
    pub paid_skills: i64,
    pub professional_skills: i64,
    pub skills_by_category: Vec<CategoryCount>,
    pub skills_by_expertise: Vec<ExpertiseCount>,
}

/// Category count for statistics
#[derive(Debug, Serialize)]
pub struct CategoryCount {
    pub category: SkillCategory,
    pub count: i64,
}

/// Expertise level count for statistics
#[derive(Debug, Serialize)]
pub struct ExpertiseCount {
    pub level: ExpertiseLevel,
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_skill_response_dto_from_skill() {
        let skill = Skill::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            SkillCategory::Technology,
            "Web Development".to_string(),
            ExpertiseLevel::Advanced,
            "Full-stack web development".to_string(),
            true,
            Some(10),
            Some(5),
            Some("Certified Developer".to_string()),
        )
        .unwrap();

        let dto = SkillResponseDto::from_skill(skill.clone(), "John Doe".to_string());

        assert_eq!(dto.owner_name, "John Doe");
        assert_eq!(dto.skill_name, "Web Development");
        assert_eq!(dto.expertise_level, ExpertiseLevel::Advanced);
        assert!(!dto.is_free);
        assert!(dto.is_professional);
    }

    #[test]
    fn test_skill_summary_dto_from_skill() {
        let skill = Skill::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            SkillCategory::Gardening,
            "Gardening".to_string(),
            ExpertiseLevel::Beginner,
            "Basic gardening".to_string(),
            true,
            None,
            Some(1),
            None,
        )
        .unwrap();

        let dto = SkillSummaryDto::from_skill(skill.clone(), "Jane Smith".to_string());

        assert_eq!(dto.owner_name, "Jane Smith");
        assert_eq!(dto.skill_name, "Gardening");
        assert!(dto.is_free);
        assert!(!dto.is_professional);
    }
}
