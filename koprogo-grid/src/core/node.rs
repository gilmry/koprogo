use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a compute node in the decentralized grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub cpu_cores: u32,
    pub has_solar: bool,
    pub location: String,
    pub status: NodeStatus,
    pub eco_score: f64,
    pub total_energy_saved_wh: f64,
    pub total_carbon_credits: f64,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Active,
    Idle,
    Offline,
    Suspended,
}

impl Node {
    /// Creates a new node with validation
    pub fn new(
        name: String,
        cpu_cores: u32,
        has_solar: bool,
        location: String,
    ) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Node name cannot be empty".to_string());
        }

        if cpu_cores == 0 {
            return Err("CPU cores must be greater than 0".to_string());
        }

        if cpu_cores > 256 {
            return Err("CPU cores cannot exceed 256".to_string());
        }

        if location.trim().is_empty() {
            return Err("Location cannot be empty".to_string());
        }

        let now = Utc::now();

        Ok(Node {
            id: Uuid::new_v4(),
            name,
            cpu_cores,
            has_solar,
            location,
            status: NodeStatus::Active,
            eco_score: 0.0,
            total_energy_saved_wh: 0.0,
            total_carbon_credits: 0.0,
            last_heartbeat: Some(now),
            created_at: now,
            updated_at: now,
        })
    }

    /// Updates the eco score based on CPU usage and solar contribution
    /// Formula: eco_score = (idle_cpu_percentage * 0.5) + (solar_contribution * 0.5)
    pub fn update_eco_score(&mut self, cpu_usage_percent: f64, solar_watts: f64) {
        let idle_cpu_percentage = 100.0 - cpu_usage_percent;
        let idle_factor = (idle_cpu_percentage / 100.0).max(0.0).min(1.0);

        let solar_factor = if self.has_solar {
            // Normalize solar watts (assuming max 1000W for residential solar)
            (solar_watts / 1000.0).max(0.0).min(1.0)
        } else {
            0.0
        };

        // Weighted average: 50% idle CPU + 50% solar contribution
        self.eco_score = (idle_factor * 0.5) + (solar_factor * 0.5);
        self.updated_at = Utc::now();
    }

    /// Marks the node as having received a heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Some(Utc::now());
        self.status = NodeStatus::Active;
        self.updated_at = Utc::now();
    }

    /// Checks if the node is considered offline (no heartbeat in 5 minutes)
    pub fn is_offline(&self) -> bool {
        match self.last_heartbeat {
            Some(last) => {
                let now = Utc::now();
                let duration = now.signed_duration_since(last);
                duration.num_minutes() > 5
            }
            None => true,
        }
    }

    /// Adds energy savings to the node's total
    pub fn add_energy_saved(&mut self, energy_wh: f64) {
        self.total_energy_saved_wh += energy_wh;
        self.updated_at = Utc::now();
    }

    /// Adds carbon credits to the node's total
    pub fn add_carbon_credits(&mut self, credits: f64) {
        self.total_carbon_credits += credits;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node_success() {
        let node = Node::new(
            "RaspberryPi-001".to_string(),
            4,
            true,
            "Brussels".to_string(),
        );

        assert!(node.is_ok());
        let node = node.unwrap();
        assert_eq!(node.name, "RaspberryPi-001");
        assert_eq!(node.cpu_cores, 4);
        assert!(node.has_solar);
        assert_eq!(node.eco_score, 0.0);
        assert_eq!(node.status, NodeStatus::Active);
    }

    #[test]
    fn test_create_node_empty_name() {
        let result = Node::new("".to_string(), 4, true, "Brussels".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Node name cannot be empty");
    }

    #[test]
    fn test_create_node_zero_cores() {
        let result = Node::new("Node-1".to_string(), 0, true, "Brussels".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "CPU cores must be greater than 0");
    }

    #[test]
    fn test_update_eco_score_with_solar() {
        let mut node = Node::new("Node-1".to_string(), 4, true, "Brussels".to_string()).unwrap();

        // 80% idle CPU, 500W solar (50% of max 1000W)
        node.update_eco_score(20.0, 500.0);

        // Expected: (0.8 * 0.5) + (0.5 * 0.5) = 0.4 + 0.25 = 0.65
        assert!((node.eco_score - 0.65).abs() < 0.01);
    }

    #[test]
    fn test_update_eco_score_without_solar() {
        let mut node = Node::new("Node-1".to_string(), 4, false, "Brussels".to_string()).unwrap();

        // 80% idle CPU, solar doesn't count
        node.update_eco_score(20.0, 500.0);

        // Expected: (0.8 * 0.5) + (0.0 * 0.5) = 0.4
        assert!((node.eco_score - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_heartbeat() {
        let mut node = Node::new("Node-1".to_string(), 4, true, "Brussels".to_string()).unwrap();
        let before = node.last_heartbeat;

        std::thread::sleep(std::time::Duration::from_millis(10));
        node.heartbeat();

        assert!(node.last_heartbeat > before);
        assert_eq!(node.status, NodeStatus::Active);
    }

    #[test]
    fn test_add_energy_saved() {
        let mut node = Node::new("Node-1".to_string(), 4, true, "Brussels".to_string()).unwrap();

        node.add_energy_saved(100.5);
        assert!((node.total_energy_saved_wh - 100.5).abs() < 0.01);

        node.add_energy_saved(50.0);
        assert!((node.total_energy_saved_wh - 150.5).abs() < 0.01);
    }

    #[test]
    fn test_add_carbon_credits() {
        let mut node = Node::new("Node-1".to_string(), 4, true, "Brussels".to_string()).unwrap();

        node.add_carbon_credits(0.5);
        assert!((node.total_carbon_credits - 0.5).abs() < 0.01);

        node.add_carbon_credits(1.0);
        assert!((node.total_carbon_credits - 1.5).abs() < 0.01);
    }
}
