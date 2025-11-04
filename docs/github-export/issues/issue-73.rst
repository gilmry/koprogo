==============================================================================
Issue #73: Système Complet d'Encodage de Factures avec Workflow de Validation
==============================================================================

:State: **OPEN**
:Milestone: Phase 2: K3s + Automation
:Labels: enhancement,javascript phase:vps,track:software priority:critical,rust
:Assignees: gilmry
:Created: 2025-10-31
:Updated: 2025-11-01
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/73>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue - Système Complet d'Encodage de Factures
   
   **Priorité**: 🔴 CRITIQUE
   **Estimation**: 12-16 heures
   **Phase**: VPS (Q4 2025 - Q2 2026)
   **Track**: Software Development
   
   ---
   
   ## 📋 Description
   
   Implémenter un système complet d'encodage de factures avec workflow de validation, gestion de la TVA, répartition automatique des charges par quotes-parts, et attachement de documents. Ce système enrichit considérablement l'entité `Expense` existante pour la transformer en un véritable système de gestion de factures pour copropriétés.
   
   **Contexte métier** : Les factures en copropriété nécessitent un processus rigoureux d'encodage, validation, et répartition. Le système actuel est trop basique et ne permet pas de gérer le workflow de validation ni les calculs complexes (TVA, répartition).
   
   ---
   
   ## 🎯 Objectifs
   
   - [ ] Enrichir l'entité Expense avec gestion complète de la TVA (HT/TTC)
   - [ ] Ajouter gestion de dates multiples (date facture, échéance, paiement effectif)
   - [ ] Implémenter workflow de validation (Brouillon → À approuver → Approuvé → Rejeté)
   - [ ] Calculer automatiquement la répartition des charges par lot/copropriétaire
   - [ ] Permettre l'attachement de documents PDF aux factures
   - [ ] Créer interface d'approbation pour syndics
   - [ ] Gérer les lignes de facturation détaillées
   
   ---
   
   ## 📐 Spécifications Techniques
   
   ### Architecture
   
   \`\`\`
   Domain (⚠️ À ENRICHIR)
     ├─ entities/expense.rs (ajouter champs TVA, dates, workflow)
     ├─ entities/invoice_line_item.rs (NOUVEAU - lignes de facture)
     └─ entities/charge_distribution.rs (NOUVEAU - répartition)
   
   Application (⚠️ À ENRICHIR)
     ├─ use_cases/expense_use_cases.rs (ajouter méthodes workflow)
     ├─ dto/invoice_dto.rs (NOUVEAU)
     └─ dto/charge_distribution_dto.rs (NOUVEAU)
   
   Infrastructure (⚠️ À ENRICHIR)
     ├─ web/handlers/expense_handlers.rs (nouveaux endpoints)
     └─ database/repositories/... (implémenter nouvelles requêtes)
   
   Frontend (⚠️ À ENRICHIR + CRÉER)
     ├─ components/InvoiceForm.svelte (NOUVEAU)
     ├─ components/InvoiceWorkflow.svelte (NOUVEAU)
     ├─ components/ChargeDistributionTable.svelte (NOUVEAU)
     └─ components/InvoiceApproval.svelte (NOUVEAU)
   \`\`\`
   
   ### Nouveaux Champs pour Expense
   
   **Gestion TVA** :
   - \`amount_excl_vat\` (DECIMAL) - Montant HT
   - \`vat_rate\` (DECIMAL) - Taux TVA (6%, 21%, etc.)
   - \`vat_amount\` (DECIMAL) - Montant TVA
   - \`amount_incl_vat\` (DECIMAL) - Montant TTC (= amount actuel)
   
   **Dates multiples** :
   - \`invoice_date\` (DATE) - Date de la facture
   - \`due_date\` (DATE) - Date d'échéance
   - \`paid_date\` (DATE, nullable) - Date de paiement effectif
   
   **Workflow** :
   - \`approval_status\` (ENUM) - Draft, PendingApproval, Approved, Rejected
   - \`submitted_at\` (TIMESTAMP, nullable)
   - \`approved_by\` (UUID, nullable) - User ID qui a approuvé
   - \`approved_at\` (TIMESTAMP, nullable)
   - \`rejection_reason\` (TEXT, nullable)
   
   ### Endpoints à implémenter
   
   | Méthode | Endpoint | Description | Auth |
   |---------|----------|-------------|------|
   | \`POST\` | \`/api/v1/invoices/draft\` | Créer facture brouillon | Accountant+ |
   | \`PUT\` | \`/api/v1/invoices/:id\` | Modifier brouillon | Accountant+ |
   | \`PUT\` | \`/api/v1/invoices/:id/submit\` | Soumettre pour validation | Accountant+ |
   | \`PUT\` | \`/api/v1/invoices/:id/approve\` | Approuver facture | Syndic+ |
   | \`PUT\` | \`/api/v1/invoices/:id/reject\` | Rejeter avec raison | Syndic+ |
   | \`GET\` | \`/api/v1/invoices/:id/distribution\` | Voir répartition charges | Owner+ |
   | \`POST\` | \`/api/v1/invoices/:id/documents\` | Attacher PDF | Accountant+ |
   | \`GET\` | \`/api/v1/invoices/pending-approval\` | Liste factures à approuver | Syndic+ |
   | \`POST\` | \`/api/v1/invoices/:id/line-items\` | Ajouter ligne de facture | Accountant+ |
   
   ---
   
   ## 📝 User Stories
   
   ### US1 - Création facture avec TVA (Comptable)
   \`\`\`gherkin
   En tant que comptable
   Je veux encoder une facture avec gestion de la TVA
   Afin de respecter les obligations comptables
   
   Scénario: Encodage facture avec TVA 21%
     Étant donné que je suis authentifié en tant que Comptable
     Quand je crée une facture avec :
       - Montant HT: 1000€
       - Taux TVA: 21%
     Alors le système calcule automatiquement :
       - Montant TVA: 210€
       - Montant TTC: 1210€
     Et la facture est créée en statut "Draft"
   \`\`\`
   
   ### US2 - Workflow de validation (Syndic)
   \`\`\`gherkin
   En tant que syndic
   Je veux approuver les factures avant paiement
   Afin de contrôler les dépenses de la copropriété
   
   Scénario: Approbation d'une facture
     Étant donné une facture en statut "PendingApproval"
     Et je suis authentifié en tant que Syndic
     Quand je consulte la facture
     Et je clique sur "Approuver"
     Alors le statut passe à "Approved"
     Et le système calcule la répartition par lot
     Et une notification est envoyée au comptable
   \`\`\`
   
   ### US3 - Répartition automatique (Système)
   \`\`\`gherkin
   En tant que système
   Je veux calculer automatiquement la répartition des charges
   Afin que chaque copropriétaire connaisse sa part
   
   Scénario: Calcul répartition facture 1000€
     Étant donné une facture approuvée de 1000€
     Et un immeuble avec 5 lots
     Et les quotes-parts : Lot A=25%, Lot B=20%, Lot C=20%, Lot D=20%, Lot E=15%
     Quand le système calcule la répartition
     Alors la charge_distribution contient :
       - Lot A: 250€
       - Lot B: 200€
       - Lot C: 200€
       - Lot D: 200€
       - Lot E: 150€
   \`\`\`
   
   ### US4 - Rejet avec motif (Syndic)
   \`\`\`gherkin
   En tant que syndic
   Je veux rejeter une facture incorrecte
   Afin que le comptable puisse la corriger
   
   Scénario: Rejet d'une facture
     Étant donné une facture en statut "PendingApproval"
     Et je suis authentifié en tant que Syndic
     Quand je rejette la facture avec motif "Montant incorrect"
     Alors le statut passe à "Rejected"
     Et la raison est enregistrée
     Et le comptable reçoit une notification
     Et le comptable peut modifier et re-soumettre
   \`\`\`
   
   ---
   
   ## 🔧 Détails d'Implémentation
   
   ### 1. Migration Database
   
   **Fichier** : \`backend/migrations/YYYYMMDD_enrich_expenses_invoice_workflow.sql\`
   
   \`\`\`sql
   -- Ajouter champs TVA
   ALTER TABLE expenses
   ADD COLUMN amount_excl_vat DECIMAL(10,2),
   ADD COLUMN vat_rate DECIMAL(5,2),
   ADD COLUMN vat_amount DECIMAL(10,2),
   ADD COLUMN amount_incl_vat DECIMAL(10,2);
   
   -- Ajouter dates multiples
   ALTER TABLE expenses
   ADD COLUMN invoice_date DATE,
   ADD COLUMN due_date DATE,
   ADD COLUMN paid_date DATE;
   
   -- Ajouter workflow
   CREATE TYPE approval_status AS ENUM ('draft', 'pending_approval', 'approved', 'rejected');
   ALTER TABLE expenses
   ADD COLUMN approval_status approval_status NOT NULL DEFAULT 'draft',
   ADD COLUMN submitted_at TIMESTAMPTZ,
   ADD COLUMN approved_by UUID REFERENCES users(id),
   ADD COLUMN approved_at TIMESTAMPTZ,
   ADD COLUMN rejection_reason TEXT;
   
   -- Créer table lignes de facture
   CREATE TABLE invoice_line_items (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       expense_id UUID NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
       description TEXT NOT NULL,
       quantity DECIMAL(10,2) NOT NULL DEFAULT 1.0,
       unit_price DECIMAL(10,2) NOT NULL,
       amount_excl_vat DECIMAL(10,2) NOT NULL,
       vat_rate DECIMAL(5,2) NOT NULL,
       vat_amount DECIMAL(10,2) NOT NULL,
       amount_incl_vat DECIMAL(10,2) NOT NULL,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   
   -- Créer table répartition charges
   CREATE TABLE charge_distributions (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       expense_id UUID NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
       unit_id UUID NOT NULL REFERENCES units(id) ON DELETE CASCADE,
       owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
       quota_percentage DECIMAL(5,4) NOT NULL,
       amount_due DECIMAL(10,2) NOT NULL,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       UNIQUE(expense_id, unit_id)
   );
   
   -- Index pour performance
   CREATE INDEX idx_expenses_approval_status ON expenses(approval_status);
   CREATE INDEX idx_expenses_invoice_date ON expenses(invoice_date);
   CREATE INDEX idx_expenses_due_date ON expenses(due_date);
   CREATE INDEX idx_invoice_line_items_expense ON invoice_line_items(expense_id);
   CREATE INDEX idx_charge_distributions_expense ON charge_distributions(expense_id);
   CREATE INDEX idx_charge_distributions_unit ON charge_distributions(unit_id);
   \`\`\`
   
   ---
   
   ## ✅ Critères d'Acceptation
   
   ### Fonctionnels
   - [ ] Création facture brouillon avec calcul TVA automatique
   - [ ] Modification brouillon avant soumission
   - [ ] Soumission pour validation change le statut
   - [ ] Syndic peut approuver/rejeter uniquement les factures "PendingApproval"
   - [ ] Approbation déclenche calcul répartition automatique
   - [ ] Répartition basée sur quotes-parts actuelles des propriétaires
   - [ ] Attachement document PDF à la facture
   - [ ] Liste factures filtrée par statut (Draft, PendingApproval, Approved)
   - [ ] Notifications pour approbations requises
   
   ### Sécurité
   - [ ] Comptable ne peut pas approuver (uniquement Syndic/Admin)
   - [ ] Owner en lecture seule sur factures approuvées
   - [ ] Impossible de modifier facture approuvée
   - [ ] Audit log pour toutes actions workflow
   
   ### Performance
   - [ ] Calcul répartition < 500ms pour 100 lots
   - [ ] Liste factures paginée
   
   ### Tests
   - [ ] Tests unitaires calcul TVA
   - [ ] Tests unitaires transitions workflow (Draft → Pending → Approved)
   - [ ] Tests intégration calcul répartition
   - [ ] Tests E2E cycle complet
   - [ ] Tests permissions par rôle
   
   ---
   
   ## 🚀 Checklist de Développement
   
   ### Phase 1 : Domain & Database (4h)
   - [ ] Enrichir entité Expense (champs TVA, dates, workflow)
   - [ ] Créer entité InvoiceLineItem
   - [ ] Créer entité ChargeDistribution
   - [ ] Écrire migration SQL complète
   - [ ] Tests unitaires domain (calculs TVA, workflow)
   
   ### Phase 2 : Application Layer (4h)
   - [ ] Créer DTOs (CreateInvoiceDto, InvoiceWorkflowDto, etc.)
   - [ ] Enrichir ExpenseUseCases avec méthodes workflow
   - [ ] Implémenter calculate_charge_distribution
   - [ ] Implémenter attach_document
   - [ ] Tests intégration use cases
   
   ### Phase 3 : Infrastructure (3h)
   - [ ] Enrichir expense_handlers avec nouveaux endpoints
   - [ ] Implémenter permissions par rôle (check_owner_readonly)
   - [ ] Ajouter routes dans routes.rs
   - [ ] Implémenter repository methods (find_pending_approval, etc.)
   - [ ] Tests E2E endpoints
   
   ### Phase 4 : Frontend (4h)
   - [ ] Créer InvoiceForm.svelte (calcul TVA automatique)
   - [ ] Créer InvoiceWorkflow.svelte (boutons actions)
   - [ ] Créer ChargeDistributionTable.svelte
   - [ ] Créer InvoiceApproval.svelte (pour Syndic)
   - [ ] Enrichir ExpenseList avec filtres statut
   - [ ] Tests frontend (Playwright)
   
   ### Phase 5 : Finitions (1-2h)
   - [ ] Documentation API (OpenAPI/Swagger)
   - [ ] Audit logs pour actions workflow
   - [ ] Configuration notifications (emails)
   - [ ] Tests de charge (performance)
   - [ ] Code review
   - [ ] Commit : \`feat: implement complete invoice encoding workflow\`
   
   ---
   
   **Créé le** : 2025-10-31
   **Dépend de** : Système multi-owner existant (quotes-parts)
   **Lié à** : Issue #002 (gestion documentaire) pour attachement PDF
   **Milestone** : v1.2 - Gestion Financière Avancée

.. raw:: html

   </div>

