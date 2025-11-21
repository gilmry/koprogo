use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Choix de vote d'un copropriétaire
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VoteChoice {
    Pour,       // Vote en faveur (For)
    Contre,     // Vote contre (Against)
    Abstention, // Abstention
}

/// Vote d'un propriétaire sur une résolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vote {
    pub id: Uuid,
    pub resolution_id: Uuid,
    pub owner_id: Uuid,
    pub unit_id: Uuid,
    pub vote_choice: VoteChoice,
    pub voting_power: f64,            // Tantièmes/millièmes du lot
    pub proxy_owner_id: Option<Uuid>, // ID du mandataire si vote par procuration
    pub voted_at: DateTime<Utc>,
}

impl Vote {
    /// Crée un nouveau vote
    pub fn new(
        resolution_id: Uuid,
        owner_id: Uuid,
        unit_id: Uuid,
        vote_choice: VoteChoice,
        voting_power: f64,
        proxy_owner_id: Option<Uuid>,
    ) -> Result<Self, String> {
        // Validation du pouvoir de vote
        if voting_power <= 0.0 {
            return Err("Voting power must be positive".to_string());
        }
        if voting_power > 1000.0 {
            return Err("Voting power exceeds maximum (1000 millièmes)".to_string());
        }

        // Validation de la procuration
        if let Some(proxy_id) = proxy_owner_id {
            if proxy_id == owner_id {
                return Err("Owner cannot be their own proxy".to_string());
            }
        }

        Ok(Self {
            id: Uuid::new_v4(),
            resolution_id,
            owner_id,
            unit_id,
            vote_choice,
            voting_power,
            proxy_owner_id,
            voted_at: Utc::now(),
        })
    }

    /// Vérifie si le vote est exprimé par procuration
    pub fn is_proxy_vote(&self) -> bool {
        self.proxy_owner_id.is_some()
    }

    /// Retourne l'ID du votant effectif (propriétaire ou mandataire)
    pub fn effective_voter_id(&self) -> Uuid {
        self.proxy_owner_id.unwrap_or(self.owner_id)
    }

    /// Modifie le choix de vote (seulement si pas encore enregistré)
    pub fn change_vote(&mut self, new_choice: VoteChoice) -> Result<(), String> {
        // En pratique, cette méthode ne serait appelée que pendant une fenêtre de temps limitée
        // Ici on autorise le changement, mais dans l'application on pourrait ajouter une validation
        // basée sur le timing (ex: vote modifiable uniquement dans les 5 minutes)
        self.vote_choice = new_choice;
        self.voted_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vote_success() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            150.0, // 150 millièmes
            None,
        );

        assert!(vote.is_ok());
        let vote = vote.unwrap();
        assert_eq!(vote.resolution_id, resolution_id);
        assert_eq!(vote.owner_id, owner_id);
        assert_eq!(vote.unit_id, unit_id);
        assert_eq!(vote.vote_choice, VoteChoice::Pour);
        assert_eq!(vote.voting_power, 150.0);
        assert!(!vote.is_proxy_vote());
        assert_eq!(vote.effective_voter_id(), owner_id);
    }

    #[test]
    fn test_create_vote_with_proxy() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let proxy_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Contre,
            200.0,
            Some(proxy_id),
        );

        assert!(vote.is_ok());
        let vote = vote.unwrap();
        assert!(vote.is_proxy_vote());
        assert_eq!(vote.effective_voter_id(), proxy_id);
        assert_eq!(vote.proxy_owner_id, Some(proxy_id));
    }

    #[test]
    fn test_create_vote_zero_voting_power_fails() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            0.0,
            None,
        );

        assert!(vote.is_err());
        assert_eq!(vote.unwrap_err(), "Voting power must be positive");
    }

    #[test]
    fn test_create_vote_negative_voting_power_fails() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            -50.0,
            None,
        );

        assert!(vote.is_err());
        assert_eq!(vote.unwrap_err(), "Voting power must be positive");
    }

    #[test]
    fn test_create_vote_excessive_voting_power_fails() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            1500.0, // Exceeds max
            None,
        );

        assert!(vote.is_err());
        assert!(vote.unwrap_err().contains("exceeds maximum"));
    }

    #[test]
    fn test_create_vote_self_proxy_fails() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            150.0,
            Some(owner_id), // Self as proxy
        );

        assert!(vote.is_err());
        assert_eq!(vote.unwrap_err(), "Owner cannot be their own proxy");
    }

    #[test]
    fn test_change_vote() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let mut vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            150.0,
            None,
        )
        .unwrap();

        assert_eq!(vote.vote_choice, VoteChoice::Pour);

        let result = vote.change_vote(VoteChoice::Contre);
        assert!(result.is_ok());
        assert_eq!(vote.vote_choice, VoteChoice::Contre);
    }

    #[test]
    fn test_vote_choice_serialization() {
        // Test serialization of VoteChoice enum
        let pour = VoteChoice::Pour;
        let contre = VoteChoice::Contre;
        let abstention = VoteChoice::Abstention;

        let json_pour = serde_json::to_string(&pour).unwrap();
        let json_contre = serde_json::to_string(&contre).unwrap();
        let json_abstention = serde_json::to_string(&abstention).unwrap();

        assert_eq!(json_pour, "\"pour\"");
        assert_eq!(json_contre, "\"contre\"");
        assert_eq!(json_abstention, "\"abstention\"");
    }

    #[test]
    fn test_vote_choice_deserialization() {
        // Test deserialization of VoteChoice enum
        let pour: VoteChoice = serde_json::from_str("\"pour\"").unwrap();
        let contre: VoteChoice = serde_json::from_str("\"contre\"").unwrap();
        let abstention: VoteChoice = serde_json::from_str("\"abstention\"").unwrap();

        assert_eq!(pour, VoteChoice::Pour);
        assert_eq!(contre, VoteChoice::Contre);
        assert_eq!(abstention, VoteChoice::Abstention);
    }
}
