===============================
Jalon 1: Sécurité & GDPR 🔒
===============================

:Number: 6
:State: open
:Due Date: No due date
:Open Issues: 7
:Closed Issues: 17
:Total Issues: 24
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

Issues (24)
========

✅ Issue #32: Rewrite E2E tests for unit_owner endpoints
--------------------------------------------------------------

:State: CLOSED
:Link: `#32 <../issues/issue-32.rst>`_

✅ Issue #39: feat(infra): Implement encryption at rest (LUKS) for VPS
----------------------------------------------------------------------------

:State: CLOSED
:Link: `#39 <../issues/issue-39.rst>`_

✅ Issue #40: feat(infra): Implement encrypted backups (GPG + S3 SSE)
---------------------------------------------------------------------------

:State: CLOSED
:Link: `#40 <../issues/issue-40.rst>`_

✅ Issue #41: feat(infra): Deploy monitoring stack (Prometheus + Grafana + Loki)
--------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#41 <../issues/issue-41.rst>`_

✅ Issue #42: feat: Implement GDPR data export & deletion (Right to be forgotten)
---------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#42 <../issues/issue-42.rst>`_

✅ Issue #43: feat(infra): Advanced security hardening (fail2ban, WAF, IDS)
---------------------------------------------------------------------------------

:State: CLOSED
:Link: `#43 <../issues/issue-43.rst>`_

✅ Issue #55: Automate MinIO/S3 bucket bootstrap
------------------------------------------------------

:State: CLOSED
:Link: `#55 <../issues/issue-55.rst>`_

✅ Issue #66: E2E: Admin login timeouts after user logout in GDPR tests
-----------------------------------------------------------------------------

:State: CLOSED
:Link: `#66 <../issues/issue-66.rst>`_

✅ Issue #69: Add Playwright E2E tests for unit management and document features
--------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#69 <../issues/issue-69.rst>`_

✅ Issue #78: feat: Security Hardening for Production (Rate limiting, 2FA, audit logs)
--------------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#78 <../issues/issue-78.rst>`_

✅ Issue #90: feat: GDPR Complementary Articles (16, 18, 21)
------------------------------------------------------------------

:State: CLOSED
:Link: `#90 <../issues/issue-90.rst>`_

✅ Issue #158: E2E tests have 200+ compilation errors after API changes
----------------------------------------------------------------------------

:State: CLOSED
:Link: `#158 <../issues/issue-158.rst>`_

✅ Issue #207: Release 0.5.0 - Test Pyramid & Documentation Umbrella
-------------------------------------------------------------------------

:State: CLOSED
:Link: `#207 <../issues/issue-207.rst>`_

✅ Issue #208: feat(tests): BDD step definitions for 24 new feature files (279 scenarios)
----------------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#208 <../issues/issue-208.rst>`_

✅ Issue #209: feat(tests): Playwright expansion - 7 new frontend E2E spec files
-------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#209 <../issues/issue-209.rst>`_

✅ Issue #210: docs: Create missing feature documentation (6 docs + 2 READMEs)
-----------------------------------------------------------------------------------

:State: CLOSED
:Link: `#210 <../issues/issue-210.rst>`_

🔵 Issue #271: fix(legal): Quorum 50%+ validation AG (Art. 3.87 §5 CC)
----------------------------------------------------------------------------

:State: OPEN
:Link: `#271 <../issues/issue-271.rst>`_

🔵 Issue #272: fix(legal): Workflow 2e convocation si quorum non atteint (Art. 3.87 §5 CC)
------------------------------------------------------------------------------------------------

:State: OPEN
:Link: `#272 <../issues/issue-272.rst>`_

✅ Issue #273: fix(legal): Réduction de vote mandataire (Art. 3.87 §7 CC) — limite procurations ✅ done
-----------------------------------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#273 <../issues/issue-273.rst>`_

🔵 Issue #301: [Bug] Permissions rôles : boutons admin visibles pour le syndic
------------------------------------------------------------------------------------

:State: OPEN
:Link: `#301 <../issues/issue-301.rst>`_

🔵 Issue #302: [Bug] CRITIQUE : Isolation multi-tenant — données non filtrées par organization_id
----------------------------------------------------------------------------------------------------------

:State: OPEN
:Link: `#302 <../issues/issue-302.rst>`_

🔵 Issue #315: [RGPD] Art. 13-14 : Publier politique de confidentialité
-----------------------------------------------------------------------------

:State: OPEN
:Link: `#315 <../issues/issue-315.rst>`_

🔵 Issue #316: [RGPD] Art. 28 : DPA avec sous-traitants (Stripe, AWS S3, email)
------------------------------------------------------------------------------------

:State: OPEN
:Link: `#316 <../issues/issue-316.rst>`_

🔵 Issue #317: [RGPD] Art. 33 : Procédure notification violation de données (72h)
----------------------------------------------------------------------------------------

:State: OPEN
:Link: `#317 <../issues/issue-317.rst>`_

