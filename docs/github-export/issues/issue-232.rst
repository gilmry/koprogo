====================================================================================
Issue #232: R&D: API Publique v1 - Design OpenAPI et stratégie d'intégration tiers
====================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,priority:medium R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/232>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'issue #111 couvre l'API publique à long terme. Cette R&D se concentre
   sur le design de la v1 et les intégrations prioritaires.
   
   **Issue liée**: #111
   
   ## Objectifs de la R&D
   
   1. **Documentation OpenAPI/Swagger** :
      - Génération automatique depuis les handlers Actix-web
      - Bibliothèques : ``utoipa`` (Rust, dérivation automatique) vs. ``paperclip``
      - Versioning strategy (URL path ``/api/v1/`` vs. header)
      - Playground interactif (Swagger UI, Redoc)
   
   2. **Authentification API** :
      - API Keys (simple, par organisation)
      - OAuth2 (pour intégrations tierces, flow client_credentials)
      - Rate limiting par tier (free: 100/h, pro: 10k/h, enterprise: unlimited)
      - Webhook event system (EventBridge pattern)
   
   3. **Intégrations prioritaires** :
      - Logiciels comptables (Noalyss, BOB, Winbooks)
      - Banques (PSD2 : BNP, ING, KBC, Belfius)
      - Assurances (déclarations sinistre)
      - Énergie (Engie, TotalEnergies, Luminus)
      - Syndics professionnels (import/export données)
   
   4. **SDK génération** :
      - TypeScript/JavaScript (npm package)
      - Python (PyPI package)
      - PHP (Composer package)
      - Auto-génération via openapi-generator
   
   ## Points de décision
   
   - [ ] Bibliothèque OpenAPI Rust (utoipa vs. paperclip)
   - [ ] Modèle de pricing API (freemium vs. pay-per-call)
   - [ ] Webhook vs. polling pour événements
   - [ ] SLA API (uptime 99.9% vs. 99.95%)
   
   ## Estimation
   
   12-16h

.. raw:: html

   </div>

