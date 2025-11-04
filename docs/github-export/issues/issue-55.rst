==============================================
Issue #55: Automate MinIO/S3 bucket bootstrap
==============================================

:State: **OPEN**
:Milestone: Phase 1: VPS MVP + Legal Compliance
:Labels: None
:Assignees: Unassigned
:Created: 2025-10-29
:Updated: 2025-11-01
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/55>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Summary
   Currently, the configuration supports both MinIO (with bootstrap container) and external S3 buckets by toggling . However, when switching to a managed S3 service, operators must manually create/configure the bucket and policies.
   
   ## Proposal
   - Provide a documented helper (Ansible role or standalone script) that can initialize the target S3 bucket (create bucket if absent, enforce private policy, optional lifecycle).
   - Ensure we avoid storing secrets in logs and allow operators to opt-in safely.
   
   ## Priority
   Low – capture the idea for later; current deployment flow works with manual steps.
   
   ],

.. raw:: html

   </div>

