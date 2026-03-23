use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Service Provider (Contractor) — Marketplace
/// Issue #276: Marketplace corps de métier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeCategory {
    Syndic,
    BureauEtude,
    Architecte,
    AssistantMaitreOeuvre,
    IngenieurStabilite,
    Plombier,
    Electricien,
    Chauffagiste,
    Menuisier,
    Peintre,
    Maconnerie,
    Etancheite,
    Ascensoriste,
    Jardinier,
    Nettoyage,
    Securite,
    Deboucheur,
    Couvreur,
    Carreleur,
    TechniquesSpeciales,
}

impl TradeCategory {
    pub fn to_sql(&self) -> &'static str {
        match self {
            TradeCategory::Syndic => "Syndic",
            TradeCategory::BureauEtude => "BureauEtude",
            TradeCategory::Architecte => "Architecte",
            TradeCategory::AssistantMaitreOeuvre => "AssistantMaitreOeuvre",
            TradeCategory::IngenieurStabilite => "IngenieurStabilite",
            TradeCategory::Plombier => "Plombier",
            TradeCategory::Electricien => "Electricien",
            TradeCategory::Chauffagiste => "Chauffagiste",
            TradeCategory::Menuisier => "Menuisier",
            TradeCategory::Peintre => "Peintre",
            TradeCategory::Maconnerie => "Maconnerie",
            TradeCategory::Etancheite => "Etancheite",
            TradeCategory::Ascensoriste => "Ascensoriste",
            TradeCategory::Jardinier => "Jardinier",
            TradeCategory::Nettoyage => "Nettoyage",
            TradeCategory::Securite => "Securite",
            TradeCategory::Deboucheur => "Deboucheur",
            TradeCategory::Couvreur => "Couvreur",
            TradeCategory::Carreleur => "Carreleur",
            TradeCategory::TechniquesSpeciales => "TechniquesSpeciales",
        }
    }

    pub fn from_sql(s: &str) -> Result<Self, String> {
        match s {
            "Syndic" => Ok(TradeCategory::Syndic),
            "BureauEtude" => Ok(TradeCategory::BureauEtude),
            "Architecte" => Ok(TradeCategory::Architecte),
            "AssistantMaitreOeuvre" => Ok(TradeCategory::AssistantMaitreOeuvre),
            "IngenieurStabilite" => Ok(TradeCategory::IngenieurStabilite),
            "Plombier" => Ok(TradeCategory::Plombier),
            "Electricien" => Ok(TradeCategory::Electricien),
            "Chauffagiste" => Ok(TradeCategory::Chauffagiste),
            "Menuisier" => Ok(TradeCategory::Menuisier),
            "Peintre" => Ok(TradeCategory::Peintre),
            "Maconnerie" => Ok(TradeCategory::Maconnerie),
            "Etancheite" => Ok(TradeCategory::Etancheite),
            "Ascensoriste" => Ok(TradeCategory::Ascensoriste),
            "Jardinier" => Ok(TradeCategory::Jardinier),
            "Nettoyage" => Ok(TradeCategory::Nettoyage),
            "Securite" => Ok(TradeCategory::Securite),
            "Deboucheur" => Ok(TradeCategory::Deboucheur),
            "Couvreur" => Ok(TradeCategory::Couvreur),
            "Carreleur" => Ok(TradeCategory::Carreleur),
            "TechniquesSpeciales" => Ok(TradeCategory::TechniquesSpeciales),
            _ => Err(format!("Invalid trade category: {}", s)),
        }
    }
}

pub struct ServiceProvider {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub company_name: String,
    pub trade_category: TradeCategory,
    pub specializations: Vec<String>,
    pub service_zone_postal_codes: Vec<String>,
    pub certifications: Vec<String>, // VCA, Saber, BOSEC, etc.
    pub ipi_registration: Option<String>,
    pub bce_number: Option<String>, // Belgian company number (BCE/KBO)
    pub rating_avg: Option<f64>,     // 0.0-5.0
    pub reviews_count: i32,
    pub is_verified: bool,
    pub public_profile_slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ServiceProvider {
    pub fn new(
        organization_id: Uuid,
        company_name: String,
        trade_category: TradeCategory,
        bce_number: Option<String>,
    ) -> Result<Self, String> {
        if company_name.is_empty() {
            return Err("company_name cannot be empty".to_string());
        }
        let slug = generate_slug(&company_name);
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            company_name,
            trade_category,
            specializations: vec![],
            service_zone_postal_codes: vec![],
            certifications: vec![],
            ipi_registration: None,
            bce_number,
            rating_avg: None,
            reviews_count: 0,
            is_verified: false,
            public_profile_slug: slug,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Update rating based on new evaluation
    pub fn update_rating(&mut self, new_score: f64) -> Result<(), String> {
        if new_score < 0.0 || new_score > 5.0 {
            return Err("Rating must be between 0.0 and 5.0".to_string());
        }
        let current_avg = self.rating_avg.unwrap_or(0.0);
        let new_reviews_count = self.reviews_count + 1;
        self.rating_avg = Some((current_avg * self.reviews_count as f64 + new_score) / new_reviews_count as f64);
        self.reviews_count = new_reviews_count;
        self.updated_at = Utc::now();
        Ok(())
    }
}

fn generate_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_provider_new_success() {
        let org_id = Uuid::new_v4();
        let provider = ServiceProvider::new(
            org_id,
            "ABC Plomberie".to_string(),
            TradeCategory::Plombier,
            Some("BE123456789".to_string()),
        );
        assert!(provider.is_ok());
        let p = provider.unwrap();
        assert_eq!(p.company_name, "ABC Plomberie");
        assert_eq!(p.public_profile_slug, "abc-plomberie");
        assert_eq!(p.reviews_count, 0);
        assert_eq!(p.rating_avg, None);
    }

    #[test]
    fn test_service_provider_empty_name() {
        let org_id = Uuid::new_v4();
        let result = ServiceProvider::new(org_id, "".to_string(), TradeCategory::Plombier, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_rating() {
        let org_id = Uuid::new_v4();
        let mut provider = ServiceProvider::new(
            org_id,
            "Test Provider".to_string(),
            TradeCategory::Electricien,
            None,
        )
        .unwrap();

        let _ = provider.update_rating(4.0);
        assert_eq!(provider.reviews_count, 1);
        assert_eq!(provider.rating_avg, Some(4.0));

        let _ = provider.update_rating(5.0);
        assert_eq!(provider.reviews_count, 2);
        assert_eq!(provider.rating_avg, Some(4.5));
    }

    #[test]
    fn test_update_rating_invalid() {
        let org_id = Uuid::new_v4();
        let mut provider = ServiceProvider::new(
            org_id,
            "Test Provider".to_string(),
            TradeCategory::Electricien,
            None,
        )
        .unwrap();

        let result = provider.update_rating(6.0);
        assert!(result.is_err());
    }
}
