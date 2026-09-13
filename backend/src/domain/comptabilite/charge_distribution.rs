// Domain Entity: ChargeDistribution
//
// MONETARY: amount_due/total_amount/quota_percentage use rust_decimal::Decimal (cf. ADR-0007).
// Quote-part exactness is critical: rounding errors in distribution sum to user invoices.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Représente la répartition d'une charge/facture par lot et propriétaire
/// Calculée automatiquement lors de l'approbation d'une facture
/// Basée sur les quotes-parts (ownership percentages) des copropriétaires
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChargeDistribution {
    pub id: Uuid,
    pub expense_id: Uuid, // Référence à la facture
    pub unit_id: Uuid,    // Lot concerné
    pub owner_id: Uuid,   // Propriétaire du lot

    pub quota_percentage: Decimal, // Quote-part (ex: dec!(0.15) pour 15%)
    pub amount_due: Decimal,       // Montant à payer par ce propriétaire

    /// Story H12 — critère légal sous lequel cette ligne a été calculée
    /// (valeur / utilité / mixte, Art. 3.84/3.86). Défaut : `Value`.
    #[serde(default)]
    pub distribution_criteria: DistributionCriteria,

    pub created_at: DateTime<Utc>,
}

/// Tolerance for distribution sum vs total (1 centime).
const DISTRIBUTION_TOLERANCE: Decimal = dec!(0.01);
/// Tolerance for total quota sum to allow rounding errors (1.0001 = 100.01%).
const QUOTA_SUM_TOLERANCE: Decimal = dec!(1.0001);

/// Domain-typed validation error for charge distribution (quote-part exactness).
///
/// Pure domain type — no infrastructure/application dependency (hexagonal
/// purity). Follows the codebase precedent `JournalEntryError`
/// (journal_entry.rs) / `ProxyValidationError` (vote.rs): the entity returns
/// its own typed error; the application layer maps it to `AppError`
/// (see `impl From<ChargeDistributionError> for AppError`) so a malformed
/// distribution surfaces as a 400 validation error, not a 500 Internal
/// (#433 / WP-A4 — EXP-005).
#[derive(Debug, Clone, PartialEq)]
pub enum ChargeDistributionError {
    /// Quota percentage is outside the valid [0, 1] range.
    QuotaOutOfRange(Decimal),
    /// Total amount to distribute is negative.
    NegativeTotalAmount,
    /// Sum of all quotas exceeds 100% beyond the rounding tolerance.
    /// Over-distribution would over-charge owners — financial integrity guard.
    QuotaSumExceeds { total_quota: Decimal },
    /// Story H12 — base de tantièmes invalide (acte de base ≤ 0) : impossible de
    /// calculer une quote-part de lot (division par zéro / base négative).
    InvalidTotalTantiemes(Decimal),
    /// Story H12 — critère de répartition non reconnu (≠ value/utility/mixed).
    /// Garde @security : un critère non prévu par la loi (Art. 3.84/3.86) est
    /// refusé, pas appliqué silencieusement.
    UnknownCriteria(String),
}

impl std::fmt::Display for ChargeDistributionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QuotaOutOfRange(q) => {
                write!(f, "Quota percentage must be between 0 and 1 (got: {})", q)
            }
            Self::NegativeTotalAmount => write!(f, "Total amount cannot be negative"),
            Self::QuotaSumExceeds { total_quota } => write!(
                f,
                "Total quota percentage exceeds 100% (got: {})",
                total_quota * dec!(100)
            ),
            Self::InvalidTotalTantiemes(t) => write!(
                f,
                "Total tantièmes (acte de base) must be strictly positive (got: {})",
                t
            ),
            Self::UnknownCriteria(c) => write!(
                f,
                "Unknown distribution criteria '{}' (expected value|utility|mixed)",
                c
            ),
        }
    }
}

impl std::error::Error for ChargeDistributionError {}

/// Bridge so existing `Result<_, String>` use-cases keep compiling while the
/// entity is typed (the use-case/port String→AppError cascade is a distinct,
/// broader slice — out of WP-A4 scope, mirrors WP-A3). Pure, std-only.
impl From<ChargeDistributionError> for String {
    fn from(e: ChargeDistributionError) -> String {
        e.to_string()
    }
}

/// Critère légal de répartition des charges communes (Story H12, Art. 3.84 /
/// 3.86 CC).
///
/// - `Value` (valeur) : répartition selon la quote-part / valeur respective du
///   lot, c.-à-d. les tantièmes de l'acte de base. **Critère par défaut.**
/// - `Utility` (utilité) : base alternative selon l'utilité de la partie commune
///   pour chaque lot (ex. ascenseur réparti selon l'étage) — Art. 3.86.
/// - `Mixed` (mixte) : combinaison valeur + utilité.
///
/// Pur domaine : sérialisé/persisté en texte (`value`/`utility`/`mixed`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DistributionCriteria {
    #[default]
    Value,
    Utility,
    Mixed,
}

impl DistributionCriteria {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Utility => "utility",
            Self::Mixed => "mixed",
        }
    }
}

impl std::fmt::Display for DistributionCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for DistributionCriteria {
    type Err = ChargeDistributionError;

    /// Strict : un critère non reconnu est REFUSÉ (garde @security H12), jamais
    /// rabattu silencieusement sur une valeur par défaut.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "value" => Ok(Self::Value),
            "utility" => Ok(Self::Utility),
            "mixed" => Ok(Self::Mixed),
            other => Err(ChargeDistributionError::UnknownCriteria(other.to_string())),
        }
    }
}

impl ChargeDistribution {
    pub fn new(
        expense_id: Uuid,
        unit_id: Uuid,
        owner_id: Uuid,
        quota_percentage: Decimal,
        total_amount: Decimal,
    ) -> Result<Self, ChargeDistributionError> {
        Self::new_with_criteria(
            expense_id,
            unit_id,
            owner_id,
            quota_percentage,
            total_amount,
            DistributionCriteria::default(),
        )
    }

    /// Story H12 — constructeur avec critère de répartition explicite.
    pub fn new_with_criteria(
        expense_id: Uuid,
        unit_id: Uuid,
        owner_id: Uuid,
        quota_percentage: Decimal,
        total_amount: Decimal,
        distribution_criteria: DistributionCriteria,
    ) -> Result<Self, ChargeDistributionError> {
        // Validations
        if quota_percentage < Decimal::ZERO || quota_percentage > Decimal::ONE {
            return Err(ChargeDistributionError::QuotaOutOfRange(quota_percentage));
        }
        if total_amount < Decimal::ZERO {
            return Err(ChargeDistributionError::NegativeTotalAmount);
        }

        // Calcul du montant dû
        let amount_due = total_amount * quota_percentage;

        Ok(Self {
            id: Uuid::new_v4(),
            expense_id,
            unit_id,
            owner_id,
            quota_percentage,
            amount_due,
            distribution_criteria,
            created_at: Utc::now(),
        })
    }

    /// Story H12 — quote-part effective d'un copropriétaire pour une charge.
    ///
    /// Clarifie les DEUX niveaux de répartition (DoD H12) :
    /// - `unit_quota / total_tantiemes` = part du **lot** dans les communs
    ///   (valeur respective, acte de base — Art. 3.84) ;
    /// - `× ownership_percentage` = part du **copropriétaire** dans le lot
    ///   (indivision / démembrement — `unit_owners`).
    ///
    /// Retourne la fraction [0,1] à appliquer au montant total de la charge.
    pub fn resolve_owner_quota(
        unit_quota: Decimal,
        total_tantiemes: Decimal,
        ownership_percentage: Decimal,
    ) -> Result<Decimal, ChargeDistributionError> {
        if total_tantiemes <= Decimal::ZERO {
            return Err(ChargeDistributionError::InvalidTotalTantiemes(
                total_tantiemes,
            ));
        }
        if unit_quota < Decimal::ZERO {
            return Err(ChargeDistributionError::QuotaOutOfRange(unit_quota));
        }
        if ownership_percentage < Decimal::ZERO || ownership_percentage > Decimal::ONE {
            return Err(ChargeDistributionError::QuotaOutOfRange(
                ownership_percentage,
            ));
        }
        Ok((unit_quota / total_tantiemes) * ownership_percentage)
    }

    /// Recalcule le montant dû si la quote-part ou le total change
    pub fn recalculate(&mut self, total_amount: Decimal) -> Result<(), ChargeDistributionError> {
        if self.quota_percentage < Decimal::ZERO || self.quota_percentage > Decimal::ONE {
            return Err(ChargeDistributionError::QuotaOutOfRange(
                self.quota_percentage,
            ));
        }
        if total_amount < Decimal::ZERO {
            return Err(ChargeDistributionError::NegativeTotalAmount);
        }

        self.amount_due = total_amount * self.quota_percentage;
        Ok(())
    }

    /// Calcule la distribution pour une facture donnée et une liste de quotes-parts
    /// Retourne une distribution pour chaque (unit, owner, quota)
    pub fn calculate_distributions(
        expense_id: Uuid,
        total_amount: Decimal,
        unit_ownerships: Vec<(Uuid, Uuid, Decimal)>, // (unit_id, owner_id, quota_percentage)
    ) -> Result<Vec<ChargeDistribution>, ChargeDistributionError> {
        Self::calculate_distributions_with_criteria(
            expense_id,
            total_amount,
            unit_ownerships,
            DistributionCriteria::default(),
        )
    }

    /// Story H12 — répartition avec critère explicite (valeur / utilité / mixte).
    /// Sous `Value`, les quotités proviennent de l'acte de base ; sous `Utility`,
    /// elles proviennent d'une base d'utilité (coefficients alternatifs). Le
    /// critère est enregistré sur chaque ligne pour la traçabilité.
    pub fn calculate_distributions_with_criteria(
        expense_id: Uuid,
        total_amount: Decimal,
        unit_ownerships: Vec<(Uuid, Uuid, Decimal)>, // (unit_id, owner_id, quota_percentage)
        criteria: DistributionCriteria,
    ) -> Result<Vec<ChargeDistribution>, ChargeDistributionError> {
        if total_amount < Decimal::ZERO {
            return Err(ChargeDistributionError::NegativeTotalAmount);
        }

        // Vérifier que la somme des quotes-parts ne dépasse pas 100%
        let total_quota: Decimal = unit_ownerships.iter().map(|(_, _, q)| *q).sum();
        if total_quota > QUOTA_SUM_TOLERANCE {
            // Tolérance pour arrondi
            return Err(ChargeDistributionError::QuotaSumExceeds { total_quota });
        }

        let mut distributions = Vec::new();
        for (unit_id, owner_id, quota) in unit_ownerships {
            let distribution = ChargeDistribution::new_with_criteria(
                expense_id,
                unit_id,
                owner_id,
                quota,
                total_amount,
                criteria,
            )?;
            distributions.push(distribution);
        }

        Ok(distributions)
    }

    /// Calcule le montant total distribué (somme des amount_due)
    pub fn total_distributed(distributions: &[ChargeDistribution]) -> Decimal {
        distributions.iter().map(|d| d.amount_due).sum()
    }

    /// Vérifie que la distribution est complète (somme = total_amount à 0.01€ près)
    pub fn verify_distribution(
        distributions: &[ChargeDistribution],
        expected_total: Decimal,
    ) -> bool {
        let total = Self::total_distributed(distributions);
        (total - expected_total).abs() < DISTRIBUTION_TOLERANCE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_charge_distribution_success() {
        let expense_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let distribution =
            ChargeDistribution::new(expense_id, unit_id, owner_id, dec!(0.25), dec!(1000));

        assert!(distribution.is_ok());
        let distribution = distribution.unwrap();
        assert_eq!(distribution.expense_id, expense_id);
        assert_eq!(distribution.unit_id, unit_id);
        assert_eq!(distribution.owner_id, owner_id);
        assert_eq!(distribution.quota_percentage, dec!(0.25));
        assert_eq!(distribution.amount_due, dec!(250.00)); // 25% de 1000€
    }

    #[test]
    fn test_create_charge_distribution_negative_quota_fails() {
        let expense_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let distribution =
            ChargeDistribution::new(expense_id, unit_id, owner_id, dec!(-0.1), dec!(1000));

        assert!(distribution.is_err());
        assert!(matches!(
            distribution.unwrap_err(),
            ChargeDistributionError::QuotaOutOfRange(_)
        ));
    }

    #[test]
    fn test_create_charge_distribution_quota_above_1_fails() {
        let expense_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let distribution =
            ChargeDistribution::new(expense_id, unit_id, owner_id, dec!(1.5), dec!(1000));

        assert!(distribution.is_err());
    }

    #[test]
    fn test_recalculate_amount_due() {
        let expense_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let mut distribution =
            ChargeDistribution::new(expense_id, unit_id, owner_id, dec!(0.20), dec!(1000)).unwrap();

        assert_eq!(distribution.amount_due, dec!(200.00));

        // Recalculer avec un nouveau montant total
        distribution.recalculate(dec!(1500)).unwrap();
        assert_eq!(distribution.amount_due, dec!(300.00)); // 20% de 1500€
    }

    #[test]
    fn test_calculate_distributions_success() {
        let expense_id = Uuid::new_v4();
        let unit1_id = Uuid::new_v4();
        let unit2_id = Uuid::new_v4();
        let unit3_id = Uuid::new_v4();
        let owner1_id = Uuid::new_v4();
        let owner2_id = Uuid::new_v4();
        let owner3_id = Uuid::new_v4();

        let unit_ownerships = vec![
            (unit1_id, owner1_id, dec!(0.25)), // 25%
            (unit2_id, owner2_id, dec!(0.35)), // 35%
            (unit3_id, owner3_id, dec!(0.40)), // 40%
        ];

        let distributions =
            ChargeDistribution::calculate_distributions(expense_id, dec!(1000), unit_ownerships);

        assert!(distributions.is_ok());
        let distributions = distributions.unwrap();
        assert_eq!(distributions.len(), 3);

        // Vérifier les montants (Decimal exact)
        assert_eq!(distributions[0].amount_due, dec!(250.00));
        assert_eq!(distributions[1].amount_due, dec!(350.00));
        assert_eq!(distributions[2].amount_due, dec!(400.00));

        // Vérifier le total
        let total = ChargeDistribution::total_distributed(&distributions);
        assert_eq!(total, dec!(1000.00));
    }

    #[test]
    fn test_calculate_distributions_quota_exceeds_100_fails() {
        let expense_id = Uuid::new_v4();
        let unit1_id = Uuid::new_v4();
        let unit2_id = Uuid::new_v4();
        let owner1_id = Uuid::new_v4();
        let owner2_id = Uuid::new_v4();

        let unit_ownerships = vec![
            (unit1_id, owner1_id, dec!(0.60)), // 60%
            (unit2_id, owner2_id, dec!(0.50)), // 50% -> Total 110%
        ];

        let distributions =
            ChargeDistribution::calculate_distributions(expense_id, dec!(1000), unit_ownerships);

        assert!(distributions.is_err());
        assert!(matches!(
            distributions.unwrap_err(),
            ChargeDistributionError::QuotaSumExceeds { .. }
        ));
    }

    #[test]
    fn test_calculate_distributions_empty_list() {
        let expense_id = Uuid::new_v4();
        let unit_ownerships = vec![];

        let distributions =
            ChargeDistribution::calculate_distributions(expense_id, dec!(1000), unit_ownerships);

        assert!(distributions.is_ok());
        let distributions = distributions.unwrap();
        assert_eq!(distributions.len(), 0);
    }

    #[test]
    fn test_verify_distribution_exact_match() {
        let expense_id = Uuid::new_v4();
        let unit_ownerships = vec![
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.50)),
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.50)),
        ];

        let distributions =
            ChargeDistribution::calculate_distributions(expense_id, dec!(1000), unit_ownerships)
                .unwrap();

        assert!(ChargeDistribution::verify_distribution(
            &distributions,
            dec!(1000)
        ));
    }

    #[test]
    fn test_verify_distribution_with_rounding() {
        let expense_id = Uuid::new_v4();
        let unit_ownerships = vec![
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.333333)), // 1/3
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.333333)), // 1/3
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.333334)), // 1/3 avec arrondi
        ];

        let distributions =
            ChargeDistribution::calculate_distributions(expense_id, dec!(1000), unit_ownerships)
                .unwrap();

        // Le total sera ~999.999 ou 1000.001 à cause des arrondis
        // Devrait passer avec tolérance de 1 centime
        assert!(ChargeDistribution::verify_distribution(
            &distributions,
            dec!(1000)
        ));
    }

    #[test]
    fn test_calculate_distributions_complex_scenario() {
        // Scénario réaliste: immeuble avec 5 lots, quotes-parts variées
        let expense_id = Uuid::new_v4();
        let unit_ownerships = vec![
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.25)), // Lot A: 25%
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.20)), // Lot B: 20%
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.20)), // Lot C: 20%
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.20)), // Lot D: 20%
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.15)), // Lot E: 15%
        ];

        let total_invoice = dec!(5000);
        let distributions =
            ChargeDistribution::calculate_distributions(expense_id, total_invoice, unit_ownerships)
                .unwrap();

        assert_eq!(distributions.len(), 5);
        assert_eq!(distributions[0].amount_due, dec!(1250.00)); // 25%
        assert_eq!(distributions[1].amount_due, dec!(1000.00)); // 20%
        assert_eq!(distributions[2].amount_due, dec!(1000.00)); // 20%
        assert_eq!(distributions[3].amount_due, dec!(1000.00)); // 20%
        assert_eq!(distributions[4].amount_due, dec!(750.00)); // 15%

        assert!(ChargeDistribution::verify_distribution(
            &distributions,
            total_invoice
        ));
    }

    #[test]
    fn test_total_distributed_empty() {
        let distributions: Vec<ChargeDistribution> = vec![];
        assert_eq!(
            ChargeDistribution::total_distributed(&distributions),
            Decimal::ZERO
        );
    }

    #[test]
    fn test_quota_percentage_zero_is_valid() {
        // Un lot peut avoir 0% de quote-part (cas particulier)
        let expense_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let distribution =
            ChargeDistribution::new(expense_id, unit_id, owner_id, Decimal::ZERO, dec!(1000));

        assert!(distribution.is_ok());
        let distribution = distribution.unwrap();
        assert_eq!(distribution.amount_due, Decimal::ZERO);
    }

    #[test]
    fn test_quota_percentage_exactly_one_is_valid() {
        // Un seul propriétaire avec 100% de quote-part
        let expense_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let distribution =
            ChargeDistribution::new(expense_id, unit_id, owner_id, Decimal::ONE, dec!(1000));

        assert!(distribution.is_ok());
        let distribution = distribution.unwrap();
        assert_eq!(distribution.amount_due, dec!(1000));
    }

    /// @edge — Decimal exactness preserved on cumul (ADR-0007).
    #[test]
    fn edge_distribution_decimal_exactness() {
        // 1/10 * 3 = 0.3 exact en Decimal (en f64, 0.1+0.1+0.1 != 0.3)
        let dist1 = ChargeDistribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            dec!(0.1),
            dec!(1),
        )
        .unwrap();
        let dist2 = ChargeDistribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            dec!(0.1),
            dec!(1),
        )
        .unwrap();
        let dist3 = ChargeDistribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            dec!(0.1),
            dec!(1),
        )
        .unwrap();

        let dists = vec![dist1, dist2, dist3];
        assert_eq!(ChargeDistribution::total_distributed(&dists), dec!(0.3));
    }

    // ------------------------------------------------------------------------
    // 4 catégories #433/WP-A4 — taxonomie typée (CRITICAL.md #3). Le glue BDD
    // (charge_distribution.feature) fixe une répartition valide à 100% via le
    // Background et ne peut donc pas exercer comportementalement les chemins de
    // rejet : ces invariants de l'entité domaine sont vérifiés ici en unitaire,
    // sur l'erreur typée `ChargeDistributionError` (précédent WP-A3
    // journal_entry.rs / commentaire journal_entries.feature).
    // ------------------------------------------------------------------------

    /// @happy — Nominal distribution: quotes-parts somment à 100%, total réparti
    /// exactement, équilibre vérifié à 1 centime.
    #[test]
    fn happy_distribution_balances_to_total() {
        let expense_id = Uuid::new_v4();
        let ownerships = vec![
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.40)),
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.35)),
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.25)),
        ];

        let dists = ChargeDistribution::calculate_distributions(expense_id, dec!(1000), ownerships)
            .unwrap();

        assert_eq!(dists.len(), 3);
        assert_eq!(ChargeDistribution::total_distributed(&dists), dec!(1000.00));
        assert!(ChargeDistribution::verify_distribution(&dists, dec!(1000)));
    }

    /// @edge — Borne exacte de la tolérance de somme des quotités :
    /// 100.01% (= QUOTA_SUM_TOLERANCE) passe, 100.011% est rejeté.
    #[test]
    fn edge_quota_sum_at_tolerance_boundary() {
        let expense_id = Uuid::new_v4();

        // Exactement 1.0001 (100.01%) — accepté (borne stricte `>`).
        let at_boundary = vec![
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.5000)),
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.5001)),
        ];
        assert!(
            ChargeDistribution::calculate_distributions(expense_id, dec!(1000), at_boundary)
                .is_ok(),
            "Σ quotités == 1.0001 doit passer (borne de tolérance)"
        );

        // 1.00011 (100.011%) — au-delà de la tolérance, rejeté.
        let over_boundary = vec![
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.50000)),
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.50011)),
        ];
        assert!(matches!(
            ChargeDistribution::calculate_distributions(expense_id, dec!(1000), over_boundary)
                .unwrap_err(),
            ChargeDistributionError::QuotaSumExceeds { .. }
        ));
    }

    /// @negative — Quote-part > 1 ou négative rejetée (erreur typée, pas de panic).
    #[test]
    fn negative_quota_out_of_range_rejected() {
        let above = ChargeDistribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            dec!(1.5),
            dec!(1000),
        );
        assert!(matches!(
            above.unwrap_err(),
            ChargeDistributionError::QuotaOutOfRange(_)
        ));

        let negative = ChargeDistribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            dec!(-0.1),
            dec!(1000),
        );
        assert!(matches!(
            negative.unwrap_err(),
            ChargeDistributionError::QuotaOutOfRange(_)
        ));
    }

    /// @negative — Montant total négatif rejeté (erreur typée).
    #[test]
    fn negative_total_amount_rejected() {
        let result = ChargeDistribution::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            dec!(0.25),
            dec!(-1000),
        );
        assert!(matches!(
            result.unwrap_err(),
            ChargeDistributionError::NegativeTotalAmount
        ));
    }

    /// @security — Une table de quotités falsifiée sommant à > 100% ne doit
    /// jamais permettre de sur-répartir une charge (sur-facturation des
    /// copropriétaires) : invariant d'intégrité financière (#433/WP-A4).
    #[test]
    fn security_quota_sum_overflow_prevents_overcharge() {
        let expense_id = Uuid::new_v4();
        // Σ = 130% — tentative de sur-distribution.
        let tampered = vec![
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.70)),
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.60)),
        ];

        let result = ChargeDistribution::calculate_distributions(expense_id, dec!(10000), tampered);

        assert!(matches!(
            result.unwrap_err(),
            ChargeDistributionError::QuotaSumExceeds { .. }
        ));
    }

    // ------------------------------------------------------------------------
    // Story H12 (CL4) — DistributionCriteria + formule deux niveaux (4-cat).
    // ------------------------------------------------------------------------

    /// @happy — Critère `value` : quote-part effective d'un copropriétaire =
    /// (quotité du lot / total tantièmes acte de base) × part dans le lot.
    #[test]
    fn happy_resolve_owner_quota_by_value() {
        // Lot 250/1000 (acte de base 1000), copropriétaire unique (100%).
        let q =
            ChargeDistribution::resolve_owner_quota(dec!(250), dec!(1000), Decimal::ONE).unwrap();
        assert_eq!(q, dec!(0.25));
        // Même lot sur base 10000 (acte de base à 10000) : 2500/10000 = 0.25.
        let q10000 =
            ChargeDistribution::resolve_owner_quota(dec!(2500), dec!(10000), Decimal::ONE).unwrap();
        assert_eq!(q10000, dec!(0.25));
        // Indivision 50/50 : 0.25 × 0.5 = 0.125.
        let half =
            ChargeDistribution::resolve_owner_quota(dec!(250), dec!(1000), dec!(0.5)).unwrap();
        assert_eq!(half, dec!(0.125));
    }

    /// @edge — Critère `utility` (base alternative) enregistré distinctement de
    /// `value` ; le défaut reste `value`.
    #[test]
    fn edge_utility_criteria_is_recorded() {
        let expense_id = Uuid::new_v4();
        let ownerships = vec![
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.70)), // utilité (ex. ascenseur)
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.30)),
        ];
        let dists = ChargeDistribution::calculate_distributions_with_criteria(
            expense_id,
            dec!(1000),
            ownerships,
            DistributionCriteria::Utility,
        )
        .unwrap();
        assert_eq!(dists.len(), 2);
        assert!(dists
            .iter()
            .all(|d| d.distribution_criteria == DistributionCriteria::Utility));

        let dflt = ChargeDistribution::new(
            expense_id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            dec!(0.5),
            dec!(1000),
        )
        .unwrap();
        assert_eq!(dflt.distribution_criteria, DistributionCriteria::Value);
    }

    /// @security — Un critère non prévu par la loi (≠ value/utility/mixed) est
    /// refusé (erreur typée), jamais rabattu silencieusement.
    #[test]
    fn security_unknown_criteria_rejected() {
        use std::str::FromStr;
        assert_eq!(
            DistributionCriteria::from_str("value").unwrap(),
            DistributionCriteria::Value
        );
        assert_eq!(
            DistributionCriteria::from_str("UTILITY").unwrap(),
            DistributionCriteria::Utility
        );
        let err = DistributionCriteria::from_str("au_pif").unwrap_err();
        assert!(matches!(err, ChargeDistributionError::UnknownCriteria(_)));
    }

    /// @negative — Somme des lignes ≠ total → distribution non équilibrée
    /// détectée ; base de tantièmes nulle → erreur typée (pas de division par 0).
    #[test]
    fn negative_sum_mismatch_and_zero_tantiemes() {
        let expense_id = Uuid::new_v4();
        // Σ quotités = 80% sur 1000 → total réparti 800 ≠ 1000.
        let ownerships = vec![
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.40)),
            (Uuid::new_v4(), Uuid::new_v4(), dec!(0.40)),
        ];
        let dists = ChargeDistribution::calculate_distributions(expense_id, dec!(1000), ownerships)
            .unwrap();
        assert_eq!(ChargeDistribution::total_distributed(&dists), dec!(800.00));
        assert!(!ChargeDistribution::verify_distribution(&dists, dec!(1000)));

        // Base de tantièmes 0 → erreur typée (acte de base invalide, pas de div/0).
        let err = ChargeDistribution::resolve_owner_quota(dec!(250), Decimal::ZERO, Decimal::ONE)
            .unwrap_err();
        assert!(matches!(
            err,
            ChargeDistributionError::InvalidTotalTantiemes(_)
        ));
    }
}
