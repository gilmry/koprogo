//! Use cases for [`ContractorEvaluation`] (Story 3.9 — FR34 FR35 INV-21
//! INV-24).
//!
//! One operation exposed to handlers:
//!
//! 1. [`ContractorEvaluationUseCases::create_evaluation`] — a syndic (or a
//!    mandated owner) records an evaluation against a contractor. Guards
//!    enforced here:
//!    a. the referenced [`TechnicalSpec`] MUST exist (else
//!       [`AppError::NotFound`]);
//!    b. it MUST be in [`TechnicalSpecStatus::Approved`] — a Draft /
//!       PendingSignatures / Superseded spec does not legitimise an
//!       evaluation (else [`AppError::TechnicalSpecRequired`] → 422);
//!    c. [`ContractorEvaluation::new`] validates every structural invariant
//!       (scores in `[1, 5]`, comment length, no self-evaluation, no
//!       duplicate linked tickets, no nil UUIDs).
//!
//! Append-only behaviour is enforced at the DB trigger level — the use case
//! exposes no update / delete; the repo trait has no such methods either.
//!
//! [`TechnicalSpec`]: crate::domain::entities::TechnicalSpec
//! [`TechnicalSpecStatus::Approved`]: crate::domain::entities::TechnicalSpecStatus::Approved

use crate::application::error::AppError;
use crate::application::ports::{ContractorEvaluationRepository, TechnicalSpecRepository};
use crate::domain::entities::{ContractorEvaluation, EvaluationScores, TechnicalSpecStatus};
use std::sync::Arc;
use uuid::Uuid;

/// Use cases container. Holds trait-object handles to the two repositories
/// (parallel to [`TechnicalSpecUseCases`](super::TechnicalSpecUseCases)) so
/// AppState can store it as a non-generic `Arc<…>` without forcing every
/// downstream consumer to carry type parameters.
pub struct ContractorEvaluationUseCases {
    repo: Arc<dyn ContractorEvaluationRepository>,
    tech_spec_repo: Arc<dyn TechnicalSpecRepository>,
}

impl ContractorEvaluationUseCases {
    pub fn new(
        repo: Arc<dyn ContractorEvaluationRepository>,
        tech_spec_repo: Arc<dyn TechnicalSpecRepository>,
    ) -> Self {
        Self {
            repo,
            tech_spec_repo,
        }
    }

    /// Record a new evaluation. Guards in this order:
    ///
    /// 1. The TechnicalSpec MUST exist — else `NotFound`.
    /// 2. It MUST be in `Approved` status — else `TechnicalSpecRequired` 422.
    ///    A Draft / PendingSignatures spec means the prestation has not been
    ///    formally signed off; a Superseded spec means a later version
    ///    governs the current works and that newer spec is what should
    ///    legitimise the evaluation.
    /// 3. The entity constructor enforces every structural invariant
    ///    (including `evaluator_user_id != contractor_user_id`, INV-21).
    pub async fn create_evaluation(
        &self,
        contractor_user_id: Uuid,
        technical_spec_id: Uuid,
        linked_ticket_ids: Vec<Uuid>,
        evaluator_user_id: Uuid,
        scores: EvaluationScores,
        comment: String,
    ) -> Result<ContractorEvaluation, AppError> {
        // 1+2. Spec exists AND is Approved.
        let spec = self
            .tech_spec_repo
            .find_by_id(technical_spec_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("technical_spec {}", technical_spec_id)))?;

        if !matches!(spec.status, TechnicalSpecStatus::Approved) {
            return Err(AppError::TechnicalSpecRequired);
        }

        // 3. Structural invariants (typed errors).
        let evaluation = ContractorEvaluation::new(
            contractor_user_id,
            technical_spec_id,
            linked_ticket_ids,
            evaluator_user_id,
            scores,
            comment,
        )?;

        self.repo.save(&evaluation).await?;
        Ok(evaluation)
    }

    pub async fn get_evaluation(&self, id: Uuid) -> Result<ContractorEvaluation, AppError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("contractor_evaluation {}", id)))
    }

    pub async fn list_for_contractor(
        &self,
        contractor_user_id: Uuid,
    ) -> Result<Vec<ContractorEvaluation>, AppError> {
        self.repo.list_for_contractor(contractor_user_id).await
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories (CRITICAL.md #3, Story 3.9)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{
        SemVer, SignatoryRole, TechnicalSpec, TechnicalSpecSignature, TechnicalSpecStatus,
    };
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Mutex;

    // ── Mock ContractorEvaluation repository ───────────────────────────────

    #[derive(Default)]
    struct InMemoryEvalRepo {
        rows: Mutex<HashMap<Uuid, ContractorEvaluation>>,
    }

    #[async_trait]
    impl ContractorEvaluationRepository for InMemoryEvalRepo {
        async fn save(&self, e: &ContractorEvaluation) -> Result<(), AppError> {
            // Idempotent: PK collision is acceptable in a mock (the production
            // INSERT would fail with 23505, but here we don't simulate that —
            // the use-case never replays the same id).
            self.rows.lock().unwrap().insert(e.id, e.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<ContractorEvaluation>, AppError> {
            Ok(self.rows.lock().unwrap().get(&id).cloned())
        }

        async fn list_for_contractor(
            &self,
            contractor_user_id: Uuid,
        ) -> Result<Vec<ContractorEvaluation>, AppError> {
            let rows = self.rows.lock().unwrap();
            let mut out: Vec<ContractorEvaluation> = rows
                .values()
                .filter(|e| e.contractor_user_id == contractor_user_id)
                .cloned()
                .collect();
            out.sort_by_key(|e| std::cmp::Reverse(e.created_at));
            Ok(out)
        }
    }

    // ── Mock TechnicalSpec repository ──────────────────────────────────────

    #[derive(Default)]
    struct InMemoryTechSpecRepo {
        specs: Mutex<HashMap<Uuid, TechnicalSpec>>,
        signatures: Mutex<Vec<TechnicalSpecSignature>>,
    }

    #[async_trait]
    impl TechnicalSpecRepository for InMemoryTechSpecRepo {
        async fn save(&self, spec: &TechnicalSpec) -> Result<(), AppError> {
            self.specs.lock().unwrap().insert(spec.id, spec.clone());
            Ok(())
        }

        async fn update_status(
            &self,
            spec_id: Uuid,
            status: &str,
            updated_at: DateTime<Utc>,
        ) -> Result<(), AppError> {
            let mut specs = self.specs.lock().unwrap();
            if let Some(spec) = specs.get_mut(&spec_id) {
                spec.status = TechnicalSpecStatus::from_str(status)?;
                spec.updated_at = updated_at;
            }
            Ok(())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<TechnicalSpec>, AppError> {
            Ok(self.specs.lock().unwrap().get(&id).cloned())
        }

        async fn list_for_acp(&self, acp_id: Uuid) -> Result<Vec<TechnicalSpec>, AppError> {
            let specs = self.specs.lock().unwrap();
            Ok(specs
                .values()
                .filter(|s| s.acp_id == acp_id)
                .cloned()
                .collect())
        }

        async fn save_signature(&self, sig: &TechnicalSpecSignature) -> Result<(), AppError> {
            self.signatures.lock().unwrap().push(sig.clone());
            Ok(())
        }

        async fn list_signatures_for_spec(
            &self,
            spec_id: Uuid,
        ) -> Result<Vec<TechnicalSpecSignature>, AppError> {
            let sigs = self.signatures.lock().unwrap();
            Ok(sigs
                .iter()
                .filter(|s| s.technical_spec_id == spec_id)
                .cloned()
                .collect())
        }
    }

    // ── Fixtures ───────────────────────────────────────────────────────────

    fn make_use_cases() -> (
        Arc<InMemoryEvalRepo>,
        Arc<InMemoryTechSpecRepo>,
        ContractorEvaluationUseCases,
    ) {
        let eval_repo: Arc<InMemoryEvalRepo> = Arc::new(InMemoryEvalRepo::default());
        let spec_repo: Arc<InMemoryTechSpecRepo> = Arc::new(InMemoryTechSpecRepo::default());
        let uc = ContractorEvaluationUseCases::new(
            eval_repo.clone() as Arc<dyn ContractorEvaluationRepository>,
            spec_repo.clone() as Arc<dyn TechnicalSpecRepository>,
        );
        (eval_repo, spec_repo, uc)
    }

    fn fixture_scores_all_top() -> EvaluationScores {
        EvaluationScores {
            quality: 5,
            timeliness: 5,
            communication: 5,
            cost_compliance: 5,
            overall: 5,
        }
    }

    fn fixture_comment() -> String {
        // 60+ chars to comfortably pass MIN_COMMENT_LEN (10).
        "Travail soigné, livré dans les délais, communication impeccable.".to_string()
    }

    fn fixture_description() -> String {
        // 50+ chars to satisfy TechnicalSpec MIN_DESCRIPTION_LEN.
        "Renovation toiture batiment A : etancheite, isolation 18 cm laine de roche.".to_string()
    }

    fn fixture_deliverables() -> Vec<String> {
        vec![
            "Plan d'execution".to_string(),
            "Cahier des charges".to_string(),
        ]
    }

    async fn insert_spec_with_status(
        spec_repo: &Arc<InMemoryTechSpecRepo>,
        status: TechnicalSpecStatus,
    ) -> Uuid {
        let mut spec = TechnicalSpec::new(
            Uuid::new_v4(),
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            vec![SignatoryRole::Syndic],
            Vec::new(),
            None,
            Uuid::new_v4(),
        )
        .unwrap();
        spec.status = status;
        let id = spec.id;
        spec_repo.save(&spec).await.unwrap();
        id
    }

    // ---- @happy ---------------------------------------------------------

    #[tokio::test]
    async fn happy_create_evaluation_when_spec_is_approved() {
        let (_eval_repo, spec_repo, uc) = make_use_cases();
        let spec_id = insert_spec_with_status(&spec_repo, TechnicalSpecStatus::Approved).await;
        let contractor = Uuid::new_v4();
        let evaluator = Uuid::new_v4();
        let e = uc
            .create_evaluation(
                contractor,
                spec_id,
                Vec::new(),
                evaluator,
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await
            .expect("evaluation must be created");
        assert_eq!(e.contractor_user_id, contractor);
        assert_eq!(e.technical_spec_id, spec_id);
        assert_eq!(e.average_score(), 5.0);
    }

    #[tokio::test]
    async fn happy_list_for_contractor_returns_newest_first() {
        let (_eval_repo, spec_repo, uc) = make_use_cases();
        let spec_id = insert_spec_with_status(&spec_repo, TechnicalSpecStatus::Approved).await;
        let contractor = Uuid::new_v4();
        let _first = uc
            .create_evaluation(
                contractor,
                spec_id,
                Vec::new(),
                Uuid::new_v4(),
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await
            .unwrap();
        // Tiny sleep so created_at ordering is observable.
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        let _second = uc
            .create_evaluation(
                contractor,
                spec_id,
                Vec::new(),
                Uuid::new_v4(),
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await
            .unwrap();
        let listed = uc.list_for_contractor(contractor).await.unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed[0].created_at >= listed[1].created_at);
    }

    // ---- @edge ----------------------------------------------------------

    #[tokio::test]
    async fn edge_empty_linked_tickets_is_accepted() {
        let (_eval_repo, spec_repo, uc) = make_use_cases();
        let spec_id = insert_spec_with_status(&spec_repo, TechnicalSpecStatus::Approved).await;
        let res = uc
            .create_evaluation(
                Uuid::new_v4(),
                spec_id,
                Vec::new(),
                Uuid::new_v4(),
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn edge_spec_freshly_promoted_to_approved_is_accepted() {
        // The use-case checks `status == Approved` strictly; a spec that was
        // marked approved the same second as the evaluation is still valid.
        let (_eval_repo, spec_repo, uc) = make_use_cases();
        let spec_id = insert_spec_with_status(&spec_repo, TechnicalSpecStatus::Approved).await;
        // Re-promote via update_status (idempotent on Approved).
        spec_repo
            .update_status(spec_id, "approved", Utc::now())
            .await
            .unwrap();
        let res = uc
            .create_evaluation(
                Uuid::new_v4(),
                spec_id,
                Vec::new(),
                Uuid::new_v4(),
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await;
        assert!(res.is_ok());
    }

    // ---- @security ------------------------------------------------------

    #[tokio::test]
    async fn security_evaluator_equals_contractor_returns_typed_error() {
        let (_eval_repo, spec_repo, uc) = make_use_cases();
        let spec_id = insert_spec_with_status(&spec_repo, TechnicalSpecStatus::Approved).await;
        let same = Uuid::new_v4();
        let err = uc
            .create_evaluation(
                same,
                spec_id,
                Vec::new(),
                same,
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::EvaluatorIsContractor));
    }

    #[tokio::test]
    async fn security_double_save_with_unique_payload_is_idempotent_at_uc_level() {
        // Two distinct create_evaluation calls mint two distinct ids; both
        // succeed and persist as separate rows. The append-only DB constraint
        // is checked in e2e/integration (cannot be unit-tested without the
        // trigger). This test guards against an accidental dedup logic in
        // the use-case.
        let (eval_repo, spec_repo, uc) = make_use_cases();
        let spec_id = insert_spec_with_status(&spec_repo, TechnicalSpecStatus::Approved).await;
        let contractor = Uuid::new_v4();
        let evaluator = Uuid::new_v4();
        let _e1 = uc
            .create_evaluation(
                contractor,
                spec_id,
                Vec::new(),
                evaluator,
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await
            .unwrap();
        let _e2 = uc
            .create_evaluation(
                contractor,
                spec_id,
                Vec::new(),
                evaluator,
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await
            .unwrap();
        assert_eq!(eval_repo.rows.lock().unwrap().len(), 2);
    }

    // ---- @negative ------------------------------------------------------

    #[tokio::test]
    async fn negative_unknown_spec_returns_not_found() {
        let (_eval_repo, _spec_repo, uc) = make_use_cases();
        let err = uc
            .create_evaluation(
                Uuid::new_v4(),
                Uuid::new_v4(), // never inserted
                Vec::new(),
                Uuid::new_v4(),
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_draft_spec_returns_technical_spec_required() {
        let (_eval_repo, spec_repo, uc) = make_use_cases();
        let spec_id = insert_spec_with_status(&spec_repo, TechnicalSpecStatus::Draft).await;
        let err = uc
            .create_evaluation(
                Uuid::new_v4(),
                spec_id,
                Vec::new(),
                Uuid::new_v4(),
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::TechnicalSpecRequired));
    }

    #[tokio::test]
    async fn negative_pending_signatures_spec_returns_technical_spec_required() {
        let (_eval_repo, spec_repo, uc) = make_use_cases();
        let spec_id =
            insert_spec_with_status(&spec_repo, TechnicalSpecStatus::PendingSignatures).await;
        let err = uc
            .create_evaluation(
                Uuid::new_v4(),
                spec_id,
                Vec::new(),
                Uuid::new_v4(),
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::TechnicalSpecRequired));
    }

    #[tokio::test]
    async fn negative_superseded_spec_returns_technical_spec_required() {
        let (_eval_repo, spec_repo, uc) = make_use_cases();
        let spec_id = insert_spec_with_status(&spec_repo, TechnicalSpecStatus::Superseded).await;
        let err = uc
            .create_evaluation(
                Uuid::new_v4(),
                spec_id,
                Vec::new(),
                Uuid::new_v4(),
                fixture_scores_all_top(),
                fixture_comment(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::TechnicalSpecRequired));
    }

    #[tokio::test]
    async fn negative_get_unknown_evaluation_returns_not_found() {
        let (_eval_repo, _spec_repo, uc) = make_use_cases();
        let err = uc.get_evaluation(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_invalid_score_propagates_validation_error() {
        let (_eval_repo, spec_repo, uc) = make_use_cases();
        let spec_id = insert_spec_with_status(&spec_repo, TechnicalSpecStatus::Approved).await;
        let bad = EvaluationScores {
            quality: 5,
            timeliness: 5,
            communication: 5,
            cost_compliance: 5,
            overall: 0,
        };
        let err = uc
            .create_evaluation(
                Uuid::new_v4(),
                spec_id,
                Vec::new(),
                Uuid::new_v4(),
                bad,
                fixture_comment(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
