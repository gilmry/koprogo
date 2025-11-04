=======================================================================================
Issue #47: feat: Extend PDF generation (Meeting minutes, contracts, financial reports)
=======================================================================================

:State: **OPEN**
:Milestone: Phase 2: K3s + Automation
:Labels: phase:vps,track:software priority:high
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-01
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/47>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   **Current PDF generation:** ⚠️ **50% implemented**
   - ✅ PCN reports (Belgian chart of accounts) via `printpdf` library
   - ✅ Excel export for PCN reports via `rust_xlsxwriter`
   - ❌ Meeting minutes (procès-verbaux) with vote results
   - ❌ Financial statements for owners
   - ❌ Contracts (ownership, services)
   - ❌ Work quotes
   
   **Files:**
   - `backend/src/domain/services/pcn_exporter.rs` ✅
   - `backend/src/infrastructure/web/handlers/document_handlers.rs` ⚠️
   
   ## Objective
   
   Extend PDF generation to cover all document types required for Belgian copropriété management.
   
   ## Required PDF Templates
   
   ### 1. Meeting Minutes (Procès-Verbal d'Assemblée Générale)
   
   **Content:**
   - Building name & address
   - Meeting type (AGO/AGE)
   - Date, time, location
   - Attendees list with voting power (tantièmes)
   - Quorum validation
   - Agenda
   - Resolutions with vote results (depends on #46)
   - Signatures section
   
   **API endpoint:** `POST /api/v1/meetings/:id/export-minutes-pdf`
   
   **Implementation:**
   ```rust
   // backend/src/domain/services/meeting_minutes_exporter.rs
   
   pub struct MeetingMinutesExporter;
   
   impl MeetingMinutesExporter {
       pub fn generate_pdf(
           meeting: &Meeting,
           building: &Building,
           resolutions: Vec<Resolution>,
           attendees: Vec<(Owner, f64)>, // (owner, voting_power)
       ) -> Result<Vec<u8>, String> {
           let doc = PdfDocument::empty("Meeting Minutes");
           
           // Header
           self.add_header(&doc, building, meeting);
           
           // Attendees section
           self.add_attendees(&doc, attendees);
           
           // Quorum validation
           self.add_quorum(&doc, meeting);
           
           // Agenda
           self.add_agenda(&doc, meeting.agenda);
           
           // Resolutions & votes
           for resolution in resolutions {
               self.add_resolution(&doc, resolution);
           }
           
           // Signatures
           self.add_signatures(&doc);
           
           doc.save_to_bytes()
       }
   }
   ```
   
   ### 2. Owner Financial Statement (Relevé de Charges)
   
   **Content:**
   - Owner name & address
   - Period (Q1 2025, Year 2025, etc.)
   - Units owned with ownership percentages
   - Expense breakdown by category
   - Payment status (paid/pending)
   - Total due
   - Payment instructions (bank details)
   
   **API endpoint:** `POST /api/v1/owners/:id/export-statement-pdf`
   
   **Use case:**
   ```rust
   pub async fn generate_owner_statement(
       &self,
       owner_id: Uuid,
       start_date: DateTime<Utc>,
       end_date: DateTime<Utc>,
   ) -> Result<Vec<u8>, String> {
       // Fetch owner data
       let owner = self.owner_repo.find_by_id(owner_id).await?;
       
       // Fetch units owned
       let units = self.unit_owner_repo.find_units_by_owner(owner_id).await?;
       
       // Fetch expenses for period
       let expenses = self.expense_repo.find_by_owner_and_period(owner_id, start_date, end_date).await?;
       
       // Generate PDF
       OwnerStatementExporter::generate_pdf(owner, units, expenses)
   }
   ```
   
   ### 3. Ownership Contract (Contrat de Copropriété)
   
   **Content:**
   - Building information
   - Unit details (number, floor, area, tantièmes)
   - Owner information
   - Ownership start date
   - Percentage owned
   - Rights and obligations
   - General assembly rules
   - Expense allocation rules
   
   **API endpoint:** `POST /api/v1/unit-owners/:id/export-contract-pdf`
   
   **Template-based approach:**
   ```rust
   pub struct ContractExporter {
       template: String, // HTML template
   }
   
   impl ContractExporter {
       pub fn generate_pdf(
           building: &Building,
           unit: &Unit,
           owner: &Owner,
           unit_owner: &UnitOwner,
       ) -> Result<Vec<u8>, String> {
           // Render HTML template with data
           let html = self.render_template(building, unit, owner, unit_owner);
           
           // Convert HTML to PDF (using headless_chrome or wkhtmltopdf)
           self.html_to_pdf(html)
       }
   }
   ```
   
   ### 4. Work Quote Document (Devis de Travaux)
   
   **Content:**
   - Building information
   - Work description
   - Cost breakdown
   - Timeline
   - Approval status
   - Signatures section
   
   **API endpoint:** `POST /api/v1/expenses/:id/export-quote-pdf` (if expense is type "Works")
   
   ### 5. Financial Report (Rapport Financier Annuel)
   
   **Content:**
   - Building information
   - Year summary
   - Income breakdown (charges paid)
   - Expense breakdown by category
   - Budget vs actual
   - Reserve fund status
   - Charts (pie chart for expenses, bar chart for trends)
   
   **API endpoint:** `POST /api/v1/buildings/:id/export-annual-report-pdf`
   
   **Includes charts:**
   ```rust
   // Use plotters or similar for chart generation
   use plotters::prelude::*;
   
   fn generate_expense_chart(expenses: Vec<Expense>) -> Vec<u8> {
       // Generate PNG chart
       // Embed in PDF
   }
   ```
   
   ## Technical Approaches
   
   ### Option A: printpdf (Current)
   
   **Pros:**
   - ✅ Pure Rust, no external dependencies
   - ✅ Fast and lightweight
   - ✅ Already used for PCN reports
   
   **Cons:**
   - ❌ Manual layout (low-level API)
   - ❌ No templating
   - ❌ Complex for multi-page documents
   
   **Best for:** Simple tabular reports (PCN, statements)
   
   ### Option B: HTML → PDF (wkhtmltopdf / headless_chrome)
   
   **Pros:**
   - ✅ Templating with HTML/CSS (Tera, Handlebars)
   - ✅ Easy styling
   - ✅ Responsive layouts
   - ✅ Complex documents easier
   
   **Cons:**
   - ❌ External dependency (wkhtmltopdf binary or Chrome)
   - ❌ Larger Docker image
   - ❌ Slower than printpdf
   
   **Best for:** Complex documents (contracts, meeting minutes)
   
   **Implementation:**
   ```rust
   use headless_chrome::{Browser, LaunchOptions};
   
   pub fn html_to_pdf(html: &str) -> Result<Vec<u8>, String> {
       let browser = Browser::new(LaunchOptions::default())?;
       let tab = browser.wait_for_initial_tab()?;
       
       tab.navigate_to(&format!("data:text/html,{}", html))?;
       tab.wait_until_navigated()?;
       
       let pdf = tab.print_to_pdf(None)?;
       Ok(pdf)
   }
   ```
   
   ### Option C: Hybrid Approach (Recommended)
   
   - **printpdf** for simple reports (PCN, statements)
   - **HTML → PDF** for complex documents (meeting minutes, contracts)
   
   ## Templating System
   
   **Use Tera templates:**
   
   `backend/templates/meeting_minutes.html`:
   ```html
   <!DOCTYPE html>
   <html>
   <head>
     <style>
       body { font-family: Arial; font-size: 12pt; }
       h1 { text-align: center; color: #2c3e50; }
       table { width: 100%; border-collapse: collapse; }
       th, td { border: 1px solid #ddd; padding: 8px; }
     </style>
   </head>
   <body>
     <h1>Procès-Verbal d'Assemblée Générale</h1>
     
     <h2>Informations</h2>
     <p>Immeuble: {{ building.name }}</p>
     <p>Date: {{ meeting.date }}</p>
     <p>Type: {{ meeting.meeting_type }}</p>
     
     <h2>Présences</h2>
     <table>
       <tr><th>Copropriétaire</th><th>Millièmes</th></tr>
       {% for attendee in attendees %}
       <tr><td>{{ attendee.name }}</td><td>{{ attendee.voting_power }}</td></tr>
       {% endfor %}
     </table>
     
     <h2>Résolutions</h2>
     {% for resolution in resolutions %}
     <div class="resolution">
       <h3>{{ resolution.title }}</h3>
       <p>{{ resolution.description }}</p>
       <p><strong>Résultat:</strong> {{ resolution.status }}</p>
       <p>Pour: {{ resolution.vote_count_pour }} | Contre: {{ resolution.vote_count_contre }}</p>
     </div>
     {% endfor %}
   </body>
   </html>
   ```
   
   **Render with Tera:**
   ```rust
   use tera::{Tera, Context};
   
   let tera = Tera::new("templates/**/*.html")?;
   let mut context = Context::new();
   context.insert("building", &building);
   context.insert("meeting", &meeting);
   context.insert("resolutions", &resolutions);
   
   let html = tera.render("meeting_minutes.html", &context)?;
   let pdf = html_to_pdf(&html)?;
   ```
   
   ## Dependencies
   
   `backend/Cargo.toml`:
   ```toml
   [dependencies]
   # Option B (HTML → PDF)
   tera = "1.19"
   headless_chrome = "1.0"
   
   # Option A (Pure Rust, existing)
   printpdf = "0.7"
   
   # Charts (optional)
   plotters = "0.3"
   ```
   
   ## Implementation Priority
   
   **Sprint 1 (High Priority):**
   1. Meeting minutes PDF (#46 voting system dependency)
   2. Owner financial statements
   
   **Sprint 2 (Medium Priority):**
   3. Ownership contracts
   4. Annual financial reports
   
   **Sprint 3 (Low Priority):**
   5. Work quote documents
   
   ## Testing
   
   - [ ] Generate meeting minutes PDF with sample data
   - [ ] Generate owner statement PDF with expenses
   - [ ] Generate ownership contract PDF
   - [ ] PDF rendering correct (margins, fonts, pagination)
   - [ ] Multi-page documents handled correctly
   - [ ] Charts render correctly in PDFs
   - [ ] Performance acceptable (<2s for typical document)
   
   ## Acceptance Criteria
   
   - [ ] Meeting minutes PDF exporter complete
   - [ ] Owner financial statement PDF exporter complete
   - [ ] Ownership contract PDF exporter complete
   - [ ] Annual report PDF exporter complete (optional charts)
   - [ ] Work quote PDF exporter complete
   - [ ] HTML template system integrated (Tera)
   - [ ] API endpoints functional
   - [ ] Tests passing
   - [ ] Documentation updated
   
   ## Effort Estimate
   
   **Medium** (3-4 days)
   - Day 1: Meeting minutes PDF + Tera templates setup
   - Day 2: Owner financial statement PDF
   - Day 3: Ownership contract PDF
   - Day 4: Annual report + charts (optional)
   
   ## Related
   
   - Depends on: Issue #46 (voting system for meeting minutes)
   - Enhances: Current PDF generation (PCN reports)
   - Supports: Document management
   
   ## References
   
   - printpdf: https://docs.rs/printpdf/
   - Tera templates: https://tera.netlify.app/
   - headless_chrome: https://docs.rs/headless_chrome/
   - plotters: https://docs.rs/plotters/

.. raw:: html

   </div>

