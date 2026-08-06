==========================
Jalon 1: Sécurité & GDPR 🔒
==========================

:Number: 6
:State: open
:Due Date: No due date
:Open Issues: 3
:Closed Issues: 31
:Total Issues: 34
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

Issues (34)
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

✅ Issue #271: fix(legal): Quorum 50%+ validation AG (Art. 3.87 §5 CC)
---------------------------------------------------------------------------

:State: CLOSED
:Link: `#271 <../issues/issue-271.rst>`_

✅ Issue #272: fix(legal): Workflow 2e convocation si quorum non atteint (Art. 3.87 §5 CC)
-----------------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#272 <../issues/issue-272.rst>`_

✅ Issue #273: fix(legal): Réduction de vote mandataire (Art. 3.87 §7 CC) — limite procurations ✅ done
-----------------------------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#273 <../issues/issue-273.rst>`_

✅ Issue #301: [Bug] Permissions rôles : boutons admin visibles pour le syndic
-----------------------------------------------------------------------------------

:State: CLOSED
:Link: `#301 <../issues/issue-301.rst>`_

✅ Issue #302: [Bug] CRITIQUE : Isolation multi-tenant — données non filtrées par organization_id
------------------------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#302 <../issues/issue-302.rst>`_

✅ Issue #315: [RGPD] Art. 13-14 : Publier politique de confidentialité
----------------------------------------------------------------------------

:State: CLOSED
:Link: `#315 <../issues/issue-315.rst>`_

✅ Issue #316: [RGPD] Art. 28 : DPA avec sous-traitants (Stripe, AWS S3, email)
------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#316 <../issues/issue-316.rst>`_

✅ Issue #317: [RGPD] Art. 33 : Procédure notification violation de données (72h)
--------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#317 <../issues/issue-317.rst>`_

✅ Issue #326: feat(gdpr): Gestion du consentement utilisateur (GDPR Art. 7)
---------------------------------------------------------------------------------

:State: CLOSED
:Link: `#326 <../issues/issue-326.rst>`_

✅ Issue #327: feat(security): Gestion des incidents de sécurité (GDPR Art. 33 registre)
---------------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#327 <../issues/issue-327.rst>`_

✅ Issue #328: feat(security): Gestion des clés API (API Keys CRUD + hashing SHA-256)
------------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#328 <../issues/issue-328.rst>`_

✅ Issue #329: feat(gdpr): Registre des traitements GDPR Art. 30
---------------------------------------------------------------------

:State: CLOSED
:Link: `#329 <../issues/issue-329.rst>`_

🔵 Issue #331: test(playwright): 48 fichiers E2E Playwright frontend couvrant tous les modules
---------------------------------------------------------------------------------------------------

:State: OPEN
:Link: `#331 <../issues/issue-331.rst>`_

✅ Issue #337: fix: Consent handlers 100% stub — #326 fermée mais aucune persistance DB
--------------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#337 <../issues/issue-337.rst>`_

✅ Issue #340: fix: RBAC manquant sur 9 endpoints gamification (TODO: Check admin role)
--------------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#340 <../issues/issue-340.rst>`_

✅ Issue #351: docs: harmoniser documentation avec état réel du code (mars 2026)
-------------------------------------------------------------------------------------

:State: CLOSED
:Link: `#351 <../issues/issue-351.rst>`_

🔵 Issue #354: refactor(infra): Tests IaC manquants — terraform validate, ansible-lint, molecule, conftest ISO 27001
-------------------------------------------------------------------------------------------------------------------------

:State: OPEN
:Link: `#354 <../issues/issue-354.rst>`_

🔵 Issue #355: refactor(infra): Restructuration IaC — repo séparé, tests, policy-as-code
---------------------------------------------------------------------------------------------

:State: OPEN
:Link: `#355 <../issues/issue-355.rst>`_

