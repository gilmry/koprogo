use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type de résolution (ordinaire ou extraordinaire)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionType {
    Ordinary,      // Résolution ordinaire (majorité simple)
    Extraordinary, // Résolution extraordinaire (majorité qualifiée)
}

/// Type de majorité requise pour adoption
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MajorityType {
    Simple,         // Majorité simple: 50% + 1 des votes exprimés
    Absolute,       // Majorité absolue: 50% + 1 de tous les votes possibles
    Qualified(f64), // Majorité qualifiée: seuil personnalisé (ex: 0.67 pour 2/3)
}

/// Statut d'une résolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionStatus {
    Pending,  // En attente de vote
    Adopted,  // Adoptée
    Rejected, // Rejetée
}

/// Résolution soumise au vote lors d'une assemblée générale
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resolution {
    pub id: Uuid,
    pub meeting_id: Uuid,
    pub title: String,
    pub description: String,
    pub resolution_type: ResolutionType,
    pub majority_required: MajorityType,
    pub vote_count_pour: i32,
    pub vote_count_contre: i32,
    pub vote_count_abstention: i32,
    pub total_voting_power_pour: f64,
    pub total_voting_power_contre: f64,
    pub total_voting_power_abstention: f64,
    pub status: ResolutionStatus,
    pub created_at: DateTime<Utc>,
    pub voted_at: Option<DateTime<Utc>>,
}

impl Resolution {
    /// Crée une nouvelle résolution
    pub fn new(
        meeting_id: Uuid,
        title: String,
        description: String,
        resolution_type: ResolutionType,
        majority_required: MajorityType,
    ) -> Result<Self, String> {
        if title.is_empty() {
            return Err("Resolution title cannot be empty".to_string());
        }
        if description.is_empty() {
            return Err("Resolution description cannot be empty".to_string());
        }

        // Validate qualified majority threshold
        if let MajorityType::Qualified(threshold) = &majority_required {
            if *threshold <= 0.0 || *threshold > 1.0 {
                return Err("Qualified majority threshold must be between 0 and 1".to_string());
            }
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            meeting_id,
            title,
            description,
            resolution_type,
            majority_required,
            vote_count_pour: 0,
            vote_count_contre: 0,
            vote_count_abstention: 0,
            total_voting_power_pour: 0.0,
            total_voting_power_contre: 0.0,
            total_voting_power_abstention: 0.0,
            status: ResolutionStatus::Pending,
            created_at: now,
            voted_at: None,
        })
    }

    /// Enregistre un vote "Pour" et met à jour les compteurs
    pub fn record_vote_pour(&mut self, voting_power: f64) {
        self.vote_count_pour += 1;
        self.total_voting_power_pour += voting_power;
    }

    /// Enregistre un vote "Contre" et met à jour les compteurs
    pub fn record_vote_contre(&mut self, voting_power: f64) {
        self.vote_count_contre += 1;
        self.total_voting_power_contre += voting_power;
    }

    /// Enregistre une abstention et met à jour les compteurs
    pub fn record_abstention(&mut self, voting_power: f64) {
        self.vote_count_abstention += 1;
        self.total_voting_power_abstention += voting_power;
    }

    /// Calcule le résultat du vote en fonction du type de majorité
    pub fn calculate_result(&self, total_voting_power: f64) -> ResolutionStatus {
        match &self.majority_required {
            MajorityType::Simple => {
                // Majorité simple: plus de voix "Pour" que "Contre" + "Abstention"
                if self.total_voting_power_pour
                    > self.total_voting_power_contre + self.total_voting_power_abstention
                {
                    ResolutionStatus::Adopted
                } else {
                    ResolutionStatus::Rejected
                }
            }
            MajorityType::Absolute => {
                // Majorité absolue: plus de 50% du pouvoir de vote total
                if self.total_voting_power_pour > total_voting_power / 2.0 {
                    ResolutionStatus::Adopted
                } else {
                    ResolutionStatus::Rejected
                }
            }
            MajorityType::Qualified(threshold) => {
                // Majorité qualifiée: ratio >= seuil défini
                let pour_ratio = if total_voting_power > 0.0 {
                    self.total_voting_power_pour / total_voting_power
                } else {
                    0.0
                };
                if pour_ratio >= *threshold {
                    ResolutionStatus::Adopted
                } else {
                    ResolutionStatus::Rejected
                }
            }
        }
    }

    /// Clôture le vote et finalise le statut
    pub fn close_voting(&mut self, total_voting_power: f64) -> Result<(), String> {
        if self.status != ResolutionStatus::Pending {
            return Err("Voting already closed for this resolution".to_string());
        }

        self.status = self.calculate_result(total_voting_power);
        self.voted_at = Some(Utc::now());
        Ok(())
    }

    /// Retourne le nombre total de votes exprimés
    pub fn total_votes(&self) -> i32 {
        self.vote_count_pour + self.vote_count_contre + self.vote_count_abstention
    }

    /// Retourne le pourcentage de votes "Pour"
    pub fn pour_percentage(&self) -> f64 {
        let total = self.total_votes();
        if total > 0 {
            (self.vote_count_pour as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Retourne le pourcentage de votes "Contre"
    pub fn contre_percentage(&self) -> f64 {
        let total = self.total_votes();
        if total > 0 {
            (self.vote_count_contre as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Retourne le pourcentage d'abstentions
    pub fn abstention_percentage(&self) -> f64 {
        let total = self.total_votes();
        if total > 0 {
            (self.vote_count_abstention as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_resolution_success() {
        let meeting_id = Uuid::new_v4();
        let resolution = Resolution::new(
            meeting_id,
            "Approbation des comptes 2024".to_string(),
            "Vote pour approuver les comptes annuels de l'exercice 2024".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Simple,
        );

        assert!(resolution.is_ok());
        let resolution = resolution.unwrap();
        assert_eq!(resolution.meeting_id, meeting_id);
        assert_eq!(resolution.status, ResolutionStatus::Pending);
        assert_eq!(resolution.total_votes(), 0);
    }

    #[test]
    fn test_create_resolution_empty_title_fails() {
        let meeting_id = Uuid::new_v4();
        let resolution = Resolution::new(
            meeting_id,
            "".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Simple,
        );

        assert!(resolution.is_err());
        assert_eq!(resolution.unwrap_err(), "Resolution title cannot be empty");
    }

    #[test]
    fn test_create_resolution_invalid_qualified_threshold_fails() {
        let meeting_id = Uuid::new_v4();
        let resolution = Resolution::new(
            meeting_id,
            "Test".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::Qualified(1.5), // Invalid: > 1.0
        );

        assert!(resolution.is_err());
        assert!(resolution
            .unwrap_err()
            .contains("threshold must be between 0 and 1"));
    }

    #[test]
    fn test_record_votes() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Simple,
        )
        .unwrap();

        resolution.record_vote_pour(100.0);
        resolution.record_vote_pour(150.0);
        resolution.record_vote_contre(200.0);
        resolution.record_abstention(50.0);

        assert_eq!(resolution.vote_count_pour, 2);
        assert_eq!(resolution.vote_count_contre, 1);
        assert_eq!(resolution.vote_count_abstention, 1);
        assert_eq!(resolution.total_voting_power_pour, 250.0);
        assert_eq!(resolution.total_voting_power_contre, 200.0);
        assert_eq!(resolution.total_voting_power_abstention, 50.0);
        assert_eq!(resolution.total_votes(), 4);
    }

    #[test]
    fn test_calculate_result_simple_majority_adopted() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Simple,
        )
        .unwrap();

        resolution.record_vote_pour(300.0); // Pour > Contre + Abstention
        resolution.record_vote_contre(150.0);
        resolution.record_abstention(50.0);

        let result = resolution.calculate_result(1000.0);
        assert_eq!(result, ResolutionStatus::Adopted);
    }

    #[test]
    fn test_calculate_result_simple_majority_rejected() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Simple,
        )
        .unwrap();

        resolution.record_vote_pour(150.0);
        resolution.record_vote_contre(300.0); // Contre + Abstention > Pour
        resolution.record_abstention(50.0);

        let result = resolution.calculate_result(1000.0);
        assert_eq!(result, ResolutionStatus::Rejected);
    }

    #[test]
    fn test_calculate_result_absolute_majority_adopted() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
        )
        .unwrap();

        resolution.record_vote_pour(600.0); // > 50% of 1000
        resolution.record_vote_contre(200.0);
        resolution.record_abstention(100.0);

        let result = resolution.calculate_result(1000.0);
        assert_eq!(result, ResolutionStatus::Adopted);
    }

    #[test]
    fn test_calculate_result_absolute_majority_rejected() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
        )
        .unwrap();

        resolution.record_vote_pour(400.0); // < 50% of 1000
        resolution.record_vote_contre(300.0);
        resolution.record_abstention(100.0);

        let result = resolution.calculate_result(1000.0);
        assert_eq!(result, ResolutionStatus::Rejected);
    }

    #[test]
    fn test_calculate_result_qualified_majority_adopted() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::Qualified(0.67), // 2/3 required
        )
        .unwrap();

        resolution.record_vote_pour(700.0); // 70% > 67%
        resolution.record_vote_contre(200.0);
        resolution.record_abstention(100.0);

        let result = resolution.calculate_result(1000.0);
        assert_eq!(result, ResolutionStatus::Adopted);
    }

    #[test]
    fn test_calculate_result_qualified_majority_rejected() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::Qualified(0.67), // 2/3 required
        )
        .unwrap();

        resolution.record_vote_pour(600.0); // 60% < 67%
        resolution.record_vote_contre(300.0);
        resolution.record_abstention(100.0);

        let result = resolution.calculate_result(1000.0);
        assert_eq!(result, ResolutionStatus::Rejected);
    }

    #[test]
    fn test_close_voting_success() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Simple,
        )
        .unwrap();

        resolution.record_vote_pour(300.0);
        resolution.record_vote_contre(150.0);

        let result = resolution.close_voting(1000.0);
        assert!(result.is_ok());
        assert_eq!(resolution.status, ResolutionStatus::Adopted);
        assert!(resolution.voted_at.is_some());
    }

    #[test]
    fn test_close_voting_already_closed_fails() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Simple,
        )
        .unwrap();

        resolution.record_vote_pour(300.0);
        resolution.close_voting(1000.0).unwrap();

        let result = resolution.close_voting(1000.0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Voting already closed for this resolution"
        );
    }

    #[test]
    fn test_percentages() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Simple,
        )
        .unwrap();

        resolution.record_vote_pour(100.0);
        resolution.record_vote_pour(100.0); // 2 votes pour
        resolution.record_vote_contre(100.0); // 1 vote contre
        resolution.record_abstention(100.0); // 1 abstention

        assert_eq!(resolution.pour_percentage(), 50.0); // 2/4 = 50%
        assert_eq!(resolution.contre_percentage(), 25.0); // 1/4 = 25%
        assert_eq!(resolution.abstention_percentage(), 25.0); // 1/4 = 25%
    }
}
