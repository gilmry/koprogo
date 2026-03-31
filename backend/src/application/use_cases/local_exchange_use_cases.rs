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

    /// Resolve a user_id (from auth) to an owner_id (from owners table)
    async fn resolve_owner_id(&self, user_id: Uuid) -> Result<Uuid, String> {
        let owner = self
            .owner_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or("User is not linked to an owner account".to_string())?;
        Ok(owner.id)
    }

    /// Create a new exchange offer
    pub async fn create_exchange(
        &self,
        user_id: Uuid, // From auth (user_id, resolved to owner_id internally)
        dto: CreateLocalExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        // Resolve user_id → owner
        let provider = self
            .owner_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or("Provider not found - user is not linked to an owner account".to_string())?;

        // Create domain entity using owner's actual ID
        let exchange = LocalExchange::new(
            dto.building_id,
            provider.id,
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
        user_id: Uuid, // From auth (user_id, resolved to owner_id internally)
        _dto: RequestExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        let owner_id = self.resolve_owner_id(user_id).await?;

        // Load exchange
        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        // Apply business logic
        exchange.request(owner_id)?;

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
        user_id: Uuid, // From auth (user_id, resolved to owner_id internally)
    ) -> Result<LocalExchangeResponseDto, String> {
        let owner_id = self.resolve_owner_id(user_id).await?;

        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        exchange.start(owner_id)?;

        let updated = self.exchange_repo.update(&exchange).await?;

        self.get_exchange(updated.id).await
    }

    /// Complete an exchange (transition: InProgress → Completed)
    /// Updates credit balances for both parties
    pub async fn complete_exchange(
        &self,
        exchange_id: Uuid,
        user_id: Uuid, // From auth (user_id, resolved to owner_id internally)
        _dto: CompleteExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        let owner_id = self.resolve_owner_id(user_id).await?;

        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        // Get requester before completing (we'll need it for balance update)
        let requester_id = exchange
            .requester_id
            .ok_or("Exchange has no requester".to_string())?;

        exchange.complete(owner_id)?;

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
        user_id: Uuid, // From auth (user_id, resolved to owner_id internally)
        dto: CancelExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        let owner_id = self.resolve_owner_id(user_id).await?;

        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        exchange.cancel(owner_id, dto.reason)?;

        let updated = self.exchange_repo.update(&exchange).await?;

        self.get_exchange(updated.id).await
    }

    /// Rate the provider (by requester)
    pub async fn rate_provider(
        &self,
        exchange_id: Uuid,
        user_id: Uuid, // From auth (user_id, resolved to owner_id internally)
        dto: RateExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        let owner_id = self.resolve_owner_id(user_id).await?;

        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        exchange.rate_provider(owner_id, dto.rating)?;

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
        user_id: Uuid, // From auth (user_id, resolved to owner_id internally)
        dto: RateExchangeDto,
    ) -> Result<LocalExchangeResponseDto, String> {
        let owner_id = self.resolve_owner_id(user_id).await?;

        let mut exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        exchange.rate_requester(owner_id, dto.rating)?;

        let updated = self.exchange_repo.update(&exchange).await?;

        // Update requester's average rating
        if let Some(requester_id) = updated.requester_id {
            self.update_average_rating(requester_id, updated.building_id)
                .await?;
        }

        self.get_exchange(updated.id).await
    }

    /// Delete an exchange (only if not completed)
    pub async fn delete_exchange(&self, exchange_id: Uuid, user_id: Uuid) -> Result<(), String> {
        let owner_id = self.resolve_owner_id(user_id).await?;

        let exchange = self
            .exchange_repo
            .find_by_id(exchange_id)
            .await?
            .ok_or("Exchange not found".to_string())?;

        // Only provider can delete
        if exchange.provider_id != owner_id {
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
            .count_by_building_and_type(building_id, ExchangeType::Service.to_sql())
            .await?;
        let object_loan_count = self
            .exchange_repo
            .count_by_building_and_type(building_id, ExchangeType::ObjectLoan.to_sql())
            .await?;
        let shared_purchase_count = self
            .exchange_repo
            .count_by_building_and_type(building_id, ExchangeType::SharedPurchase.to_sql())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{OwnerFilters, PageRequest};
    use crate::application::ports::{
        LocalExchangeRepository, OwnerCreditBalanceRepository, OwnerRepository,
    };
    use crate::domain::entities::{
        ExchangeStatus, ExchangeType, LocalExchange, Owner, OwnerCreditBalance,
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ── Mock OwnerRepository ────────────────────────────────────────────

    struct MockOwnerRepository {
        owners: Mutex<HashMap<Uuid, Owner>>,
    }

    impl MockOwnerRepository {
        fn new() -> Self {
            Self {
                owners: Mutex::new(HashMap::new()),
            }
        }

        fn insert(&self, owner: Owner) {
            self.owners.lock().unwrap().insert(owner.id, owner);
        }
    }

    #[async_trait]
    impl OwnerRepository for MockOwnerRepository {
        async fn create(&self, owner: &Owner) -> Result<Owner, String> {
            self.owners.lock().unwrap().insert(owner.id, owner.clone());
            Ok(owner.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, String> {
            Ok(self.owners.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Owner>, String> {
            Ok(self
                .owners
                .lock()
                .unwrap()
                .values()
                .find(|o| o.user_id == Some(user_id))
                .cloned())
        }

        async fn find_by_user_id_and_organization(
            &self,
            user_id: Uuid,
            organization_id: Uuid,
        ) -> Result<Option<Owner>, String> {
            Ok(self
                .owners
                .lock()
                .unwrap()
                .values()
                .find(|o| o.user_id == Some(user_id) && o.organization_id == organization_id)
                .cloned())
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<Owner>, String> {
            Ok(self
                .owners
                .lock()
                .unwrap()
                .values()
                .find(|o| o.email == email)
                .cloned())
        }

        async fn find_all(&self) -> Result<Vec<Owner>, String> {
            Ok(self.owners.lock().unwrap().values().cloned().collect())
        }

        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _filters: &OwnerFilters,
        ) -> Result<(Vec<Owner>, i64), String> {
            let owners: Vec<_> = self.owners.lock().unwrap().values().cloned().collect();
            let total = owners.len() as i64;
            Ok((owners, total))
        }

        async fn update(&self, owner: &Owner) -> Result<Owner, String> {
            self.owners.lock().unwrap().insert(owner.id, owner.clone());
            Ok(owner.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.owners.lock().unwrap().remove(&id).is_some())
        }
        async fn set_user_link(&self, owner_id: Uuid, user_id: Option<Uuid>) -> Result<bool, String> {
            let mut map = self.owners.lock().unwrap();
            if let Some(o) = map.get_mut(&owner_id) { o.user_id = user_id; Ok(true) } else { Ok(false) }
        }
    }

    // ── Mock LocalExchangeRepository ────────────────────────────────────

    struct MockLocalExchangeRepository {
        exchanges: Mutex<HashMap<Uuid, LocalExchange>>,
    }

    impl MockLocalExchangeRepository {
        fn new() -> Self {
            Self {
                exchanges: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl LocalExchangeRepository for MockLocalExchangeRepository {
        async fn create(&self, exchange: &LocalExchange) -> Result<LocalExchange, String> {
            self.exchanges
                .lock()
                .unwrap()
                .insert(exchange.id, exchange.clone());
            Ok(exchange.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<LocalExchange>, String> {
            Ok(self.exchanges.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<LocalExchange>, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_building_and_status(
            &self,
            building_id: Uuid,
            status: &str,
        ) -> Result<Vec<LocalExchange>, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.building_id == building_id && e.status.to_sql() == status)
                .cloned()
                .collect())
        }

        async fn find_by_provider(&self, provider_id: Uuid) -> Result<Vec<LocalExchange>, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.provider_id == provider_id)
                .cloned()
                .collect())
        }

        async fn find_by_requester(
            &self,
            requester_id: Uuid,
        ) -> Result<Vec<LocalExchange>, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.requester_id == Some(requester_id))
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<LocalExchange>, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.provider_id == owner_id || e.requester_id == Some(owner_id))
                .cloned()
                .collect())
        }

        async fn find_active_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<LocalExchange>, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.building_id == building_id && e.is_active())
                .cloned()
                .collect())
        }

        async fn find_available_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<LocalExchange>, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.building_id == building_id && e.status == ExchangeStatus::Offered)
                .cloned()
                .collect())
        }

        async fn find_by_type(
            &self,
            building_id: Uuid,
            exchange_type: &str,
        ) -> Result<Vec<LocalExchange>, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| {
                    e.building_id == building_id && e.exchange_type.to_sql() == exchange_type
                })
                .cloned()
                .collect())
        }

        async fn update(&self, exchange: &LocalExchange) -> Result<LocalExchange, String> {
            self.exchanges
                .lock()
                .unwrap()
                .insert(exchange.id, exchange.clone());
            Ok(exchange.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.exchanges.lock().unwrap().remove(&id).is_some())
        }

        async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.building_id == building_id)
                .count() as i64)
        }

        async fn count_by_building_and_status(
            &self,
            building_id: Uuid,
            status: &str,
        ) -> Result<i64, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.building_id == building_id && e.status.to_sql() == status)
                .count() as i64)
        }

        async fn count_by_building_and_type(
            &self,
            building_id: Uuid,
            exchange_type: &str,
        ) -> Result<i64, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| {
                    e.building_id == building_id && e.exchange_type.to_sql() == exchange_type
                })
                .count() as i64)
        }

        async fn get_total_credits_exchanged(&self, building_id: Uuid) -> Result<i32, String> {
            Ok(self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.building_id == building_id && e.status == ExchangeStatus::Completed)
                .map(|e| e.credits)
                .sum())
        }

        async fn get_average_exchange_rating(
            &self,
            building_id: Uuid,
        ) -> Result<Option<f32>, String> {
            let ratings: Vec<f32> = self
                .exchanges
                .lock()
                .unwrap()
                .values()
                .filter(|e| e.building_id == building_id)
                .flat_map(|e| {
                    let mut v = Vec::new();
                    if let Some(r) = e.provider_rating {
                        v.push(r as f32);
                    }
                    if let Some(r) = e.requester_rating {
                        v.push(r as f32);
                    }
                    v
                })
                .collect();
            if ratings.is_empty() {
                Ok(None)
            } else {
                Ok(Some(ratings.iter().sum::<f32>() / ratings.len() as f32))
            }
        }
    }

    // ── Mock OwnerCreditBalanceRepository ────────────────────────────────

    struct MockOwnerCreditBalanceRepository {
        balances: Mutex<HashMap<(Uuid, Uuid), OwnerCreditBalance>>,
    }

    impl MockOwnerCreditBalanceRepository {
        fn new() -> Self {
            Self {
                balances: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl OwnerCreditBalanceRepository for MockOwnerCreditBalanceRepository {
        async fn create(&self, balance: &OwnerCreditBalance) -> Result<OwnerCreditBalance, String> {
            self.balances
                .lock()
                .unwrap()
                .insert((balance.owner_id, balance.building_id), balance.clone());
            Ok(balance.clone())
        }

        async fn find_by_owner_and_building(
            &self,
            owner_id: Uuid,
            building_id: Uuid,
        ) -> Result<Option<OwnerCreditBalance>, String> {
            Ok(self
                .balances
                .lock()
                .unwrap()
                .get(&(owner_id, building_id))
                .cloned())
        }

        async fn find_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<OwnerCreditBalance>, String> {
            Ok(self
                .balances
                .lock()
                .unwrap()
                .values()
                .filter(|b| b.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<OwnerCreditBalance>, String> {
            Ok(self
                .balances
                .lock()
                .unwrap()
                .values()
                .filter(|b| b.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn get_or_create(
            &self,
            owner_id: Uuid,
            building_id: Uuid,
        ) -> Result<OwnerCreditBalance, String> {
            let mut map = self.balances.lock().unwrap();
            if let Some(existing) = map.get(&(owner_id, building_id)) {
                Ok(existing.clone())
            } else {
                let balance = OwnerCreditBalance::new(owner_id, building_id);
                map.insert((owner_id, building_id), balance.clone());
                Ok(balance)
            }
        }

        async fn update(&self, balance: &OwnerCreditBalance) -> Result<OwnerCreditBalance, String> {
            self.balances
                .lock()
                .unwrap()
                .insert((balance.owner_id, balance.building_id), balance.clone());
            Ok(balance.clone())
        }

        async fn delete(&self, owner_id: Uuid, building_id: Uuid) -> Result<bool, String> {
            Ok(self
                .balances
                .lock()
                .unwrap()
                .remove(&(owner_id, building_id))
                .is_some())
        }

        async fn get_leaderboard(
            &self,
            building_id: Uuid,
            limit: i32,
        ) -> Result<Vec<OwnerCreditBalance>, String> {
            let mut balances: Vec<_> = self
                .balances
                .lock()
                .unwrap()
                .values()
                .filter(|b| b.building_id == building_id)
                .cloned()
                .collect();
            balances.sort_by_key(|a| std::cmp::Reverse(a.balance));
            balances.truncate(limit as usize);
            Ok(balances)
        }

        async fn count_active_participants(&self, building_id: Uuid) -> Result<i64, String> {
            Ok(self
                .balances
                .lock()
                .unwrap()
                .values()
                .filter(|b| b.building_id == building_id && b.total_exchanges > 0)
                .count() as i64)
        }

        async fn get_total_credits_in_circulation(&self, building_id: Uuid) -> Result<i32, String> {
            Ok(self
                .balances
                .lock()
                .unwrap()
                .values()
                .filter(|b| b.building_id == building_id)
                .map(|b| b.credits_earned)
                .sum())
        }
    }

    // ── Test Helpers ────────────────────────────────────────────────────

    fn make_owner(user_id: Uuid) -> Owner {
        let now = Utc::now();
        Owner {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            user_id: Some(user_id),
            first_name: "Jean".to_string(),
            last_name: "Dupont".to_string(),
            email: "jean@example.com".to_string(),
            phone: None,
            address: "1 rue de la Loi".to_string(),
            city: "Bruxelles".to_string(),
            postal_code: "1000".to_string(),
            country: "BE".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    fn make_owner_with_name(user_id: Uuid, first: &str, last: &str) -> Owner {
        let now = Utc::now();
        Owner {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            user_id: Some(user_id),
            first_name: first.to_string(),
            last_name: last.to_string(),
            email: format!("{}@example.com", first.to_lowercase()),
            phone: None,
            address: "1 rue de la Loi".to_string(),
            city: "Bruxelles".to_string(),
            postal_code: "1000".to_string(),
            country: "BE".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    fn setup_use_cases(
        owner_repo: Arc<MockOwnerRepository>,
        exchange_repo: Arc<MockLocalExchangeRepository>,
        balance_repo: Arc<MockOwnerCreditBalanceRepository>,
    ) -> LocalExchangeUseCases {
        LocalExchangeUseCases::new(exchange_repo, balance_repo, owner_repo)
    }

    // ── Tests ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_exchange_success() {
        let owner_repo = Arc::new(MockOwnerRepository::new());
        let exchange_repo = Arc::new(MockLocalExchangeRepository::new());
        let balance_repo = Arc::new(MockOwnerCreditBalanceRepository::new());

        let user_id = Uuid::new_v4();
        let provider = make_owner_with_name(user_id, "Alice", "Martin");
        owner_repo.insert(provider.clone());

        let building_id = Uuid::new_v4();
        let uc = setup_use_cases(owner_repo, exchange_repo, balance_repo);

        let dto = CreateLocalExchangeDto {
            building_id,
            exchange_type: ExchangeType::Service,
            title: "Gardening help".to_string(),
            description: "I can help with your garden".to_string(),
            credits: 3,
        };

        let result = uc.create_exchange(user_id, dto).await;
        assert!(result.is_ok(), "create_exchange failed: {:?}", result.err());

        let resp = result.unwrap();
        assert_eq!(resp.building_id, building_id);
        assert_eq!(resp.provider_id, provider.id);
        assert_eq!(resp.provider_name, "Alice Martin");
        assert_eq!(resp.status, ExchangeStatus::Offered);
        assert_eq!(resp.credits, 3);
        assert_eq!(resp.title, "Gardening help");
        assert!(resp.requester_id.is_none());
    }

    #[tokio::test]
    async fn test_request_exchange_success() {
        let owner_repo = Arc::new(MockOwnerRepository::new());
        let exchange_repo = Arc::new(MockLocalExchangeRepository::new());
        let balance_repo = Arc::new(MockOwnerCreditBalanceRepository::new());

        let provider_user_id = Uuid::new_v4();
        let requester_user_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let provider = make_owner_with_name(provider_user_id, "Alice", "Martin");
        let requester = make_owner_with_name(requester_user_id, "Bob", "Leroy");
        owner_repo.insert(provider.clone());
        owner_repo.insert(requester.clone());

        let uc = setup_use_cases(owner_repo, exchange_repo.clone(), balance_repo);

        // Create an exchange first
        let dto = CreateLocalExchangeDto {
            building_id,
            exchange_type: ExchangeType::ObjectLoan,
            title: "Drill to lend".to_string(),
            description: "Heavy-duty drill available".to_string(),
            credits: 1,
        };
        let created = uc.create_exchange(provider_user_id, dto).await.unwrap();

        // Now requester requests it
        let result = uc
            .request_exchange(created.id, requester_user_id, RequestExchangeDto {})
            .await;
        assert!(
            result.is_ok(),
            "request_exchange failed: {:?}",
            result.err()
        );

        let resp = result.unwrap();
        assert_eq!(resp.status, ExchangeStatus::Requested);
        assert_eq!(resp.requester_id, Some(requester.id));
        assert_eq!(resp.requester_name, Some("Bob Leroy".to_string()));
    }

    #[tokio::test]
    async fn test_start_exchange_success() {
        let owner_repo = Arc::new(MockOwnerRepository::new());
        let exchange_repo = Arc::new(MockLocalExchangeRepository::new());
        let balance_repo = Arc::new(MockOwnerCreditBalanceRepository::new());

        let provider_user_id = Uuid::new_v4();
        let requester_user_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let provider = make_owner_with_name(provider_user_id, "Alice", "Martin");
        let requester = make_owner_with_name(requester_user_id, "Bob", "Leroy");
        owner_repo.insert(provider.clone());
        owner_repo.insert(requester.clone());

        let uc = setup_use_cases(owner_repo, exchange_repo.clone(), balance_repo);

        // Create + request
        let dto = CreateLocalExchangeDto {
            building_id,
            exchange_type: ExchangeType::Service,
            title: "Plumbing fix".to_string(),
            description: "Fix leaking pipe".to_string(),
            credits: 2,
        };
        let created = uc.create_exchange(provider_user_id, dto).await.unwrap();
        uc.request_exchange(created.id, requester_user_id, RequestExchangeDto {})
            .await
            .unwrap();

        // Provider starts the exchange
        let result = uc.start_exchange(created.id, provider_user_id).await;
        assert!(result.is_ok(), "start_exchange failed: {:?}", result.err());

        let resp = result.unwrap();
        assert_eq!(resp.status, ExchangeStatus::InProgress);
        assert!(resp.started_at.is_some());
    }

    #[tokio::test]
    async fn test_complete_exchange_updates_credit_balances() {
        let owner_repo = Arc::new(MockOwnerRepository::new());
        let exchange_repo = Arc::new(MockLocalExchangeRepository::new());
        let balance_repo = Arc::new(MockOwnerCreditBalanceRepository::new());

        let provider_user_id = Uuid::new_v4();
        let requester_user_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let provider = make_owner_with_name(provider_user_id, "Alice", "Martin");
        let requester = make_owner_with_name(requester_user_id, "Bob", "Leroy");
        owner_repo.insert(provider.clone());
        owner_repo.insert(requester.clone());

        let uc = setup_use_cases(
            owner_repo.clone(),
            exchange_repo.clone(),
            balance_repo.clone(),
        );

        // Create + request + start
        let credits = 5;
        let dto = CreateLocalExchangeDto {
            building_id,
            exchange_type: ExchangeType::Service,
            title: "IT Help".to_string(),
            description: "Install software".to_string(),
            credits,
        };
        let created = uc.create_exchange(provider_user_id, dto).await.unwrap();
        uc.request_exchange(created.id, requester_user_id, RequestExchangeDto {})
            .await
            .unwrap();
        uc.start_exchange(created.id, provider_user_id)
            .await
            .unwrap();

        // Provider completes the exchange
        let result = uc
            .complete_exchange(created.id, provider_user_id, CompleteExchangeDto {})
            .await;
        assert!(
            result.is_ok(),
            "complete_exchange failed: {:?}",
            result.err()
        );

        let resp = result.unwrap();
        assert_eq!(resp.status, ExchangeStatus::Completed);
        assert!(resp.completed_at.is_some());

        // Verify credit balances were updated
        let provider_balance = uc
            .get_credit_balance(provider.id, building_id)
            .await
            .unwrap();
        assert_eq!(provider_balance.credits_earned, credits);
        assert_eq!(provider_balance.balance, credits);
        assert_eq!(provider_balance.total_exchanges, 1);

        let requester_balance = uc
            .get_credit_balance(requester.id, building_id)
            .await
            .unwrap();
        assert_eq!(requester_balance.credits_spent, credits);
        assert_eq!(requester_balance.balance, -credits);
        assert_eq!(requester_balance.total_exchanges, 1);
    }

    #[tokio::test]
    async fn test_cancel_exchange_success() {
        let owner_repo = Arc::new(MockOwnerRepository::new());
        let exchange_repo = Arc::new(MockLocalExchangeRepository::new());
        let balance_repo = Arc::new(MockOwnerCreditBalanceRepository::new());

        let provider_user_id = Uuid::new_v4();
        let requester_user_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let provider = make_owner_with_name(provider_user_id, "Alice", "Martin");
        let requester = make_owner_with_name(requester_user_id, "Bob", "Leroy");
        owner_repo.insert(provider.clone());
        owner_repo.insert(requester.clone());

        let uc = setup_use_cases(owner_repo, exchange_repo, balance_repo);

        // Create + request
        let dto = CreateLocalExchangeDto {
            building_id,
            exchange_type: ExchangeType::SharedPurchase,
            title: "Bulk order".to_string(),
            description: "Shared cleaning supplies".to_string(),
            credits: 2,
        };
        let created = uc.create_exchange(provider_user_id, dto).await.unwrap();
        uc.request_exchange(created.id, requester_user_id, RequestExchangeDto {})
            .await
            .unwrap();

        // Requester cancels with reason
        let cancel_dto = CancelExchangeDto {
            reason: Some("Changed my mind".to_string()),
        };
        let result = uc
            .cancel_exchange(created.id, requester_user_id, cancel_dto)
            .await;
        assert!(result.is_ok(), "cancel_exchange failed: {:?}", result.err());

        let resp = result.unwrap();
        assert_eq!(resp.status, ExchangeStatus::Cancelled);
        assert!(resp.cancelled_at.is_some());
        assert_eq!(
            resp.cancellation_reason,
            Some("Changed my mind".to_string())
        );
    }

    #[tokio::test]
    async fn test_rate_provider_success() {
        let owner_repo = Arc::new(MockOwnerRepository::new());
        let exchange_repo = Arc::new(MockLocalExchangeRepository::new());
        let balance_repo = Arc::new(MockOwnerCreditBalanceRepository::new());

        let provider_user_id = Uuid::new_v4();
        let requester_user_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let provider = make_owner_with_name(provider_user_id, "Alice", "Martin");
        let requester = make_owner_with_name(requester_user_id, "Bob", "Leroy");
        owner_repo.insert(provider.clone());
        owner_repo.insert(requester.clone());

        let uc = setup_use_cases(owner_repo, exchange_repo.clone(), balance_repo.clone());

        // Full workflow: create + request + start + complete
        let dto = CreateLocalExchangeDto {
            building_id,
            exchange_type: ExchangeType::Service,
            title: "Painting".to_string(),
            description: "Paint the hallway".to_string(),
            credits: 4,
        };
        let created = uc.create_exchange(provider_user_id, dto).await.unwrap();
        uc.request_exchange(created.id, requester_user_id, RequestExchangeDto {})
            .await
            .unwrap();
        uc.start_exchange(created.id, provider_user_id)
            .await
            .unwrap();
        uc.complete_exchange(created.id, provider_user_id, CompleteExchangeDto {})
            .await
            .unwrap();

        // Requester rates provider
        let rate_dto = RateExchangeDto { rating: 5 };
        let result = uc
            .rate_provider(created.id, requester_user_id, rate_dto)
            .await;
        assert!(result.is_ok(), "rate_provider failed: {:?}", result.err());

        let resp = result.unwrap();
        assert_eq!(resp.provider_rating, Some(5));

        // Verify provider's average rating was updated in balance
        let provider_balance = balance_repo
            .find_by_owner_and_building(provider.id, building_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(provider_balance.average_rating, Some(5.0));
    }

    #[tokio::test]
    async fn test_rate_requester_success() {
        let owner_repo = Arc::new(MockOwnerRepository::new());
        let exchange_repo = Arc::new(MockLocalExchangeRepository::new());
        let balance_repo = Arc::new(MockOwnerCreditBalanceRepository::new());

        let provider_user_id = Uuid::new_v4();
        let requester_user_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let provider = make_owner_with_name(provider_user_id, "Alice", "Martin");
        let requester = make_owner_with_name(requester_user_id, "Bob", "Leroy");
        owner_repo.insert(provider.clone());
        owner_repo.insert(requester.clone());

        let uc = setup_use_cases(owner_repo, exchange_repo.clone(), balance_repo.clone());

        // Full workflow: create + request + start + complete
        let dto = CreateLocalExchangeDto {
            building_id,
            exchange_type: ExchangeType::Service,
            title: "Babysitting".to_string(),
            description: "Watch kids for 3 hours".to_string(),
            credits: 3,
        };
        let created = uc.create_exchange(provider_user_id, dto).await.unwrap();
        uc.request_exchange(created.id, requester_user_id, RequestExchangeDto {})
            .await
            .unwrap();
        uc.start_exchange(created.id, provider_user_id)
            .await
            .unwrap();
        uc.complete_exchange(created.id, provider_user_id, CompleteExchangeDto {})
            .await
            .unwrap();

        // Provider rates requester
        let rate_dto = RateExchangeDto { rating: 4 };
        let result = uc
            .rate_requester(created.id, provider_user_id, rate_dto)
            .await;
        assert!(result.is_ok(), "rate_requester failed: {:?}", result.err());

        let resp = result.unwrap();
        assert_eq!(resp.requester_rating, Some(4));

        // Verify requester's average rating was updated
        let requester_balance = balance_repo
            .find_by_owner_and_building(requester.id, building_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(requester_balance.average_rating, Some(4.0));
    }

    #[tokio::test]
    async fn test_get_credit_balance_creates_if_missing() {
        let owner_repo = Arc::new(MockOwnerRepository::new());
        let exchange_repo = Arc::new(MockLocalExchangeRepository::new());
        let balance_repo = Arc::new(MockOwnerCreditBalanceRepository::new());

        let user_id = Uuid::new_v4();
        let owner = make_owner(user_id);
        let owner_id = owner.id;
        let building_id = Uuid::new_v4();
        owner_repo.insert(owner);

        let uc = setup_use_cases(owner_repo, exchange_repo, balance_repo);

        // No balance exists yet; get_credit_balance should auto-create via get_or_create
        let result = uc.get_credit_balance(owner_id, building_id).await;
        assert!(
            result.is_ok(),
            "get_credit_balance failed: {:?}",
            result.err()
        );

        let balance = result.unwrap();
        assert_eq!(balance.owner_id, owner_id);
        assert_eq!(balance.building_id, building_id);
        assert_eq!(balance.credits_earned, 0);
        assert_eq!(balance.credits_spent, 0);
        assert_eq!(balance.balance, 0);
        assert_eq!(balance.total_exchanges, 0);
    }
}
