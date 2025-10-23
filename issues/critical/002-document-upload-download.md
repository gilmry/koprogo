# Issue #002 - Gestion Documentaire Complète (Upload/Download)

**Priorité**: 🔴 CRITIQUE
**Estimation**: 8-10 heures
**Labels**: `enhancement`, `backend`, `frontend`, `critical`, `file-storage`

---

## 📋 Description

Implémenter le système complet de gestion documentaire avec upload, download et stockage de fichiers. L'entité `Document` et le schéma DB existent, mais aucune fonctionnalité de manipulation de fichiers n'est disponible.

**Contexte métier** : Les documents (règlements, PV d'AG, factures, contrats) sont essentiels pour la gestion de copropriété et doivent être archivés de manière sécurisée avec traçabilité.

---

## 🎯 Objectifs

- [ ] Implémenter l'upload de fichiers multipart/form-data
- [ ] Stocker les fichiers de manière sécurisée (local ou cloud)
- [ ] Permettre le téléchargement avec contrôle d'accès
- [ ] Gérer les types MIME et validation de fichiers
- [ ] Indexer les métadonnées dans PostgreSQL
- [ ] Créer l'interface frontend pour upload/download
- [ ] Implémenter la suppression sécurisée (soft delete)

---

## 📐 Spécifications Techniques

### Architecture

```
Domain (✅ EXISTANT)
  └─ entities/document.rs (Document, DocumentType)

Application (❌ À CRÉER)
  ├─ use_cases/document_use_cases.rs
  ├─ dto/document_dto.rs
  └─ services/file_storage_service.rs (trait)

Infrastructure (❌ À CRÉER)
  ├─ web/handlers/document_handlers.rs
  ├─ storage/local_storage.rs (implémentation locale)
  ├─ storage/s3_storage.rs (optionnel, pour cloud)
  └─ web/routes.rs (ajouter routes documents)

Frontend (❌ À CRÉER)
  └─ src/components/FileUpload.svelte
```

### Endpoints à implémenter

| Méthode | Endpoint | Description | Auth |
|---------|----------|-------------|------|
| `POST` | `/api/v1/documents` | Upload document | Owner+ |
| `GET` | `/api/v1/documents` | Lister documents | Owner+ |
| `GET` | `/api/v1/documents/:id` | Métadonnées document | Owner+ |
| `GET` | `/api/v1/documents/:id/download` | Télécharger fichier | Owner+ |
| `DELETE` | `/api/v1/documents/:id` | Supprimer document | Syndic+ |
| `GET` | `/api/v1/buildings/:id/documents` | Documents d'un immeuble | Owner+ |
| `GET` | `/api/v1/meetings/:id/documents` | Documents d'une AG | Owner+ |
| `GET` | `/api/v1/expenses/:id/documents` | Documents d'une dépense | Owner+ |

---

## 📝 User Stories

### US1 - Upload d'un document (Syndic)
```gherkin
En tant que syndic
Je veux uploader un règlement de copropriété
Afin que les copropriétaires puissent le consulter

Scénario: Upload réussi d'un PDF
  Étant donné que je suis authentifié en tant que Syndic
  Quand j'uploade un fichier "reglement.pdf" avec :
    - title: "Règlement de copropriété"
    - document_type: RegulationDocument
    - building_id: <uuid>
  Alors le document est stocké sur le système de fichiers
  Et les métadonnées sont enregistrées en base
  Et je reçois l'id du document créé
```

### US2 - Téléchargement sécurisé (Copropriétaire)
```gherkin
En tant que copropriétaire
Je veux télécharger le PV de la dernière AG
Afin de consulter les décisions prises

Scénario: Téléchargement autorisé
  Étant donné qu'un document PV existe avec id "doc-123"
  Et je suis copropriétaire du building concerné
  Quand je demande GET /documents/doc-123/download
  Alors je reçois le fichier PDF avec headers appropriés
  Et le Content-Type est "application/pdf"
```

### US3 - Restriction d'accès (Sécurité)
```gherkin
En tant que système
Je veux empêcher l'accès à des documents non autorisés
Afin de protéger la confidentialité

Scénario: Accès refusé à un document d'un autre immeuble
  Étant donné que je suis copropriétaire du building A
  Et un document existe pour le building B
  Quand je demande GET /documents/<building-B-doc>/download
  Alors je reçois une erreur 403 Forbidden
```

---

## 🔧 Détails d'Implémentation

### 1. File Storage Service (Trait)

**Fichier** : `backend/src/application/services/file_storage_service.rs`

```rust
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait FileStorageService: Send + Sync {
    /// Stocke un fichier et retourne le chemin de stockage
    async fn store_file(
        &self,
        file_data: Vec<u8>,
        file_name: &str,
        content_type: &str,
    ) -> Result<String, String>;

    /// Récupère un fichier par son chemin
    async fn retrieve_file(&self, file_path: &str) -> Result<Vec<u8>, String>;

    /// Supprime un fichier
    async fn delete_file(&self, file_path: &str) -> Result<(), String>;

    /// Génère une URL signée (pour cloud storage)
    async fn generate_signed_url(
        &self,
        file_path: &str,
        expiry_seconds: u64,
    ) -> Result<String, String>;
}
```

### 2. Local Storage Implementation

**Fichier** : `backend/src/infrastructure/storage/local_storage.rs`

```rust
use crate::application::services::file_storage_service::FileStorageService;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

pub struct LocalStorageService {
    base_path: PathBuf,
}

impl LocalStorageService {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
        }
    }

    fn sanitize_filename(&self, filename: &str) -> String {
        // Remplacer caractères dangereux, limiter longueur
        filename
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .collect::<String>()
            .chars()
            .take(255)
            .collect()
    }

    fn generate_storage_path(&self, original_name: &str) -> String {
        let uuid = Uuid::new_v4();
        let sanitized = self.sanitize_filename(original_name);
        format!("{}/{}", uuid, sanitized)
    }
}

#[async_trait]
impl FileStorageService for LocalStorageService {
    async fn store_file(
        &self,
        file_data: Vec<u8>,
        file_name: &str,
        _content_type: &str,
    ) -> Result<String, String> {
        let relative_path = self.generate_storage_path(file_name);
        let full_path = self.base_path.join(&relative_path);

        // Créer dossier parent si nécessaire
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Écrire fichier
        fs::write(&full_path, file_data)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(relative_path)
    }

    async fn retrieve_file(&self, file_path: &str) -> Result<Vec<u8>, String> {
        let full_path = self.base_path.join(file_path);

        // Vérifier path traversal attack
        if !full_path.starts_with(&self.base_path) {
            return Err("Invalid file path".to_string());
        }

        fs::read(&full_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))
    }

    async fn delete_file(&self, file_path: &str) -> Result<(), String> {
        let full_path = self.base_path.join(file_path);

        if !full_path.starts_with(&self.base_path) {
            return Err("Invalid file path".to_string());
        }

        fs::remove_file(&full_path)
            .await
            .map_err(|e| format!("Failed to delete file: {}", e))
    }

    async fn generate_signed_url(
        &self,
        file_path: &str,
        _expiry_seconds: u64,
    ) -> Result<String, String> {
        // Pour local storage, retourner URL directe
        Ok(format!("/api/v1/documents/{}/download", file_path))
    }
}
```

### 3. Document Use Cases

**Fichier** : `backend/src/application/use_cases/document_use_cases.rs`

```rust
use crate::application::ports::document_repository::DocumentRepository;
use crate::application::services::file_storage_service::FileStorageService;
use crate::application::dto::document_dto::*;
use crate::domain::entities::document::{Document, DocumentType};
use std::sync::Arc;
use uuid::Uuid;

pub struct DocumentUseCases {
    document_repo: Arc<dyn DocumentRepository>,
    storage_service: Arc<dyn FileStorageService>,
}

impl DocumentUseCases {
    pub fn new(
        document_repo: Arc<dyn DocumentRepository>,
        storage_service: Arc<dyn FileStorageService>,
    ) -> Self {
        Self {
            document_repo,
            storage_service,
        }
    }

    pub async fn upload_document(
        &self,
        file_data: Vec<u8>,
        file_name: String,
        content_type: String,
        request: CreateDocumentRequest,
    ) -> Result<DocumentResponse, String> {
        // 1. Valider taille fichier (max 10MB)
        if file_data.len() > 10 * 1024 * 1024 {
            return Err("File size exceeds 10MB limit".to_string());
        }

        // 2. Valider type MIME
        let allowed_types = vec![
            "application/pdf",
            "image/jpeg",
            "image/png",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ];
        if !allowed_types.contains(&content_type.as_str()) {
            return Err("Invalid file type".to_string());
        }

        // 3. Stocker fichier
        let file_path = self
            .storage_service
            .store_file(file_data.clone(), &file_name, &content_type)
            .await?;

        // 4. Créer entité Document
        let file_size = file_data.len() as i64;
        let document = Document::new(
            request.title,
            file_path.clone(),
            request.document_type,
            file_size,
            request.building_id,
            request.meeting_id,
            request.expense_id,
        )?;

        // 5. Persister en base
        let saved = self.document_repo.create(&document).await?;

        Ok(DocumentResponse::from(saved))
    }

    pub async fn download_document(
        &self,
        document_id: Uuid,
        user_building_ids: Vec<Uuid>, // Pour vérifier accès
    ) -> Result<(Vec<u8>, String), String> {
        // 1. Récupérer document
        let document = self.document_repo.find_by_id(document_id).await?;

        // 2. Vérifier autorisation
        if let Some(building_id) = document.building_id {
            if !user_building_ids.contains(&building_id) {
                return Err("Access denied".to_string());
            }
        }

        // 3. Récupérer fichier
        let file_data = self
            .storage_service
            .retrieve_file(&document.file_path)
            .await?;

        // 4. Détecter content-type depuis extension
        let content_type = self.detect_content_type(&document.file_path);

        Ok((file_data, content_type))
    }

    fn detect_content_type(&self, file_path: &str) -> String {
        if file_path.ends_with(".pdf") {
            "application/pdf".to_string()
        } else if file_path.ends_with(".jpg") || file_path.ends_with(".jpeg") {
            "image/jpeg".to_string()
        } else if file_path.ends_with(".png") {
            "image/png".to_string()
        } else {
            "application/octet-stream".to_string()
        }
    }
}
```

### 4. Document Handlers

**Fichier** : `backend/src/infrastructure/web/handlers/document_handlers.rs`

```rust
use actix_web::{web, HttpResponse, HttpRequest, Result};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use crate::application::use_cases::document_use_cases::DocumentUseCases;
use std::sync::Arc;
use uuid::Uuid;

pub async fn upload_document(
    use_cases: web::Data<Arc<DocumentUseCases>>,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let mut file_data = Vec::new();
    let mut file_name = String::new();
    let mut content_type = String::new();
    let mut title = String::new();
    let mut document_type = String::new();
    let mut building_id: Option<Uuid> = None;

    // Parser multipart form
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let field_name = field.name().to_string();

        match field_name.as_str() {
            "file" => {
                file_name = field
                    .content_disposition()
                    .get_filename()
                    .unwrap_or("unknown")
                    .to_string();
                content_type = field
                    .content_type()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    file_data.extend_from_slice(&data);
                }
            }
            "title" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    title.push_str(&String::from_utf8_lossy(&data));
                }
            }
            "document_type" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    document_type.push_str(&String::from_utf8_lossy(&data));
                }
            }
            "building_id" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    let id_str = String::from_utf8_lossy(&data);
                    building_id = Uuid::parse_str(&id_str).ok();
                }
            }
            _ => {}
        }
    }

    // Créer request DTO
    let request = CreateDocumentRequest {
        title,
        document_type: document_type.parse().unwrap(), // À améliorer
        building_id,
        meeting_id: None,
        expense_id: None,
    };

    match use_cases
        .upload_document(file_data, file_name, content_type, request)
        .await
    {
        Ok(doc) => Ok(HttpResponse::Created().json(doc)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

pub async fn download_document(
    use_cases: web::Data<Arc<DocumentUseCases>>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let document_id = path.into_inner();

    // TODO: Extraire building_ids du JWT token
    let user_building_ids = vec![]; // À compléter avec auth middleware

    match use_cases
        .download_document(document_id, user_building_ids)
        .await
    {
        Ok((file_data, content_type)) => Ok(HttpResponse::Ok()
            .content_type(content_type)
            .append_header(("Content-Disposition", "attachment"))
            .body(file_data)),
        Err(e) => Ok(HttpResponse::NotFound().json(e)),
    }
}
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Upload de fichiers PDF, JPG, PNG, XLSX réussi
- [ ] Taille maximale 10MB par fichier
- [ ] Téléchargement avec bon Content-Type
- [ ] Restriction d'accès par building_id
- [ ] Suppression soft delete (marquer deleted_at)
- [ ] Liste documents filtrée par building/meeting/expense

### Sécurité
- [ ] Pas de path traversal attack possible
- [ ] Validation des types MIME
- [ ] Sanitization des noms de fichiers
- [ ] Vérification des permissions avant download
- [ ] Pas de stockage de fichiers exécutables

### Performance
- [ ] Streaming pour fichiers > 1MB
- [ ] Pas de chargement complet en mémoire
- [ ] P99 < 100ms pour upload < 5MB

### Tests
- [ ] Tests upload réussi
- [ ] Tests upload fichier trop gros
- [ ] Tests upload type invalide
- [ ] Tests download autorisé
- [ ] Tests download non autorisé (403)
- [ ] Tests suppression

---

## 🧪 Plan de Tests

### Tests E2E

```rust
#[actix_rt::test]
async fn test_upload_pdf_success() {
    // Créer multipart form avec PDF
    // POST /documents
    // Vérifier 201 Created
    // Vérifier fichier existe sur disque
}

#[actix_rt::test]
async fn test_upload_file_too_large() {
    // Créer fichier 11MB
    // POST /documents
    // Vérifier 400 Bad Request
}

#[actix_rt::test]
async fn test_download_authorized() {
    // Upload document pour building A
    // Authentifier user du building A
    // GET /documents/{id}/download
    // Vérifier 200 OK avec content
}

#[actix_rt::test]
async fn test_download_unauthorized() {
    // Upload document pour building A
    // Authentifier user du building B
    // GET /documents/{id}/download
    // Vérifier 403 Forbidden
}

#[actix_rt::test]
async fn test_path_traversal_protection() {
    // Tenter download avec path "../../../etc/passwd"
    // Vérifier erreur
}
```

---

## 🎨 Frontend Component

**Fichier** : `frontend/src/components/FileUpload.svelte`

```svelte
<script lang="ts">
  import { getApiUrl } from '../stores/config';
  import { authStore } from '../stores/auth';

  export let buildingId: string;
  export let documentType: string = 'Invoice';

  let file: File | null = null;
  let title: string = '';
  let uploading = false;
  let error: string = '';
  let success = false;

  async function handleUpload() {
    if (!file) {
      error = 'Veuillez sélectionner un fichier';
      return;
    }

    uploading = true;
    error = '';

    const formData = new FormData();
    formData.append('file', file);
    formData.append('title', title || file.name);
    formData.append('document_type', documentType);
    formData.append('building_id', buildingId);

    try {
      const response = await fetch(`${getApiUrl()}/documents`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${$authStore.token}`,
        },
        body: formData,
      });

      if (!response.ok) {
        throw new Error(await response.text());
      }

      success = true;
      file = null;
      title = '';
    } catch (e) {
      error = e.message;
    } finally {
      uploading = false;
    }
  }
</script>

<div class="file-upload">
  <h3>Téléverser un document</h3>

  <input
    type="file"
    accept=".pdf,.jpg,.jpeg,.png,.xlsx"
    bind:files={file}
    disabled={uploading}
  />

  <input
    type="text"
    placeholder="Titre du document (optionnel)"
    bind:value={title}
    disabled={uploading}
  />

  <select bind:value={documentType} disabled={uploading}>
    <option value="Invoice">Facture</option>
    <option value="MeetingMinutes">Procès-verbal</option>
    <option value="RegulationDocument">Règlement</option>
    <option value="Contract">Contrat</option>
    <option value="Report">Rapport</option>
  </select>

  <button on:click={handleUpload} disabled={uploading || !file}>
    {uploading ? 'Upload en cours...' : 'Téléverser'}
  </button>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if success}
    <p class="success">Document uploadé avec succès !</p>
  {/if}
</div>

<style>
  .file-upload {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    max-width: 500px;
    padding: 2rem;
    border: 1px solid #ccc;
    border-radius: 8px;
  }

  .error {
    color: red;
  }

  .success {
    color: green;
  }
</style>
```

---

## 🔗 Dépendances

### Cargo.toml (nouveaux crates)

```toml
actix-multipart = "0.6"
futures-util = "0.3"
mime = "0.3"
```

### Optionnelles (cloud storage)
- `rusoto_s3` pour AWS S3
- `azure_storage_blobs` pour Azure Blob Storage

---

## 🚀 Checklist de Développement

- [ ] 1. Créer trait FileStorageService
- [ ] 2. Implémenter LocalStorageService
- [ ] 3. Créer DocumentUseCases
- [ ] 4. Créer DocumentDTOs
- [ ] 5. Implémenter document_handlers (upload/download)
- [ ] 6. Ajouter routes dans routes.rs
- [ ] 7. Configurer LocalStorageService dans main.rs
- [ ] 8. Créer dossier uploads/ avec .gitignore
- [ ] 9. Écrire tests E2E
- [ ] 10. Créer composant Svelte FileUpload
- [ ] 11. Tester en local
- [ ] 12. Documentation
- [ ] 13. Commit : `feat: implement document upload and download`

---

**Créé le** : 2025-10-23
**Dépend de** : Aucune
**Bloque** : Issue #001 (joindre documents aux AG)
