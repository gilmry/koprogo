use crate::domain::entities::{Building, Meeting, MeetingType, Resolution, Vote};
use crate::domain::services::pdf_writer::{
    builtin_font, new_document, save_document, PdfPageBuilder,
};
use printpdf::BuiltinFont;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

/// Meeting Minutes Exporter - Generates PDF for Procès-Verbal d'Assemblée Générale
///
/// Compliant with Belgian copropriété law requirements for general assembly minutes.
pub struct MeetingMinutesExporter;

#[derive(Debug, Clone)]
pub struct AttendeeInfo {
    pub owner_id: Uuid,
    pub name: String,
    pub email: String,
    pub voting_power: Decimal, // Millièmes/tantièmes (Decimal exact — ADR-0008)
    pub is_proxy: bool,
    pub proxy_for: Option<String>, // Name of owner being represented
}

#[derive(Debug, Clone)]
pub struct ResolutionWithVotes {
    pub resolution: Resolution,
    pub votes: Vec<Vote>,
}

impl MeetingMinutesExporter {
    /// Export meeting minutes to PDF bytes
    ///
    /// Generates a complete Procès-Verbal (PV) including:
    /// - Building information
    /// - Meeting details (date, type, location)
    /// - Attendees list with voting power
    /// - Quorum validation
    /// - Resolutions with detailed vote results
    /// - Signatures section
    pub fn export_to_pdf(
        building: &Building,
        meeting: &Meeting,
        attendees: &[AttendeeInfo],
        resolutions: &[ResolutionWithVotes],
    ) -> Result<Vec<u8>, String> {
        // Create PDF document (A4: 210mm x 297mm)
        let doc = new_document("Procès-Verbal d'Assemblée Générale");
        let mut current_layer = PdfPageBuilder::new();

        // Load fonts
        let font = builtin_font(BuiltinFont::Helvetica);
        let font_bold = builtin_font(BuiltinFont::HelveticaBold);

        let mut y = 270.0; // Start from top

        // === HEADER ===
        current_layer.text(
            "PROCÈS-VERBAL D'ASSEMBLÉE GÉNÉRALE".to_string(),
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

        // Meeting information
        let meeting_type_label = match meeting.meeting_type {
            MeetingType::Ordinary => "Assemblée Générale Ordinaire (AGO)",
            MeetingType::Extraordinary => "Assemblée Générale Extraordinaire (AGE)",
        };

        current_layer.text(
            format!("Type: {}", meeting_type_label),
            10.0,
            20.0,
            y,
            &font,
        );
        y -= 6.0;

        let date_str = meeting
            .scheduled_date
            .format("%d/%m/%Y à %H:%M")
            .to_string();
        current_layer.text(format!("Date: {}", date_str), 10.0, 20.0, y, &font);
        y -= 6.0;

        current_layer.text(format!("Lieu: {}", meeting.location), 10.0, 20.0, y, &font);
        y -= 6.0;
        y -= 5.0;

        // === ATTENDEES SECTION ===
        current_layer.text(
            "PRÉSENCES ET REPRÉSENTATIONS".to_string(),
            14.0,
            20.0,
            y,
            &font_bold,
        );
        y -= 8.0;

        // Calculate total voting power (Decimal exact — ADR-0008)
        let total_voting_power: Decimal = attendees.iter().map(|a| a.voting_power).sum();
        let total_millimes = Decimal::from(building.total_units) * dec!(1000); // 1000 millièmes/unit
        let quorum_percentage = if total_millimes > Decimal::ZERO {
            (total_voting_power / total_millimes * dec!(100))
                .to_f64()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        current_layer.text(
            format!(
                "Présents ou représentés: {} millièmes sur {} ({:.2}%)",
                total_voting_power, total_millimes, quorum_percentage
            ),
            10.0,
            20.0,
            y,
            &font,
        );
        y -= 10.0;

        // Attendees table header
        current_layer.text("Copropriétaire", 10.0, 20.0, y, &font_bold);
        current_layer.text("Millièmes", 10.0, 110.0, y, &font_bold);
        current_layer.text("Présence", 10.0, 150.0, y, &font_bold);
        y -= 6.0;

        // Attendees list
        for attendee in attendees {
            if y < 30.0 {
                // TODO: Add new page if needed (for now, truncate)
                break;
            }

            current_layer.text(attendee.name.as_str(), 9.0, 20.0, y, &font);
            current_layer.text(
                format!("{:.2}", attendee.voting_power),
                9.0,
                110.0,
                y,
                &font,
            );

            let presence = if attendee.is_proxy {
                if let Some(ref proxy_for) = attendee.proxy_for {
                    format!("Mandataire pour {}", proxy_for)
                } else {
                    "Mandataire".to_string()
                }
            } else {
                "Présent".to_string()
            };

            current_layer.text(presence, 9.0, 150.0, y, &font);
            y -= 5.0;
        }
        y -= 8.0;

        // Quorum validation
        let quorum_status = if quorum_percentage >= 50.0 {
            "✓ QUORUM ATTEINT"
        } else {
            "✗ QUORUM NON ATTEINT"
        };

        current_layer.text(quorum_status.to_string(), 11.0, 20.0, y, &font_bold);
        y -= 12.0;

        // === RESOLUTIONS SECTION ===
        current_layer.text(
            "RÉSOLUTIONS ET VOTES".to_string(),
            14.0,
            20.0,
            y,
            &font_bold,
        );
        y -= 10.0;

        for (idx, res_with_votes) in resolutions.iter().enumerate() {
            if y < 50.0 {
                // TODO: Add new page if needed
                break;
            }

            let resolution = &res_with_votes.resolution;

            // Resolution number and title
            current_layer.text(
                format!("Résolution n°{}: {}", idx + 1, resolution.title),
                11.0,
                20.0,
                y,
                &font_bold,
            );
            y -= 6.0;

            // Description (truncate if too long)
            let description = if resolution.description.len() > 80 {
                format!("{}...", &resolution.description[..80])
            } else {
                resolution.description.clone()
            };

            current_layer.text(description, 9.0, 25.0, y, &font);
            y -= 6.0;

            // Majority type
            let majority_label = match &resolution.majority_required {
                crate::domain::entities::MajorityType::Absolute => {
                    "Majorité absolue (Art. 3.88 §1)"
                }
                crate::domain::entities::MajorityType::TwoThirds => {
                    "Majorité des 2/3 (Art. 3.88 §1, 1°)"
                }
                crate::domain::entities::MajorityType::FourFifths => {
                    "Majorité des 4/5 (Art. 3.88 §1, 2°)"
                }
                crate::domain::entities::MajorityType::Unanimity => "Unanimité (Art. 3.88 §1, 3°)",
            };

            current_layer.text(
                format!("Majorité requise: {}", majority_label),
                9.0,
                25.0,
                y,
                &font,
            );
            y -= 6.0;

            // Vote results
            current_layer.text(
                format!(
                    "Pour: {} votes ({:.2} millièmes) | Contre: {} votes ({:.2} millièmes) | Abstention: {} votes ({:.2} millièmes)",
                    resolution.vote_count_pour,
                    resolution.total_voting_power_pour,
                    resolution.vote_count_contre,
                    resolution.total_voting_power_contre,
                    resolution.vote_count_abstention,
                    resolution.total_voting_power_abstention
                ),
                9.0,
                25.0,
                y,
                &font,
            );
            y -= 6.0;

            // Result
            let (result_text, result_symbol) = match &resolution.status {
                crate::domain::entities::ResolutionStatus::Adopted => ("ADOPTÉE", "✓"),
                crate::domain::entities::ResolutionStatus::Rejected => ("REJETÉE", "✗"),
                crate::domain::entities::ResolutionStatus::Pending => ("EN ATTENTE", "○"),
            };

            current_layer.text(
                format!("{} Résolution {}", result_symbol, result_text),
                10.0,
                25.0,
                y,
                &font_bold,
            );
            y -= 10.0;
        }

        // === SIGNATURES SECTION ===
        if y < 40.0 {
            y = 40.0; // Force to bottom of page
        } else {
            y -= 10.0;
        }

        current_layer.text("SIGNATURES".to_string(), 12.0, 20.0, y, &font_bold);
        y -= 10.0;

        current_layer.text(
            "Le Président de séance: ________________",
            10.0,
            20.0,
            y,
            &font,
        );

        current_layer.text("Le Secrétaire: ________________", 10.0, 120.0, y, &font);

        // Save to bytes
        let page = current_layer.into_page(210.0, 297.0);
        Ok(save_document(doc, page))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{MajorityType, MeetingStatus, ResolutionStatus, ResolutionType};
    use chrono::Utc;

    #[test]
    fn test_export_meeting_minutes_pdf() {
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
        let test_org_id = Uuid::new_v4();

        let meeting = Meeting {
            id: Uuid::new_v4(),
            organization_id: test_org_id,
            building_id: building.id,
            meeting_type: MeetingType::Ordinary,
            title: "Assemblée Générale Ordinaire".to_string(),
            description: Some("Ordre du jour: budget et travaux".to_string()),
            scheduled_date: Utc::now(),
            location: "Salle communale".to_string(),
            status: MeetingStatus::Scheduled,
            agenda: vec![
                "Approbation du budget".to_string(),
                "Travaux de façade".to_string(),
            ],
            attendees_count: Some(2),
            quorum_validated: false,
            quorum_percentage: None,
            total_quotas: None,
            present_quotas: None,
            is_second_convocation: false,
            minutes_document_id: None,
            minutes_sent_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let attendees = vec![
            AttendeeInfo {
                owner_id: Uuid::new_v4(),
                name: "Jean Dupont".to_string(),
                email: "jean@example.com".to_string(),
                voting_power: dec!(150),
                is_proxy: false,
                proxy_for: None,
            },
            AttendeeInfo {
                owner_id: Uuid::new_v4(),
                name: "Marie Martin".to_string(),
                email: "marie@example.com".to_string(),
                voting_power: dec!(120),
                is_proxy: true,
                proxy_for: Some("Pierre Durant".to_string()),
            },
        ];

        let resolution = Resolution {
            id: Uuid::new_v4(),
            meeting_id: meeting.id,
            title: "Approbation du budget 2025".to_string(),
            description: "Le budget prévisionnel pour l'exercice 2025 est approuvé.".to_string(),
            resolution_type: ResolutionType::Ordinary,
            majority_required: MajorityType::Absolute,
            vote_count_pour: 2,
            vote_count_contre: 0,
            vote_count_abstention: 0,
            total_voting_power_pour: dec!(270),
            total_voting_power_contre: dec!(0),
            total_voting_power_abstention: dec!(0),
            status: ResolutionStatus::Adopted,
            agenda_item_index: None,
            voted_at: Some(Utc::now()),
            created_at: Utc::now(),
        };

        let resolutions = vec![ResolutionWithVotes {
            resolution,
            votes: vec![],
        }];

        let result =
            MeetingMinutesExporter::export_to_pdf(&building, &meeting, &attendees, &resolutions);

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.len() > 100); // PDF should have reasonable size
    }
}
