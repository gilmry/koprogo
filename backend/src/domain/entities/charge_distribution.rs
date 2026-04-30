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

    pub created_at: DateTime<Utc>,
}

/// Tolerance for distribution sum vs total (1 centime).
const DISTRIBUTION_TOLERANCE: Decimal = dec!(0.01);
/// Tolerance for total quota sum to allow rounding errors (1.0001 = 100.01%).
const QUOTA_SUM_TOLERANCE: Decimal = dec!(1.0001);

impl ChargeDistribution {
    pub fn new(
        expense_id: Uuid,
        unit_id: Uuid,
        owner_id: Uuid,
        quota_percentage: Decimal,
        total_amount: Decimal,
    ) -> Result<Self, String> {
        // Validations
        if quota_percentage < Decimal::ZERO || quota_percentage > Decimal::ONE {
            return Err(format!(
                "Quota percentage must be between 0 and 1 (got: {})",
                quota_percentage
            ));
        }
        if total_amount < Decimal::ZERO {
            return Err("Total amount cannot be negative".to_string());
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
            created_at: Utc::now(),
        })
    }

    /// Recalcule le montant dû si la quote-part ou le total change
    pub fn recalculate(&mut self, total_amount: Decimal) -> Result<(), String> {
        if self.quota_percentage < Decimal::ZERO || self.quota_percentage > Decimal::ONE {
            return Err("Quota percentage must be between 0 and 1".to_string());
        }
        if total_amount < Decimal::ZERO {
            return Err("Total amount cannot be negative".to_string());
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
    ) -> Result<Vec<ChargeDistribution>, String> {
        if total_amount < Decimal::ZERO {
            return Err("Total amount cannot be negative".to_string());
        }

        // Vérifier que la somme des quotes-parts ne dépasse pas 100%
        let total_quota: Decimal = unit_ownerships.iter().map(|(_, _, q)| *q).sum();
        if total_quota > QUOTA_SUM_TOLERANCE {
            // Tolérance pour arrondi
            return Err(format!(
                "Total quota percentage exceeds 100% (got: {})",
                total_quota * dec!(100)
            ));
        }

        let mut distributions = Vec::new();
        for (unit_id, owner_id, quota) in unit_ownerships {
            let distribution =
                ChargeDistribution::new(expense_id, unit_id, owner_id, quota, total_amount)?;
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
        assert!(distribution
            .unwrap_err()
            .contains("Quota percentage must be between 0 and 1"));
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
        assert!(distributions
            .unwrap_err()
            .contains("Total quota percentage exceeds 100%"));
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
        let distributions = ChargeDistribution::calculate_distributions(
            expense_id,
            total_invoice,
            unit_ownerships,
        )
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
}
