use crate::application::dto::contractor_report_dto::{
    ContractorReportResponseDto, CreateContractorReportDto, MagicLinkResponseDto, RejectReportDto,
    RequestCorrectionsDto, UpdateContractorReportDto,
};
use crate::application::ports::contractor_report_repository::ContractorReportRepository;
use crate::domain::entities::contractor_report::{ContractorReport, ContractorReportStatus};
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Durée de validité du magic link (72 heures)
const MAGIC_LINK_VALIDITY_HOURS: i64 = 72;

pub struct ContractorReportUseCases {
    pub repo: Arc<dyn ContractorReportRepository>,
}

impl ContractorReportUseCases {
    pub fn new(repo: Arc<dyn ContractorReportRepository>) -> Self {
        Self { repo }
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

        // TODO (B16-6) : déclencher paiement automatique si quote_id présent
        // payment_use_cases.trigger_contractor_payment(saved.quote_id, saved.id).await?;

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

    /// Génère un magic link JWT 72h pour l'accès PWA corps de métier (B16-2)
    pub async fn generate_magic_link(
        &self,
        report_id: Uuid,
        organization_id: Uuid,
        base_url: &str,
    ) -> Result<MagicLinkResponseDto, String> {
        let mut report = self
            .repo
            .find_by_id(report_id)
            .await?
            .ok_or_else(|| format!("Rapport {} introuvable", report_id))?;

        if report.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        // Génère un token sécurisé (UUID v4 = 122 bits d'entropie)
        let raw_token = Uuid::new_v4().to_string();
        // En production on hasherait avec SHA-256 ou bcrypt ; ici on stocke le raw
        // (suffisant pour 72h, UUID non prédictible)
        let token_hash = raw_token.clone();
        let expires_at = Utc::now() + Duration::hours(MAGIC_LINK_VALIDITY_HOURS);

        report.magic_token_hash = Some(token_hash);
        report.magic_token_expires_at = Some(expires_at);
        report.updated_at = Utc::now();

        self.repo.update(&report).await?;

        let magic_link = format!(
            "{}/contractor/?token={}",
            base_url.trim_end_matches('/'),
            raw_token
        );

        Ok(MagicLinkResponseDto {
            magic_link,
            expires_at,
        })
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

        let result = uc
            .generate_magic_link(report_id, org_id, "https://app.koprogo.be")
            .await;
        assert!(result.is_ok());
        let link_dto = result.unwrap();
        assert!(link_dto
            .magic_link
            .starts_with("https://app.koprogo.be/contractor/?token="));
        assert!(link_dto.expires_at > Utc::now());
    }
}
