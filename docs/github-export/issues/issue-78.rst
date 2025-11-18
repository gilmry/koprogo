====================================================================================
Issue #78: feat: Security Hardening for Production (Rate limiting, 2FA, audit logs)
====================================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: enhancement,phase:vps track:software,priority:critical security
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2025-11-17
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/78>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #005 - Security Hardening for Production
   
   **Priorité**: 🔴 CRITIQUE  
   **Estimation**: 10-12 heures  
   **Phase**: VPS MVP (Nov 2025 - Mar 2026)  
   
   ## 📋 Description
   
   Renforcement de la sécurité pour la mise en production : rate limiting, JWT refresh tokens, CORS strict, structured logging, 2FA optionnel.
   
   ## 🎯 Objectifs
   
   - [ ] Rate limiting par IP et par utilisateur
   - [ ] JWT refresh token rotation
   - [ ] CORS strict (whitelist domains)
   - [ ] Structured logging (JSON format)
   - [ ] 2FA optionnel (TOTP)
   - [ ] Audit logs pour actions sensibles
   - [ ] Headers sécurité (HSTS, CSP, etc.)
   
   ## 📐 Features
   
   ### 1. Rate Limiting
   - 100 req/min par IP (endpoints publics)
   - 1000 req/min authentifié
   - 5 tentatives login/15min
   
   ### 2. JWT Refresh Tokens
   - Access token: 15 min
   - Refresh token: 7 jours
   - Rotation automatique
   
   ### 3. Audit Logs
   - Login/Logout
   - Modifications données sensibles
   - Suppressions
   - Export GDPR
   
   ## ✅ Critères d'Acceptation
   
   - Rate limiting actif avec Redis
   - Refresh tokens fonctionnels
   - Audit logs stockés PostgreSQL
   - Tests E2E sécurité
   - CORS configuré production
   
   ---
   
   **Voir**: \`issues/critical/005-security-hardening.md\`

.. raw:: html

   </div>

