use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Skill category for classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillCategory {
    /// Home repair and maintenance (plumbing, electrical, carpentry, etc.)
    HomeRepair,
    /// Languages (teaching, translation, conversation practice)
    Languages,
    /// Technology (IT support, web development, software, hardware)
    Technology,
    /// Education and tutoring (math, science, music lessons, etc.)
    Education,
    /// Arts and crafts (painting, sewing, woodworking, etc.)
    Arts,
    /// Sports and fitness (personal training, yoga, martial arts, etc.)
    Sports,
    /// Cooking and baking
    Cooking,
    /// Gardening and landscaping
    Gardening,
    /// Health and wellness (massage, physiotherapy, counseling, etc.)
    Health,
    /// Legal and administrative (tax preparation, document assistance, etc.)
    Legal,
    /// Financial (accounting, budgeting advice, etc.)
    Financial,
    /// Pet care and training
    PetCare,
    /// Other skills
    Other,
}

/// Expertise level for skill proficiency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExpertiseLevel {
    /// Beginner (< 1 year experience)
    Beginner,
    /// Intermediate (1-3 years experience)
    Intermediate,
    /// Advanced (3-10 years experience)
    Advanced,
    /// Expert (10+ years experience or professional certification)
    Expert,
}

/// Skill profile for community members
///
/// Represents a skill that a building resident can offer to help other members.
/// Integrates with SEL (Local Exchange Trading System) for optional credit-based compensation.
///
/// # Business Rules
/// - skill_name must be 3-100 characters
/// - description max 1000 characters
/// - hourly_rate_credits: 0-100 (0 = free/volunteer, compatible with SEL system)
/// - years_of_experience: 0-50
/// - Only owner can update/delete their own skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub building_id: Uuid,
    pub skill_category: SkillCategory,
    pub skill_name: String,
    pub expertise_level: ExpertiseLevel,
    pub description: String,
    pub is_available_for_help: bool,
    /// Hourly rate in SEL credits (0 = free/volunteer, None = not specified)
    pub hourly_rate_credits: Option<i32>,
    /// Years of experience (optional)
    pub years_of_experience: Option<i32>,
    /// Professional certifications or qualifications (optional)
    pub certifications: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Skill {
    /// Create a new skill
    ///
    /// # Validation
    /// - skill_name: 3-100 characters
    /// - description: max 1000 characters
    /// - hourly_rate_credits: 0-100 if provided
    /// - years_of_experience: 0-50 if provided
    pub fn new(
        owner_id: Uuid,
        building_id: Uuid,
        skill_category: SkillCategory,
        skill_name: String,
        expertise_level: ExpertiseLevel,
        description: String,
        is_available_for_help: bool,
        hourly_rate_credits: Option<i32>,
        years_of_experience: Option<i32>,
        certifications: Option<String>,
    ) -> Result<Self, String> {
        // Validate skill_name
        if skill_name.len() < 3 {
            return Err("Skill name must be at least 3 characters".to_string());
        }
        if skill_name.len() > 100 {
            return Err("Skill name cannot exceed 100 characters".to_string());
        }

        // Validate description
        if description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        if description.len() > 1000 {
            return Err("Description cannot exceed 1000 characters".to_string());
        }

        // Validate hourly_rate_credits (compatible with SEL: 0-100 credits)
        if let Some(rate) = hourly_rate_credits {
            if rate < 0 {
                return Err("Hourly rate cannot be negative".to_string());
            }
            if rate > 100 {
                return Err("Hourly rate cannot exceed 100 credits".to_string());
            }
        }

        // Validate years_of_experience
        if let Some(years) = years_of_experience {
            if years < 0 {
                return Err("Years of experience cannot be negative".to_string());
            }
            if years > 50 {
                return Err("Years of experience cannot exceed 50".to_string());
            }
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            owner_id,
            building_id,
            skill_category,
            skill_name,
            expertise_level,
            description,
            is_available_for_help,
            hourly_rate_credits,
            years_of_experience,
            certifications,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update skill information
    ///
    /// # Validation
    /// - Same validation rules as new()
    pub fn update(
        &mut self,
        skill_name: Option<String>,
        expertise_level: Option<ExpertiseLevel>,
        description: Option<String>,
        is_available_for_help: Option<bool>,
        hourly_rate_credits: Option<Option<i32>>,
        years_of_experience: Option<Option<i32>>,
        certifications: Option<Option<String>>,
    ) -> Result<(), String> {
        // Update skill_name if provided
        if let Some(name) = skill_name {
            if name.len() < 3 {
                return Err("Skill name must be at least 3 characters".to_string());
            }
            if name.len() > 100 {
                return Err("Skill name cannot exceed 100 characters".to_string());
            }
            self.skill_name = name;
        }

        // Update expertise_level if provided
        if let Some(level) = expertise_level {
            self.expertise_level = level;
        }

        // Update description if provided
        if let Some(desc) = description {
            if desc.trim().is_empty() {
                return Err("Description cannot be empty".to_string());
            }
            if desc.len() > 1000 {
                return Err("Description cannot exceed 1000 characters".to_string());
            }
            self.description = desc;
        }

        // Update availability if provided
        if let Some(available) = is_available_for_help {
            self.is_available_for_help = available;
        }

        // Update hourly_rate_credits if provided
        if let Some(rate_opt) = hourly_rate_credits {
            if let Some(rate) = rate_opt {
                if rate < 0 {
                    return Err("Hourly rate cannot be negative".to_string());
                }
                if rate > 100 {
                    return Err("Hourly rate cannot exceed 100 credits".to_string());
                }
            }
            self.hourly_rate_credits = rate_opt;
        }

        // Update years_of_experience if provided
        if let Some(years_opt) = years_of_experience {
            if let Some(years) = years_opt {
                if years < 0 {
                    return Err("Years of experience cannot be negative".to_string());
                }
                if years > 50 {
                    return Err("Years of experience cannot exceed 50".to_string());
                }
            }
            self.years_of_experience = years_opt;
        }

        // Update certifications if provided
        if let Some(cert_opt) = certifications {
            self.certifications = cert_opt;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark skill as available for help
    pub fn mark_available(&mut self) {
        self.is_available_for_help = true;
        self.updated_at = Utc::now();
    }

    /// Mark skill as unavailable for help
    pub fn mark_unavailable(&mut self) {
        self.is_available_for_help = false;
        self.updated_at = Utc::now();
    }

    /// Check if skill is free (volunteer)
    pub fn is_free(&self) -> bool {
        self.hourly_rate_credits.is_none() || self.hourly_rate_credits == Some(0)
    }

    /// Check if skill is professional (has certifications or Expert level)
    pub fn is_professional(&self) -> bool {
        self.expertise_level == ExpertiseLevel::Expert || self.certifications.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_skill_success() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let skill = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Technology,
            "Web Development".to_string(),
            ExpertiseLevel::Advanced,
            "Full-stack web development with React and Node.js".to_string(),
            true,
            Some(10), // 10 credits/hour
            Some(5),  // 5 years experience
            Some("Certified Web Developer".to_string()),
        );

        assert!(skill.is_ok());
        let skill = skill.unwrap();
        assert_eq!(skill.owner_id, owner_id);
        assert_eq!(skill.building_id, building_id);
        assert_eq!(skill.skill_category, SkillCategory::Technology);
        assert!(skill.is_available_for_help);
        assert_eq!(skill.hourly_rate_credits, Some(10));
    }

    #[test]
    fn test_skill_name_too_short_fails() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = Skill::new(
            owner_id,
            building_id,
            SkillCategory::HomeRepair,
            "IT".to_string(), // Too short (< 3 chars)
            ExpertiseLevel::Beginner,
            "Basic IT support".to_string(),
            true,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Skill name must be at least 3 characters"
        );
    }

    #[test]
    fn test_skill_name_too_long_fails() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let long_name = "A".repeat(101); // 101 chars (> 100)

        let result = Skill::new(
            owner_id,
            building_id,
            SkillCategory::HomeRepair,
            long_name,
            ExpertiseLevel::Beginner,
            "Description".to_string(),
            true,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Skill name cannot exceed 100 characters"
        );
    }

    #[test]
    fn test_empty_description_fails() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Cooking,
            "Baking".to_string(),
            ExpertiseLevel::Intermediate,
            "   ".to_string(), // Empty/whitespace
            true,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Description cannot be empty");
    }

    #[test]
    fn test_hourly_rate_negative_fails() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Languages,
            "English".to_string(),
            ExpertiseLevel::Expert,
            "English conversation and grammar".to_string(),
            true,
            Some(-5), // Negative rate
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Hourly rate cannot be negative");
    }

    #[test]
    fn test_hourly_rate_exceeds_100_fails() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Legal,
            "Tax Consulting".to_string(),
            ExpertiseLevel::Expert,
            "Professional tax preparation".to_string(),
            true,
            Some(150), // Exceeds 100 credits
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Hourly rate cannot exceed 100 credits");
    }

    #[test]
    fn test_years_of_experience_negative_fails() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Sports,
            "Yoga".to_string(),
            ExpertiseLevel::Beginner,
            "Hatha yoga for beginners".to_string(),
            true,
            None,
            Some(-2), // Negative years
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Years of experience cannot be negative"
        );
    }

    #[test]
    fn test_update_skill_success() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut skill = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Gardening,
            "Gardening".to_string(),
            ExpertiseLevel::Beginner,
            "Basic gardening and plant care".to_string(),
            true,
            None,
            Some(1),
            None,
        )
        .unwrap();

        let result = skill.update(
            Some("Advanced Gardening".to_string()),
            Some(ExpertiseLevel::Intermediate),
            Some("Organic gardening and permaculture design".to_string()),
            Some(true),
            Some(Some(5)), // 5 credits/hour
            Some(Some(3)), // 3 years experience
            Some(Some("Permaculture Design Certificate".to_string())),
        );

        assert!(result.is_ok());
        assert_eq!(skill.skill_name, "Advanced Gardening");
        assert_eq!(skill.expertise_level, ExpertiseLevel::Intermediate);
        assert_eq!(skill.hourly_rate_credits, Some(5));
        assert_eq!(skill.years_of_experience, Some(3));
    }

    #[test]
    fn test_mark_available() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut skill = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Arts,
            "Painting".to_string(),
            ExpertiseLevel::Advanced,
            "Oil and watercolor painting".to_string(),
            false, // Initially unavailable
            None,
            None,
            None,
        )
        .unwrap();

        assert!(!skill.is_available_for_help);

        skill.mark_available();
        assert!(skill.is_available_for_help);
    }

    #[test]
    fn test_mark_unavailable() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut skill = Skill::new(
            owner_id,
            building_id,
            SkillCategory::PetCare,
            "Dog Training".to_string(),
            ExpertiseLevel::Expert,
            "Professional dog training and behavior modification".to_string(),
            true, // Initially available
            Some(15),
            Some(10),
            Some("Certified Dog Trainer".to_string()),
        )
        .unwrap();

        assert!(skill.is_available_for_help);

        skill.mark_unavailable();
        assert!(!skill.is_available_for_help);
    }

    #[test]
    fn test_is_free() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Free skill (None)
        let skill1 = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Education,
            "Math Tutoring".to_string(),
            ExpertiseLevel::Advanced,
            "High school math tutoring".to_string(),
            true,
            None, // Free/volunteer
            None,
            None,
        )
        .unwrap();
        assert!(skill1.is_free());

        // Free skill (0 credits)
        let skill2 = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Education,
            "Math Tutoring".to_string(),
            ExpertiseLevel::Advanced,
            "High school math tutoring".to_string(),
            true,
            Some(0), // Explicitly 0 credits
            None,
            None,
        )
        .unwrap();
        assert!(skill2.is_free());

        // Paid skill
        let skill3 = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Technology,
            "IT Support".to_string(),
            ExpertiseLevel::Expert,
            "Professional IT support".to_string(),
            true,
            Some(20), // 20 credits/hour
            None,
            None,
        )
        .unwrap();
        assert!(!skill3.is_free());
    }

    #[test]
    fn test_is_professional() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Professional (Expert level)
        let skill1 = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Health,
            "Massage Therapy".to_string(),
            ExpertiseLevel::Expert,
            "Therapeutic massage".to_string(),
            true,
            Some(30),
            Some(15),
            None,
        )
        .unwrap();
        assert!(skill1.is_professional());

        // Professional (has certifications)
        let skill2 = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Financial,
            "Accounting".to_string(),
            ExpertiseLevel::Advanced,
            "Small business accounting".to_string(),
            true,
            Some(25),
            Some(5),
            Some("CPA License".to_string()),
        )
        .unwrap();
        assert!(skill2.is_professional());

        // Not professional (Beginner, no certifications)
        let skill3 = Skill::new(
            owner_id,
            building_id,
            SkillCategory::Cooking,
            "Baking".to_string(),
            ExpertiseLevel::Beginner,
            "Home baking enthusiast".to_string(),
            true,
            None,
            Some(1),
            None,
        )
        .unwrap();
        assert!(!skill3.is_professional());
    }
}
