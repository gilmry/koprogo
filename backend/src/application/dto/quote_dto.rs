use crate::domain::entities::{Quote, QuoteScore, QuoteSubmission};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

fn default_warranty_years() -> i32 {
    2
}

/// Create new quote request DTO ("Demander un devis" — request phase only).
///
/// Price fields are optional: the normal path (QuoteList.svelte) never sends
/// them — nobody knows the price yet at request time, it arrives later via
/// `SubmitQuoteDto` (`POST /quotes/{id}/submit`). They're kept here only as
/// a backward-compatible escape hatch for callers that already know the
/// price when requesting (e.g. a syndic manually recording a quote received
/// by phone/email/paper) — units match the domain/DB convention (euros,
/// VAT as a fraction, e.g. 0.21 for 21%), NOT the cents/percentage
/// convention used by `SubmitQuoteDto` below.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuoteDto {
    pub building_id: String,
    pub contractor_id: String,
    pub project_title: String,
    pub project_description: String,
    #[serde(default)]
    pub work_category: Option<String>,
    #[serde(default)]
    pub amount_excl_vat: Option<Decimal>,
    #[serde(default)]
    pub vat_rate: Option<Decimal>,
    #[serde(default)]
    pub validity_date: Option<String>, // ISO 8601 string
    pub estimated_start_date: Option<String>,
    #[serde(default)]
    pub estimated_duration_days: Option<i32>,
    #[serde(default = "default_warranty_years")]
    pub warranty_years: i32,
}

/// Submit quote pricing DTO (contractor's actual quote — `POST /quotes/{id}/submit`).
///
/// Units match the frontend UI directly (`QuoteDetail.svelte`): amount in
/// cents (`ADR-0007` boundary-conversion pattern) and VAT as a percentage
/// (21, not 0.21) — converted to the domain's euros/fraction convention at
/// the use-case boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitQuoteDto {
    pub amount_excl_vat_cents: i64,
    pub vat_rate: Decimal, // percentage, e.g. 21.00
    pub validity_date: String,
    pub estimated_duration_days: i32,
    pub warranty_years: i32,
}

impl SubmitQuoteDto {
    pub fn into_domain(self) -> Result<QuoteSubmission, String> {
        let validity_date = chrono::DateTime::parse_from_rfc3339(&self.validity_date)
            .map_err(|_| "Invalid validity_date format".to_string())?
            .with_timezone(&chrono::Utc);

        Ok(QuoteSubmission {
            amount_excl_vat: Decimal::from(self.amount_excl_vat_cents) / Decimal::from(100),
            vat_rate: self.vat_rate / Decimal::from(100),
            validity_date,
            estimated_duration_days: self.estimated_duration_days,
            warranty_years: self.warranty_years,
        })
    }
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

/// Quote response DTO.
///
/// Amounts are in cents and VAT as a percentage (matches `SubmitQuoteDto`'s
/// wire units, and the frontend `Quote` interface directly) — all `None`
/// until the quote has been submitted with pricing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponseDto {
    pub id: String,
    pub building_id: String,
    pub contractor_id: String,
    pub project_title: String,
    pub project_description: String,
    pub work_category: Option<String>,

    // Quote details — set once the quote has been submitted (Received+).
    pub amount_excl_vat_cents: Option<i64>,
    pub vat_rate: Option<Decimal>, // percentage, e.g. 21.00
    pub amount_incl_vat_cents: Option<i64>,
    pub validity_date: Option<String>,
    pub estimated_start_date: Option<String>,
    pub estimated_duration_days: Option<i32>,

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

/// Decimal euros -> integer cents, rounding to the nearest cent.
fn to_cents(amount: Decimal) -> i64 {
    use rust_decimal::prelude::ToPrimitive;
    (amount * Decimal::from(100)).round().to_i64().unwrap_or(0)
}

impl From<Quote> for QuoteResponseDto {
    fn from(quote: Quote) -> Self {
        let is_expired = quote.is_expired();
        Self {
            id: quote.id.to_string(),
            building_id: quote.building_id.to_string(),
            contractor_id: quote.contractor_id.to_string(),
            project_title: quote.project_title.clone(),
            project_description: quote.project_description.clone(),
            work_category: quote.work_category.clone(),
            amount_excl_vat_cents: quote.amount_excl_vat.map(to_cents),
            vat_rate: quote.vat_rate.map(|r| r * Decimal::from(100)),
            amount_incl_vat_cents: quote.amount_incl_vat.map(to_cents),
            validity_date: quote.validity_date.map(|d| d.to_rfc3339()),
            estimated_start_date: quote.estimated_start_date.map(|d| d.to_rfc3339()),
            estimated_duration_days: quote.estimated_duration_days,
            warranty_years: quote.warranty_years,
            contractor_rating: quote.contractor_rating,
            status: quote.status.to_sql().to_string(),
            is_expired,
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
    fn test_quote_response_dto_conversion_unpriced() {
        let building_id = Uuid::new_v4();
        let contractor_id = Uuid::new_v4();

        let quote = Quote::new(
            building_id,
            contractor_id,
            "Roof Repair".to_string(),
            "Repair leaking roof tiles".to_string(),
            Some("roofing".to_string()),
            10,
        )
        .unwrap();

        let dto = QuoteResponseDto::from(quote.clone());

        assert_eq!(dto.id, quote.id.to_string());
        assert_eq!(dto.project_title, "Roof Repair");
        assert_eq!(dto.amount_excl_vat_cents, None);
        assert_eq!(dto.amount_incl_vat_cents, None);
        assert_eq!(dto.status, "Requested");
        assert!(!dto.is_expired);
        assert_eq!(dto.warranty_years, 10);
    }

    #[test]
    fn test_quote_response_dto_conversion_priced() {
        let building_id = Uuid::new_v4();
        let contractor_id = Uuid::new_v4();
        let validity_date = Utc::now() + chrono::Duration::days(30);

        let mut quote = Quote::new(
            building_id,
            contractor_id,
            "Roof Repair".to_string(),
            "Repair leaking roof tiles".to_string(),
            Some("roofing".to_string()),
            10,
        )
        .unwrap();
        quote
            .submit(Some(QuoteSubmission {
                amount_excl_vat: dec!(5000.00),
                vat_rate: dec!(0.21),
                validity_date,
                estimated_duration_days: 14,
                warranty_years: 10,
            }))
            .unwrap();

        let dto = QuoteResponseDto::from(quote.clone());

        assert_eq!(dto.amount_excl_vat_cents, Some(500_000));
        assert_eq!(dto.amount_incl_vat_cents, Some(605_000));
        assert_eq!(dto.vat_rate, Some(dec!(21.00)));
        assert_eq!(dto.status, "Received");
        assert_eq!(dto.estimated_duration_days, Some(14));
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
