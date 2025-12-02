use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::ports::EnergyCampaignRepository;
use crate::domain::entities::{
    CampaignStatus, CampaignType, ContractType, EnergyCampaign, EnergyType, ProviderOffer,
};

pub struct PostgresEnergyCampaignRepository {
    pub pool: PgPool,
}

impl PostgresEnergyCampaignRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EnergyCampaignRepository for PostgresEnergyCampaignRepository {
    async fn create(&self, campaign: &EnergyCampaign) -> Result<EnergyCampaign, String> {
        let energy_types_str: Vec<String> = campaign
            .energy_types
            .iter()
            .map(|t| t.to_string())
            .collect();

        let _result = sqlx::query!(
            r#"
            INSERT INTO energy_campaigns (
                id, organization_id, building_id, campaign_name, campaign_type, status,
                deadline_participation, deadline_vote, contract_start_date,
                energy_types, contract_duration_months, contract_type,
                total_participants, total_kwh_electricity, total_kwh_gas, avg_kwh_per_unit,
                selected_offer_id, estimated_savings_pct, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING id
            "#,
            campaign.id,
            campaign.organization_id,
            campaign.building_id,
            campaign.campaign_name,
            campaign.campaign_type.to_string(),
            campaign.status.to_string(),
            campaign.deadline_participation,
            campaign.deadline_vote,
            campaign.contract_start_date,
            &energy_types_str,
            campaign.contract_duration_months,
            campaign.contract_type.to_string(),
            campaign.total_participants,
            campaign.total_kwh_electricity,
            campaign.total_kwh_gas,
            campaign.avg_kwh_per_unit,
            campaign.selected_offer_id,
            campaign.estimated_savings_pct,
            campaign.created_by,
            campaign.created_at,
            campaign.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create energy campaign: {}", e))?;

        Ok(campaign.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<EnergyCampaign>, String> {
        let row = sqlx::query!(
            r#"
            SELECT
                id, organization_id, building_id, campaign_name, campaign_type, status,
                deadline_participation, deadline_vote, contract_start_date,
                energy_types, contract_duration_months, contract_type,
                total_participants, total_kwh_electricity, total_kwh_gas, avg_kwh_per_unit,
                selected_offer_id, estimated_savings_pct, created_by, created_at, updated_at
            FROM energy_campaigns
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find energy campaign: {}", e))?;

        match row {
            Some(r) => {
                let energy_types: Vec<EnergyType> = r
                    .energy_types
                    .iter()
                    .map(|s| s.parse().unwrap_or(EnergyType::Electricity))
                    .collect();

                // Fetch offers
                let offers = self.get_offers(id).await?;

                Ok(Some(EnergyCampaign {
                    id: r.id,
                    organization_id: r.organization_id,
                    building_id: r.building_id,
                    campaign_name: r.campaign_name,
                    campaign_type: r.campaign_type.parse().unwrap_or(CampaignType::BuyingGroup),
                    status: r.status.parse().unwrap_or(CampaignStatus::Draft),
                    deadline_participation: r.deadline_participation,
                    deadline_vote: r.deadline_vote,
                    contract_start_date: r.contract_start_date,
                    energy_types,
                    contract_duration_months: r.contract_duration_months,
                    contract_type: r.contract_type.parse().unwrap_or(ContractType::Fixed),
                    total_participants: r.total_participants,
                    total_kwh_electricity: r.total_kwh_electricity,
                    total_kwh_gas: r.total_kwh_gas,
                    avg_kwh_per_unit: r.avg_kwh_per_unit,
                    offers_received: offers,
                    selected_offer_id: r.selected_offer_id,
                    estimated_savings_pct: r.estimated_savings_pct,
                    created_by: r.created_by,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<EnergyCampaign>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, organization_id, building_id, campaign_name, campaign_type, status,
                deadline_participation, deadline_vote, contract_start_date,
                energy_types, contract_duration_months, contract_type,
                total_participants, total_kwh_electricity, total_kwh_gas, avg_kwh_per_unit,
                selected_offer_id, estimated_savings_pct, created_by, created_at, updated_at
            FROM energy_campaigns
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find campaigns by organization: {}", e))?;

        let mut campaigns = Vec::new();
        for r in rows {
            let energy_types: Vec<EnergyType> = r
                .energy_types
                .iter()
                .map(|s| s.parse().unwrap_or(EnergyType::Electricity))
                .collect();

            let offers = self.get_offers(r.id).await?;

            campaigns.push(EnergyCampaign {
                id: r.id,
                organization_id: r.organization_id,
                building_id: r.building_id,
                campaign_name: r.campaign_name,
                campaign_type: r.campaign_type.parse().unwrap_or(CampaignType::BuyingGroup),
                status: r.status.parse().unwrap_or(CampaignStatus::Draft),
                deadline_participation: r.deadline_participation,
                deadline_vote: r.deadline_vote,
                contract_start_date: r.contract_start_date,
                energy_types,
                contract_duration_months: r.contract_duration_months,
                contract_type: r.contract_type.parse().unwrap_or(ContractType::Fixed),
                total_participants: r.total_participants,
                total_kwh_electricity: r.total_kwh_electricity,
                total_kwh_gas: r.total_kwh_gas,
                avg_kwh_per_unit: r.avg_kwh_per_unit,
                offers_received: offers,
                selected_offer_id: r.selected_offer_id,
                estimated_savings_pct: r.estimated_savings_pct,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
            });
        }

        Ok(campaigns)
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<EnergyCampaign>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, organization_id, building_id, campaign_name, campaign_type, status,
                deadline_participation, deadline_vote, contract_start_date,
                energy_types, contract_duration_months, contract_type,
                total_participants, total_kwh_electricity, total_kwh_gas, avg_kwh_per_unit,
                selected_offer_id, estimated_savings_pct, created_by, created_at, updated_at
            FROM energy_campaigns
            WHERE building_id = $1
            ORDER BY created_at DESC
            "#,
            building_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find campaigns by building: {}", e))?;

        let mut campaigns = Vec::new();
        for r in rows {
            let energy_types: Vec<EnergyType> = r
                .energy_types
                .iter()
                .map(|s| s.parse().unwrap_or(EnergyType::Electricity))
                .collect();

            let offers = self.get_offers(r.id).await?;

            campaigns.push(EnergyCampaign {
                id: r.id,
                organization_id: r.organization_id,
                building_id: r.building_id,
                campaign_name: r.campaign_name,
                campaign_type: r.campaign_type.parse().unwrap_or(CampaignType::BuyingGroup),
                status: r.status.parse().unwrap_or(CampaignStatus::Draft),
                deadline_participation: r.deadline_participation,
                deadline_vote: r.deadline_vote,
                contract_start_date: r.contract_start_date,
                energy_types,
                contract_duration_months: r.contract_duration_months,
                contract_type: r.contract_type.parse().unwrap_or(ContractType::Fixed),
                total_participants: r.total_participants,
                total_kwh_electricity: r.total_kwh_electricity,
                total_kwh_gas: r.total_kwh_gas,
                avg_kwh_per_unit: r.avg_kwh_per_unit,
                offers_received: offers,
                selected_offer_id: r.selected_offer_id,
                estimated_savings_pct: r.estimated_savings_pct,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
            });
        }

        Ok(campaigns)
    }

    async fn update(&self, campaign: &EnergyCampaign) -> Result<EnergyCampaign, String> {
        let energy_types_str: Vec<String> = campaign
            .energy_types
            .iter()
            .map(|t| t.to_string())
            .collect();

        sqlx::query!(
            r#"
            UPDATE energy_campaigns
            SET
                campaign_name = $2,
                campaign_type = $3,
                status = $4,
                deadline_participation = $5,
                deadline_vote = $6,
                contract_start_date = $7,
                energy_types = $8,
                contract_duration_months = $9,
                contract_type = $10,
                total_participants = $11,
                total_kwh_electricity = $12,
                total_kwh_gas = $13,
                avg_kwh_per_unit = $14,
                selected_offer_id = $15,
                estimated_savings_pct = $16,
                updated_at = $17
            WHERE id = $1
            "#,
            campaign.id,
            campaign.campaign_name,
            campaign.campaign_type.to_string(),
            campaign.status.to_string(),
            campaign.deadline_participation,
            campaign.deadline_vote,
            campaign.contract_start_date,
            &energy_types_str,
            campaign.contract_duration_months,
            campaign.contract_type.to_string(),
            campaign.total_participants,
            campaign.total_kwh_electricity,
            campaign.total_kwh_gas,
            campaign.avg_kwh_per_unit,
            campaign.selected_offer_id,
            campaign.estimated_savings_pct,
            campaign.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update energy campaign: {}", e))?;

        Ok(campaign.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        sqlx::query!(
            r#"
            DELETE FROM energy_campaigns
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete energy campaign: {}", e))?;

        Ok(())
    }

    async fn add_offer(
        &self,
        campaign_id: Uuid,
        offer: &ProviderOffer,
    ) -> Result<ProviderOffer, String> {
        sqlx::query!(
            r#"
            INSERT INTO provider_offers (
                id, campaign_id, provider_name, price_kwh_electricity, price_kwh_gas,
                fixed_monthly_fee, green_energy_pct, contract_duration_months,
                estimated_savings_pct, offer_valid_until, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            offer.id,
            campaign_id,
            offer.provider_name,
            offer.price_kwh_electricity,
            offer.price_kwh_gas,
            offer.fixed_monthly_fee,
            offer.green_energy_pct,
            offer.contract_duration_months,
            offer.estimated_savings_pct,
            offer.offer_valid_until,
            offer.created_at,
            offer.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to add provider offer: {}", e))?;

        Ok(offer.clone())
    }

    async fn get_offers(&self, campaign_id: Uuid) -> Result<Vec<ProviderOffer>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, campaign_id, provider_name, price_kwh_electricity, price_kwh_gas,
                fixed_monthly_fee, green_energy_pct, contract_duration_months,
                estimated_savings_pct, offer_valid_until, created_at, updated_at
            FROM provider_offers
            WHERE campaign_id = $1
            ORDER BY estimated_savings_pct DESC
            "#,
            campaign_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get provider offers: {}", e))?;

        let offers = rows
            .into_iter()
            .map(|r| ProviderOffer {
                id: r.id,
                campaign_id: r.campaign_id,
                provider_name: r.provider_name,
                price_kwh_electricity: r.price_kwh_electricity,
                price_kwh_gas: r.price_kwh_gas,
                fixed_monthly_fee: r.fixed_monthly_fee,
                green_energy_pct: r.green_energy_pct,
                contract_duration_months: r.contract_duration_months,
                estimated_savings_pct: r.estimated_savings_pct,
                offer_valid_until: r.offer_valid_until,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(offers)
    }

    async fn update_offer(&self, offer: &ProviderOffer) -> Result<ProviderOffer, String> {
        sqlx::query!(
            r#"
            UPDATE provider_offers
            SET
                provider_name = $2,
                price_kwh_electricity = $3,
                price_kwh_gas = $4,
                fixed_monthly_fee = $5,
                green_energy_pct = $6,
                contract_duration_months = $7,
                estimated_savings_pct = $8,
                offer_valid_until = $9,
                updated_at = $10
            WHERE id = $1
            "#,
            offer.id,
            offer.provider_name,
            offer.price_kwh_electricity,
            offer.price_kwh_gas,
            offer.fixed_monthly_fee,
            offer.green_energy_pct,
            offer.contract_duration_months,
            offer.estimated_savings_pct,
            offer.offer_valid_until,
            offer.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update provider offer: {}", e))?;

        Ok(offer.clone())
    }

    async fn delete_offer(&self, offer_id: Uuid) -> Result<(), String> {
        sqlx::query!(
            r#"
            DELETE FROM provider_offers
            WHERE id = $1
            "#,
            offer_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete provider offer: {}", e))?;

        Ok(())
    }

    async fn find_offer_by_id(&self, offer_id: Uuid) -> Result<Option<ProviderOffer>, String> {
        let row = sqlx::query!(
            r#"
            SELECT
                id, campaign_id, provider_name, price_kwh_electricity, price_kwh_gas,
                fixed_monthly_fee, green_energy_pct, contract_duration_months,
                estimated_savings_pct, offer_valid_until, created_at, updated_at
            FROM provider_offers
            WHERE id = $1
            "#,
            offer_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find provider offer: {}", e))?;

        Ok(row.map(|r| ProviderOffer {
            id: r.id,
            campaign_id: r.campaign_id,
            provider_name: r.provider_name,
            price_kwh_electricity: r.price_kwh_electricity,
            price_kwh_gas: r.price_kwh_gas,
            fixed_monthly_fee: r.fixed_monthly_fee,
            green_energy_pct: r.green_energy_pct,
            contract_duration_months: r.contract_duration_months,
            estimated_savings_pct: r.estimated_savings_pct,
            offer_valid_until: r.offer_valid_until,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn update_aggregation(
        &self,
        campaign_id: Uuid,
        total_kwh_electricity: Option<f64>,
        total_kwh_gas: Option<f64>,
        avg_kwh_per_unit: Option<f64>,
    ) -> Result<(), String> {
        sqlx::query!(
            r#"
            UPDATE energy_campaigns
            SET
                total_kwh_electricity = $2,
                total_kwh_gas = $3,
                avg_kwh_per_unit = $4,
                updated_at = NOW()
            WHERE id = $1
            "#,
            campaign_id,
            total_kwh_electricity,
            total_kwh_gas,
            avg_kwh_per_unit,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update campaign aggregation: {}", e))?;

        Ok(())
    }
}
