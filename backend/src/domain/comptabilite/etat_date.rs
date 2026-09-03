use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

/// Atomic counter for generating unique reference numbers
static ETAT_DATE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Statut de l'état daté (workflow de génération)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "etat_date_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EtatDateStatus {
    Requested,  // Demandé par le notaire
    InProgress, // En cours de génération
    Generated,  // Généré, prêt à être délivré
    Delivered,  // Délivré au notaire
    Expired,    // Expiré (>3 mois)
}

/// Langue de génération du document (Belgique: FR/NL/DE)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "etat_date_language", rename_all = "snake_case")]
#[serde(rename_all = "lowercase")]
pub enum EtatDateLanguage {
    Fr, // Français
    Nl, // Néerlandais
    De, // Allemand
}

/// Représente un État Daté pour mutation immobilière (Art. 577-2 Code Civil belge)
///
/// Un état daté est un document légal obligatoire pour toute vente de lot en copropriété.
/// Il contient 16 sections légales détaillant la situation financière et juridique du lot.
///
/// **Délai légal**: Art. 3.94 CC — 15 jours CALENDAIRES sur simple demande
/// (§ 1er), 30 jours si le notaire adresse sa demande par recommandé (§ 2).
///
/// Texte officiel (Justel, SPF Justice — base consolidée du Code civil) :
///
/// > § 1er. […] les informations et documents suivants, que le syndic lui
/// > communique **sur simple demande, dans un délai de quinze jours** […]
/// > A défaut de réponse du syndic **dans les quinze jours** de la demande,
/// > le notaire […] avise les parties de la carence de celui-ci.
///
/// > § 2. […] le notaire instrumentant demande au syndic […] **par envoi
/// > recommandé** […] A défaut de réponse du syndic **dans les trente jours**
/// > de la demande, le notaire avise les parties de la carence de celui-ci.
///
/// Ni l'un ni l'autre ne dit « ouvrables ». Et l'argument décisif n'est pas
/// cette absence, c'est que le législateur emploie le terme AILLEURS dans le
/// même livre quand il le veut — Art. 3.31, § 2 : « Le délai […] est prolongé
/// jusqu'au premier **jour ouvrable** suivant lorsque le dernier jour dudit
/// délai est un jour de fermeture des bureaux. » Son absence à l'Art. 3.94
/// est donc un choix, pas un oubli.
///
/// Ce commentaire annonçait « jours ouvrables », soit environ 21 jours
/// calendaires : la documentation était fausse, pas le calcul.
/// **Validité**: 3 mois à partir de la date de référence (pratique professionnelle, non légale)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EtatDate {
    pub id: Uuid,

    /// L'ACP dont l'état daté rend compte.
    ///
    /// Art. 3.94 : à la transmission d'un lot, le notaire réclame au syndic
    /// l'état de la situation du copropriétaire cédant vis-à-vis de
    /// **l'association**. Les sommes dues le sont à l'ACP ; le syndic ne fait
    /// que les attester. Cf. ADR-0045.
    pub acp_id: Uuid,

    /// Le syndic qui a établi l'état daté, conservé comme trace d'auteur.
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Uuid,

    /// Date de référence pour les calculs financiers
    pub reference_date: DateTime<Utc>,

    /// Date de demande par le notaire
    pub requested_date: DateTime<Utc>,

    /// Date de génération du document
    pub generated_date: Option<DateTime<Utc>>,

    /// Date de délivrance au notaire
    pub delivered_date: Option<DateTime<Utc>>,

    /// Statut du workflow
    pub status: EtatDateStatus,

    /// Langue du document
    pub language: EtatDateLanguage,

    /// Numéro de référence unique (ex: "ED-2025-001-BLD123-U456")
    pub reference_number: String,

    /// Informations du notaire demandeur
    pub notary_name: String,
    pub notary_email: String,
    pub notary_phone: Option<String>,

    // === Section 1: Identification ===
    pub building_name: String,
    pub building_address: String,
    pub unit_number: String,
    pub unit_floor: Option<String>,
    pub unit_area: Option<f64>,

    // === Section 2: Quote-parts ===
    /// Quote-part charges ordinaires (en %) — Decimal exact (ADR-0008)
    pub ordinary_charges_quota: Decimal,
    /// Quote-part charges extraordinaires (en %) — Decimal exact (ADR-0008)
    pub extraordinary_charges_quota: Decimal,

    // === Section 3: Situation financière du propriétaire ===
    // MONÉTAIRE : Decimal exact (ADR-0007/0008). Document légal Art. 577-2
    // CC — toute dérive d'arrondi f64 vicie l'état daté. Colonnes DB déjà
    // `DECIMAL(12,2)` (#433 / WP-A5 EXP-007).
    /// Solde du propriétaire (positif = crédit, négatif = débit)
    pub owner_balance: Decimal,
    /// Montant des arriérés (dettes)
    pub arrears_amount: Decimal,

    // === Section 4: Provisions pour charges ===
    /// Montant mensuel des provisions
    pub monthly_provision_amount: Decimal,

    // === Section 5: Solde créditeur/débiteur ===
    /// Solde total (somme de tous les comptes)
    pub total_balance: Decimal,

    // === Section 6: Travaux votés non payés ===
    /// Montant total des travaux votés mais non encore payés
    pub approved_works_unpaid: Decimal,

    // === Section 7-16: Données JSONB ===
    /// Données structurées pour les sections complexes
    /// {
    ///   "ongoing_disputes": [...],           // Section 7: Litiges en cours
    ///   "building_insurance": {...},         // Section 8: Assurance immeuble
    ///   "condo_regulations": {...},          // Section 9: Règlement copropriété
    ///   "recent_meeting_minutes": [...],     // Section 10: PV dernières AG
    ///   "budget": {...},                     // Section 11: Budget prévisionnel
    ///   "reserve_fund": {...},               // Section 12: Fonds de réserve
    ///   "condo_debts_credits": {...},        // Section 13: Dettes/créances copropriété
    ///   "works_progress": [...],             // Section 14: État d'avancement travaux
    ///   "guarantees_mortgages": [...],       // Section 15: Garanties et hypothèques
    ///   "additional_observations": "..."     // Section 16: Observations diverses
    /// }
    pub additional_data: serde_json::Value,

    /// Chemin du fichier PDF généré (si généré)
    pub pdf_file_path: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Domain-typed validation error for États Datés (Art. 577-2 CC).
///
/// Pure domain type — no infra/application dependency (hexagonal purity).
/// Suit le précédent `JournalEntryError` (journal_entry.rs) : l'entité
/// renvoie son erreur typée, l'application la mappe vers `AppError`
/// (#433 / WP-A5 EXP-007) → 400 validation, jamais 500 Internal.
#[derive(Debug, Clone, PartialEq)]
pub enum EtatDateError {
    /// Un champ texte obligatoire est vide (nom/email notaire, immeuble…).
    EmptyField(&'static str),
    /// Email notaire syntaxiquement invalide.
    InvalidNotaryEmail,
    /// Quote-part hors bornes [0, 100] %.
    QuotaOutOfRange(&'static str),
    /// Montant monétaire négatif là où c'est interdit (arriérés, provisions,
    /// travaux non payés ≥ 0).
    NegativeAmount(&'static str),
    /// Transition de workflow interdite depuis le statut courant.
    InvalidTransition {
        from: EtatDateStatus,
        to: &'static str,
    },
    /// Chemin du PDF généré vide.
    EmptyPdfPath,
    /// `additional_data` n'est pas un objet JSON.
    AdditionalDataNotObject,
}

impl std::fmt::Display for EtatDateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField(name) => write!(f, "{} cannot be empty", name),
            Self::InvalidNotaryEmail => write!(f, "Invalid notary email"),
            Self::QuotaOutOfRange(name) => {
                write!(f, "{} must be between 0 and 100%", name)
            }
            Self::NegativeAmount(name) => write!(f, "{} cannot be negative", name),
            Self::InvalidTransition { from, to } => {
                write!(f, "Cannot mark as {}: current status is {:?}", to, from)
            }
            Self::EmptyPdfPath => write!(f, "PDF file path cannot be empty"),
            Self::AdditionalDataNotObject => {
                write!(f, "Additional data must be a JSON object")
            }
        }
    }
}

impl std::error::Error for EtatDateError {}

/// Bridge : les use-cases/ports `Result<_, String>` compilent inchangés
/// pendant que l'entité est typée (cascade String→AppError = slice plus
/// large, hors scope WP-A5 — précédent WP-A3/A4). Pur, std-only.
impl From<EtatDateError> for String {
    fn from(e: EtatDateError) -> String {
        e.to_string()
    }
}

impl EtatDate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        acp_id: Uuid,
        organization_id: Uuid,
        building_id: Uuid,
        unit_id: Uuid,
        reference_date: DateTime<Utc>,
        language: EtatDateLanguage,
        notary_name: String,
        notary_email: String,
        notary_phone: Option<String>,
        building_name: String,
        building_address: String,
        unit_number: String,
        unit_floor: Option<String>,
        unit_area: Option<f64>,
        ordinary_charges_quota: Decimal,
        extraordinary_charges_quota: Decimal,
    ) -> Result<Self, EtatDateError> {
        // Validations
        if notary_name.trim().is_empty() {
            return Err(EtatDateError::EmptyField("Notary name"));
        }
        if notary_email.trim().is_empty() {
            return Err(EtatDateError::EmptyField("Notary email"));
        }
        if !notary_email.contains('@') {
            return Err(EtatDateError::InvalidNotaryEmail);
        }
        if building_name.trim().is_empty() {
            return Err(EtatDateError::EmptyField("Building name"));
        }
        if building_address.trim().is_empty() {
            return Err(EtatDateError::EmptyField("Building address"));
        }
        if unit_number.trim().is_empty() {
            return Err(EtatDateError::EmptyField("Unit number"));
        }

        // Quote-parts doivent être entre 0 et 100%
        if ordinary_charges_quota < Decimal::ZERO || ordinary_charges_quota > dec!(100) {
            return Err(EtatDateError::QuotaOutOfRange("Ordinary charges quota"));
        }
        if extraordinary_charges_quota < Decimal::ZERO || extraordinary_charges_quota > dec!(100) {
            return Err(EtatDateError::QuotaOutOfRange(
                "Extraordinary charges quota",
            ));
        }

        let now = Utc::now();
        let reference_number = Self::generate_reference_number(&building_id, &unit_id, &now);

        Ok(Self {
            id: Uuid::new_v4(),
            acp_id,
            organization_id,
            building_id,
            unit_id,
            reference_date,
            requested_date: now,
            generated_date: None,
            delivered_date: None,
            status: EtatDateStatus::Requested,
            language,
            reference_number,
            notary_name,
            notary_email,
            notary_phone,
            building_name,
            building_address,
            unit_number,
            unit_floor,
            unit_area,
            ordinary_charges_quota,
            extraordinary_charges_quota,
            owner_balance: Decimal::ZERO,
            arrears_amount: Decimal::ZERO,
            monthly_provision_amount: Decimal::ZERO,
            total_balance: Decimal::ZERO,
            approved_works_unpaid: Decimal::ZERO,
            additional_data: serde_json::json!({}),
            pdf_file_path: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Génère un numéro de référence unique
    /// Format: ED-YYYY-NNN-BLD{building_id_short}-U{unit_id_short}
    fn generate_reference_number(
        building_id: &Uuid,
        unit_id: &Uuid,
        date: &DateTime<Utc>,
    ) -> String {
        let year = date.format("%Y");
        let building_short = &building_id.to_string()[..8];
        let unit_short = &unit_id.to_string()[..8];

        // Use atomic counter + timestamp for guaranteed uniqueness
        let seq = ETAT_DATE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique_id = &Uuid::new_v4().to_string()[..8];

        format!(
            "ED-{}-{:03}-{}-BLD{}-U{}",
            year,
            seq % 1000,
            unique_id,
            building_short,
            unit_short
        )
    }

    /// Marque l'état daté comme en cours de génération
    pub fn mark_in_progress(&mut self) -> Result<(), EtatDateError> {
        match self.status {
            EtatDateStatus::Requested => {
                self.status = EtatDateStatus::InProgress;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(EtatDateError::InvalidTransition {
                from: self.status.clone(),
                to: "in progress",
            }),
        }
    }

    /// Marque l'état daté comme généré
    pub fn mark_generated(&mut self, pdf_file_path: String) -> Result<(), EtatDateError> {
        if pdf_file_path.trim().is_empty() {
            return Err(EtatDateError::EmptyPdfPath);
        }

        match self.status {
            EtatDateStatus::InProgress => {
                self.status = EtatDateStatus::Generated;
                self.generated_date = Some(Utc::now());
                self.pdf_file_path = Some(pdf_file_path);
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(EtatDateError::InvalidTransition {
                from: self.status.clone(),
                to: "generated",
            }),
        }
    }

    /// Marque l'état daté comme délivré au notaire
    pub fn mark_delivered(&mut self) -> Result<(), EtatDateError> {
        match self.status {
            EtatDateStatus::Generated => {
                self.status = EtatDateStatus::Delivered;
                self.delivered_date = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(EtatDateError::InvalidTransition {
                from: self.status.clone(),
                to: "delivered",
            }),
        }
    }

    /// Vérifie si l'état daté est expiré (>3 mois depuis la date de référence)
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expiration_date = self.reference_date + chrono::Duration::days(90); // 3 mois
        now > expiration_date
    }

    /// Vérifie si la génération est en retard.
    ///
    /// Art. 3.94 § 1er CC : le syndic répond « sur simple demande endéans les
    /// quinze jours » — quinze jours CALENDAIRES. Le § 2 porte le délai à
    /// trente jours lorsque le notaire adresse sa demande par lettre
    /// recommandée.
    ///
    /// LIMITE ASSUMÉE : l'entité ne mémorise pas le CANAL de la demande
    /// (simple ou recommandé), seulement l'identité du notaire. Le délai le
    /// plus court est donc appliqué à tous les cas. C'est le sens prudent —
    /// on n'annonce jamais un retard trop tard — mais un syndic qui répond au
    /// vingtième jour à une demande recommandée sera signalé en retard alors
    /// qu'il est dans les temps. Ajouter le canal à la demande lèverait la
    /// restriction.
    pub fn is_overdue(&self) -> bool {
        if matches!(
            self.status,
            EtatDateStatus::Generated | EtatDateStatus::Delivered
        ) {
            return false; // Déjà généré ou délivré
        }

        let now = Utc::now();
        let deadline = self.requested_date + chrono::Duration::days(15);
        now > deadline
    }

    /// Calcule le nombre de jours depuis la demande
    pub fn days_since_request(&self) -> i64 {
        let now = Utc::now();
        (now - self.requested_date).num_days()
    }

    /// Met à jour les données financières
    pub fn update_financial_data(
        &mut self,
        owner_balance: Decimal,
        arrears_amount: Decimal,
        monthly_provision_amount: Decimal,
        total_balance: Decimal,
        approved_works_unpaid: Decimal,
    ) -> Result<(), EtatDateError> {
        // Validation: les arriérés ne peuvent pas être négatifs
        if arrears_amount < Decimal::ZERO {
            return Err(EtatDateError::NegativeAmount("Arrears amount"));
        }
        if monthly_provision_amount < Decimal::ZERO {
            return Err(EtatDateError::NegativeAmount("Monthly provision amount"));
        }
        if approved_works_unpaid < Decimal::ZERO {
            return Err(EtatDateError::NegativeAmount("Approved works unpaid"));
        }

        self.owner_balance = owner_balance;
        self.arrears_amount = arrears_amount;
        self.monthly_provision_amount = monthly_provision_amount;
        self.total_balance = total_balance;
        self.approved_works_unpaid = approved_works_unpaid;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Met à jour les données additionnelles (sections 7-16)
    pub fn update_additional_data(&mut self, data: serde_json::Value) -> Result<(), EtatDateError> {
        if !data.is_object() {
            return Err(EtatDateError::AdditionalDataNotObject);
        }

        self.additional_data = data;
        self.updated_at = Utc::now();
        Ok(())
    }
}

impl crate::domain::services::PieceDeGestion for EtatDate {
    fn acp_id(&self) -> Uuid {
        self.acp_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_etat_date_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let etat_date = EtatDate::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            unit_id,
            ref_date,
            EtatDateLanguage::Fr,
            "Maître Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            Some("+32 2 123 4567".to_string()),
            "Résidence Les Jardins".to_string(),
            "Rue de la Loi 123, 1000 Bruxelles".to_string(),
            "101".to_string(),
            Some("1".to_string()),
            Some(100.0),
            dec!(100), // 5%
            dec!(100), // 10%
        );

        assert!(etat_date.is_ok());
        let ed = etat_date.unwrap();
        assert_eq!(ed.status, EtatDateStatus::Requested);
        assert_eq!(ed.notary_name, "Maître Dupont");
        assert!(ed.reference_number.starts_with("ED-"));
    }

    #[test]
    fn test_create_etat_date_invalid_email() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let result = EtatDate::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            unit_id,
            ref_date,
            EtatDateLanguage::Fr,
            "Maître Dupont".to_string(),
            "invalid-email".to_string(), // Email invalide
            None,
            "Résidence Les Jardins".to_string(),
            "Rue de la Loi 123".to_string(),
            "101".to_string(),
            None,
            None,
            dec!(100),
            dec!(100),
        );

        assert!(matches!(
            result.unwrap_err(),
            EtatDateError::InvalidNotaryEmail
        ));
    }

    #[test]
    fn test_create_etat_date_invalid_quota() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let result = EtatDate::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            unit_id,
            ref_date,
            EtatDateLanguage::Fr,
            "Maître Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            None,
            "Résidence Les Jardins".to_string(),
            "Rue de la Loi 123".to_string(),
            "101".to_string(),
            None,
            None,
            dec!(150), // 150% - invalide
            dec!(100),
        );

        assert!(matches!(
            result.unwrap_err(),
            EtatDateError::QuotaOutOfRange(_)
        ));
    }

    #[test]
    fn test_workflow_transitions() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let mut ed = EtatDate::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            unit_id,
            ref_date,
            EtatDateLanguage::Fr,
            "Maître Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            None,
            "Résidence Les Jardins".to_string(),
            "Rue de la Loi 123".to_string(),
            "101".to_string(),
            None,
            None,
            dec!(100),
            dec!(100),
        )
        .unwrap();

        // Requested → InProgress
        assert!(ed.mark_in_progress().is_ok());
        assert_eq!(ed.status, EtatDateStatus::InProgress);

        // InProgress → Generated
        assert!(ed
            .mark_generated("/path/to/etat_date_001.pdf".to_string())
            .is_ok());
        assert_eq!(ed.status, EtatDateStatus::Generated);
        assert!(ed.generated_date.is_some());
        assert!(ed.pdf_file_path.is_some());

        // Generated → Delivered
        assert!(ed.mark_delivered().is_ok());
        assert_eq!(ed.status, EtatDateStatus::Delivered);
        assert!(ed.delivered_date.is_some());
    }

    #[test]
    fn test_invalid_workflow_transition() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let mut ed = EtatDate::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            unit_id,
            ref_date,
            EtatDateLanguage::Fr,
            "Maître Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            None,
            "Résidence Les Jardins".to_string(),
            "Rue de la Loi 123".to_string(),
            "101".to_string(),
            None,
            None,
            dec!(100),
            dec!(100),
        )
        .unwrap();

        // Cannot go directly from Requested to Delivered
        let result = ed.mark_delivered();
        assert!(result.is_err());
    }

    #[test]
    fn test_update_financial_data() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let mut ed = EtatDate::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            unit_id,
            ref_date,
            EtatDateLanguage::Fr,
            "Maître Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            None,
            "Résidence Les Jardins".to_string(),
            "Rue de la Loi 123".to_string(),
            "101".to_string(),
            None,
            None,
            dec!(100),
            dec!(100),
        )
        .unwrap();

        let result = ed.update_financial_data(
            dec!(-500.00), // débit
            dec!(100.0),   // arriérés
            dec!(100.0),   // provision/mois
            dec!(-500.00), // total
            dec!(100.0),   // travaux votés non payés
        );

        assert!(result.is_ok());
        assert_eq!(ed.owner_balance, dec!(-500.00));
        assert_eq!(ed.arrears_amount, dec!(100.0));
    }

    #[test]
    fn test_is_overdue() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let mut ed = EtatDate::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            unit_id,
            ref_date,
            EtatDateLanguage::Fr,
            "Maître Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            None,
            "Résidence Les Jardins".to_string(),
            "Rue de la Loi 123".to_string(),
            "101".to_string(),
            None,
            None,
            dec!(100),
            dec!(100),
        )
        .unwrap();

        // Simuler une demande vieille de 16 jours (>15 jours Art. 3.94 CC)
        ed.requested_date = Utc::now() - chrono::Duration::days(16);

        assert!(ed.is_overdue());
    }

    /// Le délai de l'Art. 3.94 § 1er CC se compte en jours CALENDAIRES.
    ///
    /// Ce test existe parce que la documentation de l'entité annonçait
    /// « 15 jours ouvrables », soit environ 21 jours calendaires — le calcul
    /// et sa description se contredisaient, et une seule des deux lectures
    /// pouvait être juridiquement correcte.
    ///
    /// Tranché le 2026-09-02 sur le texte officiel (Justel, SPF Justice) :
    ///
    ///   « le syndic lui communique sur simple demande, dans un délai de
    ///     quinze jours »  —  Art. 3.94, § 1er
    ///
    /// Ni « ouvrables », ni « werkdagen » dans la version néerlandaise
    /// (« binnen een termijn van vijftien dagen »), qui fait également foi.
    ///
    /// L'argument décisif n'est pas l'absence du mot mais son emploi AILLEURS :
    /// l'Art. 3.31, § 2 du même livre écrit « prolongé jusqu'au premier jour
    /// ouvrable suivant ». Le législateur sait donc le dire quand il le veut ;
    /// son silence à l'Art. 3.94 est un choix, pas un oubli.
    ///
    /// Le calcul était juste, la documentation fausse.
    ///
    /// Les bornes ci-dessous verrouillent cette lecture : au quatorzième jour
    /// on est dans les temps, au seizième on ne l'est plus. Si quelqu'un
    /// bascule un jour sur les jours ouvrables, ce test le forcera à
    /// argumenter plutôt qu'à le faire en passant.
    #[test]
    fn test_delai_art_3_94_se_compte_en_jours_calendaires() {
        let ed_neuf = || {
            EtatDate::new(
                Uuid::new_v4(), // acp_id
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                Utc::now(),
                EtatDateLanguage::Fr,
                "Maître Dupont".to_string(),
                "dupont@notaire.be".to_string(),
                None,
                "Résidence Les Jardins".to_string(),
                "Rue de la Loi 123".to_string(),
                "101".to_string(),
                None,
                None,
                dec!(100),
                dec!(100),
            )
            .unwrap()
        };

        // 14 jours calendaires : dans les temps.
        let mut avant = ed_neuf();
        avant.requested_date = Utc::now() - chrono::Duration::days(14);
        assert!(
            !avant.is_overdue(),
            "quatorze jours calendaires restent dans le délai légal"
        );

        // 16 jours calendaires : hors délai.
        let mut apres = ed_neuf();
        apres.requested_date = Utc::now() - chrono::Duration::days(16);
        assert!(
            apres.is_overdue(),
            "seize jours calendaires dépassent le délai légal"
        );

        // 18 jours : encore dans les temps SI l'on comptait en jours
        // ouvrables (≈ 21 jours calendaires pour 15 ouvrables). Le test
        // affirme le contraire — c'est là que se joue la différence.
        let mut ouvrables = ed_neuf();
        ouvrables.requested_date = Utc::now() - chrono::Duration::days(18);
        assert!(
            ouvrables.is_overdue(),
            "le délai se compte en jours calendaires, pas en jours ouvrables"
        );
    }

    #[test]
    fn test_days_since_request() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let mut ed = EtatDate::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            unit_id,
            ref_date,
            EtatDateLanguage::Fr,
            "Maître Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            None,
            "Résidence Les Jardins".to_string(),
            "Rue de la Loi 123".to_string(),
            "101".to_string(),
            None,
            None,
            dec!(100),
            dec!(100),
        )
        .unwrap();

        // Simuler une demande vieille de 5 jours
        ed.requested_date = Utc::now() - chrono::Duration::days(5);

        assert_eq!(ed.days_since_request(), 5);
    }

    // ------------------------------------------------------------------------
    // 4 catégories #433/WP-A5 EXP-007 — erreur typée + exactitude Decimal
    // (CRITICAL.md #3). Document légal Art. 577-2 CC : zéro dérive d'arrondi.
    // ------------------------------------------------------------------------

    fn sample() -> EtatDate {
        EtatDate::new(
            Uuid::new_v4(), // acp_id
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now(),
            EtatDateLanguage::Fr,
            "Maître Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            None,
            "Résidence Les Jardins".to_string(),
            "Rue de la Loi 123".to_string(),
            "101".to_string(),
            None,
            None,
            dec!(50),
            dec!(50),
        )
        .unwrap()
    }

    /// @happy — màj financière nominale : montants Decimal stockés exacts.
    #[test]
    fn happy_update_financial_data_decimal_exact() {
        let mut ed = sample();
        ed.update_financial_data(
            dec!(-1234.56),
            dec!(789.01),
            dec!(150.00),
            dec!(-445.55),
            dec!(2000.00),
        )
        .unwrap();
        assert_eq!(ed.owner_balance, dec!(-1234.56));
        assert_eq!(ed.total_balance, dec!(-445.55));
    }

    /// @edge — exactitude Decimal sur cumul (0.1+0.2=0.3 ; f64 échoue) +
    /// borne quota exactement 100% acceptée.
    #[test]
    fn edge_decimal_exactness_and_quota_boundary() {
        let mut ed = sample();
        ed.update_financial_data(
            dec!(0.1) + dec!(0.2),
            Decimal::ZERO,
            Decimal::ZERO,
            dec!(0.3),
            Decimal::ZERO,
        )
        .unwrap();
        assert_eq!(ed.owner_balance, dec!(0.3));
        assert_eq!(ed.owner_balance, ed.total_balance);

        // Quota exactement 100% (borne incluse) accepté.
        let ok = EtatDate::new(
            Uuid::new_v4(), // acp_id
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now(),
            EtatDateLanguage::Nl,
            "N".to_string(),
            "n@x.be".to_string(),
            None,
            "B".to_string(),
            "A".to_string(),
            "1".to_string(),
            None,
            None,
            dec!(100),
            dec!(0),
        );
        assert!(ok.is_ok());
    }

    /// @negative — montant interdit négatif & transition invalide rejetés
    /// (erreur typée, pas de panic).
    #[test]
    fn negative_amount_and_transition_rejected() {
        let mut ed = sample();
        assert!(matches!(
            ed.update_financial_data(
                Decimal::ZERO,
                dec!(-1), // arriérés négatifs interdits
                Decimal::ZERO,
                Decimal::ZERO,
                Decimal::ZERO,
            )
            .unwrap_err(),
            EtatDateError::NegativeAmount(_)
        ));

        // Requested -> Delivered direct interdit.
        assert!(matches!(
            ed.mark_delivered().unwrap_err(),
            EtatDateError::InvalidTransition { .. }
        ));
    }

    /// @security — un état daté (acte légal Art. 577-2 CC) ne peut être créé
    /// avec une quote-part falsifiée hors [0,100] % : intégrité du document
    /// opposable au notaire/acquéreur.
    #[test]
    fn security_tampered_quota_rejected() {
        let result = EtatDate::new(
            Uuid::new_v4(), // acp_id
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now(),
            EtatDateLanguage::Fr,
            "Maître Dupont".to_string(),
            "dupont@notaire.be".to_string(),
            None,
            "Résidence".to_string(),
            "Rue".to_string(),
            "101".to_string(),
            None,
            None,
            dec!(250), // falsifié > 100%
            dec!(50),
        );
        assert!(matches!(
            result.unwrap_err(),
            EtatDateError::QuotaOutOfRange(_)
        ));
    }
}
