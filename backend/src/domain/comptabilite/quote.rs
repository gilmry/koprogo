use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Quote for contractor work (Belgian professional best practice: 3 quotes for works >5000€)
///
/// 2-phase workflow: a quote is *requested* (title/description/category only —
/// nobody knows the price yet) then *submitted* (contractor's actual pricing,
/// via `submit()`). Price/terms fields are therefore `None` until the quote
/// reaches `QuoteStatus::Received`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quote {
    pub id: Uuid,
    pub building_id: Uuid,
    pub contractor_id: Uuid,
    pub project_title: String,
    pub project_description: String,
    pub work_category: Option<String>,

    // Quote details — set at submission, not at request time.
    pub amount_excl_vat: Option<Decimal>,
    pub vat_rate: Option<Decimal>,
    pub amount_incl_vat: Option<Decimal>,
    pub validity_date: Option<DateTime<Utc>>,
    pub estimated_start_date: Option<DateTime<Utc>>,
    pub estimated_duration_days: Option<i32>,

    // Scoring factors (Belgian best practices)
    pub warranty_years: i32, // 2 years (apparent defects), 10 years (structural)
    pub contractor_rating: Option<i32>, // 0-100 based on history

    // Status & workflow
    pub status: QuoteStatus,
    pub requested_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub decision_at: Option<DateTime<Utc>>,
    pub decision_by: Option<Uuid>, // User who made decision
    pub decision_notes: Option<String>,

    // Audit trail
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub enum QuoteStatus {
    Requested,   // Quote requested from contractor
    Received,    // Contractor submitted quote
    UnderReview, // Syndic reviewing/comparing quotes
    Accepted,    // Quote accepted (winner)
    Rejected,    // Quote rejected (loser or unqualified)
    Expired,     // Validity date passed
    Withdrawn,   // Contractor withdrew quote
}

impl QuoteStatus {
    pub fn to_sql(&self) -> &'static str {
        match self {
            QuoteStatus::Requested => "Requested",
            QuoteStatus::Received => "Received",
            QuoteStatus::UnderReview => "UnderReview",
            QuoteStatus::Accepted => "Accepted",
            QuoteStatus::Rejected => "Rejected",
            QuoteStatus::Expired => "Expired",
            QuoteStatus::Withdrawn => "Withdrawn",
        }
    }

    pub fn from_sql(s: &str) -> Result<Self, String> {
        match s {
            "Requested" => Ok(QuoteStatus::Requested),
            "Received" => Ok(QuoteStatus::Received),
            "UnderReview" => Ok(QuoteStatus::UnderReview),
            "Accepted" => Ok(QuoteStatus::Accepted),
            "Rejected" => Ok(QuoteStatus::Rejected),
            "Expired" => Ok(QuoteStatus::Expired),
            "Withdrawn" => Ok(QuoteStatus::Withdrawn),
            _ => Err(format!("Invalid quote status: {}", s)),
        }
    }
}

/// Automatic scoring result (Belgian best practices)
/// Scoring algorithm: price (40%), delay (30%), warranty (20%), reputation (10%)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuoteScore {
    pub quote_id: Uuid,
    pub total_score: f32,      // 0-100
    pub price_score: f32,      // 0-100 (lower price = higher score)
    pub delay_score: f32,      // 0-100 (shorter delay = higher score)
    pub warranty_score: f32,   // 0-100 (longer warranty = higher score)
    pub reputation_score: f32, // 0-100 (contractor rating)
}

/// Pricing/terms carried by a quote submission — see [`Quote::submit`].
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSubmission {
    pub amount_excl_vat: Decimal,
    pub vat_rate: Decimal,
    pub validity_date: DateTime<Utc>,
    pub estimated_duration_days: i32,
    pub warranty_years: i32,
}

impl Quote {
    /// Create new quote request — request phase only, no pricing (cf. struct docs).
    pub fn new(
        building_id: Uuid,
        contractor_id: Uuid,
        project_title: String,
        project_description: String,
        work_category: Option<String>,
        warranty_years: i32,
    ) -> Result<Self, String> {
        if project_title.is_empty() {
            return Err("Project title cannot be empty".to_string());
        }
        if warranty_years < 0 {
            return Err("Warranty years cannot be negative".to_string());
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            building_id,
            contractor_id,
            project_title,
            project_description,
            work_category,
            amount_excl_vat: None,
            vat_rate: None,
            amount_incl_vat: None,
            validity_date: None,
            estimated_start_date: None,
            estimated_duration_days: None,
            warranty_years,
            contractor_rating: None,
            status: QuoteStatus::Requested,
            requested_at: now,
            submitted_at: None,
            reviewed_at: None,
            decision_at: None,
            decision_by: None,
            decision_notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Submit quote (contractor/syndic action) — moves Requested -> Received.
    ///
    /// `pricing` carries the contractor's actual price/terms and is validated
    /// here (this is where those rules now live, moved from `new()`). Passing
    /// `None` is only valid when the quote already carries price data (e.g.
    /// it was created with pricing already known) — otherwise this errors,
    /// since a `Received` quote without a price makes no sense.
    pub fn submit(&mut self, pricing: Option<QuoteSubmission>) -> Result<(), String> {
        if self.status != QuoteStatus::Requested {
            return Err(format!(
                "Cannot submit quote with status: {:?}",
                self.status
            ));
        }

        match pricing {
            Some(p) => self.apply_pricing(p)?,
            None => {
                if self.amount_excl_vat.is_none() {
                    return Err("Quote has no price data — provide pricing to submit".to_string());
                }
            }
        }

        self.status = QuoteStatus::Received;
        self.submitted_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set pricing on a quote that is still `Requested` (does NOT transition
    /// status — unlike `submit()`). Backward-compat escape hatch for callers
    /// that already know the price when requesting the quote (e.g. a syndic
    /// manually recording a quote received by phone/email/paper): the quote
    /// still goes through the normal `submit()` step afterward.
    pub fn set_initial_pricing(&mut self, pricing: QuoteSubmission) -> Result<(), String> {
        if self.status != QuoteStatus::Requested {
            return Err(format!(
                "Cannot set pricing on quote with status: {:?}",
                self.status
            ));
        }
        self.apply_pricing(pricing)
    }

    fn apply_pricing(&mut self, p: QuoteSubmission) -> Result<(), String> {
        if p.amount_excl_vat <= Decimal::ZERO {
            return Err("Amount must be greater than 0".to_string());
        }
        if p.estimated_duration_days <= 0 {
            return Err("Estimated duration must be greater than 0 days".to_string());
        }
        if p.warranty_years < 0 {
            return Err("Warranty years cannot be negative".to_string());
        }
        if p.validity_date <= Utc::now() {
            return Err("Validity date must be in the future".to_string());
        }

        self.amount_excl_vat = Some(p.amount_excl_vat);
        self.vat_rate = Some(p.vat_rate);
        self.amount_incl_vat = Some(p.amount_excl_vat * (Decimal::ONE + p.vat_rate));
        self.validity_date = Some(p.validity_date);
        self.estimated_duration_days = Some(p.estimated_duration_days);
        self.warranty_years = p.warranty_years;
        Ok(())
    }

    /// Mark quote under review (Syndic action)
    pub fn start_review(&mut self) -> Result<(), String> {
        if self.status != QuoteStatus::Received {
            return Err(format!(
                "Cannot review quote with status: {:?}",
                self.status
            ));
        }
        self.status = QuoteStatus::UnderReview;
        self.reviewed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Accept quote (winning bid)
    pub fn accept(
        &mut self,
        decision_by: Uuid,
        decision_notes: Option<String>,
    ) -> Result<(), String> {
        if self.status != QuoteStatus::UnderReview && self.status != QuoteStatus::Received {
            return Err(format!(
                "Cannot accept quote with status: {:?}",
                self.status
            ));
        }
        if self.is_expired() {
            return Err("Cannot accept expired quote".to_string());
        }
        self.status = QuoteStatus::Accepted;
        self.decision_at = Some(Utc::now());
        self.decision_by = Some(decision_by);
        self.decision_notes = decision_notes;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Reject quote (losing bid or unqualified)
    pub fn reject(
        &mut self,
        decision_by: Uuid,
        decision_notes: Option<String>,
    ) -> Result<(), String> {
        if self.status == QuoteStatus::Accepted {
            return Err("Cannot reject already accepted quote".to_string());
        }
        self.status = QuoteStatus::Rejected;
        self.decision_at = Some(Utc::now());
        self.decision_by = Some(decision_by);
        self.decision_notes = decision_notes;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Withdraw quote (contractor action)
    pub fn withdraw(&mut self) -> Result<(), String> {
        if self.status == QuoteStatus::Accepted {
            return Err("Cannot withdraw accepted quote".to_string());
        }
        if self.status == QuoteStatus::Rejected {
            return Err("Cannot withdraw rejected quote".to_string());
        }
        self.status = QuoteStatus::Withdrawn;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if quote is expired
    pub fn is_expired(&self) -> bool {
        // No validity_date yet (not submitted) means "not expired" — there's
        // nothing to expire before a price/validity has ever been set.
        self.validity_date.is_some_and(|d| Utc::now() > d)
    }

    /// Mark quote as expired (background job)
    pub fn mark_expired(&mut self) -> Result<(), String> {
        if !self.is_expired() {
            return Err("Quote is not yet expired".to_string());
        }
        if self.status == QuoteStatus::Accepted {
            return Err("Cannot expire accepted quote".to_string());
        }
        self.status = QuoteStatus::Expired;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update contractor rating (from historical data)
    pub fn set_contractor_rating(&mut self, rating: i32) -> Result<(), String> {
        if rating < 0 || rating > 100 {
            return Err("Contractor rating must be between 0 and 100".to_string());
        }
        self.contractor_rating = Some(rating);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Calculate automatic score (Belgian best practices)
    /// Algorithm: price (40%), delay (30%), warranty (20%), reputation (10%)
    /// Returns QuoteScore with breakdown
    pub fn calculate_score(
        &self,
        min_price: Decimal,
        max_price: Decimal,
        min_duration: i32,
        max_duration: i32,
        max_warranty: i32,
    ) -> Result<QuoteScore, String> {
        if max_price <= min_price {
            return Err("Invalid price range for scoring".to_string());
        }
        if max_duration <= min_duration {
            return Err("Invalid duration range for scoring".to_string());
        }
        if max_warranty <= 0 {
            return Err("Max warranty must be positive".to_string());
        }
        let amount_incl_vat = self
            .amount_incl_vat
            .ok_or("Quote has no price data (not yet submitted)")?;
        let estimated_duration_days = self
            .estimated_duration_days
            .ok_or("Quote has no price data (not yet submitted)")?;

        // Price score: lower price = higher score (inverted normalization)
        let price_score = if amount_incl_vat <= min_price {
            100.0
        } else if amount_incl_vat >= max_price {
            0.0
        } else {
            let price_range = max_price - min_price;
            let price_delta = max_price - amount_incl_vat;
            (price_delta / price_range * Decimal::from(100))
                .to_f32()
                .unwrap_or(0.0)
        };

        // Delay score: shorter duration = higher score (inverted normalization)
        let delay_score = if estimated_duration_days <= min_duration {
            100.0
        } else if estimated_duration_days >= max_duration {
            0.0
        } else {
            let duration_range = (max_duration - min_duration) as f32;
            let duration_delta = (max_duration - estimated_duration_days) as f32;
            (duration_delta / duration_range) * 100.0
        };

        // Warranty score: longer warranty = higher score (direct normalization)
        let warranty_score = if max_warranty == 0 {
            0.0
        } else {
            ((self.warranty_years as f32 / max_warranty as f32) * 100.0).min(100.0)
        };

        // Reputation score: contractor rating (0-100)
        let reputation_score = self.contractor_rating.unwrap_or(50) as f32;

        // Weighted total score: price (40%), delay (30%), warranty (20%), reputation (10%)
        let total_score = (price_score * 0.4)
            + (delay_score * 0.3)
            + (warranty_score * 0.2)
            + (reputation_score * 0.1);

        Ok(QuoteScore {
            quote_id: self.id,
            total_score,
            price_score,
            delay_score,
            warranty_score,
            reputation_score,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // Helper macro since dec! is not available in rust_decimal 1.36
    macro_rules! dec {
        ($val:expr) => {
            Decimal::from_str(stringify!($val)).unwrap()
        };
    }

    fn test_submission(
        amount: Decimal,
        duration_days: i32,
        warranty_years: i32,
    ) -> QuoteSubmission {
        QuoteSubmission {
            amount_excl_vat: amount,
            vat_rate: dec!(0.21), // 21% VAT (Belgian standard)
            validity_date: Utc::now() + chrono::Duration::days(30),
            estimated_duration_days: duration_days,
            warranty_years,
        }
    }

    #[test]
    fn test_create_quote_success() {
        let building_id = Uuid::new_v4();
        let contractor_id = Uuid::new_v4();

        let quote = Quote::new(
            building_id,
            contractor_id,
            "Roof Repair".to_string(),
            "Repair leaking roof tiles".to_string(),
            Some("roofing".to_string()),
            10, // 10 years warranty (structural work)
        );

        assert!(quote.is_ok());
        let quote = quote.unwrap();
        assert_eq!(quote.status, QuoteStatus::Requested);
        // Request phase carries no pricing yet — that's the whole point.
        assert_eq!(quote.amount_incl_vat, None);
        assert_eq!(quote.estimated_duration_days, None);
        assert_eq!(quote.warranty_years, 10);
    }

    #[test]
    fn test_create_quote_validation_failures() {
        let building_id = Uuid::new_v4();
        let contractor_id = Uuid::new_v4();

        // Empty title
        let result = Quote::new(
            building_id,
            contractor_id,
            "".to_string(),
            "Description".to_string(),
            None,
            10,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Project title cannot be empty");
    }

    #[test]
    fn test_submit_quote_validation_failures() {
        // Zero amount, past validity date, non-positive duration: these
        // validations used to live in `Quote::new()` — they moved to
        // `submit()` along with the pricing data itself.
        let mut quote = create_test_quote();

        let mut zero_amount = test_submission(dec!(0.00), 14, 10);
        zero_amount.amount_excl_vat = dec!(0.00);
        let result = quote.submit(Some(zero_amount));
        assert!(result.is_err());

        let mut past_validity = test_submission(dec!(5000.00), 14, 10);
        past_validity.validity_date = Utc::now() - chrono::Duration::days(1);
        let result = quote.submit(Some(past_validity));
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_without_pricing_requires_existing_price() {
        // A quote created without pricing cannot be bodyless-submitted —
        // there is nothing to persist as its price.
        let mut quote = create_test_quote();
        let result = quote.submit(None);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Quote has no price data — provide pricing to submit"
        );
    }

    #[test]
    fn test_quote_workflow_submit() {
        let mut quote = create_test_quote();
        assert_eq!(quote.status, QuoteStatus::Requested);

        let result = quote.submit(Some(test_submission(dec!(5000.00), 14, 10)));
        assert!(result.is_ok());
        assert_eq!(quote.status, QuoteStatus::Received);
        assert!(quote.submitted_at.is_some());
        assert_eq!(quote.amount_incl_vat, Some(dec!(6050.00))); // 5000 * 1.21
    }

    #[test]
    fn test_quote_workflow_review() {
        let mut quote = create_test_quote();
        quote
            .submit(Some(test_submission(dec!(5000.00), 14, 10)))
            .unwrap();

        let result = quote.start_review();
        assert!(result.is_ok());
        assert_eq!(quote.status, QuoteStatus::UnderReview);
        assert!(quote.reviewed_at.is_some());
    }

    #[test]
    fn test_quote_workflow_accept() {
        let mut quote = create_test_quote();
        quote
            .submit(Some(test_submission(dec!(5000.00), 14, 10)))
            .unwrap();
        quote.start_review().unwrap();

        let decision_by = Uuid::new_v4();
        let result = quote.accept(decision_by, Some("Best value for money".to_string()));
        assert!(result.is_ok());
        assert_eq!(quote.status, QuoteStatus::Accepted);
        assert_eq!(quote.decision_by, Some(decision_by));
        assert_eq!(
            quote.decision_notes,
            Some("Best value for money".to_string())
        );
    }

    #[test]
    fn test_quote_workflow_reject() {
        let mut quote = create_test_quote();
        quote
            .submit(Some(test_submission(dec!(5000.00), 14, 10)))
            .unwrap();

        let decision_by = Uuid::new_v4();
        let result = quote.reject(decision_by, Some("Price too high".to_string()));
        assert!(result.is_ok());
        assert_eq!(quote.status, QuoteStatus::Rejected);
    }

    #[test]
    fn test_quote_cannot_reject_accepted() {
        let mut quote = create_test_quote();
        quote
            .submit(Some(test_submission(dec!(5000.00), 14, 10)))
            .unwrap();
        quote.start_review().unwrap();
        quote.accept(Uuid::new_v4(), None).unwrap();

        let result = quote.reject(Uuid::new_v4(), None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot reject already accepted quote");
    }

    #[test]
    fn test_quote_withdraw() {
        let mut quote = create_test_quote();
        quote
            .submit(Some(test_submission(dec!(5000.00), 14, 10)))
            .unwrap();

        let result = quote.withdraw();
        assert!(result.is_ok());
        assert_eq!(quote.status, QuoteStatus::Withdrawn);
    }

    #[test]
    fn test_quote_scoring_algorithm() {
        let mut quote1 = create_test_quote_with_details(dec!(5000.00), 14, 10, Some(80));
        let mut quote2 = create_test_quote_with_details(dec!(7000.00), 10, 2, Some(90));
        let mut quote3 = create_test_quote_with_details(dec!(6000.00), 12, 5, Some(70));

        quote1
            .submit(Some(test_submission(dec!(5000.00), 14, 10)))
            .unwrap();
        quote2
            .submit(Some(test_submission(dec!(7000.00), 10, 2)))
            .unwrap();
        quote3
            .submit(Some(test_submission(dec!(6000.00), 12, 5)))
            .unwrap();

        // Score with min/max ranges (must use amount_incl_vat since quotes store VAT-included prices)
        // quote1: 5000 * 1.21 = 6050, quote2: 7000 * 1.21 = 8470, quote3: 6000 * 1.21 = 7260
        let score1 = quote1
            .calculate_score(dec!(6050.00), dec!(8470.00), 10, 14, 10)
            .unwrap();
        let score2 = quote2
            .calculate_score(dec!(6050.00), dec!(8470.00), 10, 14, 10)
            .unwrap();
        let score3 = quote3
            .calculate_score(dec!(6050.00), dec!(8470.00), 10, 14, 10)
            .unwrap();

        // Quote1: lowest price (100 * 0.4) + longest delay (0 * 0.3) + best warranty (100 * 0.2) + good reputation (80 * 0.1) = 68
        // Quote2: highest price (0 * 0.4) + shortest delay (100 * 0.3) + low warranty (20 * 0.2) + best reputation (90 * 0.1) = 43
        // Quote3: mid price (50 * 0.4) + mid delay (50 * 0.3) + mid warranty (50 * 0.2) + low reputation (70 * 0.1) = 52

        assert!(score1.total_score > score3.total_score);
        assert!(score3.total_score > score2.total_score);
        assert!(score1.total_score > 60.0); // Quote1 should be best (price + warranty)
    }

    #[test]
    fn test_quote_not_yet_submitted_cannot_be_scored() {
        let quote = create_test_quote();
        let result = quote.calculate_score(dec!(1000.00), dec!(2000.00), 5, 10, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_quote_expiration() {
        let mut quote = create_test_quote();
        // Not yet submitted: no validity_date at all, so definitely not expired.
        assert!(!quote.is_expired());

        quote
            .submit(Some(test_submission(dec!(5000.00), 14, 10)))
            .unwrap();
        assert!(!quote.is_expired());

        // Manually set validity_date to past (simulates time passing).
        quote.validity_date = Some(Utc::now() - chrono::Duration::seconds(1));
        assert!(quote.is_expired());

        let result = quote.mark_expired();
        assert!(result.is_ok());
        assert_eq!(quote.status, QuoteStatus::Expired);
    }

    #[test]
    fn test_contractor_rating_validation() {
        let mut quote = create_test_quote();

        let result = quote.set_contractor_rating(150);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Contractor rating must be between 0 and 100"
        );

        let result = quote.set_contractor_rating(85);
        assert!(result.is_ok());
        assert_eq!(quote.contractor_rating, Some(85));
    }

    // Helper functions

    fn create_test_quote() -> Quote {
        let building_id = Uuid::new_v4();
        let contractor_id = Uuid::new_v4();

        Quote::new(
            building_id,
            contractor_id,
            "Test Project".to_string(),
            "Test Description".to_string(),
            Some("roofing".to_string()),
            10,
        )
        .unwrap()
    }

    fn create_test_quote_with_details(
        amount: Decimal,
        _duration_days: i32,
        warranty_years: i32,
        rating: Option<i32>,
    ) -> Quote {
        let _ = amount; // pricing now set via submit(), not new()
        let building_id = Uuid::new_v4();
        let contractor_id = Uuid::new_v4();

        let mut quote = Quote::new(
            building_id,
            contractor_id,
            "Test Project".to_string(),
            "Test Description".to_string(),
            Some("roofing".to_string()),
            warranty_years,
        )
        .unwrap();

        if let Some(r) = rating {
            quote.set_contractor_rating(r).unwrap();
        }

        quote
    }
}
