use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents carbon credits earned through green computing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonCredit {
    pub id: Uuid,
    pub node_id: Uuid,
    pub task_id: Uuid,
    pub proof_id: Uuid,
    pub amount_kg_co2: f64,
    pub euro_value: f64,
    pub status: CreditStatus,
    pub cooperative_share: f64,
    pub node_share: f64,
    pub created_at: DateTime<Utc>,
    pub redeemed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditStatus {
    Pending,
    Verified,
    Redeemed,
    Expired,
}

impl CarbonCredit {
    /// Creates a new carbon credit
    /// The cooperative takes 30% for the solidarity fund, node gets 70%
    pub fn new(
        node_id: Uuid,
        task_id: Uuid,
        proof_id: Uuid,
        carbon_saved_kg: f64,
    ) -> Result<Self, String> {
        if carbon_saved_kg < 0.0 {
            return Err("Carbon saved cannot be negative".to_string());
        }

        // Belgian carbon credit price: ~€25 per ton CO2 = €0.025 per kg CO2
        let euro_value = carbon_saved_kg * 0.025;

        // Split: 30% cooperative, 70% node
        let cooperative_share = euro_value * 0.30;
        let node_share = euro_value * 0.70;

        Ok(CarbonCredit {
            id: Uuid::new_v4(),
            node_id,
            task_id,
            proof_id,
            amount_kg_co2: carbon_saved_kg,
            euro_value,
            status: CreditStatus::Pending,
            cooperative_share,
            node_share,
            created_at: Utc::now(),
            redeemed_at: None,
        })
    }

    /// Verifies the carbon credit
    pub fn verify(&mut self) -> Result<(), String> {
        if self.status != CreditStatus::Pending {
            return Err(format!("Cannot verify credit in status {:?}", self.status));
        }

        self.status = CreditStatus::Verified;
        Ok(())
    }

    /// Redeems the carbon credit
    pub fn redeem(&mut self) -> Result<(), String> {
        if self.status != CreditStatus::Verified {
            return Err(format!("Cannot redeem credit in status {:?}", self.status));
        }

        self.status = CreditStatus::Redeemed;
        self.redeemed_at = Some(Utc::now());
        Ok(())
    }

    /// Returns the total value split as (cooperative_eur, node_eur)
    pub fn value_split(&self) -> (f64, f64) {
        (self.cooperative_share, self.node_share)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_carbon_credit_success() {
        let node_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let proof_id = Uuid::new_v4();

        let credit = CarbonCredit::new(node_id, task_id, proof_id, 0.0108);

        assert!(credit.is_ok());
        let credit = credit.unwrap();
        assert_eq!(credit.node_id, node_id);
        assert_eq!(credit.amount_kg_co2, 0.0108);
        assert_eq!(credit.status, CreditStatus::Pending);

        // Euro value: 0.0108 * 0.025 = 0.00027
        assert!((credit.euro_value - 0.00027).abs() < 0.000001);

        // Cooperative share: 0.00027 * 0.30 = 0.000081
        assert!((credit.cooperative_share - 0.000081).abs() < 0.0000001);

        // Node share: 0.00027 * 0.70 = 0.000189
        assert!((credit.node_share - 0.000189).abs() < 0.0000001);
    }

    #[test]
    fn test_create_carbon_credit_negative() {
        let result = CarbonCredit::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), -0.01);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Carbon saved cannot be negative");
    }

    #[test]
    fn test_verify_credit() {
        let mut credit =
            CarbonCredit::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), 0.01).unwrap();

        let result = credit.verify();
        assert!(result.is_ok());
        assert_eq!(credit.status, CreditStatus::Verified);
    }

    #[test]
    fn test_redeem_credit() {
        let mut credit =
            CarbonCredit::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), 0.01).unwrap();

        credit.verify().unwrap();
        let result = credit.redeem();

        assert!(result.is_ok());
        assert_eq!(credit.status, CreditStatus::Redeemed);
        assert!(credit.redeemed_at.is_some());
    }

    #[test]
    fn test_cannot_redeem_unverified() {
        let mut credit =
            CarbonCredit::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), 0.01).unwrap();

        let result = credit.redeem();
        assert!(result.is_err());
    }

    #[test]
    fn test_value_split() {
        let credit =
            CarbonCredit::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), 1.0).unwrap();

        let (coop, node) = credit.value_split();

        // 1.0 kg * €0.025 = €0.025 total
        // Cooperative: €0.025 * 0.30 = €0.0075
        // Node: €0.025 * 0.70 = €0.0175
        assert!((coop - 0.0075).abs() < 0.00001);
        assert!((node - 0.0175).abs() < 0.00001);
    }
}
