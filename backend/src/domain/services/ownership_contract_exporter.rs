use crate::domain::entities::{Building, Owner, Unit};
use chrono::{DateTime, Utc};
use printpdf::*;
use std::io::BufWriter;

/// Ownership Contract Exporter - Generates PDF for Contrat de Copropriété
///
/// Generates formal ownership contracts for unit purchases.
pub struct OwnershipContractExporter;

impl OwnershipContractExporter {
    /// Export ownership contract to PDF bytes
    ///
    /// Generates a Contrat de Copropriété including:
    /// - Building information
    /// - Unit details (number, floor, area, tantièmes)
    /// - Owner information
    /// - Ownership start date
    /// - Percentage owned
    /// - Rights and obligations
    /// - General assembly rules
    /// - Expense allocation rules
    pub fn export_to_pdf(
        building: &Building,
        unit: &Unit,
        owner: &Owner,
        ownership_percentage: f64, // 0.0 to 1.0
        ownership_start_date: DateTime<Utc>,
    ) -> Result<Vec<u8>, String> {
        // Create PDF document (A4: 210mm x 297mm)
        let (doc, page1, layer1) =
            PdfDocument::new("Contrat de Copropriété", Mm(210.0), Mm(297.0), "Layer 1");
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
            "CONTRAT DE COPROPRIÉTÉ".to_string(),
            18.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 15.0;

        current_layer.use_text(
            format!("Date d'établissement: {}", Utc::now().format("%d/%m/%Y")),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 15.0;

        // === ARTICLE 1: BUILDING INFORMATION ===
        current_layer.use_text(
            "ARTICLE 1 - IMMEUBLE CONCERNÉ".to_string(),
            12.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        current_layer.use_text(
            format!("Dénomination: {}", building.name),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        current_layer.use_text(
            format!(
                "Adresse: {}, {} {}, {}",
                building.address, building.postal_code, building.city, building.country
            ),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        current_layer.use_text(
            format!("Nombre total de lots: {}", building.total_units),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        if let Some(year) = building.construction_year {
            current_layer.use_text(
                format!("Année de construction: {}", year),
                10.0,
                Mm(20.0),
                Mm(y),
                &font,
            );
            y -= 6.0;
        }
        y -= 8.0;

        // === ARTICLE 2: UNIT DETAILS ===
        current_layer.use_text(
            "ARTICLE 2 - DESCRIPTION DU LOT".to_string(),
            12.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        current_layer.use_text(
            format!("Numéro de lot: {}", unit.unit_number),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        if let Some(floor) = unit.floor {
            current_layer.use_text(format!("Étage: {}", floor), 10.0, Mm(20.0), Mm(y), &font);
            y -= 6.0;
        }

        current_layer.use_text(
            format!("Superficie: {:.2} m²", unit.surface_area),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        current_layer.use_text(
            format!("Type: {:?}", unit.unit_type),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        let tantiemes = (ownership_percentage * building.total_tantiemes as f64) as i32;
        current_layer.use_text(
            format!("Tantièmes: {} sur {}", tantiemes, building.total_tantiemes),
            10.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 6.0;

        current_layer.use_text(
            format!("Quote-part: {:.2}%", ownership_percentage * 100.0),
            10.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        // === ARTICLE 3: OWNER INFORMATION ===
        current_layer.use_text(
            "ARTICLE 3 - COPROPRIÉTAIRE".to_string(),
            12.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        let owner_name = format!("{} {}", owner.first_name, owner.last_name);

        current_layer.use_text(format!("Nom: {}", owner_name), 10.0, Mm(20.0), Mm(y), &font);
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

        current_layer.use_text(
            format!(
                "Date d'entrée en copropriété: {}",
                ownership_start_date.format("%d/%m/%Y")
            ),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 8.0;

        // === ARTICLE 4: RIGHTS AND OBLIGATIONS ===
        current_layer.use_text(
            "ARTICLE 4 - DROITS ET OBLIGATIONS".to_string(),
            12.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        let rights_text = [
            "Le copropriétaire dispose des droits suivants:",
            "• Droit d'usage exclusif du lot ci-dessus désigné",
            "• Droit de participation aux assemblées générales",
            "• Droit de vote proportionnel à sa quote-part",
            "• Droit d'accès aux parties communes",
            "",
            "Le copropriétaire est tenu aux obligations suivantes:",
            "• Paiement des charges communes proportionnellement à sa quote-part",
            "• Respect du règlement de copropriété",
            "• Participation aux travaux votés en assemblée générale",
            "• Entretien de son lot privatif",
        ];

        for line in rights_text.iter() {
            if y < 80.0 {
                break;
            }
            current_layer.use_text(line.to_string(), 9.0, Mm(20.0), Mm(y), &font);
            y -= 5.0;
        }
        y -= 5.0;

        // === ARTICLE 5: EXPENSES ===
        current_layer.use_text(
            "ARTICLE 5 - RÉPARTITION DES CHARGES".to_string(),
            12.0,
            Mm(20.0),
            Mm(y),
            &font_bold,
        );
        y -= 8.0;

        current_layer.use_text(
            format!(
                "Les charges communes sont réparties selon la quote-part de {:.2}%",
                ownership_percentage * 100.0
            ),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 6.0;

        current_layer.use_text(
            "correspondant aux tantièmes du lot.".to_string(),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );
        y -= 10.0;

        // === SIGNATURES ===
        if y < 40.0 {
            y = 40.0;
        }

        current_layer.use_text("SIGNATURES".to_string(), 12.0, Mm(20.0), Mm(y), &font_bold);
        y -= 10.0;

        current_layer.use_text(
            "Le Syndic: ________________".to_string(),
            10.0,
            Mm(20.0),
            Mm(y),
            &font,
        );

        current_layer.use_text(
            "Le Copropriétaire: ________________".to_string(),
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

        current_layer.use_text(
            "Date: ________________".to_string(),
            10.0,
            Mm(120.0),
            Mm(y),
            &font,
        );

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
    use uuid::Uuid;

    #[test]
    fn test_export_ownership_contract_pdf() {
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

        let unit = Unit {
            id: Uuid::new_v4(),
            organization_id: building.organization_id,
            building_id: building.id,
            unit_number: "A1".to_string(),
            unit_type: crate::domain::entities::UnitType::Apartment,
            floor: Some(1),
            surface_area: 75.5,
            quota: 150.0,
            owner_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let owner = Owner {
            id: Uuid::new_v4(),
            organization_id: building.organization_id,
            user_id: None,
            first_name: "Jean".to_string(),
            last_name: "Dupont".to_string(),
            email: "jean@example.com".to_string(),
            phone: Some("+32 2 123 45 67".to_string()),
            address: "123 Rue de Test".to_string(),
            city: "Bruxelles".to_string(),
            postal_code: "1000".to_string(),
            country: "Belgium".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = OwnershipContractExporter::export_to_pdf(
            &building,
            &unit,
            &owner,
            0.15,                                     // 15% ownership
            Utc::now() - chrono::Duration::days(365), // Started 1 year ago
        );

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.len() > 100);
    }
}
