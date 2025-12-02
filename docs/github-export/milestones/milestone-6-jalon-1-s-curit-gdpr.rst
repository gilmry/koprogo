==========================
Jalon 1: Sécurité & GDPR 🔒
==========================

:Number: 6
:State: open
:Due Date: No due date
:Open Issues: 9
:Closed Issues: 2
:Total Issues: 11
:URL: `View on GitHub <https://github.com/gilmry/koprogo/milestone/6>`_

Description
===========

**Débloque**: 50-100 copropriétés (beta publique possible)

**Issues critiques**: #39 (LUKS), #40 (Backups GPG), #42 (GDPR Art 15/17), #48 (Auth forte itsme®), #43 (Hardening)

**Livrables**:
🔐 Données chiffrées au repos (LUKS)
💾 Backups quotidiens automatisés (GPG + S3)
📜 Conformité GDPR Articles 15 & 17 (export + effacement)
🔑 Authentification multi-facteur (itsme®)
🛡️ Security hardening (fail2ban, WAF, IDS)

**Conformité légale**: 40%

**Conditions de déblocage**: Tous les tests sécurité + GDPR passent

**Effort estimé**: Solo dev (10-20h/sem) = 2-3 mois | Duo (40-60h/sem) = 6-8 semaines

Issues (11)
========

✅ Issue #32: Rewrite E2E tests for unit_owner endpoints
--------------------------------------------------------------

:State: CLOSED
:Link: `#32 <../issues/issue-32.rst>`_

🔵 Issue #39: feat(infra): Implement encryption at rest (LUKS) for VPS
----------------------------------------------------------------------------

:State: OPEN
:Link: `#39 <../issues/issue-39.rst>`_

🔵 Issue #40: feat(infra): Implement encrypted backups (GPG + S3 SSE)
---------------------------------------------------------------------------

:State: OPEN
:Link: `#40 <../issues/issue-40.rst>`_

🔵 Issue #41: feat(infra): Deploy monitoring stack (Prometheus + Grafana + Loki)
--------------------------------------------------------------------------------------

:State: OPEN
:Link: `#41 <../issues/issue-41.rst>`_

✅ Issue #42: feat: Implement GDPR data export & deletion (Right to be forgotten)
---------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#42 <../issues/issue-42.rst>`_

🔵 Issue #43: feat(infra): Advanced security hardening (fail2ban, WAF, IDS)
---------------------------------------------------------------------------------

:State: OPEN
:Link: `#43 <../issues/issue-43.rst>`_

🔵 Issue #48: feat: Implement strong authentication for voting (itsme, eID)
---------------------------------------------------------------------------------

:State: OPEN
:Link: `#48 <../issues/issue-48.rst>`_

🔵 Issue #55: Automate MinIO/S3 bucket bootstrap
------------------------------------------------------

:State: OPEN
:Link: `#55 <../issues/issue-55.rst>`_

🔵 Issue #66: E2E: Admin login timeouts after user logout in GDPR tests
-----------------------------------------------------------------------------

:State: OPEN
:Link: `#66 <../issues/issue-66.rst>`_

🔵 Issue #69: Add Playwright E2E tests for unit management and document features
--------------------------------------------------------------------------------------

:State: OPEN
:Link: `#69 <../issues/issue-69.rst>`_

🔵 Issue #78: feat: Security Hardening for Production (Rate limiting, 2FA, audit logs)
--------------------------------------------------------------------------------------------

:State: OPEN
:Link: `#78 <../issues/issue-78.rst>`_

