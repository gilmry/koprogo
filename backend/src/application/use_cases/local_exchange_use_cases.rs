use crate::application::dto::{
    CancelExchangeDto, CompleteExchangeDto, CreateLocalExchangeDto, LocalExchangeResponseDto,
    OwnerCreditBalanceDto, OwnerExchangeSummaryDto, RateExchangeDto, RequestExchangeDto,
    SelStatisticsDto,
};
use crate::application::ports::{
    LocalExchangeRepository, OwnerCreditBalanceRepository, OwnerRepository,
};
use crate::domain::entities::{ExchangeStatus, ExchangeType, LocalExchange};
use std::sync::Arc;
use uuid::Uuid;

/// Use cases for Local Exchange Trading System (SEL)
pub struct LocalExchangeUseCases {
    exchange_repo: Arc<dyn LocalExchangeRepository>,
    balance_repo: Arc<dyn OwnerCreditBalanceRepository>,
    owner_repo: Arc<dyn OwnerRepository>,
}

impl LocalExchangeUseCases {
    pub fn new(
        exchange_repo: Arc<dyn LocalExchangeRepository>,
        balance_repo: Arc<dyn OwnerCreditBalanceRepository>,
        owner_repo: Arc<dyn OwnerRepository>,
    ) -> Self {
        Self {
            exchange_repo,
            balance_repo,
            owner_repo,
        }
    }

    /// Create a new exchange offer
    pub async fn create_exchange(
        &self,
        provider_id: Uuid, // From auth
        dto: CreateLocalExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        // Verify owner exists
        let provider = self
            .owner_repo
            .find_by_id(provider_id)
            .await?
            .ok_or("Provider not found".to_string())?;

        // Verify building exists (via owner)
        let _ = self
            .owner_repo
            .find_by_id(dto.building_id) // Check building ownership
            .await?;

        // Create domain entity
        let exchange = LocalExchange::new(
            dto.building_id,
            provider_id,
            dto.exchange_type,
            dto.title,
            dto.description,
            dto.credits,
        )?;

        // Persist
        let created = self.exchange_repo.create(&exchange).await?;

        // Return DTO with provider name
        Ok(LocalExchangeResponseDto::from_entity(
            created,
            format!("{} {}", provider.first_name, provider.last_name),
            None,
        ))
    }

    /// Get exchange by ID
    pub async fn get_exchange(&self, id: Uuid) -> Result<LocalExchangeResponseDto, String> {
        let exchange = self
            .exchange_repo
            .find_by_id(id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        // Get provider name
        let provider = self
            .owner_repo
            .find_by_id(exchange.provider_id)
            .await?
            .ok_or("Provider not found".to_string())?;
        let provider_name = format!("{} {}", provider.first_name, provider.last_name);

        // Get requester name if exists
        let requester_name = if let Some(requester_id) = exchange.requester_id {
            let requester = self.owner_repo.find_by_id(requester_id).await?;
            requester.map(|r| format!("{} {}", r.first_name, r.last_name))
        } else {
            None
        };

        Ok(LocalExchangeResponseDto::from_entity(
            exchange,
            provider_name,
            requester_name,
        ))
    }

    /// List all exchanges for a building
    pub async fn list_building_exchanges(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<LocalExchangeResponseDto>, String> {
        let exchanges = self.exchange_repo.find_by_building(building_id).await?;

        self.enrich_exchanges_with_names(exchanges).await
    }

    /// List available exchanges (status = Offered)
    pub async fn list_available_exchanges(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<LocalExchangeResponseDto>, String> {
        let exchanges = self
            .exchange_repo
            .find_available_by_building(building_id)
            .await?;

        self.enrich_exchanges_with_names(exchanges).await
    }

    /// List exchanges by owner (as provider OR requester)
    pub async fn list_owner_exchanges(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<LocalExchangeResponseDto>, String> {
        let exchanges = self.exchange_repo.find_by_owner(owner_id).await?;

        self.enrich_exchanges_with_names(exchanges).await
    }

    /// List exchanges by type
    pub async fn list_exchanges_by_type(
        &self,
        building_id: Uuid,
        exchange_type: ExchangeType,
    ) -> Result<Vec<LocalExchangeResponseDto>, String> {
        let exchanges = self
            .exchange_repo
            .find_by_type(building_id, exchange_type.to_sql())
            .await?;

        self.enrich_exchanges_with_names(exchanges).await
    }

    /// Request an exchange (transition: Offered → Requested)
    pub async fn request_exchange(
        &self,
        exchange_id: Uuid,
        requester_id: Uuid, // From auth
        _dto: RequestExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        // Load exchange
        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        // Apply business logic
        exchange.request(requester_id)?;

        // Persist
        let updated = self.exchange_repo.update(&exchange).await?;

        // Return enriched DTO
        self.get_exchange(updated.id).await
    }

    /// Start an exchange (transition: Requested → InProgress)
    /// Only provider can start
    pub async fn start_exchange(
        &self,
        exchange_id: Uuid,
        actor_id: Uuid, // From auth (must be provider)
    ) -> Result<LocalExchangeResponseDto, String> {
        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        exchange.start(actor_id)?;

        let updated = self.exchange_repo.update(&exchange).await?;

        self.get_exchange(updated.id).await
    }

    /// Complete an exchange (transition: InProgress → Completed)
    /// Updates credit balances for both parties
    pub async fn complete_exchange(
        &self,
        exchange_id: Uuid,
        actor_id: Uuid, // From auth (provider or requester)
        _dto: CompleteExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        // Get requester before completing (we'll need it for balance update)
        let requester_id = exchange
            .requester_id
            .ok_or("Exchange has no requester".to_string())?;

        exchange.complete(actor_id)?;

        let updated = self.exchange_repo.update(&exchange).await?;

        // Update credit balances
        self.update_credit_balances_on_completion(&updated).await?;

        // Increment exchange counters
        let mut provider_balance = self
            .balance_repo
            .get_or_create(updated.provider_id, updated.building_id)
            .await?;
        provider_balance.increment_exchanges();
        self.balance_repo.update(&provider_balance).await?;

        let mut requester_balance = self
            .balance_repo
            .get_or_create(requester_id, updated.building_id)
            .await?;
        requester_balance.increment_exchanges();
        self.balance_repo.update(&requester_balance).await?;

        self.get_exchange(updated.id).await
    }

    /// Cancel an exchange
    pub async fn cancel_exchange(
        &self,
        exchange_id: Uuid,
        actor_id: Uuid, // From auth (provider or requester)
        dto: CancelExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        exchange.cancel(actor_id, dto.reason)?;

        let updated = self.exchange_repo.update(&exchange).await?;

        self.get_exchange(updated.id).await
    }

    /// Rate the provider (by requester)
    pub async fn rate_provider(
        &self,
        exchange_id: Uuid,
        requester_id: Uuid, // From auth
        dto: RateExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        exchange.rate_provider(requester_id, dto.rating)?;

        let updated = self.exchange_repo.update(&exchange).await?;

        // Update provider's average rating
        self.update_average_rating(updated.provider_id, updated.building_id)
            .await?;

        self.get_exchange(updated.id).await
    }

    /// Rate the requester (by provider)
    pub async fn rate_requester(
        &self,
        exchange_id: Uuid,
        provider_id: Uuid, // From auth
        dto: RateExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        exchange.rate_requester(provider_id, dto.rating)?;

        let updated = self.exchange_repo.update(&exchange).await?;

        // Update requester's average rating
        if let Some(requester_id) = updated.requester_id {
            self.update_average_rating(requester_id, updated.building_id)
                .await?;
        }

        self.get_exchange(updated.id).await
    }

    /// Delete an exchange (only if not completed)
    pub async fn delete_exchange(&self, exchange_id: Uuid, actor_id: Uuid) -> Result<(), String> {
        let exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        // Only provider can delete
        if exchange.provider_id != actor_id {
            return Err("Only the provider can delete the exchange".to_string());
        }

        // Cannot delete completed exchanges
        if exchange.status == ExchangeStatus::Completed {
            return Err("Cannot delete a completed exchange".to_string());
        }

        self.exchange_repo.delete(exchange_id).await?;

        Ok(())
    }

    /// Get credit balance for an owner in a building
    pub async fn get_credit_balance(
        &self,
        owner_id: Uuid,
        building_id: Uuid,
    ) -> Result<OwnerCreditBalanceDto, String> {
        let balance = self
            .balance_repo
            .get_or_create(owner_id, building_id)
            .await?;

        let owner = self
            .owner_repo
            .find_by_id(owner_id)
            .await?
            .ok_or("Owner not found".to_string())?;
        let owner_name = format!("{} {}", owner.first_name, owner.last_name);

        Ok(OwnerCreditBalanceDto::from_entity(balance, owner_name))
    }

    /// Get leaderboard (top contributors)
    pub async fn get_leaderboard(
        &self,
        building_id: Uuid,
        limit: i32,
    ) -> Result<Vec<OwnerCreditBalanceDto>, String> {
        let balances = self
            .balance_repo
            .get_leaderboard(building_id, limit)
            .await?;

        let mut dtos = Vec::new();
        for balance in balances {
            if let Some(owner) = self.owner_repo.find_by_id(balance.owner_id).await? {
                let owner_name = format!("{} {}", owner.first_name, owner.last_name);
                dtos.push(OwnerCreditBalanceDto::from_entity(balance, owner_name));
            }
        }

        Ok(dtos)
    }

    /// Get SEL statistics for a building
    pub async fn get_statistics(&self, building_id: Uuid) -> Result<SelStatisticsDto, String> {
        let total_exchanges = self.exchange_repo.count_by_building(building_id).await? as i32;

        let active_exchanges = self
            .exchange_repo
            .count_by_building_and_status(building_id, ExchangeStatus::Offered.to_sql())
            .await? as i32
            + self
                .exchange_repo
                .count_by_building_and_status(building_id, ExchangeStatus::Requested.to_sql())
                .await? as i32
            + self
                .exchange_repo
                .count_by_building_and_status(building_id, ExchangeStatus::InProgress.to_sql())
                .await? as i32;

        let completed_exchanges = self
            .exchange_repo
            .count_by_building_and_status(building_id, ExchangeStatus::Completed.to_sql())
            .await? as i32;

        let total_credits_exchanged = self
            .exchange_repo
            .get_total_credits_exchanged(building_id)
            .await?;

        let active_participants = self
            .balance_repo
            .count_active_participants(building_id)
            .await? as i32;

        let average_exchange_rating = self
            .exchange_repo
            .get_average_exchange_rating(building_id)
            .await?;

        // Determine most popular exchange type
        let service_count = self
            .exchange_repo
            .count_by_building_and_status(building_id, ExchangeType::Service.to_sql())
            .await?;
        let object_loan_count = self
            .exchange_repo
            .count_by_building_and_status(building_id, ExchangeType::ObjectLoan.to_sql())
            .await?;
        let shared_purchase_count = self
            .exchange_repo
            .count_by_building_and_status(building_id, ExchangeType::SharedPurchase.to_sql())
            .await?;

        let most_popular_exchange_type =
            if service_count >= object_loan_count && service_count >= shared_purchase_count {
                Some(ExchangeType::Service)
            } else if object_loan_count >= shared_purchase_count {
                Some(ExchangeType::ObjectLoan)
            } else {
                Some(ExchangeType::SharedPurchase)
            };

        Ok(SelStatisticsDto {
            building_id,
            total_exchanges,
            active_exchanges,
            completed_exchanges,
            total_credits_exchanged,
            active_participants,
            average_exchange_rating,
            most_popular_exchange_type,
        })
    }

    /// Get owner exchange summary
    pub async fn get_owner_summary(
        &self,
        owner_id: Uuid,
    ) -> Result<OwnerExchangeSummaryDto, String> {
        let owner = self
            .owner_repo
            .find_by_id(owner_id)
            .await?
            .ok_or("Owner not found".to_string())?;
        let owner_name = format!("{} {}", owner.first_name, owner.last_name);

        let exchanges = self.exchange_repo.find_by_owner(owner_id).await?;

        let as_provider = exchanges
            .iter()
            .filter(|e| e.provider_id == owner_id)
            .count() as i32;

        let as_requester = exchanges
            .iter()
            .filter(|e| e.requester_id == Some(owner_id))
            .count() as i32;

        // Calculate average rating (average of all ratings received)
        let mut total_ratings = 0;
        let mut rating_count = 0;

        for exchange in &exchanges {
            if exchange.provider_id == owner_id {
                if let Some(rating) = exchange.provider_rating {
                    total_ratings += rating;
                    rating_count += 1;
                }
            }
            if exchange.requester_id == Some(owner_id) {
                if let Some(rating) = exchange.requester_rating {
                    total_ratings += rating;
                    rating_count += 1;
                }
            }
        }

        let average_rating = if rating_count > 0 {
            Some(total_ratings as f32 / rating_count as f32)
        } else {
            None
        };

        // Get recent 5 exchanges
        let recent: Vec<_> = exchanges.into_iter().take(5).collect();
        let recent_dtos = self.enrich_exchanges_with_names(recent).await?;

        Ok(OwnerExchangeSummaryDto {
            owner_id,
            owner_name,
            as_provider,
            as_requester,
            total_exchanges: as_provider + as_requester,
            average_rating,
            recent_exchanges: recent_dtos,
        })
    }

    // Private helper methods

    /// Update credit balances when exchange is completed
    async fn update_credit_balances_on_completion(
        &self,
        exchange: &LocalExchange,
    ) -> Result<(), String> {
        let requester_id = exchange
            .requester_id
            .ok_or("Exchange has no requester".to_string())?;

        // Provider earns credits
        let mut provider_balance = self
            .balance_repo
            .get_or_create(exchange.provider_id, exchange.building_id)
            .await?;
        provider_balance.earn_credits(exchange.credits)?;
        self.balance_repo.update(&provider_balance).await?;

        // Requester spends credits
        let mut requester_balance = self
            .balance_repo
            .get_or_create(requester_id, exchange.building_id)
            .await?;
        requester_balance.spend_credits(exchange.credits)?;
        self.balance_repo.update(&requester_balance).await?;

        Ok(())
    }

    /// Update average rating for an owner
    async fn update_average_rating(&self, owner_id: Uuid, building_id: Uuid) -> Result<(), String> {
        let exchanges = self.exchange_repo.find_by_owner(owner_id).await?;

        let mut total_ratings = 0;
        let mut rating_count = 0;

        for exchange in exchanges {
            if exchange.provider_id == owner_id {
                if let Some(rating) = exchange.provider_rating {
                    total_ratings += rating;
                    rating_count += 1;
                }
            }
            if exchange.requester_id == Some(owner_id) {
                if let Some(rating) = exchange.requester_rating {
                    total_ratings += rating;
                    rating_count += 1;
                }
            }
        }

        if rating_count > 0 {
            let average = total_ratings as f32 / rating_count as f32;

            let mut balance = self
                .balance_repo
                .get_or_create(owner_id, building_id)
                .await?;
            balance.update_rating(average)?;
            self.balance_repo.update(&balance).await?;
        }

        Ok(())
    }

    /// Enrich exchanges with owner names (provider + requester)
    async fn enrich_exchanges_with_names(
        &self,
        exchanges: Vec<LocalExchange>,
    ) -> Result<Vec<LocalExchangeResponseDto>, String> {
        let mut dtos = Vec::new();

        for exchange in exchanges {
            // Get provider name
            let provider = self.owner_repo.find_by_id(exchange.provider_id).await?;
            let provider_name = if let Some(p) = provider {
                format!("{} {}", p.first_name, p.last_name)
            } else {
                "Unknown".to_string()
            };

            // Get requester name if exists
            let requester_name = if let Some(requester_id) = exchange.requester_id {
                let requester = self.owner_repo.find_by_id(requester_id).await?;
                requester.map(|r| format!("{} {}", r.first_name, r.last_name))
            } else {
                None
            };

            dtos.push(LocalExchangeResponseDto::from_entity(
                exchange,
                provider_name,
                requester_name,
            ));
        }

        Ok(dtos)
    }
}
