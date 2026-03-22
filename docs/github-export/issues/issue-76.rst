====================================================================================
Issue #76: feat: Document Upload & Download System (Gestion documentaire complète)
====================================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: enhancement,phase:vps track:software,priority:critical
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2025-11-17
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/76>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #002 - Gestion Documentaire Complète
   
   **Priorité**: 🔴 CRITIQUE  
   **Estimation**: 8-10 heures  
   **Phase**: VPS MVP (Nov 2025 - Mar 2026)  
   
   ## 📋 Description
   
   Système complet de gestion documentaire avec upload/download de documents (PV assemblées, factures, règlements intérieurs, contrats prestataires). Le système doit supporter le stockage sécurisé, la catégorisation et l'accès contrôlé aux documents.
   
   **Contexte métier**: Une copropriété génère de nombreux documents légaux qui doivent être accessibles aux copropriétaires et archivés pendant des années.
   
   ## 🎯 Objectifs
   
   - [ ] Implémenter le trait `FileStorageService` (abstraction stockage)
   - [ ] Créer les handlers upload/download (multipart form-data)
   - [ ] Ajouter catégorisation documents (PV, factures, contrats, etc.)
   - [ ] Implémenter le contrôle d'accès par rôle
   - [ ] Créer le composant Svelte avec drag-and-drop
   - [ ] Ajouter preview images/PDFs
   - [ ] Tests E2E upload/download
   
   ## 📐 Spécifications Techniques
   
   ### Types de Documents
   
   ```rust
   pub enum DocumentType {
       MeetingMinutes,    // Procès-verbaux AG
       Invoice,           // Factures
       Contract,          // Contrats prestataires
       Regulation,        // Règlement intérieur
       Insurance,         // Assurances
       WorkReport,        // Rapports travaux
       Other,             // Autre
   }
   ```
   
   ### Endpoints
   
   | Méthode | Endpoint | Description | Auth |
   |---------|----------|-------------|------|
   | `POST` | `/api/v1/documents/upload` | Upload un document | Syndic+ |
   | `GET` | `/api/v1/documents/:id` | Télécharger un document | Owner+ |
   | `GET` | `/api/v1/documents` | Lister tous les documents | Owner+ |
   | `DELETE` | `/api/v1/documents/:id` | Supprimer un document | Syndic+ |
   | `GET` | `/api/v1/buildings/:id/documents` | Documents d'un immeuble | Owner+ |
   
   ## 🔗 Dépendances
   
   **Dépend de**: #44 (Storage Strategy - CLOSED ✅)  
   **Bloque**: #017 (État Daté PDF), #020 (Carnet Entretien), #024 (Devis)
   
   ## 📚 Frontend Component
   
   ```svelte
   <FileUploader
     accept=".pdf,.jpg,.png"
     maxSize={10 * 1024 * 1024}
     onUpload={handleUpload}
     showPreview={true}
   />
   ```
   
   ## ✅ Critères d'Acceptation
   
   - Upload multipart form-data fonctionnel
   - Validation taille max (10MB par fichier)
   - Validation types MIME (PDF, images, Office)
   - Preview images et PDFs dans UI
   - Drag & drop fonctionnel
   - Progress bar upload
   - Tests E2E complets
   
   ---
   
   **Voir**: `issues/critical/002-document-upload-download.md` pour détails complets

.. raw:: html

   </div>

