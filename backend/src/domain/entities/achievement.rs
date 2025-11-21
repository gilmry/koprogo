use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Achievement category for organizational purposes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AchievementCategory {
    Community,     // Community participation achievements
    Sel,           // SEL (Local Exchange) achievements
    Booking,       // Resource booking achievements
    Sharing,       // Object sharing achievements
    Skills,        // Skills directory achievements
    Notice,        // Notice board achievements
    Governance,    // Meeting/voting participation achievements
    Milestone,     // Platform usage milestones
}

/// Achievement tier for progression
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AchievementTier {
    Bronze,   // Entry-level achievements
    Silver,   // Intermediate achievements
    Gold,     // Advanced achievements
    Platinum, // Expert-level achievements
    Diamond,  // Exceptional achievements
}

/// Achievement entity - Defines badges/achievements users can earn
///
/// Represents a specific accomplishment or milestone in the platform.
/// Achievements encourage community participation and engagement.
///
/// # Belgian Context
/// - Promotes active participation in copropriété management
/// - Encourages use of community features (SEL, notice board, bookings)
/// - Recognizes contributions to building community
///
/// # Business Rules
/// - name must be 3-100 characters
/// - description must be 10-500 characters
/// - icon must be valid emoji or URL
/// - points_value must be 0-1000
/// - requirements stored as JSON for flexibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub category: AchievementCategory,
    pub tier: AchievementTier,
    pub name: String,                  // e.g., "SEL Pioneer", "First Booking", "Community Helper"
    pub description: String,           // What this achievement represents
    pub icon: String,                  // Emoji or icon URL
    pub points_value: i32,             // Points awarded when earned (0-1000)
    pub requirements: String,          // JSON criteria (e.g., {"action": "sel_exchange", "count": 10})
    pub is_secret: bool,               // Hidden until earned
    pub is_repeatable: bool,           // Can be earned multiple times
    pub display_order: i32,            // For sorting in UI
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Achievement {
    /// Minimum name length
    pub const MIN_NAME_LENGTH: usize = 3;
    /// Maximum name length
    pub const MAX_NAME_LENGTH: usize = 100;
    /// Minimum description length
    pub const MIN_DESCRIPTION_LENGTH: usize = 10;
    /// Maximum description length
    pub const MAX_DESCRIPTION_LENGTH: usize = 500;
    /// Maximum points value
    pub const MAX_POINTS_VALUE: i32 = 1000;

    /// Create a new achievement
    ///
    /// # Validation
    /// - name must be 3-100 characters
    /// - description must be 10-500 characters
    /// - icon must not be empty
    /// - points_value must be 0-1000
    /// - requirements must not be empty
    pub fn new(
        organization_id: Uuid,
        category: AchievementCategory,
        tier: AchievementTier,
        name: String,
        description: String,
        icon: String,
        points_value: i32,
        requirements: String,
        is_secret: bool,
        is_repeatable: bool,
        display_order: i32,
    ) -> Result<Self, String> {
        // Validate name
        if name.len() < Self::MIN_NAME_LENGTH || name.len() > Self::MAX_NAME_LENGTH {
            return Err(format!(
                "Achievement name must be {}-{} characters",
                Self::MIN_NAME_LENGTH,
                Self::MAX_NAME_LENGTH
            ));
        }

        // Validate description
        if description.len() < Self::MIN_DESCRIPTION_LENGTH
            || description.len() > Self::MAX_DESCRIPTION_LENGTH
        {
            return Err(format!(
                "Achievement description must be {}-{} characters",
                Self::MIN_DESCRIPTION_LENGTH,
                Self::MAX_DESCRIPTION_LENGTH
            ));
        }

        // Validate icon
        if icon.trim().is_empty() {
            return Err("Achievement icon cannot be empty".to_string());
        }

        // Validate points
        if points_value < 0 || points_value > Self::MAX_POINTS_VALUE {
            return Err(format!(
                "Points value must be 0-{} points",
                Self::MAX_POINTS_VALUE
            ));
        }

        // Validate requirements
        if requirements.trim().is_empty() {
            return Err("Achievement requirements cannot be empty".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            category,
            tier,
            name,
            description,
            icon,
            points_value,
            requirements,
            is_secret,
            is_repeatable,
            display_order,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update achievement details
    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        icon: Option<String>,
        points_value: Option<i32>,
        requirements: Option<String>,
        is_secret: Option<bool>,
        is_repeatable: Option<bool>,
        display_order: Option<i32>,
    ) -> Result<(), String> {
        // Update name if provided
        if let Some(n) = name {
            if n.len() < Self::MIN_NAME_LENGTH || n.len() > Self::MAX_NAME_LENGTH {
                return Err(format!(
                    "Achievement name must be {}-{} characters",
                    Self::MIN_NAME_LENGTH,
                    Self::MAX_NAME_LENGTH
                ));
            }
            self.name = n;
        }

        // Update description if provided
        if let Some(d) = description {
            if d.len() < Self::MIN_DESCRIPTION_LENGTH || d.len() > Self::MAX_DESCRIPTION_LENGTH {
                return Err(format!(
                    "Achievement description must be {}-{} characters",
                    Self::MIN_DESCRIPTION_LENGTH,
                    Self::MAX_DESCRIPTION_LENGTH
                ));
            }
            self.description = d;
        }

        // Update icon if provided
        if let Some(i) = icon {
            if i.trim().is_empty() {
                return Err("Achievement icon cannot be empty".to_string());
            }
            self.icon = i;
        }

        // Update points if provided
        if let Some(p) = points_value {
            if p < 0 || p > Self::MAX_POINTS_VALUE {
                return Err(format!(
                    "Points value must be 0-{} points",
                    Self::MAX_POINTS_VALUE
                ));
            }
            self.points_value = p;
        }

        // Update requirements if provided
        if let Some(r) = requirements {
            if r.trim().is_empty() {
                return Err("Achievement requirements cannot be empty".to_string());
            }
            self.requirements = r;
        }

        // Update flags
        if let Some(s) = is_secret {
            self.is_secret = s;
        }
        if let Some(r) = is_repeatable {
            self.is_repeatable = r;
        }
        if let Some(o) = display_order {
            self.display_order = o;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Calculate points for tier (helper for auto-calculation)
    pub fn default_points_for_tier(tier: &AchievementTier) -> i32 {
        match tier {
            AchievementTier::Bronze => 10,
            AchievementTier::Silver => 25,
            AchievementTier::Gold => 50,
            AchievementTier::Platinum => 100,
            AchievementTier::Diamond => 250,
        }
    }

    /// Update achievement name
    pub fn update_name(&mut self, name: String) -> Result<(), String> {
        self.update(Some(name), None, None, None, None, None, None, None)
    }

    /// Update achievement description
    pub fn update_description(&mut self, description: String) -> Result<(), String> {
        self.update(None, Some(description), None, None, None, None, None, None)
    }

    /// Update achievement icon
    pub fn update_icon(&mut self, icon: String) -> Result<(), String> {
        self.update(None, None, Some(icon), None, None, None, None, None)
    }

    /// Update achievement points value
    pub fn update_points_value(&mut self, points_value: i32) -> Result<(), String> {
        self.update(None, None, None, Some(points_value), None, None, None, None)
    }

    /// Update achievement requirements
    pub fn update_requirements(&mut self, requirements: String) -> Result<(), String> {
        self.update(None, None, None, None, Some(requirements), None, None, None)
    }
}

/// User achievement record - Tracks which achievements a user has earned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAchievement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub achievement_id: Uuid,
    pub earned_at: DateTime<Utc>,
    pub progress_data: Option<String>, // JSON for tracking progress towards repeatable achievements
    pub times_earned: i32,              // For repeatable achievements
}

impl UserAchievement {
    /// Award achievement to user
    pub fn new(
        user_id: Uuid,
        achievement_id: Uuid,
        progress_data: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            achievement_id,
            earned_at: Utc::now(),
            progress_data,
            times_earned: 1,
        }
    }

    /// Increment times earned (for repeatable achievements)
    pub fn increment_earned(&mut self) {
        self.times_earned += 1;
    }

    /// Repeat earn achievement (alias for increment_earned)
    pub fn repeat_earn(&mut self) -> Result<(), String> {
        self.increment_earned();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_achievement() -> Achievement {
        let organization_id = Uuid::new_v4();
        Achievement::new(
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
        .unwrap()
    }

    #[test]
    fn test_create_achievement_success() {
        let achievement = create_test_achievement();
        assert_eq!(achievement.name, "First Booking");
        assert_eq!(achievement.category, AchievementCategory::Community);
        assert_eq!(achievement.tier, AchievementTier::Bronze);
        assert_eq!(achievement.points_value, 10);
        assert!(!achievement.is_secret);
        assert!(!achievement.is_repeatable);
    }

    #[test]
    fn test_create_achievement_invalid_name() {
        let organization_id = Uuid::new_v4();
        let result = Achievement::new(
            organization_id,
            AchievementCategory::Community,
            AchievementTier::Bronze,
            "AB".to_string(), // Too short
            "Made your first resource booking".to_string(),
            "🎉".to_string(),
            10,
            r#"{"action": "booking_created", "count": 1}"#.to_string(),
            false,
            false,
            1,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Achievement name must be"));
    }

    #[test]
    fn test_create_achievement_invalid_description() {
        let organization_id = Uuid::new_v4();
        let result = Achievement::new(
            organization_id,
            AchievementCategory::Community,
            AchievementTier::Bronze,
            "First Booking".to_string(),
            "Short".to_string(), // Too short
            "🎉".to_string(),
            10,
            r#"{"action": "booking_created", "count": 1}"#.to_string(),
            false,
            false,
            1,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Achievement description must be"));
    }

    #[test]
    fn test_create_achievement_invalid_icon() {
        let organization_id = Uuid::new_v4();
        let result = Achievement::new(
            organization_id,
            AchievementCategory::Community,
            AchievementTier::Bronze,
            "First Booking".to_string(),
            "Made your first resource booking".to_string(),
            "".to_string(), // Empty icon
            10,
            r#"{"action": "booking_created", "count": 1}"#.to_string(),
            false,
            false,
            1,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Achievement icon cannot be empty"));
    }

    #[test]
    fn test_create_achievement_invalid_points() {
        let organization_id = Uuid::new_v4();
        let result = Achievement::new(
            organization_id,
            AchievementCategory::Community,
            AchievementTier::Bronze,
            "First Booking".to_string(),
            "Made your first resource booking".to_string(),
            "🎉".to_string(),
            2000, // Exceeds max
            r#"{"action": "booking_created", "count": 1}"#.to_string(),
            false,
            false,
            1,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Points value must be"));
    }

    #[test]
    fn test_create_achievement_invalid_requirements() {
        let organization_id = Uuid::new_v4();
        let result = Achievement::new(
            organization_id,
            AchievementCategory::Community,
            AchievementTier::Bronze,
            "First Booking".to_string(),
            "Made your first resource booking".to_string(),
            "🎉".to_string(),
            10,
            "".to_string(), // Empty requirements
            false,
            false,
            1,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Achievement requirements cannot be empty"));
    }

    #[test]
    fn test_update_achievement_success() {
        let mut achievement = create_test_achievement();
        let result = achievement.update(
            Some("Updated Name".to_string()),
            Some("Updated description for this achievement".to_string()),
            Some("🏆".to_string()),
            Some(25),
            None,
            None,
            None,
            Some(10),
        );

        assert!(result.is_ok());
        assert_eq!(achievement.name, "Updated Name");
        assert_eq!(achievement.icon, "🏆");
        assert_eq!(achievement.points_value, 25);
        assert_eq!(achievement.display_order, 10);
    }

    #[test]
    fn test_update_achievement_invalid_name() {
        let mut achievement = create_test_achievement();
        let result = achievement.update(
            Some("AB".to_string()), // Too short
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Achievement name must be"));
    }

    #[test]
    fn test_default_points_for_tier() {
        assert_eq!(Achievement::default_points_for_tier(&AchievementTier::Bronze), 10);
        assert_eq!(Achievement::default_points_for_tier(&AchievementTier::Silver), 25);
        assert_eq!(Achievement::default_points_for_tier(&AchievementTier::Gold), 50);
        assert_eq!(Achievement::default_points_for_tier(&AchievementTier::Platinum), 100);
        assert_eq!(Achievement::default_points_for_tier(&AchievementTier::Diamond), 250);
    }

    #[test]
    fn test_user_achievement_new() {
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();
        let user_achievement = UserAchievement::new(user_id, achievement_id, None);

        assert_eq!(user_achievement.user_id, user_id);
        assert_eq!(user_achievement.achievement_id, achievement_id);
        assert_eq!(user_achievement.times_earned, 1);
        assert!(user_achievement.progress_data.is_none());
    }

    #[test]
    fn test_user_achievement_increment() {
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();
        let mut user_achievement = UserAchievement::new(user_id, achievement_id, None);

        user_achievement.increment_earned();
        assert_eq!(user_achievement.times_earned, 2);

        user_achievement.increment_earned();
        assert_eq!(user_achievement.times_earned, 3);
    }

    #[test]
    fn test_achievement_tier_ordering() {
        assert!(AchievementTier::Bronze < AchievementTier::Silver);
        assert!(AchievementTier::Silver < AchievementTier::Gold);
        assert!(AchievementTier::Gold < AchievementTier::Platinum);
        assert!(AchievementTier::Platinum < AchievementTier::Diamond);
    }
}
