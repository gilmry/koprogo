=============================================================
Issue #45: feat: Implement file upload UI with drag-and-drop
=============================================================

:State: **CLOSED**
:Milestone: Phase 1: VPS MVP + Legal Compliance
:Labels: phase:vps,track:software priority:high
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-01
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/45>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   **Backend implementation:** ✅ **70% complete**
   - Document entity with 7 types
   - FileStorage service (needs S3/MinIO integration - see #44)
   - Document upload/download API endpoints
   - MIME type support, file metadata tracking
   
   **Frontend implementation:** ⚠️ **30% complete**
   - DocumentList component displays documents
   - **Missing:** Upload UI, drag-and-drop, progress indicators
   
   ## Objective
   
   Implement user-friendly file upload UI with drag-and-drop for documents.
   
   ## Dependencies
   
   **⚠️ Blocked by:** Issue #44 (Document storage strategy - Local/MinIO/S3)
   - Upload UI can be developed in parallel
   - Integration requires storage backend decision
   
   ## API Endpoint (Already Implemented)
   
   `POST /api/v1/documents/upload` (multipart/form-data)
   
   ## Implementation Plan
   
   ### 1. File Upload Component
   
   **Create:** `frontend/src/components/FileUpload.svelte`
   
   Features:
   - File input button
   - Drag-and-drop zone
   - Multi-file upload
   - Upload progress bars (XHR progress events)
   - File size validation (max 50MB)
   - MIME type validation
   - Document type selector
   - Building/Meeting/Expense context
   - Error handling
   
   ### 2. File Preview Component
   
   **Create:** `frontend/src/components/FilePreview.svelte`
   - Image preview (JPEG, PNG)
   - PDF preview (thumbnail or icon)
   - File type icons
   
   ### 3. Integration with Documents Page
   
   **Update:** `frontend/src/pages/documents.astro`
   - Add FileUpload component
   - Refresh DocumentList after upload
   - Success/error toasts
   
   ## Testing
   
   - [ ] Single file upload
   - [ ] Multi-file upload
   - [ ] Drag-and-drop
   - [ ] Progress indicator
   - [ ] Size validation (client + server)
   - [ ] Type validation
   - [ ] Document list refresh
   
   ## Accessibility
   
   - [ ] Keyboard accessible
   - [ ] Screen reader support
   - [ ] Error announcements (role="alert")
   - [ ] Progress aria-valuenow
   
   ## Acceptance Criteria
   
   - [ ] FileUpload component complete
   - [ ] Drag-and-drop functional
   - [ ] Multi-file support
   - [ ] Progress visible
   - [ ] Document type selector
   - [ ] Image preview
   - [ ] Validation (size, type)
   - [ ] Integrated with documents page
   
   ## Effort Estimate
   
   **Medium** (2 days)
   
   ## Related
   
   - **Depends on:** Issue #44 (storage backend)
   - Supports: Meeting minutes, invoices, contracts
   
   ## References
   
   - MDN File API: https://developer.mozilla.org/en-US/docs/Web/API/File
   - Drag and Drop API: https://developer.mozilla.org/en-US/docs/Web/API/HTML_Drag_and_Drop_API

.. raw:: html

   </div>

