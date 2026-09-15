//! Use cases for the notary link feature (#845 — ADR 0048, ADR 0051).
//!
//! Four operations, all syndic-gated except the last:
//! 1. [`LienNotaireUseCases::issue`] — a syndic issues a link for one état
//!    daté. Returns the clear token ONCE.
//! 2. [`LienNotaireUseCases::renew`] — the syndic extends the active link by
//!    seven more days from now. Same token, new `expire_le`.
//! 3. [`LienNotaireUseCases::revoke`] — the syndic kills the active link
//!    before term.
//! 4. [`LienNotaireUseCases::verify_token`] — called on every
//!    `GET /etats-dates/reference/{reference_number}?token=...`. Repeatable
//!    (no consumption): the notary may read the same état daté several times
//!    within the seven-day window (ADR 0051).

use crate::application::error::AppError;
use crate::application::ports::LienNotaireRepository;
use crate::domain::entities::LienNotaire;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedLienNotaireDto {
    pub id: Uuid,
    /// Jeton clair — à renvoyer au syndic UNE FOIS, jamais persisté ailleurs.
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LienNotaireStatusDto {
    pub id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub renewed_at: Option<DateTime<Utc>>,
}

impl From<&LienNotaire> for LienNotaireStatusDto {
    fn from(lien: &LienNotaire) -> Self {
        Self {
            id: lien.id,
            expires_at: lien.expire_le,
            renewed_at: lien.renouvele_le,
        }
    }
}

pub struct LienNotaireUseCases {
    repo: Arc<dyn LienNotaireRepository>,
}

impl LienNotaireUseCases {
    pub fn new(repo: Arc<dyn LienNotaireRepository>) -> Self {
        Self { repo }
    }

    /// Issue a new link. Caller MUST have already authorised the request
    /// (org-scope check on `etat_date_id` happens at the handler level, via
    /// `verify_etat_date_org_access`).
    pub async fn issue(
        &self,
        etat_date_id: Uuid,
        emis_par: Uuid,
    ) -> Result<IssuedLienNotaireDto, AppError> {
        let (lien, clair) = LienNotaire::emettre(etat_date_id, emis_par)?;
        self.repo.save(&lien).await?;

        Ok(IssuedLienNotaireDto {
            id: lien.id,
            token: clair,
            expires_at: lien.expire_le,
        })
    }

    /// Renew the active link for a given état daté — same token, expiry
    /// pushed seven days from now.
    pub async fn renew(&self, etat_date_id: Uuid) -> Result<LienNotaireStatusDto, AppError> {
        let mut lien = self
            .repo
            .find_active_by_etat_date_id(etat_date_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("notary link for {etat_date_id}")))?;

        lien.renouveler(Utc::now())?;
        self.repo.update(&lien).await?;

        Ok(LienNotaireStatusDto::from(&lien))
    }

    /// Revoke the active link for a given état daté.
    pub async fn revoke(&self, etat_date_id: Uuid, revoked_by: Uuid) -> Result<(), AppError> {
        let mut lien = self
            .repo
            .find_active_by_etat_date_id(etat_date_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("notary link for {etat_date_id}")))?;

        lien.revoquer(Utc::now(), revoked_by);
        self.repo.update(&lien).await
    }

    /// Verify a clear token grants access to `etat_date_id`. Repeatable —
    /// does NOT consume the link (ADR 0051, multi-lecture).
    ///
    /// Possible errors (all 403 by design, uniform with "unknown token" to
    /// defeat enumeration — same rationale as `MagicLinkInvalid`):
    /// - `AppError::NotaryLinkInvalid` — forged / unknown / wrong état daté.
    /// - `AppError::NotaryLinkExpired` — the seven days elapsed.
    /// - `AppError::NotaryLinkRevoked` — the syndic killed it early.
    pub async fn verify_token(
        &self,
        etat_date_id: Uuid,
        clear_token: &str,
    ) -> Result<LienNotaire, AppError> {
        if clear_token.trim().is_empty() {
            return Err(AppError::NotaryLinkInvalid);
        }

        let token_hash = LienNotaire::hacher(clear_token);
        let lien = self
            .repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(AppError::NotaryLinkInvalid)?;

        if lien.etat_date_id != etat_date_id {
            // Un jeton d'un autre état daté ne doit pas distinguablement
            // échouer d'un jeton forgé (#845 @security).
            return Err(AppError::NotaryLinkInvalid);
        }
        if lien.est_revoque() {
            return Err(AppError::NotaryLinkRevoked);
        }
        if lien.est_expire(Utc::now()) {
            return Err(AppError::NotaryLinkExpired);
        }

        Ok(lien)
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories (CRITICAL.md #3). Bornes reprises d'ADR 0051
// §"Les quatre classes de tests".
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Duration;
    use std::sync::Mutex;

    #[derive(Default)]
    struct InMemoryRepo {
        rows: Mutex<Vec<LienNotaire>>,
    }

    #[async_trait]
    impl LienNotaireRepository for InMemoryRepo {
        async fn save(&self, lien: &LienNotaire) -> Result<(), AppError> {
            self.rows.lock().unwrap().push(lien.clone());
            Ok(())
        }

        async fn find_by_token_hash(
            &self,
            token_hash: &str,
        ) -> Result<Option<LienNotaire>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|l| l.token_hash == token_hash)
                .cloned())
        }

        async fn find_active_by_etat_date_id(
            &self,
            etat_date_id: Uuid,
        ) -> Result<Option<LienNotaire>, AppError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|l| l.etat_date_id == etat_date_id && !l.est_revoque())
                .max_by_key(|l| l.cree_le)
                .cloned())
        }

        async fn update(&self, lien: &LienNotaire) -> Result<(), AppError> {
            let mut rows = self.rows.lock().unwrap();
            if let Some(row) = rows.iter_mut().find(|l| l.id == lien.id) {
                *row = lien.clone();
            }
            Ok(())
        }
    }

    fn use_cases() -> (Arc<InMemoryRepo>, LienNotaireUseCases) {
        let repo: Arc<InMemoryRepo> = Arc::new(InMemoryRepo::default());
        let uc = LienNotaireUseCases::new(repo.clone() as Arc<dyn LienNotaireRepository>);
        (repo, uc)
    }

    // ---- @happy — lecture dans la fenêtre --------------------------------

    #[tokio::test]
    async fn happy_issue_then_verify_succeeds_and_is_repeatable() {
        let (_repo, uc) = use_cases();
        let etat_date_id = Uuid::new_v4();
        let syndic = Uuid::new_v4();

        let issued = uc.issue(etat_date_id, syndic).await.unwrap();
        assert!(!issued.token.is_empty());

        let lien_1 = uc.verify_token(etat_date_id, &issued.token).await.unwrap();
        let lien_2 = uc.verify_token(etat_date_id, &issued.token).await.unwrap();
        assert_eq!(
            lien_1.id, lien_2.id,
            "multi-lecture : le même jeton se relit"
        );
    }

    #[tokio::test]
    async fn happy_renew_extends_expiry_and_keeps_same_token() {
        let (_repo, uc) = use_cases();
        let etat_date_id = Uuid::new_v4();
        let issued = uc.issue(etat_date_id, Uuid::new_v4()).await.unwrap();

        let status = uc.renew(etat_date_id).await.unwrap();
        assert!(status.expires_at > issued.expires_at);
        assert!(status.renewed_at.is_some());

        // Le jeton d'origine ouvre toujours — le renouvellement ne le change pas.
        uc.verify_token(etat_date_id, &issued.token).await.unwrap();
    }

    // ---- @edge — bornes J+7 / J+8, renouvellement en chaîne ----------------

    #[tokio::test]
    async fn edge_verify_at_j8_after_expiry_fails() {
        let repo = Arc::new(InMemoryRepo::default());
        let (mut lien, clair) = LienNotaire::emettre(Uuid::new_v4(), Uuid::new_v4()).unwrap();
        lien.expire_le = Utc::now() - Duration::seconds(1);
        let etat_date_id = lien.etat_date_id;
        repo.rows.lock().unwrap().push(lien);

        let uc = LienNotaireUseCases::new(repo as Arc<dyn LienNotaireRepository>);
        let err = uc.verify_token(etat_date_id, &clair).await.unwrap_err();
        assert!(matches!(err, AppError::NotaryLinkExpired));
    }

    #[tokio::test]
    async fn edge_renew_missing_link_returns_not_found() {
        let (_repo, uc) = use_cases();
        let err = uc.renew(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn edge_renewal_chain_keeps_pushing_the_deadline() {
        let (_repo, uc) = use_cases();
        let etat_date_id = Uuid::new_v4();
        uc.issue(etat_date_id, Uuid::new_v4()).await.unwrap();

        let first = uc.renew(etat_date_id).await.unwrap();
        let second = uc.renew(etat_date_id).await.unwrap();
        assert!(second.expires_at >= first.expires_at);
    }

    // ---- @security — jeton forgé, autre état daté, révoqué, rejeu ----------

    #[tokio::test]
    async fn security_forged_token_returns_invalid() {
        let (_repo, uc) = use_cases();
        let err = uc
            .verify_token(Uuid::new_v4(), "forged-not-in-db")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotaryLinkInvalid));
    }

    #[tokio::test]
    async fn security_token_scoped_to_another_etat_date_is_rejected_uniformly() {
        let (_repo, uc) = use_cases();
        let etat_date_a = Uuid::new_v4();
        let etat_date_b = Uuid::new_v4();
        let issued = uc.issue(etat_date_a, Uuid::new_v4()).await.unwrap();

        let err = uc
            .verify_token(etat_date_b, &issued.token)
            .await
            .unwrap_err();
        // Même erreur qu'un jeton forgé — pas de "wrong scope" distinguable.
        assert!(matches!(err, AppError::NotaryLinkInvalid));
    }

    #[tokio::test]
    async fn security_revoked_link_is_rejected_even_within_the_window() {
        let (_repo, uc) = use_cases();
        let etat_date_id = Uuid::new_v4();
        let issued = uc.issue(etat_date_id, Uuid::new_v4()).await.unwrap();

        uc.revoke(etat_date_id, Uuid::new_v4()).await.unwrap();

        let err = uc
            .verify_token(etat_date_id, &issued.token)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotaryLinkRevoked));
    }

    #[tokio::test]
    async fn security_replay_after_expiry_still_fails() {
        // Le jeton reste multi-lecture DANS la fenêtre, mais un rejeu après
        // l'échéance échoue comme n'importe quelle autre lecture tardive.
        let repo = Arc::new(InMemoryRepo::default());
        let (mut lien, clair) = LienNotaire::emettre(Uuid::new_v4(), Uuid::new_v4()).unwrap();
        lien.expire_le = Utc::now() - Duration::days(1);
        let etat_date_id = lien.etat_date_id;
        repo.rows.lock().unwrap().push(lien);

        let uc = LienNotaireUseCases::new(repo as Arc<dyn LienNotaireRepository>);
        uc.verify_token(etat_date_id, &clair).await.unwrap_err();
        let err = uc.verify_token(etat_date_id, &clair).await.unwrap_err();
        assert!(matches!(err, AppError::NotaryLinkExpired));
    }

    // ---- @negative — référence inconnue, jeton malformé --------------------

    #[tokio::test]
    async fn negative_empty_token_returns_invalid_without_db_lookup() {
        let (repo, uc) = use_cases();
        let err = uc.verify_token(Uuid::new_v4(), "   ").await.unwrap_err();
        assert!(matches!(err, AppError::NotaryLinkInvalid));
        assert!(repo.rows.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn negative_revoke_missing_link_returns_not_found_not_panic() {
        let (_repo, uc) = use_cases();
        let err = uc.revoke(Uuid::new_v4(), Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_issue_with_nil_etat_date_id_is_typed_not_a_panic() {
        let (_repo, uc) = use_cases();
        let err = uc.issue(Uuid::nil(), Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
