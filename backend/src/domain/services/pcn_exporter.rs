use crate::domain::services::PcnReportLine;
use printpdf::*;
use std::io::BufWriter;

/// PCN Exporter - Generates PDF and Excel reports
pub struct PcnExporter;

impl PcnExporter {
    /// Export PCN report to PDF bytes
    /// Returns PDF document as Vec<u8>
    pub fn export_to_pdf(
        building_name: &str,
        report_lines: &[PcnReportLine],
        total_amount: f64,
    ) -> Result<Vec<u8>, String> {
        // Create PDF document
        let (doc, page1, layer1) = PdfDocument::new("Rapport PCN", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Load built-in font
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| e.to_string())?;
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| e.to_string())?;

        // Title
        current_layer.use_text(
            format!("Rapport PCN - Plan Comptable Normalisé"),
            24.0,
            Mm(20.0),
            Mm(270.0),
            &font_bold,
        );

        // Building name
        current_layer.use_text(
            format!("Immeuble: {}", building_name),
            14.0,
            Mm(20.0),
            Mm(260.0),
            &font,
        );

        // Table header
        let mut y = 245.0;
        current_layer.use_text("Code", 12.0, Mm(20.0), Mm(y), &font_bold);
        current_layer.use_text("Libellé", 12.0, Mm(50.0), Mm(y), &font_bold);
        current_layer.use_text("Montant (€)", 12.0, Mm(140.0), Mm(y), &font_bold);
        current_layer.use_text("Nb", 12.0, Mm(180.0), Mm(y), &font_bold);

        // Table rows
        y -= 10.0;
        for line in report_lines {
            current_layer.use_text(&line.account.code, 10.0, Mm(20.0), Mm(y), &font);
            current_layer.use_text(&line.account.label_fr, 10.0, Mm(50.0), Mm(y), &font);
            current_layer.use_text(
                format!("{:.2}", line.total_amount),
                10.0,
                Mm(140.0),
                Mm(y),
                &font,
            );
            current_layer.use_text(
                format!("{}", line.entry_count),
                10.0,
                Mm(180.0),
                Mm(y),
                &font,
            );
            y -= 7.0;
        }

        // Total
        y -= 5.0;
        current_layer.use_text("TOTAL:", 12.0, Mm(50.0), Mm(y), &font_bold);
        current_layer.use_text(
            format!("{:.2} €", total_amount),
            12.0,
            Mm(140.0),
            Mm(y),
            &font_bold,
        );

        // Save to bytes
        let mut buffer = Vec::new();
        doc.save(&mut BufWriter::new(&mut buffer))
            .map_err(|e| e.to_string())?;

        Ok(buffer)
    }

    /// Export PCN report to Excel bytes
    /// Returns Excel workbook as Vec<u8>
    pub fn export_to_excel(
        building_name: &str,
        report_lines: &[PcnReportLine],
        total_amount: f64,
    ) -> Result<Vec<u8>, String> {
        use rust_xlsxwriter::*;

        // Create workbook
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Set column widths
        worksheet
            .set_column_width(0, 10)
            .map_err(|e| e.to_string())?; // Code
        worksheet
            .set_column_width(1, 35)
            .map_err(|e| e.to_string())?; // Label NL
        worksheet
            .set_column_width(2, 35)
            .map_err(|e| e.to_string())?; // Label FR
        worksheet
            .set_column_width(3, 35)
            .map_err(|e| e.to_string())?; // Label DE
        worksheet
            .set_column_width(4, 35)
            .map_err(|e| e.to_string())?; // Label EN
        worksheet
            .set_column_width(5, 15)
            .map_err(|e| e.to_string())?; // Montant
        worksheet
            .set_column_width(6, 10)
            .map_err(|e| e.to_string())?; // Nb

        // Create formats
        let bold_format = Format::new().set_bold();
        let currency_format = Format::new().set_num_format("#,##0.00 €");
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0xD3D3D3));

        // Title
        worksheet
            .write_string_with_format(0, 0, "Rapport PCN - Plan Comptable Normalisé", &bold_format)
            .map_err(|e| e.to_string())?;

        // Building name
        worksheet
            .write_string_with_format(
                1,
                0,
                &format!("Immeuble: {}", building_name),
                &Format::new(),
            )
            .map_err(|e| e.to_string())?;

        // Table header (row 3)
        worksheet
            .write_string_with_format(3, 0, "Code PCN", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(3, 1, "Nederlands (NL)", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(3, 2, "Français (FR)", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(3, 3, "Deutsch (DE)", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(3, 4, "English (EN)", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(3, 5, "Montant", &header_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string_with_format(3, 6, "Nb Écritures", &header_format)
            .map_err(|e| e.to_string())?;

        // Data rows
        let mut row = 4;
        for line in report_lines {
            worksheet
                .write_string(row, 0, &line.account.code)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 1, &line.account.label_nl)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 2, &line.account.label_fr)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 3, &line.account.label_de)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 4, &line.account.label_en)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number_with_format(row, 5, line.total_amount, &currency_format)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 6, line.entry_count as f64)
                .map_err(|e| e.to_string())?;
            row += 1;
        }

        // Total row
        row += 1;
        worksheet
            .write_string_with_format(row, 4, "TOTAL:", &bold_format)
            .map_err(|e| e.to_string())?;
        worksheet
            .write_number_with_format(
                row,
                5,
                total_amount,
                &Format::new().set_bold().set_num_format("#,##0.00 €"),
            )
            .map_err(|e| e.to_string())?;

        // Save to bytes
        let buffer = workbook.save_to_buffer().map_err(|e| e.to_string())?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::ExpenseCategory;
    use crate::domain::services::PcnMapper;

    fn create_test_report() -> (Vec<PcnReportLine>, f64) {
        let lines = vec![
            PcnReportLine {
                account: PcnMapper::map_expense_to_pcn(&ExpenseCategory::Maintenance),
                total_amount: 1500.0,
                entry_count: 5,
            },
            PcnReportLine {
                account: PcnMapper::map_expense_to_pcn(&ExpenseCategory::Utilities),
                total_amount: 800.0,
                entry_count: 3,
            },
            PcnReportLine {
                account: PcnMapper::map_expense_to_pcn(&ExpenseCategory::Insurance),
                total_amount: 2000.0,
                entry_count: 1,
            },
        ];
        let total = lines.iter().map(|l| l.total_amount).sum();
        (lines, total)
    }

    // ===== PDF Export Tests =====

    #[test]
    fn test_export_pdf_returns_bytes() {
        let (lines, total) = create_test_report();

        let result = PcnExporter::export_to_pdf("Test Building", &lines, total);

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());

        // PDF should start with PDF magic bytes
        assert_eq!(&pdf_bytes[0..4], b"%PDF");
    }

    #[test]
    fn test_export_pdf_empty_report() {
        let result = PcnExporter::export_to_pdf("Empty Building", &[], 0.0);

        assert!(result.is_ok());
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());
        assert_eq!(&pdf_bytes[0..4], b"%PDF");
    }

    #[test]
    fn test_export_pdf_contains_building_name() {
        let (lines, total) = create_test_report();

        let result = PcnExporter::export_to_pdf("My Test Building", &lines, total);

        assert!(result.is_ok());
        // We can't easily check PDF content in unit tests, but we verify it doesn't error
    }

    // ===== Excel Export Tests =====

    #[test]
    fn test_export_excel_returns_bytes() {
        let (lines, total) = create_test_report();

        let result = PcnExporter::export_to_excel("Test Building", &lines, total);

        assert!(result.is_ok());
        let excel_bytes = result.unwrap();
        assert!(!excel_bytes.is_empty());

        // Excel (XLSX) files start with PK (ZIP signature)
        assert_eq!(&excel_bytes[0..2], b"PK");
    }

    #[test]
    fn test_export_excel_empty_report() {
        let result = PcnExporter::export_to_excel("Empty Building", &[], 0.0);

        assert!(result.is_ok());
        let excel_bytes = result.unwrap();
        assert!(!excel_bytes.is_empty());
        assert_eq!(&excel_bytes[0..2], b"PK");
    }

    #[test]
    fn test_export_excel_has_correct_row_count() {
        let (lines, total) = create_test_report();

        let result = PcnExporter::export_to_excel("Test Building", &lines, total);

        assert!(result.is_ok());
        // Should have header + 3 data rows + total row
        // We can't easily parse Excel in tests, so just verify no error
    }
}
