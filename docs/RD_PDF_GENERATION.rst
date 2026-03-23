==========================================================
R&D: Architecture PDF Generation for Legal Documents
==========================================================

Issue: #222
Status: Design Approved
Phase: Jalon 2 (Legal Compliance)
Date: 2026-03-23

.. contents::
   :depth: 3

Overview
========

KoproGo generates PDF documents for Belgian legal compliance:

* **Convocations AG** (15-day minimum notice per Art. 3.87 §3 Code Civil Belge)
* **Procès-verbaux AG** (minutes to be distributed within 30 days)
* **États datés** (property sale documents per Art. 29 Décret Copropriété)
* **Appels de fonds** (calls for funds with payment details)
* **GDPR data exports** (Art. 15 right to access - structured data + attachments)
* **Contractor quotes** (with signature pages per Belgian law)
* **Technical inspection reports** (with certificate of compliance)

Technology Decision
===================

After evaluation of PDF generation libraries:

+-------------------+-----------+------------+---------+----------+-----------+
| Library           | Language  | Template   | Quality | License  | Complexity|
+===================+===========+============+=========+==========+===========+
| **typst** (Recommended) | Rust | .typst   | ★★★★★   | Apache 2 | Low       |
+-------------------+-----------+------------+---------+----------+-----------+
| WeasyPrint        | Python    | HTML/CSS   | ★★★★☆   | BSD      | Medium    |
+-------------------+-----------+------------+---------+----------+-----------+
| wkhtmltopdf       | C++       | HTML/CSS   | ★★★☆☆   | LGPL     | Medium    |
+-------------------+-----------+------------+---------+----------+-----------+
| LaTeX/XeLaTeX     | TeX       | .tex       | ★★★★★   | GPL      | High      |
+-------------------+-----------+------------+---------+----------+-----------+
| Puppeteer/headless| Node.js   | HTML/CSS   | ★★★★☆   | MIT      | High      |
+-------------------+-----------+------------+---------+----------+-----------+

**Decision Rationale**:

* **Typst** selected for:
  * Pure Rust (no external process spawning)
  * Compile-time template validation
  * Fast generation (<500ms per document)
  * Modern syntax (Markdown-like)
  * EU-hosted (no data transmitted)
  * Minimal attack surface

* **Not chosen**:
  * WeasyPrint: Python dependency, process overhead
  * wkhtmltopdf: C++ binary, security concerns (CVE history)
  * LaTeX: Complex syntax, slow compilation
  * Puppeteer: Node.js overhead, requires browser

Architecture
============

.. code-block::

   Request Handler
        ↓
   PDF Use Case
        ↓ (orchestrates)
   Template Selector (language, document type)
        ↓
   Context Renderer (data anonymization, formatting)
        ↓
   Typst Compilation
        ↓
   PDF Binary (bytes)
        ↓
   Storage Service (S3 or local filesystem)
        ↓
   Response (download link or email)

Data Flow for Convocation Example
==================================

.. code-block:: rust

   // 1. Handler receives request
   POST /api/v1/meetings/{meeting_id}/generate-convocation
   Body: { "language": "fr", "include_agenda": true }

   // 2. Use case retrieves data
   let convocation = convocation_repo.find(meeting_id)?;
   let recipients = convocation_recipient_repo.find_by_meeting(meeting_id)?;
   let building = building_repo.find(convocation.building_id)?;

   // 3. Build context (ANONYMIZED where needed)
   let context = ConvocationPdfContext {
       building_name: building.name,              // ✓ OK
       meeting_date: convocation.meeting_date,   // ✓ OK
       agenda_items: convocation.agenda_items,   // ✓ OK
       recipients: recipients.iter().map(|r| RecipientInfo {
           // NO: owner.name, owner.email, owner.phone (NOT INCLUDED)
           unit_number: r.unit.number,            // ✓ OK
           // attendance tracked separately in post-meeting
       }).collect(),
       legal_notice: format!(
           "Convocation conforme à l'article 3.87 §3 du Code Civil belge"
       ),
   };

   // 4. Select template based on language
   let template = match language {
       "nl" => CONVOCATION_NL_TYPST,
       "de" => CONVOCATION_DE_TYPST,
       "en" => CONVOCATION_EN_TYPST,
       _    => CONVOCATION_FR_TYPST,
   };

   // 5. Render and compile
   let pdf_bytes = typst_pdf::compile(&template, &context)?;

   // 6. Store and return
   let storage_path = format!("s3://koprogo/{org_id}/convocations/{meeting_id}.pdf");
   storage_service.upload(storage_path, pdf_bytes).await?;

Typst Integration
=================

Cargo Dependencies:

.. code-block:: toml

   [dependencies]
   typst = "0.11"
   typst-pdf = "0.11"

Module Structure:

.. code-block:: text

   backend/src/infrastructure/pdf/
   ├── mod.rs                          (exports, trait definitions)
   ├── typst_engine.rs                 (Typst compilation wrapper)
   ├── context.rs                      (PDF context builders)
   ├── templates/                      (*.typst template files)
   │   ├── convocation_fr.typst        (Convocation French)
   │   ├── convocation_nl.typst        (Convocation Dutch)
   │   ├── convocation_de.typst        (Convocation German)
   │   ├── pv_ag_fr.typst              (PV French)
   │   ├── etat_date_fr.typst          (État daté French)
   │   ├── appel_fonds_fr.typst        (Appel de fonds French)
   │   └── gdpr_export_fr.typst        (GDPR export French)
   └── tests/                          (unit tests)

Sample Convocation Template (convocation_fr.typst):

.. code-block:: typst

   #set page(
       paper: "a4",
       margin: (left: 2cm, right: 2cm, top: 2cm, bottom: 2cm),
   )

   #set text(lang: "fr", region: "BE")

   = Convocation à Assemblée Générale

   Bâtiment: #{ building_name }

   Date: #{ meeting_date.format("%d/%m/%Y") }

   == Ordre du jour

   #for item in agenda_items [
     - #{ item }
   ]

   == Avis légal

   Conformément à l'article 3.87 §3 du Code Civil belge, cette assemblée
   générale est dûment convoquée. La date minimum de notification de 15 jours
   est respectée.

   #set text(size: 9pt, style: "italic")
   Généré par KoproGo - #{ now().format("%d/%m/%Y à %H:%M:%S") }

Use Case Pattern
================

.. code-block:: rust

   pub struct PdfGenerationUseCase {
       convocation_repo: Arc<dyn ConvocationRepository>,
       building_repo: Arc<dyn BuildingRepository>,
       storage_service: Arc<dyn StorageService>,
       pdf_engine: Arc<dyn PdfEngine>,
       audit_repo: Arc<dyn AuditRepository>,
   }

   impl PdfGenerationUseCase {
       pub async fn generate_convocation_pdf(
           &self,
           meeting_id: Uuid,
           language: &str,
           requester_id: Uuid,
       ) -> Result<GeneratedPdfResponse, String> {
           // 1. Authorization
           let requester = self.user_repo.find(requester_id)?;
           if !requester.can_generate_documents() {
               return Err("Unauthorized".to_string());
           }

           // 2. Fetch data
           let convocation = self.convocation_repo.find_by_meeting(meeting_id)?;
           let recipients = self.convocation_repo.find_recipients(meeting_id)?;
           let building = self.building_repo.find(convocation.building_id)?;

           // 3. Build context
           let context = ConvocationPdfContext {
               building_name: building.name.clone(),
               meeting_date: convocation.meeting_date,
               agenda_items: convocation.agenda_items.clone(),
               // ... other fields
           };

           // 4. Generate PDF
           let pdf_bytes = self.pdf_engine
               .compile_typst(language, "convocation", &context)
               .await?;

           // 5. Store in S3
           let storage_key = format!(
               "{}/{}/convocations/{}.pdf",
               building.organization_id, building.id, meeting_id
           );
           let url = self.storage_service.upload(&storage_key, pdf_bytes).await?;

           // 6. Audit log
           self.audit_repo.log(AuditEvent {
               action: "convocation_generated",
               resource: "convocation",
               resource_id: meeting_id,
               user_id: requester_id,
               timestamp: Utc::now(),
               metadata: serde_json::json!({
                   "language": language,
                   "file_size_bytes": pdf_bytes.len(),
               }),
           }).await?;

           Ok(GeneratedPdfResponse {
               meeting_id,
               pdf_url: url,
               language: language.to_string(),
               generated_at: Utc::now(),
           })
       }
   }

Multi-language Support
======================

Template selection by language:

.. code-block:: rust

   fn select_template(document_type: &str, language: &str) -> &'static str {
       match (document_type, language) {
           ("convocation", "nl") => CONVOCATION_NL_TYPST,
           ("convocation", "de") => CONVOCATION_DE_TYPST,
           ("convocation", "en") => CONVOCATION_EN_TYPST,
           ("convocation", _)     => CONVOCATION_FR_TYPST,
           ("pv", _)              => PV_FR_TYPST,
           ("etat_date", _)       => ETAT_DATE_FR_TYPST,
           ("appel_fonds", _)     => APPEL_FONDS_FR_TYPST,
           _ => return Err(format!("Unknown document type: {}", document_type)),
       }
   }

Language data insertion in templates:

.. code-block:: typst

   // convocation_fr.typst
   #let labels = (
       title: "Convocation à Assemblée Générale",
       agenda: "Ordre du jour",
       date: "Date",
       location: "Lieu",
       signature: "Signature du syndic",
   )

   = #{ labels.title }
   == #{ labels.agenda }

Storage
=======

Generated PDFs stored in S3 (or local filesystem in dev):

.. code-block:: text

   Development (local filesystem):
     ./storage/koprogo_docs/
       {organization_id}/
         {building_id}/
           convocations/
             {meeting_id}.pdf
             {meeting_id}_metadata.json
           pv/
             {meeting_id}.pdf
           etats-dates/
             {etat_date_id}.pdf

   Production (S3):
     s3://koprogo-documents/
       {organization_id}/
         {building_id}/
           convocations/{meeting_id}.pdf
           pv/{meeting_id}.pdf
           etats-dates/{etat_date_id}.pdf

Document Tracking
=================

Database table for document metadata:

.. code-block:: sql

   CREATE TABLE IF NOT EXISTS generated_documents (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       meeting_id UUID REFERENCES meetings(id),
       document_type VARCHAR(50) NOT NULL, -- convocation, pv, etat_date, appel_fonds
       language VARCHAR(10) NOT NULL DEFAULT 'fr',
       storage_path VARCHAR(1000) NOT NULL,
       file_size_bytes BIGINT,
       generated_by UUID NOT NULL REFERENCES users(id),
       generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       downloaded_count INTEGER DEFAULT 0,
       last_downloaded_at TIMESTAMPTZ
   );

GDPR Compliance
===============

Personal Data Protection:

* **Names**: NOT included in PDFs (use unit numbers only)
* **Emails**: Only in metadata, never in document body
* **Phone numbers**: Removed
* **IBAN/Bank details**: Completely excluded
* **Personal financial data**: Anonymized (aggregate only)

Audit Trail:

* Every PDF generation logged to `generated_documents` table
* IP address of requester tracked in audit_log
* User agent recorded for security analysis

Retention Policy:

* PDFs stored for 7 years (Belgian legal requirement for accounting)
* Older documents auto-archived to cold storage (Glacier)
* GDPR erasure removes associated documents within 30 days

Implementation Roadmap
======================

**Phase 1 - Foundation (Week 1)**:
  1. Add typst dependency to Cargo.toml (0.5h)
  2. Create PDF module structure (0.5h)
  3. Implement typst_engine wrapper (2h)
  4. Create ConvocationPdfContext (1h)
  5. Write convocation_fr.typst template (3h)
  6. Unit tests for context builders (2h)

  **Subtotal**: ~9 hours

**Phase 2 - Integration (Week 2)**:
  7. Create PdfGenerationUseCase (2h)
  8. Implement REST handler for generate-convocation (1.5h)
  9. Add storage_service integration (2h)
  10. E2E tests for full pipeline (3h)
  11. Audit logging (1h)

  **Subtotal**: ~9.5 hours

**Phase 3 - Additional Templates (Week 3)**:
  12. Add multi-language support (nl, de, en) (4h)
  13. Procès-verbal template + use case (4h)
  14. État daté template + use case (4h)
  15. Appel de fonds template + use case (3h)

  **Subtotal**: ~15 hours

**Phase 4 - Advanced Features (Week 4)**:
  16. Digital signatures (Trident or similar) (6h)
  17. Batch PDF generation (background job) (3h)
  18. Email delivery integration (2h)
  19. Performance optimization & benchmarking (2h)

  **Subtotal**: ~13 hours

**Total Estimate**: ~46.5 hours (5-6 weeks full-time, or 2-3 weeks with 2 devs)

Key Implementation Details
==========================

1. **Idempotency**: PDF generation should be deterministic (same input = same output)
   - Use fixed timestamps in templates
   - Ensure reproducible random elements

2. **Error Handling**:
   - Typst compilation errors should be caught and logged
   - Return user-friendly error messages
   - Include stack trace in audit log only

3. **Performance Optimization**:
   - Cache compiled templates in memory
   - Use async I/O for storage upload
   - Consider lazy-loading PDF modules

4. **Testing Strategy**:
   - Unit tests: Template rendering with mock data
   - Integration tests: Full pipeline with real database
   - E2E tests: Generate PDF, download, verify content
   - Visual regression tests: Compare PDF output against baseline

5. **Deployment Considerations**:
   - Typst compiler is ~10MB binary
   - No external process spawning (security advantage)
   - Memory usage: ~50-100MB per compilation
   - Scaling: Use worker pool for batch generation

Success Criteria
================

✓ All PDFs generated within 500ms (P99)
✓ PDF size < 2MB per document
✓ 99.9% compilation success rate
✓ Support 4+ languages (FR, NL, DE, EN)
✓ Zero personal data leakage in templates
✓ Full GDPR compliance (Art. 30 audit trail)
✓ Belgian legal format compliance verified
✓ No external dependencies (pure Rust)

References
==========

* `Typst Documentation <https://typst.app/docs/>`_
* `Article 3.87 Code Civil Belge <https://www.ejustice.just.fgov.be/cgi_loi/change_lg.pl?language=fr&la=F&cn=2013081302&table_name=loi&rech=1&number=1&section_name=CoCC&sectionsource=&caller=list&fromtab=loi_all&all_lang=1>`_
* `GDPR Article 30 - Records of Processing <https://gdpr-info.eu/art-30-gdpr/>`_
