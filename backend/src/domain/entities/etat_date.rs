use chrono::{DateTime, Utc};
use f64;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Statut de l'état daté (workflow de génération)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
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
/// **Délai légal**: Maximum 15 jours pour génération (rappels si > 10j)
/// **Validité**: 3 mois à partir de la date de référence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EtatDate {
    pub id: Uuid,
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
    /// Quote-part charges ordinaires (en %)
    pub ordinary_charges_quota: f64,
    /// Quote-part charges extraordinaires (en %)
    pub extraordinary_charges_quota: f64,

    // === Section 3: Situation financière du propriétaire ===
    /// Solde du propriétaire (positif = crédit, négatif = débit)
    pub owner_balance: f64,
    /// Montant des arriérés (dettes)
    pub arrears_amount: f64,

    // === Section 4: Provisions pour charges ===
    /// Montant mensuel des provisions
    pub monthly_provision_amount: f64,

    // === Section 5: Solde créditeur/débiteur ===
    /// Solde total (somme de tous les comptes)
    pub total_balance: f64,

    // === Section 6: Travaux votés non payés ===
    /// Montant total des travaux votés mais non encore payés
    pub approved_works_unpaid: f64,

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

impl EtatDate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
        ordinary_charges_quota: f64,
        extraordinary_charges_quota: f64,
    ) -> Result<Self, String> {
        // Validations
        if notary_name.trim().is_empty() {
            return Err("Notary name cannot be empty".to_string());
        }
        if notary_email.trim().is_empty() {
            return Err("Notary email cannot be empty".to_string());
        }
        if !notary_email.contains('@') {
            return Err("Invalid notary email".to_string());
        }
        if building_name.trim().is_empty() {
            return Err("Building name cannot be empty".to_string());
        }
        if building_address.trim().is_empty() {
            return Err("Building address cannot be empty".to_string());
        }
        if unit_number.trim().is_empty() {
            return Err("Unit number cannot be empty".to_string());
        }

        // Quote-parts doivent être entre 0 et 100%
        if ordinary_charges_quota < 0.0 || ordinary_charges_quota > 100.0 {
            return Err("Ordinary charges quota must be between 0 and 100%".to_string());
        }
        if extraordinary_charges_quota < 0.0 || extraordinary_charges_quota > 100.0 {
            return Err("Extraordinary charges quota must be between 0 and 100%".to_string());
        }

        let now = Utc::now();
        let reference_number = Self::generate_reference_number(&building_id, &unit_id, &now);

        Ok(Self {
            id: Uuid::new_v4(),
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
            owner_balance: 0.0,
            arrears_amount: 0.0,
            monthly_provision_amount: 0.0,
            total_balance: 0.0,
            approved_works_unpaid: 0.0,
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

        // Le compteur (NNN) devrait idéalement venir de la DB, mais pour simplifier on utilise un timestamp
        let counter = date.timestamp() % 1000;

        format!(
            "ED-{}-{:03}-BLD{}-U{}",
            year, counter, building_short, unit_short
        )
    }

    /// Marque l'état daté comme en cours de génération
    pub fn mark_in_progress(&mut self) -> Result<(), String> {
        match self.status {
            EtatDateStatus::Requested => {
                self.status = EtatDateStatus::InProgress;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot mark as in progress: current status is {:?}",
                self.status
            )),
        }
    }

    /// Marque l'état daté comme généré
    pub fn mark_generated(&mut self, pdf_file_path: String) -> Result<(), String> {
        if pdf_file_path.trim().is_empty() {
            return Err("PDF file path cannot be empty".to_string());
        }

        match self.status {
            EtatDateStatus::InProgress => {
                self.status = EtatDateStatus::Generated;
                self.generated_date = Some(Utc::now());
                self.pdf_file_path = Some(pdf_file_path);
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot mark as generated: current status is {:?}",
                self.status
            )),
        }
    }

    /// Marque l'état daté comme délivré au notaire
    pub fn mark_delivered(&mut self) -> Result<(), String> {
        match self.status {
            EtatDateStatus::Generated => {
                self.status = EtatDateStatus::Delivered;
                self.delivered_date = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot mark as delivered: current status is {:?}",
                self.status
            )),
        }
    }

    /// Vérifie si l'état daté est expiré (>3 mois depuis la date de référence)
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expiration_date = self.reference_date + chrono::Duration::days(90); // 3 mois
        now > expiration_date
    }

    /// Vérifie si la génération est en retard (>10 jours depuis la demande)
    pub fn is_overdue(&self) -> bool {
        if matches!(
            self.status,
            EtatDateStatus::Generated | EtatDateStatus::Delivered
        ) {
            return false; // Déjà généré ou délivré
        }

        let now = Utc::now();
        let deadline = self.requested_date + chrono::Duration::days(10);
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
        owner_balance: f64,
        arrears_amount: f64,
        monthly_provision_amount: f64,
        total_balance: f64,
        approved_works_unpaid: f64,
    ) -> Result<(), String> {
        // Validation: les arriérés ne peuvent pas être négatifs
        if arrears_amount < 0.0 {
            return Err("Arrears amount cannot be negative".to_string());
        }
        if monthly_provision_amount < 0.0 {
            return Err("Monthly provision amount cannot be negative".to_string());
        }
        if approved_works_unpaid < 0.0 {
            return Err("Approved works unpaid cannot be negative".to_string());
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
    pub fn update_additional_data(&mut self, data: serde_json::Value) -> Result<(), String> {
        if !data.is_object() {
            return Err("Additional data must be a JSON object".to_string());
        }

        self.additional_data = data;
        self.updated_at = Utc::now();
        Ok(())
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
            100.0, // 5%
            100.0, // 10%
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
            100.0,
            100.0,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid notary email");
    }

    #[test]
    fn test_create_etat_date_invalid_quota() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let result = EtatDate::new(
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
            150.0, // 150% - invalide
            100.0,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 0 and 100%"));
    }

    #[test]
    fn test_workflow_transitions() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let mut ed = EtatDate::new(
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
            100.0,
            100.0,
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
            100.0,
            100.0,
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
            100.0,
            100.0,
        )
        .unwrap();

        let result = ed.update_financial_data(
            -500.00, // -500.00 EUR (débit)
            100.0,   // 500.00 EUR arriérés
            100.0,   // 150.00 EUR/mois
            -500.00, // -500.00 EUR total
            100.0,   // 2000.00 EUR travaux votés
        );

        assert!(result.is_ok());
        assert_eq!(ed.owner_balance, -500.00);
        assert_eq!(ed.arrears_amount, 100.0);
    }

    #[test]
    fn test_is_overdue() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let mut ed = EtatDate::new(
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
            100.0,
            100.0,
        )
        .unwrap();

        // Simuler une demande vieille de 11 jours
        ed.requested_date = Utc::now() - chrono::Duration::days(11);

        assert!(ed.is_overdue());
    }

    #[test]
    fn test_days_since_request() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let ref_date = Utc::now();

        let mut ed = EtatDate::new(
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
            100.0,
            100.0,
        )
        .unwrap();

        // Simuler une demande vieille de 5 jours
        ed.requested_date = Utc::now() - chrono::Duration::days(5);

        assert_eq!(ed.days_since_request(), 5);
    }
}
