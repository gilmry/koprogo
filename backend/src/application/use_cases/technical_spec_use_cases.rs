//! Use cases for [`TechnicalSpec`] + signatures (Story 3.8 — FR33).
//!
//! Operations:
//!
//! 1. [`TechnicalSpecUseCases::create_spec`] — syndic creates a new spec in
//!    `Draft`. Caller (handler) MUST already have enforced the syndic /
//!    superadmin role check.
//! 2. [`TechnicalSpecUseCases::bump_version`] — create a new version from a
//!    previous spec. Always returns a fresh `Draft`; the version must be
//!    strictly greater than the previous one. The use case marks the
//!    previous spec as `Superseded` once the bump succeeds.
//! 3. [`TechnicalSpecUseCases::submit_for_signatures`] — Draft -> Pending.
//! 4. [`TechnicalSpecUseCases::sign_spec`] — record a signature, returning
//!    409 on duplicate, 403 when the signatory role is not in the
//!    `required_signatures` list. When the new signature completes the
//!    required set, the spec auto-promotes to `Approved`.
//!
//! Mandate verification for AMO / Lawyer / Architect roles is handled at
//! the entity level ([`TechnicalSpecSignature::new`] returns
//! `SignatoryNotAuthorized` when `mandate_id` is missing for a role that
//! requires it). Real cryptographic e-signatures and Mandate-active checks
//! against the [`crate::application::use_cases::MandateUseCases`] are
//! follow-ups tracked in the Story 3.8 acceptance notes.

use crate::application::error::AppError;
use crate::application::ports::TechnicalSpecRepository;
use crate::domain::entities::{
    SemVer, SignatoryRole, TechnicalSpec, TechnicalSpecSignature, TechnicalSpecStatus,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct TechnicalSpecUseCases {
    repo: Arc<dyn TechnicalSpecRepository>,
}

impl TechnicalSpecUseCases {
    pub fn new(repo: Arc<dyn TechnicalSpecRepository>) -> Self {
        Self { repo }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_spec(
        &self,
        acp_id: Uuid,
        building_id: Option<Uuid>,
        title: String,
        description: String,
        version: SemVer,
        deliverables: Vec<String>,
        required_signatures: Vec<SignatoryRole>,
        attachments: Vec<String>,
        created_by: Uuid,
    ) -> Result<TechnicalSpec, AppError> {
        let spec = TechnicalSpec::new(
            acp_id,
            building_id,
            title,
            description,
            version,
            deliverables,
            required_signatures,
            attachments,
            None,
            created_by,
        )?;
        self.repo.save(&spec).await?;
        Ok(spec)
    }

    /// Create a new version of an existing spec. The previous spec is moved
    /// to `Superseded` so list views can flag the live version. The returned
    /// spec is always a fresh `Draft`.
    pub async fn bump_version(
        &self,
        previous_spec_id: Uuid,
        new_version: SemVer,
        new_title: Option<String>,
        new_description: Option<String>,
        new_deliverables: Option<Vec<String>>,
        new_required_signatures: Option<Vec<SignatoryRole>>,
        new_attachments: Option<Vec<String>>,
    ) -> Result<TechnicalSpec, AppError> {
        let prev = self
            .repo
            .find_by_id(previous_spec_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("technical_spec {}", previous_spec_id)))?;

        let new_spec = prev.bump(
            new_version,
            new_title,
            new_description,
            new_deliverables,
            new_required_signatures,
            new_attachments,
        )?;
        self.repo.save(&new_spec).await?;
        // Mark previous as Superseded (best-effort: the bump itself already
        // succeeded; subsequent failure to flip the prior status is logged
        // by the caller, but the new draft is the source of truth).
        self.repo
            .update_status(
                prev.id,
                &TechnicalSpecStatus::Superseded.to_string(),
                chrono::Utc::now(),
            )
            .await?;

        Ok(new_spec)
    }

    /// Draft -> PendingSignatures.
    pub async fn submit_for_signatures(&self, spec_id: Uuid) -> Result<TechnicalSpec, AppError> {
        let mut spec = self
            .repo
            .find_by_id(spec_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("technical_spec {}", spec_id)))?;
        spec.submit_for_signatures()?;
        self.repo
            .update_status(spec.id, &spec.status.to_string(), spec.updated_at)
            .await?;
        Ok(spec)
    }

    /// Sign a TechnicalSpec.
    ///
    /// Guards (in order):
    /// - spec MUST exist (404 NotFound),
    /// - spec MUST be in `PendingSignatures` (else `Validation` error: the
    ///   workflow has not opened the signature window yet),
    /// - `role` MUST be in `spec.required_signatures` (else `SignatoryNotAuthorized`),
    /// - Mandataire roles (AMO / Lawyer / Architect) MUST carry a
    ///   `mandate_id` (entity-level guard surfaces `SignatoryNotAuthorized`),
    /// - duplicate (spec_id, user, role) tuples return `SignatureAlreadyExists`.
    ///
    /// When this signature completes the required set, the spec auto-promotes
    /// to `Approved` (idempotent at the SQL level: `update_status` is safe to
    /// repeat).
    pub async fn sign_spec(
        &self,
        spec_id: Uuid,
        signatory_user_id: Uuid,
        role: SignatoryRole,
        mandate_id: Option<Uuid>,
    ) -> Result<TechnicalSpecSignature, AppError> {
        let mut spec = self
            .repo
            .find_by_id(spec_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("technical_spec {}", spec_id)))?;

        if !matches!(spec.status, TechnicalSpecStatus::PendingSignatures) {
            return Err(AppError::Validation(format!(
                "TechnicalSpec must be PendingSignatures to be signed (is {})",
                spec.status
            )));
        }
        if !spec.required_signatures.contains(&role) {
            return Err(AppError::SignatoryNotAuthorized);
        }

        // Check for duplicate signature on (spec, user, role).
        let existing = self.repo.list_signatures_for_spec(spec_id).await?;
        if existing
            .iter()
            .any(|s| s.signatory_user_id == signatory_user_id && s.role == role)
        {
            return Err(AppError::SignatureAlreadyExists);
        }

        let sig = TechnicalSpecSignature::new(spec_id, signatory_user_id, role, mandate_id)?;
        self.repo.save_signature(&sig).await?;

        // Auto-approve when all required roles are now covered.
        let mut collected: Vec<SignatoryRole> = existing.iter().map(|s| s.role).collect();
        collected.push(role);
        if spec.has_all_required_signatures(&collected) {
            spec.mark_approved();
            self.repo
                .update_status(spec.id, &spec.status.to_string(), spec.updated_at)
                .await?;
        }

        Ok(sig)
    }

    pub async fn get(&self, spec_id: Uuid) -> Result<TechnicalSpec, AppError> {
        self.repo
            .find_by_id(spec_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("technical_spec {}", spec_id)))
    }

    pub async fn list_for_acp(&self, acp_id: Uuid) -> Result<Vec<TechnicalSpec>, AppError> {
        self.repo.list_for_acp(acp_id).await
    }

    pub async fn list_signatures_for_spec(
        &self,
        spec_id: Uuid,
    ) -> Result<Vec<TechnicalSpecSignature>, AppError> {
        self.repo.list_signatures_for_spec(spec_id).await
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories (CRITICAL.md #3, Story 3.8)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Mutex;

    // ── Mock repository ─────────────────────────────────────────────────

    #[derive(Default)]
    struct InMemoryRepo {
        specs: Mutex<HashMap<Uuid, TechnicalSpec>>,
        signatures: Mutex<Vec<TechnicalSpecSignature>>,
    }

    #[async_trait]
    impl TechnicalSpecRepository for InMemoryRepo {
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
            let mut out: Vec<TechnicalSpec> = specs
                .values()
                .filter(|s| s.acp_id == acp_id)
                .cloned()
                .collect();
            out.sort_by_key(|s| std::cmp::Reverse(s.created_at));
            Ok(out)
        }

        async fn save_signature(&self, sig: &TechnicalSpecSignature) -> Result<(), AppError> {
            // Mirror the SQL UNIQUE (spec_id, user, role) constraint.
            let mut sigs = self.signatures.lock().unwrap();
            if sigs.iter().any(|s| {
                s.technical_spec_id == sig.technical_spec_id
                    && s.signatory_user_id == sig.signatory_user_id
                    && s.role == sig.role
            }) {
                return Err(AppError::SignatureAlreadyExists);
            }
            sigs.push(sig.clone());
            Ok(())
        }

        async fn list_signatures_for_spec(
            &self,
            spec_id: Uuid,
        ) -> Result<Vec<TechnicalSpecSignature>, AppError> {
            let sigs = self.signatures.lock().unwrap();
            let mut out: Vec<TechnicalSpecSignature> = sigs
                .iter()
                .filter(|s| s.technical_spec_id == spec_id)
                .cloned()
                .collect();
            out.sort_by_key(|s| s.signed_at);
            Ok(out)
        }
    }

    fn make_use_cases() -> (Arc<InMemoryRepo>, TechnicalSpecUseCases) {
        let repo: Arc<InMemoryRepo> = Arc::new(InMemoryRepo::default());
        let uc = TechnicalSpecUseCases::new(repo.clone() as Arc<dyn TechnicalSpecRepository>);
        (repo, uc)
    }

    fn fixture_description() -> String {
        "Renovation toiture batiment A : etancheite, isolation 18 cm.".to_string()
    }

    fn fixture_deliverables() -> Vec<String> {
        vec![
            "Plan d'execution".to_string(),
            "Cahier des charges".to_string(),
        ]
    }

    async fn create_pending_spec(
        uc: &TechnicalSpecUseCases,
        required: Vec<SignatoryRole>,
    ) -> TechnicalSpec {
        let (acp, user) = (Uuid::new_v4(), Uuid::new_v4());
        let spec = uc
            .create_spec(
                acp,
                None,
                "Toiture".to_string(),
                fixture_description(),
                SemVer::new(1, 0, 0),
                fixture_deliverables(),
                required,
                Vec::new(),
                user,
            )
            .await
            .expect("create_spec must succeed");
        uc.submit_for_signatures(spec.id).await.unwrap()
    }

    // ---- @happy ---------------------------------------------------------

    #[tokio::test]
    async fn happy_create_submit_then_signatures_complete_approves() {
        let (_repo, uc) = make_use_cases();
        let spec = create_pending_spec(
            &uc,
            vec![SignatoryRole::Syndic, SignatoryRole::AcpRepresentative],
        )
        .await;

        // First sig — still PendingSignatures.
        let _sig1 = uc
            .sign_spec(spec.id, Uuid::new_v4(), SignatoryRole::Syndic, None)
            .await
            .unwrap();
        let mid = uc.get(spec.id).await.unwrap();
        assert_eq!(mid.status, TechnicalSpecStatus::PendingSignatures);

        // Second sig — now Approved.
        let _sig2 = uc
            .sign_spec(
                spec.id,
                Uuid::new_v4(),
                SignatoryRole::AcpRepresentative,
                None,
            )
            .await
            .unwrap();
        let final_spec = uc.get(spec.id).await.unwrap();
        assert_eq!(final_spec.status, TechnicalSpecStatus::Approved);
    }

    #[tokio::test]
    async fn happy_list_for_acp_returns_newest_first() {
        let (_repo, uc) = make_use_cases();
        let acp = Uuid::new_v4();
        let user = Uuid::new_v4();
        let _first = uc
            .create_spec(
                acp,
                None,
                "Toiture".to_string(),
                fixture_description(),
                SemVer::new(1, 0, 0),
                fixture_deliverables(),
                vec![SignatoryRole::Syndic],
                Vec::new(),
                user,
            )
            .await
            .unwrap();
        // Tiny sleep to make sure created_at ordering is observable.
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
        let _second = uc
            .create_spec(
                acp,
                None,
                "Facade".to_string(),
                fixture_description(),
                SemVer::new(1, 0, 0),
                fixture_deliverables(),
                vec![SignatoryRole::Syndic],
                Vec::new(),
                user,
            )
            .await
            .unwrap();
        let listed = uc.list_for_acp(acp).await.unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed[0].created_at >= listed[1].created_at);
    }

    // ---- @edge ----------------------------------------------------------

    #[tokio::test]
    async fn edge_bump_minor_keeps_previous_signatures_conceptually() {
        let (_repo, uc) = make_use_cases();
        let v1 = create_pending_spec(&uc, vec![SignatoryRole::Syndic]).await;
        // Approve v1.
        uc.sign_spec(v1.id, Uuid::new_v4(), SignatoryRole::Syndic, None)
            .await
            .unwrap();
        assert_eq!(
            uc.get(v1.id).await.unwrap().status,
            TechnicalSpecStatus::Approved
        );

        // Minor bump v1.0.0 -> v1.1.0.
        let v2 = uc
            .bump_version(v1.id, SemVer::new(1, 1, 0), None, None, None, None, None)
            .await
            .unwrap();
        assert_eq!(v2.status, TechnicalSpecStatus::Draft);
        assert_eq!(v2.previous_version_id, Some(v1.id));
        // Previous is now Superseded.
        assert_eq!(
            uc.get(v1.id).await.unwrap().status,
            TechnicalSpecStatus::Superseded
        );
        // No new signatures auto-attached: v2 has none yet.
        assert!(uc.list_signatures_for_spec(v2.id).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn edge_signing_when_spec_not_pending_is_rejected() {
        let (_repo, uc) = make_use_cases();
        // Draft spec — not submitted yet.
        let (acp, user) = (Uuid::new_v4(), Uuid::new_v4());
        let draft = uc
            .create_spec(
                acp,
                None,
                "Toiture".to_string(),
                fixture_description(),
                SemVer::new(1, 0, 0),
                fixture_deliverables(),
                vec![SignatoryRole::Syndic],
                Vec::new(),
                user,
            )
            .await
            .unwrap();
        let err = uc
            .sign_spec(draft.id, Uuid::new_v4(), SignatoryRole::Syndic, None)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ---- @security ------------------------------------------------------

    #[tokio::test]
    async fn security_sign_with_role_not_in_required_set_rejected() {
        let (_repo, uc) = make_use_cases();
        let spec = create_pending_spec(&uc, vec![SignatoryRole::Syndic]).await;
        let err = uc
            .sign_spec(
                spec.id,
                Uuid::new_v4(),
                SignatoryRole::AcpRepresentative,
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::SignatoryNotAuthorized));
    }

    #[tokio::test]
    async fn security_mandataire_role_without_mandate_id_rejected() {
        let (_repo, uc) = make_use_cases();
        let spec = create_pending_spec(&uc, vec![SignatoryRole::Lawyer]).await;
        let err = uc
            .sign_spec(spec.id, Uuid::new_v4(), SignatoryRole::Lawyer, None)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::SignatoryNotAuthorized));
    }

    // ---- @negative ------------------------------------------------------

    #[tokio::test]
    async fn negative_duplicate_signature_returns_conflict() {
        let (_repo, uc) = make_use_cases();
        let spec = create_pending_spec(
            &uc,
            vec![SignatoryRole::Syndic, SignatoryRole::AcpRepresentative],
        )
        .await;
        let syndic_user = Uuid::new_v4();
        uc.sign_spec(spec.id, syndic_user, SignatoryRole::Syndic, None)
            .await
            .unwrap();
        // Same (user, role) — duplicate.
        let err = uc
            .sign_spec(spec.id, syndic_user, SignatoryRole::Syndic, None)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::SignatureAlreadyExists));
    }

    #[tokio::test]
    async fn negative_submit_unknown_spec_returns_not_found() {
        let (_repo, uc) = make_use_cases();
        let err = uc.submit_for_signatures(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_bump_unknown_spec_returns_not_found() {
        let (_repo, uc) = make_use_cases();
        let err = uc
            .bump_version(
                Uuid::new_v4(),
                SemVer::new(2, 0, 0),
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_bump_to_equal_version_rejected_at_entity_level() {
        let (_repo, uc) = make_use_cases();
        let spec = create_pending_spec(&uc, vec![SignatoryRole::Syndic]).await;
        let err = uc
            .bump_version(spec.id, SemVer::new(1, 0, 0), None, None, None, None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
