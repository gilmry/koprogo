====================================================================
Issue #39: feat(infra): Implement encryption at rest (LUKS) for VPS
====================================================================

:State: **OPEN**
:Milestone: Phase 1: VPS MVP + Legal Compliance
:Labels: phase:vps,track:infrastructure priority:critical
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-08
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/39>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Production VPS deployment requires encryption at rest for GDPR compliance and data security. Sensitive data (owner information, financial records, meeting minutes) must be protected.
   
   ## Objective
   
   Implement LUKS full-disk encryption for VPS data volumes, particularly for PostgreSQL data and document storage.
   
   ## Implementation Plan
   
   ### 1. LUKS Setup for Data Volumes
   
   **VPS Volume Configuration:**
   - Create encrypted volume for PostgreSQL data (`/var/lib/docker/volumes/postgres_data`)
   - Create encrypted volume for document uploads (`/var/lib/docker/volumes/backend_uploads`)
   - Configure LUKS passphrase management (avoid manual unlock on reboot)
   
   **Steps:**
   ```bash
   # Create encrypted volumes
   cryptsetup luksFormat /dev/vdb
   cryptsetup luksOpen /dev/vdb postgres_encrypted
   
   # Format and mount
   mkfs.ext4 /dev/mapper/postgres_encrypted
   mount /dev/mapper/postgres_encrypted /var/lib/docker/volumes/postgres_data
   ```
   
   ### 2. Key Management Strategy
   
   **Options:**
   - **Option A (Recommended)**: TPM 2.0 integration if available on VPS
   - **Option B**: Network-based unlock (Tang server)
   - **Option C**: Manual unlock on reboot (requires intervention)
   
   **Chosen approach**: Document in implementation
   
   ### 3. Ansible Integration
   
   **Update playbook:**
   - Add LUKS encryption tasks before Docker volume setup
   - Configure automatic unlock mechanism
   - Add fstab entries for encrypted volumes
   - Test encryption with Ansible dry-run
   
   **File:** `infrastructure/ansible/playbook.yml`
   
   ### 4. Terraform Updates
   
   **If additional volumes needed:**
   - Provision separate data volume via Terraform
   - Update OVH OpenStack configuration
   - Document volume attachment
   
   **File:** `infrastructure/terraform/main.tf`
   
   ### 5. Docker Compose Adjustments
   
   **Ensure volumes use encrypted paths:**
   ```yaml
   volumes:
     postgres_data:
       driver: local
       driver_opts:
         type: none
         o: bind
         device: /var/lib/docker/volumes/postgres_data/_data
   ```
   
   ### 6. Testing & Validation
   
   - [ ] Verify encryption with `cryptsetup status`
   - [ ] Test database read/write performance impact (<10% acceptable)
   - [ ] Test VPS reboot with auto-unlock
   - [ ] Verify Docker Compose services restart correctly
   - [ ] Document unlock procedure for disaster recovery
   
   ## Security Considerations
   
   - **Passphrase strength**: Use strong passphrase (min 20 chars)
   - **Key backup**: Store LUKS header backup securely off-site
   - **Access control**: Limit LUKS key access to root only
   - **Audit**: Log encryption/decryption events
   
   ## Performance Impact
   
   - Expected: 3-7% performance overhead (acceptable for compliance)
   - Mitigation: Modern CPUs have AES-NI acceleration
   - Benchmarks: Test P99 latency before/after encryption
   
   ## Documentation
   
   - [ ] Update `infrastructure/README.md` with encryption setup
   - [ ] Document key management procedures
   - [ ] Create disaster recovery playbook for encrypted volumes
   - [ ] Update CLAUDE.md with security posture
   
   ## Acceptance Criteria
   
   - [ ] PostgreSQL data volume is LUKS-encrypted
   - [ ] Document uploads volume is LUKS-encrypted
   - [ ] VPS can reboot and auto-unlock volumes (no manual intervention)
   - [ ] Docker Compose services start correctly after reboot
   - [ ] Performance degradation < 10%
   - [ ] Documentation complete
   - [ ] Disaster recovery procedure tested
   
   ## Related
   
   - Depends on: Encrypted backups issue (backup encrypted data)
   - Supports: GDPR compliance
   - Security audit requirement
   
   ## Effort Estimate
   
   **Medium** (1-2 days)
   
   ## References
   
   - LUKS documentation: https://gitlab.com/cryptsetup/cryptsetup
   - OVH VPS storage: https://help.ovh.com/csm/en-gb-public-cloud-compute-volume-overview

.. raw:: html

   </div>

