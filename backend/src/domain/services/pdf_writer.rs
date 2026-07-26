use printpdf::{
    BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt, TextItem,
};

/// Adaptateur autour de l'API "Ops" de printpdf 0.11 (réécriture complète vs 0.7 —
/// plus de `Layer`/`PdfLayerReference`, tout passe par une liste d'`Op` empilés sur
/// `PdfPage`). Centralise le pattern "texte positionné en absolu" utilisé par les
/// exporteurs PDF `domain/services/*_exporter.rs` pour éviter une réimplémentation
/// par fichier. Migration liée à la fermeture de #658 (build cassé) + #636 (RUSTSEC
/// lopdf) — cf. `docs/agent-activity/2026-07-26-sync-audit-feature-dev.md`.
pub struct PdfPageBuilder {
    ops: Vec<Op>,
}

impl Default for PdfPageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfPageBuilder {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    /// Équivalent de l'ancien `layer.use_text(text, size, Mm(x), Mm(y), &font)` (API 0.7).
    pub fn text(
        &mut self,
        text: impl Into<String>,
        size_pt: f32,
        x_mm: f32,
        y_mm: f32,
        font: &PdfFontHandle,
    ) {
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetFont {
            font: font.clone(),
            size: Pt(size_pt),
        });
        self.ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(x_mm).into(),
                y: Mm(y_mm).into(),
            },
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(text.into())],
        });
        self.ops.push(Op::EndTextSection);
    }

    pub fn into_page(self, width_mm: f32, height_mm: f32) -> PdfPage {
        PdfPage::new(Mm(width_mm), Mm(height_mm), self.ops)
    }
}

/// Police intégrée (Helvetica, Times, ...) — pas de résolution via `resources`
/// nécessaire pour `PdfFontHandle::Builtin` (contrairement à une police externe).
pub fn builtin_font(font: BuiltinFont) -> PdfFontHandle {
    PdfFontHandle::Builtin(font)
}

pub fn new_document(title: &str) -> PdfDocument {
    PdfDocument::new(title)
}

pub fn save_document(mut doc: PdfDocument, page: PdfPage) -> Vec<u8> {
    doc.pages.push(page);
    let mut warnings = Vec::new();
    doc.save(&PdfSaveOptions::default(), &mut warnings)
}
