use crate::domain::entities::{Building, Expense, ExpenseCategory};
use crate::domain::services::pdf_writer::{
    builtin_font, new_document, save_document, PdfPageBuilder,
};
use chrono::Utc;
use printpdf::BuiltinFont;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

/// Annual Financial Report Exporter - Generates PDF for Rapport Financier Annuel
///
/// Generates comprehensive annual financial reports with expense breakdowns.
///
/// MONETARY: budgeted/actual/total_income/reserve_fund use rust_decimal::Decimal
/// (cf. ADR-0007). Le PDF affiche via `{:.2}` lequel formate Decimal correctement.
pub struct AnnualReportExporter;

#[derive(Debug, Clone)]
pub struct BudgetItem {
    pub category: ExpenseCategory,
    pub budgeted: Decimal,
    pub actual: Decimal,
}

impl AnnualReportExporter {
    /// Export annual financial report to PDF bytes
    ///
    /// Generates a Rapport Financier Annuel including:
    /// - Building information
    /// - Year summary
    /// - Income breakdown (charges paid)
    /// - Expense breakdown by category
    /// - Budget vs actual
    /// - Reserve fund status
    pub fn export_to_pdf(
        building: &Building,
        year: i32,
        expenses: &[Expense],
        budget_items: &[BudgetItem],
        total_income: Decimal,
        reserve_fund: Decimal,
    ) -> Result<Vec<u8>, String> {
        // Create PDF document (A4: 210mm x 297mm)
        let doc = new_document("Rapport Financier Annuel");
        let mut current_layer = PdfPageBuilder::new();

        // Load fonts
        let font = builtin_font(BuiltinFont::Helvetica);
        let font_bold = builtin_font(BuiltinFont::HelveticaBold);

        let mut y = 270.0; // Start from top

        // === HEADER ===
        current_layer.text(
            "RAPPORT FINANCIER ANNUEL".to_string(),
            18.0,
            20.0,
            y,
            &font_bold,
        );
        y -= 15.0;

        // Building information
        current_layer.text(
            format!("Copropriété: {}", building.name),
            12.0,
            20.0,
            y,
            &font_bold,
        );
        y -= 7.0;

        current_layer.text(
            format!("Adresse: {}", building.address),
            10.0,
            20.0,
            y,
            &font,
        );
        y -= 10.0;

        current_layer.text(format!("Exercice: {}", year), 12.0, 20.0, y, &font_bold);
        y -= 10.0;

        current_layer.text(
            format!("Date d'établissement: {}", Utc::now().format("%d/%m/%Y")),
            10.0,
            20.0,
            y,
            &font,
        );
        y -= 15.0;

        // === SUMMARY ===
        current_layer.text("SYNTHÈSE FINANCIÈRE".to_string(), 14.0, 20.0, y, &font_bold);
        y -= 8.0;

        let total_expenses: Decimal = expenses.iter().map(|e| e.amount).sum();

        current_layer.text(
            format!(
                "Total des produits (charges perçues): {:.2} €",
                total_income
            ),
            11.0,
            20.0,
            y,
            &font,
        );
        y -= 6.0;

        current_layer.text(
            format!("Total des charges: {:.2} €", total_expenses),
            11.0,
            20.0,
            y,
            &font,
        );
        y -= 6.0;

        let balance = total_income - total_expenses;
        let balance_label = if balance >= Decimal::ZERO {
            "Excédent"
        } else {
            "Déficit"
        };
        current_layer.text(
            format!("{}: {:.2} €", balance_label, balance.abs()),
            12.0,
            20.0,
            y,
            &font_bold,
        );
        y -= 6.0;

        current_layer.text(
            format!("Fonds de réserve: {:.2} €", reserve_fund),
            11.0,
            20.0,
            y,
            &font,
        );
        y -= 12.0;

        // === EXPENSE BREAKDOWN BY CATEGORY ===
        current_layer.text(
            "RÉPARTITION DES CHARGES PAR CATÉGORIE".to_string(),
            14.0,
            20.0,
            y,
            &font_bold,
        );
        y -= 8.0;

        // Calculate expenses by category
        let mut category_totals: HashMap<String, Decimal> = HashMap::new();
        for expense in expenses {
            let category_name = Self::category_name(&expense.category);
            *category_totals
                .entry(category_name)
                .or_insert(Decimal::ZERO) += expense.amount;
        }

        // Sort categories by amount (descending)
        let mut sorted_categories: Vec<_> = category_totals.iter().collect();
        sorted_categories.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        // Table header
        current_layer.text("Catégorie", 10.0, 20.0, y, &font_bold);
        current_layer.text("Montant", 10.0, 120.0, y, &font_bold);
        current_layer.text("% Total", 10.0, 160.0, y, &font_bold);
        y -= 6.0;

        for (category, amount) in sorted_categories {
            if y < 100.0 {
                // Reserve space for budget comparison
                break;
            }

            let percentage: Decimal = if total_expenses > Decimal::ZERO {
                (*amount / total_expenses) * dec!(100)
            } else {
                Decimal::ZERO
            };

            current_layer.text(category.clone(), 9.0, 20.0, y, &font);
            current_layer.text(format!("{:.2} €", amount), 9.0, 120.0, y, &font);
            current_layer.text(format!("{:.1}%", percentage), 9.0, 160.0, y, &font);
            y -= 5.0;
        }
        y -= 10.0;

        // === BUDGET VS ACTUAL ===
        current_layer.text(
            "COMPARAISON BUDGET / RÉALISÉ".to_string(),
            14.0,
            20.0,
            y,
            &font_bold,
        );
        y -= 8.0;

        // Table header
        current_layer.text("Catégorie", 10.0, 20.0, y, &font_bold);
        current_layer.text("Budget", 10.0, 100.0, y, &font_bold);
        current_layer.text("Réalisé", 10.0, 130.0, y, &font_bold);
        current_layer.text("Écart", 10.0, 160.0, y, &font_bold);
        y -= 6.0;

        let mut total_budgeted = Decimal::ZERO;
        let mut total_actual = Decimal::ZERO;

        for item in budget_items {
            if y < 50.0 {
                // Reserve space for signatures
                break;
            }

            let category_name = Self::category_name(&item.category);
            let variance = item.budgeted - item.actual;
            let variance_sign = if variance >= Decimal::ZERO { "+" } else { "" };

            current_layer.text(category_name, 9.0, 20.0, y, &font);
            current_layer.text(format!("{:.2} €", item.budgeted), 9.0, 100.0, y, &font);
            current_layer.text(format!("{:.2} €", item.actual), 9.0, 130.0, y, &font);
            current_layer.text(
                format!("{}{:.2} €", variance_sign, variance),
                9.0,
                160.0,
                y,
                &font,
            );

            total_budgeted += item.budgeted;
            total_actual += item.actual;
            y -= 5.0;
        }
        y -= 3.0;

        // Totals line
        current_layer.text("TOTAL", 10.0, 20.0, y, &font_bold);
        current_layer.text(
            format!("{:.2} €", total_budgeted),
            10.0,
            100.0,
            y,
            &font_bold,
        );
        current_layer.text(format!("{:.2} €", total_actual), 10.0, 130.0, y, &font_bold);

        let total_variance = total_budgeted - total_actual;
        let total_variance_sign = if total_variance >= Decimal::ZERO {
            "+"
        } else {
            ""
        };
        current_layer.text(
            format!("{}{:.2} €", total_variance_sign, total_variance),
            10.0,
            160.0,
            y,
            &font_bold,
        );
        y -= 15.0;

        // === SIGNATURES ===
        if y < 40.0 {
            y = 40.0;
        }

        current_layer.text("SIGNATURES".to_string(), 12.0, 20.0, y, &font_bold);
        y -= 10.0;

        current_layer.text(
            "Le Syndic: ________________".to_string(),
            10.0,
            20.0,
            y,
            &font,
        );

        current_layer.text(
            "Le Trésorier: ________________".to_string(),
            10.0,
            120.0,
            y,
            &font,
        );
        y -= 6.0;

        current_layer.text("Date: ________________".to_string(), 10.0, 20.0, y, &font);

        // Save to bytes
        let page = current_layer.into_page(210.0, 297.0);
        Ok(save_document(doc, page))
    }

    fn category_name(category: &ExpenseCategory) -> String {
        match category {
            ExpenseCategory::Maintenance => "Entretien".to_string(),
            ExpenseCategory::Utilities => "Charges courantes".to_string(),
            ExpenseCategory::Insurance => "Assurances".to_string(),
            ExpenseCategory::Repairs => "Réparations".to_string(),
            ExpenseCategory::Administration => "Administration".to_string(),
            ExpenseCategory::Cleaning => "Nettoyage".to_string(),
            ExpenseCategory::Works => "Travaux".to_string(),
            ExpenseCategory::Other => "Autres".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::ApprovalStatus;
    use uuid::Uuid;

    #[test]
    fn test_export_annual_report_pdf() {
        let test_org_id = Uuid::new_v4();
        let building = Building {
            id: Uuid::new_v4(),
            name: "Les Jardins de Bruxelles".to_string(),
            address: "123 Avenue Louise".to_string(),
            city: "Bruxelles".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
            total_units: 10,
            total_tantiemes: 1000,
            construction_year: Some(1990),
            syndic_name: None,
            syndic_email: None,
            syndic_phone: None,
            syndic_address: None,
            syndic_office_hours: None,
            syndic_emergency_contact: None,
            slug: None,
            acp_id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let expenses = vec![
            Expense {
                id: Uuid::new_v4(),
                acp_id: Uuid::new_v4(),
                building_id: building.id,
                organization_id: test_org_id,
                description: "Entretien ascenseur".to_string(),
                amount: dec!(1500),
                amount_excl_vat: Some(dec!(1239.67)),
                vat_rate: Some(dec!(21)),
                vat_amount: Some(dec!(260.33)),
                amount_incl_vat: Some(dec!(1500)),
                expense_date: Utc::now(),
                invoice_date: None,
                due_date: None,
                paid_date: Some(Utc::now()),
                category: ExpenseCategory::Maintenance,
                approval_status: ApprovalStatus::Approved,
                submitted_at: None,
                approved_by: None,
                approved_at: None,
                rejection_reason: None,
                payment_status: crate::domain::entities::PaymentStatus::Paid,
                supplier: None,
                invoice_number: Some("INV-001".to_string()),
                account_code: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                contractor_report_id: None,
            },
            Expense {
                id: Uuid::new_v4(),
                acp_id: Uuid::new_v4(),
                building_id: building.id,
                organization_id: test_org_id,
                description: "Électricité parties communes".to_string(),
                amount: dec!(800),
                amount_excl_vat: Some(dec!(661.16)),
                vat_rate: Some(dec!(21)),
                vat_amount: Some(dec!(138.84)),
                amount_incl_vat: Some(dec!(800)),
                expense_date: Utc::now(),
                invoice_date: None,
                due_date: None,
                paid_date: Some(Utc::now()),
                category: ExpenseCategory::Utilities,
                approval_status: ApprovalStatus::Approved,
                submitted_at: None,
                approved_by: None,
                approved_at: None,
                rejection_reason: None,
                payment_status: crate::domain::entities::PaymentStatus::Paid,
                supplier: None,
                invoice_number: Some("INV-002".to_string()),
                account_code: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                contractor_report_id: None,
            },
        ];

        let budget_items = vec![
            BudgetItem {
                category: ExpenseCategory::Maintenance,
                budgeted: dec!(2000),
                actual: dec!(1500),
            },
            BudgetItem {
                category: ExpenseCategory::Utilities,
                budgeted: dec!(1000),
                actual: dec!(800),
            },
        ];

        let result = AnnualReportExporter::export_to_pdf(
            &building,
            2025,
            &expenses,
            &budget_items,
            dec!(3000), // Total income
            dec!(5000), // Reserve fund
        );

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.len() > 100);
    }
}
