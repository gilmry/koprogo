use crate::domain::entities::{Building, Expense, ExpenseCategory};
use chrono::Utc;
use printpdf::*;
use std::collections::HashMap;
use std::io::BufWriter;

/// Annual Financial Report Exporter - Generates PDF for Rapport Financier Annuel
///
/// Generates comprehensive annual financial reports with expense breakdowns.
pub struct AnnualReportExporter;

#[derive(Debug, Clone)]
pub struct BudgetItem {
    pub category: ExpenseCategory,
    pub budgeted: f64,
    pub actual: f64,
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
        total_income: f64,
        reserve_fund: f64,
    ) -> Result<Vec<u8>, String> {
        // Create PDF document (A4: 210mm x 297mm)
        let (doc, page1, layer1) = PdfDocument::new(
            "Rapport Financier Annuel",
            Mm(210.0),
            Mm(297.0),
            "Layer 1",
        );
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Load fonts
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| e.to_string())?;
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| e.to_string())?;

        let mut y = 270.0; // Start from top

        // === HEADER ===
        current_layer.use_text(
            "RAPPORT FINANCIER ANNUEL".to_string(),
            18.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 15.0;

        // Building information
        current_layer.use_text(
            format!("Copropriété: {}", building.name),
            12.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 7.0;

        current_layer.use_text(
            format!("Adresse: {}", building.address),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 10.0;

        current_layer.use_text(
            format!("Exercice: {}", year),
            12.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 10.0;

        current_layer.use_text(
            format!("Date d'établissement: {}", Utc::now().format("%d/%m/%Y")),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 15.0;

        // === SUMMARY ===
        current_layer.use_text(
            "SYNTHÈSE FINANCIÈRE".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        let total_expenses: f64 = expenses.iter().map(|e| e.amount).sum();

        current_layer.use_text(
            format!("Total des produits (charges perçues): {:.2} €", total_income),
            11.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        current_layer.use_text(
            format!("Total des charges: {:.2} €", total_expenses),
            11.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        let balance = total_income - total_expenses;
        let balance_label = if balance >= 0.0 { "Excédent" } else { "Déficit" };
        current_layer.use_text(
            format!("{}: {:.2} €", balance_label, balance.abs()),
            12.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 6.0;

        current_layer.use_text(
            format!("Fonds de réserve: {:.2} €", reserve_fund),
            11.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 12.0;

        // === EXPENSE BREAKDOWN BY CATEGORY ===
        current_layer.use_text(
            "RÉPARTITION DES CHARGES PAR CATÉGORIE".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        // Calculate expenses by category
        let mut category_totals: HashMap<String, f64> = HashMap::new();
        for expense in expenses {
            let category_name = Self::category_name(&expense.category);
            *category_totals.entry(category_name).or_insert(0.0) += expense.amount;
        }

        // Sort categories by amount (descending)
        let mut sorted_categories: Vec<_> = category_totals.iter().collect();
        sorted_categories.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        // Table header
        current_layer.use_text("Catégorie", 10.0, Mm(20.0), Mm(y), &font_bold);
        current_layer.use_text("Montant", 10.0, Mm(120.0), Mm(y), &font_bold);
        current_layer.use_text("% Total", 10.0, Mm(160.0), Mm(y), &font_bold);
        y -= 6.0;

        for (category, amount) in sorted_categories {
            if y < 100.0 {
                // Reserve space for budget comparison
                break;
            }

            let percentage = if total_expenses > 0.0 {
                (amount / total_expenses) * 100.0
            } else {
                0.0
            };

            current_layer.use_text(category.clone(), 9.0, Mm(20.0), Mm(y), &font);
            current_layer.use_text(
                format!("{:.2} €", amount),
                9.0,
                Mm(120.0),
                Mm(y),
                &font,
            );
            current_layer.use_text(
                format!("{:.1}%", percentage),
                9.0,
                Mm(160.0),
                Mm(y),
                &font,
            );
            y -= 5.0;
        }
        y -= 10.0;

        // === BUDGET VS ACTUAL ===
        current_layer.use_text(
            "COMPARAISON BUDGET / RÉALISÉ".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        // Table header
        current_layer.use_text("Catégorie", 10.0, Mm(20.0), Mm(y), &font_bold);
        current_layer.use_text("Budget", 10.0, Mm(100.0), Mm(y), &font_bold);
        current_layer.use_text("Réalisé", 10.0, Mm(130.0), Mm(y), &font_bold);
        current_layer.use_text("Écart", 10.0, Mm(160.0), Mm(y), &font_bold);
        y -= 6.0;

        let mut total_budgeted = 0.0;
        let mut total_actual = 0.0;

        for item in budget_items {
            if y < 50.0 {
                // Reserve space for signatures
                break;
            }

            let category_name = Self::category_name(&item.category);
            let variance = item.budgeted - item.actual;
            let variance_sign = if variance >= 0.0 { "+" } else { "" };

            current_layer.use_text(category_name, 9.0, Mm(20.0), Mm(y), &font);
            current_layer.use_text(
                format!("{:.2} €", item.budgeted),
                9.0,
                Mm(100.0),
                Mm(y),
                &font,
            );
            current_layer.use_text(
                format!("{:.2} €", item.actual),
                9.0,
                Mm(130.0),
                Mm(y),
                &font,
            );
            current_layer.use_text(
                format!("{}{:.2} €", variance_sign, variance),
                9.0,
                Mm(160.0),
                Mm(y),
                &font,
            );

            total_budgeted += item.budgeted;
            total_actual += item.actual;
            y -= 5.0;
        }
        y -= 3.0;

        // Totals line
        current_layer.use_text("TOTAL", 10.0, Mm(20.0), Mm(y), &font_bold);
        current_layer.use_text(
            format!("{:.2} €", total_budgeted),
            10.0,
            Mm(100.0),
            Mm(y),
            &font_bold,
        );
        current_layer.use_text(
            format!("{:.2} €", total_actual),
            10.0,
            Mm(130.0),
            Mm(y),
            &font_bold,
        );

        let total_variance = total_budgeted - total_actual;
        let total_variance_sign = if total_variance >= 0.0 { "+" } else { "" };
        current_layer.use_text(
            format!("{}{:.2} €", total_variance_sign, total_variance),
            10.0,
            Mm(160.0),
            Mm(y),
            &font_bold,
        );
        y -= 15.0;

        // === SIGNATURES ===
        if y < 40.0 {
            y = 40.0;
        }

        current_layer.use_text(
            "SIGNATURES".to_string(),
            12.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 10.0;

        current_layer.use_text(
            "Le Syndic: ________________".to_string(),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );

        current_layer.use_text(
            "Le Trésorier: ________________".to_string(),
            10.0,
            Mm(120.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        current_layer.use_text(
            "Date: ________________".to_string(),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );

        // Save to bytes
        let mut buffer = Vec::new();
        doc.save(&mut BufWriter::new(&mut buffer))
            .map_err(|e| e.to_string())?;

        Ok(buffer)
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
            organization_id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let expenses = vec![
            Expense {
                id: Uuid::new_v4(),
                building_id: building.id,
                organization_id: building.organization_id,
                description: "Entretien ascenseur".to_string(),
                amount: 1500.0,
                amount_excl_vat: Some(1239.67),
                vat_rate: Some(21.0),
                vat_amount: Some(260.33),
                amount_incl_vat: Some(1500.0),
                expense_date: Utc::now(),
                invoice_date: Some(Utc::now()),
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
            },
            Expense {
                id: Uuid::new_v4(),
                building_id: building.id,
                organization_id: building.organization_id,
                description: "Électricité parties communes".to_string(),
                amount: 800.0,
                amount_excl_vat: Some(661.16),
                vat_rate: Some(21.0),
                vat_amount: Some(138.84),
                amount_incl_vat: Some(800.0),
                expense_date: Utc::now(),
                invoice_date: Some(Utc::now()),
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
            },
        ];

        let budget_items = vec![
            BudgetItem {
                category: ExpenseCategory::Maintenance,
                budgeted: 2000.0,
                actual: 1500.0,
            },
            BudgetItem {
                category: ExpenseCategory::Utilities,
                budgeted: 1000.0,
                actual: 800.0,
            },
        ];

        let result = AnnualReportExporter::export_to_pdf(
            &building,
            2025,
            &expenses,
            &budget_items,
            3000.0, // Total income
            5000.0, // Reserve fund
        );

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.len() > 100);
    }
}
