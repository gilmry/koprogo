use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Represents a Proof of Green (PoG) blockchain entry
/// This is a lightweight blockchain implementation for validating green energy contributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreenProof {
    pub id: Uuid,
    pub task_id: Uuid,
    pub node_id: Uuid,
    pub block_hash: String,
    pub previous_hash: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub energy_used_wh: f64,
    pub solar_contribution_wh: f64,
    pub carbon_saved_kg: f64,
    pub nonce: u64,
}

impl GreenProof {
    /// Creates a new Proof of Green
    pub fn new(
        task_id: Uuid,
        node_id: Uuid,
        energy_used_wh: f64,
        solar_contribution_wh: f64,
        previous_hash: Option<String>,
    ) -> Result<Self, String> {
        if energy_used_wh < 0.0 {
            return Err("Energy used cannot be negative".to_string());
        }

        if solar_contribution_wh < 0.0 {
            return Err("Solar contribution cannot be negative".to_string());
        }

        if solar_contribution_wh > energy_used_wh {
            return Err("Solar contribution cannot exceed total energy used".to_string());
        }

        // Calculate carbon saved
        // Belgian grid carbon intensity: ~0.18 kg CO2/kWh = 0.00018 kg CO2/Wh
        // Solar contribution saves this carbon
        let carbon_saved_kg = (solar_contribution_wh / 1000.0) * 0.18;

        let timestamp = Utc::now();
        let mut proof = GreenProof {
            id: Uuid::new_v4(),
            task_id,
            node_id,
            block_hash: String::new(),
            previous_hash,
            timestamp,
            energy_used_wh,
            solar_contribution_wh,
            carbon_saved_kg,
            nonce: 0,
        };

        // Mine the block (lightweight PoW with difficulty = 1 leading zero)
        proof.mine_block(1)?;

        Ok(proof)
    }

    /// Mines the block by finding a valid hash with specified difficulty
    /// Difficulty = number of leading zeros required in the hash
    fn mine_block(&mut self, difficulty: usize) -> Result<(), String> {
        let target = "0".repeat(difficulty);

        loop {
            let hash = self.calculate_hash();
            if hash.starts_with(&target) {
                self.block_hash = hash;
                return Ok(());
            }

            self.nonce += 1;

            // Prevent infinite loops in case of errors
            if self.nonce > 1_000_000 {
                return Err("Mining timeout: could not find valid hash".to_string());
            }
        }
    }

    /// Calculates the hash of the block
    fn calculate_hash(&self) -> String {
        let data = format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.id,
            self.task_id,
            self.node_id,
            self.previous_hash.as_ref().unwrap_or(&"genesis".to_string()),
            self.timestamp.timestamp(),
            self.energy_used_wh,
            self.solar_contribution_wh,
            self.carbon_saved_kg,
            self.nonce
        );

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verifies the integrity of the proof
    pub fn verify(&self) -> bool {
        // Check that the stored hash matches the calculated hash
        let calculated_hash = self.calculate_hash();
        if self.block_hash != calculated_hash {
            return false;
        }

        // Check that the hash has at least 1 leading zero (difficulty = 1)
        if !self.block_hash.starts_with('0') {
            return false;
        }

        // Check business logic constraints
        if self.energy_used_wh < 0.0 || self.solar_contribution_wh < 0.0 {
            return false;
        }

        if self.solar_contribution_wh > self.energy_used_wh {
            return false;
        }

        true
    }

    /// Returns the percentage of energy that came from solar
    pub fn green_percentage(&self) -> f64 {
        if self.energy_used_wh == 0.0 {
            0.0
        } else {
            (self.solar_contribution_wh / self.energy_used_wh) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_green_proof_success() {
        let task_id = Uuid::new_v4();
        let node_id = Uuid::new_v4();

        let proof = GreenProof::new(task_id, node_id, 100.0, 60.0, None);

        assert!(proof.is_ok());
        let proof = proof.unwrap();
        assert_eq!(proof.task_id, task_id);
        assert_eq!(proof.node_id, node_id);
        assert_eq!(proof.energy_used_wh, 100.0);
        assert_eq!(proof.solar_contribution_wh, 60.0);
        assert!(!proof.block_hash.is_empty());
        assert!(proof.block_hash.starts_with('0')); // Difficulty = 1
    }

    #[test]
    fn test_create_green_proof_negative_energy() {
        let result = GreenProof::new(Uuid::new_v4(), Uuid::new_v4(), -10.0, 5.0, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Energy used cannot be negative");
    }

    #[test]
    fn test_create_green_proof_solar_exceeds_total() {
        let result = GreenProof::new(Uuid::new_v4(), Uuid::new_v4(), 50.0, 60.0, None);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Solar contribution cannot exceed total energy used"
        );
    }

    #[test]
    fn test_verify_valid_proof() {
        let proof = GreenProof::new(Uuid::new_v4(), Uuid::new_v4(), 100.0, 60.0, None).unwrap();
        assert!(proof.verify());
    }

    #[test]
    fn test_verify_invalid_hash() {
        let mut proof = GreenProof::new(Uuid::new_v4(), Uuid::new_v4(), 100.0, 60.0, None).unwrap();

        // Tamper with the hash
        proof.block_hash = "invalid_hash".to_string();

        assert!(!proof.verify());
    }

    #[test]
    fn test_green_percentage() {
        let proof = GreenProof::new(Uuid::new_v4(), Uuid::new_v4(), 100.0, 60.0, None).unwrap();
        assert!((proof.green_percentage() - 60.0).abs() < 0.01);

        let proof_full = GreenProof::new(Uuid::new_v4(), Uuid::new_v4(), 100.0, 100.0, None).unwrap();
        assert!((proof_full.green_percentage() - 100.0).abs() < 0.01);

        let proof_zero = GreenProof::new(Uuid::new_v4(), Uuid::new_v4(), 100.0, 0.0, None).unwrap();
        assert!((proof_zero.green_percentage() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_carbon_saved_calculation() {
        // 100 Wh with 60 Wh from solar
        // Carbon saved = (60 / 1000) * 0.18 = 0.0108 kg CO2
        let proof = GreenProof::new(Uuid::new_v4(), Uuid::new_v4(), 100.0, 60.0, None).unwrap();
        assert!((proof.carbon_saved_kg - 0.0108).abs() < 0.0001);
    }

    #[test]
    fn test_blockchain_chaining() {
        let proof1 = GreenProof::new(Uuid::new_v4(), Uuid::new_v4(), 100.0, 60.0, None).unwrap();

        let proof2 = GreenProof::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            150.0,
            90.0,
            Some(proof1.block_hash.clone()),
        )
        .unwrap();

        assert_eq!(proof2.previous_hash, Some(proof1.block_hash));
        assert!(proof2.verify());
    }
}
