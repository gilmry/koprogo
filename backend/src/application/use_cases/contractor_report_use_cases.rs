use crate::application::dto::contractor_report_dto::{
    ContractorReportResponseDto, CreateContractorReportDto, MagicLinkResponseDto, RejectReportDto,
    RequestCorrectionsDto, UpdateContractorReportDto,
};
use crate::application::dto::payment_dto::CreatePaymentRequest;
use crate::application::error::AppError;
use crate::application::ports::contractor_report_repository::ContractorReportRepository;
use crate::application::ports::quote_repository::QuoteRepository;
use crate::application::use_cases::{MagicLinkUseCases, PaymentUseCases};
use crate::domain::entities::contractor_report::{ContractorReport, ContractorReportStatus};
use crate::domain::entities::{MagicLinkScopeKind, PaymentMethodType};
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Durée de validité du magic link (72 heures)
const MAGIC_LINK_VALIDITY_HOURS: i64 = 72;

pub struct ContractorReportUseCases {
    pub repo: Arc<dyn ContractorReportRepository>,
    pub quote_repo: Option<Arc<dyn QuoteRepository>>,
    pub payment_use_cases: Option<Arc<PaymentUseCases>>,
    /// #835 — émission/résolution des liens magiques unifiés (scope
    /// `ContractorReport`). `None` seulement en test unitaire pur (mocks qui
    /// n'exercent pas `generate_magic_link`).
    pub magic_link_use_cases: Option<Arc<MagicLinkUseCases>>,
}

impl ContractorReportUseCases {
    pub fn new(repo: Arc<dyn ContractorReportRepository>) -> Self {
        Self {
            repo,
            quote_repo: None,
            payment_use_cases: None,
            magic_link_use_cases: None,
        }
    }

    pub fn with_payment_support(
        mut self,
        quote_repo: Arc<dyn QuoteRepository>,
        payment_use_cases: Arc<PaymentUseCases>,
    ) -> Self {
        self.quote_repo = Some(quote_repo);
        self.payment_use_cases = Some(payment_use_cases);
        self
    }

    /// #835 — branche le système générique de liens magiques (absorption du
    /// second système qui stockait un token brut sur `contractor_reports`).
    pub fn with_magic_link_support(mut self, magic_link_use_cases: Arc<MagicLinkUseCases>) -> Self {
        self.magic_link_use_cases = Some(magic_link_use_cases);
        self
    }

    /// Crée un nouveau rapport de travaux (B16-1)
    pub async fn create(
        &self,
        organization_id: Uuid,
        dto: CreateContractorReportDto,
    ) -> Result<ContractorReportResponseDto, String> {
        let report = ContractorReport::new(
            organization_id,
            dto.building_id,
            dto.contractor_name,
            dto.ticket_id,
            dto.quote_id,
            dto.contractor_user_id,
        )?;
        let saved = self.repo.create(&report).await?;
        Ok(ContractorReportResponseDto::from(&saved))
    }

    /// Récupère un rapport par son ID (vérification organisation)
    pub async fn get(
        &self,
        id: Uuid,
        organization_id: Uuid,
    ) -> Result<ContractorReportResponseDto, String> {
        let report = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Rapport {} introuvable", id))?;

        if report.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }
        Ok(ContractorReportResponseDto::from(&report))
    }

    /// Récupère un rapport via magic token (accès PWA sans auth)
    pub async fn get_by_token(
        &self,
        token_hash: &str,
    ) -> Result<ContractorReportResponseDto, String> {
        let report = self
            .repo
            .find_by_magic_token(token_hash)
            .await?
            .ok_or("Lien invalide ou expiré".to_string())?;

        if !report.is_magic_token_valid() {
            return Err("Le lien magic a expiré (validité 72h)".to_string());
        }
        Ok(ContractorReportResponseDto::from(&report))
    }

    /// Liste les rapports d'un bâtiment
    pub async fn list_by_building(
        &self,
        building_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<ContractorReportResponseDto>, String> {
        let reports = self.repo.find_by_building(building_id).await?;
        Ok(reports
            .iter()
            .filter(|r| r.organization_id == organization_id)
            .map(ContractorReportResponseDto::from)
            .collect())
    }

    /// Liste les rapports d'un ticket
    pub async fn list_by_ticket(
        &self,
        ticket_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<ContractorReportResponseDto>, String> {
        let reports = self.repo.find_by_ticket(ticket_id).await?;
        Ok(reports
            .iter()
            .filter(|r| r.organization_id == organization_id)
            .map(ContractorReportResponseDto::from)
            .collect())
    }

    /// Met à jour le brouillon du rapport (photos, pièces, compte-rendu)
    pub async fn update(
        &self,
        id: Uuid,
        organization_id: Uuid,
        dto: UpdateContractorReportDto,
    ) -> Result<ContractorReportResponseDto, String> {
        let mut report = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Rapport {} introuvable", id))?;

        if report.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }
        if report.status != ContractorReportStatus::Draft
            && report.status != ContractorReportStatus::RequiresCorrection
        {
            return Err(format!(
                "Impossible de modifier depuis l'état {:?}",
                report.status
            ));
        }

        if let Some(date) = dto.work_date {
            report.work_date = Some(date);
        }
        if let Some(cr) = dto.compte_rendu {
            report.compte_rendu = Some(cr);
        }
        if let Some(photos) = dto.photos_before {
            report.photos_before = photos;
        }
        if let Some(photos) = dto.photos_after {
            report.photos_after = photos;
        }
        if let Some(parts) = dto.parts_replaced {
            report.parts_replaced = parts.into_iter().map(Into::into).collect();
        }
        report.updated_at = Utc::now();

        let saved = self.repo.update(&report).await?;
        Ok(ContractorReportResponseDto::from(&saved))
    }

    /// Corps de métier soumet le rapport pour validation CdC (B16-3)
    pub async fn submit(
        &self,
        id: Uuid,
        organization_id: Uuid,
    ) -> Result<ContractorReportResponseDto, String> {
        let mut report = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Rapport {} introuvable", id))?;

        if report.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }
        report.submit()?;
        let saved = self.repo.update(&report).await?;
        Ok(ContractorReportResponseDto::from(&saved))
    }

    /// Accès via magic link : soumet le rapport sans authentification classique
    pub async fn submit_by_token(
        &self,
        token_hash: &str,
    ) -> Result<ContractorReportResponseDto, String> {
        let mut report = self
            .repo
            .find_by_magic_token(token_hash)
            .await?
            .ok_or("Lien invalide ou expiré".to_string())?;

        if !report.is_magic_token_valid() {
            return Err("Le lien magic a expiré (validité 72h)".to_string());
        }
        report.submit()?;
        let saved = self.repo.update(&report).await?;
        Ok(ContractorReportResponseDto::from(&saved))
    }

    /// CdC valide le rapport → paiement automatique déclenché (B16-6)
    ///
    /// Dans une implémentation complète, on ferait appel au PaymentUseCases ici.
    /// Pour l'instant on retourne le rapport validé et on documente le hook.
    pub async fn validate(
        &self,
        id: Uuid,
        organization_id: Uuid,
        validated_by: Uuid,
    ) -> Result<ContractorReportResponseDto, String> {
        let mut report = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Rapport {} introuvable", id))?;

        if report.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }
        report.validate(validated_by)?;
        let saved = self.repo.update(&report).await?;

        // B16-6: Trigger automatic payment if quote_id is present
        if let Some(quote_id) = saved.quote_id {
            if let (Some(quote_repo), Some(payment_uc)) =
                (&self.quote_repo, &self.payment_use_cases)
            {
                if let Ok(Some(quote)) = quote_repo.find_by_id(quote_id).await {
                    // A quote that was never submitted (Requested, no price
                    // data) has nothing to pay — skip the auto-payment.
                    let amount_cents = quote
                        .amount_incl_vat
                        .map(|a| {
                            (a * rust_decimal::Decimal::from(100))
                                .to_string()
                                .parse::<i64>()
                                .unwrap_or(0)
                        })
                        .unwrap_or(0);

                    if amount_cents > 0 {
                        let payment_req = CreatePaymentRequest {
                            building_id: quote.building_id,
                            owner_id: quote.contractor_id,
                            expense_id: None,
                            // Paiement SORTANT vers un prestataire : il ne
                            // solde aucune quote-part de coproprietaire.
                            contribution_id: None,
                            amount_cents,
                            payment_method_type: PaymentMethodType::BankTransfer,
                            payment_method_id: None,
                            description: Some(format!(
                                "Paiement prestataire — Rapport #{} validé (Devis {})",
                                saved.id, quote.project_title
                            )),
                            metadata: Some(
                                serde_json::json!({
                                    "contractor_report_id": saved.id,
                                    "quote_id": quote_id,
                                })
                                .to_string(),
                            ),
                        };
                        // Fire-and-forget: payment creation failure should not block report validation
                        let _ = payment_uc
                            .create_payment(organization_id, payment_req)
                            .await;
                    }
                }
            }
        }

        Ok(ContractorReportResponseDto::from(&saved))
    }

    /// CdC demande des corrections au corps de métier
    pub async fn request_corrections(
        &self,
        id: Uuid,
        organization_id: Uuid,
        dto: RequestCorrectionsDto,
    ) -> Result<ContractorReportResponseDto, String> {
        let mut report = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Rapport {} introuvable", id))?;

        if report.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }
        report.request_corrections(dto.comments)?;
        let saved = self.repo.update(&report).await?;
        Ok(ContractorReportResponseDto::from(&saved))
    }

    /// CdC rejette le rapport
    pub async fn reject(
        &self,
        id: Uuid,
        organization_id: Uuid,
        dto: RejectReportDto,
        rejected_by: Uuid,
    ) -> Result<ContractorReportResponseDto, String> {
        let mut report = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Rapport {} introuvable", id))?;

        if report.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }
        report.reject(dto.comments, rejected_by)?;
        let saved = self.repo.update(&report).await?;
        Ok(ContractorReportResponseDto::from(&saved))
    }

    /// Génère un lien magique pour l'accès PWA corps de métier (B16-2).
    ///
    /// #835 — délègue au système générique (`MagicLinkUseCases`, scope
    /// `ContractorReport`) au lieu d'écrire un token brut dans
    /// `contractor_reports.magic_token_hash`. Le prestataire reçoit désormais
    /// UN lien `/c?t=...`, celui que sait déjà lire `MagicLinkContractorPage`,
    /// au lieu d'un second format (`/contractor/?token=...`) menant à une
    /// page distincte. Cf. issue #835 — deux systèmes de liens magiques
    /// parallèles pour un même chantier.
    ///
    /// `subject_user_id` : le prestataire n'a souvent PAS de compte (#815 —
    /// la voie nominale est le lien, pas le compte). `contractor_user_id` est
    /// alors `None` et on utilise `Uuid::nil()`, un sentinel accepté par
    /// `MagicLink::issue` (seule contrainte : différer de `issued_by`) — le
    /// token brut, pas l'identité du sujet, est ce qui autorise l'accès ici.
    pub async fn generate_magic_link(
        &self,
        report_id: Uuid,
        organization_id: Uuid,
        issued_by: Uuid,
        base_url: &str,
    ) -> Result<MagicLinkResponseDto, AppError> {
        let report = self
            .repo
            .find_by_id(report_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound(format!("contractor_report {}", report_id)))?;

        if report.organization_id != organization_id {
            return Err(AppError::Forbidden("Accès refusé".to_string()));
        }

        let magic_link_use_cases = self
            .magic_link_use_cases
            .as_ref()
            .ok_or_else(|| AppError::Internal("MagicLinkUseCases non configuré".to_string()))?;

        let subject_user_id = report.contractor_user_id.unwrap_or(Uuid::nil());
        let issued = magic_link_use_cases
            .issue(
                subject_user_id,
                MagicLinkScopeKind::ContractorReport,
                report_id,
                issued_by,
                Duration::hours(MAGIC_LINK_VALIDITY_HOURS).num_seconds(),
            )
            .await?;

        let magic_link = format!("{}/c?t={}", base_url.trim_end_matches('/'), issued.token);

        Ok(MagicLinkResponseDto {
            magic_link,
            expires_at: issued.expires_at,
        })
    }

    /// Lecture du rapport pour l'écran unifié lien magique (#835). L'autorisation
    /// est déjà assurée en amont par la résolution du jeton (scope cloisonné,
    /// cf. `MagicLinkUseCases::ensure_scope`) — pas de filtre organisation ici,
    /// l'appelant est anonyme par nature (c'est le principe même du lien).
    pub async fn get_via_magic_link(
        &self,
        report_id: Uuid,
    ) -> Result<ContractorReportResponseDto, AppError> {
        let report = self
            .repo
            .find_by_id(report_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound(format!("contractor_report {}", report_id)))?;
        Ok(ContractorReportResponseDto::from(&report))
    }

    /// Applique les champs du brouillon puis soumet, en un seul aller-retour —
    /// c'est l'action « respond » du lien magique unifié (#835). Contrairement
    /// à `submit_by_token` (système B, retiré), les champs soumis (compte-rendu,
    /// date, pièces) sont réellement persistés avant la transition d'état :
    /// l'ancien endpoint les ignorait silencieusement (le corps de la requête
    /// n'était jamais lu), ce qui rendait la soumission systématiquement en
    /// échec faute de `compte_rendu` déjà enregistré par ailleurs.
    pub async fn respond_via_magic_link(
        &self,
        report_id: Uuid,
        dto: UpdateContractorReportDto,
    ) -> Result<ContractorReportResponseDto, AppError> {
        let mut report = self
            .repo
            .find_by_id(report_id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound(format!("contractor_report {}", report_id)))?;

        if let Some(date) = dto.work_date {
            report.work_date = Some(date);
        }
        if let Some(cr) = dto.compte_rendu {
            report.compte_rendu = Some(cr);
        }
        if let Some(photos) = dto.photos_before {
            report.photos_before = photos;
        }
        if let Some(photos) = dto.photos_after {
            report.photos_after = photos;
        }
        if let Some(parts) = dto.parts_replaced {
            report.parts_replaced = parts.into_iter().map(Into::into).collect();
        }
        report.updated_at = Utc::now();

        report.submit().map_err(AppError::Validation)?;

        let saved = self
            .repo
            .update(&report)
            .await
            .map_err(AppError::Internal)?;
        Ok(ContractorReportResponseDto::from(&saved))
    }

    /// Supprime un rapport (Draft seulement)
    pub async fn delete(&self, id: Uuid, organization_id: Uuid) -> Result<(), String> {
        let report = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Rapport {} introuvable", id))?;

        if report.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }
        if report.status != ContractorReportStatus::Draft {
            return Err(format!(
                "Seuls les rapports en brouillon peuvent être supprimés (état actuel: {:?})",
                report.status
            ));
        }
        self.repo.delete(id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::contractor_report_repository::ContractorReportRepository;
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        ContractorReportRepo {}

        #[async_trait]
        impl ContractorReportRepository for ContractorReportRepo {
            async fn create(&self, report: &ContractorReport) -> Result<ContractorReport, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<ContractorReport>, String>;
            async fn find_by_magic_token(&self, token_hash: &str) -> Result<Option<ContractorReport>, String>;
            async fn find_by_ticket(&self, ticket_id: Uuid) -> Result<Vec<ContractorReport>, String>;
            async fn find_by_quote(&self, quote_id: Uuid) -> Result<Vec<ContractorReport>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<ContractorReport>, String>;
            async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<ContractorReport>, String>;
            async fn update(&self, report: &ContractorReport) -> Result<ContractorReport, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
        }
    }

    fn make_draft_report(org_id: Uuid) -> ContractorReport {
        let mut r = ContractorReport::new(
            org_id,
            Uuid::new_v4(),
            "Martin Plomberie SPRL".to_string(),
            Some(Uuid::new_v4()),
            None,
            None,
        )
        .unwrap();
        r.compte_rendu = Some("Travaux effectués conformément au devis".to_string());
        r
    }

    #[tokio::test]
    async fn test_create_report_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let ticket_id = Uuid::new_v4();

        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo.expect_create().returning(|r| Ok(r.clone()));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));

        let dto = CreateContractorReportDto {
            building_id,
            contractor_name: "Plombier SA".to_string(),
            ticket_id: Some(ticket_id),
            quote_id: None,
            contractor_user_id: None,
        };

        let result = uc.create(org_id, dto).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.organization_id, org_id);
        assert_eq!(resp.building_id, building_id);
        assert_eq!(resp.contractor_name, "Plombier SA");
        assert_eq!(resp.status, "draft");
    }

    #[tokio::test]
    async fn test_get_by_token_success() {
        let org_id = Uuid::new_v4();
        let mut report = make_draft_report(org_id);
        let token = "valid-token-hash";
        report.magic_token_hash = Some(token.to_string());
        report.magic_token_expires_at = Some(Utc::now() + Duration::hours(24));

        let report_clone = report.clone();
        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_magic_token()
            .withf(|t| t == "valid-token-hash")
            .returning(move |_| Ok(Some(report_clone.clone())));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));

        let result = uc.get_by_token(token).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.contractor_name, "Martin Plomberie SPRL");
    }

    #[tokio::test]
    async fn test_submit_report_success() {
        let org_id = Uuid::new_v4();
        let report = make_draft_report(org_id);
        let report_id = report.id;

        let report_for_find = report.clone();
        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == report_id)
            .returning(move |_| Ok(Some(report_for_find.clone())));
        mock_repo.expect_update().returning(|r| Ok(r.clone()));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));

        let result = uc.submit(report_id, org_id).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, "submitted");
    }

    #[tokio::test]
    async fn test_start_review_via_update() {
        // Test the review workflow by submitting then validating (which accepts Submitted state)
        let org_id = Uuid::new_v4();
        let mut report = make_draft_report(org_id);
        // Pre-set to Submitted state to test review path
        report.status = ContractorReportStatus::Submitted;
        report.submitted_at = Some(Utc::now());
        let report_id = report.id;

        let report_for_find = report.clone();
        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == report_id)
            .returning(move |_| Ok(Some(report_for_find.clone())));
        mock_repo.expect_update().returning(|r| Ok(r.clone()));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));
        let validator_id = Uuid::new_v4();

        let result = uc.validate(report_id, org_id, validator_id).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, "validated");
        assert_eq!(resp.validated_by, Some(validator_id));
    }

    #[tokio::test]
    async fn test_validate_report_success() {
        let org_id = Uuid::new_v4();
        let validator_id = Uuid::new_v4();
        let mut report = make_draft_report(org_id);
        report.status = ContractorReportStatus::UnderReview;
        let report_id = report.id;

        let report_for_find = report.clone();
        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == report_id)
            .returning(move |_| Ok(Some(report_for_find.clone())));
        mock_repo.expect_update().returning(|r| Ok(r.clone()));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));

        let result = uc.validate(report_id, org_id, validator_id).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, "validated");
        assert!(resp.validated_at.is_some());
        assert_eq!(resp.validated_by, Some(validator_id));
    }

    #[tokio::test]
    async fn test_generate_magic_link_success() {
        // #835 — la génération délègue désormais au système générique de
        // liens magiques et produit une URL `/c?t=...`, celle que sait déjà
        // rendre `MagicLinkContractorPage`, au lieu de `/contractor/?token=`
        // (second système, absorbé).
        let org_id = Uuid::new_v4();
        let issued_by = Uuid::new_v4();
        let report = make_draft_report(org_id);
        let report_id = report.id;

        let report_for_find = report.clone();
        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == report_id)
            .returning(move |_| Ok(Some(report_for_find.clone())));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo))
            .with_magic_link_support(new_magic_link_use_cases());

        let result = uc
            .generate_magic_link(report_id, org_id, issued_by, "https://app.koprogo.be")
            .await;
        assert!(result.is_ok(), "{:?}", result.err());
        let link_dto = result.unwrap();
        assert!(
            link_dto
                .magic_link
                .starts_with("https://app.koprogo.be/c?t="),
            "unexpected magic_link: {}",
            link_dto.magic_link
        );
        assert!(link_dto.expires_at > Utc::now());
    }

    // ========================================================================
    // #835 — liens magiques unifiés (scope ContractorReport)
    // ========================================================================

    /// Dépôt en mémoire pour `MagicLinkRepository`, suffisant pour exercer
    /// `generate_magic_link` / `get_via_magic_link` / `respond_via_magic_link`
    /// sans DB réelle.
    #[derive(Default)]
    struct InMemoryMagicLinkRepo {
        rows: std::sync::Mutex<Vec<crate::domain::entities::MagicLink>>,
    }

    #[async_trait]
    impl crate::application::ports::MagicLinkRepository for InMemoryMagicLinkRepo {
        async fn save(&self, link: &crate::domain::entities::MagicLink) -> Result<(), AppError> {
            self.rows.lock().unwrap().push(link.clone());
            Ok(())
        }

        async fn find_by_token_hash(
            &self,
            token_hash: &str,
        ) -> Result<Option<crate::domain::entities::MagicLink>, AppError> {
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
                row.consumed_at = Some(Utc::now());
            }
            Ok(())
        }
    }

    fn new_magic_link_use_cases() -> Arc<MagicLinkUseCases> {
        let repo: Arc<dyn crate::application::ports::MagicLinkRepository> =
            Arc::new(InMemoryMagicLinkRepo::default());
        Arc::new(MagicLinkUseCases::new(repo))
    }

    // ---- @happy --------------------------------------------------------------

    #[tokio::test]
    async fn happy_generate_magic_link_uses_nil_subject_when_contractor_has_no_account() {
        // #815 — le prestataire n'a souvent PAS de compte : contractor_user_id
        // est None et ne doit pas empêcher l'émission.
        let org_id = Uuid::new_v4();
        let issued_by = Uuid::new_v4();
        let report = make_draft_report(org_id);
        assert!(report.contractor_user_id.is_none());
        let report_id = report.id;

        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(report.clone())));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo))
            .with_magic_link_support(new_magic_link_use_cases());

        let result = uc
            .generate_magic_link(report_id, org_id, issued_by, "https://app.koprogo.be")
            .await;
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[tokio::test]
    async fn happy_get_via_magic_link_returns_report_dto() {
        let org_id = Uuid::new_v4();
        let report = make_draft_report(org_id);
        let report_id = report.id;

        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(report.clone())));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));
        let dto = uc.get_via_magic_link(report_id).await.unwrap();
        assert_eq!(dto.id, report_id);
        assert_eq!(dto.contractor_name, "Martin Plomberie SPRL");
    }

    #[tokio::test]
    async fn happy_respond_via_magic_link_applies_fields_then_submits() {
        let org_id = Uuid::new_v4();
        let mut report = make_draft_report(org_id);
        report.compte_rendu = None; // le brouillon n'a encore rien — tout vient du respond
        let report_id = report.id;

        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(report.clone())));
        mock_repo.expect_update().returning(|r| Ok(r.clone()));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));
        let dto = UpdateContractorReportDto {
            work_date: Some(Utc::now()),
            compte_rendu: Some("Remplacement du joint défectueux.".to_string()),
            photos_before: None,
            photos_after: None,
            parts_replaced: None,
        };

        let result = uc.respond_via_magic_link(report_id, dto).await.unwrap();
        assert_eq!(result.status, "submitted");
        assert_eq!(
            result.compte_rendu.as_deref(),
            Some("Remplacement du joint défectueux.")
        );
    }

    // ---- @edge -----------------------------------------------------------------

    #[tokio::test]
    async fn edge_respond_via_magic_link_on_terminal_state_is_rejected() {
        let org_id = Uuid::new_v4();
        let mut report = make_draft_report(org_id);
        report.status = ContractorReportStatus::Validated; // état terminal
        let report_id = report.id;

        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(report.clone())));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));
        let dto = UpdateContractorReportDto {
            work_date: None,
            compte_rendu: Some("Tentative de re-soumission".to_string()),
            photos_before: None,
            photos_after: None,
            parts_replaced: None,
        };

        let err = uc.respond_via_magic_link(report_id, dto).await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ---- @security ---------------------------------------------------------------

    #[tokio::test]
    async fn security_generate_magic_link_rejects_foreign_organization() {
        let org_id = Uuid::new_v4();
        let other_org_id = Uuid::new_v4();
        let report = make_draft_report(org_id);
        let report_id = report.id;

        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(report.clone())));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo))
            .with_magic_link_support(new_magic_link_use_cases());

        let err = uc
            .generate_magic_link(
                report_id,
                other_org_id,
                Uuid::new_v4(),
                "https://app.koprogo.be",
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Forbidden(_)));
    }

    // ---- @negative ---------------------------------------------------------------

    #[tokio::test]
    async fn negative_generate_magic_link_without_support_configured_is_internal_error() {
        let org_id = Uuid::new_v4();
        let report = make_draft_report(org_id);
        let report_id = report.id;

        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(report.clone())));

        // Pas de `.with_magic_link_support(...)` — configuration incomplète.
        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));

        let err = uc
            .generate_magic_link(report_id, org_id, Uuid::new_v4(), "https://app.koprogo.be")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Internal(_)));
    }

    #[tokio::test]
    async fn negative_get_via_magic_link_unknown_report_returns_not_found() {
        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo.expect_find_by_id().returning(|_| Ok(None));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));
        let err = uc.get_via_magic_link(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn negative_respond_via_magic_link_without_compte_rendu_fails_validation() {
        let org_id = Uuid::new_v4();
        let mut report = make_draft_report(org_id);
        report.compte_rendu = None;
        let report_id = report.id;

        let mut mock_repo = MockContractorReportRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(report.clone())));

        let uc = ContractorReportUseCases::new(Arc::new(mock_repo));
        let dto = UpdateContractorReportDto {
            work_date: None,
            compte_rendu: None,
            photos_before: None,
            photos_after: None,
            parts_replaced: None,
        };

        let err = uc.respond_via_magic_link(report_id, dto).await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
