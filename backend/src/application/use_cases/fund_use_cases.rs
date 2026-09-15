//! Use cases for the Fund feature (issue #635 — fonds affectés &
//! thésaurisation, ADR-0054).
//!
//! Trois opérations couvrent la checklist d'acceptation :
//!
//! 1. [`FundUseCases::create_working_capital_or_reserve`] /
//!    [`FundUseCases::create_earmarked_fund`] — créer un fonds. Un fonds
//!    affecté exige la majorité obtenue en AG (`MajorityType`), vérifiée par
//!    `Fund::new_earmarked` (Art. 3.88, 2/3).
//! 2. [`FundUseCases::contribute`] — alimenter un fonds (épargne).
//! 3. [`FundUseCases::reassign`] — réaffecter un fonds affecté vers une
//!    autre fin, sur décision d'AG adoptée, avec audit trail persistant
//!    (`fund_reassignments`).
//!
//! Aussi : [`FundUseCases::record_expense`] (garde-fou dépense / objet,
//! avec échappatoire d'AG) et des lectures (`get`, `list_for_acp`,
//! `list_reassignments`).

use crate::application::error::AppError;
use crate::application::ports::FundRepository;
use crate::domain::copropriete::resolution::{MajorityType, ResolutionStatus};
use crate::domain::entities::{Fund, FundKind, FundReassignment};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct FundUseCases {
    repo: Arc<dyn FundRepository>,
}

impl FundUseCases {
    pub fn new(repo: Arc<dyn FundRepository>) -> Self {
        Self { repo }
    }

    /// Crée un fonds de roulement ou de réserve (pas de vote qualifié
    /// requis à ce niveau — la réserve légale reste gouvernée par
    /// `Acp::assert_reserve_fund_compliant`, ADR-0012).
    pub async fn create_working_capital_or_reserve(
        &self,
        acp_id: Uuid,
        kind: FundKind,
        name: String,
    ) -> Result<Fund, AppError> {
        let fund = Fund::new(acp_id, kind, name, None, None)?;
        self.repo.create(&fund).await
    }

    /// Crée un fonds affecté (thésaurisation), en exigeant la majorité des
    /// gros travaux (2/3, Art. 3.88) — appliquée par `Fund::new_earmarked`.
    pub async fn create_earmarked_fund(
        &self,
        acp_id: Uuid,
        name: String,
        purpose: String,
        target_amount: Decimal,
        majority_used: MajorityType,
    ) -> Result<Fund, AppError> {
        let fund = Fund::new_earmarked(acp_id, name, purpose, target_amount, majority_used)?;
        self.repo.create(&fund).await
    }

    /// Alimente un fonds existant.
    pub async fn contribute(&self, fund_id: Uuid, amount: Decimal) -> Result<Fund, AppError> {
        let mut fund = self
            .repo
            .find_by_id(fund_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("fund {fund_id}")))?;
        fund.contribute(amount)?;
        self.repo.update(&fund).await
    }

    /// Vérifie qu'une dépense imputée à `fund_id` correspond à son objet —
    /// sauf si une décision d'AG adoptée l'autorise explicitement
    /// (`ag_override_status`).
    pub async fn record_expense(
        &self,
        fund_id: Uuid,
        expense_work_ref: &str,
        ag_override_status: Option<ResolutionStatus>,
    ) -> Result<(), AppError> {
        let fund = self
            .repo
            .find_by_id(fund_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("fund {fund_id}")))?;
        fund.assert_expense_matches_purpose(expense_work_ref, ag_override_status)?;
        Ok(())
    }

    /// Réaffecte tout ou partie du solde d'un fonds affecté vers une autre
    /// fin, sur décision d'AG adoptée. Persiste le fonds mis à jour ET
    /// l'audit trail (`fund_reassignments`).
    pub async fn reassign(
        &self,
        fund_id: Uuid,
        new_purpose: String,
        amount: Option<Decimal>,
        resolution_id: Uuid,
        resolution_status: ResolutionStatus,
    ) -> Result<FundReassignment, AppError> {
        let mut fund = self
            .repo
            .find_by_id(fund_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("fund {fund_id}")))?;
        let record = fund.reassign(new_purpose, amount, resolution_id, resolution_status)?;
        self.repo.update(&fund).await?;
        self.repo.record_reassignment(&record).await?;
        Ok(record)
    }

    pub async fn get(&self, fund_id: Uuid) -> Result<Fund, AppError> {
        self.repo
            .find_by_id(fund_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("fund {fund_id}")))
    }

    pub async fn list_for_acp(&self, acp_id: Uuid) -> Result<Vec<Fund>, AppError> {
        self.repo.find_by_acp_id(acp_id).await
    }

    pub async fn list_reassignments(
        &self,
        fund_id: Uuid,
    ) -> Result<Vec<FundReassignment>, AppError> {
        self.repo.list_reassignments(fund_id).await
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories (CRITICAL.md #3)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rust_decimal_macros::dec;
    use std::sync::Mutex;

    #[derive(Default)]
    struct InMemoryRepo {
        funds: Mutex<Vec<Fund>>,
        reassignments: Mutex<Vec<FundReassignment>>,
    }

    #[async_trait]
    impl FundRepository for InMemoryRepo {
        async fn create(&self, fund: &Fund) -> Result<Fund, AppError> {
            self.funds.lock().unwrap().push(fund.clone());
            Ok(fund.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Fund>, AppError> {
            Ok(self
                .funds
                .lock()
                .unwrap()
                .iter()
                .find(|f| f.id == id)
                .cloned())
        }

        async fn find_by_acp_id(&self, acp_id: Uuid) -> Result<Vec<Fund>, AppError> {
            Ok(self
                .funds
                .lock()
                .unwrap()
                .iter()
                .filter(|f| f.acp_id == acp_id)
                .cloned()
                .collect())
        }

        async fn update(&self, fund: &Fund) -> Result<Fund, AppError> {
            let mut funds = self.funds.lock().unwrap();
            if let Some(existing) = funds.iter_mut().find(|f| f.id == fund.id) {
                *existing = fund.clone();
            }
            Ok(fund.clone())
        }

        async fn record_reassignment(
            &self,
            reassignment: &FundReassignment,
        ) -> Result<(), AppError> {
            self.reassignments
                .lock()
                .unwrap()
                .push(reassignment.clone());
            Ok(())
        }

        async fn list_reassignments(
            &self,
            fund_id: Uuid,
        ) -> Result<Vec<FundReassignment>, AppError> {
            Ok(self
                .reassignments
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.fund_id == fund_id)
                .cloned()
                .collect())
        }
    }

    fn factory() -> (Arc<InMemoryRepo>, FundUseCases) {
        let repo: Arc<InMemoryRepo> = Arc::new(InMemoryRepo::default());
        let uc = FundUseCases::new(repo.clone() as Arc<dyn FundRepository>);
        (repo, uc)
    }

    // ---- @happy -------------------------------------------------------------

    #[tokio::test]
    async fn happy_earmarked_fund_created_and_fed_towards_its_target() {
        let (_repo, uc) = factory();
        let acp_id = Uuid::new_v4();

        let fund = uc
            .create_earmarked_fund(
                acp_id,
                "Toiture".to_string(),
                "Réfection toiture".to_string(),
                dec!(50000),
                MajorityType::TwoThirds,
            )
            .await
            .expect("vote suffisant");

        let fed = uc.contribute(fund.id, dec!(12500)).await.unwrap();
        assert_eq!(fed.balance, dec!(12500));
        assert_eq!(fed.progress(), Some(dec!(0.25)));
    }

    #[tokio::test]
    async fn happy_reassignment_persists_fund_and_audit_trail() {
        let (_repo, uc) = factory();
        let acp_id = Uuid::new_v4();
        let fund = uc
            .create_earmarked_fund(
                acp_id,
                "Ascenseur".to_string(),
                "Remplacement ascenseur".to_string(),
                dec!(20000),
                MajorityType::TwoThirds,
            )
            .await
            .unwrap();
        uc.contribute(fund.id, dec!(20000)).await.unwrap();
        let resolution_id = Uuid::new_v4();

        let record = uc
            .reassign(
                fund.id,
                "Ravalement façade".to_string(),
                None,
                resolution_id,
                ResolutionStatus::Adopted,
            )
            .await
            .expect("réaffectation autorisée par l'AG");

        assert_eq!(record.resolution_id, resolution_id);

        let updated = uc.get(fund.id).await.unwrap();
        assert_eq!(updated.purpose, Some("Ravalement façade".to_string()));

        let history = uc.list_reassignments(fund.id).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].previous_purpose, "Remplacement ascenseur");
    }

    // ---- @edge ----------------------------------------------------------------

    #[tokio::test]
    async fn edge_working_capital_and_reserve_keep_separate_balances() {
        let (_repo, uc) = factory();
        let acp_id = Uuid::new_v4();

        let roulement = uc
            .create_working_capital_or_reserve(
                acp_id,
                FundKind::WorkingCapital,
                "Roulement".to_string(),
            )
            .await
            .unwrap();
        let reserve = uc
            .create_working_capital_or_reserve(
                acp_id,
                FundKind::Reserve,
                "Réserve légale".to_string(),
            )
            .await
            .unwrap();

        uc.contribute(roulement.id, dec!(3000)).await.unwrap();
        uc.contribute(reserve.id, dec!(7000)).await.unwrap();

        let funds = uc.list_for_acp(acp_id).await.unwrap();
        assert_eq!(funds.len(), 2);
        let roulement_after = funds.iter().find(|f| f.id == roulement.id).unwrap();
        let reserve_after = funds.iter().find(|f| f.id == reserve.id).unwrap();
        assert_eq!(roulement_after.balance, dec!(3000));
        assert_eq!(reserve_after.balance, dec!(7000));
    }

    #[tokio::test]
    async fn edge_get_unknown_fund_returns_not_found() {
        let (_repo, uc) = factory();
        let err = uc.get(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    // ---- @security --------------------------------------------------------

    #[tokio::test]
    async fn security_creating_earmarked_fund_with_simple_majority_is_rejected() {
        let (_repo, uc) = factory();
        let err = uc
            .create_earmarked_fund(
                Uuid::new_v4(),
                "Toiture".to_string(),
                "Réfection toiture".to_string(),
                dec!(50000),
                MajorityType::Absolute,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn security_expense_outside_purpose_is_rejected_unless_ag_overrides() {
        let (_repo, uc) = factory();
        let fund = uc
            .create_earmarked_fund(
                Uuid::new_v4(),
                "Toiture".to_string(),
                "Réfection toiture".to_string(),
                dec!(50000),
                MajorityType::TwoThirds,
            )
            .await
            .unwrap();

        let refused = uc
            .record_expense(fund.id, "Ravalement façade", None)
            .await
            .unwrap_err();
        assert!(matches!(refused, AppError::Validation(_)));

        // Une décision d'AG adoptée lève la restriction.
        uc.record_expense(
            fund.id,
            "Ravalement façade",
            Some(ResolutionStatus::Adopted),
        )
        .await
        .expect("l'AG autorise la dépense hors objet");
    }

    // ---- @negative ----------------------------------------------------------

    #[tokio::test]
    async fn negative_reassign_unknown_fund_returns_not_found() {
        let (_repo, uc) = factory();
        let err = uc
            .reassign(
                Uuid::new_v4(),
                "Autre objet".to_string(),
                None,
                Uuid::new_v4(),
                ResolutionStatus::Adopted,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_reassign_without_adopted_resolution_is_rejected() {
        let (_repo, uc) = factory();
        let fund = uc
            .create_earmarked_fund(
                Uuid::new_v4(),
                "Toiture".to_string(),
                "Réfection toiture".to_string(),
                dec!(50000),
                MajorityType::TwoThirds,
            )
            .await
            .unwrap();
        uc.contribute(fund.id, dec!(1000)).await.unwrap();

        let err = uc
            .reassign(
                fund.id,
                "Autre objet".to_string(),
                None,
                Uuid::new_v4(),
                ResolutionStatus::Pending,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        // Aucun audit trail ne doit être créé pour une réaffectation refusée.
        assert!(uc.list_reassignments(fund.id).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn negative_contribute_to_unknown_fund_returns_not_found() {
        let (_repo, uc) = factory();
        let err = uc.contribute(Uuid::new_v4(), dec!(10)).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
