// Domain Entity: Owner Contribution
//
// Represents payments made BY owners TO the ACP (incoming money = revenue)
// Complements Expense entity which represents payments made BY ACP TO suppliers (outgoing money = charges)
//
// Maps to PCMN classe 7 (Produits/Revenue)

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of owner contribution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ContributionType {
    /// Regular quarterly fees (appels de fonds ordinaires)
    Regular,
    /// Extraordinary fees for special works (appels de fonds extraordinaires)
    Extraordinary,
    /// Advance payment
    Advance,
    /// Adjustment (regularisation)
    Adjustment,
}

/// Payment status for contributions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ContributionPaymentStatus {
    /// Not yet paid
    Pending,
    /// Fully paid
    Paid,
    /// Partially paid
    Partial,
    /// Cancelled
    Cancelled,
}

/// Payment method for contributions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContributionPaymentMethod {
    /// Bank transfer (virement)
    BankTransfer,
    /// Cash (espèces)
    Cash,
    /// Check (chèque)
    Check,
    /// Direct debit (domiciliation)
    Domiciliation,
}

/// Traduction du moyen de paiement TECHNIQUE (module de paiement) vers le
/// moyen COMPTABLE inscrit sur la quote-part.
///
/// Les deux enumerations ne se recouvrent pas : le module de paiement raisonne
/// en canal d'encaissement (`Card`, `SepaDebit`, `BankTransfer`, `Cash`), la
/// comptabilite de copropriete en mode de reglement (virement, especes,
/// cheque, domiciliation). La correspondance est donc etablie ici, une seule
/// fois, plutot que devinee a chaque appel.
///
/// Deux points assumes :
///   - `Card` -> `BankTransfer`, faute de valeur « carte » cote comptable :
///     un paiement par carte arrive sur le compte de l'ACP sous forme de
///     virement du prestataire.
///   - `SepaDebit` -> `Domiciliation`, qui est exactement la meme chose sous
///     son nom belge.
///
/// Si la distinction devenait necessaire (rapprochement bancaire fin), c'est
/// `ContributionPaymentMethod` qu'il faudrait etendre, pas cette conversion
/// qu'il faudrait contourner. Le `match` est EXHAUSTIF sans bras `_` : ajouter
/// un canal d'encaissement doit forcer a decider de sa traduction comptable,
/// pas le laisser tomber silencieusement dans un defaut.
impl From<crate::domain::entities::PaymentMethodType> for ContributionPaymentMethod {
    fn from(value: crate::domain::entities::PaymentMethodType) -> Self {
        use crate::domain::entities::PaymentMethodType;
        match value {
            PaymentMethodType::SepaDebit => ContributionPaymentMethod::Domiciliation,
            PaymentMethodType::BankTransfer => ContributionPaymentMethod::BankTransfer,
            PaymentMethodType::Cash => ContributionPaymentMethod::Cash,
            PaymentMethodType::Card => ContributionPaymentMethod::BankTransfer,
        }
    }
}

/// Owner contribution (appel de fonds / cotisation)
///
/// Represents money paid BY owners TO the ACP (REVENUE - classe 7 PCMN)
/// This is the opposite of Expense which represents money paid BY ACP TO suppliers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OwnerContribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub owner_id: Uuid,
    pub unit_id: Option<Uuid>,

    // Financial details
    pub description: String,
    pub amount: Decimal,

    // Accounting
    /// PCMN code (classe 7 - Produits)
    /// Examples: "7000" = regular fees, "7100" = extraordinary fees
    pub account_code: Option<String>,

    // Contribution details
    pub contribution_type: ContributionType,

    // Dates
    pub contribution_date: DateTime<Utc>, // When due/requested
    pub payment_date: Option<DateTime<Utc>>, // When actually paid

    // Payment details
    pub payment_method: Option<ContributionPaymentMethod>,
    pub payment_reference: Option<String>,

    // Status
    pub payment_status: ContributionPaymentStatus,

    // Link to collective call for funds (if generated from CallForFunds)
    pub call_for_funds_id: Option<Uuid>,

    // Metadata
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// Domain-typed validation error for owner contributions (PCMN classe 7).
///
/// Pure domain type — no infra/application dependency (hexagonal purity).
/// Précédent `JournalEntryError`/`ChargeDistributionError` : l'entité
/// renvoie son erreur typée, l'application la mappe vers `AppError`
/// (#433 / WP-A6 EXP-008) → 400 validation, jamais 500 Internal.
#[derive(Debug, Clone, PartialEq)]
pub enum OwnerContributionError {
    /// Montant négatif (un revenu entrant ne peut être < 0).
    NonPositiveAmount,
    /// Description vide.
    EmptyDescription,
}

impl std::fmt::Display for OwnerContributionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonPositiveAmount => write!(
                f,
                "Contribution amount must be positive (revenue = money coming IN)"
            ),
            Self::EmptyDescription => write!(f, "Description cannot be empty"),
        }
    }
}

impl std::error::Error for OwnerContributionError {}

/// Bridge : use-cases/ports `Result<_, String>` inchangés pendant que
/// l'entité est typée (cascade String→AppError = slice large différée,
/// précédent WP-A3/A4/A5). Pur, std-only.
impl From<OwnerContributionError> for String {
    fn from(e: OwnerContributionError) -> String {
        e.to_string()
    }
}

impl OwnerContribution {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        owner_id: Uuid,
        unit_id: Option<Uuid>,
        description: String,
        amount: Decimal,
        contribution_type: ContributionType,
        contribution_date: DateTime<Utc>,
        account_code: Option<String>,
    ) -> Result<Self, OwnerContributionError> {
        // Validate amount is positive (revenue = money coming IN)
        if amount < Decimal::ZERO {
            return Err(OwnerContributionError::NonPositiveAmount);
        }

        // Validate description
        if description.trim().is_empty() {
            return Err(OwnerContributionError::EmptyDescription);
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            owner_id,
            unit_id,
            description,
            amount,
            account_code,
            contribution_type,
            contribution_date,
            payment_date: None,
            payment_method: None,
            payment_reference: None,
            payment_status: ContributionPaymentStatus::Pending,
            call_for_funds_id: None,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
        })
    }

    /// Mark contribution as paid
    pub fn mark_as_paid(
        &mut self,
        payment_date: DateTime<Utc>,
        payment_method: ContributionPaymentMethod,
        payment_reference: Option<String>,
    ) {
        self.payment_date = Some(payment_date);
        self.payment_method = Some(payment_method);
        self.payment_reference = payment_reference;
        self.payment_status = ContributionPaymentStatus::Paid;
        self.updated_at = Utc::now();
    }

    /// Check if contribution is paid
    pub fn is_paid(&self) -> bool {
        self.payment_status == ContributionPaymentStatus::Paid
    }

    /// Check if contribution is overdue (not paid and past contribution_date)
    pub fn is_overdue(&self) -> bool {
        !self.is_paid() && Utc::now() > self.contribution_date
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_contribution_success() {
        let contrib = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            "Appel de fonds Q1 2025".to_string(),
            rust_decimal_macros::dec!(500),
            ContributionType::Regular,
            Utc::now(),
            Some("7000".to_string()),
        );

        assert!(contrib.is_ok());
        let contrib = contrib.unwrap();
        assert_eq!(contrib.amount, rust_decimal_macros::dec!(500));
        assert_eq!(contrib.payment_status, ContributionPaymentStatus::Pending);
        assert!(!contrib.is_paid());
    }

    #[test]
    fn test_create_contribution_negative_amount() {
        let contrib = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "Test".to_string(),
            rust_decimal_macros::dec!(-100), // Negative amount
            ContributionType::Regular,
            Utc::now(),
            None,
        );

        assert!(matches!(
            contrib.unwrap_err(),
            OwnerContributionError::NonPositiveAmount
        ));
    }

    #[test]
    fn test_create_contribution_empty_description() {
        let contrib = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "   ".to_string(), // Empty description
            rust_decimal_macros::dec!(100),
            ContributionType::Regular,
            Utc::now(),
            None,
        );

        assert!(matches!(
            contrib.unwrap_err(),
            OwnerContributionError::EmptyDescription
        ));
    }

    #[test]
    fn test_mark_as_paid() {
        let mut contrib = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "Test payment".to_string(),
            rust_decimal_macros::dec!(100),
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();

        assert!(!contrib.is_paid());

        contrib.mark_as_paid(
            Utc::now(),
            ContributionPaymentMethod::BankTransfer,
            Some("REF-123".to_string()),
        );

        assert!(contrib.is_paid());
        assert!(contrib.payment_date.is_some());
        assert_eq!(
            contrib.payment_method,
            Some(ContributionPaymentMethod::BankTransfer)
        );
        assert_eq!(contrib.payment_reference, Some("REF-123".to_string()));
    }

    #[test]
    fn test_is_overdue() {
        let past_date = Utc::now() - chrono::Duration::days(30);

        let contrib = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "Overdue contribution".to_string(),
            rust_decimal_macros::dec!(100),
            ContributionType::Regular,
            past_date,
            None,
        )
        .unwrap();

        assert!(contrib.is_overdue());
    }

    // ------------------------------------------------------------------------
    // 4 catégories #433/WP-A6 EXP-008 — erreur typée (CRITICAL.md #3).
    // Entité déjà Decimal (PCMN classe 7) ; ce WP type l'erreur domaine.
    // ------------------------------------------------------------------------

    /// @happy — contribution nominale : montant Decimal exact conservé.
    #[test]
    fn happy_contribution_amount_decimal_exact() {
        let c = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "Provision Q1".to_string(),
            rust_decimal_macros::dec!(1234.56),
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();
        assert_eq!(c.amount, rust_decimal_macros::dec!(1234.56));
    }

    /// @edge — montant exactement zéro accepté (revenu nul, borne incluse) ;
    /// addition Decimal exacte (0.1+0.2=0.3, f64 échoue).
    #[test]
    fn edge_zero_amount_and_decimal_exactness() {
        let zero = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "Régularisation nulle".to_string(),
            Decimal::ZERO,
            ContributionType::Regular,
            Utc::now(),
            None,
        );
        assert!(zero.is_ok());

        let c = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "x".to_string(),
            rust_decimal_macros::dec!(0.1) + rust_decimal_macros::dec!(0.2),
            ContributionType::Regular,
            Utc::now(),
            None,
        )
        .unwrap();
        assert_eq!(c.amount, rust_decimal_macros::dec!(0.3));
    }

    /// @negative — montant négatif & description vide rejetés (erreur typée).
    #[test]
    fn negative_amount_and_empty_description_rejected() {
        assert!(matches!(
            OwnerContribution::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                None,
                "ok".to_string(),
                rust_decimal_macros::dec!(-0.01),
                ContributionType::Regular,
                Utc::now(),
                None,
            )
            .unwrap_err(),
            OwnerContributionError::NonPositiveAmount
        ));
        assert!(matches!(
            OwnerContribution::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                None,
                "  ".to_string(),
                rust_decimal_macros::dec!(10),
                ContributionType::Regular,
                Utc::now(),
                None,
            )
            .unwrap_err(),
            OwnerContributionError::EmptyDescription
        ));
    }

    /// @security — un montant de revenu falsifié négatif (détournement
    /// comptable PCMN classe 7) ne peut jamais être persisté.
    #[test]
    fn security_tampered_negative_revenue_rejected() {
        let result = OwnerContribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "Faux avoir".to_string(),
            rust_decimal_macros::dec!(-99999.99),
            ContributionType::Regular,
            Utc::now(),
            None,
        );
        assert!(matches!(
            result.unwrap_err(),
            OwnerContributionError::NonPositiveAmount
        ));
    }
}
