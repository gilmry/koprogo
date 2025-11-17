use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Owner Credit Balance for Local Exchange Trading System (SEL)
///
/// Tracks time-based currency balance for each owner per building.
/// Credits are earned by providing services and spent by receiving services.
///
/// Balance = credits_earned - credits_spent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OwnerCreditBalance {
    pub owner_id: Uuid,
    pub building_id: Uuid,
    pub credits_earned: i32,   // Services provided (positive)
    pub credits_spent: i32,    // Services received (positive)
    pub balance: i32,          // earned - spent (can be negative)
    pub total_exchanges: i32,  // Total number of completed exchanges
    pub average_rating: Option<f32>, // Average rating received (1-5 stars)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OwnerCreditBalance {
    /// Create a new credit balance (starts at 0)
    pub fn new(owner_id: Uuid, building_id: Uuid) -> Self {
        let now = Utc::now();

        OwnerCreditBalance {
            owner_id,
            building_id,
            credits_earned: 0,
            credits_spent: 0,
            balance: 0,
            total_exchanges: 0,
            average_rating: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Earn credits (when providing a service)
    pub fn earn_credits(&mut self, amount: i32) -> Result<(), String> {
        if amount <= 0 {
            return Err("Credits to earn must be positive".to_string());
        }

        self.credits_earned += amount;
        self.balance += amount;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Spend credits (when receiving a service)
    pub fn spend_credits(&mut self, amount: i32) -> Result<(), String> {
        if amount <= 0 {
            return Err("Credits to spend must be positive".to_string());
        }

        // Note: We allow negative balance (community trust model)
        // Some SEL systems don't allow negative balance, but we do for flexibility
        self.credits_spent += amount;
        self.balance -= amount;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Increment exchange counter
    pub fn increment_exchanges(&mut self) {
        self.total_exchanges += 1;
        self.updated_at = Utc::now();
    }

    /// Update average rating
    pub fn update_rating(&mut self, new_rating: f32) -> Result<(), String> {
        if !(1.0..=5.0).contains(&new_rating) {
            return Err("Rating must be between 1.0 and 5.0".to_string());
        }

        self.average_rating = Some(new_rating);
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Check if owner has sufficient credits (for systems that enforce limits)
    pub fn has_sufficient_credits(&self, required: i32) -> bool {
        self.balance >= required
    }

    /// Get credit status
    pub fn credit_status(&self) -> CreditStatus {
        if self.balance > 0 {
            CreditStatus::Positive
        } else if self.balance < 0 {
            CreditStatus::Negative
        } else {
            CreditStatus::Balanced
        }
    }

    /// Check if owner is a new member (no exchanges yet)
    pub fn is_new_member(&self) -> bool {
        self.total_exchanges == 0
    }

    /// Get participation level
    pub fn participation_level(&self) -> ParticipationLevel {
        match self.total_exchanges {
            0 => ParticipationLevel::New,
            1..=5 => ParticipationLevel::Beginner,
            6..=20 => ParticipationLevel::Active,
            21..=50 => ParticipationLevel::Veteran,
            _ => ParticipationLevel::Expert,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CreditStatus {
    Positive,  // Balance > 0 (net provider)
    Balanced,  // Balance = 0
    Negative,  // Balance < 0 (net receiver)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParticipationLevel {
    New,       // 0 exchanges
    Beginner,  // 1-5 exchanges
    Active,    // 6-20 exchanges
    Veteran,   // 21-50 exchanges
    Expert,    // 51+ exchanges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_credit_balance() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let balance = OwnerCreditBalance::new(owner_id, building_id);

        assert_eq!(balance.owner_id, owner_id);
        assert_eq!(balance.building_id, building_id);
        assert_eq!(balance.credits_earned, 0);
        assert_eq!(balance.credits_spent, 0);
        assert_eq!(balance.balance, 0);
        assert_eq!(balance.total_exchanges, 0);
        assert!(balance.average_rating.is_none());
        assert_eq!(balance.credit_status(), CreditStatus::Balanced);
        assert!(balance.is_new_member());
    }

    #[test]
    fn test_earn_credits() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut balance = OwnerCreditBalance::new(owner_id, building_id);

        // Earn 5 credits
        assert!(balance.earn_credits(5).is_ok());
        assert_eq!(balance.credits_earned, 5);
        assert_eq!(balance.balance, 5);
        assert_eq!(balance.credit_status(), CreditStatus::Positive);

        // Earn 3 more credits
        assert!(balance.earn_credits(3).is_ok());
        assert_eq!(balance.credits_earned, 8);
        assert_eq!(balance.balance, 8);
    }

    #[test]
    fn test_spend_credits() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut balance = OwnerCreditBalance::new(owner_id, building_id);

        // Earn 10 credits first
        balance.earn_credits(10).unwrap();

        // Spend 6 credits
        assert!(balance.spend_credits(6).is_ok());
        assert_eq!(balance.credits_spent, 6);
        assert_eq!(balance.balance, 4);
        assert_eq!(balance.credit_status(), CreditStatus::Positive);

        // Spend 5 more credits (goes negative)
        assert!(balance.spend_credits(5).is_ok());
        assert_eq!(balance.credits_spent, 11);
        assert_eq!(balance.balance, -1);
        assert_eq!(balance.credit_status(), CreditStatus::Negative);
    }

    #[test]
    fn test_earn_credits_validation() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut balance = OwnerCreditBalance::new(owner_id, building_id);

        // Cannot earn 0 credits
        assert!(balance.earn_credits(0).is_err());

        // Cannot earn negative credits
        assert!(balance.earn_credits(-5).is_err());
    }

    #[test]
    fn test_spend_credits_validation() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut balance = OwnerCreditBalance::new(owner_id, building_id);

        // Cannot spend 0 credits
        assert!(balance.spend_credits(0).is_err());

        // Cannot spend negative credits
        assert!(balance.spend_credits(-5).is_err());
    }

    #[test]
    fn test_increment_exchanges() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut balance = OwnerCreditBalance::new(owner_id, building_id);

        balance.increment_exchanges();
        assert_eq!(balance.total_exchanges, 1);

        balance.increment_exchanges();
        assert_eq!(balance.total_exchanges, 2);
    }

    #[test]
    fn test_update_rating() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut balance = OwnerCreditBalance::new(owner_id, building_id);

        // Valid rating
        assert!(balance.update_rating(4.5).is_ok());
        assert_eq!(balance.average_rating, Some(4.5));

        // Invalid rating (too low)
        assert!(balance.update_rating(0.5).is_err());

        // Invalid rating (too high)
        assert!(balance.update_rating(5.5).is_err());
    }

    #[test]
    fn test_has_sufficient_credits() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut balance = OwnerCreditBalance::new(owner_id, building_id);

        balance.earn_credits(10).unwrap();

        assert!(balance.has_sufficient_credits(5));
        assert!(balance.has_sufficient_credits(10));
        assert!(!balance.has_sufficient_credits(11));
    }

    #[test]
    fn test_participation_levels() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut balance = OwnerCreditBalance::new(owner_id, building_id);

        // New
        assert_eq!(balance.participation_level(), ParticipationLevel::New);

        // Beginner (1-5)
        for _ in 0..3 {
            balance.increment_exchanges();
        }
        assert_eq!(balance.participation_level(), ParticipationLevel::Beginner);

        // Active (6-20)
        for _ in 0..10 {
            balance.increment_exchanges();
        }
        assert_eq!(balance.participation_level(), ParticipationLevel::Active);

        // Veteran (21-50)
        for _ in 0..15 {
            balance.increment_exchanges();
        }
        assert_eq!(balance.participation_level(), ParticipationLevel::Veteran);

        // Expert (51+)
        for _ in 0..25 {
            balance.increment_exchanges();
        }
        assert_eq!(balance.participation_level(), ParticipationLevel::Expert);
    }

    #[test]
    fn test_credit_status() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut balance = OwnerCreditBalance::new(owner_id, building_id);

        // Balanced
        assert_eq!(balance.credit_status(), CreditStatus::Balanced);

        // Positive
        balance.earn_credits(5).unwrap();
        assert_eq!(balance.credit_status(), CreditStatus::Positive);

        // Negative
        balance.spend_credits(10).unwrap();
        assert_eq!(balance.credit_status(), CreditStatus::Negative);
    }
}
