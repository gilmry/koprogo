=================================================================
Issue #223: R&D: Authentification forte eID/itsme® pour votes AG
=================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,priority:high security,legal-compliance R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/223>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'authentification forte est requise pour la validation légale des votes en AG
   (conformité eIDAS). L'issue #48 couvre l'implémentation.
   
   **Issue liée**: #48
   
   ## Objectifs de la R&D
   
   1. **itsme® Integration** :
      - API Contracts (OpenID Connect flow)
      - Niveaux d'identification (Identification, Authentication, Signing)
      - Coût par transaction (0.15-0.50€/identification)
      - Sandbox/test environment setup
      - Multi-language support (FR/NL/DE/EN)
   
   2. **eID belge** :
      - Middleware eID (browser extension/applet)
      - Lecture de certificat X.509
      - Compatibilité navigateurs (Chrome, Firefox, Safari)
      - eIDAS compliance levels (Low, Substantial, High)
   
   3. **Impact architecture** :
      - Intégration avec le JWT existant (claims supplémentaires)
      - Flow OAuth2/OIDC dans Actix-web
      - Fallback pour utilisateurs sans eID/itsme®
      - Stockage des preuves d'authentification (audit trail AG)
   
   ## Points de décision
   
   - [ ] itsme® vs. eID vs. les deux
   - [ ] Mandatory vs. optional pour les votes AG
   - [ ] Coût par transaction × nombre de copropriétaires attendus
   - [ ] Timing : quand rendre obligatoire (nombre minimum de copros)
   
   ## Livrables
   
   - Étude comparative itsme® vs. eID (coût, UX, compliance)
   - PoC OAuth2/OIDC avec sandbox itsme®
   - Estimation coût opérationnel annuel
   
   ## Estimation
   
   8-12h

.. raw:: html

   </div>

