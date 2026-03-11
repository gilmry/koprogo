================================================================================
Issue #44: feat: Implement document storage strategy (Local volume vs S3/MinIO)
================================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: phase:vps,track:infrastructure priority:high
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-13
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/44>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   KoproGo needs a production-ready document storage solution for:
   - Meeting minutes (PDF)
   - Financial statements
   - Invoices
   - Contracts
   - Work quotes
   - Owner documents
   
   **Current implementation:**
   - Backend FileStorage service uses local filesystem (`backend/src/infrastructure/storage/file_storage.rs`)
   - Files stored in `/uploads/building-id/filename`
   - Max file size: 50MB
   - Works in development but needs production strategy
   
   ## Problem
   
   **Local filesystem limitations:**
   - Single point of failure (VPS disk)
   - Backup complexity (need to backup file volumes separately)
   - Scaling limitations (disk space on single VPS)
   - No built-in redundancy
   - Docker volume management overhead
   
   **Production requirements:**
   - Reliable storage with backups
   - Encrypted at rest
   - Accessible from multiple instances (future K3s/K8s)
   - Cost-effective for MVP phase
   
   ## Options Analysis
   
   ### Option 1: Local Docker Volume (Simple)
   
   **Architecture:**
   ```yaml
   # docker-compose.yml
   volumes:
     backend_uploads:
       driver: local
       driver_opts:
         type: none
         o: bind
         device: /opt/koprogo/uploads  # Mounted on encrypted LUKS volume
   ```
   
   **Pros:**
   - ✅ Simple setup (no external service)
   - ✅ Low cost (included in VPS)
   - ✅ Fast access (local disk)
   - ✅ No egress costs
   - ✅ GDPR compliant (data stays in EU)
   
   **Cons:**
   - ❌ Single point of failure
   - ❌ Manual backup required (via cron + GPG + S3)
   - ❌ Limited by VPS disk size (40GB for d2-2)
   - ❌ Migration complexity when scaling to K3s
   
   **Cost:** €0/month (included in VPS)
   
   ---
   
   ### Option 2: MinIO (Self-Hosted S3-Compatible)
   
   **Architecture:**
   ```yaml
   # docker-compose.yml
   services:
     minio:
       image: minio/minio:latest
       command: server /data --console-address ":9001"
       volumes:
         - minio_data:/data
       environment:
         MINIO_ROOT_USER: ${MINIO_ROOT_USER}
         MINIO_ROOT_PASSWORD: ${MINIO_ROOT_PASSWORD}
       ports:
         - "9000:9000"
         - "9001:9001"  # Console
       
   volumes:
     minio_data:
       driver: local
   ```
   
   **Backend integration:**
   ```rust
   // Use aws-sdk-s3 crate with MinIO endpoint
   let config = aws_config::load_from_env().await;
   let client = aws_sdk_s3::Client::new(&config);
   ```
   
   **Pros:**
   - ✅ S3-compatible API (easy migration to cloud S3 later)
   - ✅ Built-in versioning
   - ✅ Web console for management
   - ✅ Erasure coding for redundancy (if multi-disk)
   - ✅ Encryption at rest support
   - ✅ Stays on VPS (no external dependency)
   
   **Cons:**
   - ❌ Additional resource overhead (~200MB RAM)
   - ❌ Still single-server (no HA on single VPS)
   - ❌ Complexity vs local filesystem
   - ❌ Still needs backup strategy
   
   **Cost:** €0/month (self-hosted on VPS)
   **RAM impact:** ~200MB (10% of 2GB VPS)
   
   ---
   
   ### Option 3: External S3 (Scaleway/OVH Object Storage)
   
   **Backend integration:**
   ```rust
   // Use aws-sdk-s3 with Scaleway/OVH endpoint
   let config = Config::builder()
       .endpoint_url("https://s3.fr-par.scw.cloud")
       .build();
   let client = aws_sdk_s3::Client::from_conf(config);
   ```
   
   **Providers:**
   
   | Provider | Storage Cost | Transfer Out | Requests |
   |----------|-------------|--------------|----------|
   | **Scaleway** | €0.01/GB/month | €0.01/GB | Free (first 75GB) |
   | **OVH** | €0.011/GB/month | Free | €0.0001/10k |
   | **Backblaze B2** | €0.005/GB/month | €0.01/GB (first 3x free) | Free |
   
   **Example cost (100GB storage, 10GB transfer/month):**
   - Scaleway: €1.10/month
   - OVH: €1.10/month
   - Backblaze B2: €0.50/month
   
   **Pros:**
   - ✅ High availability (99.9% SLA)
   - ✅ Automatic backups/replication
   - ✅ Unlimited scalability
   - ✅ Offloads VPS resources
   - ✅ Built-in encryption at rest
   - ✅ Easy K3s/K8s integration (same S3 backend)
   - ✅ GDPR compliant (EU regions available)
   
   **Cons:**
   - ❌ External dependency (network required)
   - ❌ Monthly cost (starts low, grows with usage)
   - ❌ Egress costs for downloads
   - ❌ Slightly higher latency vs local disk
   
   **Cost:** ~€1-2/month initially (MVP scale)
   
   ---
   
   ## Recommendation: **Hybrid Approach** (Best of Both Worlds)
   
   ### Phase 1 (MVP - Q4 2025): MinIO on VPS
   - Self-hosted MinIO container
   - S3-compatible API from day 1
   - Backup to external S3 (Backblaze B2 cheapest)
   - Encryption at rest via LUKS volume
   
   ### Phase 2 (Production - Q1 2026): Migrate to External S3
   - Switch to Scaleway/OVH Object Storage
   - Same S3 API (minimal code changes)
   - Better HA and redundancy
   - Offload VPS resources
   
   **Migration path:**
   ```bash
   # Sync MinIO to external S3
   mc mirror minio/koprogo-documents s3/koprogo-documents
   # Update backend env vars (S3 endpoint + credentials)
   # No code changes needed (same AWS S3 SDK)
   ```
   
   ---
   
   ## Implementation Plan
   
   ### 1. Backend Storage Abstraction
   
   **Create trait:** `backend/src/application/ports/object_storage.rs`
   
   ```rust
   #[async_trait]
   pub trait ObjectStorage: Send + Sync {
       async fn upload(&self, key: &str, data: Vec<u8>) -> Result<String, String>;
       async fn download(&self, key: &str) -> Result<Vec<u8>, String>;
       async fn delete(&self, key: &str) -> Result<(), String>;
       async fn exists(&self, key: &str) -> Result<bool, String>;
       async fn list(&self, prefix: &str) -> Result<Vec<String>, String>;
   }
   ```
   
   **Implementations:**
   
   `backend/src/infrastructure/storage/local_storage.rs` (existing):
   ```rust
   pub struct LocalStorage {
       base_path: PathBuf,
   }
   
   impl ObjectStorage for LocalStorage {
       async fn upload(&self, key: &str, data: Vec<u8>) -> Result<String, String> {
           let path = self.base_path.join(key);
           tokio::fs::write(path, data).await.map_err(|e| e.to_string())?;
           Ok(key.to_string())
       }
       // ...
   }
   ```
   
   `backend/src/infrastructure/storage/s3_storage.rs` (new):
   ```rust
   use aws_sdk_s3::Client;
   
   pub struct S3Storage {
       client: Client,
       bucket: String,
   }
   
   impl ObjectStorage for S3Storage {
       async fn upload(&self, key: &str, data: Vec<u8>) -> Result<String, String> {
           self.client
               .put_object()
               .bucket(&self.bucket)
               .key(key)
               .body(data.into())
               .send()
               .await
               .map_err(|e| e.to_string())?;
           Ok(key.to_string())
       }
       // ...
   }
   ```
   
   ### 2. Configuration (Environment Variables)
   
   `backend/.env`:
   ```bash
   # Storage backend: "local" or "s3"
   STORAGE_BACKEND=s3
   
   # S3 Configuration (MinIO or external S3)
   S3_ENDPOINT=http://minio:9000  # MinIO local
   # S3_ENDPOINT=https://s3.fr-par.scw.cloud  # Scaleway
   S3_BUCKET=koprogo-documents
   S3_REGION=fr-par
   S3_ACCESS_KEY=minioadmin
   S3_SECRET_KEY=minioadmin
   S3_USE_PATH_STYLE=true  # Required for MinIO
   
   # Local storage fallback
   LOCAL_STORAGE_PATH=/uploads
   ```
   
   ### 3. Docker Compose - MinIO
   
   **Add to:** `deploy/production/docker-compose.yml`
   
   ```yaml
   services:
     minio:
       image: minio/minio:RELEASE.2024-10-13T13-34-11Z
       command: server /data --console-address ":9001"
       volumes:
         - minio_data:/data
       environment:
         MINIO_ROOT_USER: ${MINIO_ROOT_USER}
         MINIO_ROOT_PASSWORD: ${MINIO_ROOT_PASSWORD}
         MINIO_SERVER_URL: http://minio:9000
       networks:
         - koprogo-network
       restart: unless-stopped
       healthcheck:
         test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
         interval: 30s
         timeout: 10s
         retries: 3
       labels:
         - "traefik.enable=true"
         - "traefik.http.routers.minio-console.rule=Host(`minio.${DOMAIN}`)"
         - "traefik.http.services.minio-console.loadbalancer.server.port=9001"
   
     # MinIO client for initial bucket creation
     minio-init:
       image: minio/mc:latest
       depends_on:
         - minio
       entrypoint: >
         /bin/sh -c "
         mc alias set minio http://minio:9000 ${MINIO_ROOT_USER} ${MINIO_ROOT_PASSWORD};
         mc mb minio/koprogo-documents --ignore-existing;
         mc anonymous set download minio/koprogo-documents;
         exit 0;
         "
       networks:
         - koprogo-network
   
   volumes:
     minio_data:
       driver: local
   ```
   
   ### 4. Backend Cargo Dependencies
   
   `backend/Cargo.toml`:
   ```toml
   [dependencies]
   aws-config = "1.1.0"
   aws-sdk-s3 = "1.10.0"
   ```
   
   ### 5. Dependency Injection
   
   `backend/src/main.rs`:
   ```rust
   let storage: Arc<dyn ObjectStorage> = match env::var("STORAGE_BACKEND").as_deref() {
       Ok("s3") => {
           let config = aws_config::load_from_env().await;
           let s3_client = aws_sdk_s3::Client::new(&config);
           Arc::new(S3Storage::new(s3_client, env::var("S3_BUCKET").unwrap()))
       }
       _ => {
           Arc::new(LocalStorage::new(env::var("LOCAL_STORAGE_PATH").unwrap()))
       }
   };
   ```
   
   ### 6. Backup Strategy (MinIO → External S3)
   
   **Add to backup script:**
   
   `infrastructure/ansible/templates/backup-minio.sh.j2`:
   ```bash
   #!/bin/bash
   # Backup MinIO to Backblaze B2
   
   mc alias set minio http://localhost:9000 ${MINIO_ROOT_USER} ${MINIO_ROOT_PASSWORD}
   mc alias set b2 https://s3.us-west-000.backblazeb2.com ${B2_KEY_ID} ${B2_APPLICATION_KEY}
   
   # Mirror MinIO to B2
   mc mirror --overwrite minio/koprogo-documents b2/koprogo-backups/documents
   
   # Encrypt and upload metadata
   mc ls minio/koprogo-documents --json | gzip | gpg --encrypt > /backups/minio-metadata-$(date +%Y%m%d).json.gz.gpg
   ```
   
   Cron: Daily at 3am UTC
   
   ### 7. Monitoring
   
   **Prometheus metrics for MinIO:**
   ```yaml
   scrape_configs:
     - job_name: 'minio'
       metrics_path: /minio/v2/metrics/cluster
       static_configs:
         - targets: ['minio:9000']
   ```
   
   **Grafana dashboard:** MinIO Overview (ID: 13502)
   
   ---
   
   ## Testing & Validation
   
   - [ ] LocalStorage implementation works (existing)
   - [ ] S3Storage implementation works (MinIO)
   - [ ] Upload/download via MinIO successful
   - [ ] Switch between local and S3 via env var
   - [ ] MinIO console accessible (minio.domain.com)
   - [ ] Backup script syncs MinIO → B2
   - [ ] Encryption at rest (LUKS volume under MinIO)
   - [ ] Performance acceptable (<100ms upload for 1MB file)
   - [ ] Migration script tested (MinIO → external S3)
   
   ## Security
   
   - [ ] MinIO credentials strong (min 20 chars)
   - [ ] MinIO console behind Traefik auth
   - [ ] S3 bucket not publicly readable (except specific files)
   - [ ] Encryption at rest (LUKS volume)
   - [ ] Backup encryption (GPG)
   
   ## Documentation
   
   - [ ] Update CLAUDE.md with storage architecture
   - [ ] Document MinIO setup and configuration
   - [ ] Create migration guide (local → MinIO → external S3)
   - [ ] Document backup and restore procedures
   
   ## Acceptance Criteria
   
   - [ ] ObjectStorage trait abstraction implemented
   - [ ] LocalStorage implementation complete
   - [ ] S3Storage implementation complete (MinIO compatible)
   - [ ] MinIO container in docker-compose
   - [ ] Environment-based storage backend selection
   - [ ] MinIO backup to external S3 (Backblaze B2)
   - [ ] Monitoring integrated (Prometheus + Grafana)
   - [ ] Documentation complete
   - [ ] Migration path validated
   
   ## Cost Analysis (MVP Phase)
   
   **Option 1 (Local Only):**
   - Storage: €0 (VPS disk)
   - Backup: €0.50/month (Backblaze B2, 100GB)
   - **Total: €0.50/month**
   
   **Option 2 (MinIO + B2 Backup):**
   - MinIO: €0 (self-hosted)
   - Backup: €0.50/month (Backblaze B2, 100GB)
   - RAM overhead: 200MB (10% of VPS)
   - **Total: €0.50/month + RAM overhead**
   
   **Option 3 (External S3 Only):**
   - Scaleway S3: €1.10/month (100GB + 10GB transfer)
   - No backup needed (built-in redundancy)
   - **Total: €1.10/month**
   
   **Recommendation for MVP:** **Option 2 (MinIO + B2)** - Best migration path
   
   ## Effort Estimate
   
   **Medium** (2 days)
   - Day 1: ObjectStorage trait + S3Storage implementation + MinIO docker-compose
   - Day 2: Backup script + monitoring + testing + documentation
   
   ## Related
   
   - Depends on: Issue #39 (encryption at rest - LUKS volume for MinIO)
   - Depends on: Issue #40 (encrypted backups - B2 integration)
   - Enables: File upload UI issue (needs storage backend)
   - Future: Migrate to external S3 in Q1 2026
   
   ## References
   
   - MinIO: https://min.io/docs/minio/linux/index.html
   - AWS S3 SDK Rust: https://docs.rs/aws-sdk-s3/
   - Backblaze B2: https://www.backblaze.com/b2/cloud-storage.html
   - Scaleway Object Storage: https://www.scaleway.com/en/object-storage/

.. raw:: html

   </div>

