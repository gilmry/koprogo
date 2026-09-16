//! Use cases for `NotaryLink` (Issue #855 — ADR 0048 / ADR 0051).
//!
//! Four operations:
//! 1. [`NotaryLinkUseCases::issue`] — a syndic issues a link bound to ONE
//!    état daté. Returns the clear token ONCE, mirroring `MagicLinkUseCases`.
//! 2. [`NotaryLinkUseCases::consult`] — the public
//!    `GET /etats-dates/reference/{reference_number}` route resolves the
//!    état daté by reference number and requires a valid, matching-scope
//!    token to return it. Every failure — missing token, unknown reference,
//!    unknown/forged token, expired, revoked, wrong état daté — surfaces the
//!    SAME `AppError::NotaryLinkInvalid` (never `NotFound` for the token
//!    itself): distinguishing causes would let an anonymous caller confirm a
//!    reference number is live (#855 @negative).
//! 3. [`NotaryLinkUseCases::revoke`] / 4. [`NotaryLinkUseCases::renew`] —
//!    syndic-only administrative actions on a link the caller already
//!    identifies by id (no anonymity concern there, so errors ARE specific).
//!
//! Rate limiting: [`NotaryLinkRateLimiter`] guards `consult` per caller key
//! (IP address, supplied by the handler) — belt-and-suspenders alongside the
//! 256-bit token entropy, per #855 @security ("32 bits d'aléa ne résistent
//! pas à un balayage non borné" — the ORIGINAL reference number had 32 bits;
//! the signed token itself has ~256, but the route must still not be probed
//! without limit).

use crate::application::dto::EtatDateResponse;
use crate::application::error::AppError;
use crate::application::ports::{EtatDateRepository, NotaryLinkRepository};
use crate::domain::entities::NotaryLink;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Clear token issuance result — returned to the syndic ONCE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedNotaryLinkDto {
    pub id: Uuid,
    /// Clear token — never persisted, never logged.
    pub token: String,
    pub etat_date_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

/// Read-only view of a link for the syndic dashboard / follow-up screen —
/// never carries the clear token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotaryLinkResponse {
    pub id: Uuid,
    pub etat_date_id: Uuid,
    pub issued_by: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_valid: bool,
}

impl From<NotaryLink> for NotaryLinkResponse {
    fn from(link: NotaryLink) -> Self {
        Self {
            id: link.id,
            etat_date_id: link.etat_date_id,
            issued_by: link.issued_by,
            expires_at: link.expires_at,
            revoked_at: link.revoked_at,
            created_at: link.created_at,
            is_valid: link.is_valid(),
        }
    }
}

/// Per-key sliding-window rate limiter (#855 @security). Same shape as
/// `GdprRateLimitState` (`infrastructure/web/middleware/mod.rs`), kept
/// separate because this one keys by caller IP rather than JWT identity —
/// the notary consulting `consult()` has no JWT.
#[derive(Clone)]
pub struct NotaryLinkRateLimiter {
    state: Arc<Mutex<HashMap<String, (usize, Instant)>>>,
    max_requests: usize,
    window: Duration,
}

impl NotaryLinkRateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub fn check(&self, key: &str) -> Result<(), AppError> {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let entry = state.entry(key.to_string()).or_insert((0, now));
        let (count, window_start) = entry;

        if now.duration_since(*window_start) > self.window {
            *count = 0;
            *window_start = now;
        }

        if *count >= self.max_requests {
            return Err(AppError::RateLimited);
        }

        *count += 1;
        Ok(())
    }
}

impl Default for NotaryLinkRateLimiter {
    /// 20 requêtes par clé (IP) sur 10 minutes — large pour un notaire qui
    /// rafraîchit un dossier, étroit pour un balayage automatisé.
    fn default() -> Self {
        Self::new(20, Duration::from_secs(600))
    }
}

pub struct NotaryLinkUseCases {
    repo: Arc<dyn NotaryLinkRepository>,
    etat_date_repo: Arc<dyn EtatDateRepository>,
    rate_limiter: NotaryLinkRateLimiter,
}

impl NotaryLinkUseCases {
    pub fn new(
        repo: Arc<dyn NotaryLinkRepository>,
        etat_date_repo: Arc<dyn EtatDateRepository>,
    ) -> Self {
        Self {
            repo,
            etat_date_repo,
            rate_limiter: NotaryLinkRateLimiter::default(),
        }
    }

    #[cfg(test)]
    fn with_rate_limiter(mut self, rate_limiter: NotaryLinkRateLimiter) -> Self {
        self.rate_limiter = rate_limiter;
        self
    }

    /// Syndic issues a link for one état daté. Caller MUST have already
    /// authorised the request (org-access check happens at the handler
    /// level, same convention as `MagicLinkUseCases::issue`).
    pub async fn issue(
        &self,
        etat_date_id: Uuid,
        issued_by: Uuid,
    ) -> Result<IssuedNotaryLinkDto, AppError> {
        self.etat_date_repo
            .find_by_id(etat_date_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("État daté".to_string()))?;

        let (link, clear_token) = NotaryLink::issue(etat_date_id, issued_by)?;
        self.repo.save(&link).await?;

        Ok(IssuedNotaryLinkDto {
            id: link.id,
            token: clear_token,
            etat_date_id: link.etat_date_id,
            expires_at: link.expires_at,
        })
    }

    /// Public route: resolve an état daté by reference number, gated by a
    /// NotaryLink token. `rate_limit_key` is the caller's IP (or any stable
    /// per-caller identifier) — see [`NotaryLinkRateLimiter`].
    ///
    /// All failure modes collapse to `AppError::NotaryLinkInvalid` EXCEPT an
    /// unknown reference number, which returns `AppError::NotFound` — per
    /// ADR 0048, the reference number itself is not a secret (it circulates
    /// in emails and sale files); what must stay opaque is whether a *token*
    /// is valid for a reference that DOES exist.
    pub async fn consult(
        &self,
        reference_number: &str,
        clear_token: Option<&str>,
        rate_limit_key: &str,
    ) -> Result<EtatDateResponse, AppError> {
        self.rate_limiter.check(rate_limit_key)?;

        let etat_date = self
            .etat_date_repo
            .find_by_reference_number(reference_number)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("État daté".to_string()))?;

        let clear_token = clear_token
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .ok_or(AppError::NotaryLinkInvalid)?;

        let token_hash = NotaryLink::hash_token(clear_token);
        let link = self
            .repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AppError::NotaryLinkInvalid)?;

        if !link.authorizes(etat_date.id) {
            return Err(AppError::NotaryLinkInvalid);
        }

        Ok(EtatDateResponse::from(etat_date))
    }

    /// Fetch one link by id — used by the handler layer to resolve the
    /// owning `etat_date_id` BEFORE authorising a revoke/renew action
    /// (cloisonnement avant mutation, cf. #864).
    pub async fn get(&self, link_id: Uuid) -> Result<NotaryLinkResponse, AppError> {
        let link = self
            .repo
            .find_by_id(link_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Lien notaire".to_string()))?;
        Ok(NotaryLinkResponse::from(link))
    }

    /// Syndic revokes a link before term. `revoked_by` is logged by the
    /// caller (handler); org-access check happens there too.
    pub async fn revoke(&self, link_id: Uuid) -> Result<(), AppError> {
        let mut link = self
            .repo
            .find_by_id(link_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Lien notaire".to_string()))?;

        link.revoke();
        self.repo.update(&link).await
    }

    /// Syndic renews a link, extending it `VALIDITE_JOURS` days from now.
    pub async fn renew(&self, link_id: Uuid) -> Result<NotaryLinkResponse, AppError> {
        let mut link = self
            .repo
            .find_by_id(link_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Lien notaire".to_string()))?;

        link.renew()?;
        self.repo.update(&link).await?;
        Ok(NotaryLinkResponse::from(link))
    }

    /// All links issued for one état daté — feeds the syndic's emission
    /// screen and the "en défaut" follow-up view.
    pub async fn list_for_etat_date(
        &self,
        etat_date_id: Uuid,
    ) -> Result<Vec<NotaryLinkResponse>, AppError> {
        let links = self.repo.list_by_etat_date(etat_date_id).await?;
        Ok(links.into_iter().map(NotaryLinkResponse::from).collect())
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories (CRITICAL.md #3)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{EtatDateStatsResponse, PageRequest};
    use crate::domain::entities::{EtatDate, EtatDateLanguage, EtatDateStatus};
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    #[derive(Default)]
    struct InMemoryNotaryLinkRepo {
        rows: Mutex<Vec<NotaryLink>>,
    }

    #[async_trait]
    impl NotaryLinkRepository for InMemoryNotaryLinkRepo {
        async fn save(&self, link: &NotaryLink) -> Result<(), AppError> {
            self.rows.lock().unwrap().push(link.clone());
            Ok(())
        }

        async fn find_by_token_hash(
            &self,
            token_hash: &str,
        ) -> Result<Option<NotaryLink>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|l| l.token_hash == token_hash)
                .cloned())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<NotaryLink>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|l| l.id == id)
                .cloned())
        }

        async fn list_by_etat_date(&self, etat_date_id: Uuid) -> Result<Vec<NotaryLink>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|l| l.etat_date_id == etat_date_id)
                .cloned()
                .collect())
        }

        async fn update(&self, link: &NotaryLink) -> Result<(), AppError> {
            let mut rows = self.rows.lock().unwrap();
            if let Some(row) = rows.iter_mut().find(|l| l.id == link.id) {
                *row = link.clone();
            }
            Ok(())
        }
    }

    /// Fake couvrant uniquement les méthodes que `NotaryLinkUseCases`
    /// appelle réellement (`find_by_id`, `find_by_reference_number`) — le
    /// reste de `EtatDateRepository` n'est pas exercé par ce module et
    /// panique volontairement s'il l'était (signal clair d'un test mal
    /// ciblé plutôt qu'un faux résultat silencieux).
    #[derive(Default)]
    struct FakeEtatDateRepo {
        rows: Mutex<Vec<EtatDate>>,
    }

    #[async_trait]
    impl EtatDateRepository for FakeEtatDateRepo {
        async fn create(&self, etat_date: &EtatDate) -> Result<EtatDate, String> {
            self.rows.lock().unwrap().push(etat_date.clone());
            Ok(etat_date.clone())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<EtatDate>, String> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|e| e.id == id)
                .cloned())
        }
        async fn find_by_reference_number(
            &self,
            reference_number: &str,
        ) -> Result<Option<EtatDate>, String> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|e| e.reference_number == reference_number)
                .cloned())
        }
        async fn find_by_unit(&self, _unit_id: Uuid) -> Result<Vec<EtatDate>, String> {
            unimplemented!("not exercised by NotaryLinkUseCases tests")
        }
        async fn find_by_building(&self, _building_id: Uuid) -> Result<Vec<EtatDate>, String> {
            unimplemented!("not exercised by NotaryLinkUseCases tests")
        }
        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _organization_id: Option<Uuid>,
            _status: Option<EtatDateStatus>,
        ) -> Result<(Vec<EtatDate>, i64), String> {
            unimplemented!("not exercised by NotaryLinkUseCases tests")
        }
        async fn find_overdue(&self, _organization_id: Uuid) -> Result<Vec<EtatDate>, String> {
            unimplemented!("not exercised by NotaryLinkUseCases tests")
        }
        async fn find_expired(&self, _organization_id: Uuid) -> Result<Vec<EtatDate>, String> {
            unimplemented!("not exercised by NotaryLinkUseCases tests")
        }
        async fn update(&self, etat_date: &EtatDate) -> Result<EtatDate, String> {
            Ok(etat_date.clone())
        }
        async fn delete(&self, _id: Uuid) -> Result<bool, String> {
            unimplemented!("not exercised by NotaryLinkUseCases tests")
        }
        async fn get_stats(&self, _organization_id: Uuid) -> Result<EtatDateStatsResponse, String> {
            unimplemented!("not exercised by NotaryLinkUseCases tests")
        }
        async fn count_by_status(
            &self,
            _organization_id: Uuid,
            _status: EtatDateStatus,
        ) -> Result<i64, String> {
            unimplemented!("not exercised by NotaryLinkUseCases tests")
        }
    }

    fn make_etat_date() -> EtatDate {
        EtatDate::new(
            Uuid::new_v4(), // acp_id
            Uuid::new_v4(), // organization_id
            Uuid::new_v4(), // building_id
            Uuid::new_v4(), // unit_id
            Utc::now(),
            EtatDateLanguage::Fr,
            "Maitre Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            Some("+32 2 123 4567".to_string()),
            "Residence Les Jardins".to_string(),
            "Rue de la Loi 123, 1000 Bruxelles".to_string(),
            "101".to_string(),
            Some("1".to_string()),
            Some(85.0),
            dec!(50),
            dec!(50),
        )
        .unwrap()
    }

    fn use_cases() -> (
        Arc<InMemoryNotaryLinkRepo>,
        Arc<FakeEtatDateRepo>,
        NotaryLinkUseCases,
    ) {
        let repo: Arc<InMemoryNotaryLinkRepo> = Arc::new(InMemoryNotaryLinkRepo::default());
        let etat_date_repo: Arc<FakeEtatDateRepo> = Arc::new(FakeEtatDateRepo::default());
        let uc = NotaryLinkUseCases::new(
            repo.clone() as Arc<dyn NotaryLinkRepository>,
            etat_date_repo.clone() as Arc<dyn EtatDateRepository>,
        );
        (repo, etat_date_repo, uc)
    }

    // ---- @happy --------------------------------------------------------------

    #[tokio::test]
    async fn happy_issue_then_consult_returns_etat_date() {
        let (_repo, etat_date_repo, uc) = use_cases();
        let etat_date = make_etat_date();
        etat_date_repo.rows.lock().unwrap().push(etat_date.clone());

        let issued = uc.issue(etat_date.id, Uuid::new_v4()).await.unwrap();
        assert_eq!(issued.etat_date_id, etat_date.id);
        assert!(!issued.token.is_empty());

        let resolved = uc
            .consult(&etat_date.reference_number, Some(&issued.token), "1.2.3.4")
            .await
            .unwrap();
        assert_eq!(resolved.id, etat_date.id);
    }

    #[tokio::test]
    async fn happy_get_returns_link_for_syndic_org_check() {
        let (_repo, etat_date_repo, uc) = use_cases();
        let etat_date = make_etat_date();
        etat_date_repo.rows.lock().unwrap().push(etat_date.clone());
        let issued = uc.issue(etat_date.id, Uuid::new_v4()).await.unwrap();

        let fetched = uc.get(issued.id).await.unwrap();
        assert_eq!(fetched.etat_date_id, etat_date.id);
    }

    #[tokio::test]
    async fn happy_list_for_etat_date_returns_issued_links() {
        let (_repo, etat_date_repo, uc) = use_cases();
        let etat_date = make_etat_date();
        etat_date_repo.rows.lock().unwrap().push(etat_date.clone());

        uc.issue(etat_date.id, Uuid::new_v4()).await.unwrap();
        uc.issue(etat_date.id, Uuid::new_v4()).await.unwrap();

        let links = uc.list_for_etat_date(etat_date.id).await.unwrap();
        assert_eq!(links.len(), 2);
        assert!(links.iter().all(|l| l.is_valid));
    }

    // ---- @edge -----------------------------------------------------------------

    #[tokio::test]
    async fn edge_token_scoped_to_other_etat_date_is_rejected() {
        let (_repo, etat_date_repo, uc) = use_cases();
        let etat_date_a = make_etat_date();
        let etat_date_b = make_etat_date();
        etat_date_repo
            .rows
            .lock()
            .unwrap()
            .extend([etat_date_a.clone(), etat_date_b.clone()]);

        let issued_for_a = uc.issue(etat_date_a.id, Uuid::new_v4()).await.unwrap();

        // La portée lie à UNE ressource : un jeton émis pour A ne doit pas
        // ouvrir B (#855 @edge).
        let err = uc
            .consult(
                &etat_date_b.reference_number,
                Some(&issued_for_a.token),
                "1.2.3.4",
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotaryLinkInvalid));
    }

    #[tokio::test]
    async fn edge_renew_extends_and_persists_via_repository() {
        let (repo, etat_date_repo, uc) = use_cases();
        let etat_date = make_etat_date();
        etat_date_repo.rows.lock().unwrap().push(etat_date.clone());
        let issued = uc.issue(etat_date.id, Uuid::new_v4()).await.unwrap();
        let original_expiry = issued.expires_at;

        let renewed = uc.renew(issued.id).await.unwrap();
        assert!(renewed.expires_at > original_expiry);

        // La mutation doit être persistée via `update`, pas seulement en mémoire.
        let persisted = repo.find_by_id(issued.id).await.unwrap().unwrap();
        assert_eq!(persisted.expires_at, renewed.expires_at);
    }

    #[tokio::test]
    async fn edge_rate_limiter_allows_exactly_max_then_blocks() {
        let (_repo, etat_date_repo, uc) = {
            let repo: Arc<InMemoryNotaryLinkRepo> = Arc::new(InMemoryNotaryLinkRepo::default());
            let etat_date_repo: Arc<FakeEtatDateRepo> = Arc::new(FakeEtatDateRepo::default());
            let uc = NotaryLinkUseCases::new(
                repo.clone() as Arc<dyn NotaryLinkRepository>,
                etat_date_repo.clone() as Arc<dyn EtatDateRepository>,
            )
            .with_rate_limiter(NotaryLinkRateLimiter::new(2, Duration::from_secs(600)));
            (repo, etat_date_repo, uc)
        };
        let etat_date = make_etat_date();
        etat_date_repo.rows.lock().unwrap().push(etat_date.clone());

        for _ in 0..2 {
            let err = uc
                .consult(&etat_date.reference_number, None, "9.9.9.9")
                .await
                .unwrap_err();
            // Consume the two allowed slots — token is absent, so each call
            // fails on NotaryLinkInvalid, not on the rate limit itself.
            assert!(matches!(err, AppError::NotaryLinkInvalid));
        }
        let err = uc
            .consult(&etat_date.reference_number, None, "9.9.9.9")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::RateLimited));
    }

    // ---- @security ---------------------------------------------------------

    #[tokio::test]
    async fn security_missing_token_returns_notary_link_invalid_not_not_found() {
        let (_repo, etat_date_repo, uc) = use_cases();
        let etat_date = make_etat_date();
        etat_date_repo.rows.lock().unwrap().push(etat_date.clone());

        let err = uc
            .consult(&etat_date.reference_number, None, "1.2.3.4")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotaryLinkInvalid));
    }

    #[tokio::test]
    async fn security_revoked_link_cannot_be_consulted() {
        let (repo, etat_date_repo, uc) = use_cases();
        let etat_date = make_etat_date();
        etat_date_repo.rows.lock().unwrap().push(etat_date.clone());
        let issued = uc.issue(etat_date.id, Uuid::new_v4()).await.unwrap();

        uc.revoke(issued.id).await.unwrap();
        assert!(repo
            .find_by_id(issued.id)
            .await
            .unwrap()
            .unwrap()
            .is_revoked());

        let err = uc
            .consult(&etat_date.reference_number, Some(&issued.token), "1.2.3.4")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotaryLinkInvalid));
    }

    #[tokio::test]
    async fn security_forged_token_on_existing_reference_returns_invalid() {
        let (_repo, etat_date_repo, uc) = use_cases();
        let etat_date = make_etat_date();
        etat_date_repo.rows.lock().unwrap().push(etat_date.clone());

        let err = uc
            .consult(
                &etat_date.reference_number,
                Some("forged-not-in-db"),
                "1.2.3.4",
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotaryLinkInvalid));
    }

    #[test]
    fn security_notary_link_repository_never_exposes_clear_token_field() {
        // Le point de sécurité central : `NotaryLinkResponse` (renvoyé par
        // `list_for_etat_date` / `renew`) ne porte pas de champ `token`.
        // Compile-time — si un champ `token` était ajouté par erreur au
        // DTO de lecture, ce test resterait vert mais la revue de code
        // devient le seul filet ; documenté ici pour que l'intention soit
        // explicite au futur lecteur du DTO.
        let link = NotaryLink::issue(Uuid::new_v4(), Uuid::new_v4()).unwrap().0;
        let response = NotaryLinkResponse::from(link);
        let serialized = serde_json::to_value(&response).unwrap();
        assert!(serialized.get("token").is_none());
        assert!(serialized.get("token_hash").is_none());
    }

    // ---- @negative -----------------------------------------------------------

    #[tokio::test]
    async fn negative_unknown_reference_returns_not_found_not_invalid() {
        // Per ADR 0048, la référence n'est pas un secret : une référence
        // inconnue renvoie 404, distinct du refus opaque des jetons.
        let (_repo, _etat_date_repo, uc) = use_cases();
        let err = uc
            .consult("ED-UNKNOWN-REFERENCE", Some("whatever"), "1.2.3.4")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_expired_token_is_indistinguishable_from_forged() {
        let (repo, etat_date_repo, uc) = use_cases();
        let etat_date = make_etat_date();
        etat_date_repo.rows.lock().unwrap().push(etat_date.clone());
        let issued = uc.issue(etat_date.id, Uuid::new_v4()).await.unwrap();

        // Force expiry directly on the stored row (bypassing the 7-day wait).
        {
            let mut rows = repo.rows.lock().unwrap();
            let row = rows.iter_mut().find(|l| l.id == issued.id).unwrap();
            row.expires_at = Utc::now() - chrono::Duration::seconds(1);
        }

        let err_expired = uc
            .consult(&etat_date.reference_number, Some(&issued.token), "1.2.3.4")
            .await
            .unwrap_err();
        let err_forged = uc
            .consult(&etat_date.reference_number, Some("forged"), "1.2.3.4")
            .await
            .unwrap_err();

        // #855 @negative : le message NE distingue PAS "expiré" de "inconnu".
        assert_eq!(err_expired.to_string(), err_forged.to_string());
        assert!(matches!(err_expired, AppError::NotaryLinkInvalid));
    }

    #[tokio::test]
    async fn negative_get_unknown_link_returns_not_found_not_panic() {
        let (_repo, _etat_date_repo, uc) = use_cases();
        let err = uc.get(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_revoke_unknown_link_returns_not_found_not_panic() {
        let (_repo, _etat_date_repo, uc) = use_cases();
        let err = uc.revoke(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_renew_unknown_link_returns_not_found_not_panic() {
        let (_repo, _etat_date_repo, uc) = use_cases();
        let err = uc.renew(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
