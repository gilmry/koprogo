use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type de résolution (ordinaire ou extraordinaire)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionType {
    Ordinary,      // Résolution ordinaire (majorité simple)
    Extraordinary, // Résolution extraordinaire (majorité qualifiée)
}

/// Type de majorité requise pour adoption — Art. 3.88 §1 Code Civil belge
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MajorityType {
    /// Plus de 50% des présents/représentés, abstentions EXCLUES — Art. 3.88 §1 (DÉFAUT).
    /// Comptes, budget, syndic, commissaire, entretien courant, travaux imposés par la loi.
    Absolute,
    /// ≥2/3 des présents/représentés, abstentions EXCLUES — Art. 3.88 §1, 1°
    /// Modification statuts (jouissance/usage), travaux parties communes, mise en concurrence
    TwoThirds,
    /// ≥4/5 des présents/représentés, abstentions EXCLUES — Art. 3.88 §1, 2°
    /// Modification répartition charges, destination, reconstruction partielle, aliénation
    FourFifths,
    /// 100% de TOUS les tantièmes (y compris absents) — Art. 3.88 §1, 3°
    /// Modification quotités de copropriété, reconstruction totale
    Unanimity,
}

/// Statut d'une résolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionStatus {
    Pending,  // En attente de vote
    Adopted,  // Adoptée
    Rejected, // Rejetée
}

/// Résolution soumise au vote lors d'une assemblée générale
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
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
    pub total_voting_power_pour: Decimal,
    pub total_voting_power_contre: Decimal,
    pub total_voting_power_abstention: Decimal,
    pub status: ResolutionStatus,
    // Issue #310: Link resolution to agenda item
    pub agenda_item_index: Option<usize>, // Index into meeting.agenda Vec
    pub created_at: DateTime<Utc>,
    pub voted_at: Option<DateTime<Utc>>,
}

impl Resolution {
    /// Crée une nouvelle résolution
    /// Issue #310: Optional agenda_item_index for Art. 3.87 CC compliance (Belgian law)
    pub fn new(
        meeting_id: Uuid,
        title: String,
        description: String,
        resolution_type: ResolutionType,
        majority_required: MajorityType,
        agenda_item_index: Option<usize>,
    ) -> Result<Self, String> {
        if title.is_empty() {
            return Err("Resolution title cannot be empty".to_string());
        }
        if description.is_empty() {
            return Err("Resolution description cannot be empty".to_string());
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
            total_voting_power_pour: Decimal::ZERO,
            total_voting_power_contre: Decimal::ZERO,
            total_voting_power_abstention: Decimal::ZERO,
            status: ResolutionStatus::Pending,
            agenda_item_index,
            created_at: now,
            voted_at: None,
        })
    }

    /// Enregistre un vote "Pour" et met à jour les compteurs
    pub fn record_vote_pour(&mut self, voting_power: Decimal) {
        self.vote_count_pour += 1;
        self.total_voting_power_pour += voting_power;
    }

    /// Enregistre un vote "Contre" et met à jour les compteurs
    pub fn record_vote_contre(&mut self, voting_power: Decimal) {
        self.vote_count_contre += 1;
        self.total_voting_power_contre += voting_power;
    }

    /// Enregistre une abstention et met à jour les compteurs
    pub fn record_abstention(&mut self, voting_power: Decimal) {
        self.vote_count_abstention += 1;
        self.total_voting_power_abstention += voting_power;
    }

    /// Calcule le résultat du vote en fonction du type de majorité — Art. 3.88 §1 Code Civil belge
    pub fn calculate_result(&self, total_voting_power: Decimal) -> ResolutionStatus {
        let expressed = self.total_voting_power_pour + self.total_voting_power_contre;

        match &self.majority_required {
            MajorityType::Absolute => {
                // Art. 3.88 §1: >50% des voix exprimées (hors abstentions)
                if expressed > Decimal::ZERO && self.total_voting_power_pour > expressed / dec!(2) {
                    ResolutionStatus::Adopted
                } else {
                    ResolutionStatus::Rejected
                }
            }
            MajorityType::TwoThirds => {
                // Art. 3.88 §1, 1°: ≥2/3 des voix exprimées
                if expressed > Decimal::ZERO
                    && self.total_voting_power_pour / expressed >= dec!(2) / dec!(3)
                {
                    ResolutionStatus::Adopted
                } else {
                    ResolutionStatus::Rejected
                }
            }
            MajorityType::FourFifths => {
                // Art. 3.88 §1, 2°: ≥4/5 des voix exprimées
                if expressed > Decimal::ZERO
                    && self.total_voting_power_pour / expressed >= dec!(4) / dec!(5)
                {
                    ResolutionStatus::Adopted
                } else {
                    ResolutionStatus::Rejected
                }
            }
            MajorityType::Unanimity => {
                // Art. 3.88 §1, 3°: 100% de TOUS les tantièmes (pas juste les présents)
                // total_voting_power = total building tantièmes (e.g. 10000)
                if total_voting_power > Decimal::ZERO
                    && (self.total_voting_power_pour - total_voting_power).abs() < dec!(0.01)
                {
                    ResolutionStatus::Adopted
                } else {
                    ResolutionStatus::Rejected
                }
            }
        }
    }

    /// Clôture le vote et finalise le statut
    pub fn close_voting(&mut self, total_voting_power: Decimal) -> Result<(), String> {
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
            MajorityType::Absolute,
            Some(0),
        );

        assert!(resolution.is_ok());
        let resolution = resolution.unwrap();
        assert_eq!(resolution.meeting_id, meeting_id);
        assert_eq!(resolution.status, ResolutionStatus::Pending);
        assert_eq!(resolution.total_votes(), 0);
        assert_eq!(resolution.agenda_item_index, Some(0));
    }

    #[test]
    fn test_create_resolution_without_agenda_item() {
        let meeting_id = Uuid::new_v4();
        let resolution = Resolution::new(
            meeting_id,
            "Approbation des comptes 2024".to_string(),
            "Vote pour approuver les comptes annuels de l'exercice 2024".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            None,
        );

        assert!(resolution.is_ok());
        let resolution = resolution.unwrap();
        assert_eq!(resolution.agenda_item_index, None);
    }

    #[test]
    fn test_create_resolution_empty_title_fails() {
        let meeting_id = Uuid::new_v4();
        let resolution = Resolution::new(
            meeting_id,
            "".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            Some(0),
        );

        assert!(resolution.is_err());
        assert_eq!(resolution.unwrap_err(), "Resolution title cannot be empty");
    }

    #[test]
    fn test_record_votes() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            Some(0),
        )
        .unwrap();

        resolution.record_vote_pour(dec!(100));
        resolution.record_vote_pour(dec!(150));
        resolution.record_vote_contre(dec!(200));
        resolution.record_abstention(dec!(50));

        assert_eq!(resolution.vote_count_pour, 2);
        assert_eq!(resolution.vote_count_contre, 1);
        assert_eq!(resolution.vote_count_abstention, 1);
        assert_eq!(resolution.total_voting_power_pour, dec!(250));
        assert_eq!(resolution.total_voting_power_contre, dec!(200));
        assert_eq!(resolution.total_voting_power_abstention, dec!(50));
        assert_eq!(resolution.total_votes(), 4);
    }

    // ===== Absolute majority (Art. 3.88 §1) — abstentions excluded =====

    #[test]
    fn test_calculate_result_absolute_majority_adopted() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            Some(0),
        )
        .unwrap();

        // Pour=300, Contre=150 → expressed=450, 300 > 225 → Adopted
        resolution.record_vote_pour(dec!(300));
        resolution.record_vote_contre(dec!(150));
        resolution.record_abstention(dec!(50));

        let result = resolution.calculate_result(dec!(1000));
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
            Some(0),
        )
        .unwrap();

        // Pour=150, Contre=300 → expressed=450, 150 < 225 → Rejected
        resolution.record_vote_pour(dec!(150));
        resolution.record_vote_contre(dec!(300));
        resolution.record_abstention(dec!(50));

        let result = resolution.calculate_result(dec!(1000));
        assert_eq!(result, ResolutionStatus::Rejected);
    }

    #[test]
    fn test_absolute_majority_abstentions_excluded() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            Some(0),
        )
        .unwrap();

        // Pour=300, Contre=200, Abstention=500 → expressed=500, 300 > 250 → Adopted
        // Abstentions are excluded: 300 is more than half of (300+200)
        resolution.record_vote_pour(dec!(300));
        resolution.record_vote_contre(dec!(200));
        resolution.record_abstention(dec!(500));

        let result = resolution.calculate_result(dec!(1000));
        assert_eq!(result, ResolutionStatus::Adopted);
    }

    // ===== Two-thirds majority (Art. 3.88 §1, 1°) — abstentions excluded =====

    #[test]
    fn test_calculate_result_two_thirds_majority_adopted() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::TwoThirds,
            Some(0),
        )
        .unwrap();

        // Pour=700, Contre=200 → expressed=900, 700/900 = 77.8% >= 66.7% → Adopted
        resolution.record_vote_pour(dec!(700));
        resolution.record_vote_contre(dec!(200));
        resolution.record_abstention(dec!(100));

        let result = resolution.calculate_result(dec!(1000));
        assert_eq!(result, ResolutionStatus::Adopted);
    }

    #[test]
    fn test_calculate_result_two_thirds_majority_rejected() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::TwoThirds,
            Some(0),
        )
        .unwrap();

        // Pour=600, Contre=300 → expressed=900, 600/900 = 66.7% >= 66.7% → Adopted (boundary)
        // Actually 600/900 = 0.6667 which is >= 2/3 = 0.6667 → Adopted
        resolution.record_vote_pour(dec!(600));
        resolution.record_vote_contre(dec!(300));
        resolution.record_abstention(dec!(100));

        let result = resolution.calculate_result(dec!(1000));
        assert_eq!(result, ResolutionStatus::Adopted);
    }

    #[test]
    fn test_two_thirds_majority_barely_rejected() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::TwoThirds,
            Some(0),
        )
        .unwrap();

        // Pour=500, Contre=300 → expressed=800, 500/800 = 62.5% < 66.7% → Rejected
        resolution.record_vote_pour(dec!(500));
        resolution.record_vote_contre(dec!(300));
        resolution.record_abstention(dec!(200));

        let result = resolution.calculate_result(dec!(1000));
        assert_eq!(result, ResolutionStatus::Rejected);
    }

    #[test]
    fn test_two_thirds_abstentions_excluded() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::TwoThirds,
            Some(0),
        )
        .unwrap();

        // Pour=400, Contre=100, Abstention=500 → expressed=500, 400/500 = 80% >= 66.7% → Adopted
        resolution.record_vote_pour(dec!(400));
        resolution.record_vote_contre(dec!(100));
        resolution.record_abstention(dec!(500));

        let result = resolution.calculate_result(dec!(1000));
        assert_eq!(result, ResolutionStatus::Adopted);
    }

    // ===== Four-fifths majority (Art. 3.88 §1, 2°) — abstentions excluded =====

    #[test]
    fn test_calculate_result_four_fifths_majority_adopted() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::FourFifths,
            Some(0),
        )
        .unwrap();

        // Pour=800, Contre=100 → expressed=900, 800/900 = 88.9% >= 80% → Adopted
        resolution.record_vote_pour(dec!(800));
        resolution.record_vote_contre(dec!(100));
        resolution.record_abstention(dec!(100));

        let result = resolution.calculate_result(dec!(1000));
        assert_eq!(result, ResolutionStatus::Adopted);
    }

    #[test]
    fn test_calculate_result_four_fifths_majority_rejected() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::FourFifths,
            Some(0),
        )
        .unwrap();

        // Pour=700, Contre=200 → expressed=900, 700/900 = 77.8% < 80% → Rejected
        resolution.record_vote_pour(dec!(700));
        resolution.record_vote_contre(dec!(200));
        resolution.record_abstention(dec!(100));

        let result = resolution.calculate_result(dec!(1000));
        assert_eq!(result, ResolutionStatus::Rejected);
    }

    #[test]
    fn test_four_fifths_abstentions_excluded() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::FourFifths,
            Some(0),
        )
        .unwrap();

        // Pour=400, Contre=50, Abstention=550 → expressed=450, 400/450 = 88.9% >= 80% → Adopted
        resolution.record_vote_pour(dec!(400));
        resolution.record_vote_contre(dec!(50));
        resolution.record_abstention(dec!(550));

        let result = resolution.calculate_result(dec!(1000));
        assert_eq!(result, ResolutionStatus::Adopted);
    }

    // ===== Unanimity (Art. 3.88 §1, 3°) — requires ALL tantièmes =====

    #[test]
    fn test_calculate_result_unanimity_adopted() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::Unanimity,
            Some(0),
        )
        .unwrap();

        // Pour=10000 == total_voting_power → Adopted
        resolution.record_vote_pour(dec!(10000));

        let result = resolution.calculate_result(dec!(10000));
        assert_eq!(result, ResolutionStatus::Adopted);
    }

    #[test]
    fn test_calculate_result_unanimity_rejected_missing_votes() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::Unanimity,
            Some(0),
        )
        .unwrap();

        // Pour=9000 < total_voting_power=10000 (absent owners not accounted for) → Rejected
        resolution.record_vote_pour(dec!(9000));

        let result = resolution.calculate_result(dec!(10000));
        assert_eq!(result, ResolutionStatus::Rejected);
    }

    #[test]
    fn test_unanimity_requires_all_tantiemes_not_just_present() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::Unanimity,
            Some(0),
        )
        .unwrap();

        // All present vote Pour but some owners are absent
        // Pour=8000, Contre=0, Abstention=0, but total building = 10000 → Rejected
        resolution.record_vote_pour(dec!(8000));

        let result = resolution.calculate_result(dec!(10000));
        assert_eq!(result, ResolutionStatus::Rejected);
    }

    #[test]
    fn test_unanimity_rejected_with_abstention() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Extraordinary,
            MajorityType::Unanimity,
            Some(0),
        )
        .unwrap();

        // Pour=9500, Abstention=500 → Pour != total → Rejected (abstentions count as NOT pour)
        resolution.record_vote_pour(dec!(9500));
        resolution.record_abstention(dec!(500));

        let result = resolution.calculate_result(dec!(10000));
        assert_eq!(result, ResolutionStatus::Rejected);
    }

    // ===== Close voting =====

    #[test]
    fn test_close_voting_success() {
        let meeting_id = Uuid::new_v4();
        let mut resolution = Resolution::new(
            meeting_id,
            "Test Resolution".to_string(),
            "Description".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            Some(0),
        )
        .unwrap();

        resolution.record_vote_pour(dec!(300));
        resolution.record_vote_contre(dec!(150));

        let result = resolution.close_voting(dec!(1000));
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
            MajorityType::Absolute,
            Some(0),
        )
        .unwrap();

        resolution.record_vote_pour(dec!(300));
        resolution.close_voting(dec!(1000)).unwrap();

        let result = resolution.close_voting(dec!(1000));
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
            MajorityType::Absolute,
            Some(0),
        )
        .unwrap();

        resolution.record_vote_pour(dec!(100));
        resolution.record_vote_pour(dec!(100)); // 2 votes pour
        resolution.record_vote_contre(dec!(100)); // 1 vote contre
        resolution.record_abstention(dec!(100)); // 1 abstention

        assert_eq!(resolution.pour_percentage(), 50.0); // 2/4 = 50%
        assert_eq!(resolution.contre_percentage(), 25.0); // 1/4 = 25%
        assert_eq!(resolution.abstention_percentage(), 25.0); // 1/4 = 25%
    }
}
