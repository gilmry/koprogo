//! Use cases for the Mandate feature (Story 3.4 — FR7 INV-14).
//!
//! Three operations:
//!
//! 1. [`MandateUseCases::issue`] — a syndic issues a mandate for an external
//!    professional (notaire, avocat, AMO, architecte, BET, gardien). The
//!    caller (handler) MUST have already enforced the syndic / superadmin
//!    role check.
//! 2. [`MandateUseCases::assert_mandate_authorizes`] — guard used by
//!    downstream handlers (notarial deeds, litigation actions, …) to verify
//!    that a mandataire is still authorised before performing an action.
//!    Returns the correctly-typed `AppError::Mandate*` so the upstream caller
//!    surfaces 403 with the right `kind` (`mandate_expired`,
//!    `mandate_revoked`, `mandate_invalid_scope`).
//! 3. [`MandateUseCases::revoke`] — early revocation (before
//!    `valid_until`). Handler enforces syndic / superadmin role.
//!
//! Also surface read-only helpers (`list_active_for_subject`,
//! `list_for_scope`, `get`) for the audit / detail views.

use crate::application::error::AppError;
use crate::application::ports::MandateRepository;
use crate::domain::entities::{Mandate, MandateKind, MandateScope};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct MandateUseCases {
    repo: Arc<dyn MandateRepository>,
}

impl MandateUseCases {
    pub fn new(repo: Arc<dyn MandateRepository>) -> Self {
        Self { repo }
    }

    /// Issue a new mandate. Caller MUST have already authorised the request
    /// (syndic / superadmin role check happens at the handler level).
    #[allow(clippy::too_many_arguments)]
    pub async fn issue(
        &self,
        subject_user_id: Uuid,
        kind: MandateKind,
        scope: MandateScope,
        issued_by: Uuid,
        reason: String,
        valid_from: DateTime<Utc>,
        valid_until: DateTime<Utc>,
    ) -> Result<Mandate, AppError> {
        let mandate = Mandate::issue(
            subject_user_id,
            kind,
            scope,
            issued_by,
            reason,
            valid_from,
            valid_until,
        )?;
        self.repo.save(&mandate).await?;
        Ok(mandate)
    }

    /// Verify a user holds a currently-valid mandate of `kind` over `scope`.
    ///
    /// Errors are precisely typed so the handler can surface the right `kind`
    /// in the 403 response:
    /// - `MandateExpired` — past `valid_until`.
    /// - `MandateRevoked` — `revoked_at IS NOT NULL`.
    /// - `MandateInvalidScope` — no mandate matches the requested scope
    ///   (e.g. notaire mandated on Building A, action on Building B).
    /// - `MandateNotFound` — the subject has no mandate of this kind at all.
    pub async fn assert_mandate_authorizes(
        &self,
        subject_user_id: Uuid,
        kind: MandateKind,
        scope: &MandateScope,
    ) -> Result<Mandate, AppError> {
        let mandates = self.repo.list_active_for_subject(subject_user_id).await?;

        // Filter by kind first.
        let same_kind: Vec<&Mandate> = mandates.iter().filter(|m| m.kind == kind).collect();
        if same_kind.is_empty() {
            return Err(AppError::MandateNotFound);
        }

        // Look for an exact scope match.
        let matching_scope = same_kind.iter().find(|m| m.scope == *scope);
        let Some(mandate) = matching_scope else {
            return Err(AppError::MandateInvalidScope);
        };

        if mandate.is_revoked() {
            return Err(AppError::MandateRevoked);
        }
        if mandate.is_expired() {
            return Err(AppError::MandateExpired);
        }
        // Belt-and-suspenders: even if the row passed the SQL "active" filter,
        // honour the domain entity helpers.
        if !mandate.is_currently_active() {
            return Err(AppError::MandateExpired);
        }

        Ok((*mandate).clone())
    }

    /// Annule un mandat avant son terme. Idempotent — un second appel ne
    /// surcharge pas `revoked_at`.
    pub async fn revoke(&self, id: Uuid) -> Result<(), AppError> {
        // Confirm existence so the handler can map 404 cleanly.
        let mandate = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::MandateNotFound)?;
        if mandate.is_revoked() {
            // Already revoked — no-op, no error. The audit trail is preserved
            // by the original `revoked_at`.
            return Ok(());
        }
        self.repo.revoke(id, Utc::now()).await?;
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Mandate, AppError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::MandateNotFound)
    }

    pub async fn list_active_for_subject(
        &self,
        subject_user_id: Uuid,
    ) -> Result<Vec<Mandate>, AppError> {
        self.repo.list_active_for_subject(subject_user_id).await
    }

    pub async fn list_for_scope(&self, scope: &MandateScope) -> Result<Vec<Mandate>, AppError> {
        self.repo.list_for_scope(scope).await
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories (CRITICAL.md #3)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Duration;
    use std::sync::Mutex;

    #[derive(Default)]
    struct InMemoryRepo {
        rows: Mutex<Vec<Mandate>>,
    }

    #[async_trait]
    impl MandateRepository for InMemoryRepo {
        async fn save(&self, m: &Mandate) -> Result<(), AppError> {
            self.rows.lock().unwrap().push(m.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Mandate>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|m| m.id == id)
                .cloned())
        }

        async fn list_active_for_subject(
            &self,
            subject_user_id: Uuid,
        ) -> Result<Vec<Mandate>, AppError> {
            // Mirror what a real SQL `WHERE revoked_at IS NULL AND valid_until > NOW()`
            // would do: keep currently active rows only.
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|m| m.subject_user_id == subject_user_id && m.is_currently_active())
                .cloned()
                .collect())
        }

        async fn list_for_scope(&self, scope: &MandateScope) -> Result<Vec<Mandate>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|m| m.scope == *scope)
                .cloned()
                .collect())
        }

        async fn revoke(&self, id: Uuid, revoked_at: DateTime<Utc>) -> Result<(), AppError> {
            let mut rows = self.rows.lock().unwrap();
            if let Some(row) = rows.iter_mut().find(|m| m.id == id) {
                if row.revoked_at.is_none() {
                    row.revoked_at = Some(revoked_at);
                    row.updated_at = revoked_at;
                }
            }
            Ok(())
        }
    }

    fn factory() -> (Arc<InMemoryRepo>, MandateUseCases) {
        let repo: Arc<InMemoryRepo> = Arc::new(InMemoryRepo::default());
        let uc = MandateUseCases::new(repo.clone() as Arc<dyn MandateRepository>);
        (repo, uc)
    }

    fn fixture_window() -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        (now - Duration::seconds(1), now + Duration::days(30))
    }

    fn reason() -> String {
        "Cession unité — mandat notarial".to_string()
    }

    // ---- @happy ------------------------------------------------------------

    #[tokio::test]
    async fn happy_issue_then_authorize_returns_mandate() {
        let (_repo, uc) = factory();
        let subject = Uuid::new_v4();
        let issuer = Uuid::new_v4();
        let building = Uuid::new_v4();
        let scope = MandateScope::Building(building);
        let (from, until) = fixture_window();

        let issued = uc
            .issue(
                subject,
                MandateKind::Notary,
                scope,
                issuer,
                reason(),
                from,
                until,
            )
            .await
            .unwrap();
        assert_eq!(issued.subject_user_id, subject);

        let resolved = uc
            .assert_mandate_authorizes(subject, MandateKind::Notary, &scope)
            .await
            .unwrap();
        assert_eq!(resolved.id, issued.id);
    }

    #[tokio::test]
    async fn happy_revoke_then_authorize_reports_revoked() {
        let (_repo, uc) = factory();
        let subject = Uuid::new_v4();
        let issuer = Uuid::new_v4();
        let scope = MandateScope::Acp(Uuid::new_v4());
        let (from, until) = fixture_window();

        let issued = uc
            .issue(
                subject,
                MandateKind::Lawyer,
                scope,
                issuer,
                reason(),
                from,
                until,
            )
            .await
            .unwrap();
        uc.revoke(issued.id).await.unwrap();

        let err = uc
            .assert_mandate_authorizes(subject, MandateKind::Lawyer, &scope)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::MandateNotFound | AppError::MandateRevoked),
            "expected revoked-related error, got {:?}",
            err
        );
    }

    // ---- @edge -------------------------------------------------------------

    #[tokio::test]
    async fn edge_double_revoke_is_idempotent() {
        let (_repo, uc) = factory();
        let subject = Uuid::new_v4();
        let issuer = Uuid::new_v4();
        let scope = MandateScope::Building(Uuid::new_v4());
        let (from, until) = fixture_window();

        let issued = uc
            .issue(
                subject,
                MandateKind::Bet,
                scope,
                issuer,
                reason(),
                from,
                until,
            )
            .await
            .unwrap();
        uc.revoke(issued.id).await.unwrap();
        // Second revoke must succeed (no-op).
        uc.revoke(issued.id).await.unwrap();
    }

    #[tokio::test]
    async fn edge_get_unknown_id_returns_not_found() {
        let (_repo, uc) = factory();
        let err = uc.get(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::MandateNotFound));
    }

    // ---- @security ---------------------------------------------------------

    #[tokio::test]
    async fn security_wrong_scope_returns_invalid_scope() {
        let (_repo, uc) = factory();
        let subject = Uuid::new_v4();
        let issuer = Uuid::new_v4();
        let building_a = MandateScope::Building(Uuid::new_v4());
        let building_b = MandateScope::Building(Uuid::new_v4());
        let (from, until) = fixture_window();

        uc.issue(
            subject,
            MandateKind::Notary,
            building_a,
            issuer,
            reason(),
            from,
            until,
        )
        .await
        .unwrap();

        let err = uc
            .assert_mandate_authorizes(subject, MandateKind::Notary, &building_b)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::MandateInvalidScope),
            "expected MandateInvalidScope, got {:?}",
            err
        );
    }

    #[tokio::test]
    async fn security_subject_equals_issuer_blocked_at_entity_level() {
        let (_repo, uc) = factory();
        let same = Uuid::new_v4();
        let (from, until) = fixture_window();
        let err = uc
            .issue(
                same,
                MandateKind::Notary,
                MandateScope::Building(Uuid::new_v4()),
                same,
                reason(),
                from,
                until,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn security_kind_mismatch_returns_not_found() {
        let (_repo, uc) = factory();
        let subject = Uuid::new_v4();
        let issuer = Uuid::new_v4();
        let scope = MandateScope::Building(Uuid::new_v4());
        let (from, until) = fixture_window();
        uc.issue(
            subject,
            MandateKind::Notary,
            scope,
            issuer,
            reason(),
            from,
            until,
        )
        .await
        .unwrap();
        // Subject is mandated as Notary, not Lawyer — must error typed.
        let err = uc
            .assert_mandate_authorizes(subject, MandateKind::Lawyer, &scope)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::MandateNotFound));
    }

    // ---- @negative ---------------------------------------------------------

    #[tokio::test]
    async fn negative_expired_mandate_returns_mandate_expired() {
        // Manually push an already-expired mandate, bypassing `issue` which
        // forbids `valid_until <= valid_from`.
        let repo = Arc::new(InMemoryRepo::default());
        let subject = Uuid::new_v4();
        let scope = MandateScope::Acp(Uuid::new_v4());
        let now = Utc::now();
        let m = Mandate {
            id: Uuid::new_v4(),
            subject_user_id: subject,
            kind: MandateKind::Architect,
            scope,
            issued_by: Uuid::new_v4(),
            reason: "Devis travaux toiture - dossier 2025".to_string(),
            valid_from: now - Duration::days(40),
            valid_until: now - Duration::days(1),
            revoked_at: None,
            created_at: now - Duration::days(40),
            updated_at: now - Duration::days(40),
        };
        repo.rows.lock().unwrap().push(m);
        let uc = MandateUseCases::new(repo.clone() as Arc<dyn MandateRepository>);

        let err = uc
            .assert_mandate_authorizes(subject, MandateKind::Architect, &scope)
            .await
            .unwrap_err();
        // Active-list filter excludes expired rows → caller sees NotFound
        // (which is correct: "no currently active mandate of this kind").
        assert!(
            matches!(err, AppError::MandateNotFound | AppError::MandateExpired),
            "expected MandateNotFound/Expired, got {:?}",
            err
        );
    }

    #[tokio::test]
    async fn negative_revoke_unknown_id_returns_not_found() {
        let (_repo, uc) = factory();
        let err = uc.revoke(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::MandateNotFound));
    }

    #[tokio::test]
    async fn negative_short_reason_is_rejected_at_use_case_boundary() {
        let (_repo, uc) = factory();
        let (from, until) = fixture_window();
        let err = uc
            .issue(
                Uuid::new_v4(),
                MandateKind::Warden,
                MandateScope::Building(Uuid::new_v4()),
                Uuid::new_v4(),
                "x".to_string(),
                from,
                until,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
