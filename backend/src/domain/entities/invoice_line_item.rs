use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Représente une ligne de facture détaillée
/// Permet de décomposer une facture en plusieurs postes avec quantité et prix unitaire
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvoiceLineItem {
    pub id: Uuid,
    pub expense_id: Uuid, // Référence à la facture parent
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64, // Prix unitaire HT

    // Montants calculés
    pub amount_excl_vat: f64, // quantity * unit_price
    pub vat_rate: f64,        // Taux TVA (ex: 21.0 pour 21%)
    pub vat_amount: f64,      // Montant TVA
    pub amount_incl_vat: f64, // Montant TTC

    pub created_at: DateTime<Utc>,
}

impl InvoiceLineItem {
    pub fn new(
        expense_id: Uuid,
        description: String,
        quantity: f64,
        unit_price: f64,
        vat_rate: f64,
    ) -> Result<Self, String> {
        // Validations
        if description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        if quantity <= 0.0 {
            return Err("Quantity must be greater than 0".to_string());
        }
        if unit_price < 0.0 {
            return Err("Unit price cannot be negative".to_string());
        }
        if !(0.0..=100.0).contains(&vat_rate) {
            return Err("VAT rate must be between 0 and 100".to_string());
        }

        // Calculs automatiques
        let amount_excl_vat = quantity * unit_price;
        let vat_amount = (amount_excl_vat * vat_rate) / 100.0;
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

    /// Recalcule les montants si quantity ou unit_price changent
    pub fn recalculate(&mut self) -> Result<(), String> {
        if self.quantity <= 0.0 {
            return Err("Quantity must be greater than 0".to_string());
        }
        if self.unit_price < 0.0 {
            return Err("Unit price cannot be negative".to_string());
        }
        if self.vat_rate < 0.0 || self.vat_rate > 100.0 {
            return Err("VAT rate must be between 0 and 100".to_string());
        }

        self.amount_excl_vat = self.quantity * self.unit_price;
        self.vat_amount = (self.amount_excl_vat * self.vat_rate) / 100.0;
        self.amount_incl_vat = self.amount_excl_vat + self.vat_amount;
        Ok(())
    }

    /// Calcule le total HT pour toutes les lignes
    pub fn total_excl_vat(items: &[InvoiceLineItem]) -> f64 {
        items.iter().map(|item| item.amount_excl_vat).sum()
    }

    /// Calcule le total TVA pour toutes les lignes
    pub fn total_vat(items: &[InvoiceLineItem]) -> f64 {
        items.iter().map(|item| item.vat_amount).sum()
    }

    /// Calcule le total TTC pour toutes les lignes
    pub fn total_incl_vat(items: &[InvoiceLineItem]) -> f64 {
        items.iter().map(|item| item.amount_incl_vat).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_line_item_success() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "Réparation porte principale".to_string(),
            2.0,   // quantity
            150.0, // unit_price
            21.0,  // vat_rate
        );

        assert!(line.is_ok());
        let line = line.unwrap();
        assert_eq!(line.expense_id, expense_id);
        assert_eq!(line.quantity, 2.0);
        assert_eq!(line.unit_price, 150.0);
        assert_eq!(line.amount_excl_vat, 300.0); // 2 * 150
        assert_eq!(line.vat_amount, 63.0); // 300 * 21%
        assert_eq!(line.amount_incl_vat, 363.0); // 300 + 63
    }

    #[test]
    fn test_create_line_item_with_vat_6_percent() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "Travaux isolation toit".to_string(),
            1.0,
            10000.0,
            6.0, // TVA réduite 6%
        )
        .unwrap();

        assert_eq!(line.amount_excl_vat, 10000.0);
        assert_eq!(line.vat_amount, 600.0); // 10000 * 6%
        assert_eq!(line.amount_incl_vat, 10600.0);
    }

    #[test]
    fn test_create_line_item_empty_description_fails() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(expense_id, "   ".to_string(), 1.0, 100.0, 21.0);

        assert!(line.is_err());
        assert_eq!(line.unwrap_err(), "Description cannot be empty");
    }

    #[test]
    fn test_create_line_item_zero_quantity_fails() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(expense_id, "Test".to_string(), 0.0, 100.0, 21.0);

        assert!(line.is_err());
        assert_eq!(line.unwrap_err(), "Quantity must be greater than 0");
    }

    #[test]
    fn test_create_line_item_negative_unit_price_fails() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(expense_id, "Test".to_string(), 1.0, -50.0, 21.0);

        assert!(line.is_err());
        assert_eq!(line.unwrap_err(), "Unit price cannot be negative");
    }

    #[test]
    fn test_create_line_item_invalid_vat_rate_fails() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(expense_id, "Test".to_string(), 1.0, 100.0, 150.0);

        assert!(line.is_err());
    }

    #[test]
    fn test_recalculate_after_quantity_change() {
        let expense_id = Uuid::new_v4();
        let mut line =
            InvoiceLineItem::new(expense_id, "Test".to_string(), 1.0, 100.0, 21.0).unwrap();

        assert_eq!(line.amount_excl_vat, 100.0);

        // Changer la quantité
        line.quantity = 3.0;
        line.recalculate().unwrap();

        assert_eq!(line.amount_excl_vat, 300.0); // 3 * 100
        assert_eq!(line.vat_amount, 63.0); // 300 * 21%
        assert_eq!(line.amount_incl_vat, 363.0);
    }

    #[test]
    fn test_recalculate_after_unit_price_change() {
        let expense_id = Uuid::new_v4();
        let mut line =
            InvoiceLineItem::new(expense_id, "Test".to_string(), 2.0, 100.0, 21.0).unwrap();

        // Changer le prix unitaire
        line.unit_price = 200.0;
        line.recalculate().unwrap();

        assert_eq!(line.amount_excl_vat, 400.0); // 2 * 200
        assert_eq!(line.vat_amount, 84.0); // 400 * 21%
        assert_eq!(line.amount_incl_vat, 484.0);
    }

    #[test]
    fn test_total_calculations_multiple_lines() {
        let expense_id = Uuid::new_v4();

        let lines = vec![
            InvoiceLineItem::new(expense_id, "Item 1".to_string(), 2.0, 100.0, 21.0).unwrap(),
            InvoiceLineItem::new(expense_id, "Item 2".to_string(), 1.0, 300.0, 21.0).unwrap(),
            InvoiceLineItem::new(expense_id, "Item 3".to_string(), 3.0, 50.0, 6.0).unwrap(),
        ];

        // Item 1: 200 HT, 42 TVA, 242 TTC
        // Item 2: 300 HT, 63 TVA, 363 TTC
        // Item 3: 150 HT, 9 TVA, 159 TTC
        // Total: 650 HT, 114 TVA, 764 TTC

        let total_excl_vat = InvoiceLineItem::total_excl_vat(&lines);
        let total_vat = InvoiceLineItem::total_vat(&lines);
        let total_incl_vat = InvoiceLineItem::total_incl_vat(&lines);

        assert_eq!(total_excl_vat, 650.0);
        assert_eq!(total_vat, 114.0);
        assert_eq!(total_incl_vat, 764.0);
    }

    #[test]
    fn test_total_calculations_empty_list() {
        let lines: Vec<InvoiceLineItem> = vec![];

        assert_eq!(InvoiceLineItem::total_excl_vat(&lines), 0.0);
        assert_eq!(InvoiceLineItem::total_vat(&lines), 0.0);
        assert_eq!(InvoiceLineItem::total_incl_vat(&lines), 0.0);
    }

    #[test]
    fn test_description_trimmed() {
        let expense_id = Uuid::new_v4();
        let line = InvoiceLineItem::new(
            expense_id,
            "  Peinture couloir   ".to_string(),
            1.0,
            500.0,
            21.0,
        )
        .unwrap();

        assert_eq!(line.description, "Peinture couloir");
    }
}
