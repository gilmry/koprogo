===========================================================================
Issue #210: docs: Create missing feature documentation (6 docs + 2 READMEs)
===========================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: documentation,priority:medium release:v0.5.0
:Assignees: Unassigned
:Created: 2026-02-26
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/210>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Seulement 8/30 features ont une documentation dédiée (27%). 22+ features ne sont documentées que dans CLAUDE.md.
   
   ## Documents à créer (~20h)
   
   ### Feature docs (15h)
   
   | Document | Feature(s) | Lignes est. | Heures |
   |----------|-----------|-------------|--------|
   | `docs/PAYMENT_INTEGRATION.rst` | Stripe, SEPA, remboursements, méthodes paiement | ~200 | 3h |
   | `docs/TICKETS_MAINTENANCE.rst` | Tickets, workflow, SLA, catégories, priorités | ~150 | 2h |
   | `docs/NOTIFICATIONS_SYSTEM.rst` | Multi-canal, préférences, 22 types, statistiques | ~150 | 2h |
   | `docs/CONVOCATIONS_AG.rst` | Délais légaux belges, tracking email, procuration | ~200 | 2h |
   | `docs/CONTRACTOR_QUOTES.rst` | Loi 3 devis, scoring, workflow, TVA belge | ~150 | 2h |
   | `docs/COMMUNITY_FEATURES.rst` | SEL, notices, skills, sharing, bookings, gamification | ~400 | 3h |
   
   ### READMEs (4h)
   
   | Document | Contenu | Heures |
   |----------|---------|--------|
   | `backend/README.md` | Architecture hexagonale, modules, how-to, tests | 2h |
   | `frontend/README.md` | Stack Astro+Svelte, structure, scripts, Islands pattern | 2h |
   
   ### Pattern à suivre
   
   Utiliser `docs/INVOICE_WORKFLOW.rst` ou `docs/PAYMENT_RECOVERY_WORKFLOW.rst` comme modèle :
   - Overview + contexte légal belge
   - Modèle domaine + diagramme d'états
   - Endpoints API
   - Configuration
   - Tests
   
   ### Transverse (3.5h)
   
   - [ ] Restructurer `CHANGELOG.md` : [Unreleased] → [0.5.0] (3h)
   - [ ] Mettre à jour `docs/index.rst` toctree (0.5h)
   
   Parent: #207

.. raw:: html

   </div>

