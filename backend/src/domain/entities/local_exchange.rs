use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Local Exchange Trading System (SEL) - Système d'Échange Local
///
/// Enables co-owners to exchange services, objects, and shared purchases
/// using time-based currency (1 hour = 1 credit).
///
/// Belgian Legal Context:
/// - SELs are legal and recognized in Belgium
/// - No taxation if non-commercial (barter)
/// - Must not replace professional services (insurance issues)
/// - Clear T&Cs required (liability disclaimer)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalExchange {
    pub id: Uuid,
    pub building_id: Uuid,
    pub provider_id: Uuid,  // Owner offering the exchange
    pub requester_id: Option<Uuid>, // Owner requesting (None if still offered)
    pub exchange_type: ExchangeType,
    pub title: String,
    pub description: String,
    pub credits: i32, // Time in hours (or custom unit)
    pub status: ExchangeStatus,
    pub offered_at: DateTime<Utc>,
    pub requested_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub provider_rating: Option<i32>, // 1-5 stars from requester
    pub requester_rating: Option<i32>, // 1-5 stars from provider
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum ExchangeType {
    Service,         // Skills (plumbing, gardening, tutoring, etc.)
    ObjectLoan,      // Temporary loan (tools, books, equipment)
    SharedPurchase,  // Co-buying (bulk food, equipment rental)
}

impl ExchangeType {
    pub fn to_sql(&self) -> &'static str {
        match self {
            ExchangeType::Service => "Service",
            ExchangeType::ObjectLoan => "ObjectLoan",
            ExchangeType::SharedPurchase => "SharedPurchase",
        }
    }

    pub fn from_sql(s: &str) -> Result<Self, String> {
        match s {
            "Service" => Ok(ExchangeType::Service),
            "ObjectLoan" => Ok(ExchangeType::ObjectLoan),
            "SharedPurchase" => Ok(ExchangeType::SharedPurchase),
            _ => Err(format!("Invalid exchange type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum ExchangeStatus {
    Offered,     // Available for anyone to request
    Requested,   // Someone claimed it (pending provider acceptance)
    InProgress,  // Exchange is happening
    Completed,   // Both parties confirmed completion
    Cancelled,   // Exchange was cancelled
}

impl ExchangeStatus {
    pub fn to_sql(&self) -> &'static str {
        match self {
            ExchangeStatus::Offered => "Offered",
            ExchangeStatus::Requested => "Requested",
            ExchangeStatus::InProgress => "InProgress",
            ExchangeStatus::Completed => "Completed",
            ExchangeStatus::Cancelled => "Cancelled",
        }
    }

    pub fn from_sql(s: &str) -> Result<Self, String> {
        match s {
            "Offered" => Ok(ExchangeStatus::Offered),
            "Requested" => Ok(ExchangeStatus::Requested),
            "InProgress" => Ok(ExchangeStatus::InProgress),
            "Completed" => Ok(ExchangeStatus::Completed),
            "Cancelled" => Ok(ExchangeStatus::Cancelled),
            _ => Err(format!("Invalid exchange status: {}", s)),
        }
    }
}

impl LocalExchange {
    /// Create a new exchange offer
    pub fn new(
        building_id: Uuid,
        provider_id: Uuid,
        exchange_type: ExchangeType,
        title: String,
        description: String,
        credits: i32,
    ) -> Result<Self, String> {
        // Validation
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        if title.len() > 255 {
            return Err("Title cannot exceed 255 characters".to_string());
        }

        if description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }

        if description.len() > 2000 {
            return Err("Description cannot exceed 2000 characters".to_string());
        }

        if credits <= 0 {
            return Err("Credits must be positive".to_string());
        }

        if credits > 100 {
            return Err("Credits cannot exceed 100 (maximum 100 hours)".to_string());
        }

        let now = Utc::now();

        Ok(LocalExchange {
            id: Uuid::new_v4(),
            building_id,
            provider_id,
            requester_id: None,
            exchange_type,
            title: title.trim().to_string(),
            description: description.trim().to_string(),
            credits,
            status: ExchangeStatus::Offered,
            offered_at: now,
            requested_at: None,
            started_at: None,
            completed_at: None,
            cancelled_at: None,
            cancellation_reason: None,
            provider_rating: None,
            requester_rating: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Request an exchange (transition: Offered → Requested)
    pub fn request(&mut self, requester_id: Uuid) -> Result<(), String> {
        if self.status != ExchangeStatus::Offered {
            return Err(format!(
                "Cannot request exchange in status {:?}",
                self.status
            ));
        }

        if self.provider_id == requester_id {
            return Err("Provider cannot request their own exchange".to_string());
        }

        self.requester_id = Some(requester_id);
        self.status = ExchangeStatus::Requested;
        self.requested_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Start an exchange (transition: Requested → InProgress)
    pub fn start(&mut self, actor_id: Uuid) -> Result<(), String> {
        if self.status != ExchangeStatus::Requested {
            return Err(format!("Cannot start exchange in status {:?}", self.status));
        }

        // Only provider can start the exchange
        if self.provider_id != actor_id {
            return Err("Only the provider can start the exchange".to_string());
        }

        self.status = ExchangeStatus::InProgress;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Complete an exchange (transition: InProgress → Completed)
    /// Both provider and requester must confirm completion
    pub fn complete(&mut self, actor_id: Uuid) -> Result<(), String> {
        if self.status != ExchangeStatus::InProgress {
            return Err(format!(
                "Cannot complete exchange in status {:?}",
                self.status
            ));
        }

        // Only provider or requester can complete
        if self.provider_id != actor_id
            && self.requester_id != Some(actor_id)
        {
            return Err("Only provider or requester can complete the exchange".to_string());
        }

        self.status = ExchangeStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Cancel an exchange
    pub fn cancel(&mut self, actor_id: Uuid, reason: Option<String>) -> Result<(), String> {
        // Cannot cancel completed exchanges
        if self.status == ExchangeStatus::Completed {
            return Err("Cannot cancel a completed exchange".to_string());
        }

        if self.status == ExchangeStatus::Cancelled {
            return Err("Exchange is already cancelled".to_string());
        }

        // Only provider or requester can cancel
        if self.provider_id != actor_id
            && self.requester_id != Some(actor_id)
        {
            return Err("Only provider or requester can cancel the exchange".to_string());
        }

        self.status = ExchangeStatus::Cancelled;
        self.cancelled_at = Some(Utc::now());
        self.cancellation_reason = reason;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Rate the provider (by requester)
    pub fn rate_provider(&mut self, requester_id: Uuid, rating: i32) -> Result<(), String> {
        if self.status != ExchangeStatus::Completed {
            return Err("Can only rate completed exchanges".to_string());
        }

        if self.requester_id != Some(requester_id) {
            return Err("Only the requester can rate the provider".to_string());
        }

        if !(1..=5).contains(&rating) {
            return Err("Rating must be between 1 and 5".to_string());
        }

        self.provider_rating = Some(rating);
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Rate the requester (by provider)
    pub fn rate_requester(&mut self, provider_id: Uuid, rating: i32) -> Result<(), String> {
        if self.status != ExchangeStatus::Completed {
            return Err("Can only rate completed exchanges".to_string());
        }

        if self.provider_id != provider_id {
            return Err("Only the provider can rate the requester".to_string());
        }

        if !(1..=5).contains(&rating) {
            return Err("Rating must be between 1 and 5".to_string());
        }

        self.requester_rating = Some(rating);
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Check if the exchange is active (not completed or cancelled)
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            ExchangeStatus::Offered | ExchangeStatus::Requested | ExchangeStatus::InProgress
        )
    }

    /// Check if ratings are complete
    pub fn has_mutual_ratings(&self) -> bool {
        self.provider_rating.is_some() && self.requester_rating.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_exchange_success() {
        let building_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();

        let exchange = LocalExchange::new(
            building_id,
            provider_id,
            ExchangeType::Service,
            "Gardening help".to_string(),
            "I can help with planting and weeding".to_string(),
            2,
        );

        assert!(exchange.is_ok());
        let exchange = exchange.unwrap();
        assert_eq!(exchange.building_id, building_id);
        assert_eq!(exchange.provider_id, provider_id);
        assert_eq!(exchange.status, ExchangeStatus::Offered);
        assert_eq!(exchange.credits, 2);
        assert!(exchange.requester_id.is_none());
    }

    #[test]
    fn test_create_exchange_validation() {
        let building_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();

        // Empty title
        let result = LocalExchange::new(
            building_id,
            provider_id,
            ExchangeType::Service,
            "".to_string(),
            "Description".to_string(),
            2,
        );
        assert!(result.is_err());

        // Negative credits
        let result = LocalExchange::new(
            building_id,
            provider_id,
            ExchangeType::Service,
            "Title".to_string(),
            "Description".to_string(),
            -1,
        );
        assert!(result.is_err());

        // Too many credits
        let result = LocalExchange::new(
            building_id,
            provider_id,
            ExchangeType::Service,
            "Title".to_string(),
            "Description".to_string(),
            101,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_exchange_workflow() {
        let building_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();

        let mut exchange = LocalExchange::new(
            building_id,
            provider_id,
            ExchangeType::Service,
            "Babysitting".to_string(),
            "Can watch kids for 3 hours".to_string(),
            3,
        )
        .unwrap();

        // Request
        assert!(exchange.request(requester_id).is_ok());
        assert_eq!(exchange.status, ExchangeStatus::Requested);
        assert_eq!(exchange.requester_id, Some(requester_id));

        // Start
        assert!(exchange.start(provider_id).is_ok());
        assert_eq!(exchange.status, ExchangeStatus::InProgress);

        // Complete
        assert!(exchange.complete(provider_id).is_ok());
        assert_eq!(exchange.status, ExchangeStatus::Completed);
        assert!(exchange.completed_at.is_some());
    }

    #[test]
    fn test_cannot_request_own_exchange() {
        let building_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();

        let mut exchange = LocalExchange::new(
            building_id,
            provider_id,
            ExchangeType::Service,
            "Service".to_string(),
            "Description".to_string(),
            2,
        )
        .unwrap();

        let result = exchange.request(provider_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_exchange() {
        let building_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();

        let mut exchange = LocalExchange::new(
            building_id,
            provider_id,
            ExchangeType::Service,
            "Service".to_string(),
            "Description".to_string(),
            2,
        )
        .unwrap();

        exchange.request(requester_id).unwrap();

        // Requester cancels
        assert!(exchange
            .cancel(requester_id, Some("Changed my mind".to_string()))
            .is_ok());
        assert_eq!(exchange.status, ExchangeStatus::Cancelled);
        assert!(exchange.cancelled_at.is_some());
        assert_eq!(exchange.cancellation_reason, Some("Changed my mind".to_string()));
    }

    #[test]
    fn test_cannot_cancel_completed_exchange() {
        let building_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();

        let mut exchange = LocalExchange::new(
            building_id,
            provider_id,
            ExchangeType::Service,
            "Service".to_string(),
            "Description".to_string(),
            2,
        )
        .unwrap();

        exchange.request(requester_id).unwrap();
        exchange.start(provider_id).unwrap();
        exchange.complete(provider_id).unwrap();

        let result = exchange.cancel(provider_id, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_ratings() {
        let building_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();

        let mut exchange = LocalExchange::new(
            building_id,
            provider_id,
            ExchangeType::Service,
            "Service".to_string(),
            "Description".to_string(),
            2,
        )
        .unwrap();

        exchange.request(requester_id).unwrap();
        exchange.start(provider_id).unwrap();
        exchange.complete(provider_id).unwrap();

        // Rate provider
        assert!(exchange.rate_provider(requester_id, 5).is_ok());
        assert_eq!(exchange.provider_rating, Some(5));

        // Rate requester
        assert!(exchange.rate_requester(provider_id, 4).is_ok());
        assert_eq!(exchange.requester_rating, Some(4));

        assert!(exchange.has_mutual_ratings());
    }

    #[test]
    fn test_rating_validation() {
        let building_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();

        let mut exchange = LocalExchange::new(
            building_id,
            provider_id,
            ExchangeType::Service,
            "Service".to_string(),
            "Description".to_string(),
            2,
        )
        .unwrap();

        exchange.request(requester_id).unwrap();
        exchange.start(provider_id).unwrap();
        exchange.complete(provider_id).unwrap();

        // Invalid rating (too low)
        assert!(exchange.rate_provider(requester_id, 0).is_err());

        // Invalid rating (too high)
        assert!(exchange.rate_provider(requester_id, 6).is_err());

        // Wrong actor
        assert!(exchange.rate_provider(provider_id, 5).is_err());
    }

    #[test]
    fn test_exchange_type_sql_conversion() {
        assert_eq!(ExchangeType::Service.to_sql(), "Service");
        assert_eq!(ExchangeType::ObjectLoan.to_sql(), "ObjectLoan");
        assert_eq!(ExchangeType::SharedPurchase.to_sql(), "SharedPurchase");

        assert_eq!(
            ExchangeType::from_sql("Service").unwrap(),
            ExchangeType::Service
        );
        assert_eq!(
            ExchangeType::from_sql("ObjectLoan").unwrap(),
            ExchangeType::ObjectLoan
        );
        assert_eq!(
            ExchangeType::from_sql("SharedPurchase").unwrap(),
            ExchangeType::SharedPurchase
        );
    }

    #[test]
    fn test_exchange_status_sql_conversion() {
        assert_eq!(ExchangeStatus::Offered.to_sql(), "Offered");
        assert_eq!(ExchangeStatus::Requested.to_sql(), "Requested");
        assert_eq!(ExchangeStatus::InProgress.to_sql(), "InProgress");
        assert_eq!(ExchangeStatus::Completed.to_sql(), "Completed");
        assert_eq!(ExchangeStatus::Cancelled.to_sql(), "Cancelled");

        assert_eq!(
            ExchangeStatus::from_sql("Offered").unwrap(),
            ExchangeStatus::Offered
        );
        assert_eq!(
            ExchangeStatus::from_sql("Requested").unwrap(),
            ExchangeStatus::Requested
        );
        assert_eq!(
            ExchangeStatus::from_sql("InProgress").unwrap(),
            ExchangeStatus::InProgress
        );
        assert_eq!(
            ExchangeStatus::from_sql("Completed").unwrap(),
            ExchangeStatus::Completed
        );
        assert_eq!(
            ExchangeStatus::from_sql("Cancelled").unwrap(),
            ExchangeStatus::Cancelled
        );
    }
}
