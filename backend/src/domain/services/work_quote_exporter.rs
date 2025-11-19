use crate::domain::entities::{Building, Expense, ExpenseCategory};
use printpdf::*;
use std::io::BufWriter;

/// Work Quote Document Exporter - Generates PDF for Devis de Travaux
///
/// Generates detailed work quotes for building maintenance and renovations.
pub struct WorkQuoteExporter;

#[derive(Debug, Clone)]
pub struct QuoteLineItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total: f64,
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
            return Err("Expense must be work-related category (Maintenance/Repairs/Insurance)".to_string());
        }

        // Create PDF document (A4: 210mm x 297mm)
        let (doc, page1, layer1) = PdfDocument::new(
            "Devis de Travaux",
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
            "DEVIS DE TRAVAUX".to_string(),
            18.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 15.0;

        // Quote information
        if let Some(ref invoice_num) = expense.invoice_number {
            current_layer.use_text(
                format!("Devis N°: {}", invoice_num),
                11.0,
                Mm(20.0),
                Mm(y),
                &font_bold,
            );
            y -= 7.0;
        }

        current_layer.use_text(
            format!("Date: {}", expense.expense_date.format("%d/%m/%Y")),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 10.0;

        // Building information
        current_layer.use_text(
            "COPROPRIÉTÉ".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        current_layer.use_text(
            building.name.clone(),
            11.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        current_layer.use_text(
            format!("{}, {} {}", building.address, building.postal_code, building.city),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 10.0;

        // Contractor information
        current_layer.use_text(
            "PRESTATAIRE".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        current_layer.use_text(
            contractor_name.to_string(),
            11.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        current_layer.use_text(
            contractor_contact.to_string(),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 10.0;

        // Work description
        current_layer.use_text(
            "DESCRIPTION DES TRAVAUX".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        // Wrap long description
        let description_lines = Self::wrap_text(&expense.description, 80);
        for line in description_lines {
            current_layer.use_text(line, 10.0, Mm(20.0), Mm(y), &font);
            y -= 6.0;
        }
        y -= 5.0;

        // Timeline
        current_layer.use_text(
            format!("Délai d'exécution: {}", timeline),
            10.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 10.0;

        // === LINE ITEMS ===
        current_layer.use_text(
            "DÉTAIL DU DEVIS".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        // Table header
        current_layer.use_text("Description", 10.0, Mm(20.0), Mm(y), &font_bold);
        current_layer.use_text("Quantité", 10.0, Mm(110.0), Mm(y), &font_bold);
        current_layer.use_text("Prix Unit.", 10.0, Mm(140.0), Mm(y), &font_bold);
        current_layer.use_text("Total", 10.0, Mm(170.0), Mm(y), &font_bold);
        y -= 6.0;

        let mut subtotal = 0.0;

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
            current_layer.use_text(desc, 9.0, Mm(20.0), Mm(y), &font);

            current_layer.use_text(
                format!("{:.2}", item.quantity),
                9.0,
                Mm(110.0),
                Mm(y),
                &font,
            );

            current_layer.use_text(
                format!("{:.2} €", item.unit_price),
                9.0,
                Mm(140.0),
                Mm(y),
                &font,
            );

            current_layer.use_text(
                format!("{:.2} €", item.total),
                9.0,
                Mm(170.0),
                Mm(y),
                &font,
            );

            subtotal += item.total;
            y -= 5.0;
        }
        y -= 8.0;

        // === TOTALS ===
        current_layer.use_text(
            format!("SOUS-TOTAL: {:.2} €", subtotal),
            11.0,
            Mm(140.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        let tva = subtotal * 0.21; // Belgian VAT 21% for work
        current_layer.use_text(
            format!("TVA (21%): {:.2} €", tva),
            11.0,
            Mm(140.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        let total = subtotal + tva;
        current_layer.use_text(
            format!("TOTAL TTC: {:.2} €", total),
            12.0,
            Mm(140.0),
            Mm(y),
            &font_bold,
        );
        y -= 10.0;

        // Approval status
        let approval_text = match expense.approval_status {
            crate::domain::entities::ApprovalStatus::Approved => "✓ Devis APPROUVÉ",
            crate::domain::entities::ApprovalStatus::Rejected => "✗ Devis REJETÉ",
            crate::domain::entities::ApprovalStatus::PendingApproval => "○ En attente d'approbation",
            crate::domain::entities::ApprovalStatus::Draft => "○ Brouillon",
        };

        current_layer.use_text(
            approval_text.to_string(),
            11.0,
            Mm(20.0),
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
            "Le Prestataire: ________________".to_string(),
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

        let expense = Expense {
            id: Uuid::new_v4(),
            building_id: building.id,
            organization_id: building.organization_id,
            description: "Rénovation de la façade principale".to_string(),
            amount: 15000.0,
            amount_excl_vat: Some(12396.69),
            vat_rate: Some(21.0),
            vat_amount: Some(2603.31),
            amount_incl_vat: Some(15000.0),
            expense_date: Utc::now(),
            invoice_date: Some(Utc::now()),
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let line_items = vec![
            QuoteLineItem {
                description: "Nettoyage haute pression".to_string(),
                quantity: 100.0,
                unit_price: 15.0,
                total: 1500.0,
            },
            QuoteLineItem {
                description: "Réparation briques endommagées".to_string(),
                quantity: 50.0,
                unit_price: 25.0,
                total: 1250.0,
            },
            QuoteLineItem {
                description: "Peinture façade".to_string(),
                quantity: 100.0,
                unit_price: 20.0,
                total: 2000.0,
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
