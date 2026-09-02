use crate::domain::entities::{Building, Expense, ExpenseCategory};
use crate::domain::services::pdf_writer::{
    builtin_font, new_document, save_document, PdfPageBuilder,
};
use printpdf::BuiltinFont;
use rust_decimal::Decimal;

/// Work Quote Document Exporter - Generates PDF for Devis de Travaux
///
/// Generates detailed work quotes for building maintenance and renovations.
///
/// MONETARY: quantity/unit_price/total use rust_decimal::Decimal (cf. ADR-0007).
pub struct WorkQuoteExporter;

#[derive(Debug, Clone)]
pub struct QuoteLineItem {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub total: Decimal,
}

impl WorkQuoteExporter {
    /// Export work quote to PDF bytes
    ///
    /// Generates a Devis de Travaux including:
    /// - Building information
    /// - Work description
    /// - Cost breakdown
    /// - Timeline
    /// - Approval status
    /// - Signatures section
    pub fn export_to_pdf(
        building: &Building,
        expense: &Expense,
        line_items: &[QuoteLineItem],
        contractor_name: &str,
        contractor_contact: &str,
        timeline: &str,
    ) -> Result<Vec<u8>, String> {
        // Validate that expense is a work-related category
        if !matches!(
            expense.category,
            ExpenseCategory::Maintenance | ExpenseCategory::Repairs | ExpenseCategory::Insurance
        ) {
            return Err(
                "Expense must be work-related category (Maintenance/Repairs/Insurance)".to_string(),
            );
        }

        // Create PDF document (A4: 210mm x 297mm)
        let doc = new_document("Devis de Travaux");
        let mut current_layer = PdfPageBuilder::new();

        // Load fonts
        let font = builtin_font(BuiltinFont::Helvetica);
        let font_bold = builtin_font(BuiltinFont::HelveticaBold);

        let mut y = 270.0; // Start from top

        // === HEADER ===
        current_layer.text("DEVIS DE TRAVAUX".to_string(), 18.0, 20.0, y, &font_bold);
        y -= 15.0;

        // Quote information
        if let Some(ref invoice_num) = expense.invoice_number {
            current_layer.text(
                format!("Devis N°: {}", invoice_num),
                11.0,
                20.0,
                y,
                &font_bold,
            );
            y -= 7.0;
        }

        current_layer.text(
            format!("Date: {}", expense.expense_date.format("%d/%m/%Y")),
            10.0,
            20.0,
            y,
            &font,
        );
        y -= 10.0;

        // Building information
        current_layer.text("COPROPRIÉTÉ".to_string(), 14.0, 20.0, y, &font_bold);
        y -= 8.0;

        current_layer.text(building.name.clone(), 11.0, 20.0, y, &font);
        y -= 6.0;

        current_layer.text(
            format!(
                "{}, {} {}",
                building.address, building.postal_code, building.city
            ),
            10.0,
            20.0,
            y,
            &font,
        );
        y -= 10.0;

        // Contractor information
        current_layer.text("PRESTATAIRE".to_string(), 14.0, 20.0, y, &font_bold);
        y -= 8.0;

        current_layer.text(contractor_name.to_string(), 11.0, 20.0, y, &font);
        y -= 6.0;

        current_layer.text(contractor_contact.to_string(), 10.0, 20.0, y, &font);
        y -= 10.0;

        // Work description
        current_layer.text(
            "DESCRIPTION DES TRAVAUX".to_string(),
            14.0,
            20.0,
            y,
            &font_bold,
        );
        y -= 8.0;

        // Wrap long description
        let description_lines = Self::wrap_text(&expense.description, 80);
        for line in description_lines {
            current_layer.text(line, 10.0, 20.0, y, &font);
            y -= 6.0;
        }
        y -= 5.0;

        // Timeline
        current_layer.text(
            format!("Délai d'exécution: {}", timeline),
            10.0,
            20.0,
            y,
            &font_bold,
        );
        y -= 10.0;

        // === LINE ITEMS ===
        current_layer.text("DÉTAIL DU DEVIS".to_string(), 14.0, 20.0, y, &font_bold);
        y -= 8.0;

        // Table header
        current_layer.text("Description", 10.0, 20.0, y, &font_bold);
        current_layer.text("Quantité", 10.0, 110.0, y, &font_bold);
        current_layer.text("Prix Unit.", 10.0, 140.0, y, &font_bold);
        current_layer.text("Total", 10.0, 170.0, y, &font_bold);
        y -= 6.0;

        let mut subtotal = Decimal::ZERO;

        for item in line_items {
            if y < 80.0 {
                // Reserve space for totals and signatures
                break;
            }

            let desc = if item.description.len() > 40 {
                format!("{}...", &item.description[..40])
            } else {
                item.description.clone()
            };
            current_layer.text(desc, 9.0, 20.0, y, &font);

            current_layer.text(format!("{:.2}", item.quantity), 9.0, 110.0, y, &font);

            current_layer.text(format!("{:.2} €", item.unit_price), 9.0, 140.0, y, &font);

            current_layer.text(format!("{:.2} €", item.total), 9.0, 170.0, y, &font);

            subtotal += item.total;
            y -= 5.0;
        }
        y -= 8.0;

        // === TOTALS ===
        current_layer.text(
            format!("SOUS-TOTAL: {:.2} €", subtotal),
            11.0,
            140.0,
            y,
            &font,
        );
        y -= 6.0;

        let tva = subtotal * rust_decimal_macros::dec!(0.21); // Belgian VAT 21% for work
        current_layer.text(format!("TVA (21%): {:.2} €", tva), 11.0, 140.0, y, &font);
        y -= 6.0;

        let total = subtotal + tva;
        current_layer.text(
            format!("TOTAL TTC: {:.2} €", total),
            12.0,
            140.0,
            y,
            &font_bold,
        );
        y -= 10.0;

        // Approval status
        let approval_text = match expense.approval_status {
            crate::domain::entities::ApprovalStatus::Approved => "✓ Devis APPROUVÉ",
            crate::domain::entities::ApprovalStatus::Rejected => "✗ Devis REJETÉ",
            crate::domain::entities::ApprovalStatus::PendingApproval => {
                "○ En attente d'approbation"
            }
            crate::domain::entities::ApprovalStatus::Draft => "○ Brouillon",
        };

        current_layer.text(approval_text.to_string(), 11.0, 20.0, y, &font_bold);
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
            "Le Prestataire: ________________".to_string(),
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

    fn wrap_text(text: &str, max_len: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            if current_line.len() + word.len() + 1 > max_len {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
            }
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::ApprovalStatus;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_export_work_quote_pdf() {
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

        let expense = Expense {
            id: Uuid::new_v4(),
            acp_id: Uuid::new_v4(),
            building_id: building.id,
            organization_id: test_org_id,
            description: "Rénovation de la façade principale".to_string(),
            amount: rust_decimal_macros::dec!(15000),
            amount_excl_vat: Some(rust_decimal_macros::dec!(12396.69)),
            vat_rate: Some(rust_decimal_macros::dec!(21)),
            vat_amount: Some(rust_decimal_macros::dec!(2603.31)),
            amount_incl_vat: Some(rust_decimal_macros::dec!(15000)),
            expense_date: Utc::now(),
            invoice_date: None,
            due_date: None,
            paid_date: None,
            category: ExpenseCategory::Maintenance,
            approval_status: ApprovalStatus::PendingApproval,
            submitted_at: None,
            approved_by: None,
            approved_at: None,
            rejection_reason: None,
            payment_status: crate::domain::entities::PaymentStatus::Pending,
            supplier: None,
            invoice_number: Some("DEV-2025-001".to_string()),
            account_code: None,
            contractor_report_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let line_items = vec![
            QuoteLineItem {
                description: "Nettoyage haute pression".to_string(),
                quantity: rust_decimal_macros::dec!(100),
                unit_price: rust_decimal_macros::dec!(15),
                total: rust_decimal_macros::dec!(1500),
            },
            QuoteLineItem {
                description: "Réparation briques endommagées".to_string(),
                quantity: rust_decimal_macros::dec!(50),
                unit_price: rust_decimal_macros::dec!(25),
                total: rust_decimal_macros::dec!(1250),
            },
            QuoteLineItem {
                description: "Peinture façade".to_string(),
                quantity: rust_decimal_macros::dec!(100),
                unit_price: rust_decimal_macros::dec!(20),
                total: rust_decimal_macros::dec!(2000),
            },
        ];

        let result = WorkQuoteExporter::export_to_pdf(
            &building,
            &expense,
            &line_items,
            "BatiPro SPRL",
            "contact@batipro.be | +32 2 555 66 77",
            "4 semaines",
        );

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.len() > 100);
    }
}
