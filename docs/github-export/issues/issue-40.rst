===================================================================
Issue #40: feat(infra): Implement encrypted backups (GPG + S3 SSE)
===================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: phase:vps,track:infrastructure priority:critical
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-17
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/40>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Current backup system (`infrastructure/ansible/templates/backup.sh.j2`) creates daily PostgreSQL dumps but does **not encrypt them**. For GDPR compliance and security best practices, backups containing sensitive data must be encrypted at rest.
   
   ## Objective
   
   Implement encrypted backup strategy with:
   1. GPG encryption for PostgreSQL dumps
   2. Secure off-site storage (S3 with SSE-KMS or SSE-S3)
   3. Key management procedures
   4. Restore testing automation
   
   ## Current Implementation
   
   **Existing backup script** (Ansible-deployed):
   - Daily cron job at 2am UTC
   - PostgreSQL dumps to local filesystem
   - No encryption
   - No off-site storage
   
   **File:** `infrastructure/ansible/templates/backup.sh.j2`
   
   ## Implementation Plan
   
   ### 1. GPG Encryption Layer
   
   **Setup GPG key:**
   ```bash
   # Generate dedicated backup encryption key
   gpg --full-generate-key
   # Export public key for backup operations
   gpg --export --armor backup@koprogo.local > backup-public-key.asc
   # Export private key for restore (store securely off-site!)
   gpg --export-secret-keys --armor backup@koprogo.local > backup-private-key.asc
   ```
   
   **Update backup script:**
   ```bash
   #!/bin/bash
   # PostgreSQL dump with compression + GPG encryption
   docker exec postgres pg_dump -U koprogo koprogo_db | \
     gzip -9 | \
     gpg --encrypt --recipient backup@koprogo.local \
     > /backups/koprogo_$(date +%Y%m%d_%H%M%S).sql.gz.gpg
   ```
   
   ### 2. S3 Off-Site Storage
   
   **Provider options:**
   - AWS S3 with SSE-KMS
   - Scaleway Object Storage
   - Backblaze B2
   - OVH Object Storage (same provider as VPS)
   
   **Recommended: OVH Object Storage** (integrated with Terraform)
   
   **Setup S3 sync:**
   ```bash
   # Install s3cmd or aws-cli
   apt install s3cmd
   
   # Configure S3 credentials (Ansible template)
   s3cmd --configure
   
   # Sync encrypted backups
   s3cmd put /backups/*.gpg s3://koprogo-backups/ --server-side-encryption
   ```
   
   ### 3. Ansible Integration
   
   **Update tasks:**
   ```yaml
   - name: Install GPG and s3cmd
     apt:
       name:
         - gnupg
         - s3cmd
       state: present
   
   - name: Generate GPG backup key
     command: gpg --batch --gen-key gpg-key-params
     args:
       creates: /root/.gnupg/pubring.kbx
   
   - name: Configure s3cmd
     template:
       src: s3cfg.j2
       dest: /root/.s3cfg
       mode: '0600'
   
   - name: Deploy encrypted backup script
     template:
       src: backup-encrypted.sh.j2
       dest: /usr/local/bin/backup-koprogo.sh
       mode: '0700'
   ```
   
   **Files to create:**
   - `infrastructure/ansible/templates/backup-encrypted.sh.j2`
   - `infrastructure/ansible/templates/s3cfg.j2`
   - `infrastructure/ansible/templates/gpg-key-params.j2`
   
   ### 4. Backup Retention Policy
   
   **Strategy:**
   - Daily backups: Keep last 7 days
   - Weekly backups: Keep last 4 weeks
   - Monthly backups: Keep last 12 months
   - Yearly backups: Keep indefinitely
   
   **S3 lifecycle rules:**
   ```bash
   # Configure lifecycle via S3 bucket policy
   # Delete objects older than retention period
   ```
   
   ### 5. Restore Testing
   
   **Automated restore test (monthly cron):**
   ```bash
   #!/bin/bash
   # Download latest backup from S3
   s3cmd get s3://koprogo-backups/latest.sql.gz.gpg /tmp/restore-test.gpg
   
   # Decrypt backup
   gpg --decrypt /tmp/restore-test.gpg | gunzip > /tmp/restore.sql
   
   # Restore to test database
   docker exec postgres psql -U koprogo -d koprogo_test < /tmp/restore.sql
   
   # Verify row counts match
   # Send alert if restore fails
   ```
   
   ### 6. Key Management
   
   **Security practices:**
   - GPG private key stored in password manager (1Password, Bitwarden)
   - S3 credentials in Ansible vault
   - Key rotation every 12 months
   - Multiple recovery key holders (2-of-3 threshold)
   
   **Disaster recovery:**
   - Document key recovery procedures
   - Store encrypted private key copy in separate cloud provider
   - Test key recovery quarterly
   
   ## Terraform Integration
   
   **Provision S3 bucket:**
   ```hcl
   resource "ovh_cloud_project_object_storage_bucket" "backups" {
     service_name = var.service_name
     region       = "GRA"
     name         = "koprogo-backups"
     
     lifecycle_rule {
       enabled = true
       expiration {
         days = 365
       }
     }
   }
   ```
   
   **File:** `infrastructure/terraform/s3-backups.tf` (new)
   
   ## Testing & Validation
   
   - [ ] GPG encryption works (test encrypt/decrypt)
   - [ ] S3 upload successful
   - [ ] Backup size acceptable (compression ratio > 10x)
   - [ ] Restore from encrypted backup succeeds
   - [ ] Automated restore test passes
   - [ ] Performance impact negligible (<5 min total backup time)
   - [ ] Disaster recovery procedure documented and tested
   
   ## Security Considerations
   
   - **Encryption strength**: GPG with RSA 4096-bit or Ed25519
   - **S3 bucket security**: Private, versioning enabled, MFA delete
   - **Access logs**: Enable S3 access logging for audit trail
   - **Key compromise**: Procedure to rotate keys if compromised
   
   ## Monitoring & Alerts
   
   - [ ] Alert if backup fails
   - [ ] Alert if S3 sync fails
   - [ ] Alert if restore test fails
   - [ ] Dashboard showing backup status, size, retention
   
   **Integration:** Prometheus alertmanager + Grafana
   
   ## Documentation
   
   - [ ] Update `infrastructure/README.md` with backup procedures
   - [ ] Document GPG key generation and management
   - [ ] Create disaster recovery playbook
   - [ ] Document S3 bucket configuration
   - [ ] Update CLAUDE.md with backup strategy
   
   ## Acceptance Criteria
   
   - [ ] Daily backups are GPG-encrypted
   - [ ] Encrypted backups uploaded to S3 with SSE
   - [ ] Retention policy enforced (7d/4w/12m/∞y)
   - [ ] Restore from encrypted backup tested successfully
   - [ ] Automated monthly restore test cron job active
   - [ ] GPG private key securely stored off-site
   - [ ] Documentation complete
   - [ ] Alerts configured for backup failures
   
   ## Cost Estimate
   
   **S3 Storage (OVH):**
   - 10 GB backups/month × €0.011/GB = €0.11/month
   - Negligible cost
   
   ## Effort Estimate
   
   **Small** (1 day)
   
   ## Related
   
   - Depends on: Issue #39 (encryption at rest)
   - Supports: GDPR compliance
   - Related: Monitoring stack issue (backup alerts)
   
   ## References
   
   - GPG best practices: https://riseup.net/en/security/message-security/openpgp/best-practices
   - OVH Object Storage: https://www.ovhcloud.com/en/public-cloud/object-storage/
   - PostgreSQL backup: https://www.postgresql.org/docs/current/backup.html

.. raw:: html

   </div>

