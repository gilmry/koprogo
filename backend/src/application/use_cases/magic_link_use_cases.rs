//! Use cases for the MagicLink feature (Story 3.2 — FR6, INV-13, INV-17;
//! extended by issue #855 for notary access to états datés).
//!
//! Operations:
//! 1. [`MagicLinkUseCases::issue`] — a syndic issues a magic link bound to a
//!    `(scope_kind, scope_id)` + recipient. Returns the clear token ONCE.
//! 2. [`MagicLinkUseCases::validate_and_consume`] — the public `/c/{token}`
//!    endpoint hashes the incoming token, looks it up, validates it, and marks
//!    it consumed atomically. Returns the resolved [`MagicLink`] so the caller
//!    handler can fetch the underlying scope resource. Distinguishes error
//!    causes (invalid / expired / already consumed) — acceptable here because
//!    the caller already holds the token, so the distinction leaks nothing
//!    about a *guessable* identifier.
//! 3. [`MagicLinkUseCases::verify_token`] — for scopes bound to a
//!    non-secret, guessable identifier (issue #855: `EtatDate` behind a
//!    reference number), refuses **uniformly** regardless of cause, and
//!    checks the token's scope matches the resource actually being accessed.
//! 4. [`MagicLinkUseCases::revoke`] — the issuer invalidates a link before
//!    its natural expiry (issue #855: "révocable").
//!
//! Security highlights:
//! - Clear token is generated inside `MagicLink::issue` and returned to the
//!   handler. It is NEVER logged and NEVER re-fetched from DB.
//! - Lookup uses `find_by_token_hash(sha256(token))` — a forged token returns
//!   `None` → translated to `MagicLinkInvalid` (uniform with "unknown token"
//!   to defeat enumeration).
//! - Single-use enforced by `mark_consumed` (race-safe `UPDATE ... WHERE
//!   consumed_at IS NULL` at the repository layer) — but only invoked when
//!   `MagicLink::single_use` holds; re-readable scopes stay valid until TTL.

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
    /// Le lien se consomme-t-il à la première lecture ? Cf.
    /// `MagicLinkScopeKind::is_single_use` (issue #855).
    pub single_use: bool,
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
    ///
    /// `subject_user_id` is `None` for a recipient without a KoproGo account
    /// (e.g. a notary, issue #855) — in that case `recipient_label` MUST
    /// carry their identity (enforced by `MagicLink::issue`).
    pub async fn issue(
        &self,
        subject_user_id: Option<Uuid>,
        recipient_label: Option<String>,
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
        let (link, clear_token) = MagicLink::issue(
            subject_user_id,
            recipient_label,
            scope_kind,
            scope_id,
            issued_by,
            ttl,
        )?;

        self.repo.save(&link).await?;

        Ok(IssuedMagicLinkDto {
            id: link.id,
            token: clear_token,
            expires_at: link.expires_at,
            scope_kind: link.scope_kind,
            scope_id: link.scope_id,
            single_use: link.single_use,
        })
    }

    /// Validate a clear token and atomically consume it — **if** its scope is
    /// single-use. Used by the generic public `/c/{token}` endpoint, which
    /// resolves the scope only after lookup and may distinguish error causes
    /// in its response (the token itself, not a guessable identifier, is
    /// what the caller already holds).
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

        let mut consumed = link;
        if consumed.single_use {
            self.repo.mark_consumed(consumed.id).await?;
        }
        consumed.consume();
        Ok(consumed)
    }

    /// Vérifie un jeton pour une portée et une ressource précises, sans
    /// jamais distinguer au client la cause du refus.
    ///
    /// Jeton inconnu, expiré, révoqué/consommé, ou lié à une AUTRE ressource
    /// renvoient tous la même erreur `AppError::MagicLinkInvalid` avec le
    /// même message. C'est délibéré (issue #855, critère `@negative`) : dire
    /// « expiré » plutôt que « inconnu » confirmerait à l'appelant qu'une
    /// référence non secrète (un numéro d'état daté, par exemple) existe
    /// réellement — ce que la garantie de ce mécanisme doit précisément
    /// empêcher.
    ///
    /// Ne consomme le lien que si sa portée est à usage unique
    /// (`MagicLink::single_use`) — un état daté reste lisible jusqu'à
    /// expiration ou révocation explicite, pour la relecture pendant
    /// l'instruction d'une vente (issue #855, sous-question 1).
    pub async fn verify_token(
        &self,
        clear_token: &str,
        expected_scope_kind: MagicLinkScopeKind,
        expected_scope_id: Uuid,
    ) -> Result<MagicLink, AppError> {
        let refuse = || AppError::MagicLinkInvalid;

        if clear_token.trim().is_empty() {
            return Err(refuse());
        }

        let token_hash = MagicLink::hash_token(clear_token);
        let mut link = self
            .repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or_else(refuse)?;

        if link.scope_kind != expected_scope_kind
            || link.scope_id != expected_scope_id
            || !link.is_valid()
        {
            return Err(refuse());
        }

        if link.single_use {
            // Seul un état de jeton (déjà consommé — course perdue contre un
            // appel concurrent) est refusé uniformément. Une vraie panne
            // (`AppError::Database`) doit remonter telle quelle : elle ne dit
            // rien sur l'existence de la ressource, la masquer en 403 ferait
            // passer une panne d'infrastructure pour un refus délibéré et
            // égarerait qui diagnostique l'incident (revue rust-expert,
            // issue #855).
            match self.repo.mark_consumed(link.id).await {
                Ok(()) => {}
                Err(AppError::MagicLinkAlreadyConsumed) => return Err(refuse()),
                Err(other) => return Err(other),
            }
            // Même comportement que `validate_and_consume` : l'entité rendue
            // à l'appelant reflète l'état qui vient d'être persisté.
            link.consume();
        }

        Ok(link)
    }

    /// Révoque un lien avant son terme naturel (issue #855 : « révocable » —
    /// le syndic qui l'a émis change d'avis, ou la vente n'aboutit pas).
    ///
    /// Réutilise `mark_consumed` plutôt qu'un champ `revoked_at` dédié : un
    /// lien consommé, qu'il le soit par lecture (usage unique) ou par
    /// révocation explicite, n'est de toute façon plus valide — `is_valid()`
    /// ne distingue pas les deux et n'a pas à le faire.
    ///
    /// L'autorisation (seul l'émetteur peut révoquer son propre lien) est une
    /// décision de la couche appelante (handler), pas de ce use case.
    pub async fn revoke(&self, link_id: Uuid) -> Result<(), AppError> {
        self.repo.mark_consumed(link_id).await
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
                Some(subject),
                None,
                MagicLinkScopeKind::Ticket,
                scope_id,
                issuer,
                7 * 24 * 3600,
            )
            .await
            .unwrap();

        assert_eq!(issued.scope_kind, MagicLinkScopeKind::Ticket);
        assert_eq!(issued.scope_id, scope_id);
        assert!(issued.single_use);
        assert!(!issued.token.is_empty());

        let resolved = uc.validate_and_consume(&issued.token).await.unwrap();
        assert_eq!(resolved.scope_id, scope_id);
        assert!(resolved.is_consumed());
    }

    /// @happy — un notaire relit un état daté sans jamais consommer le lien,
    /// et le journal peut nommer l'émetteur (issue #855).
    #[tokio::test]
    async fn happy_verify_token_etat_date_is_rereadable_without_consuming() {
        let (_repo, uc) = use_cases();
        let issuer = Uuid::new_v4();
        let etat_date_id = Uuid::new_v4();

        let issued = uc
            .issue(
                None,
                Some("Me Dupont <dupont@notaire.be>".to_string()),
                MagicLinkScopeKind::EtatDate,
                etat_date_id,
                issuer,
                3600,
            )
            .await
            .unwrap();
        assert!(!issued.single_use);

        let first = uc
            .verify_token(&issued.token, MagicLinkScopeKind::EtatDate, etat_date_id)
            .await
            .unwrap();
        assert!(!first.is_consumed());
        assert_eq!(first.issued_by, issuer);

        // Relecture : toujours valide, pas d'erreur "déjà consommé".
        let second = uc
            .verify_token(&issued.token, MagicLinkScopeKind::EtatDate, etat_date_id)
            .await
            .unwrap();
        assert_eq!(second.id, first.id);
    }

    /// @happy — révoquer un lien re-lisible avant son terme le rend refusé.
    #[tokio::test]
    async fn happy_revoke_invalidates_a_rereadable_link_before_its_term() {
        let (_repo, uc) = use_cases();
        let issuer = Uuid::new_v4();
        let etat_date_id = Uuid::new_v4();

        let issued = uc
            .issue(
                None,
                Some("Me Dupont".to_string()),
                MagicLinkScopeKind::EtatDate,
                etat_date_id,
                issuer,
                3600,
            )
            .await
            .unwrap();

        uc.revoke(issued.id).await.unwrap();

        let err = uc
            .verify_token(&issued.token, MagicLinkScopeKind::EtatDate, etat_date_id)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::MagicLinkInvalid));
    }

    // ---- @edge -------------------------------------------------------------

    #[tokio::test]
    async fn edge_double_consume_returns_already_consumed() {
        let (_repo, uc) = use_cases();
        let subject = Uuid::new_v4();
        let issuer = Uuid::new_v4();
        let scope_id = Uuid::new_v4();

        let issued = uc
            .issue(
                Some(subject),
                None,
                MagicLinkScopeKind::Quote,
                scope_id,
                issuer,
                3600,
            )
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
                Some(Uuid::new_v4()),
                None,
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
                Some(Uuid::new_v4()),
                None,
                MagicLinkScopeKind::Invoice,
                Uuid::new_v4(),
                Uuid::new_v4(),
                MAX_TTL_SECONDS + 1,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    /// @edge — un lien émis pour l'état daté A employé sur la référence B est
    /// refusé : la portée lie à UNE ressource (issue #855, critère `@edge`).
    #[tokio::test]
    async fn edge_token_issued_for_one_resource_rejected_on_another() {
        let (_repo, uc) = use_cases();
        let issuer = Uuid::new_v4();
        let etat_date_a = Uuid::new_v4();
        let etat_date_b = Uuid::new_v4();

        let issued = uc
            .issue(
                None,
                Some("Me Dupont".to_string()),
                MagicLinkScopeKind::EtatDate,
                etat_date_a,
                issuer,
                3600,
            )
            .await
            .unwrap();

        let err = uc
            .verify_token(&issued.token, MagicLinkScopeKind::EtatDate, etat_date_b)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::MagicLinkInvalid));
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
            .issue(
                Some(same),
                None,
                MagicLinkScopeKind::Ticket,
                Uuid::new_v4(),
                same,
                3600,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    /// @security — le refus de `verify_token` ne distingue jamais un jeton
    /// inconnu d'un jeton expiré : mêmes variante et message (issue #855,
    /// critère `@negative` — un message différent confirmerait l'existence
    /// de la ressource à un appelant qui n'a pas le bon jeton).
    #[tokio::test]
    async fn security_verify_token_never_distinguishes_failure_reasons() {
        let (repo, uc) = use_cases();
        let issuer = Uuid::new_v4();
        let etat_date_id = Uuid::new_v4();

        let err_unknown = uc
            .verify_token("forged-token", MagicLinkScopeKind::EtatDate, etat_date_id)
            .await
            .unwrap_err();

        let (mut link, clear) = MagicLink::issue(
            None,
            Some("Me Dupont".to_string()),
            MagicLinkScopeKind::EtatDate,
            etat_date_id,
            issuer,
            Duration::hours(1),
        )
        .unwrap();
        link.expires_at = Utc::now() - Duration::seconds(10);
        repo.rows.lock().unwrap().push(link);

        let err_expired = uc
            .verify_token(&clear, MagicLinkScopeKind::EtatDate, etat_date_id)
            .await
            .unwrap_err();

        assert!(matches!(err_unknown, AppError::MagicLinkInvalid));
        assert!(matches!(err_expired, AppError::MagicLinkInvalid));
        assert_eq!(err_unknown.to_string(), err_expired.to_string());
    }

    // ---- @negative ---------------------------------------------------------

    #[tokio::test]
    async fn negative_expired_link_returns_magic_link_expired() {
        let repo = Arc::new(InMemoryRepo::default());

        // Manually push an already-expired link bypassing the use case (since
        // the issue path forbids negative TTL).
        let (mut link, clear) = MagicLink::issue(
            Some(Uuid::new_v4()),
            None,
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
            Some(Uuid::new_v4()),
            None,
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

    /// @negative — un jeton valide pour une AUTRE portée (Ticket) n'ouvre pas
    /// un état daté, sans distinguer la cause (issue #855).
    #[tokio::test]
    async fn negative_verify_token_rejects_wrong_scope_kind() {
        let (_repo, uc) = use_cases();
        let issuer = Uuid::new_v4();
        let scope_id = Uuid::new_v4();

        let issued = uc
            .issue(
                Some(Uuid::new_v4()),
                None,
                MagicLinkScopeKind::Ticket,
                scope_id,
                issuer,
                3600,
            )
            .await
            .unwrap();

        let err = uc
            .verify_token(&issued.token, MagicLinkScopeKind::EtatDate, scope_id)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::MagicLinkInvalid));
    }

    /// @negative — une vraie panne d'infrastructure ne doit pas se travestir
    /// en refus « jeton invalide » : la garantie d'uniformité couvre les
    /// verdicts sur le jeton (inconnu / expiré / hors-portée), pas les
    /// pannes, qui ne disent rien sur l'existence de la ressource et doivent
    /// remonter telles quelles pour être diagnosticables (revue rust-expert,
    /// issue #855).
    #[tokio::test]
    async fn negative_verify_token_propagates_database_errors_untouched() {
        struct FailingMarkConsumedRepo {
            link: MagicLink,
        }

        #[async_trait]
        impl MagicLinkRepository for FailingMarkConsumedRepo {
            async fn save(&self, _link: &MagicLink) -> Result<(), AppError> {
                Ok(())
            }
            async fn find_by_token_hash(
                &self,
                token_hash: &str,
            ) -> Result<Option<MagicLink>, AppError> {
                if token_hash == self.link.token_hash {
                    Ok(Some(self.link.clone()))
                } else {
                    Ok(None)
                }
            }
            async fn mark_consumed(&self, _id: Uuid) -> Result<(), AppError> {
                Err(AppError::Database("connection pool exhausted".to_string()))
            }
        }

        let issuer = Uuid::new_v4();
        let scope_id = Uuid::new_v4();
        let (link, clear) = MagicLink::issue(
            Some(Uuid::new_v4()),
            None,
            MagicLinkScopeKind::Ticket,
            scope_id,
            issuer,
            Duration::hours(1),
        )
        .unwrap();

        let repo = Arc::new(FailingMarkConsumedRepo { link });
        let uc = MagicLinkUseCases::new(repo as Arc<dyn MagicLinkRepository>);

        let err = uc
            .verify_token(&clear, MagicLinkScopeKind::Ticket, scope_id)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Database(_)),
            "une panne DB doit remonter telle quelle, pas se travestir en MagicLinkInvalid"
        );
    }
}
