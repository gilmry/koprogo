use crate::application::ports::{
    AcpRepository, BuildingRepository, CallForFundsRepository, OwnerContributionRepository,
    UnitOwnerRepository,
};
use crate::domain::entities::{CallForFunds, ContributionType, OwnerContribution};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct CallForFundsUseCases {
    call_for_funds_repository: Arc<dyn CallForFundsRepository>,
    owner_contribution_repository: Arc<dyn OwnerContributionRepository>,
    unit_owner_repository: Arc<dyn UnitOwnerRepository>,
    /// Track H Story H7 — validate-before-compute ACP-level (FR-CL1). Optional.
    /// Quand présents (wiring `main.rs`), le pre-check `Acp::assert_conformant?`
    /// s'exécute avant `create_call_for_funds` / `send_call_for_funds`.
    building_repository: Option<Arc<dyn BuildingRepository>>,
    acp_repository: Option<Arc<dyn AcpRepository>>,
}

impl CallForFundsUseCases {
    pub fn new(
        call_for_funds_repository: Arc<dyn CallForFundsRepository>,
        owner_contribution_repository: Arc<dyn OwnerContributionRepository>,
        unit_owner_repository: Arc<dyn UnitOwnerRepository>,
    ) -> Self {
        Self {
            call_for_funds_repository,
            owner_contribution_repository,
            unit_owner_repository,
            building_repository: None,
            acp_repository: None,
        }
    }

    /// Track H Story H7 — wiring complet (validate-before-compute ACP-level).
    pub fn with_full_wiring(
        call_for_funds_repository: Arc<dyn CallForFundsRepository>,
        owner_contribution_repository: Arc<dyn OwnerContributionRepository>,
        unit_owner_repository: Arc<dyn UnitOwnerRepository>,
        building_repository: Arc<dyn BuildingRepository>,
        acp_repository: Arc<dyn AcpRepository>,
    ) -> Self {
        Self {
            call_for_funds_repository,
            owner_contribution_repository,
            unit_owner_repository,
            building_repository: Some(building_repository),
            acp_repository: Some(acp_repository),
        }
    }

    /// Résout l'ACP de l'immeuble, et vérifie sa conformité au passage.
    ///
    /// Deux responsabilités volontairement réunies : on ne peut pas vérifier
    /// la conformité d'une ACP sans l'avoir identifiée, et on ne veut pas
    /// appeler des fonds au nom d'une ACP dont l'acte de base ne boucle pas
    /// (Story H7, Art. 3.85 § 1er).
    ///
    /// **Le dépôt d'immeubles est requis.** Sans lui, on ne sait pas au nom de
    /// qui l'argent est appelé — et un appel de fonds dont on ignore le
    /// créancier n'a pas à exister. On échoue plutôt que de retomber sur
    /// l'identifiant du syndic, qui est précisément la confusion que
    /// l'ADR-0045 supprime.
    ///
    /// La vérification de conformité, elle, reste facultative : elle dépend du
    /// dépôt d'ACP, câblé séparément.
    async fn resoudre_lacp_conforme(&self, building_id: Uuid) -> Result<Uuid, String> {
        let Some(building_repo) = &self.building_repository else {
            return Err(
                "Impossible d'appeler des fonds : l'ACP créancière n'est pas résoluble \
                 (dépôt d'immeubles non câblé)"
                    .to_string(),
            );
        };
        let building = building_repo
            .find_by_id(building_id)
            .await?
            .ok_or_else(|| "Building not found".to_string())?;

        self.verifier_conformite(building.acp_id).await?;
        Ok(building.acp_id)
    }

    /// Vérifie que l'acte de base d'une ACP boucle, quand le dépôt est câblé.
    ///
    /// Facultatif à dessein : la conformité est un garde-fou de calcul
    /// (Story H7), pas une condition d'identité. Une ACP non conforme existe,
    /// elle n'est simplement pas en état qu'on réparte des charges dessus.
    async fn verifier_conformite(&self, acp_id: Uuid) -> Result<(), String> {
        let Some(acp_repo) = &self.acp_repository else {
            return Ok(());
        };
        let (acp, metrics) = acp_repo
            .find_by_id_with_metrics(acp_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "ACP not found".to_string())?;
        acp.assert_conformant(&metrics)?; // bridge String livré par H5
        Ok(())
    }

    /// Create a new call for funds
    #[allow(clippy::too_many_arguments)]
    pub async fn create_call_for_funds(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
        title: String,
        description: String,
        total_amount: rust_decimal::Decimal,
        contribution_type: ContributionType,
        call_date: DateTime<Utc>,
        due_date: DateTime<Utc>,
        account_code: Option<String>,
        created_by: Option<Uuid>,
        reserve_fund_share: rust_decimal::Decimal,
    ) -> Result<CallForFunds, String> {
        // Track H Story H2 — validate-before-compute gate (Art. 3.85 CC),
        // et résolution de l'ACP créancière (ADR-0045).
        let acp_id = self.resoudre_lacp_conforme(building_id).await?;

        // Create the call for funds entity
        let mut call_for_funds = CallForFunds::new(
            acp_id,
            organization_id,
            building_id,
            title,
            description,
            total_amount,
            contribution_type.clone(),
            call_date,
            due_date,
            account_code,
            reserve_fund_share,
        )?;

        call_for_funds.created_by = created_by;

        // Save to database
        self.call_for_funds_repository.create(&call_for_funds).await
    }

    /// Get a call for funds by ID
    pub async fn get_call_for_funds(&self, id: Uuid) -> Result<Option<CallForFunds>, String> {
        self.call_for_funds_repository.find_by_id(id).await
    }

    /// List all calls for funds for a building
    pub async fn list_by_building(&self, building_id: Uuid) -> Result<Vec<CallForFunds>, String> {
        self.call_for_funds_repository
            .find_by_building(building_id)
            .await
    }

    /// List all calls for funds for an organization
    pub async fn list_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<CallForFunds>, String> {
        self.call_for_funds_repository
            .find_by_organization(organization_id)
            .await
    }

    /// Mark call for funds as sent and generate individual owner contributions
    /// This is the key operation that automatically creates contributions for all owners
    pub async fn send_call_for_funds(&self, id: Uuid) -> Result<CallForFunds, String> {
        // Get the call for funds
        let mut call_for_funds = self
            .call_for_funds_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Call for funds not found".to_string())?;

        // Track H Story H2 — validate-before-compute gate (Art. 3.85 CC).
        // Le send génère les contributions — calcul interdit sur immeuble drift.
        //
        // On interroge l'ACP portée par l'appel, pas celle de l'immeuble : la
        // créance a été constituée au nom d'une ACP donnée, et c'est celle-là
        // qui doit être conforme au moment où on répartit.
        self.verifier_conformite(call_for_funds.acp_id).await?;

        // Mark as sent
        call_for_funds.mark_as_sent();

        // Update in database
        let updated_call = self
            .call_for_funds_repository
            .update(&call_for_funds)
            .await?;

        // Generate individual contributions for all owners in the building
        self.generate_owner_contributions(&updated_call).await?;

        Ok(updated_call)
    }

    /// Generate individual owner contributions based on ownership percentages
    async fn generate_owner_contributions(
        &self,
        call_for_funds: &CallForFunds,
    ) -> Result<Vec<OwnerContribution>, String> {
        // Quotes-parts de CHARGE, pas pourcentages de détention.
        //
        // `find_active_by_building` renvoyait le `ownership_percentage` brut :
        // 1.0 pour tout propriétaire unique de son lot, quel que soit le poids
        // du lot. Multiplié par le montant total, cela appelait le montant
        // ENTIER à chaque copropriétaire — un appel de 10 000 € sur un
        // immeuble conforme à 4 lots générait 4 quotes-parts de 10 000 €,
        // soit 40 000 € appelés, et répondait 200.
        //
        // Les tantièmes de l'acte de base étaient purement ignorés.
        let unit_owners = self
            .unit_owner_repository
            .find_active_quota_shares_by_building(call_for_funds.building_id)
            .await?;

        if unit_owners.is_empty() {
            return Err("No active owners found for this building".to_string());
        }

        let mut contributions = Vec::new();

        for (unit_id, owner_id, percentage) in unit_owners {
            // Calculate individual amount based on ownership percentage
            let individual_amount = call_for_funds.total_amount * percentage;

            // Create contribution description
            let description = format!(
                "{} - Quote-part: {}%",
                call_for_funds.title,
                percentage * rust_decimal_macros::dec!(100)
            );

            // Create owner contribution
            let mut contribution = OwnerContribution::new(
                // La quote-part est due à l'ACP créancière de l'appel, pas au
                // cabinet qui l'a émis (Art. 3.86 § 3, ADR-0045).
                call_for_funds.acp_id,
                call_for_funds.organization_id,
                owner_id,
                Some(unit_id),
                description,
                individual_amount,
                call_for_funds.contribution_type.clone(),
                call_for_funds.call_date,
                call_for_funds.account_code.clone(),
            )?;

            // Link to the call for funds
            contribution.call_for_funds_id = Some(call_for_funds.id);

            // Save contribution
            let saved = self
                .owner_contribution_repository
                .create(&contribution)
                .await?;

            contributions.push(saved);
        }

        Ok(contributions)
    }

    /// Cancel a call for funds
    pub async fn cancel_call_for_funds(&self, id: Uuid) -> Result<CallForFunds, String> {
        let mut call_for_funds = self
            .call_for_funds_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Call for funds not found".to_string())?;

        call_for_funds.cancel();

        self.call_for_funds_repository.update(&call_for_funds).await
    }

    /// Get all overdue calls for funds
    pub async fn get_overdue_calls(&self) -> Result<Vec<CallForFunds>, String> {
        self.call_for_funds_repository.find_overdue().await
    }

    /// Delete a call for funds (only if not sent)
    pub async fn delete_call_for_funds(&self, id: Uuid) -> Result<bool, String> {
        let call_for_funds = self
            .call_for_funds_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Call for funds not found".to_string())?;

        // Don't allow deletion if already sent
        if call_for_funds.status != crate::domain::entities::CallForFundsStatus::Draft {
            return Err("Cannot delete a call for funds that has been sent".to_string());
        }

        self.call_for_funds_repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::application::ports::{
        CallForFundsRepository, OwnerContributionRepository, UnitOwnerRepository,
    };
    use crate::domain::entities::{
        CallForFunds, CallForFundsStatus, ContributionType, OwnerContribution, UnitOwner,
    };
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    // ── Mock: CallForFundsRepository ──────────────────────────────────

    struct MockCallForFundsRepo {
        store: Mutex<HashMap<Uuid, CallForFunds>>,
        overdue: Mutex<Vec<CallForFunds>>,
    }

    impl MockCallForFundsRepo {
        fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
                overdue: Mutex::new(Vec::new()),
            }
        }

        fn with_overdue(overdue: Vec<CallForFunds>) -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
                overdue: Mutex::new(overdue),
            }
        }
    }

    #[async_trait]
    impl CallForFundsRepository for MockCallForFundsRepo {
        async fn create(&self, cff: &CallForFunds) -> Result<CallForFunds, String> {
            let mut store = self.store.lock().unwrap();
            store.insert(cff.id, cff.clone());
            Ok(cff.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<CallForFunds>, String> {
            Ok(self.store.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<CallForFunds>, String> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .values()
                .filter(|c| c.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<CallForFunds>, String> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .values()
                .filter(|c| c.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn update(&self, cff: &CallForFunds) -> Result<CallForFunds, String> {
            let mut store = self.store.lock().unwrap();
            store.insert(cff.id, cff.clone());
            Ok(cff.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.store.lock().unwrap().remove(&id).is_some())
        }

        async fn find_overdue(&self) -> Result<Vec<CallForFunds>, String> {
            Ok(self.overdue.lock().unwrap().clone())
        }
    }

    // ── Mock: OwnerContributionRepository ─────────────────────────────

    struct MockOwnerContributionRepo {
        store: Mutex<Vec<OwnerContribution>>,
    }

    impl MockOwnerContributionRepo {
        fn new() -> Self {
            Self {
                store: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl OwnerContributionRepository for MockOwnerContributionRepo {
        async fn create(
            &self,
            contribution: &OwnerContribution,
        ) -> Result<OwnerContribution, String> {
            self.store.lock().unwrap().push(contribution.clone());
            Ok(contribution.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<OwnerContribution>, String> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.id == id)
                .cloned())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<OwnerContribution>, String> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .iter()
                .filter(|c| c.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<OwnerContribution>, String> {
            Ok(self
                .store
                .lock()
                .unwrap()
                .iter()
                .filter(|c| c.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn update(
            &self,
            contribution: &OwnerContribution,
        ) -> Result<OwnerContribution, String> {
            Ok(contribution.clone())
        }
    }

    // ── Mock: UnitOwnerRepository ─────────────────────────────────────

    struct MockUnitOwnerRepo {
        /// Pourcentages de détention BRUTS (1.0 par propriétaire unique).
        active_by_building: Mutex<Vec<(Uuid, Uuid, rust_decimal::Decimal)>>,
        /// Quotes-parts de CHARGE résolues (somme = 1 sur immeuble conforme).
        ///
        /// Distinctes de la précédente À DESSEIN : c'est ce qui permet de
        /// prouver LAQUELLE des deux le cas d'usage consomme. Un mock qui
        /// renverrait la même chose des deux côtés laisserait passer le défaut
        /// qui a produit 40 000 € appelés pour 10 000 € dus.
        quota_shares: Mutex<Vec<(Uuid, Uuid, rust_decimal::Decimal)>>,
    }

    impl MockUnitOwnerRepo {
        fn new() -> Self {
            Self {
                active_by_building: Mutex::new(Vec::new()),
                quota_shares: Mutex::new(Vec::new()),
            }
        }

        fn with_owners(owners: Vec<(Uuid, Uuid, rust_decimal::Decimal)>) -> Self {
            Self {
                active_by_building: Mutex::new(owners.clone()),
                quota_shares: Mutex::new(owners),
            }
        }

        /// Les deux sources divergent : détentions brutes d'un côté,
        /// quotes-parts de l'autre.
        fn with_divergent(
            brut: Vec<(Uuid, Uuid, rust_decimal::Decimal)>,
            parts: Vec<(Uuid, Uuid, rust_decimal::Decimal)>,
        ) -> Self {
            Self {
                active_by_building: Mutex::new(brut),
                quota_shares: Mutex::new(parts),
            }
        }
    }

    #[async_trait]
    impl UnitOwnerRepository for MockUnitOwnerRepo {
        async fn create(&self, _uo: &UnitOwner) -> Result<UnitOwner, String> {
            unimplemented!()
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<UnitOwner>, String> {
            unimplemented!()
        }
        async fn find_current_owners_by_unit(
            &self,
            _unit_id: Uuid,
        ) -> Result<Vec<UnitOwner>, String> {
            unimplemented!()
        }
        async fn find_current_units_by_owner(
            &self,
            _owner_id: Uuid,
        ) -> Result<Vec<UnitOwner>, String> {
            unimplemented!()
        }
        async fn find_all_owners_by_unit(&self, _unit_id: Uuid) -> Result<Vec<UnitOwner>, String> {
            unimplemented!()
        }
        async fn find_all_units_by_owner(&self, _owner_id: Uuid) -> Result<Vec<UnitOwner>, String> {
            unimplemented!()
        }
        async fn update(&self, _uo: &UnitOwner) -> Result<UnitOwner, String> {
            unimplemented!()
        }
        async fn delete(&self, _id: Uuid) -> Result<(), String> {
            unimplemented!()
        }
        async fn has_active_owners(&self, _unit_id: Uuid) -> Result<bool, String> {
            unimplemented!()
        }
        async fn get_total_ownership_percentage(
            &self,
            _unit_id: Uuid,
        ) -> Result<rust_decimal::Decimal, String> {
            unimplemented!()
        }
        async fn find_active_by_unit_and_owner(
            &self,
            _unit_id: Uuid,
            _owner_id: Uuid,
        ) -> Result<Option<UnitOwner>, String> {
            unimplemented!()
        }
        async fn find_active_by_building(
            &self,
            _building_id: Uuid,
        ) -> Result<Vec<(Uuid, Uuid, rust_decimal::Decimal)>, String> {
            Ok(self.active_by_building.lock().unwrap().clone())
        }

        async fn find_active_quota_shares_by_building(
            &self,
            _building_id: Uuid,
        ) -> Result<Vec<(Uuid, Uuid, rust_decimal::Decimal)>, String> {
            Ok(self.quota_shares.lock().unwrap().clone())
        }

        async fn find_voting_holders_by_unit(
            &self,
            _unit_id: Uuid,
        ) -> Result<Vec<crate::domain::entities::LotHolder>, String> {
            Ok(vec![])
        }
    }

    // ── Dépôt d'immeubles ─────────────────────────────────────────────
    //
    // Il n'était pas câblé dans ces tests, parce que la résolution de l'ACP
    // n'existait pas. Elle est désormais obligatoire à la création : on ne
    // lance pas un appel de fonds sans savoir qui en est créancier
    // (ADR-0045).

    struct MockBuildingRepo {
        acp_id: Uuid,
    }

    impl MockBuildingRepo {
        fn rattache_a(acp_id: Uuid) -> Self {
            Self { acp_id }
        }

        fn immeuble(&self) -> crate::domain::entities::Building {
            crate::domain::entities::Building::new(
                self.acp_id,
                "Résidence du Parc".to_string(),
                "12 Rue de la Loi".to_string(),
                "Brussels".to_string(),
                "1000".to_string(),
                "Belgium".to_string(),
                10,
                1000,
                Some(2015),
            )
            .expect("immeuble valide")
        }
    }

    #[async_trait]
    impl BuildingRepository for MockBuildingRepo {
        async fn create(
            &self,
            b: &crate::domain::entities::Building,
        ) -> Result<crate::domain::entities::Building, String> {
            Ok(b.clone())
        }
        async fn find_by_id(
            &self,
            _id: Uuid,
        ) -> Result<Option<crate::domain::entities::Building>, String> {
            Ok(Some(self.immeuble()))
        }
        async fn find_all(&self) -> Result<Vec<crate::domain::entities::Building>, String> {
            Ok(vec![self.immeuble()])
        }
        async fn find_all_paginated(
            &self,
            _p: &crate::application::dto::PageRequest,
            _f: &crate::application::dto::BuildingFilters,
        ) -> Result<(Vec<crate::domain::entities::Building>, i64), String> {
            Ok((vec![self.immeuble()], 1))
        }
        async fn update(
            &self,
            b: &crate::domain::entities::Building,
        ) -> Result<crate::domain::entities::Building, String> {
            Ok(b.clone())
        }
        async fn delete(&self, _id: Uuid) -> Result<bool, String> {
            Ok(true)
        }
        async fn find_by_slug(
            &self,
            _slug: &str,
        ) -> Result<Option<crate::domain::entities::Building>, String> {
            Ok(Some(self.immeuble()))
        }
        async fn find_by_id_with_metrics(
            &self,
            _id: Uuid,
        ) -> Result<
            Option<(
                crate::domain::entities::Building,
                crate::domain::entities::BuildingMetrics,
            )>,
            String,
        > {
            Ok(None)
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────

    fn make_use_cases(
        cff_repo: Arc<dyn CallForFundsRepository>,
        contrib_repo: Arc<dyn OwnerContributionRepository>,
        uo_repo: Arc<dyn UnitOwnerRepository>,
    ) -> CallForFundsUseCases {
        make_use_cases_pour_lacp(cff_repo, contrib_repo, uo_repo, Uuid::new_v4())
    }

    /// Les mêmes use-cases, mais en nommant l'ACP de l'immeuble.
    fn make_use_cases_pour_lacp(
        cff_repo: Arc<dyn CallForFundsRepository>,
        contrib_repo: Arc<dyn OwnerContributionRepository>,
        uo_repo: Arc<dyn UnitOwnerRepository>,
        acp_id: Uuid,
    ) -> CallForFundsUseCases {
        let mut uc = CallForFundsUseCases::new(cff_repo, contrib_repo, uo_repo);
        uc.building_repository = Some(Arc::new(MockBuildingRepo::rattache_a(acp_id)));
        uc
    }

    fn sample_dates() -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
        let call_date = Utc::now();
        let due_date = call_date + Duration::days(30);
        (call_date, due_date)
    }

    // ── 1. Create ─────────────────────────────────────────────────────

    /// Art. 3.86 § 3 et ADR-0045 : l'ACP est créancière des fonds appelés.
    ///
    /// L'ACP se déduit de l'immeuble, jamais de l'appelant. Le lien
    /// immeuble → ACP est fixé par l'acte de base ; un cabinet ne peut donc
    /// pas appeler des fonds au nom d'une ACP qu'il désigne lui-même.
    #[tokio::test]
    async fn test_lappel_de_fonds_a_pour_creanciere_lacp_de_limmeuble() {
        let acp_creanciere = Uuid::new_v4();
        let cabinet_emetteur = Uuid::new_v4();

        let uc = make_use_cases_pour_lacp(
            Arc::new(MockCallForFundsRepo::new()),
            Arc::new(MockOwnerContributionRepo::new()),
            Arc::new(MockUnitOwnerRepo::new()),
            acp_creanciere,
        );
        let (call_date, due_date) = sample_dates();

        let appel = uc
            .create_call_for_funds(
                cabinet_emetteur,
                Uuid::new_v4(),
                "Provision T1 2026".to_string(),
                "Charges ordinaires".to_string(),
                dec!(10000),
                ContributionType::Regular,
                call_date,
                due_date,
                None,
                None,
                rust_decimal::Decimal::ZERO, // part fonds de réserve
            )
            .await
            .expect("création valide");

        assert_eq!(
            appel.acp_id, acp_creanciere,
            "les fonds sont appelés au nom de l'ACP de l'immeuble"
        );
        assert_eq!(
            appel.organization_id, cabinet_emetteur,
            "le syndic reste tracé comme émetteur, sans devenir créancier"
        );
    }

    /// Sans dépôt d'immeubles, on ne sait pas au nom de qui l'argent est
    /// appelé. On refuse, plutôt que de retomber sur l'identifiant du syndic
    /// — c'est exactement la confusion que l'ADR-0045 supprime.
    #[tokio::test]
    async fn test_pas_dappel_de_fonds_sans_creanciere_resoluble() {
        let uc = CallForFundsUseCases::new(
            Arc::new(MockCallForFundsRepo::new()),
            Arc::new(MockOwnerContributionRepo::new()),
            Arc::new(MockUnitOwnerRepo::new()),
        );
        let (call_date, due_date) = sample_dates();

        let resultat = uc
            .create_call_for_funds(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Provision".to_string(),
                "Charges".to_string(),
                dec!(10000),
                ContributionType::Regular,
                call_date,
                due_date,
                None,
                None,
                rust_decimal::Decimal::ZERO, // part fonds de réserve
            )
            .await;

        let erreur = resultat.expect_err("doit refuser");
        assert!(
            erreur.contains("créancière"),
            "le refus doit nommer ce qui manque, pas échouer obscurément : {erreur}"
        );
    }

    #[tokio::test]
    async fn test_create_call_for_funds_success() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::new());
        let uc = make_use_cases(cff_repo.clone(), contrib_repo, uo_repo);

        let (call_date, due_date) = sample_dates();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = uc
            .create_call_for_funds(
                org_id,
                building_id,
                "Appel Q1".to_string(),
                "Charges courantes".to_string(),
                rust_decimal_macros::dec!(10_000),
                ContributionType::Regular,
                call_date,
                due_date,
                Some("7000".to_string()),
                Some(Uuid::new_v4()),
                rust_decimal::Decimal::ZERO, // part fonds de réserve
            )
            .await;

        assert!(result.is_ok());
        let cff = result.unwrap();
        assert_eq!(cff.total_amount, rust_decimal_macros::dec!(10_000));
        assert_eq!(cff.status, CallForFundsStatus::Draft);
        assert_eq!(cff.organization_id, org_id);
        assert_eq!(cff.building_id, building_id);
        // Verify it was persisted in the mock store
        assert!(cff_repo.store.lock().unwrap().contains_key(&cff.id));
    }

    // ── 2. Send (generates contributions) ─────────────────────────────

    #[tokio::test]
    async fn test_send_call_for_funds_generates_contributions() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());

        let unit1 = Uuid::new_v4();
        let unit2 = Uuid::new_v4();
        let owner1 = Uuid::new_v4();
        let owner2 = Uuid::new_v4();
        let uo_repo = Arc::new(MockUnitOwnerRepo::with_owners(vec![
            (unit1, owner1, rust_decimal_macros::dec!(0.60)),
            (unit2, owner2, rust_decimal_macros::dec!(0.40)),
        ]));

        let uc = make_use_cases(cff_repo.clone(), contrib_repo.clone(), uo_repo);

        let (call_date, due_date) = sample_dates();

        let cff = uc
            .create_call_for_funds(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Appel Q2".to_string(),
                "Charges extraordinaires".to_string(),
                rust_decimal_macros::dec!(5_000),
                ContributionType::Extraordinary,
                call_date,
                due_date,
                None,
                None,
                rust_decimal::Decimal::ZERO, // part fonds de réserve
            )
            .await
            .unwrap();

        // Send — should generate individual contributions
        let result = uc.send_call_for_funds(cff.id).await;
        assert!(result.is_ok());

        let sent = result.unwrap();
        assert_eq!(sent.status, CallForFundsStatus::Sent);
        assert!(sent.sent_date.is_some());

        // Verify two contributions were created with correct amounts
        let contributions = contrib_repo.store.lock().unwrap();
        assert_eq!(contributions.len(), 2);

        let mut amounts: Vec<rust_decimal::Decimal> =
            contributions.iter().map(|c| c.amount).collect();
        amounts.sort();
        // 40% of 5000 = 2000, 60% of 5000 = 3000
        assert_eq!(amounts[0], rust_decimal_macros::dec!(2_000));
        assert_eq!(amounts[1], rust_decimal_macros::dec!(3_000));
    }

    /// Non-régression — l'appel de fonds lit les QUOTES-PARTS DE CHARGE, pas
    /// les pourcentages de détention.
    ///
    /// Le défaut, mesuré en production le 2026-09-02 sur un immeuble conforme
    /// à 4 lots (200/200/300/300 millièmes, un propriétaire unique par lot) :
    /// un appel de 10 000 € générait QUATRE quotes-parts de 10 000 €, soit
    /// 40 000 € appelés, chacune étiquetée « Quote-part: 100 % ». Les
    /// tantièmes de l'acte de base étaient purement ignorés, et la route
    /// répondait 200.
    ///
    /// Cause : `find_active_by_building` renvoie `ownership_percentage` brut,
    /// qui vaut 1.0 pour tout propriétaire unique de son lot. Multiplié par le
    /// montant total, il appelle l'intégralité à chacun. La formule légale
    /// (Art. 3.84) — `(quota / total_tantiemes) × ownership_percentage` —
    /// existait dans `ChargeDistribution::resolve_owner_quota`, testée, et
    /// n'avait AUCUN appelant en production.
    ///
    /// Le test précédent ne pouvait pas le voir : ses fixtures posent
    /// directement 0.60/0.40, c'est-à-dire des quotes-parts déjà résolues. Il
    /// encodait donc le bon contrat pendant que le dépôt le violait. Ici les
    /// deux sources DIVERGENT, ce qui rend la confusion détectable.
    #[tokio::test]
    async fn test_appel_de_fonds_utilise_les_quotes_parts_pas_les_detentions() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());

        let lots: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
        let proprios: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();

        // Immeuble conforme : 200/200/300/300 millièmes sur 1000.
        // Détention brute : 100 % de son lot pour chacun → somme = 4.0.
        let brut: Vec<_> = lots
            .iter()
            .zip(&proprios)
            .map(|(u, o)| (*u, *o, rust_decimal_macros::dec!(1.0)))
            .collect();
        // Quotes-parts de charge : 0,2 / 0,2 / 0,3 / 0,3 → somme = 1.0.
        let parts = vec![
            (lots[0], proprios[0], rust_decimal_macros::dec!(0.2)),
            (lots[1], proprios[1], rust_decimal_macros::dec!(0.2)),
            (lots[2], proprios[2], rust_decimal_macros::dec!(0.3)),
            (lots[3], proprios[3], rust_decimal_macros::dec!(0.3)),
        ];

        let uo_repo = Arc::new(MockUnitOwnerRepo::with_divergent(brut, parts));
        let uc = make_use_cases(cff_repo.clone(), contrib_repo.clone(), uo_repo);
        let (call_date, due_date) = sample_dates();

        let cff = uc
            .create_call_for_funds(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Charges Q3".to_string(),
                "Non-régression répartition".to_string(),
                rust_decimal_macros::dec!(10_000),
                ContributionType::Regular,
                call_date,
                due_date,
                None,
                None,
                rust_decimal::Decimal::ZERO, // part fonds de réserve
            )
            .await
            .unwrap();

        uc.send_call_for_funds(cff.id).await.expect("envoi accepté");

        let contributions = contrib_repo.store.lock().unwrap();
        assert_eq!(contributions.len(), 4, "une quote-part par lot");

        let mut montants: Vec<rust_decimal::Decimal> =
            contributions.iter().map(|c| c.amount).collect();
        montants.sort();
        assert_eq!(
            montants,
            vec![
                rust_decimal_macros::dec!(2_000),
                rust_decimal_macros::dec!(2_000),
                rust_decimal_macros::dec!(3_000),
                rust_decimal_macros::dec!(3_000),
            ],
            "chaque copropriétaire doit être appelé au prorata de ses tantièmes"
        );

        // L'invariant qui compte pour le syndic : on n'appelle jamais plus que
        // ce qui est dû. Avant correction, cette somme valait 40 000.
        let total: rust_decimal::Decimal = montants.iter().sum();
        assert_eq!(
            total,
            rust_decimal_macros::dec!(10_000),
            "la somme appelée doit égaler le montant de l'appel de fonds"
        );
    }

    // ── 3. Cancel ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_cancel_call_for_funds() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::new());
        let uc = make_use_cases(cff_repo.clone(), contrib_repo, uo_repo);

        let (call_date, due_date) = sample_dates();

        let cff = uc
            .create_call_for_funds(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Appel annulable".to_string(),
                "Description".to_string(),
                rust_decimal_macros::dec!(1_000),
                ContributionType::Regular,
                call_date,
                due_date,
                None,
                None,
                rust_decimal::Decimal::ZERO, // part fonds de réserve
            )
            .await
            .unwrap();

        let result = uc.cancel_call_for_funds(cff.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, CallForFundsStatus::Cancelled);
    }

    // ── 4. Delete (draft only, rejects sent) ──────────────────────────

    #[tokio::test]
    async fn test_delete_call_for_funds_draft_succeeds() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::new());
        let uc = make_use_cases(cff_repo.clone(), contrib_repo, uo_repo);

        let (call_date, due_date) = sample_dates();

        let cff = uc
            .create_call_for_funds(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Supprimable".to_string(),
                "Description".to_string(),
                rust_decimal_macros::dec!(500),
                ContributionType::Advance,
                call_date,
                due_date,
                None,
                None,
                rust_decimal::Decimal::ZERO, // part fonds de réserve
            )
            .await
            .unwrap();

        let result = uc.delete_call_for_funds(cff.id).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(!cff_repo.store.lock().unwrap().contains_key(&cff.id));
    }

    #[tokio::test]
    async fn test_delete_call_for_funds_rejects_non_draft() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::with_owners(vec![(
            Uuid::new_v4(),
            Uuid::new_v4(),
            rust_decimal_macros::dec!(1),
        )]));
        let uc = make_use_cases(cff_repo.clone(), contrib_repo, uo_repo);

        let (call_date, due_date) = sample_dates();

        let cff = uc
            .create_call_for_funds(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "Sent call".to_string(),
                "Description".to_string(),
                rust_decimal_macros::dec!(500),
                ContributionType::Regular,
                call_date,
                due_date,
                None,
                None,
                rust_decimal::Decimal::ZERO, // part fonds de réserve
            )
            .await
            .unwrap();

        // Send so it is no longer Draft
        uc.send_call_for_funds(cff.id).await.unwrap();

        let result = uc.delete_call_for_funds(cff.id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot delete a call for funds that has been sent"));
    }

    // ── 5. Find overdue ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_overdue_calls() {
        let call_date = Utc::now() - Duration::days(60);
        let due_date = Utc::now() - Duration::days(30);
        let overdue_cff = CallForFunds::new(
            Uuid::new_v4(), // acp_id
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Overdue call".to_string(),
            "Past due".to_string(),
            rust_decimal_macros::dec!(2_000),
            ContributionType::Regular,
            call_date,
            due_date,
            None,
            rust_decimal::Decimal::ZERO, // part fonds de réserve
        )
        .unwrap();

        let cff_repo = Arc::new(MockCallForFundsRepo::with_overdue(
            vec![overdue_cff.clone()],
        ));
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::new());
        let uc = make_use_cases(cff_repo, contrib_repo, uo_repo);

        let result = uc.get_overdue_calls().await;
        assert!(result.is_ok());
        let overdue = result.unwrap();
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].title, "Overdue call");
    }

    // ── 6. List by building ───────────────────────────────────────────

    #[tokio::test]
    async fn test_list_by_building() {
        let cff_repo = Arc::new(MockCallForFundsRepo::new());
        let contrib_repo = Arc::new(MockOwnerContributionRepo::new());
        let uo_repo = Arc::new(MockUnitOwnerRepo::new());
        let uc = make_use_cases(cff_repo.clone(), contrib_repo, uo_repo);

        let building_id = Uuid::new_v4();
        let other_building = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let (call_date, due_date) = sample_dates();

        // Two calls for our building
        uc.create_call_for_funds(
            org_id,
            building_id,
            "Appel 1".to_string(),
            "Desc 1".to_string(),
            rust_decimal_macros::dec!(1_000),
            ContributionType::Regular,
            call_date,
            due_date,
            None,
            None,
            rust_decimal::Decimal::ZERO, // part fonds de réserve
        )
        .await
        .unwrap();

        uc.create_call_for_funds(
            org_id,
            building_id,
            "Appel 2".to_string(),
            "Desc 2".to_string(),
            rust_decimal_macros::dec!(2_000),
            ContributionType::Extraordinary,
            call_date,
            due_date,
            None,
            None,
            rust_decimal::Decimal::ZERO, // part fonds de réserve
        )
        .await
        .unwrap();

        // One call for another building (noise)
        uc.create_call_for_funds(
            org_id,
            other_building,
            "Autre appel".to_string(),
            "Autre desc".to_string(),
            rust_decimal_macros::dec!(500),
            ContributionType::Regular,
            call_date,
            due_date,
            None,
            None,
            rust_decimal::Decimal::ZERO, // part fonds de réserve
        )
        .await
        .unwrap();

        let result = uc.list_by_building(building_id).await;
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().all(|c| c.building_id == building_id));
    }
}
