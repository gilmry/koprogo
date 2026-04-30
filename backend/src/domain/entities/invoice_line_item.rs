//! Invoice line item entity — monetary fields use `rust_decimal::Decimal`
//!
//! Migration story EXP-003 (cf. ADR-0007 `docs/adr/0007-decimal-vs-f64-for-money.md`,
//! ADR-0008 `docs/adr/0008-numeric-vs-double-precision-postgresql.md`).
//!
//! All `f64` monetary fields migrated to `Decimal` for PCMN belge exactness
//! (Arrêté Royal du 12 juillet 2012). VAT computations no longer subject to
//! IEEE 754 cumulative drift.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Représente une ligne de facture détaillée
/// Permet de décomposer une facture en plusieurs postes avec quantité et prix unitaire.
///
/// **Précision** : tous les montants et taux utilisent `Decimal` pour conformité PCMN.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvoiceLineItem {
    pub id: Uuid,
    pub expense_id: Uuid, // Référence à la facture parent
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal, // Prix unitaire HT

    // Montants calculés (exact, conforme PCMN)
    pub amount_excl_vat: Decimal, // quantity * unit_price
    pub vat_rate: Decimal,        // Taux TVA (ex: 21.0 pour 21%, ou 6.0)
    pub vat_amount: Decimal,      // Montant TVA = amount_excl_vat * vat_rate / 100
    pub amount_incl_vat: Decimal, // Montant TTC = amount_excl_vat + vat_amount

    pub created_at: DateTime<Utc>,
}

impl InvoiceLineItem {
    pub fn new(
        expense_id: Uuid,
        description: String,
        quantity: Decimal,
        unit_price: Decimal,
        vat_rate: Decimal,
    ) -> Result<Self, String> {
        // Validations
        if description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        if quantity <= Decimal::ZERO {
            return Err("Quantity must be greater than 0".to_string());
        }
        if unit_price < Decimal::ZERO {
            return Err("Unit price cannot be negative".to_string());
        }
        if vat_rate < Decimal::ZERO || vat_rate > dec!(100) {
            return Err("VAT rate must be between 0 and 100".to_string());
        }

        // Calculs automatiques (exact decimal arithmetic, no IEEE 754 drift)
        let amount_excl_vat = quantity * unit_price;
        let vat_amount = (amount_excl_vat * vat_rate) / dec!(100);
        let amount_incl_vat = amount_excl_vat + vat_amount;

        Ok(Self {
            id: Uuid::new_v4(),
            expense_id,
            description: description.trim().to_string(),
            quantity,
            unit_price,
            amount_excl_vat,
            vat_rate,
            vat_amount,
            amount_incl_vat,
            created_at: Utc::now(),
        })
    }

    /// Recalcule les montants si quantity, unit_price ou vat_rate changent.
    pub fn recalculate(&mut self) -> Result<(), String> {
        if self.quantity <= Decimal::ZERO {
            return Err("Quantity must be greater than 0".to_string());
        }
        if self.unit_price < Decimal::ZERO {
            return Err("Unit price cannot be negative".to_string());
        }
        if self.vat_rate < Decimal::ZERO || self.vat_rate > dec!(100) {
            return Err("VAT rate must be between 0 and 100".to_string());
        }

        self.amount_excl_vat = self.quantity * self.unit_price;
        self.vat_amount = (self.amount_excl_vat * self.vat_rate) / dec!(100);
        self.amount_incl_vat = self.amount_excl_vat + self.vat_amount;
        Ok(())
    }

    /// Calcule le total HT pour toutes les lignes.
    /// Exact : pas de drift IEEE 754 sur cumul.
    pub fn total_excl_vat(items: &[InvoiceLineItem]) -> Decimal {
        items.iter().map(|item| item.amount_excl_vat).sum()
    }

    /// Calcule le total TVA pour toutes les lignes.
    pub fn total_vat(items: &[InvoiceLineItem]) -> Decimal {
        items.iter().map(|item| item.vat_amount).sum()
    }

    /// Calcule le total TTC pour toutes les lignes.
    pub fn total_incl_vat(items: &[InvoiceLineItem]) -> Decimal {
        items.iter().map(|item| item.amount_incl_vat).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------
    // @happy — chemin nominal end-to-end
    // -------------------------------------------------------------------

    #[test]
    fn happy_create_line_item_success() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "Réparation porte principale".to_string(),
            dec!(2),   // quantity
            dec!(150), // unit_price
            dec!(21),  // vat_rate
        );

        assert!(line.is_ok());
        let line = line.unwrap();
        assert_eq!(line.expense_id, expense_id);
        assert_eq!(line.quantity, dec!(2));
        assert_eq!(line.unit_price, dec!(150));
        assert_eq!(line.amount_excl_vat, dec!(300)); // 2 * 150
        assert_eq!(line.vat_amount, dec!(63)); // 300 * 21%
        assert_eq!(line.amount_incl_vat, dec!(363)); // 300 + 63
    }

    #[test]
    fn happy_create_line_item_with_vat_6_percent() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "Travaux isolation toit".to_string(),
            dec!(1),
            dec!(10000),
            dec!(6), // TVA réduite 6%
        )
        .unwrap();

        assert_eq!(line.amount_excl_vat, dec!(10000));
        assert_eq!(line.vat_amount, dec!(600)); // 10000 * 6%
        assert_eq!(line.amount_incl_vat, dec!(10600));
    }

    #[test]
    fn happy_total_calculations_multiple_lines() {
        let expense_id = Uuid::new_v4();

        let lines = vec![
            InvoiceLineItem::new(expense_id, "Item 1".to_string(), dec!(2), dec!(100), dec!(21))
                .unwrap(),
            InvoiceLineItem::new(expense_id, "Item 2".to_string(), dec!(1), dec!(300), dec!(21))
                .unwrap(),
            InvoiceLineItem::new(expense_id, "Item 3".to_string(), dec!(3), dec!(50), dec!(6))
                .unwrap(),
        ];

        // Item 1: 200 HT, 42 TVA, 242 TTC
        // Item 2: 300 HT, 63 TVA, 363 TTC
        // Item 3: 150 HT, 9 TVA, 159 TTC
        // Total: 650 HT, 114 TVA, 764 TTC
        assert_eq!(InvoiceLineItem::total_excl_vat(&lines), dec!(650));
        assert_eq!(InvoiceLineItem::total_vat(&lines), dec!(114));
        assert_eq!(InvoiceLineItem::total_incl_vat(&lines), dec!(764));
    }

    // -------------------------------------------------------------------
    // @edge — bornes (max/min/empty/0/1/N, dates limites, precision)
    // -------------------------------------------------------------------

    #[test]
    fn edge_total_calculations_empty_list() {
        let lines: Vec<InvoiceLineItem> = vec![];
        assert_eq!(InvoiceLineItem::total_excl_vat(&lines), Decimal::ZERO);
        assert_eq!(InvoiceLineItem::total_vat(&lines), Decimal::ZERO);
        assert_eq!(InvoiceLineItem::total_incl_vat(&lines), Decimal::ZERO);
    }

    #[test]
    fn edge_description_trimmed() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "  Peinture couloir   ".to_string(),
            dec!(1),
            dec!(500),
            dec!(21),
        )
        .unwrap();
        assert_eq!(line.description, "Peinture couloir");
    }

    #[test]
    fn edge_recalculate_after_quantity_change() {
        let expense_id = Uuid::new_v4();
        let mut line =
            InvoiceLineItem::new(expense_id, "Test".to_string(), dec!(1), dec!(100), dec!(21))
                .unwrap();

        assert_eq!(line.amount_excl_vat, dec!(100));

        line.quantity = dec!(3);
        line.recalculate().unwrap();

        assert_eq!(line.amount_excl_vat, dec!(300));
        assert_eq!(line.vat_amount, dec!(63));
        assert_eq!(line.amount_incl_vat, dec!(363));
    }

    #[test]
    fn edge_recalculate_after_unit_price_change() {
        let expense_id = Uuid::new_v4();
        let mut line =
            InvoiceLineItem::new(expense_id, "Test".to_string(), dec!(2), dec!(100), dec!(21))
                .unwrap();

        line.unit_price = dec!(200);
        line.recalculate().unwrap();

        assert_eq!(line.amount_excl_vat, dec!(400));
        assert_eq!(line.vat_amount, dec!(84));
        assert_eq!(line.amount_incl_vat, dec!(484));
    }

    #[test]
    fn edge_decimal_exactness_preserved_on_cumul() {
        // Le test critique du f64 vs Decimal :
        // 0.1 + 0.2 == 0.3 EXACTEMENT en Decimal (ce qui est faux en f64).
        let expense_id = Uuid::new_v4();
        let lines = vec![
            InvoiceLineItem::new(
                expense_id,
                "Item small".to_string(),
                dec!(1),
                dec!(0.1),
                dec!(0),
            )
            .unwrap(),
            InvoiceLineItem::new(
                expense_id,
                "Item small 2".to_string(),
                dec!(1),
                dec!(0.2),
                dec!(0),
            )
            .unwrap(),
        ];

        assert_eq!(InvoiceLineItem::total_excl_vat(&lines), dec!(0.3));
    }

    #[test]
    fn edge_vat_rate_zero_allowed() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "Exonéré TVA".to_string(),
            dec!(1),
            dec!(100),
            Decimal::ZERO,
        )
        .unwrap();
        assert_eq!(line.vat_amount, Decimal::ZERO);
        assert_eq!(line.amount_incl_vat, dec!(100));
    }

    #[test]
    fn edge_vat_rate_max_100_allowed() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "Border case".to_string(),
            dec!(1),
            dec!(100),
            dec!(100),
        )
        .unwrap();
        assert_eq!(line.vat_amount, dec!(100)); // 100% TVA
        assert_eq!(line.amount_incl_vat, dec!(200));
    }

    // -------------------------------------------------------------------
    // @security — RBAC, auth, injection (n/a entité pure) ; ici on
    // teste les invariants de saisie qui empêcheraient un client
    // malveillant de créer une ligne incohérente ou abusive
    // -------------------------------------------------------------------

    #[test]
    fn security_create_line_item_empty_description_fails() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "   ".to_string(),
            dec!(1),
            dec!(100),
            dec!(21),
        );
        assert!(line.is_err());
        assert_eq!(line.unwrap_err(), "Description cannot be empty");
    }

    #[test]
    fn security_create_line_item_negative_unit_price_fails() {
        // Empêche un client malveillant de créer une 'remise' déguisée
        // en prix négatif qui contournerait les workflows d'approbation.
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "Test".to_string(),
            dec!(1),
            dec!(-50),
            dec!(21),
        );
        assert!(line.is_err());
        assert_eq!(line.unwrap_err(), "Unit price cannot be negative");
    }

    #[test]
    fn security_create_line_item_vat_rate_above_100_fails() {
        // Empêche un taux de TVA aberrant (e.g., 9999%) qui pourrait
        // gonfler artificiellement le montant TTC.
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "Test".to_string(),
            dec!(1),
            dec!(100),
            dec!(150),
        );
        assert!(line.is_err());
    }

    #[test]
    fn security_create_line_item_negative_vat_rate_fails() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "Test".to_string(),
            dec!(1),
            dec!(100),
            dec!(-1),
        );
        assert!(line.is_err());
    }

    // -------------------------------------------------------------------
    // @negative — défaillance correcte (pas de panic, erreur typée)
    // -------------------------------------------------------------------

    #[test]
    fn negative_create_line_item_zero_quantity_fails() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "Test".to_string(),
            Decimal::ZERO,
            dec!(100),
            dec!(21),
        );
        assert!(line.is_err());
        assert_eq!(line.unwrap_err(), "Quantity must be greater than 0");
    }

    #[test]
    fn negative_recalculate_with_invalid_state_returns_error() {
        // Si on modifie directement les fields invalides puis on
        // recalcule, l'erreur est retournée proprement (pas de panic).
        let expense_id = Uuid::new_v4();
        let mut line =
            InvoiceLineItem::new(expense_id, "Test".to_string(), dec!(1), dec!(100), dec!(21))
                .unwrap();

        line.quantity = dec!(-5); // invalide
        let result = line.recalculate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Quantity must be greater than 0");
    }

    #[test]
    fn negative_no_panic_on_extreme_values() {
        // Decimal supports up to ~28 digits. Au-delà, retourne erreur
        // ou comportement défini (pas de panic IEEE 754 NaN/Inf).
        let expense_id = Uuid::new_v4();
        // Valeur grande mais dans la plage Decimal
        let line = InvoiceLineItem::new(
            expense_id,
            "Big".to_string(),
            dec!(1),
            dec!(99999999999.99), // proche de la limite NUMERIC(15,2)
            dec!(21),
        )
        .unwrap();
        // Le calcul doit produire un Decimal valide (pas de NaN)
        assert!(line.amount_incl_vat > line.amount_excl_vat);
    }

    #[test]
    fn negative_description_only_whitespace_fails() {
        let expense_id = Uuid::new_v4();
        let line =
            InvoiceLineItem::new(expense_id, "\t\n  ".to_string(), dec!(1), dec!(100), dec!(21));
        assert!(line.is_err());
    }
}
