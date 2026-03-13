use crate::domain::entities::contractor_report::{ContractorReport, ReplacedPart};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Création d'un rapport par le corps de métier (via magic link ou syndic)
#[derive(Debug, Deserialize)]
pub struct CreateContractorReportDto {
    pub building_id: Uuid,
    pub contractor_name: String,
    pub ticket_id: Option<Uuid>,
    pub quote_id: Option<Uuid>,
    pub contractor_user_id: Option<Uuid>,
}

/// Mise à jour du brouillon (photos, pièces, compte-rendu)
#[derive(Debug, Deserialize)]
pub struct UpdateContractorReportDto {
    pub work_date: Option<DateTime<Utc>>,
    pub compte_rendu: Option<String>,
    pub photos_before: Option<Vec<Uuid>>,
    pub photos_after: Option<Vec<Uuid>>,
    pub parts_replaced: Option<Vec<ReplacedPartDto>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReplacedPartDto {
    pub name: String,
    pub reference: Option<String>,
    pub quantity: u32,
    pub photo_document_id: Option<Uuid>,
}

impl From<ReplacedPartDto> for ReplacedPart {
    fn from(d: ReplacedPartDto) -> Self {
        ReplacedPart {
            name: d.name,
            reference: d.reference,
            quantity: d.quantity,
            photo_document_id: d.photo_document_id,
        }
    }
}

impl From<&ReplacedPart> for ReplacedPartDto {
    fn from(p: &ReplacedPart) -> Self {
        ReplacedPartDto {
            name: p.name.clone(),
            reference: p.reference.clone(),
            quantity: p.quantity,
            photo_document_id: p.photo_document_id,
        }
    }
}

/// Demande de corrections par le CdC
#[derive(Debug, Deserialize)]
pub struct RequestCorrectionsDto {
    pub comments: String,
}

/// Rejet par le CdC
#[derive(Debug, Deserialize)]
pub struct RejectReportDto {
    pub comments: String,
}

/// Génération du magic link (syndic → corps de métier)
#[derive(Debug, Deserialize)]
pub struct GenerateMagicLinkDto {
    pub report_id: Uuid,
}

/// Réponse magic link
#[derive(Debug, Serialize)]
pub struct MagicLinkResponseDto {
    pub magic_link: String,
    pub expires_at: DateTime<Utc>,
}

/// Réponse complète d'un rapport de travaux
#[derive(Debug, Serialize)]
pub struct ContractorReportResponseDto {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub ticket_id: Option<Uuid>,
    pub quote_id: Option<Uuid>,
    pub contractor_user_id: Option<Uuid>,
    pub contractor_name: String,
    pub work_date: Option<DateTime<Utc>>,
    pub compte_rendu: Option<String>,
    pub photos_before: Vec<Uuid>,
    pub photos_after: Vec<Uuid>,
    pub parts_replaced: Vec<ReplacedPartDto>,
    pub status: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub validated_at: Option<DateTime<Utc>>,
    pub validated_by: Option<Uuid>,
    pub review_comments: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&ContractorReport> for ContractorReportResponseDto {
    fn from(r: &ContractorReport) -> Self {
        ContractorReportResponseDto {
            id: r.id,
            organization_id: r.organization_id,
            building_id: r.building_id,
            ticket_id: r.ticket_id,
            quote_id: r.quote_id,
            contractor_user_id: r.contractor_user_id,
            contractor_name: r.contractor_name.clone(),
            work_date: r.work_date,
            compte_rendu: r.compte_rendu.clone(),
            photos_before: r.photos_before.clone(),
            photos_after: r.photos_after.clone(),
            parts_replaced: r.parts_replaced.iter().map(ReplacedPartDto::from).collect(),
            status: r.status.to_db_str().to_string(),
            submitted_at: r.submitted_at,
            validated_at: r.validated_at,
            validated_by: r.validated_by,
            review_comments: r.review_comments.clone(),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
