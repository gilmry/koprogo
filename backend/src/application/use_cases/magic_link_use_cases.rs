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

    /// Resolve a token WITHOUT consuming it — for scopes whose workflow spans
    /// several round-trips after the first `GET /c/{token}` (#835 @edge).
    ///
    /// `ContractorReport` is the first such scope: a prestataire opens the
    /// link (which consumes it via [`Self::validate_and_consume`] as an audit
    /// marker of "first redemption"), then edits a draft and submits later —
    /// possibly offline, possibly the next day. Gating that later write on
    /// `consumed_at` would make the very first view burn the only chance to
    /// ever submit, which is incompatible with the offline requirement this
    /// scope must keep (cf. system B being absorbed, which only ever checked
    /// TTL). So `peek` checks hash lookup + expiry only, and deliberately
    /// ignores `consumed_at`. Scope cloisonnement is enforced separately by
    /// [`Self::ensure_scope`] at the call site — a token's `scope_kind` is
    /// fixed at issuance and never reinterpreted.
    pub async fn peek(&self, clear_token: &str) -> Result<MagicLink, AppError> {
        if clear_token.trim().is_empty() {
            return Err(AppError::MagicLinkInvalid);
        }

        let token_hash = MagicLink::hash_token(clear_token);
        let link = self
            .repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AppError::MagicLinkInvalid)?;

        if link.is_expired() {
            return Err(AppError::MagicLinkExpired);
        }

        Ok(link)
    }

    /// Enforce that a resolved link matches the scope the caller expects.
    ///
    /// Returns the same uniform `MagicLinkInvalid` as an unknown token — a
    /// link issued for `Quote` must not distinguishably fail as "wrong scope"
    /// (anti-enumeration, same rationale as `validate_and_consume`'s uniform
    /// "unknown token" error), and must not open a `ContractorReport` (#835
    /// @security — élargir un scope est le moyen le plus simple de
    /// transformer un lien ciblé en passe-partout).
    pub fn ensure_scope(link: &MagicLink, expected: MagicLinkScopeKind) -> Result<(), AppError> {
        if link.scope_kind != expected {
            return Err(AppError::MagicLinkInvalid);
        }
        Ok(())
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

    // ---- @happy (peek / ensure_scope — #835) --------------------------------

    #[tokio::test]
    async fn happy_peek_resolves_valid_token_without_consuming() {
        let (repo, uc) = use_cases();
        let issued = uc
            .issue(
                Uuid::new_v4(),
                MagicLinkScopeKind::ContractorReport,
                Uuid::new_v4(),
                Uuid::new_v4(),
                3600,
            )
            .await
            .unwrap();

        let peeked = uc.peek(&issued.token).await.unwrap();
        assert!(!peeked.is_consumed());

        // Peeking again must still work — unlike validate_and_consume, it is
        // repeatable (cf. #835 @edge — offline draft round-trips).
        let peeked_again = uc.peek(&issued.token).await.unwrap();
        assert!(!peeked_again.is_consumed());
        assert_eq!(
            repo.rows.lock().unwrap().len(),
            1,
            "peek must not create/consume rows"
        );
    }

    #[tokio::test]
    async fn happy_peek_still_resolves_after_the_link_was_consumed_elsewhere() {
        // The initial GET /c/{token} DOES consume the link (audit marker of
        // first redemption). A later write action (submit) must still be able
        // to `peek` the same token — this is the crux of #835 @edge.
        let (_repo, uc) = use_cases();
        let issued = uc
            .issue(
                Uuid::new_v4(),
                MagicLinkScopeKind::ContractorReport,
                Uuid::new_v4(),
                Uuid::new_v4(),
                3600,
            )
            .await
            .unwrap();

        uc.validate_and_consume(&issued.token).await.unwrap();

        let peeked = uc.peek(&issued.token).await.unwrap();
        assert!(
            peeked.is_consumed(),
            "consumed flag is preserved, informational only"
        );
    }

    #[tokio::test]
    async fn happy_ensure_scope_accepts_matching_kind() {
        let (subject, issuer, scope) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let (link, _) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::ContractorReport,
            scope,
            issuer,
            Duration::hours(1),
        )
        .unwrap();
        assert!(
            MagicLinkUseCases::ensure_scope(&link, MagicLinkScopeKind::ContractorReport).is_ok()
        );
    }

    // ---- @edge (peek — #835) ------------------------------------------------

    #[tokio::test]
    async fn edge_peek_at_exact_expiry_boundary_matches_validate_and_consume() {
        let repo = Arc::new(InMemoryRepo::default());
        let (mut link, clear) = MagicLink::issue(
            Uuid::new_v4(),
            MagicLinkScopeKind::ContractorReport,
            Uuid::new_v4(),
            Uuid::new_v4(),
            Duration::hours(1),
        )
        .unwrap();
        link.expires_at = Utc::now() - Duration::seconds(1);
        repo.rows.lock().unwrap().push(link);

        let uc = MagicLinkUseCases::new(repo as Arc<dyn MagicLinkRepository>);
        let err = uc.peek(&clear).await.unwrap_err();
        assert!(matches!(err, AppError::MagicLinkExpired));
    }

    // ---- @security (peek / ensure_scope — #835) -----------------------------

    #[tokio::test]
    async fn security_peek_forged_token_returns_invalid() {
        let (_repo, uc) = use_cases();
        let err = uc.peek("forged-not-in-db").await.unwrap_err();
        assert!(matches!(err, AppError::MagicLinkInvalid));
    }

    #[test]
    fn security_ensure_scope_rejects_mismatched_kind_uniformly() {
        let (subject, issuer, scope) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let (link, _) = MagicLink::issue(
            subject,
            MagicLinkScopeKind::Quote,
            scope,
            issuer,
            Duration::hours(1),
        )
        .unwrap();
        let err = MagicLinkUseCases::ensure_scope(&link, MagicLinkScopeKind::ContractorReport)
            .unwrap_err();
        // Uniform with "unknown token" — never a distinguishable "wrong scope"
        // error that would help an attacker probe which scope a token holds.
        assert!(matches!(err, AppError::MagicLinkInvalid));
    }

    // ---- @negative (peek — #835) ---------------------------------------------

    #[tokio::test]
    async fn negative_peek_empty_token_returns_invalid_without_db_lookup() {
        let (_repo, uc) = use_cases();
        let err = uc.peek("   ").await.unwrap_err();
        assert!(matches!(err, AppError::MagicLinkInvalid));
    }

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
