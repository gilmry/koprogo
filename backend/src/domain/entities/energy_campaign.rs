use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Campagne d'achat groupé d'énergie
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnergyCampaign {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Option<Uuid>, // NULL si multi-buildings

    // Méta
    pub campaign_name: String,
    pub campaign_type: CampaignType,
    pub status: CampaignStatus,

    // Timeline
    pub deadline_participation: DateTime<Utc>,
    pub deadline_vote: Option<DateTime<Utc>>,
    pub contract_start_date: Option<DateTime<Utc>>,

    // Configuration
    pub energy_types: Vec<EnergyType>,
    pub contract_duration_months: i32, // 12, 24, 36
    pub contract_type: ContractType,   // Fixed, Variable

    // Agrégation (données anonymes)
    pub total_participants: i32,
    pub total_kwh_electricity: Option<f64>,
    pub total_kwh_gas: Option<f64>,
    pub avg_kwh_per_unit: Option<f64>,

    // Résultats négociation
    pub offers_received: Vec<ProviderOffer>,
    pub selected_offer_id: Option<Uuid>,
    pub estimated_savings_pct: Option<f64>,

    // Audit
    pub created_by: Uuid, // User ID (syndic)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CampaignType {
    BuyingGroup,      // Achat groupé classique
    CollectiveSwitch, // Switch collectif
}

impl std::fmt::Display for CampaignType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CampaignType::BuyingGroup => write!(f, "BuyingGroup"),
            CampaignType::CollectiveSwitch => write!(f, "CollectiveSwitch"),
        }
    }
}

impl std::str::FromStr for CampaignType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BuyingGroup" => Ok(CampaignType::BuyingGroup),
            "CollectiveSwitch" => Ok(CampaignType::CollectiveSwitch),
            _ => Err(format!("Invalid campaign type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CampaignStatus {
    Draft,             // En préparation
    AwaitingAGVote,    // En attente vote AG
    CollectingData,    // Collecte factures
    Negotiating,       // Négociation courtier
    AwaitingFinalVote, // Vote final offre
    Finalized,         // Switch en cours
    Completed,         // Contrats actifs
    Cancelled,         // Annulée
}

impl std::fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CampaignStatus::Draft => write!(f, "Draft"),
            CampaignStatus::AwaitingAGVote => write!(f, "AwaitingAGVote"),
            CampaignStatus::CollectingData => write!(f, "CollectingData"),
            CampaignStatus::Negotiating => write!(f, "Negotiating"),
            CampaignStatus::AwaitingFinalVote => write!(f, "AwaitingFinalVote"),
            CampaignStatus::Finalized => write!(f, "Finalized"),
            CampaignStatus::Completed => write!(f, "Completed"),
            CampaignStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::str::FromStr for CampaignStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Draft" => Ok(CampaignStatus::Draft),
            "AwaitingAGVote" => Ok(CampaignStatus::AwaitingAGVote),
            "CollectingData" => Ok(CampaignStatus::CollectingData),
            "Negotiating" => Ok(CampaignStatus::Negotiating),
            "AwaitingFinalVote" => Ok(CampaignStatus::AwaitingFinalVote),
            "Finalized" => Ok(CampaignStatus::Finalized),
            "Completed" => Ok(CampaignStatus::Completed),
            "Cancelled" => Ok(CampaignStatus::Cancelled),
            _ => Err(format!("Invalid campaign status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnergyType {
    Electricity,
    Gas,
    Both,
}

impl std::fmt::Display for EnergyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnergyType::Electricity => write!(f, "Electricity"),
            EnergyType::Gas => write!(f, "Gas"),
            EnergyType::Both => write!(f, "Both"),
        }
    }
}

impl std::str::FromStr for EnergyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Electricity" => Ok(EnergyType::Electricity),
            "Gas" => Ok(EnergyType::Gas),
            "Both" => Ok(EnergyType::Both),
            _ => Err(format!("Invalid energy type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractType {
    Fixed,    // Prix fixe
    Variable, // Prix variable (indexé)
}

impl std::fmt::Display for ContractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractType::Fixed => write!(f, "Fixed"),
            ContractType::Variable => write!(f, "Variable"),
        }
    }
}

impl std::str::FromStr for ContractType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Fixed" => Ok(ContractType::Fixed),
            "Variable" => Ok(ContractType::Variable),
            _ => Err(format!("Invalid contract type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderOffer {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub provider_name: String,
    pub price_kwh_electricity: Option<f64>,
    pub price_kwh_gas: Option<f64>,
    pub fixed_monthly_fee: f64,
    pub green_energy_pct: f64, // 0-100
    pub contract_duration_months: i32,
    pub estimated_savings_pct: f64,
    pub offer_valid_until: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProviderOffer {
    /// Créer nouvelle offre fournisseur
    pub fn new(
        campaign_id: Uuid,
        provider_name: String,
        price_kwh_electricity: Option<f64>,
        price_kwh_gas: Option<f64>,
        fixed_monthly_fee: f64,
        green_energy_pct: f64,
        contract_duration_months: i32,
        estimated_savings_pct: f64,
        offer_valid_until: DateTime<Utc>,
    ) -> Result<Self, String> {
        if provider_name.trim().is_empty() {
            return Err("Provider name cannot be empty".to_string());
        }

        if green_energy_pct < 0.0 || green_energy_pct > 100.0 {
            return Err("Green energy percentage must be between 0 and 100".to_string());
        }

        if contract_duration_months <= 0 {
            return Err("Contract duration must be positive".to_string());
        }

        if offer_valid_until <= Utc::now() {
            return Err("Offer validity date must be in the future".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            campaign_id,
            provider_name,
            price_kwh_electricity,
            price_kwh_gas,
            fixed_monthly_fee,
            green_energy_pct,
            contract_duration_months,
            estimated_savings_pct,
            offer_valid_until,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Calculer score vert (pour nudge behavioral)
    pub fn green_score(&self) -> i32 {
        if self.green_energy_pct >= 100.0 {
            10
        } else if self.green_energy_pct >= 50.0 {
            5
        } else {
            0
        }
    }
}

impl EnergyCampaign {
    /// Créer nouvelle campagne
    pub fn new(
        organization_id: Uuid,
        building_id: Option<Uuid>,
        campaign_name: String,
        deadline_participation: DateTime<Utc>,
        energy_types: Vec<EnergyType>,
        created_by: Uuid,
    ) -> Result<Self, String> {
        if campaign_name.trim().is_empty() {
            return Err("Campaign name cannot be empty".to_string());
        }

        if energy_types.is_empty() {
            return Err("At least one energy type required".to_string());
        }

        if deadline_participation <= Utc::now() {
            return Err("Deadline must be in the future".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            campaign_name,
            campaign_type: CampaignType::BuyingGroup,
            status: CampaignStatus::Draft,
            deadline_participation,
            deadline_vote: None,
            contract_start_date: None,
            energy_types,
            contract_duration_months: 12,
            contract_type: ContractType::Fixed,
            total_participants: 0,
            total_kwh_electricity: None,
            total_kwh_gas: None,
            avg_kwh_per_unit: None,
            offers_received: Vec::new(),
            selected_offer_id: None,
            estimated_savings_pct: None,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Lancer collecte données (après vote AG)
    pub fn start_data_collection(&mut self) -> Result<(), String> {
        if self.status != CampaignStatus::AwaitingAGVote {
            return Err("Campaign must be in AwaitingAGVote status".to_string());
        }

        self.status = CampaignStatus::CollectingData;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Calculer taux de participation
    pub fn participation_rate(&self, total_units: i32) -> f64 {
        if total_units == 0 {
            return 0.0;
        }
        (self.total_participants as f64 / total_units as f64) * 100.0
    }

    /// Vérifier si éligible négociation (min 60% participation)
    pub fn can_negotiate(&self, total_units: i32) -> bool {
        self.participation_rate(total_units) >= 60.0
    }

    /// Ajouter une offre fournisseur
    pub fn add_offer(&mut self, offer: ProviderOffer) -> Result<(), String> {
        if self.status != CampaignStatus::Negotiating {
            return Err("Campaign must be in Negotiating status".to_string());
        }

        self.offers_received.push(offer);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Sélectionner offre gagnante
    pub fn select_offer(&mut self, offer_id: Uuid) -> Result<(), String> {
        if self.status != CampaignStatus::AwaitingFinalVote
            && self.status != CampaignStatus::Negotiating
        {
            return Err("Campaign must be in AwaitingFinalVote or Negotiating status".to_string());
        }

        // Vérifier que l'offre existe
        if !self.offers_received.iter().any(|o| o.id == offer_id) {
            return Err("Offer not found in campaign".to_string());
        }

        self.selected_offer_id = Some(offer_id);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Finaliser campagne (après vote final)
    pub fn finalize(&mut self) -> Result<(), String> {
        if self.status != CampaignStatus::AwaitingFinalVote {
            return Err("Campaign must be in AwaitingFinalVote status".to_string());
        }

        if self.selected_offer_id.is_none() {
            return Err("No offer selected".to_string());
        }

        self.status = CampaignStatus::Finalized;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Marquer comme complétée (contrats signés)
    pub fn complete(&mut self) -> Result<(), String> {
        if self.status != CampaignStatus::Finalized {
            return Err("Campaign must be in Finalized status".to_string());
        }

        self.status = CampaignStatus::Completed;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Annuler campagne
    pub fn cancel(&mut self) -> Result<(), String> {
        if self.status == CampaignStatus::Completed || self.status == CampaignStatus::Cancelled {
            return Err("Cannot cancel completed or already cancelled campaign".to_string());
        }

        self.status = CampaignStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_campaign_success() {
        let campaign = EnergyCampaign::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "Campagne Hiver 2025-2026".to_string(),
            Utc::now() + chrono::Duration::days(30),
            vec![EnergyType::Electricity],
            Uuid::new_v4(),
        );

        assert!(campaign.is_ok());
        let campaign = campaign.unwrap();
        assert_eq!(campaign.status, CampaignStatus::Draft);
        assert_eq!(campaign.total_participants, 0);
        assert_eq!(campaign.contract_duration_months, 12);
    }

    #[test]
    fn test_create_campaign_empty_name() {
        let result = EnergyCampaign::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "".to_string(),
            Utc::now() + chrono::Duration::days(30),
            vec![EnergyType::Electricity],
            Uuid::new_v4(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Campaign name cannot be empty");
    }

    #[test]
    fn test_create_campaign_no_energy_types() {
        let result = EnergyCampaign::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "Campagne Test".to_string(),
            Utc::now() + chrono::Duration::days(30),
            vec![],
            Uuid::new_v4(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "At least one energy type required");
    }

    #[test]
    fn test_create_campaign_deadline_in_past() {
        let result = EnergyCampaign::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "Campagne Test".to_string(),
            Utc::now() - chrono::Duration::days(1),
            vec![EnergyType::Electricity],
            Uuid::new_v4(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Deadline must be in the future");
    }

    #[test]
    fn test_participation_rate() {
        let mut campaign = EnergyCampaign::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "Campagne Test".to_string(),
            Utc::now() + chrono::Duration::days(30),
            vec![EnergyType::Electricity],
            Uuid::new_v4(),
        )
        .unwrap();

        campaign.total_participants = 18;
        let rate = campaign.participation_rate(25);
        assert_eq!(rate, 72.0);
    }

    #[test]
    fn test_can_negotiate() {
        let mut campaign = EnergyCampaign::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "Campagne Test".to_string(),
            Utc::now() + chrono::Duration::days(30),
            vec![EnergyType::Electricity],
            Uuid::new_v4(),
        )
        .unwrap();

        campaign.total_participants = 15; // 60% de 25
        assert!(campaign.can_negotiate(25));

        campaign.total_participants = 14; // 56% de 25
        assert!(!campaign.can_negotiate(25));
    }

    #[test]
    fn test_provider_offer_creation() {
        let offer = ProviderOffer::new(
            Uuid::new_v4(),
            "Lampiris".to_string(),
            Some(0.27),
            None,
            12.50,
            100.0,
            12,
            15.0,
            Utc::now() + chrono::Duration::days(30),
        );

        assert!(offer.is_ok());
        let offer = offer.unwrap();
        assert_eq!(offer.provider_name, "Lampiris");
        assert_eq!(offer.green_score(), 10);
    }

    #[test]
    fn test_green_score() {
        let offer_100 = ProviderOffer::new(
            Uuid::new_v4(),
            "Lampiris".to_string(),
            Some(0.27),
            None,
            12.50,
            100.0,
            12,
            15.0,
            Utc::now() + chrono::Duration::days(30),
        )
        .unwrap();
        assert_eq!(offer_100.green_score(), 10);

        let offer_75 = ProviderOffer::new(
            Uuid::new_v4(),
            "Engie".to_string(),
            Some(0.25),
            None,
            12.50,
            75.0,
            12,
            18.0,
            Utc::now() + chrono::Duration::days(30),
        )
        .unwrap();
        assert_eq!(offer_75.green_score(), 5);

        let offer_30 = ProviderOffer::new(
            Uuid::new_v4(),
            "Luminus".to_string(),
            Some(0.26),
            None,
            12.50,
            30.0,
            12,
            16.0,
            Utc::now() + chrono::Duration::days(30),
        )
        .unwrap();
        assert_eq!(offer_30.green_score(), 0);
    }

    #[test]
    fn test_workflow_state_machine() {
        let mut campaign = EnergyCampaign::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "Campagne Test".to_string(),
            Utc::now() + chrono::Duration::days(30),
            vec![EnergyType::Electricity],
            Uuid::new_v4(),
        )
        .unwrap();

        // Draft → AwaitingAGVote
        campaign.status = CampaignStatus::AwaitingAGVote;

        // AwaitingAGVote → CollectingData
        assert!(campaign.start_data_collection().is_ok());
        assert_eq!(campaign.status, CampaignStatus::CollectingData);

        // CollectingData → Negotiating
        campaign.status = CampaignStatus::Negotiating;

        // Ajouter offre
        let offer = ProviderOffer::new(
            campaign.id,
            "Lampiris".to_string(),
            Some(0.27),
            None,
            12.50,
            100.0,
            12,
            15.0,
            Utc::now() + chrono::Duration::days(30),
        )
        .unwrap();
        assert!(campaign.add_offer(offer.clone()).is_ok());

        // Negotiating → AwaitingFinalVote
        campaign.status = CampaignStatus::AwaitingFinalVote;

        // Sélectionner offre
        assert!(campaign.select_offer(offer.id).is_ok());
        assert_eq!(campaign.selected_offer_id, Some(offer.id));

        // Finaliser
        assert!(campaign.finalize().is_ok());
        assert_eq!(campaign.status, CampaignStatus::Finalized);

        // Compléter
        assert!(campaign.complete().is_ok());
        assert_eq!(campaign.status, CampaignStatus::Completed);
    }

    #[test]
    fn test_cancel_campaign() {
        let mut campaign = EnergyCampaign::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "Campagne Test".to_string(),
            Utc::now() + chrono::Duration::days(30),
            vec![EnergyType::Electricity],
            Uuid::new_v4(),
        )
        .unwrap();

        assert!(campaign.cancel().is_ok());
        assert_eq!(campaign.status, CampaignStatus::Cancelled);

        // Cannot cancel twice
        assert!(campaign.cancel().is_err());
    }
}
