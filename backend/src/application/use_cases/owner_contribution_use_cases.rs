use crate::application::ports::OwnerContributionRepository;
use crate::application::services::expense_accounting_service::ExpenseAccountingService;
use crate::domain::entities::{ContributionPaymentMethod, ContributionType, OwnerContribution};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct OwnerContributionUseCases {
    repository: Arc<dyn OwnerContributionRepository>,
    /// Résolution de l'ACP créancière, depuis le lot.
    ///
    /// Optionnel comme `accounting_service`, pour ne pas casser les
    /// constructeurs des tests unitaires — mais son absence fait échouer la
    /// création, elle ne la laisse pas passer avec un identifiant de repli.
    unit_repository: Option<Arc<dyn crate::application::ports::UnitRepository>>,
    /// Optionnel pour préserver les constructeurs des tests unitaires, qui ne
    /// montent qu'un dépôt de contributions. Le câblage de `main.rs` le
    /// fournit toujours.
    accounting_service: Option<Arc<ExpenseAccountingService>>,
}

impl OwnerContributionUseCases {
    pub fn new(repository: Arc<dyn OwnerContributionRepository>) -> Self {
        Self {
            repository,
            unit_repository: None,
            accounting_service: None,
        }
    }

    /// Câble la résolution de l'ACP créancière depuis le lot.
    pub fn with_acp_resolution(
        mut self,
        unit_repository: Arc<dyn crate::application::ports::UnitRepository>,
    ) -> Self {
        self.unit_repository = Some(unit_repository);
        self
    }

    pub fn with_accounting(mut self, accounting_service: Arc<ExpenseAccountingService>) -> Self {
        self.accounting_service = Some(accounting_service);
        self
    }

    /// Enregistre l'écriture d'encaissement d'une quote-part soldée.
    ///
    /// Volontairement INFAILLIBLE, comme du côté des dépenses : le paiement a
    /// été enregistré et persisté ; refuser l'opération à cause d'une écriture
    /// comptable ferait perdre l'encaissement lui-même.
    ///
    /// Ce silence a un précédent coûteux — c'est lui qui a laissé la
    /// génération automatique côté dépense échouer pendant des mois sur des
    /// codes de compte inexistants (constat F7 du 2026-09-01). Le garde-fou
    /// n'est donc pas ici mais en amont :
    /// `test_les_comptes_utilises_existent_dans_le_plan` vérifie que les
    /// comptes référencés existent bel et bien dans le plan provisionné.
    pub(crate) async fn enregistrer_encaissement(&self, contribution: &OwnerContribution) {
        let Some(ref accounting) = self.accounting_service else {
            return;
        };
        // `building_id` à None : la quote-part ne porte qu'un `unit_id`
        // optionnel, et remonter au bâtiment demanderait un dépôt de lots ici.
        // Conséquence assumée et limitée : l'écriture est bien au grand livre,
        // mais n'apparaît pas dans les rapports financiers PAR IMMEUBLE. La
        // voie `/payments`, elle, connaît son immeuble et le renseigne.
        if let Err(e) = accounting
            .generate_contribution_receipt_entry(contribution, None, None, None)
            .await
        {
            log::warn!(
                "Écriture d'encaissement non générée pour la quote-part {} : {}",
                contribution.id,
                e
            );
        }
    }

    /// L'ACP à laquelle la quote-part est due, lue sur le lot.
    ///
    /// Le lot porte déjà son ACP (Story H15) : le rattachement vient de l'acte
    /// de base, il ne dépend ni de l'appelant ni du mandat en cours. On échoue
    /// si on ne peut pas le lire, plutôt que de retomber sur l'identifiant du
    /// syndic — une quote-part due à personne n'est pas une quote-part
    /// (ADR-0045).
    async fn resoudre_lacp_creanciere(&self, unit_id: Option<Uuid>) -> Result<Uuid, String> {
        let unit_id = unit_id.ok_or_else(|| {
            "Impossible de déterminer l'ACP créancière : la quote-part doit porter un lot"
                .to_string()
        })?;
        let Some(unit_repo) = &self.unit_repository else {
            return Err(
                "Impossible de déterminer l'ACP créancière : dépôt de lots non câblé".to_string(),
            );
        };
        let unit = unit_repo
            .find_by_id(unit_id)
            .await?
            .ok_or_else(|| "Lot introuvable".to_string())?;
        Ok(unit.acp_id)
    }

    /// Create a new owner contribution (appel de fonds)
    #[allow(clippy::too_many_arguments)]
    pub async fn create_contribution(
        &self,
        organization_id: Uuid,
        owner_id: Uuid,
        unit_id: Option<Uuid>,
        description: String,
        amount: Decimal,
        contribution_type: ContributionType,
        contribution_date: DateTime<Utc>,
        account_code: Option<String>,
    ) -> Result<OwnerContribution, String> {
        let acp_id = self.resoudre_lacp_creanciere(unit_id).await?;

        // Create domain entity (validates business rules)
        let contribution = OwnerContribution::new(
            acp_id,
            organization_id,
            owner_id,
            unit_id,
            description,
            amount,
            contribution_type,
            contribution_date,
            account_code,
        )?;

        // Persist
        self.repository.create(&contribution).await
    }

    /// Record payment for a contribution
    pub async fn record_payment(
        &self,
        contribution_id: Uuid,
        payment_date: DateTime<Utc>,
        payment_method: ContributionPaymentMethod,
        payment_reference: Option<String>,
    ) -> Result<OwnerContribution, String> {
        // Find contribution
        let mut contribution = self
            .repository
            .find_by_id(contribution_id)
            .await?
            .ok_or_else(|| format!("Contribution not found: {}", contribution_id))?;

        // Prevent double payment
        if contribution.is_paid() {
            return Err("Contribution is already paid".to_string());
        }

        // Mark as paid (domain logic)
        contribution.mark_as_paid(payment_date, payment_method, payment_reference);

        // Update
        let updated = self.repository.update(&contribution).await?;

        // D 550 (banque) / C 400 (copropriétaires) — constat F7.
        self.enregistrer_encaissement(&updated).await;

        Ok(updated)
    }

    /// Get contribution by ID
    pub async fn get_contribution(
        &self,
        contribution_id: Uuid,
    ) -> Result<Option<OwnerContribution>, String> {
        self.repository.find_by_id(contribution_id).await
    }

    /// Get all contributions for an organization
    pub async fn get_contributions_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OwnerContribution>, String> {
        self.repository.find_by_organization(organization_id).await
    }

    /// Get all contributions for an owner
    pub async fn get_contributions_by_owner(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<OwnerContribution>, String> {
        self.repository.find_by_owner(owner_id).await
    }

    /// Get outstanding (unpaid) contributions for an owner
    pub async fn get_outstanding_contributions(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<OwnerContribution>, String> {
        let contributions = self.repository.find_by_owner(owner_id).await?;

        // Filter unpaid
        Ok(contributions.into_iter().filter(|c| !c.is_paid()).collect())
    }

    /// Get overdue contributions for an owner
    pub async fn get_overdue_contributions(
        &self,
        owner_id: Uuid,
    ) -> Result<Vec<OwnerContribution>, String> {
        let contributions = self.repository.find_by_owner(owner_id).await?;

        // Filter overdue
        Ok(contributions
            .into_iter()
            .filter(|c| c.is_overdue())
            .collect())
    }

    /// Get total outstanding amount for an owner
    pub async fn get_outstanding_amount(&self, owner_id: Uuid) -> Result<Decimal, String> {
        let outstanding = self.get_outstanding_contributions(owner_id).await?;
        Ok(outstanding.iter().map(|c| c.amount).sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockOwnerContributionRepository {
        items: Mutex<HashMap<Uuid, OwnerContribution>>,
    }

    impl MockOwnerContributionRepository {
        fn new() -> Self {
            Self {
                items: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl OwnerContributionRepository for MockOwnerContributionRepository {
        async fn create(
            &self,
            contribution: &OwnerContribution,
        ) -> Result<OwnerContribution, String> {
            let mut items = self.items.lock().unwrap();
            items.insert(contribution.id, contribution.clone());
            Ok(contribution.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<OwnerContribution>, String> {
            let items = self.items.lock().unwrap();
            Ok(items.get(&id).cloned())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<OwnerContribution>, String> {
            let items = self.items.lock().unwrap();
            Ok(items
                .values()
                .filter(|c| c.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<OwnerContribution>, String> {
            let items = self.items.lock().unwrap();
            Ok(items
                .values()
                .filter(|c| c.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn update(
            &self,
            contribution: &OwnerContribution,
        ) -> Result<OwnerContribution, String> {
            let mut items = self.items.lock().unwrap();
            items.insert(contribution.id, contribution.clone());
            Ok(contribution.clone())
        }
    }

    /// Dépôt de lots minimal : un lot rattaché à une ACP nommée.
    ///
    /// Il n'existait pas ici parce que la quote-part ne cherchait pas son
    /// créancier. Elle le cherche désormais (ADR-0045).
    struct MockUnitRepository {
        acp_id: Uuid,
    }

    impl MockUnitRepository {
        fn lot_de(acp_id: Uuid) -> Self {
            Self { acp_id }
        }

        fn lot(&self) -> crate::domain::entities::Unit {
            crate::domain::entities::Unit::new(
                self.acp_id,
                Uuid::new_v4(),
                "A101".to_string(),
                crate::domain::entities::UnitType::Apartment,
                Some(1),
                85.0,
                rust_decimal_macros::dec!(100),
            )
            .expect("lot valide")
        }
    }

    #[async_trait::async_trait]
    impl crate::application::ports::UnitRepository for MockUnitRepository {
        async fn create(
            &self,
            u: &crate::domain::entities::Unit,
        ) -> Result<crate::domain::entities::Unit, String> {
            Ok(u.clone())
        }
        async fn find_by_id(
            &self,
            _id: Uuid,
        ) -> Result<Option<crate::domain::entities::Unit>, String> {
            Ok(Some(self.lot()))
        }
        async fn find_by_building(
            &self,
            _b: Uuid,
        ) -> Result<Vec<crate::domain::entities::Unit>, String> {
            Ok(vec![self.lot()])
        }
        async fn find_by_owner(
            &self,
            _o: Uuid,
        ) -> Result<Vec<crate::domain::entities::Unit>, String> {
            Ok(vec![self.lot()])
        }
        async fn find_all_paginated(
            &self,
            _p: &crate::application::dto::PageRequest,
            _f: &crate::application::dto::UnitFilters,
        ) -> Result<(Vec<crate::domain::entities::Unit>, i64), String> {
            Ok((vec![self.lot()], 1))
        }
        async fn update(
            &self,
            u: &crate::domain::entities::Unit,
        ) -> Result<crate::domain::entities::Unit, String> {
            Ok(u.clone())
        }
        async fn delete(&self, _id: Uuid) -> Result<bool, String> {
            Ok(true)
        }
    }

    fn make_use_cases(repo: MockOwnerContributionRepository) -> OwnerContributionUseCases {
        make_use_cases_pour_lacp(repo, Uuid::new_v4())
    }

    /// Les mêmes use-cases, en nommant l'ACP à laquelle le lot est rattaché.
    fn make_use_cases_pour_lacp(
        repo: MockOwnerContributionRepository,
        acp_id: Uuid,
    ) -> OwnerContributionUseCases {
        OwnerContributionUseCases::new(Arc::new(repo))
            .with_acp_resolution(Arc::new(MockUnitRepository::lot_de(acp_id)))
    }

    /// Art. 3.86 § 3 et ADR-0045 : la quote-part est due à l'ACP.
    ///
    /// L'ACP est lue sur le lot, jamais sur l'appelant. Un cabinet ne peut
    /// donc pas émettre une quote-part au profit d'une ACP qu'il désigne.
    #[tokio::test]
    async fn test_la_quote_part_est_due_a_lacp_du_lot_pas_au_syndic() {
        let acp_creanciere = Uuid::new_v4();
        let cabinet_emetteur = Uuid::new_v4();
        let use_cases =
            make_use_cases_pour_lacp(MockOwnerContributionRepository::new(), acp_creanciere);

        let quote_part = use_cases
            .create_contribution(
                cabinet_emetteur,
                Uuid::new_v4(),
                Some(Uuid::new_v4()),
                "Appel de fonds Q1 2026".to_string(),
                rust_decimal_macros::dec!(750),
                ContributionType::Regular,
                Utc::now(),
                Some("7000".to_string()),
            )
            .await
            .expect("création valide");

        assert_eq!(
            quote_part.acp_id, acp_creanciere,
            "la quote-part est due à l'ACP du lot"
        );
        assert_eq!(
            quote_part.organization_id, cabinet_emetteur,
            "le syndic reste tracé comme émetteur, sans devenir créancier"
        );
    }

    /// Sans lot, on ne sait pas à quelle ACP la somme est due. On refuse.
    #[tokio::test]
    async fn test_pas_de_quote_part_sans_lot_donc_sans_creanciere() {
        let use_cases = make_use_cases(MockOwnerContributionRepository::new());

        let erreur = use_cases
            .create_contribution(
                Uuid::new_v4(),
                Uuid::new_v4(),
                None, // pas de lot
                "Appel hors lot".to_string(),
                rust_decimal_macros::dec!(750),
                ContributionType::Regular,
                Utc::now(),
                None,
            )
            .await
            .expect_err("doit refuser");

        assert!(
            erreur.contains("créancière"),
            "le refus doit nommer ce qui manque : {erreur}"
        );
    }

    #[tokio::test]
    async fn test_create_contribution_success() {
        let repo = MockOwnerContributionRepository::new();
        let use_cases = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let result = use_cases
            .create_contribution(
                org_id,
                owner_id,
                Some(unit_id),
                "Appel de fonds Q1 2026".to_string(),
                rust_decimal_macros::dec!(750),
                ContributionType::Regular,
                Utc::now(),
                Some("7000".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let contrib = result.unwrap();
        assert_eq!(contrib.organization_id, org_id);
        assert_eq!(contrib.owner_id, owner_id);
        assert_eq!(contrib.unit_id, Some(unit_id));
        assert_eq!(contrib.amount, rust_decimal_macros::dec!(750));
        assert_eq!(contrib.contribution_type, ContributionType::Regular);
        assert!(!contrib.is_paid());
    }

    #[tokio::test]
    async fn test_record_payment_success() {
        let repo = MockOwnerContributionRepository::new();
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Pre-populate with a pending contribution
        let contrib = OwnerContribution::new(
            Uuid::new_v4(), // acp_id
            org_id,
            owner_id,
            None,
            "Charges Q2".to_string(),
            rust_decimal_macros::dec!(500),
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();
        let contrib_id = contrib.id;
        repo.items.lock().unwrap().insert(contrib.id, contrib);

        let use_cases = make_use_cases(repo);
        let result = use_cases
            .record_payment(
                contrib_id,
                Utc::now(),
                ContributionPaymentMethod::BankTransfer,
                Some("VIR-2026-001".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let paid = result.unwrap();
        assert!(paid.is_paid());
        assert!(paid.payment_date.is_some());
        assert_eq!(
            paid.payment_method,
            Some(ContributionPaymentMethod::BankTransfer)
        );
        assert_eq!(paid.payment_reference, Some("VIR-2026-001".to_string()));
    }

    #[tokio::test]
    async fn test_record_payment_double_payment_rejected() {
        let repo = MockOwnerContributionRepository::new();
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Pre-populate with an already-paid contribution
        let mut contrib = OwnerContribution::new(
            Uuid::new_v4(), // acp_id
            org_id,
            owner_id,
            None,
            "Charges Q3".to_string(),
            rust_decimal_macros::dec!(300),
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();
        contrib.mark_as_paid(Utc::now(), ContributionPaymentMethod::Cash, None);
        let contrib_id = contrib.id;
        repo.items.lock().unwrap().insert(contrib.id, contrib);

        let use_cases = make_use_cases(repo);
        let result = use_cases
            .record_payment(
                contrib_id,
                Utc::now(),
                ContributionPaymentMethod::BankTransfer,
                None,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Contribution is already paid");
    }

    #[tokio::test]
    async fn test_get_outstanding_contributions() {
        let repo = MockOwnerContributionRepository::new();
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Create one paid and two unpaid contributions
        let mut paid_contrib = OwnerContribution::new(
            Uuid::new_v4(), // acp_id
            org_id,
            owner_id,
            None,
            "Charges Q1 - paid".to_string(),
            rust_decimal_macros::dec!(200),
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();
        paid_contrib.mark_as_paid(Utc::now(), ContributionPaymentMethod::Domiciliation, None);

        let unpaid1 = OwnerContribution::new(
            Uuid::new_v4(), // acp_id
            org_id,
            owner_id,
            None,
            "Charges Q2 - unpaid".to_string(),
            rust_decimal_macros::dec!(300),
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();

        let unpaid2 = OwnerContribution::new(
            Uuid::new_v4(), // acp_id
            org_id,
            owner_id,
            None,
            "Travaux extraordinaires".to_string(),
            rust_decimal_macros::dec!(1500),
            ContributionType::Extraordinary,
            Utc::now(),
            None,
        )
        .unwrap();

        {
            let mut items = repo.items.lock().unwrap();
            items.insert(paid_contrib.id, paid_contrib);
            items.insert(unpaid1.id, unpaid1);
            items.insert(unpaid2.id, unpaid2);
        }

        let use_cases = make_use_cases(repo);
        let result = use_cases.get_outstanding_contributions(owner_id).await;

        assert!(result.is_ok());
        let outstanding = result.unwrap();
        assert_eq!(outstanding.len(), 2);
        assert!(outstanding.iter().all(|c| !c.is_paid()));
    }

    #[tokio::test]
    async fn test_get_outstanding_amount() {
        let repo = MockOwnerContributionRepository::new();
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        // Create one paid (should not count) and two unpaid
        let mut paid = OwnerContribution::new(
            Uuid::new_v4(), // acp_id
            org_id,
            owner_id,
            None,
            "Paid contribution".to_string(),
            rust_decimal_macros::dec!(100),
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();
        paid.mark_as_paid(Utc::now(), ContributionPaymentMethod::Check, None);

        let unpaid1 = OwnerContribution::new(
            Uuid::new_v4(), // acp_id
            org_id,
            owner_id,
            None,
            "Unpaid 1".to_string(),
            rust_decimal_macros::dec!(250),
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();

        let unpaid2 = OwnerContribution::new(
            Uuid::new_v4(), // acp_id
            org_id,
            owner_id,
            None,
            "Unpaid 2".to_string(),
            rust_decimal_macros::dec!(400),
            ContributionType::Extraordinary,
            Utc::now(),
            None,
        )
        .unwrap();

        {
            let mut items = repo.items.lock().unwrap();
            items.insert(paid.id, paid);
            items.insert(unpaid1.id, unpaid1);
            items.insert(unpaid2.id, unpaid2);
        }

        let use_cases = make_use_cases(repo);
        let result = use_cases.get_outstanding_amount(owner_id).await;

        assert!(result.is_ok());
        let amount = result.unwrap();
        assert_eq!(amount, rust_decimal_macros::dec!(650));
    }
}
