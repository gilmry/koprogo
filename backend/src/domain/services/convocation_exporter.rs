use crate::domain::entities::{Building, Convocation, ConvocationType, Meeting};
use printpdf::*;
use std::io::BufWriter;

/// Convocation Exporter - Generates PDF for Convocations d'Assemblée Générale
///
/// Compliant with Belgian copropriété law requirements for meeting invitations:
/// - Ordinary AG: 15 days minimum notice
/// - Extraordinary AG: 8 days minimum notice
/// - Second convocation: 8 days after quorum not reached
pub struct ConvocationExporter;

impl ConvocationExporter {
    /// Export convocation to PDF bytes
    ///
    /// Generates a complete convocation including:
    /// - Building information
    /// - Meeting details (date, type, location, agenda)
    /// - Legal compliance notice (minimum notice period)
    /// - Attendance instructions
    /// - Proxy information
    /// - Syndic contact information
    pub fn export_to_pdf(
        building: &Building,
        meeting: &Meeting,
        convocation: &Convocation,
    ) -> Result<Vec<u8>, String> {
        // Create PDF document (A4: 210mm x 297mm)
        let (doc, page1, layer1) = PdfDocument::new(
            "Convocation Assemblée Générale",
            Mm(210.0),
            Mm(297.0),
            "Layer 1",
        );
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Load fonts
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| format!("Failed to load bold font: {}", e))?;
        let font_regular = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| format!("Failed to load regular font: {}", e))?;

        let mut y_position = 277.0; // Start from top (A4 = 297mm height, 20mm margin)

        // Helper to add text line
        let add_text = |layer: &PdfLayerReference,
                        text: &str,
                        font: &IndirectFontRef,
                        size: f64,
                        x: f64,
                        y: &mut f64,
                        _bold: bool| {
            layer.use_text(text, size as f32, Mm(x as f32), Mm(*y as f32), font);
            *y -= size * 0.5; // Line spacing (approx 1.5x font size)
        };

        // HEADER: Building name and type of meeting
        add_text(
            &current_layer,
            &building.name,
            &font_bold,
            16.0,
            20.0,
            &mut y_position,
            true,
        );

        // Building address
        let address_line = format!(
            "{}, {} {}",
            building.address, building.postal_code, building.city
        );
        add_text(
            &current_layer,
            &address_line,
            &font_regular,
            10.0,
            20.0,
            &mut y_position,
            false,
        );

        y_position -= 10.0; // Extra spacing

        // TITLE: Convocation type
        let meeting_type_label = match convocation.meeting_type {
            ConvocationType::Ordinary => {
                if convocation.language == "FR" {
                    "CONVOCATION À L'ASSEMBLÉE GÉNÉRALE ORDINAIRE"
                } else if convocation.language == "NL" {
                    "OPROEP TOT GEWONE ALGEMENE VERGADERING"
                } else if convocation.language == "DE" {
                    "EINLADUNG ZUR ORDENTLICHEN GENERALVERSAMMLUNG"
                } else {
                    "CONVOCATION TO ORDINARY GENERAL ASSEMBLY"
                }
            }
            ConvocationType::Extraordinary => {
                if convocation.language == "FR" {
                    "CONVOCATION À L'ASSEMBLÉE GÉNÉRALE EXTRAORDINAIRE"
                } else if convocation.language == "NL" {
                    "OPROEP TOT BUITENGEWONE ALGEMENE VERGADERING"
                } else if convocation.language == "DE" {
                    "EINLADUNG ZUR AUSSERORDENTLICHEN GENERALVERSAMMLUNG"
                } else {
                    "CONVOCATION TO EXTRAORDINARY GENERAL ASSEMBLY"
                }
            }
            ConvocationType::SecondConvocation => {
                if convocation.language == "FR" {
                    "CONVOCATION À LA SECONDE ASSEMBLÉE GÉNÉRALE"
                } else if convocation.language == "NL" {
                    "OPROEP TOT TWEEDE ALGEMENE VERGADERING"
                } else if convocation.language == "DE" {
                    "EINLADUNG ZUR ZWEITEN GENERALVERSAMMLUNG"
                } else {
                    "CONVOCATION TO SECOND GENERAL ASSEMBLY"
                }
            }
        };

        add_text(
            &current_layer,
            meeting_type_label,
            &font_bold,
            14.0,
            20.0,
            &mut y_position,
            true,
        );

        y_position -= 10.0;

        // LEGAL NOTICE
        let minimum_notice_days = convocation.meeting_type.minimum_notice_days();
        let legal_notice = if convocation.language == "FR" {
            format!(
                "Conformément à la loi belge sur la copropriété, cette convocation respecte le délai légal minimum de {} jours.",
                minimum_notice_days
            )
        } else if convocation.language == "NL" {
            format!(
                "In overeenstemming met de Belgische mede-eigenheidswet respecteert deze oproeping de wettelijke minimumtermijn van {} dagen.",
                minimum_notice_days
            )
        } else if convocation.language == "DE" {
            format!(
                "Gemäß dem belgischen Wohnungseigentumsgesetz entspricht diese Einberufung der gesetzlichen Mindestfrist von {} Tagen.",
                minimum_notice_days
            )
        } else {
            format!(
                "In accordance with Belgian copropriété law, this convocation respects the legal minimum notice period of {} days.",
                minimum_notice_days
            )
        };

        add_text(
            &current_layer,
            &legal_notice,
            &font_regular,
            9.0,
            20.0,
            &mut y_position,
            false,
        );

        y_position -= 10.0;

        // MEETING DETAILS
        let details_label = if convocation.language == "FR" {
            "DÉTAILS DE LA RÉUNION:"
        } else if convocation.language == "NL" {
            "VERGADERINGSDETAILS:"
        } else if convocation.language == "DE" {
            "VERSAMMLUNGSDETAILS:"
        } else {
            "MEETING DETAILS:"
        };

        add_text(
            &current_layer,
            details_label,
            &font_bold,
            12.0,
            20.0,
            &mut y_position,
            true,
        );

        // Title
        add_text(
            &current_layer,
            &format!("📋 {}", meeting.title),
            &font_regular,
            10.0,
            20.0,
            &mut y_position,
            false,
        );

        // Date and time
        let date_label = if convocation.language == "FR" {
            "📅 Date"
        } else {
            "📅 Datum" // NL, DE, EN all use "Datum"
        };
        add_text(
            &current_layer,
            &format!(
                "{}: {}",
                date_label,
                convocation.meeting_date.format("%d/%m/%Y à %H:%M")
            ),
            &font_regular,
            10.0,
            20.0,
            &mut y_position,
            false,
        );

        // Location
        let location_label = if convocation.language == "FR" {
            "📍 Lieu"
        } else if convocation.language == "NL" {
            "📍 Locatie"
        } else if convocation.language == "DE" {
            "📍 Ort"
        } else {
            "📍 Location"
        };
        add_text(
            &current_layer,
            &format!("{}: {}", location_label, meeting.location),
            &font_regular,
            10.0,
            20.0,
            &mut y_position,
            false,
        );

        y_position -= 10.0;

        // AGENDA
        let agenda_label = if convocation.language == "FR" {
            "ORDRE DU JOUR:"
        } else if convocation.language == "NL" {
            "AGENDA:"
        } else if convocation.language == "DE" {
            "TAGESORDNUNG:"
        } else {
            "AGENDA:"
        };

        add_text(
            &current_layer,
            agenda_label,
            &font_bold,
            12.0,
            20.0,
            &mut y_position,
            true,
        );

        for (index, item) in meeting.agenda.iter().enumerate() {
            add_text(
                &current_layer,
                &format!("{}. {}", index + 1, item),
                &font_regular,
                10.0,
                25.0,
                &mut y_position,
                false,
            );
        }

        y_position -= 10.0;

        // ATTENDANCE INSTRUCTIONS
        let attendance_label = if convocation.language == "FR" {
            "MODALITÉS DE PARTICIPATION:"
        } else if convocation.language == "NL" {
            "DEELNAMEVOORWAARDEN:"
        } else if convocation.language == "DE" {
            "TEILNAHMEBEDINGUNGEN:"
        } else {
            "ATTENDANCE INSTRUCTIONS:"
        };

        add_text(
            &current_layer,
            attendance_label,
            &font_bold,
            12.0,
            20.0,
            &mut y_position,
            true,
        );

        let attendance_text = if convocation.language == "FR" {
            "• Vous pouvez participer en personne à l'assemblée générale\n\
             • Si vous ne pouvez pas assister, vous pouvez donner procuration à un autre copropriétaire\n\
             • Merci de confirmer votre présence via le lien de confirmation dans l'email"
        } else if convocation.language == "NL" {
            "• U kunt persoonlijk deelnemen aan de algemene vergadering\n\
             • Als u niet kunt deelnemen, kunt u een volmacht geven aan een andere mede-eigenaar\n\
             • Gelieve uw aanwezigheid te bevestigen via de bevestigingslink in de e-mail"
        } else if convocation.language == "DE" {
            "• Sie können persönlich an der Generalversammlung teilnehmen\n\
             • Wenn Sie nicht teilnehmen können, können Sie einem anderen Miteigentümer eine Vollmacht erteilen\n\
             • Bitte bestätigen Sie Ihre Anwesenheit über den Bestätigungslink in der E-Mail"
        } else {
            "• You can participate in person at the general assembly\n\
             • If you cannot attend, you can give proxy to another co-owner\n\
             • Please confirm your attendance via the confirmation link in the email"
        };

        for line in attendance_text.lines() {
            add_text(
                &current_layer,
                line,
                &font_regular,
                9.0,
                20.0,
                &mut y_position,
                false,
            );
        }

        y_position -= 10.0;

        // SYNDIC CONTACT INFORMATION
        if let Some(syndic_name) = &building.syndic_name {
            let contact_label = if convocation.language == "FR" {
                "CONTACT DU SYNDIC:"
            } else if convocation.language == "NL" {
                "CONTACT SYNDICUS:"
            } else if convocation.language == "DE" {
                "KONTAKT VERWALTER:"
            } else {
                "SYNDIC CONTACT:"
            };

            add_text(
                &current_layer,
                contact_label,
                &font_bold,
                12.0,
                20.0,
                &mut y_position,
                true,
            );

            add_text(
                &current_layer,
                syndic_name,
                &font_regular,
                10.0,
                20.0,
                &mut y_position,
                false,
            );

            if let Some(email) = &building.syndic_email {
                add_text(
                    &current_layer,
                    &format!("📧 {}", email),
                    &font_regular,
                    10.0,
                    20.0,
                    &mut y_position,
                    false,
                );
            }

            if let Some(phone) = &building.syndic_phone {
                add_text(
                    &current_layer,
                    &format!("📞 {}", phone),
                    &font_regular,
                    10.0,
                    20.0,
                    &mut y_position,
                    false,
                );
            }

            if let Some(office_hours) = &building.syndic_office_hours {
                let hours_label = if convocation.language == "FR" {
                    "Heures d'ouverture"
                } else if convocation.language == "NL" {
                    "Openingsuren"
                } else if convocation.language == "DE" {
                    "Öffnungszeiten"
                } else {
                    "Office hours"
                };
                add_text(
                    &current_layer,
                    &format!("🕒 {}: {}", hours_label, office_hours),
                    &font_regular,
                    10.0,
                    20.0,
                    &mut y_position,
                    false,
                );
            }
        }

        y_position -= 15.0;

        // FOOTER
        let footer_text = if convocation.language == "FR" {
            "Cette convocation a été générée automatiquement par KoproGo."
        } else if convocation.language == "NL" {
            "Deze oproeping werd automatisch gegenereerd door KoproGo."
        } else if convocation.language == "DE" {
            "Diese Einladung wurde automatisch von KoproGo generiert."
        } else {
            "This convocation was automatically generated by KoproGo."
        };

        add_text(
            &current_layer,
            footer_text,
            &font_regular,
            8.0,
            20.0,
            &mut y_position,
            false,
        );

        // Save PDF to bytes
        let mut buffer = BufWriter::new(Vec::new());
        doc.save(&mut buffer)
            .map_err(|e| format!("Failed to save PDF: {}", e))?;

        buffer.into_inner().map_err(|e| e.to_string())
    }

    /// Save PDF bytes to file
    ///
    /// # Arguments
    /// * `pdf_bytes` - PDF content as bytes
    /// * `file_path` - Destination file path
    ///
    /// # Returns
    /// Result with file path or error
    pub fn save_to_file(pdf_bytes: &[u8], file_path: &str) -> Result<String, String> {
        use std::fs;
        use std::path::Path;

        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(file_path).parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Write PDF bytes to file
        fs::write(file_path, pdf_bytes).map_err(|e| format!("Failed to write PDF file: {}", e))?;

        Ok(file_path.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_building() -> Building {
        Building {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "Résidence Les Lilas".to_string(),
            address: "Avenue Louise 123".to_string(),
            city: "Bruxelles".to_string(),
            postal_code: "1050".to_string(),
            country: "Belgium".to_string(),
            total_units: 20,
            total_tantiemes: 1000,
            construction_year: Some(1995),
            slug: Some("residence-les-lilas-bruxelles".to_string()),
            syndic_name: Some("Syndic Pro SPRL".to_string()),
            syndic_email: Some("contact@syndicpro.be".to_string()),
            syndic_phone: Some("+32 2 123 45 67".to_string()),
            syndic_address: Some("Rue du Commerce 45, 1000 Bruxelles".to_string()),
            syndic_office_hours: Some("Lun-Ven 9h-17h".to_string()),
            syndic_emergency_contact: Some("+32 475 12 34 56".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_meeting() -> Meeting {
        let mut meeting = Meeting::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            crate::domain::entities::MeetingType::Ordinary,
            "Assemblée Générale Ordinaire 2025".to_string(),
            Some("Discussion du budget annuel et travaux de rénovation".to_string()),
            Utc::now() + chrono::Duration::days(20),
            "Salle de réunion, Rez-de-chaussée".to_string(),
        )
        .unwrap();

        meeting
            .add_agenda_item("Approbation du procès-verbal de la dernière AG".to_string())
            .unwrap();
        meeting
            .add_agenda_item("Présentation et vote du budget annuel 2025".to_string())
            .unwrap();
        meeting
            .add_agenda_item("Travaux de rénovation de la toiture - Devis".to_string())
            .unwrap();
        meeting
            .add_agenda_item("Questions diverses".to_string())
            .unwrap();

        meeting
    }

    fn create_test_convocation(building_id: Uuid, meeting_id: Uuid) -> Convocation {
        Convocation::new(
            Uuid::new_v4(),
            building_id,
            meeting_id,
            ConvocationType::Ordinary,
            Utc::now() + chrono::Duration::days(20),
            "FR".to_string(),
            Uuid::new_v4(),
        )
        .unwrap()
    }

    #[test]
    fn test_convocation_pdf_generation() {
        let building = create_test_building();
        let meeting = create_test_meeting();
        let convocation = create_test_convocation(building.id, meeting.id);

        let pdf_bytes = ConvocationExporter::export_to_pdf(&building, &meeting, &convocation);

        assert!(pdf_bytes.is_ok());
        let bytes = pdf_bytes.unwrap();
        assert!(bytes.len() > 1000); // PDF should be at least 1KB
        assert!(bytes.starts_with(b"%PDF")); // Valid PDF header
    }

    #[test]
    fn test_convocation_pdf_all_languages() {
        let building = create_test_building();
        let meeting = create_test_meeting();

        for lang in &["FR", "NL", "DE", "EN"] {
            let convocation = Convocation::new(
                Uuid::new_v4(),
                building.id,
                meeting.id,
                ConvocationType::Ordinary,
                Utc::now() + chrono::Duration::days(20),
                lang.to_string(),
                Uuid::new_v4(),
            )
            .unwrap();

            let pdf_bytes = ConvocationExporter::export_to_pdf(&building, &meeting, &convocation);

            assert!(pdf_bytes.is_ok(), "Failed for language: {}", lang);
            let bytes = pdf_bytes.unwrap();
            assert!(bytes.len() > 1000, "PDF too small for language: {}", lang);
            assert!(
                bytes.starts_with(b"%PDF"),
                "Invalid PDF for language: {}",
                lang
            );
        }
    }

    #[test]
    fn test_extraordinary_meeting_convocation() {
        let building = create_test_building();
        let meeting = create_test_meeting();
        let convocation = Convocation::new(
            Uuid::new_v4(),
            building.id,
            meeting.id,
            ConvocationType::Extraordinary,
            Utc::now() + chrono::Duration::days(30),
            "FR".to_string(),
            Uuid::new_v4(),
        )
        .unwrap();

        let pdf_bytes = ConvocationExporter::export_to_pdf(&building, &meeting, &convocation);

        assert!(pdf_bytes.is_ok());
        let bytes = pdf_bytes.unwrap();
        assert!(bytes.len() > 1000);
    }
}
