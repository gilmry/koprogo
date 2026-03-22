==================================================================================
Issue #52: feat: Contractor backoffice (Work reports, photos, payment validation)
==================================================================================

:State: **CLOSED**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: phase:k3s,track:software priority:medium
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2026-02-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/52>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   **Prestataires/Entrepreneurs** effectuent travaux dans l'immeuble :
   - Plombier
   - Électricien
   - Peintre
   - Société de nettoyage
   - Jardinier
   - Ascensoriste
   - Entrepreneur général
   
   **Problème actuel :**
   - Pas de traçabilité des interventions
   - Pas de preuve photographique des travaux
   - Communication email/téléphone fragmentée
   - Validation paiement manuelle et lente
   
   **Besoin :**
   Espace dédié pour que les prestataires documentent leurs interventions et déclenchent la validation de paiement.
   
   ## Objective
   
   **Backoffice léger pour prestataires** permettant :
   1. Poster compte-rendu d'intervention
   2. Uploader photos (avant/après, pièces changées, résultat fini)
   3. Déclarer intervention terminée
   4. Déclencher validation paiement par Syndic
   5. Suivre statut facture (en attente, approuvée, payée)
   
   ## User Flow
   
   ```
   Entrepreneur termine travaux
            ↓
   Se connecte au portail KoproGo (login simplifié)
            ↓
   Crée compte-rendu intervention
     - Sélectionne chantier/tâche assignée
     - Description travaux effectués
     - Upload photos (avant/après, pièces changées)
     - Temps passé (heures)
     - Matériaux utilisés
            ↓
   Marque intervention comme "Terminée"
            ↓
   Système notifie Syndic
            ↓
   Syndic voit compte-rendu + photos
            ↓
   Syndic valide ou demande corrections
            ↓
   Si validé → Déclenche paiement
            ↓
   Entrepreneur voit statut "Approuvé - Paiement en cours"
   ```
   
   ## Domain Model
   
   ### 1. Contractor (Prestataire)
   
   ```rust
   pub struct Contractor {
       pub id: Uuid,
       pub company_name: String,
       pub contact_person: String,
       pub email: String,
       pub phone: String,
       pub vat_number: String,  // TVA
       pub iban: Option<String>,
       pub contractor_type: ContractorType,
       pub is_active: bool,
       pub created_at: DateTime<Utc>,
   }
   
   pub enum ContractorType {
       Plumber,        // Plombier
       Electrician,    // Électricien
       Painter,        // Peintre
       Cleaner,        // Nettoyage
       Gardener,       // Jardinier
       Elevator,       // Ascensoriste
       GeneralWork,    // Travaux généraux
       Other,
   }
   ```
   
   ### 2. Work Order (Ordre de Travail)
   
   ```rust
   pub struct WorkOrder {
       pub id: Uuid,
       pub building_id: Uuid,
       pub contractor_id: Uuid,
       pub title: String,
       pub description: String,
       pub location: String,  // "Hall d'entrée", "Appartement 3B"
       pub quote_amount: Option<Decimal>,  // Montant devis
       pub status: WorkOrderStatus,
       pub assigned_at: DateTime<Utc>,
       pub due_date: Option<DateTime<Utc>>,
       pub related_issue_id: Option<Uuid>,  // Lien avec signalement
       pub related_expense_id: Option<Uuid>,
   }
   
   pub enum WorkOrderStatus {
       Assigned,      // Assigné à entrepreneur
       InProgress,    // En cours
       Completed,     // Terminé par entrepreneur
       Validated,     // Validé par Syndic
       PaymentPending, // Paiement en attente
       Paid,          // Payé
       Rejected,      // Refusé par Syndic
   }
   ```
   
   ### 3. Work Report (Compte-Rendu)
   
   ```rust
   pub struct WorkReport {
       pub id: Uuid,
       pub work_order_id: Uuid,
       pub contractor_id: Uuid,
       pub summary: String,
       pub detailed_description: String,
       pub work_date: DateTime<Utc>,
       pub hours_spent: f32,
       pub materials_used: Vec<MaterialUsed>,
       pub before_photos: Vec<String>,  // Photos avant travaux
       pub after_photos: Vec<String>,   // Photos après travaux
       pub parts_photos: Vec<String>,   // Photos pièces changées
       pub submitted_at: DateTime<Utc>,
       pub validated_by: Option<Uuid>,  // Syndic
       pub validated_at: Option<DateTime<Utc>>,
       pub validation_notes: Option<String>,
   }
   
   pub struct MaterialUsed {
       pub name: String,
       pub quantity: i32,
       pub unit: String,  // "pièce", "mètre", "litre"
       pub cost: Option<Decimal>,
   }
   ```
   
   ### 4. Contractor Invoice (Facture)
   
   ```rust
   pub struct ContractorInvoice {
       pub id: Uuid,
       pub work_order_id: Uuid,
       pub contractor_id: Uuid,
       pub invoice_number: String,
       pub invoice_date: DateTime<Utc>,
       pub amount_excl_vat: Decimal,
       pub vat_rate: f32,
       pub amount_incl_vat: Decimal,
       pub invoice_file_url: Option<String>,  // PDF facture
       pub status: InvoiceStatus,
       pub payment_deadline: DateTime<Utc>,
       pub paid_at: Option<DateTime<Utc>>,
   }
   
   pub enum InvoiceStatus {
       Draft,          // Brouillon
       Submitted,      // Soumise
       Approved,       // Approuvée par Syndic
       Rejected,       // Refusée
       PaymentScheduled, // Paiement planifié
       Paid,           // Payée
   }
   ```
   
   ## API Endpoints
   
   ### Contractor Authentication
   
   **Login simplifié (pas de compte utilisateur complet) :**
   - Email + code PIN (envoyé par Syndic)
   - ou Email + lien magique (passwordless)
   
   ```
   POST /api/v1/contractors/auth/login
   POST /api/v1/contractors/auth/verify-code
   ```
   
   ### Work Orders (Read-only pour contractor)
   
   ```
   GET /api/v1/contractors/:id/work-orders
   GET /api/v1/work-orders/:id
   ```
   
   ### Work Reports
   
   ```
   POST /api/v1/work-orders/:id/reports
   PUT /api/v1/work-reports/:id
   POST /api/v1/work-reports/:id/photos
   PUT /api/v1/work-reports/:id/submit
   ```
   
   ### Invoices
   
   ```
   POST /api/v1/work-orders/:id/invoice
   GET /api/v1/contractors/:id/invoices
   ```
   
   ### Syndic validation
   
   ```
   PUT /api/v1/work-reports/:id/validate
   PUT /api/v1/work-reports/:id/reject
   PUT /api/v1/invoices/:id/approve
   PUT /api/v1/invoices/:id/mark-paid
   ```
   
   ## Frontend - Contractor Portal
   
   ### 1. Login Page
   
   ```svelte
   <!-- /contractor-portal/login.astro -->
   <ContractorLoginForm />
   ```
   
   **Formulaire :**
   - Email
   - Code PIN (4-6 chiffres)
   - Bouton "Renvoyer code"
   
   ### 2. Dashboard Contractor
   
   ```svelte
   <!-- /contractor-portal/dashboard.astro -->
   <ContractorDashboard contractorId={session.contractor_id} />
   ```
   
   **Sections :**
   - Chantiers en cours (3)
   - Chantiers terminés en attente validation (2)
   - Factures en attente paiement (€2,450)
   - Derniers paiements reçus
   
   ### 3. Work Order Detail + Report Form
   
   ```svelte
   <!-- /contractor-portal/work-order/[id].astro -->
   <WorkOrderDetail workOrderId={id} />
   <WorkReportForm workOrderId={id} />
   ```
   
   **Formulaire compte-rendu :**
   
   ```svelte
   <script lang="ts">
     let summary = '';
     let detailedDescription = '';
     let hoursSpent = 0;
     let materialsUsed = [];
     let beforePhotos = [];
     let afterPhotos = [];
     let partsPhotos = [];
     
     function addMaterial() {
       materialsUsed.push({
         name: '',
         quantity: 1,
         unit: 'pièce',
         cost: 0
       });
     }
     
     async function submitReport() {
       const formData = new FormData();
       formData.append('summary', summary);
       formData.append('detailed_description', detailedDescription);
       formData.append('hours_spent', hoursSpent);
       formData.append('materials_used', JSON.stringify(materialsUsed));
       
       beforePhotos.forEach(photo => formData.append('before_photos', photo));
       afterPhotos.forEach(photo => formData.append('after_photos', photo));
       partsPhotos.forEach(photo => formData.append('parts_photos', photo));
       
       await fetch(`/api/v1/work-orders/${workOrderId}/reports`, {
         method: 'POST',
         body: formData
       });
       
       showToast('Compte-rendu soumis avec succès');
     }
   </script>
   
   <form on:submit|preventDefault={submitReport}>
     <h2>Compte-rendu d'intervention</h2>
     
     <input 
       type="text" 
       bind:value={summary}
       placeholder="Résumé (ex: Remplacement robinet fuyant)"
       required
     />
     
     <textarea 
       bind:value={detailedDescription}
       placeholder="Description détaillée des travaux effectués"
       rows="6"
       required
     />
     
     <label>
       Heures passées
       <input type="number" bind:value={hoursSpent} step="0.5" min="0" />
     </label>
     
     <h3>Matériaux utilisés</h3>
     {#each materialsUsed as material, i}
       <div class="material-row">
         <input bind:value={material.name} placeholder="Nom" />
         <input bind:value={material.quantity} type="number" placeholder="Qté" />
         <input bind:value={material.unit} placeholder="Unité" />
         <input bind:value={material.cost} type="number" step="0.01" placeholder="Coût (€)" />
         <button on:click={() => materialsUsed.splice(i, 1)}>❌</button>
       </div>
     {/each}
     <button type="button" on:click={addMaterial}>+ Ajouter matériau</button>
     
     <h3>Photos</h3>
     
     <FileUpload
       bind:files={beforePhotos}
       label="Photos AVANT travaux"
       accept="image/*"
       multiple
     />
     
     <FileUpload
       bind:files={partsPhotos}
       label="Photos pièces changées"
       accept="image/*"
       multiple
     />
     
     <FileUpload
       bind:files={afterPhotos}
       label="Photos APRÈS travaux / Résultat fini"
       accept="image/*"
       multiple
     />
     
     <button type="submit">Soumettre compte-rendu</button>
   </form>
   ```
   
   ### 4. Invoice Upload
   
   ```svelte
   <InvoiceUploadForm workOrderId={workOrderId} />
   ```
   
   **Champs :**
   - Numéro facture
   - Date facture
   - Montant HT
   - Taux TVA (21% BE par défaut)
   - Upload PDF facture
   - Date échéance paiement
   
   ## Frontend - Syndic View
   
   ### Work Report Validation
   
   ```svelte
   <!-- /admin/work-reports/[id].astro -->
   <WorkReportReview reportId={id} />
   ```
   
   **Vue Syndic :**
   - Infos chantier
   - Compte-rendu entrepreneur
   - Galerie photos (avant/après/pièces)
   - Matériaux utilisés avec coûts
   - Heures passées
   - Actions :
     - ✅ Valider → Déclenche approbation facture
     - ❌ Rejeter → Demander corrections
     - 💬 Demander clarifications
   
   ## Notifications
   
   **Entrepreneur reçoit :**
   - Nouveau chantier assigné
   - Compte-rendu validé
   - Compte-rendu rejeté (avec raisons)
   - Facture approuvée
   - Paiement effectué
   
   **Syndic reçoit :**
   - Compte-rendu soumis (à valider)
   - Facture soumise (à approuver)
   - Échéance paiement approche
   
   ## Security & Access Control
   
   **Contractor access :**
   - Voit uniquement SES chantiers
   - Voit uniquement SES factures
   - Pas d'accès aux autres données immeuble
   - Session limitée (24h expiration)
   - Code PIN rotatif (nouveau code tous les 3 mois)
   
   **Syndic :**
   - Assigne chantiers aux contractors
   - Valide comptes-rendus
   - Approuve factures
   - Marque paiements effectués
   
   ## Testing
   
   - [ ] Contractor login with PIN code
   - [ ] View assigned work orders
   - [ ] Submit work report with photos
   - [ ] Upload materials used
   - [ ] Submit invoice
   - [ ] Syndic validates report
   - [ ] Syndic approves invoice
   - [ ] Syndic marks as paid
   - [ ] Notifications delivery
   - [ ] Photo gallery display
   
   ## Acceptance Criteria
   
   - [ ] Contractor entities + authentication
   - [ ] Work order management
   - [ ] Work report submission with photos
   - [ ] Material tracking
   - [ ] Invoice submission
   - [ ] Syndic validation workflow
   - [ ] Contractor portal UI (login, dashboard, report form)
   - [ ] Syndic review UI
   - [ ] Photo upload/gallery
   - [ ] Notifications configured
   - [ ] Access control enforced
   - [ ] Mobile-responsive (contractors use phones on-site)
   
   ## Effort Estimate
   
   **Large** (8-10 days)
   - Days 1-2: Contractor entities + auth (PIN code)
   - Days 3-4: Work report submission + photos
   - Days 5-6: Invoice management
   - Days 7-8: Syndic validation UI
   - Days 9-10: Notifications, testing
   
   ## Benefits
   
   **Pour Syndic :**
   - Traçabilité complète des interventions
   - Preuve photographique des travaux
   - Validation paiement facilitée
   - Historique consultable
   - Réduction litiges
   
   **Pour Entrepreneur :**
   - Paiements plus rapides (validation digitale)
   - Pas d'emails/papiers perdus
   - Historique travaux accessible
   - Proof of work pour portfolio
   
   **Pour ASBL :**
   - Professionnalisation gestion copropriété
   - Transparence pour copropriétaires
   - Réduction charges administratives
   
   ## Future Enhancements
   
   - Signature électronique PV intervention
   - Géolocalisation check-in/check-out (proof of presence)
   - Time tracking automatique (chronomètre)
   - Rating/reviews entrepreneurs par Syndic
   - Marketplace entrepreneurs (annuaire)
   - Automated reminders for overdue work
   - Integration comptabilité (export factures vers logiciel compta)
   
   ## References
   
   - Fieldwire (construction management): https://www.fieldwire.com/
   - Procore (contractor portal): https://www.procore.com/
   - ServiceTitan (field service software): https://www.servicetitan.com/

.. raw:: html

   </div>

