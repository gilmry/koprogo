use crate::domain::entities::{
    CreditStatus, ExchangeStatus, ExchangeType, LocalExchange, OwnerCreditBalance,
    ParticipationLevel,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DTO for creating a new local exchange offer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLocalExchangeDto {
    pub building_id: Uuid,
    pub exchange_type: ExchangeType,
    pub title: String,
    pub description: String,
    pub credits: i32,
}

/// DTO for requesting an exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestExchangeDto {
    // Empty body - requester_id comes from auth
}

/// DTO for completing an exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteExchangeDto {
    // Empty body - actor_id comes from auth
}

/// DTO for cancelling an exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelExchangeDto {
    pub reason: Option<String>,
}

/// DTO for rating an exchange partner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateExchangeDto {
    pub rating: i32, // 1-5 stars
}

/// DTO for returning exchange data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalExchangeResponseDto {
    pub id: Uuid,
    pub building_id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String, // Joined from owner table
    pub requester_id: Option<Uuid>,
    pub requester_name: Option<String>, // Joined from owner table
    pub exchange_type: ExchangeType,
    pub title: String,
    pub description: String,
    pub credits: i32,
    pub status: ExchangeStatus,
    pub offered_at: DateTime<Utc>,
    pub requested_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub provider_rating: Option<i32>,
    pub requester_rating: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LocalExchangeResponseDto {
    pub fn from_entity(
        exchange: LocalExchange,
        provider_name: String,
        requester_name: Option<String>,
    ) -> Self {
        LocalExchangeResponseDto {
            id: exchange.id,
            building_id: exchange.building_id,
            provider_id: exchange.provider_id,
            provider_name,
            requester_id: exchange.requester_id,
            requester_name,
            exchange_type: exchange.exchange_type,
            title: exchange.title,
            description: exchange.description,
            credits: exchange.credits,
            status: exchange.status,
            offered_at: exchange.offered_at,
            requested_at: exchange.requested_at,
            started_at: exchange.started_at,
            completed_at: exchange.completed_at,
            cancelled_at: exchange.cancelled_at,
            cancellation_reason: exchange.cancellation_reason,
            provider_rating: exchange.provider_rating,
            requester_rating: exchange.requester_rating,
            created_at: exchange.created_at,
            updated_at: exchange.updated_at,
        }
    }
}

/// DTO for returning owner credit balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerCreditBalanceDto {
    pub owner_id: Uuid,
    pub owner_name: String, // Joined from owner table
    pub building_id: Uuid,
    pub credits_earned: i32,
    pub credits_spent: i32,
    pub balance: i32,
    pub credit_status: CreditStatus,
    pub total_exchanges: i32,
    pub average_rating: Option<f32>,
    pub participation_level: ParticipationLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OwnerCreditBalanceDto {
    pub fn from_entity(balance: OwnerCreditBalance, owner_name: String) -> Self {
        OwnerCreditBalanceDto {
            owner_id: balance.owner_id,
            owner_name,
            building_id: balance.building_id,
            credits_earned: balance.credits_earned,
            credits_spent: balance.credits_spent,
            balance: balance.balance,
            credit_status: balance.credit_status(),
            total_exchanges: balance.total_exchanges,
            average_rating: balance.average_rating,
            participation_level: balance.participation_level(),
            created_at: balance.created_at,
            updated_at: balance.updated_at,
        }
    }
}

/// DTO for building-level SEL statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelStatisticsDto {
    pub building_id: Uuid,
    pub total_exchanges: i32,
    pub active_exchanges: i32,
    pub completed_exchanges: i32,
    pub total_credits_exchanged: i32,
    pub active_participants: i32,
    pub average_exchange_rating: Option<f32>,
    pub most_popular_exchange_type: Option<ExchangeType>,
}

/// DTO for owner exchange history summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerExchangeSummaryDto {
    pub owner_id: Uuid,
    pub owner_name: String,
    pub as_provider: i32,     // Number of exchanges as provider
    pub as_requester: i32,    // Number of exchanges as requester
    pub total_exchanges: i32, // Sum of both
    pub average_rating: Option<f32>,
    pub recent_exchanges: Vec<LocalExchangeResponseDto>, // Last 5
}
