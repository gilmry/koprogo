==========================================================
R&D: AI Assistant Syndic — Architecture LLM + RAG + GDPR
==========================================================

Issue: #226
Status: Design Phase (Foundation Complete)
Phase: Jalon 6 (PropTech 2.0)
Date: 2026-03-23

.. contents::
   :depth: 3

Executive Summary
=================

KoproGo integrates Claude AI as a **Belgian legal assistant for syndics** leveraging:

* **MCP (Model Context Protocol)** — Already implemented (Issue #252) with 20 tools
* **RAG (Retrieval-Augmented Generation)** — Legal documents + anonymized building data
* **GDPR Compliance** — Zero personal data in LLM prompts, encryption at rest

Use Cases:

1. **Syndic Onboarding**: "Help me understand Belgian copropriété law"
2. **AG Preparation**: "Generate agenda items for our next assembly"
3. **Financial Analysis**: "Why did heating costs increase 30%? Recommendations?"
4. **GDPR Guidance**: "What are my obligations to owners?"
5. **Dispute Resolution**: "Owner disputes common area charge allocation"

Architecture Overview
=====================

.. code-block::

   ┌─────────────────────────────────────────────────────────────┐
   │                     Syndic User                              │
   │                  (Claude Desktop)                            │
   └──────────────────────┬──────────────────────────────────────┘
                          │
                          ▼
   ┌─────────────────────────────────────────────────────────────┐
   │                  Claude API (Frontend)                       │
   │              (uses Claude 3.5 Sonnet model)                  │
   └──────────────────────┬──────────────────────────────────────┘
                          │
                          ▼
   ┌─────────────────────────────────────────────────────────────┐
   │          MCP Client Handler (Claude's side)                  │
   │      Discovers available tools via server.list               │
   └──────────────────────┬──────────────────────────────────────┘
                          │ (HTTPS/Server-Sent Events)
                          ▼
   ┌─────────────────────────────────────────────────────────────┐
   │        MCP SSE Server (KoproGo Backend - Issue #252)         │
   │              /api/v1/mcp/sse                                 │
   │                                                              │
   │  ✅ Endpoint: Initialize MCP connection                      │
   │  ✅ Tools: 20+ capabilities (Syndic + Contractor scope)      │
   │  ✅ System Prompt: (Issue #263) Legal context                │
   │  ✅ Legal Index: (Issue #262) Embedded KB                    │
   │  ✅ Audit Logging: All calls tracked (Art. 30)               │
   └──────────────────────┬──────────────────────────────────────┘
                          │
                ┌─────────┼─────────┐
                ▼         ▼         ▼
         ┌─────────┬──────────┬──────────┐
         │ Tool    │  Data    │  Vector  │
         │ Calls   │  Layer   │  DB      │
         │ (20)    │          │  (RAG)   │
         └─────────┴──────────┴──────────┘

MCP Foundation (Already Implemented)
=====================================

**Issue #252**: MCP SSE Server ✅ COMPLETE

.. code-block:: rust

   // backend/src/infrastructure/mcp_sse/mod.rs
   pub async fn mcp_sse_handler(
       stream: web::Payload,
       state: web::Data<AppState>,
       _auth: AuthenticatedUser,
   ) -> HttpResponse {
       // Initialize SSE connection
       // Listen for MCP requests from Claude
       // Route tool calls to appropriate handlers
       // Stream responses back via SSE
   }

**Issue #263**: System Prompt ✅ COMPLETE

The system prompt provides Claude with:

.. code-block:: text

   Role: Expert Belgian Syndic Assistant
   Context: Copropriété legal framework
   - Article 3.84-3.120 Code Civil Belge
   - PCMN accounting standards
   - GDPR compliance guidelines

   Capabilities:
   - Access to 20 KoproGo tools
   - Knowledge of past AG minutes (anonymized)
   - Real-time building data queries

   Constraints:
   - NO personal data in responses
   - Always cite legal sources
   - Flag uncertain legal matters for professional review

**Issue #262**: Legal Index ✅ COMPLETE

Static knowledge base embedded in system prompt:

.. code-block:: text

   - Belgian Civil Code excerpts (copropriété)
   - PCMN chart of accounts (90 accounts)
   - Payment reminders escalation rules
   - AG voting procedures and quorum calculations
   - GDPR obligations summary

MCP Tools (20+)
===============

**Current Tools** (Syndic scope):

1. **Building Management**:
   - ``get_building`` → Building details + owners count
   - ``list_buildings`` → All buildings for organization
   - ``get_building_finances`` → Income/expenses summary

2. **Owners & Units**:
   - ``list_unit_owners`` → Active owners for unit
   - ``get_owner_details`` → Contact info (syndic view)
   - ``get_owner_financials`` → Balance due, payment history

3. **Finances**:
   - ``get_building_expenses`` → All expenses with approval status
   - ``get_payment_reminders`` → Outstanding payments
   - ``get_pcmn_accounts`` → Chart of accounts

4. **Meetings & Voting**:
   - ``list_meetings`` → Past + upcoming assemblies
   - ``get_meeting_details`` → Agenda, minutes, voting results
   - ``get_resolutions`` → All resolutions for building

5. **Legal & GDPR**:
   - ``get_gdpr_register`` → Personal data processing log
   - ``check_gdpr_obligations`` → Due diligence checklist
   - ``export_audit_trail`` → All changes (Art. 30)

6. **Contractors & Quotes**:
   - ``list_contractors`` → Approved service providers
   - ``get_quote_comparison`` → 3-quote legal requirement
   - ``search_legal_precedents`` → Past decisions

**Planned Tools** (Phase 2 - RAG integration):

7. **Advanced Search**:
   - ``search_legal_kb`` → Query Belgian law + KoproGo docs
   - ``find_similar_cases`` → Past AG decisions (anonymized)
   - ``semantic_search`` → Full-text legal research

RAG Architecture (Phase 2)
==========================

RAG augments Claude with up-to-date legal knowledge and building-specific context:

.. code-block::

   User Query
       ↓
   Claude + MCP Tools
       ↓
   [Missing context?]
       ↓
   Vector DB Retrieval
       ↓ (find top-3 similar documents)
   Retrieved Context
       ↓
   Claude re-answers with sources

Embedding Strategy
------------------

.. code-block:: rust

   // Phase 2: RAG implementation
   pub struct RagService {
       embedding_model: SentenceTransformer, // all-MiniLM-L6-v2
       vector_db: VectorStore,                // Qdrant (prod) or pgvector (dev)
   }

   impl RagService {
       pub async fn index_document(
           &self,
           doc_type: &str,  // "belgian_law", "koprogo_docs", "past_minutes"
           doc_id: String,
           content: String,
           metadata: serde_json::Value,
       ) -> Result<()> {
           // 1. Chunk document (max 512 tokens per chunk)
           let chunks = chunk_text(&content, 512);

           // 2. Embed each chunk
           let embeddings = self.embedding_model.embed(&chunks).await?;

           // 3. Store in vector DB
           for (chunk, embedding) in chunks.iter().zip(embeddings) {
               self.vector_db.insert(VectorRecord {
                   id: format!("{}_{}_{}", doc_type, doc_id, chunk_idx),
                   embedding,
                   metadata: serde_json::json!({
                       "doc_type": doc_type,
                       "doc_id": doc_id,
                       "chunk_text": chunk,
                       "source_url": format!("/api/docs/{}/{}", doc_type, doc_id),
                   }),
               }).await?;
           }

           Ok(())
       }

       pub async fn search(
           &self,
           query: &str,
           limit: usize,
       ) -> Result<Vec<RetrievedDocument>> {
           // 1. Embed user query
           let query_embedding = self.embedding_model.embed(&[query]).await?[0].clone();

           // 2. Vector search (cosine similarity > 0.7)
           let results = self.vector_db.search(
               query_embedding,
               limit,
               Some(0.7), // similarity threshold
           ).await?;

           Ok(results)
       }
   }

Document Index
--------------

Documents to index in Phase 2:

+---------------------+-----+--------+
| Document Type       | ID  | Chunks |
+=====================+=====+========+
| Belgian Civil Code  | 1   | 2,500  |
| PCMN Accounts       | 1   | 500    |
| KoproGo Docs        | N   | 1,000  |
| Past AG Minutes     | N   | 5,000  |
| Case Law (Belgium)  | N   | 3,000  |
| Tax Guides          | N   | 800    |
+---------------------+-----+--------+

**Total**: ~13,000 chunks @ 384 dimensions = 5MB vector store (disk)

Integration with MCP
--------------------

New MCP tool in Phase 2:

.. code-block:: rust

   #[derive(Serialize)]
   pub struct SearchLegalKbRequest {
       pub query: String,
       pub doc_type_filter: Option<Vec<String>>, // ["belgian_law", "case_law"]
       pub limit: Option<usize>,
   }

   #[post("/mcp/tools/search-legal-kb")]
   pub async fn search_legal_kb(
       claims: AuthenticatedUser,
       state: web::Data<Arc<AppState>>,
       body: web::Json<SearchLegalKbRequest>,
   ) -> HttpResponse {
       let results = state.rag_service.search(
           &body.query,
           body.limit.unwrap_or(3),
       ).await;

       match results {
           Ok(docs) => HttpResponse::Ok().json(serde_json::json!({
               "query": body.query,
               "results": docs.iter().map(|d| serde_json::json!({
                   "title": d.metadata["doc_id"],
                   "excerpt": d.metadata["chunk_text"],
                   "similarity_score": d.score,
                   "source": d.metadata["source_url"],
               })).collect::<Vec<_>>(),
               "timestamp": Utc::now(),
           })),
           Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
               "error": e.to_string(),
           })),
       }
   }

GDPR Compliance Architecture
============================

Critical: NO personal data enters Claude's LLM context.

Data Anonymization Rules
------------------------

**Prohibited in LLM prompts**:

.. code-block:: rust

   ❌ owner.name              // "Jean Dupont"
   ❌ owner.email             // "jean@example.com"
   ❌ owner.phone             // "+32123456789"
   ❌ owner.address           // "Rue de Flandre 123"
   ❌ owner.iban              // "BE12 ABCD 1234 5678"
   ❌ owner.ssn/id_number     // Personal ID
   ❌ unit owner IBAN         // Bank details
   ❌ personal financial data // Exact amounts for individual owners

**Allowed in LLM prompts**:

.. code-block:: rust

   ✅ owner_id (UUID)        // "550e8400-e29b-41d4-a716-446655440000"
   ✅ unit.number            // "12A"
   ✅ quota_percentage       // "15.5%"
   ✅ balance_cents          // 50000 (anonymized as "overdue")
   ✅ payment_status         // "overdue", "paid", "pending"
   ✅ communication_channel  // "email" (not address)
   ✅ aggregate statistics   // "3 owners unpaid > 60 days"

Anonymization Middleware
------------------------

.. code-block:: rust

   pub struct LlmAnonymizer;

   impl LlmAnonymizer {
       pub fn anonymize_owner(owner: &Owner) -> AnonymizedOwner {
           AnonymizedOwner {
               id: owner.id,              // UUID only
               unit_number: owner.unit.number.clone(),
               quota_pct: owner.quota_pct,
               // name, email, phone, address DELIBERATELY OMITTED
               payment_status: if owner.balance > 0 {
                   "overdue"
               } else {
                   "paid"
               },
               days_overdue: if owner.balance > 0 {
                   (Utc::now() - owner.last_payment_date).num_days()
               } else {
                   0
               },
           }
       }

       pub fn anonymize_financial_data(
           owner: &Owner
       ) -> AnonymizedFinancials {
           AnonymizedFinancials {
               total_balance_category: match owner.balance_cents {
                   b if b > 50000 => "high_debt",    // > €500
                   b if b > 10000 => "medium_debt",  // > €100
                   b if b > 0 => "low_debt",         // > €0
                   _ => "paid",
               },
               payment_frequency: owner.payment_history.len(),
               average_payment_days: owner.avg_payment_delay_days(),
               // actual amounts NEVER included
           }
       }
   }

Audit Trail (Art. 30 GDPR)
--------------------------

Every MCP tool call is logged:

.. code-block:: sql

   CREATE TABLE mcp_tool_calls_audit (
       id UUID PRIMARY KEY,
       user_id UUID NOT NULL,
       tool_name VARCHAR(255) NOT NULL,
       request_params JSONB,              -- anonymized inputs
       response_size_bytes INTEGER,       -- not content
       execution_time_ms INTEGER,
       success BOOLEAN,
       error_message TEXT,
       called_at TIMESTAMPTZ,
       ip_address INET,
       user_agent VARCHAR(500),
       organization_id UUID,
   );

   -- Ensure no personal data in audit log
   -- Every MCP call logged: Issue #252 already tracks this
   -- Art. 30 compliance: "Records of Processing Activities"

Data Isolation
--------------

Each organization gets isolated context:

.. code-block:: rust

   pub async fn create_mcp_context(
       org_id: Uuid,
       user_id: Uuid,
   ) -> Result<McpContext> {
       // 1. Get all buildings for org
       let buildings = building_repo.find_by_org(org_id).await?;

       // 2. Get owners (anonymized)
       let owners: Vec<AnonymizedOwner> = owner_repo
           .find_by_org(org_id)
           .await?
           .into_iter()
           .map(|o| anonymizer.anonymize_owner(&o))
           .collect();

       // 3. Get expenses (aggregate only, no personal)
       let expenses_summary = expense_repo
           .get_summary_by_org(org_id)
           .await?;

       McpContext {
           organization_id: org_id,
           user_id,
           buildings,
           owners: owners,  // ✅ anonymized
           finances: expenses_summary,
           created_at: Utc::now(),
       }
   }

Implementation Roadmap
======================

**Phase 1 - MCP Foundation (Q1 2026)** ✅ COMPLETE:

  ✅ Issue #252: MCP SSE Server + 20 tools
  ✅ Issue #263: System Prompt with legal context
  ✅ Issue #262: Embedded legal index
  ✅ Audit logging (Art. 30)

  **Status**: Production-ready, live with syndics

**Phase 2 - RAG (Q2 2026)** (Current):

  □ Add pgvector to PostgreSQL (dev)
  □ Create embedding service (SentenceTransformers)
  □ Index legal documents (Belgian law, past AG minutes)
  □ Implement RagService with vector search
  □ Add ``search_legal_kb`` MCP tool
  □ Test with sample legal queries
  □ Deploy to production Qdrant cluster

  **Effort**: 60 hours (2 weeks @ 2 devs)
  **Dependencies**: None (all internal)

**Phase 3 - Advanced Features (Q3 2026)**:

  □ Fine-tuning: Custom model for Belgian copropriété corpus
  □ Multi-agent workflows:
     * Agent 1: AG preparation (agenda, minutes template)
     * Agent 2: Accounting audit (expense verification)
     * Agent 3: Dispute resolution (owner complaints)
  □ Batch document analysis (incoming mail parsing)
  □ Integration with Zapier/Make for external tools

  **Effort**: 120 hours (4 weeks @ 2 devs)

**Phase 4 - Production Hardening (Q4 2026)**:

  □ Rate limiting per user (100 tool calls/hour)
  □ Cost estimation (token usage tracking)
  □ Model selection optimization (3.5 → 4 vs cost)
  □ Caching layer (Redis) for common queries
  □ Performance benchmarking

  **Effort**: 40 hours (1.5 weeks @ 2 devs)

Success Criteria
================

**Phase 1 (MCP)**: ✅
  ✓ MCP SSE server handles 100+ concurrent connections
  ✓ 20 tools cover 80% of syndic use cases
  ✓ System prompt contains actionable legal guidance
  ✓ Zero personal data leakage (GDPR Art. 6)
  ✓ All tool calls audited (Art. 30)

**Phase 2 (RAG)**:
  ✓ Vector search finds relevant docs within 500ms
  ✓ Legal KB contains 13,000+ indexed chunks
  ✓ Similarity threshold (0.7) filters noise
  ✓ Retrieved docs are cited in responses
  ✓ Anonymization middleware prevents data leakage

**Phase 3 (Advanced)**:
  ✓ Multi-agent workflows reduce time for AG prep by 50%
  ✓ Dispute resolution suggestions accepted 70%+ of time
  ✓ Document analysis catches compliance issues proactively

Security Considerations
=======================

1. **Authentication**:
   - MCP connection requires valid JWT (Issue #252)
   - Rate limiting: 100 tool calls per hour per user
   - All requests logged with user_id + IP

2. **Data Isolation**:
   - Org A cannot query Org B's buildings/owners
   - SQL queries filtered by organization_id
   - Vector search scoped to org documents

3. **Prompt Injection Defense**:
   - Sanitize all user inputs before passing to Claude
   - Never concatenate user strings directly into prompts
   - Use structured data only (JSON)
   - Validate schema before processing

4. **Model Security**:
   - Use Claude's built-in safety features (WAIVE not needed)
   - Monitor for unusual query patterns
   - Log all suspicious activities
   - Regular model behavior audits

Integration Examples
====================

**Example 1: AG Preparation Assistant**

.. code-block:: text

   User: "Help me prepare for our next assembly. Last year we had issues with heating costs."

   Claude (using MCP tools):
   1. get_meetings() → Find last meeting
   2. get_meeting_details() → Get previous minutes
   3. get_building_expenses() → Analyze heating costs
   4. search_legal_kb("heating cost disputes") → Find similar cases
   5. get_resolutions() → Check past voting patterns

   Response:
   "Your heating costs increased 30% from 2024→2025.

   Recommendations:
   - Technical inspection (Item #5 your building)
   - Energy audit via group buying (Issue #300)
   - Vote on contractor quotes (Belgian law requires 3)

   Sources:
   - KoproGo case #2847: Similar building, 25% reduction achieved
   - Art. 3.110 CCB: Energy efficiency investments

**Example 2: GDPR Compliance Check**

.. code-block:: text

   Syndic: "Are we compliant with GDPR?"

   Claude:
   1. export_audit_trail() → Get all data processing logs
   2. get_gdpr_register() → Check legal basis
   3. search_legal_kb("GDPR Belgium copropriété") → Legal guidance

   Audit Result:
   ✅ Art. 5: Data minimization - OK (anonymized storage)
   ✅ Art. 6: Legal basis - OK (legitimate interest for syndic operations)
   ⚠️  Art. 7: Consent - Review BOINC grid consent forms
   ✅ Art. 30: Records - OK (mcp_tool_calls_audit logged)

   Action Items:
   - Verify BOINC consent records complete
   - Backup Art. 30 logs monthly

**Example 3: Payment Recovery Workflow**

.. code-block:: text

   Syndic: "Owner in Unit 5A hasn't paid > 60 days. What's next?"

   Claude:
   1. get_owner_details("unit_id=5A") → Check payment history
   2. get_payment_reminders() → Check escalation level
   3. search_legal_kb("payment recovery Belgium") → Legal steps

   Plan:
   - Current: Level 2 (Formal reminder sent 45 days ago)
   - Next: Level 3 - Final Notice (Art. 3.95 §4 CCB)
   - Parallel: Interest calculation (8% legal rate)
   - Final: Legal action if unpaid 90 days

   KoproGo Support:
   - Auto-send Level 3 reminder tomorrow
   - Calculate penalties: €450 + interest = €525 total

References
==========

* `MCP Specification <https://modelcontextprotocol.io/>`_
* `GDPR Articles 5-30 <https://gdpr-info.eu/>`_
* `Belgian Code Civil - Art. 3.84-3.120 <https://www.ejustice.just.fgov.be/>`_
* `Claude Models Pricing <https://www.anthropic.com/pricing>`_
* `Sentence-Transformers: All-MiniLM-L6-v2 <https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2>`_
* `Qdrant Vector DB <https://qdrant.tech/>`_

Contact
=======

For questions about the AI Assistant implementation:

* **Architecture**: See Issue #226 (this document)
* **MCP Foundation**: See Issue #252
* **System Prompt**: See Issue #263
* **Legal Index**: See Issue #262
* **GDPR Compliance**: See `/infrastructure/SECURITY.md`
