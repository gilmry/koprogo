use crate::domain::entities::{Quote, QuoteScore};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Create new quote request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuoteDto {
    pub building_id: String,
    pub contractor_id: String,
    pub project_title: String,
    pub project_description: String,
    pub amount_excl_vat: Decimal,
    pub vat_rate: Decimal,
    pub validity_date: String, // ISO 8601 string
    pub estimated_start_date: Option<String>,
    pub estimated_duration_days: i32,
    pub warranty_years: i32,
}

/// Update quote details DTO (contractor submission)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQuoteDto {
    pub amount_excl_vat: Option<Decimal>,
    pub vat_rate: Option<Decimal>,
    pub estimated_start_date: Option<String>,
    pub estimated_duration_days: Option<i32>,
    pub warranty_years: Option<i32>,
}

/// Quote decision DTO (Syndic accept/reject)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteDecisionDto {
    pub decision_notes: Option<String>,
}

/// Quote comparison request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteComparisonRequestDto {
    pub quote_ids: Vec<String>, // At least 3 quotes (Belgian law)
}

/// Quote response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponseDto {
    pub id: String,
    pub building_id: String,
    pub contractor_id: String,
    pub project_title: String,
    pub project_description: String,

    // Quote details
    pub amount_excl_vat: String, // Decimal as string
    pub vat_rate: String,        // Decimal as string
    pub amount_incl_vat: String, // Decimal as string
    pub validity_date: String,
    pub estimated_start_date: Option<String>,
    pub estimated_duration_days: i32,

    // Scoring factors
    pub warranty_years: i32,
    pub contractor_rating: Option<i32>,

    // Status
    pub status: String,
    pub is_expired: bool,

    // Workflow metadata
    pub requested_at: String,
    pub submitted_at: Option<String>,
    pub reviewed_at: Option<String>,
    pub decision_at: Option<String>,
    pub decision_by: Option<String>,
    pub decision_notes: Option<String>,

    // Audit trail
    pub created_at: String,
    pub updated_at: String,
}

impl From<Quote> for QuoteResponseDto {
    fn from(quote: Quote) -> Self {
        Self {
            id: quote.id.to_string(),
            building_id: quote.building_id.to_string(),
            contractor_id: quote.contractor_id.to_string(),
            project_title: quote.project_title.clone(),
            project_description: quote.project_description.clone(),
            amount_excl_vat: format!("{:.2}", quote.amount_excl_vat),
            vat_rate: format!("{:.2}", quote.vat_rate),
            amount_incl_vat: format!("{:.2}", quote.amount_incl_vat),
            validity_date: quote.validity_date.to_rfc3339(),
            estimated_start_date: quote.estimated_start_date.map(|d| d.to_rfc3339()),
            estimated_duration_days: quote.estimated_duration_days,
            warranty_years: quote.warranty_years,
            contractor_rating: quote.contractor_rating,
            status: quote.status.to_sql().to_string(),
            is_expired: quote.is_expired(),
            requested_at: quote.requested_at.to_rfc3339(),
            submitted_at: quote.submitted_at.map(|d| d.to_rfc3339()),
            reviewed_at: quote.reviewed_at.map(|d| d.to_rfc3339()),
            decision_at: quote.decision_at.map(|d| d.to_rfc3339()),
            decision_by: quote.decision_by.map(|u| u.to_string()),
            decision_notes: quote.decision_notes,
            created_at: quote.created_at.to_rfc3339(),
            updated_at: quote.updated_at.to_rfc3339(),
        }
    }
}

/// Quote score response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteScoreResponseDto {
    pub quote_id: String,
    pub total_score: f32,
    pub price_score: f32,
    pub delay_score: f32,
    pub warranty_score: f32,
    pub reputation_score: f32,
}

impl From<QuoteScore> for QuoteScoreResponseDto {
    fn from(score: QuoteScore) -> Self {
        Self {
            quote_id: score.quote_id.to_string(),
            total_score: score.total_score,
            price_score: score.price_score,
            delay_score: score.delay_score,
            warranty_score: score.warranty_score,
            reputation_score: score.reputation_score,
        }
    }
}

/// Quote comparison result DTO (includes quote + score)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteComparisonItemDto {
    pub quote: QuoteResponseDto,
    pub score: Option<QuoteScoreResponseDto>,
    pub rank: usize, // 1, 2, 3, etc. (sorted by score)
}

/// Quote comparison response DTO (Belgian professional best practice: 3 quotes minimum)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteComparisonResponseDto {
    pub project_title: String,
    pub building_id: String,
    pub total_quotes: usize,
    pub comparison_items: Vec<QuoteComparisonItemDto>,

    // Aggregated statistics
    pub min_price: String, // Decimal as string
    pub max_price: String,
    pub avg_price: String,
    pub min_duration_days: i32,
    pub max_duration_days: i32,
    pub avg_duration_days: f32,

    // Recommendation (top-ranked quote)
    pub recommended_quote_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Quote;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use uuid::Uuid;

    // Helper macro since dec! is not available in rust_decimal 1.36
    macro_rules! dec {
        ($val:expr) => {
            Decimal::from_str(stringify!($val)).unwrap()
        };
    }

    #[test]
    fn test_quote_response_dto_conversion() {
        let building_id = Uuid::new_v4();
        let contractor_id = Uuid::new_v4();
        let validity_date = Utc::now() + chrono::Duration::days(30);

        let quote = Quote::new(
            building_id,
            contractor_id,
            "Roof Repair".to_string(),
            "Repair leaking roof tiles".to_string(),
            dec!(5000.00),
            dec!(0.21),
            validity_date,
            14,
            10,
        )
        .unwrap();

        let dto = QuoteResponseDto::from(quote.clone());

        assert_eq!(dto.id, quote.id.to_string());
        assert_eq!(dto.project_title, "Roof Repair");
        assert_eq!(dto.amount_excl_vat, "5000.00");
        assert_eq!(dto.amount_incl_vat, "6050.00");
        assert_eq!(dto.status, "Requested");
        assert!(!dto.is_expired);
        assert_eq!(dto.estimated_duration_days, 14);
        assert_eq!(dto.warranty_years, 10);
    }

    #[test]
    fn test_quote_score_dto_conversion() {
        let quote_id = Uuid::new_v4();
        let score = QuoteScore {
            quote_id,
            total_score: 75.5,
            price_score: 80.0,
            delay_score: 70.0,
            warranty_score: 90.0,
            reputation_score: 60.0,
        };

        let dto = QuoteScoreResponseDto::from(score.clone());

        assert_eq!(dto.quote_id, quote_id.to_string());
        assert_eq!(dto.total_score, 75.5);
        assert_eq!(dto.price_score, 80.0);
        assert_eq!(dto.delay_score, 70.0);
        assert_eq!(dto.warranty_score, 90.0);
        assert_eq!(dto.reputation_score, 60.0);
    }
}
