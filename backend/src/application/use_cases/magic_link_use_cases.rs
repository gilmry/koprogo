//! Use cases for the MagicLink feature (Story 3.2 — FR6, INV-13, INV-17).
//!
//! Two operations:
//! 1. [`MagicLinkUseCases::issue`] — a syndic issues a magic link bound to a
//!    `(scope_kind, scope_id)` + recipient user. Returns the clear token ONCE.
//! 2. [`MagicLinkUseCases::validate_and_consume`] — the public `/c/{token}`
//!    endpoint hashes the incoming token, looks it up, validates it, and marks
//!    it consumed atomically. Returns the resolved [`MagicLink`] so the caller
//!    handler can fetch the underlying scope resource.
//!
//! Security highlights:
//! - Clear token is generated inside `MagicLink::issue` and returned to the
//!   handler. It is NEVER logged and NEVER re-fetched from DB.
//! - Lookup uses `find_by_token_hash(sha256(token))` — a forged token returns
//!   `None` → translated to `MagicLinkInvalid` (uniform with "unknown token"
//!   to defeat enumeration).
//! - Single-use enforced by `mark_consumed` (race-safe `UPDATE ... WHERE
//!   consumed_at IS NULL` at the repository layer).

use crate::application::error::AppError;
use crate::application::ports::MagicLinkRepository;
use crate::domain::entities::{MagicLink, MagicLinkScopeKind};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Bounds for a MagicLink TTL (seconds). Smaller values are easy to misuse
/// (expired before SMS arrives); larger values weaken the security model.
const MIN_TTL_SECONDS: i64 = 60; // 1 minute
const MAX_TTL_SECONDS: i64 = 60 * 60 * 24 * 30; // 30 days

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedMagicLinkDto {
    pub id: Uuid,
    /// Clear token — return to the client ONCE, never persist elsewhere.
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub scope_kind: MagicLinkScopeKind,
    pub scope_id: Uuid,
}

pub struct MagicLinkUseCases {
    repo: Arc<dyn MagicLinkRepository>,
}

impl MagicLinkUseCases {
    pub fn new(repo: Arc<dyn MagicLinkRepository>) -> Self {
        Self { repo }
    }

    /// Issue a new MagicLink. Caller MUST have already authorised the request
    /// (syndic / superadmin role check happens at the handler level).
    pub async fn issue(
        &self,
        subject_user_id: Uuid,
        scope_kind: MagicLinkScopeKind,
        scope_id: Uuid,
        issued_by: Uuid,
        expires_in_seconds: i64,
    ) -> Result<IssuedMagicLinkDto, AppError> {
        if !(MIN_TTL_SECONDS..=MAX_TTL_SECONDS).contains(&expires_in_seconds) {
            return Err(AppError::Validation(format!(
                "expires_in_seconds must be in [{}, {}], got {}",
                MIN_TTL_SECONDS, MAX_TTL_SECONDS, expires_in_seconds
            )));
        }

        let ttl = Duration::seconds(expires_in_seconds);
        let (link, clear_token) =
            MagicLink::issue(subject_user_id, scope_kind, scope_id, issued_by, ttl)?;

        self.repo.save(&link).await?;

        Ok(IssuedMagicLinkDto {
            id: link.id,
            token: clear_token,
            expires_at: link.expires_at,
            scope_kind: link.scope_kind,
            scope_id: link.scope_id,
        })
    }

    /// Validate a clear token and atomically consume it.
    ///
    /// Possible errors (all map to HTTP 403 by design — see CRITICAL.md #4 and
    /// the AppError mapping in error.rs):
    /// - `AppError::MagicLinkInvalid` — token not found (forged / unknown).
    /// - `AppError::MagicLinkExpired` — TTL elapsed.
    /// - `AppError::MagicLinkAlreadyConsumed` — replay attempt on used token.
    pub async fn validate_and_consume(&self, clear_token: &str) -> Result<MagicLink, AppError> {
        if clear_token.trim().is_empty() {
            return Err(AppError::MagicLinkInvalid);
        }

        let token_hash = MagicLink::hash_token(clear_token);
        let link = self
            .repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AppError::MagicLinkInvalid)?;

        if link.is_consumed() {
            return Err(AppError::MagicLinkAlreadyConsumed);
        }
        if link.is_expired() {
            return Err(AppError::MagicLinkExpired);
        }

        self.repo.mark_consumed(link.id).await?;

        let mut consumed = link;
        consumed.consume();
        Ok(consumed)
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories (CRITICAL.md #3)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// In-memory MagicLinkRepository mock for use-case unit tests.
    #[derive(Default)]
    struct InMemoryRepo {
        rows: Mutex<Vec<MagicLink>>,
    }

    #[async_trait]
    impl MagicLinkRepository for InMemoryRepo {
        async fn save(&self, link: &MagicLink) -> Result<(), AppError> {
            self.rows.lock().unwrap().push(link.clone());
            Ok(())
        }

        async fn find_by_token_hash(
            &self,
            token_hash: &str,
        ) -> Result<Option<MagicLink>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|l| l.token_hash == token_hash)
                .cloned())
        }

        async fn mark_consumed(&self, id: Uuid) -> Result<(), AppError> {
            let mut rows = self.rows.lock().unwrap();
            if let Some(row) = rows.iter_mut().find(|l| l.id == id) {
                if row.consumed_at.is_some() {
                    return Err(AppError::MagicLinkAlreadyConsumed);
                }
                row.consumed_at = Some(Utc::now());
                row.updated_at = Utc::now();
            }
            Ok(())
        }
    }

    fn use_cases() -> (Arc<InMemoryRepo>, MagicLinkUseCases) {
        let repo: Arc<InMemoryRepo> = Arc::new(InMemoryRepo::default());
        let uc = MagicLinkUseCases::new(repo.clone() as Arc<dyn MagicLinkRepository>);
        (repo, uc)
    }

    // ---- @happy ------------------------------------------------------------

    #[tokio::test]
    async fn happy_issue_then_validate_consumes_once() {
        let (_repo, uc) = use_cases();
        let subject = Uuid::new_v4();
        let issuer = Uuid::new_v4();
        let scope_id = Uuid::new_v4();

        let issued = uc
            .issue(
                subject,
                MagicLinkScopeKind::Ticket,
                scope_id,
                issuer,
                7 * 24 * 3600,
            )
            .await
            .unwrap();

        assert_eq!(issued.scope_kind, MagicLinkScopeKind::Ticket);
        assert_eq!(issued.scope_id, scope_id);
        assert!(!issued.token.is_empty());

        let resolved = uc.validate_and_consume(&issued.token).await.unwrap();
        assert_eq!(resolved.scope_id, scope_id);
        assert!(resolved.is_consumed());
    }

    // ---- @edge -------------------------------------------------------------

    #[tokio::test]
    async fn edge_double_consume_returns_already_consumed() {
        let (_repo, uc) = use_cases();
        let subject = Uuid::new_v4();
        let issuer = Uuid::new_v4();
        let scope_id = Uuid::new_v4();

        let issued = uc
            .issue(subject, MagicLinkScopeKind::Quote, scope_id, issuer, 3600)
            .await
            .unwrap();

        uc.validate_and_consume(&issued.token).await.unwrap();
        let err = uc.validate_and_consume(&issued.token).await.unwrap_err();
        assert!(matches!(err, AppError::MagicLinkAlreadyConsumed));
    }

    #[tokio::test]
    async fn edge_ttl_below_min_is_rejected() {
        let (_repo, uc) = use_cases();
        let err = uc
            .issue(
                Uuid::new_v4(),
                MagicLinkScopeKind::Invoice,
                Uuid::new_v4(),
                Uuid::new_v4(),
                30, // below MIN_TTL_SECONDS=60
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn edge_ttl_above_max_is_rejected() {
        let (_repo, uc) = use_cases();
        let err = uc
            .issue(
                Uuid::new_v4(),
                MagicLinkScopeKind::Invoice,
                Uuid::new_v4(),
                Uuid::new_v4(),
                MAX_TTL_SECONDS + 1,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ---- @security ---------------------------------------------------------

    #[tokio::test]
    async fn security_forged_token_returns_invalid() {
        let (_repo, uc) = use_cases();
        let err = uc
            .validate_and_consume("forged-not-in-db")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::MagicLinkInvalid));
    }

    #[tokio::test]
    async fn security_empty_token_returns_invalid_without_db_lookup() {
        let (_repo, uc) = use_cases();
        let err = uc.validate_and_consume("   ").await.unwrap_err();
        assert!(matches!(err, AppError::MagicLinkInvalid));
    }

    #[tokio::test]
    async fn security_subject_equals_issuer_is_blocked_at_entity_level() {
        let (_repo, uc) = use_cases();
        let same = Uuid::new_v4();
        let err = uc
            .issue(same, MagicLinkScopeKind::Ticket, Uuid::new_v4(), same, 3600)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ---- @negative ---------------------------------------------------------

    #[tokio::test]
    async fn negative_expired_link_returns_magic_link_expired() {
        let repo = Arc::new(InMemoryRepo::default());

        // Manually push an already-expired link bypassing the use case (since
        // the issue path forbids negative TTL).
        let (mut link, clear) = MagicLink::issue(
            Uuid::new_v4(),
            MagicLinkScopeKind::Ticket,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Duration::hours(1),
        )
        .unwrap();
        link.expires_at = Utc::now() - Duration::seconds(10);
        repo.rows.lock().unwrap().push(link);

        let uc = MagicLinkUseCases::new(repo.clone() as Arc<dyn MagicLinkRepository>);
        let err = uc.validate_and_consume(&clear).await.unwrap_err();
        assert!(matches!(err, AppError::MagicLinkExpired));
    }

    #[tokio::test]
    async fn negative_consumed_check_precedes_expired_check() {
        // If a link is both expired AND consumed, surface the "already consumed"
        // signal first — it's the more actionable message for the user.
        let repo = Arc::new(InMemoryRepo::default());
        let (mut link, clear) = MagicLink::issue(
            Uuid::new_v4(),
            MagicLinkScopeKind::Ticket,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Duration::hours(1),
        )
        .unwrap();
        link.consumed_at = Some(Utc::now() - Duration::minutes(10));
        link.expires_at = Utc::now() - Duration::seconds(10);
        repo.rows.lock().unwrap().push(link);

        let uc = MagicLinkUseCases::new(repo.clone() as Arc<dyn MagicLinkRepository>);
        let err = uc.validate_and_consume(&clear).await.unwrap_err();
        assert!(matches!(err, AppError::MagicLinkAlreadyConsumed));
    }
}
