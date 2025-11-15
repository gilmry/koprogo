===============================================
RFC XXXX: [Titre Court de la Proposition]
===============================================

:RFC: XXXX
:Auteur: [Votre Nom] <email@example.com>
:Date: AAAA-MM-JJ
:Statut: Draft
:Type: [Feature / Architecture / Process]
:Équipes: [backend, frontend, infra, ia]
:Jalon: [1, 2, 3, 4, 5, 6]

.. contents:: Table des matières
   :depth: 3
   :local:

Résumé (TL;DR)
==============

[1-3 phrases décrivant la proposition de manière concise.]

**Exemple** :

   *Cette RFC propose d'intégrer le protocole MCP (Model Context Protocol) pour créer un réseau décentralisé de compute edge, permettant aux participants KoproGo de monétiser leurs ressources (Raspberry Pi, vieux laptops) en échange de revenus IA. Impact : 0 CO₂ cloud, revenus distribués (80% développement, 20% fonds solidarité).*

Métadonnées
===========

.. list-table::
   :widths: 30 70
   :header-rows: 0

   * - **RFC**
     - XXXX (numéro séquentiel)
   * - **Auteur**
     - [Nom Complet] <email@example.com>
   * - **Date création**
     - AAAA-MM-JJ
   * - **Statut**
     - Draft | Review | Accepted | Rejected | Implemented | Deprecated
   * - **Type**
     - Feature (nouvelle fonctionnalité) | Architecture (changement archi) | Process (processus organisationnel)
   * - **Équipes impliquées**
     - [backend, frontend, infra, ia] (sélectionner équipes concernées)
   * - **Jalon cible**
     - [1-6] (voir ROADMAP.md)
   * - **Priorité**
     - P0 (blocker) | P1 (high) | P2 (medium) | P3 (low)
   * - **Effort estimé**
     - [Sprints] (ex: 2 sprints, 1 mois, 3 mois)
   * - **Dépendances**
     - RFC-XXXX (si dépend autre RFC), ADR-XXXX (si dépend ADR)

Statuts RFC
-----------

.. list-table::
   :header-rows: 1
   :widths: 20 50 30

   * - Statut
     - Signification
     - Actions
   * - **Draft**
     - Rédaction en cours
     - Auteur rédige, peut demander feedback informel
   * - **Review**
     - Soumis pour revue communauté
     - Commentaires GitHub Discussions, review 7j min
   * - **Accepted**
     - Approuvé, prêt implémentation
     - Passe en Sprint Backlog, création issues GitHub
   * - **Rejected**
     - Refusé (justification obligatoire)
     - Archivé, raisons documentées
   * - **Implemented**
     - Implémenté et déployé production
     - Archivé, lien vers PR/release
   * - **Deprecated**
     - Obsolète (remplacé par autre RFC)
     - Archivé, lien RFC remplaçante

Contexte et Problème
====================

Problème adressé
----------------

[Décrire le problème métier ou technique que cette RFC résout.]

**Format recommandé** :

- Qui est affecté ? (utilisateurs, équipes, stakeholders)
- Quelle douleur ? (inefficacité, coût, risque)
- Impact si non résolu ? (quantifié si possible)

**Exemple** :

   **Qui** : Syndics et copropriétaires

   **Problème** : La génération des convocations d'assemblée générale (AG) est entièrement manuelle (Word, copier-coller adresses), prend 2-4 heures par AG, et génère des erreurs (mauvaise adresse, oubli copropriétaire).

   **Impact si non résolu** :

   - Temps perdu syndics : 4h × 4 AG/an = 16h/an/copropriété
   - Erreurs légales : 5% AG invalidées (convocation incomplète)
   - Frustration copropriétaires : Retards, mauvaises infos

Contexte organisationnel
-------------------------

[Expliquer le contexte KoproGo : jalon actuel, contraintes, opportunités.]

**Exemple** :

   - **Jalon actuel** : 3 (Impact, 1.000 copropriétés)
   - **Contrainte** : Budget serré (5€/mois/copropriété, marge 98%)
   - **Opportunité** : 1.000 copropriétés = 4.000 AG/an = 16.000h syndics perdues = ROI énorme
   - **Alignement roadmap** : Jalon 3 cible "Assemblées numériques complètes"

Pourquoi maintenant ?
---------------------

[Justifier le timing : pourquoi cette RFC maintenant, pas avant/après ?]

**Exemple** :

   - Jalon 2 (Viabilité) terminé → Comptabilité PCMN opérationnelle
   - 500 copropriétés demandent feature AG (top feedback Sprint Review)
   - Compétition : 2 concurrents lancent feature similaire (urgence stratégique)

Solution Proposée
=================

Vue d'ensemble
--------------

[Décrire la solution de manière concise, compréhensible par non-devs.]

**Exemple** :

   Implémenter un système de génération automatique de convocations AG :

   1. **Template éditable** : Syndic édite template Word/HTML (via éditeur WYSIWYG)
   2. **Variables dynamiques** : ``{{building.name}}``, ``{{meeting.date}}``, ``{{owner.name}}``
   3. **Génération PDF** : 1 clic → PDFs personnalisés (1 par copropriétaire)
   4. **Envoi email** : Automatique via SendGrid (tracking ouverture)
   5. **Archivage légal** : PDFs stockés MinIO (10 ans, RGPD-compliant)

Détails techniques
------------------

[Décrire implémentation technique détaillée (architecture, stack, patterns).]

**Format recommandé** :

- Architecture (diagrammes, composants)
- Stack technique (langages, frameworks, libraries)
- API endpoints (si applicable)
- Database schema (migrations SQL)
- Tests (stratégie, coverage cible)

**Exemple** :

.. code-block:: text

   Architecture:

   ┌─────────────────┐
   │  Frontend       │
   │  - Editor WYSIWYG (TinyMCE)
   │  - Preview PDF  │
   └────────┬────────┘
            │ POST /meetings/:id/generate-convocations
   ┌────────▼────────┐
   │  Backend        │
   │  - TemplateEngine (Handlebars)
   │  - PDF Generator (wkhtmltopdf)
   │  - Email Service (SendGrid)
   └────────┬────────┘
            │
   ┌────────▼────────┐
   │  Infrastructure │
   │  - MinIO (storage PDFs)
   │  - PostgreSQL (templates, logs)
   └─────────────────┘

   Stack:
   - Backend: Rust + handlebars-rust + wkhtmltopdf-rs
   - Frontend: Svelte + TinyMCE (WYSIWYG editor)
   - Storage: MinIO S3-compatible (chiffrement AES-256)
   - Email: SendGrid API (tracking + analytics)

   API Endpoints:
   - POST /api/v1/meetings/:id/templates (create/update template)
   - GET /api/v1/meetings/:id/templates (retrieve template)
   - POST /api/v1/meetings/:id/generate-convocations (generate PDFs)
   - GET /api/v1/meetings/:id/convocations (list generated PDFs)
   - POST /api/v1/meetings/:id/send-convocations (send emails)

   Database Schema (migration):
   ```sql
   CREATE TABLE meeting_templates (
       id UUID PRIMARY KEY,
       meeting_id UUID REFERENCES meetings(id),
       template_html TEXT NOT NULL,
       variables JSONB, -- {'building.name', 'meeting.date', ...}
       created_at TIMESTAMPTZ,
       updated_at TIMESTAMPTZ
   );

   CREATE TABLE convocations (
       id UUID PRIMARY KEY,
       meeting_id UUID REFERENCES meetings(id),
       owner_id UUID REFERENCES owners(id),
       pdf_path TEXT NOT NULL, -- MinIO path
       sent_at TIMESTAMPTZ,
       opened_at TIMESTAMPTZ, -- SendGrid webhook
       created_at TIMESTAMPTZ
   );
   ```

   Tests:
   - Unit: TemplateEngine rendering (100% coverage)
   - Integration: PDF generation + MinIO upload
   - E2E: Full workflow (edit template → generate → send → verify email)
   - BDD: Cucumber scenario "Generate convocation for AG"

Interfaces utilisateur
-----------------------

[Décrire UX/UI : wireframes, mockups, user flows.]

**Exemple** :

   **User Flow Syndic** :

   1. Page "Nouvelle AG" → Remplir date, heure, agenda
   2. Onglet "Convocation" → Éditeur WYSIWYG (template préchargé)
   3. Éditer template : Insérer variables (dropdown ``{{...}}``)
   4. Prévisualiser PDF (1 copropriétaire exemple)
   5. Clic "Générer convocations" → Loader 5s
   6. Liste PDFs générés (1 par copropriétaire, download individuel)
   7. Clic "Envoyer par email" → Confirmation modal → Envoi SendGrid
   8. Dashboard : Taux ouverture emails (temps réel)

   **Wireframe** : (insérer mockup Figma/Excalidraw)

Impact et Métriques
===================

Impact métier
-------------

[Quantifier impact pour utilisateurs, ASBL, et écologie.]

.. list-table::
   :header-rows: 1
   :widths: 30 40 30

   * - Métrique
     - Avant RFC
     - Après RFC
   * - **Temps génération convocations**
     - 4h manuelle
     - 10 min automatique
   * - **Erreurs convocations**
     - 5% AG (adresses, oublis)
     - 0% (automatique)
   * - **Coût syndic**
     - 4h × 50€/h = 200€/AG
     - 10 min × 50€/h = 8€/AG
   * - **Satisfaction copropriétaires**
     - 6/10 (retards, erreurs)
     - 9/10 (rapide, pro)

**ROI ASBL** :

- 1.000 copropriétés × 4 AG/an = 4.000 AG
- Économie : 192€/AG × 4.000 AG = **768.000€/an** (temps syndics)
- Coût développement : 2 sprints × 4 devs × 20h = 160h → **8.000€ équivalent**
- ROI : 768k€ / 8k€ = **96x** 🚀

Impact technique
----------------

[Quantifier impact performance, CO₂, maintenance.]

.. list-table::
   :header-rows: 1
   :widths: 30 40 30

   * - Métrique
     - Avant RFC
     - Après RFC
   * - **Performance P99**
     - N/A (manuel)
     - < 5s génération PDF
   * - **CO₂/convocation**
     - ~10g (Word, email manuel)
     - 0,5g (backend + SendGrid)
   * - **Storage**
     - 0 (rien archivé)
     - 2MB/AG × 4.000 AG = 8GB/an
   * - **Maintenance**
     - 0 (pas de code)
     - Faible (libs stables)

**Dette technique** :

- ➕ Ajoute dépendance wkhtmltopdf (binary externe)
- ➕ Ajoute complexité templates (Handlebars)
- ➖ Réduit dette UX (feature demandée top 3)

Impact roadmap
--------------

[Expliquer impact sur jalons, autres RFCs, dépendances.]

**Exemple** :

   - **Jalon 3** : Bloqueur (AG numériques = promesse jalon 3)
   - **RFC-0042** (Signatures électroniques) : Dépendance (convocation = 1ère étape, signature = 2ème)
   - **ADR-0015** (Document storage) : Impacté (MinIO déjà choisi, OK)

Alternatives Considérées
========================

Alternative 1 : [Nom Alternative]
----------------------------------

**Description** : [Décrire alternative]

**Avantages** :

- ➕ [Avantage 1]
- ➕ [Avantage 2]

**Inconvénients** :

- ➖ [Inconvénient 1]
- ➖ [Inconvénient 2]

**Décision** : ❌ Rejetée (justification)

**Exemple** :

   **Alternative 1 : Intégration Google Docs API**

   **Description** : Utiliser Google Docs pour templates, export PDF via Google API

   **Avantages** :

   - ➕ Éditeur Google Docs = familier utilisateurs
   - ➕ Pas de lib PDF à maintenir (Google gère)

   **Inconvénients** :

   - ➖ Dépendance externe (Google) = risque souveraineté données
   - ➖ Coût Google Workspace API (0,01$/requête × 4.000 AG = 40$/an)
   - ➖ Latence (appel externe, P99 > 500ms)
   - ➖ RGPD : Données passent par USA (non conforme)

   **Décision** : ❌ Rejetée (souveraineté + RGPD)

Alternative 2 : [Nom Alternative]
----------------------------------

[Même structure que Alternative 1]

Alternative 3 : Ne rien faire
------------------------------

**Description** : Garder processus manuel actuel

**Avantages** :

- ➕ 0 développement (0 coût)
- ➕ Pas de dette technique

**Inconvénients** :

- ➖ Perte compétitive (concurrents ont feature)
- ➖ Frustration utilisateurs (top demande)
- ➖ Temps perdu syndics (768k€/an)

**Décision** : ❌ Rejetée (ROI 96x trop élevé pour ignorer)

Risques et Mitigation
======================

Risques techniques
------------------

.. list-table::
   :header-rows: 1
   :widths: 30 30 30 10

   * - Risque
     - Impact
     - Mitigation
     - Probabilité
   * - wkhtmltopdf cassé (update OS)
     - Bloque génération PDF
     - Dockerize (version fixe), tests CI
     - Faible
   * - SendGrid quota dépassé
     - Emails non envoyés
     - Monitoring + alertes, upgrade plan auto
     - Moyenne
   * - MinIO storage full
     - Échec upload PDF
     - Monitoring disk, rotation auto (> 10 ans)
     - Faible
   * - Template XSS injection
     - Sécurité (script malveillant)
     - Sanitize HTML (DOMPurify), CSP headers
     - Moyenne

**Total risque** : Moyen (mitigations en place)

Risques métier
--------------

.. list-table::
   :header-rows: 1
   :widths: 30 30 30 10

   * - Risque
     - Impact
     - Mitigation
     - Probabilité
   * - Adoption faible (syndics préfèrent Word)
     - ROI non atteint
     - Onboarding vidéo, support chat
     - Faible
   * - Bugs légaux (convocation invalide)
     - AG annulée, risque juridique
     - Review legal expert, tests exhaustifs
     - Faible
   * - Performance lente (> 10s génération)
     - UX dégradée
     - Benchmarks CI, optimisation async
     - Moyenne

**Total risque** : Faible-Moyen

Plan de Rollback
----------------

[Décrire comment revenir en arrière si échec production.]

**Exemple** :

   1. Feature flag ``ENABLE_CONVOCATIONS`` (défaut: false)
   2. Déploiement progressif : 10% copropriétés → 50% → 100% (2 semaines)
   3. Monitoring Grafana : Taux erreur, P99 génération, taux ouverture emails
   4. **Trigger rollback** : Taux erreur > 5% OU P99 > 10s OU feedback négatif > 20%
   5. **Rollback** : Feature flag → false (1 min), redéploiement version N-1 (5 min)

Plan d'Implémentation
=====================

Décomposition tâches
--------------------

[Décomposer RFC en user stories / tasks, estimées.]

**Exemple** :

.. list-table::
   :header-rows: 1
   :widths: 10 50 15 15 10

   * - ID
     - User Story / Task
     - Équipe
     - Points
     - Sprint
   * - #201
     - Domain: MeetingTemplate entity
     - Backend
     - 3
     - Sprint 15
   * - #202
     - Application: GenerateConvocationsUseCase
     - Backend
     - 8
     - Sprint 15-16
   * - #203
     - Infrastructure: wkhtmltopdf wrapper
     - Backend
     - 5
     - Sprint 16
   * - #204
     - Infrastructure: MinIO upload service
     - Backend
     - 3
     - Sprint 16
   * - #205
     - Infrastructure: SendGrid email service
     - Backend
     - 5
     - Sprint 16
   * - #206
     - Frontend: WYSIWYG editor (TinyMCE)
     - Frontend
     - 8
     - Sprint 15-16
   * - #207
     - Frontend: Preview PDF modal
     - Frontend
     - 5
     - Sprint 16
   * - #208
     - Tests E2E: Full workflow
     - Backend + Frontend
     - 8
     - Sprint 17
   * - #209
     - Docs: User guide syndic
     - Docs
     - 2
     - Sprint 17
   * - #210
     - Infra: MinIO production setup
     - Infra
     - 3
     - Sprint 17

**Total** : 50 points ≈ 2,5 sprints (arrondir 3 sprints)

Jalons (Milestones)
-------------------

.. list-table::
   :header-rows: 1
   :widths: 15 40 25 20

   * - Jalon
     - Livrable
     - Date cible
     - Statut
   * - **M1**
     - Backend API génération PDF (mockup)
     - Sprint 15 (S2)
     - ⏳ Pending
   * - **M2**
     - Frontend WYSIWYG + preview
     - Sprint 16 (S2)
     - ⏳ Pending
   * - **M3**
     - Intégration complète + tests E2E
     - Sprint 17 (S1)
     - ⏳ Pending
   * - **M4**
     - Déploiement staging + beta test (10 copropriétés)
     - Sprint 17 (S2)
     - ⏳ Pending
   * - **M5**
     - Production (100% copropriétés)
     - Sprint 18 (S1)
     - ⏳ Pending

Dépendances
-----------

[Lister dépendances externes, RFCs, ADRs.]

**Exemple** :

- **ADR-0044** : Document Storage Strategy (MinIO) → ✅ Accepté
- **Infra** : SendGrid production account (quota 10k emails/mois) → ⏳ En cours setup
- **Design** : Mockups WYSIWYG editor → ⏳ Attente designer bénévole
- **Legal** : Review template légal convocation (avocat ASBL) → ❌ Pas démarré

Critères d'Acceptation
=======================

Critères fonctionnels
---------------------

[Critères acceptation métier, testables.]

**Format** : Given/When/Then (Gherkin-style)

**Exemple** :

1. **Template édition**

   - **Given** : Syndic logged in
   - **When** : Navigate to "Nouvelle AG" → Onglet "Convocation"
   - **Then** : WYSIWYG editor displayed avec template pré-rempli
   - **And** : Variables disponibles (dropdown ``{{...}}``)

2. **Génération PDFs**

   - **Given** : Template édité et sauvegardé
   - **When** : Clic "Générer convocations"
   - **Then** : PDFs générés (1 par copropriétaire actif)
   - **And** : PDFs stored MinIO (path ``/meetings/{id}/convocations/{owner_id}.pdf``)
   - **And** : Generation time < 5s (P99)

3. **Envoi emails**

   - **Given** : PDFs générés
   - **When** : Clic "Envoyer par email"
   - **Then** : Emails sent via SendGrid (tracking ID returned)
   - **And** : Dashboard shows delivery status (sent, opened, bounced)

Critères techniques
-------------------

[Critères acceptation techniques, mesurables.]

**Exemple** :

1. ✅ Tests unit coverage > 90% (domain + application)
2. ✅ Tests integration PostgreSQL + MinIO (testcontainers)
3. ✅ Tests E2E Playwright (full workflow)
4. ✅ Performance P99 < 5s (génération 50 PDFs)
5. ✅ Security audit OK (cargo audit, npm audit)
6. ✅ RGPD compliant (PDFs chiffrés AES-256, logs anonymisés)
7. ✅ Documentation Sphinx mise à jour (API, user guide)
8. ✅ Déployé staging (smoke tests OK)

Critères non-fonctionnels
--------------------------

[Critères performance, sécurité, UX, etc.]

**Exemple** :

1. **Performance** : P99 < 5s (génération 50 PDFs, backend)
2. **Scalabilité** : Support 1.000 copropriétés × 4 AG/an = 4.000 générations/an
3. **Disponibilité** : 99.9% uptime (monitoring Grafana)
4. **Sécurité** : Sanitize HTML templates (XSS protection), HTTPS only
5. **Accessibilité** : WYSIWYG editor WCAG 2.1 AA (keyboard navigation)
6. **UX** : Mobile-friendly (responsive < 768px)

Processus Revue RFC
===================

Soumission
----------

1. **Créer fichier RFC** : ``docs/governance/rfc/XXXX-titre-court.rst`` (copier template)
2. **Numéroter** : XXXX = numéro séquentiel (check dernière RFC, +1)
3. **Rédiger** : Compléter toutes sections (min 80% rempli)
4. **Commit** : Branch ``rfc/XXXX-titre-court``, commit, push
5. **Pull Request** : Ouvrir PR vers ``main``, tag ``rfc``, assigner reviewers

Revue communauté
----------------

1. **GitHub Discussions** : Créer discussion liée PR (commentaires asynchrones)
2. **Durée revue** : 7 jours min (permettre contributeurs distributed de répondre)
3. **Reviewers** : NIT (PO + SM + Tech Leads) + communauté (tous contributeurs)
4. **Critères approval** :

   - ✅ PO approve (alignement vision produit)
   - ✅ 2+ Tech Leads approve (faisabilité technique)
   - ✅ 0 objections majeures non résolues (communauté)

Décision
--------

**Accepted** :

- Statut → ``Accepted``
- Merge PR
- Créer issues GitHub (décomposition tâches)
- Ajouter Sprint Backlog (si prioritaire)

**Rejected** :

- Statut → ``Rejected``
- Ajouter section "Raisons rejet" (justification obligatoire)
- Merge PR (archiver, traçabilité)
- Close issues liées

Post-implémentation
-------------------

**Implemented** :

- Statut → ``Implemented``
- Ajouter liens PR, release notes, docs
- Retro : Lessons learned (ce qui a bien/mal marché)

**Deprecated** :

- Si RFC remplacée : Statut → ``Deprecated``, lien RFC remplaçante

Références
==========

- :doc:`/governance/togaf/adm` : TOGAF ADM (architecture d'entreprise)
- :doc:`/governance/nexus/framework` : Nexus (coordination équipes)
- :doc:`/governance/scrum/ceremonies` : Scrum local (sprints, DoD)
- :doc:`/ROADMAP_PAR_CAPACITES` : Roadmap par jalons

**RFCs liées** :

- RFC-XXXX : [Titre RFC dépendance]

**ADRs liées** :

- :doc:`/adr/0001-rust-actix-web-backend` : Choix stack backend

**Issues GitHub** :

- #XXX : [Issue GitHub liée]

Annexes
=======

Annexe A : Diagrammes
---------------------

[Insérer diagrammes architecture, séquence, etc.]

**Exemple** :

.. code-block:: text

   Séquence génération convocations:

   Syndic → Frontend : Clic "Générer"
   Frontend → Backend : POST /meetings/:id/generate-convocations
   Backend → PostgreSQL : Fetch meeting + owners
   Backend → Handlebars : Render template × N owners
   Backend → wkhtmltopdf : Generate PDFs × N
   Backend → MinIO : Upload PDFs
   Backend → PostgreSQL : Insert convocations records
   Backend → Frontend : 200 OK {pdf_urls: [...]}
   Frontend → Syndic : Display PDF list

Annexe B : Benchmarks
---------------------

[Insérer résultats benchmarks performance.]

**Exemple** :

.. code-block:: text

   Benchmark génération 50 PDFs (Criterion):

   test generate_50_pdfs ... bench: 3,245 ms/iter (+/- 234 ms)

   Breakdown:
   - Template rendering (Handlebars): 450 ms
   - PDF generation (wkhtmltopdf): 2,100 ms
   - MinIO upload: 695 ms

   P50: 3,1s
   P99: 4,8s ✅ (< 5s target)

Annexe C : Mockups
------------------

[Insérer mockups UI/UX, wireframes.]

**Exemple** :

   - Figma : https://figma.com/file/...
   - Excalidraw : (insérer image PNG)

---

**Instructions utilisation template** :

1. **Copier** ce fichier : ``cp template.rst XXXX-votre-titre.rst``
2. **Numéroter** : Remplacer ``XXXX`` par numéro séquentiel (ex: 0001, 0042)
3. **Remplir** : Compléter TOUTES sections (supprimer exemples)
4. **Commit** : Branch ``rfc/XXXX-titre``, PR vers ``main``
5. **Soumettre** : Tag ``rfc``, assigner reviewers, GitHub Discussions

**Critères RFC complète** :

- ✅ Métadonnées remplies (auteur, date, statut, équipes, jalon)
- ✅ Problème clairement défini (qui, quoi, impact)
- ✅ Solution détaillée (architecture, stack, API)
- ✅ Alternatives évaluées (min 2 alternatives + justification rejet)
- ✅ Impact quantifié (métriques avant/après, ROI)
- ✅ Risques identifiés + mitigation
- ✅ Plan implémentation (tâches, sprints, jalons)
- ✅ Critères acceptation (fonctionnels, techniques, non-fonctionnels)

---

*Template RFC KoproGo ASBL - Inspiré de IETF RFC, Rust RFC, Python PEP*
