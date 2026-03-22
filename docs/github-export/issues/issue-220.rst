=========================================================================================
Issue #220: R&D: Implémentation RBAC - Choix architectural (Hybrid vs Dynamic vs Casbin)
=========================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,priority:high R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/220>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'étude RBAC (``docs/RBAC_STUDY.rst``) a identifié 3 approches possibles pour
   l'implémentation du système de permissions granulaires. Une décision architecturale
   est nécessaire avant l'implémentation.
   
   **Issues liées**: #71, #72
   
   ## Objectifs de la R&D
   
   1. **Évaluer les 3 alternatives** documentées dans RBAC_STUDY.rst :
      - **Alternative 1 (recommandée)** : Hybrid system (fixed roles + ``extra_permissions`` JSONB)
      - **Alternative 2** : Full dynamic permission matrix (15-20 jours)
      - **Alternative 3** : Casbin/Oso library integration (dépendance externe)
   
   2. **Prototyper** l'alternative recommandée avec les 8 rôles :
      - Existants : SuperAdmin, Syndic, Accountant, Owner
      - Nouveaux : Organization Admin, Building Manager, Tenant, Guest
   
   3. **Valider la compatibilité** avec le middleware Actix-web existant (``AuthenticatedUser``)
   
   ## Points de décision
   
   - [ ] Choix de l'approche (Hybrid vs Dynamic vs Casbin)
   - [ ] Migration strategy pour les rôles existants
   - [ ] Impact JWT claims (``extra_permissions`` field)
   - [ ] Mécanisme d'héritage des permissions entre rôles
   - [ ] Tests de performance (overhead middleware)
   
   ## Livrables attendus
   
   - ADR (Architecture Decision Record) en RST
   - Proof of Concept branch
   - Estimation d'effort pour l'implémentation complète
   
   ## Estimation
   
   8-12h de recherche + PoC

.. raw:: html

   </div>

