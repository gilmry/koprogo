use crate::domain::entities::{Building, Expense, Owner, Unit};
use chrono::{DateTime, Utc};
use printpdf::*;
use std::io::BufWriter;

/// Owner Financial Statement Exporter - Generates PDF for Relevé de Charges
///
/// Generates statements showing an owner's expenses over a period.
pub struct OwnerStatementExporter;

#[derive(Debug, Clone)]
pub struct UnitWithOwnership {
    pub unit: Unit,
    pub ownership_percentage: f64, // 0.0 to 1.0
}

impl OwnerStatementExporter {
    /// Export owner financial statement to PDF bytes
    ///
    /// Generates a Relevé de Charges including:
    /// - Owner information
    /// - Period covered
    /// - Units owned with percentages
    /// - Expense breakdown by category
    /// - Payment status
    /// - Total due
    pub fn export_to_pdf(
        owner: &Owner,
        building: &Building,
        units: &[UnitWithOwnership],
        expenses: &[Expense],
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<u8>, String> {
        // Create PDF document (A4: 210mm x 297mm)
        let (doc, page1, layer1) = PdfDocument::new(
            "Relevé de Charges",
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
            "RELEVÉ DE CHARGES".to_string(),
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

        // Period
        let period = format!(
            "Période: du {} au {}",
            start_date.format("%d/%m/%Y"),
            end_date.format("%d/%m/%Y")
        );
        current_layer.use_text(period, 10.0, Mm(20.0), Mm(y), &font);
        y -= 10.0;

        // Owner information
        current_layer.use_text(
            "COPROPRIÉTAIRE".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        current_layer.use_text(
            format!("{} {}", owner.first_name, owner.last_name),
            11.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        current_layer.use_text(
            format!("Email: {}", owner.email),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        if let Some(ref phone) = owner.phone {
            current_layer.use_text(
                format!("Téléphone: {}", phone),
                10.0,
                Mm(20.0),
                Mm(y),
                &font,
            );
            y -= 6.0;
        }
        y -= 5.0;

        // === UNITS OWNED ===
        current_layer.use_text(
            "LOTS DÉTENUS".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        current_layer.use_text("Lot", 10.0, Mm(20.0), Mm(y), &font_bold);
        current_layer.use_text("Étage", 10.0, Mm(60.0), Mm(y), &font_bold);
        current_layer.use_text("Surface", 10.0, Mm(90.0), Mm(y), &font_bold);
        current_layer.use_text("Quote-part", 10.0, Mm(130.0), Mm(y), &font_bold);
        y -= 6.0;

        for unit_info in units {
            if y < 100.0 {
                // Reserve space for totals
                break;
            }

            current_layer.use_text(
                &unit_info.unit.unit_number,
                9.0,
                Mm(20.0),
                Mm(y),
                &font,
            );

            if let Some(floor) = unit_info.unit.floor {
                current_layer.use_text(
                    floor.to_string(),
                    9.0,
                    Mm(60.0),
                    Mm(y),
                    &font,
                );
            }

            current_layer.use_text(
                format!("{:.2} m²", unit_info.unit.surface_area),
                9.0,
                Mm(90.0),
                Mm(y),
                &font,
            );

            current_layer.use_text(
                format!("{:.2}%", unit_info.ownership_percentage * 100.0),
                9.0,
                Mm(130.0),
                Mm(y),
                &font,
            );
            y -= 5.0;
        }
        y -= 8.0;

        // === EXPENSES ===
        current_layer.use_text(
            "DÉTAIL DES CHARGES".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        current_layer.use_text("Date", 10.0, Mm(20.0), Mm(y), &font_bold);
        current_layer.use_text("Description", 10.0, Mm(50.0), Mm(y), &font_bold);
        current_layer.use_text("Montant", 10.0, Mm(140.0), Mm(y), &font_bold);
        current_layer.use_text("Statut", 10.0, Mm(170.0), Mm(y), &font_bold);
        y -= 6.0;

        let mut total_amount = 0.0;
        let mut total_paid = 0.0;

        for expense in expenses {
            if y < 50.0 {
                // Reserve space for footer
                break;
            }

            current_layer.use_text(
                expense.expense_date.format("%d/%m/%Y").to_string(),
                9.0,
                Mm(20.0),
                Mm(y),
                &font,
            );

            let description = if expense.description.len() > 30 {
                format!("{}...", &expense.description[..30])
            } else {
                expense.description.clone()
            };
            current_layer.use_text(description, 9.0, Mm(50.0), Mm(y), &font);

            current_layer.use_text(
                format!("{:.2} €", expense.amount),
                9.0,
                Mm(140.0),
                Mm(y),
                &font,
            );

            let status = if expense.is_paid() { "Payée" } else { "En attente" };
            current_layer.use_text(status.to_string(), 9.0, Mm(170.0), Mm(y), &font);

            total_amount += expense.amount;
            if expense.is_paid() {
                total_paid += expense.amount;
            }

            y -= 5.0;
        }
        y -= 10.0;

        // === SUMMARY ===
        current_layer.use_text(
            "RÉCAPITULATIF".to_string(),
            14.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        current_layer.use_text(
            format!("Total des charges: {:.2} €", total_amount),
            11.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        current_layer.use_text(
            format!("Montant payé: {:.2} €", total_paid),
            11.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        let amount_due = total_amount - total_paid;
        current_layer.use_text(
            format!("Montant dû: {:.2} €", amount_due),
            12.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 10.0;

        // Payment instructions
        if amount_due > 0.0 {
            current_layer.use_text(
                "Modalités de paiement:".to_string(),
                10.0,
                Mm(20.0),
                Mm(y),
                &font_bold,
            );
            y -= 6.0;

            current_layer.use_text(
                "Merci d'effectuer votre paiement par virement bancaire".to_string(),
                9.0,
                Mm(20.0),
                Mm(y),
                &font,
            );
            y -= 5.0;

            current_layer.use_text(
                "avec la référence suivante en communication.".to_string(),
                9.0,
                Mm(20.0),
                Mm(y),
                &font,
            );
        }

        // Save to bytes
        let mut buffer = Vec::new();
        doc.save(&mut BufWriter::new(&mut buffer))
            .map_err(|e| e.to_string())?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::ExpenseCategory;
    use uuid::Uuid;

    #[test]
    fn test_export_owner_statement_pdf() {
        let owner = Owner {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            user_id: None,
            first_name: "Jean".to_string(),
            last_name: "Dupont".to_string(),
            email: "jean@example.com".to_string(),
            phone: Some("+32 2 123 45 67".to_string()),
            address: "123 Rue Example".to_string(),
            city: "Bruxelles".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

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
            organization_id: owner.organization_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let unit = Unit {
            id: Uuid::new_v4(),
            organization_id: building.organization_id,
            building_id: building.id,
            unit_number: "A1".to_string(),
            unit_type: crate::domain::entities::UnitType::Apartment,
            floor: Some(1),
            surface_area: 75.5,
            quota: 150.0, // 150 millièmes (15%)
            owner_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let units = vec![UnitWithOwnership {
            unit,
            ownership_percentage: 0.15, // 15%
        }];

        let expenses = vec![Expense {
            id: Uuid::new_v4(),
            building_id: building.id,
            organization_id: building.organization_id,
            description: "Entretien ascenseur".to_string(),
            amount: 150.0,
            amount_excl_vat: Some(123.97),
            vat_rate: Some(21.0),
            vat_amount: Some(26.03),
            amount_incl_vat: Some(150.0),
            expense_date: Utc::now(),
            invoice_date: Some(Utc::now()),
            due_date: None,
            paid_date: None,
            category: ExpenseCategory::Maintenance,
            approval_status: crate::domain::entities::ApprovalStatus::Approved,
            submitted_at: None,
            approved_by: None,
            approved_at: None,
            rejection_reason: None,
            payment_status: crate::domain::entities::PaymentStatus::Pending,
            supplier: None,
            invoice_number: Some("INV-001".to_string()),
            account_code: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        let result = OwnerStatementExporter::export_to_pdf(
            &owner,
            &building,
            &units,
            &expenses,
            Utc::now() - chrono::Duration::days(30),
            Utc::now(),
        );

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.len() > 100);
    }
}
