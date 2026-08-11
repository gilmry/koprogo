use crate::application::dto::{
    CreateQuoteDto, QuoteComparisonItemDto, QuoteComparisonRequestDto, QuoteComparisonResponseDto,
    QuoteDecisionDto, QuoteResponseDto, QuoteScoreResponseDto, SubmitQuoteDto,
};
use crate::application::ports::QuoteRepository;
use crate::domain::entities::{Quote, QuoteScore, QuoteSubmission};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct QuoteUseCases {
    repository: Arc<dyn QuoteRepository>,
}

impl QuoteUseCases {
    pub fn new(repository: Arc<dyn QuoteRepository>) -> Self {
        Self { repository }
    }

    /// Create new quote request (Syndic action) — request phase, no pricing.
    pub async fn create_quote(&self, dto: CreateQuoteDto) -> Result<QuoteResponseDto, String> {
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building_id format".to_string())?;
        let contractor_id = Uuid::parse_str(&dto.contractor_id)
            .map_err(|_| "Invalid contractor_id format".to_string())?;

        let mut quote = Quote::new(
            building_id,
            contractor_id,
            dto.project_title,
            dto.project_description,
            dto.work_category,
            dto.warranty_years,
        )?;

        // Backward-compat escape hatch: price provided directly at request
        // time (cf. CreateQuoteDto docs) — stays Requested, still needs a
        // separate submit() to become Received.
        if let (Some(amount_excl_vat), Some(vat_rate), Some(validity_date_str)) =
            (dto.amount_excl_vat, dto.vat_rate, dto.validity_date)
        {
            let validity_date = DateTime::parse_from_rfc3339(&validity_date_str)
                .map_err(|_| "Invalid validity_date format".to_string())?
                .with_timezone(&Utc);
            quote.set_initial_pricing(QuoteSubmission {
                amount_excl_vat,
                vat_rate,
                validity_date,
                estimated_duration_days: dto.estimated_duration_days.unwrap_or(0),
                warranty_years: dto.warranty_years,
            })?;
        }

        let created = self.repository.create(&quote).await?;
        Ok(QuoteResponseDto::from(created))
    }

    /// Submit quote (Contractor/Syndic action). `pricing` is optional: a
    /// quote already carrying price data (cf. `create_quote`'s escape
    /// hatch) can be submitted bodyless, otherwise pricing is required.
    pub async fn submit_quote(
        &self,
        quote_id: Uuid,
        pricing: Option<SubmitQuoteDto>,
    ) -> Result<QuoteResponseDto, String> {
        let mut quote = self
            .repository
            .find_by_id(quote_id)
            .await?
            .ok_or_else(|| format!("Quote not found: {}", quote_id))?;

        let pricing = pricing.map(|dto| dto.into_domain()).transpose()?;
        quote.submit(pricing)?;

        let updated = self.repository.update(&quote).await?;
        Ok(QuoteResponseDto::from(updated))
    }

    /// Start quote review (Syndic action)
    pub async fn start_review(&self, quote_id: Uuid) -> Result<QuoteResponseDto, String> {
        let mut quote = self
            .repository
            .find_by_id(quote_id)
            .await?
            .ok_or_else(|| format!("Quote not found: {}", quote_id))?;

        quote.start_review()?;

        let updated = self.repository.update(&quote).await?;
        Ok(QuoteResponseDto::from(updated))
    }

    /// Accept quote (Syndic action - winner)
    pub async fn accept_quote(
        &self,
        quote_id: Uuid,
        decision_by: Uuid,
        dto: QuoteDecisionDto,
    ) -> Result<QuoteResponseDto, String> {
        let mut quote = self
            .repository
            .find_by_id(quote_id)
            .await?
            .ok_or_else(|| format!("Quote not found: {}", quote_id))?;

        quote.accept(decision_by, dto.decision_notes)?;

        let updated = self.repository.update(&quote).await?;
        Ok(QuoteResponseDto::from(updated))
    }

    /// Reject quote (Syndic action - loser or unqualified)
    pub async fn reject_quote(
        &self,
        quote_id: Uuid,
        decision_by: Uuid,
        dto: QuoteDecisionDto,
    ) -> Result<QuoteResponseDto, String> {
        let mut quote = self
            .repository
            .find_by_id(quote_id)
            .await?
            .ok_or_else(|| format!("Quote not found: {}", quote_id))?;

        quote.reject(decision_by, dto.decision_notes)?;

        let updated = self.repository.update(&quote).await?;
        Ok(QuoteResponseDto::from(updated))
    }

    /// Withdraw quote (Contractor action)
    pub async fn withdraw_quote(&self, quote_id: Uuid) -> Result<QuoteResponseDto, String> {
        let mut quote = self
            .repository
            .find_by_id(quote_id)
            .await?
            .ok_or_else(|| format!("Quote not found: {}", quote_id))?;

        quote.withdraw()?;

        let updated = self.repository.update(&quote).await?;
        Ok(QuoteResponseDto::from(updated))
    }

    /// Compare multiple quotes (Belgian professional best practice: 3 quotes minimum for works >5000€)
    /// Returns quotes sorted by total score (best first)
    pub async fn compare_quotes(
        &self,
        dto: QuoteComparisonRequestDto,
    ) -> Result<QuoteComparisonResponseDto, String> {
        if dto.quote_ids.len() < 3 {
            return Err("Best practice requires at least 3 quotes for comparison".to_string());
        }

        // Parse quote IDs
        let quote_ids: Result<Vec<Uuid>, _> = dto
            .quote_ids
            .iter()
            .map(|id_str| {
                Uuid::parse_str(id_str).map_err(|_| format!("Invalid quote_id format: {}", id_str))
            })
            .collect();
        let quote_ids = quote_ids?;

        // Fetch all quotes
        let quotes = self.repository.find_by_ids(quote_ids).await?;

        if quotes.len() < 3 {
            return Err(format!(
                "Found only {} quotes, Belgian law requires at least 3",
                quotes.len()
            ));
        }

        // Ensure all quotes are for the same project, and that all have been
        // submitted (priced) — comparing "Requested" quotes with no price
        // data would make the scoring/aggregation below meaningless.
        let building_id = quotes[0].building_id;
        let project_title = quotes[0].project_title.clone();
        for quote in &quotes {
            if quote.building_id != building_id {
                return Err("All quotes must be for the same building".to_string());
            }
            if quote.project_title != project_title {
                return Err("All quotes must be for the same project".to_string());
            }
            if quote.amount_incl_vat.is_none() {
                return Err(format!(
                    "Quote {} has not been submitted yet (no price data) — cannot compare",
                    quote.id
                ));
            }
        }

        // Calculate aggregated statistics
        let min_price = quotes
            .iter()
            .filter_map(|q| q.amount_incl_vat)
            .min()
            .unwrap_or(Decimal::ZERO);
        let max_price = quotes
            .iter()
            .filter_map(|q| q.amount_incl_vat)
            .max()
            .unwrap_or(Decimal::ZERO);
        let avg_price = quotes
            .iter()
            .filter_map(|q| q.amount_incl_vat)
            .sum::<Decimal>()
            / Decimal::from(quotes.len());

        let min_duration_days = quotes
            .iter()
            .filter_map(|q| q.estimated_duration_days)
            .min()
            .unwrap_or(0);
        let max_duration_days = quotes
            .iter()
            .filter_map(|q| q.estimated_duration_days)
            .max()
            .unwrap_or(0);
        let avg_duration_days = quotes
            .iter()
            .filter_map(|q| q.estimated_duration_days)
            .sum::<i32>() as f32
            / quotes.len() as f32;

        let max_warranty = quotes.iter().map(|q| q.warranty_years).max().unwrap_or(0);

        // Calculate scores for each quote
        let mut scored_quotes: Vec<(Quote, QuoteScore)> = Vec::new();
        for quote in quotes {
            let score = quote.calculate_score(
                min_price,
                max_price,
                min_duration_days,
                max_duration_days,
                max_warranty,
            )?;
            scored_quotes.push((quote, score));
        }

        // Sort by total score (descending - best first)
        scored_quotes.sort_by(|a, b| {
            b.1.total_score
                .partial_cmp(&a.1.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Build comparison items with ranking
        let comparison_items: Vec<QuoteComparisonItemDto> = scored_quotes
            .into_iter()
            .enumerate()
            .map(|(index, (quote, score))| QuoteComparisonItemDto {
                quote: QuoteResponseDto::from(quote),
                score: Some(QuoteScoreResponseDto::from(score)),
                rank: index + 1, // 1-indexed ranking
            })
            .collect();

        // Recommend top-ranked quote
        let recommended_quote_id = comparison_items.first().map(|item| item.quote.id.clone());

        Ok(QuoteComparisonResponseDto {
            project_title,
            building_id: building_id.to_string(),
            total_quotes: comparison_items.len(),
            comparison_items,
            min_price: min_price.to_string(),
            max_price: max_price.to_string(),
            avg_price: avg_price.to_string(),
            min_duration_days,
            max_duration_days,
            avg_duration_days,
            recommended_quote_id,
        })
    }

    /// Get quote by ID
    pub async fn get_quote(&self, quote_id: Uuid) -> Result<Option<QuoteResponseDto>, String> {
        let quote = self.repository.find_by_id(quote_id).await?;
        Ok(quote.map(QuoteResponseDto::from))
    }

    /// List quotes by building
    pub async fn list_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<QuoteResponseDto>, String> {
        let quotes = self.repository.find_by_building(building_id).await?;
        Ok(quotes.into_iter().map(QuoteResponseDto::from).collect())
    }

    /// List quotes by contractor
    pub async fn list_by_contractor(
        &self,
        contractor_id: Uuid,
    ) -> Result<Vec<QuoteResponseDto>, String> {
        let quotes = self.repository.find_by_contractor(contractor_id).await?;
        Ok(quotes.into_iter().map(QuoteResponseDto::from).collect())
    }

    /// List quotes by status
    pub async fn list_by_status(
        &self,
        building_id: Uuid,
        status: &str,
    ) -> Result<Vec<QuoteResponseDto>, String> {
        let quotes = self.repository.find_by_status(building_id, status).await?;
        Ok(quotes.into_iter().map(QuoteResponseDto::from).collect())
    }

    /// List quotes by project title
    pub async fn list_by_project_title(
        &self,
        building_id: Uuid,
        project_title: &str,
    ) -> Result<Vec<QuoteResponseDto>, String> {
        let quotes = self
            .repository
            .find_by_project_title(building_id, project_title)
            .await?;
        Ok(quotes.into_iter().map(QuoteResponseDto::from).collect())
    }

    /// Update contractor rating (for scoring)
    pub async fn update_contractor_rating(
        &self,
        quote_id: Uuid,
        rating: i32,
    ) -> Result<QuoteResponseDto, String> {
        let mut quote = self
            .repository
            .find_by_id(quote_id)
            .await?
            .ok_or_else(|| format!("Quote not found: {}", quote_id))?;

        quote.set_contractor_rating(rating)?;

        let updated = self.repository.update(&quote).await?;
        Ok(QuoteResponseDto::from(updated))
    }

    /// Mark expired quotes (background job)
    /// Returns count of quotes marked as expired
    pub async fn mark_expired_quotes(&self) -> Result<usize, String> {
        let expired_quotes = self.repository.find_expired().await?;

        let mut count = 0;
        for mut quote in expired_quotes {
            if quote.mark_expired().is_ok() {
                self.repository.update(&quote).await?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Delete quote
    pub async fn delete_quote(&self, quote_id: Uuid) -> Result<bool, String> {
        self.repository.delete(quote_id).await
    }

    /// Count quotes by building
    pub async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        self.repository.count_by_building(building_id).await
    }

    /// Count quotes by status
    pub async fn count_by_status(&self, building_id: Uuid, status: &str) -> Result<i64, String> {
        self.repository.count_by_status(building_id, status).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::QuoteRepository;
    use crate::domain::entities::Quote;
    use async_trait::async_trait;
    use mockall::mock;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // Helper macro since dec! is not available in rust_decimal 1.36
    macro_rules! dec {
        ($val:expr) => {
            Decimal::from_str(stringify!($val)).unwrap()
        };
    }

    mock! {
        QuoteRepo {}

        #[async_trait]
        impl QuoteRepository for QuoteRepo {
            async fn create(&self, quote: &Quote) -> Result<Quote, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Quote>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Quote>, String>;
            async fn find_by_contractor(&self, contractor_id: Uuid) -> Result<Vec<Quote>, String>;
            async fn find_by_status(&self, building_id: Uuid, status: &str) -> Result<Vec<Quote>, String>;
            async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<Quote>, String>;
            async fn find_by_project_title(&self, building_id: Uuid, project_title: &str) -> Result<Vec<Quote>, String>;
            async fn find_expired(&self) -> Result<Vec<Quote>, String>;
            async fn update(&self, quote: &Quote) -> Result<Quote, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String>;
            async fn count_by_status(&self, building_id: Uuid, status: &str) -> Result<i64, String>;
        }
    }

    #[tokio::test]
    async fn test_create_quote_success() {
        let mut mock_repo = MockQuoteRepo::new();

        mock_repo
            .expect_create()
            .returning(|quote| Ok(quote.clone()));

        let use_cases = QuoteUseCases::new(Arc::new(mock_repo));

        let dto = CreateQuoteDto {
            building_id: Uuid::new_v4().to_string(),
            contractor_id: Uuid::new_v4().to_string(),
            project_title: "Roof Repair".to_string(),
            project_description: "Fix leaking roof".to_string(),
            work_category: Some("roofing".to_string()),
            amount_excl_vat: None,
            vat_rate: None,
            validity_date: None,
            estimated_start_date: None,
            estimated_duration_days: None,
            warranty_years: 10,
        };

        let result = use_cases.create_quote(dto).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "Requested");
    }

    #[tokio::test]
    async fn test_submit_quote() {
        let mut mock_repo = MockQuoteRepo::new();
        let quote_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let contractor_id = Uuid::new_v4();

        let quote = Quote::new(
            building_id,
            contractor_id,
            "Test".to_string(),
            "Desc".to_string(),
            Some("roofing".to_string()),
            10,
        )
        .unwrap();

        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(quote.clone())));
        mock_repo
            .expect_update()
            .returning(|quote| Ok(quote.clone()));

        let use_cases = QuoteUseCases::new(Arc::new(mock_repo));

        let pricing = SubmitQuoteDto {
            amount_excl_vat_cents: 500_000,
            vat_rate: dec!(21.00),
            validity_date: (Utc::now() + chrono::Duration::days(30)).to_rfc3339(),
            estimated_duration_days: 14,
            warranty_years: 10,
        };
        let result = use_cases.submit_quote(quote_id, Some(pricing)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "Received");
    }

    #[tokio::test]
    async fn test_compare_quotes_requires_minimum_3() {
        let mock_repo = MockQuoteRepo::new();
        let use_cases = QuoteUseCases::new(Arc::new(mock_repo));

        let dto = QuoteComparisonRequestDto {
            quote_ids: vec![Uuid::new_v4().to_string(), Uuid::new_v4().to_string()],
        };

        let result = use_cases.compare_quotes(dto).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Best practice requires at least 3 quotes for comparison"
        );
    }
}
