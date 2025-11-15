===============================================
Cérémonies Scrum Locales - KoproGo
===============================================

:Auteur: KoproGo ASBL
:Date: 2025-01-15
:Version: 1.0
:Statut: Actif

.. contents:: Table des matières
   :depth: 3
   :local:

Introduction
============

Ce document décrit les **cérémonies Scrum locales** pratiquées par chaque équipe KoproGo (Backend, Frontend, Infra, IA/Grid).

Les cérémonies Scrum sont complétées par les **événements Nexus** (voir :doc:`/governance/nexus/framework`) pour la coordination cross-équipes.

Principes Scrum KoproGo
=======================

Valeurs Scrum
-------------

KoproGo applique les 5 valeurs Scrum :

1. **Commitment** : Équipes s'engagent sur Sprint Goal
2. **Focus** : Concentration sur Sprint Backlog (WIP limit)
3. **Openness** : Transparence (code, docs, décisions)
4. **Respect** : Code reviews bienveillantes, mentoring
5. **Courage** : Feedback honnête, remise en question

Adaptations ASBL
----------------

**Différences vs Scrum corporate** :

.. list-table::
   :header-rows: 1
   :widths: 30 35 35

   * - Aspect
     - Scrum Standard
     - KoproGo ASBL
   * - **Équipe**
     - Salariés full-time
     - Contributeurs bénévoles + salariés (futur)
   * - **Disponibilité**
     - 40h/semaine
     - Variable (5-20h/semaine)
   * - **Engagement**
     - Contrat de travail
     - Contribution volontaire
   * - **Rôles**
     - Fixes (PO, SM, Dev)
     - Rotatifs (SM, Tech Lead)
   * - **Réunions**
     - Synchrones (bureau)
     - Asynchrones-first (distributed)

**Conséquences** :

- Sprint planning flexible (participation asynchrone OK)
- Daily Scrum écrit (Slack/GitHub) si pas dispo sync
- Retro toujours synchrone (cohésion équipe)

Sprints KoproGo
===============

Durée et Cadence
----------------

**Durée sprint** : 2 semaines (10 jours ouvrés)

**Début** : Lundi (Sprint Planning)

**Fin** : Vendredi soir (Sprint Review + Retro)

**Calendrier type Sprint N** :

.. code-block:: text

   Semaine 1:
   - Lundi:    Sprint Planning (Nexus 4h, local 2h)
   - Mardi-Vendredi: Daily Scrum (15 min/jour)

   Semaine 2:
   - Lundi-Jeudi: Daily Scrum (15 min/jour)
   - Jeudi:   Backlog Refinement (1h, mid-sprint)
   - Vendredi: Sprint Review (Nexus 2h) + Retro (Nexus 1h30)

**Pas de sprint "0"** : Dès le premier sprint, livrer increment déployable.

Sprint Goal
-----------

**Définition** : Objectif court (1 phrase) du sprint, aligné avec Nexus Sprint Goal.

**Critères bon Sprint Goal** :

1. **Concret** : Décrit une fonctionnalité utilisateur (pas "refactoring")
2. **Testable** : Critères acceptation clairs
3. **Atteignable** : Réaliste pour 2 semaines
4. **Aligné Nexus** : Contribue au Nexus Sprint Goal

**Exemples** :

.. code-block:: text

   ❌ Mauvais Sprint Goal:
   "Améliorer le code backend"
   → Vague, non testable

   ✅ Bon Sprint Goal (Backend):
   "Implémenter workflow approbation factures (Draft → PendingApproval → Approved)"
   → Concret, testable (tests E2E), aligné Nexus

   ✅ Bon Sprint Goal (Frontend):
   "UI responsive mobile pour liste factures + approbation (< 768px)"
   → Concret, testable (Playwright mobile), aligné Nexus

**Affichage** : GitHub Projects (custom field "Sprint Goal")

Cérémonies Scrum Locales
=========================

1. Sprint Planning (Local)
---------------------------

**Objectif** : Planifier travail équipe pour le sprint

**Quand** : Lundi matin (APRÈS Nexus Sprint Planning)

**Durée** : 2h

**Participants** : Équipe complète (3-5 devs + SM + optionnel PO)

**Ordre du jour** :

**Part 1 : Quoi (1h)** - Sélection items backlog

1. SM présente Nexus Sprint Goal + dépendances identifiées
2. Équipe sélectionne items top backlog (GitHub Projects)
3. Vérification critères "Ready" (voir :doc:`/governance/nexus/framework`)
4. Équipe formule Sprint Goal local (aligné Nexus)

**Part 2 : Comment (1h)** - Décomposition tâches

1. Équipe décompose chaque item en tâches techniques (< 1 jour)
2. Attribution tâches volontaire (pas imposée par SM !)
3. Identification risques techniques (POCs nécessaires ?)
4. Commit équipe sur Sprint Goal

**Outputs** :

- Sprint Backlog (items + tâches GitHub Issues)
- Sprint Goal affiché (GitHub Projects)
- Vélocité cible (basée sur moyenne 3 derniers sprints)

**Exemple Sprint Planning Backend (Sprint 12)** :

.. code-block:: text

   Nexus Sprint Goal:
   "Workflow factures avec approbation + monitoring production"

   Sprint Goal Backend:
   "API workflow factures (submit, approve, reject) + tests E2E + metrics Prometheus"

   Items sélectionnés (28 points):
   - #125: POST /expenses/:id/submit-for-approval (8 pts)
   - #126: POST /expenses/:id/approve (5 pts)
   - #127: POST /expenses/:id/reject (5 pts)
   - #128: Tests E2E workflow complet (5 pts)
   - #129: Prometheus metrics (expense_approvals_total) (5 pts)

   Tâches #125 (exemple décomposition):
   - [ ] Domain: Expense.submit_for_approval() (2h)
   - [ ] Application: ExpenseUseCases.submit_for_approval() (3h)
   - [ ] Infrastructure: POST handler (2h)
   - [ ] Tests: Unit + integration (4h)
   - [ ] Migration DB: add approved_by_id column (1h)

   Risques:
   - ⚠️ Email notification conseil syndical (dépend Infra SendGrid)
   - ⚠️ Idempotence (submit multiple fois ?)

   Attribution:
   - Alice: #125, #128 (lead)
   - Bob: #126, #127
   - Charlie: #129 (Prometheus metrics)

2. Daily Scrum (Local)
-----------------------

**Objectif** : Synchroniser équipe, détecter blocages

**Quand** : Chaque jour ouvré, même heure (ex: 10h)

**Durée** : 15 min MAX (timebox strict)

**Participants** : Équipe dev uniquement (PO/SM optionnels observateurs)

**Format** : Debout (stand-up) ou écrit (Slack si async)

**3 questions par dev** :

1. **Hier** : Qu'ai-je terminé vers le Sprint Goal ?
2. **Aujourd'hui** : Que vais-je faire vers le Sprint Goal ?
3. **Blocages** : Qu'est-ce qui m'empêche d'avancer ?

**Exemple Daily Scrum Backend (écrit, Slack)** :

.. code-block:: text

   Alice:
   - Hier: ✅ Domain Expense.submit_for_approval() + tests unit
   - Aujourd'hui: Application layer ExpenseUseCases
   - Blocages: Aucun

   Bob:
   - Hier: ✅ POST /expenses/:id/approve handler
   - Aujourd'hui: Tests intégration PostgreSQL
   - Blocages: ⚠️ Testcontainers lent (5 min startup)

   Charlie:
   - Hier: 🔨 Prometheus metrics (WIP)
   - Aujourd'hui: Finaliser metrics + tests
   - Blocages: ⚠️ Pas clair comment tester metrics (mock Prometheus ?)

   Actions:
   - Bob: Optimiser testcontainers (reuse containers between tests)
   - Charlie: Pair avec Alice (elle a déjà testé metrics dans autre projet)

**Règles Daily Scrum** :

- ❌ PAS une réunion de status pour le SM/PO
- ❌ PAS de résolution problèmes (parking lot → après Daily)
- ✅ Focus Sprint Goal (pas "hier j'ai fait review PR #42")
- ✅ Auto-organisation (équipe décide actions, pas SM)

**Daily Scrum asynchrone** (si contributeurs timezone différentes) :

- Poster daily update dans Slack channel #backend-daily avant 10h
- SM résume + partage blocages dans Nexus Daily Scrum
- Résolution blocages : GitHub Issues (commentaires asynchrones)

3. Sprint Review (Local)
-------------------------

**Objectif** : Inspecter Increment, adapter backlog

**Quand** : Vendredi après-midi (AVANT Nexus Sprint Review)

**Durée** : 1h

**Participants** : Équipe + PO + stakeholders invités (optionnel)

**Ordre du jour** :

**1. Rappel Sprint Goal (5 min)**

SM rappelle Sprint Goal et vélocité cible vs réelle.

**2. Démo Increment (30 min)**

Équipe démontre items **Done** (respectant DoD).

**Format démo** :

- Live coding (pas slides !)
- Environnement staging (pas local)
- Données réalistes (seed script)
- Tests automatisés projetés (démo TDD)

**Exemple démo Backend Sprint 12** :

.. code-block:: text

   Démo API workflow factures:

   1. cURL POST /expenses (création facture Draft)
   2. cURL POST /expenses/:id/submit-for-approval
      → Retour 200, state = PendingApproval
      → Email SendGrid envoyé (montré dans logs)
   3. cURL POST /expenses/:id/approve
      → Retour 200, state = Approved
   4. Tests E2E Cucumber (projeté terminal):
      ✅ Scenario: Submit expense for approval
      ✅ Scenario: Approve expense
      ✅ Scenario: Reject expense
   5. Grafana dashboard Prometheus:
      → Métrique expense_approvals_total = 1

**3. Feedback stakeholders (15 min)**

PO/stakeholders posent questions, suggèrent améliorations.

**4. Mise à jour backlog (10 min)**

PO ajuste backlog selon feedback (nouveaux items, re-priorisation).

**Outputs** :

- Increment déployé staging
- Vélocité réelle (points Done)
- Backlog ajusté

**Note** : Sprint Review local ≠ Nexus Sprint Review (cross-équipes)

4. Sprint Retrospective (Local)
--------------------------------

**Objectif** : Améliorer processus équipe

**Quand** : Vendredi après-midi (AVANT Nexus Sprint Retrospective)

**Durée** : 1h

**Participants** : Équipe dev + SM (PO optionnel si invité)

**Règle d'or** : Safe space (confidentialité, bienveillance, pas de blâme)

**Format** : Varie chaque sprint (exemples ci-dessous)

**Format 1 : Start/Stop/Continue**

Équipe brainstorm (10 min silencieux, post-its) :

- **Start** : Quoi commencer à faire ?
- **Stop** : Quoi arrêter de faire ?
- **Continue** : Quoi continuer (fonctionne bien) ?

Vote dot (5 dots/personne) → Top 3 actions

**Exemple Retro Backend Sprint 12** :

.. code-block:: text

   START:
   - Pair programming pour features complexes (8 votes) ✅
   - Benchmarks systématiques (Criterion) (3 votes)

   STOP:
   - Meetings après 17h (trop tard, fatigue) (7 votes) ✅
   - PRs > 500 lignes (trop longues à reviewer) (6 votes) ✅

   CONTINUE:
   - Tests BDD Cucumber (super lisibles) (5 votes)
   - Async daily Slack (timezone-friendly) (4 votes)

   Top 3 actions Sprint 13:
   1. ✅ Pair programming 2h/semaine (Alice + Bob, Charlie + nouveau)
   2. ✅ Meetings max 16h30 (sauf urgence)
   3. ✅ PRs max 300 lignes (split si plus gros)

**Format 2 : Timeline**

Dessiner timeline sprint (15 jours), équipe ajoute émotions (😀😐😞) + événements clés.

**Exemple** :

.. code-block:: text

   Jour 1 (Lundi): 😀 Sprint planning clair
   Jour 3 (Mercredi): 😞 Testcontainers cassé (CI rouge)
   Jour 5 (Vendredi): 😐 PR review lente (48h attente)
   Jour 8 (Lundi S2): 😀 Testcontainers fixé (Bob hero!)
   Jour 10 (Mercredi S2): 😀 Feature terminée early
   Jour 12 (Vendredi S2): 😐 Stress demo (peur bugs)

   Discussion: Pourquoi stress demo ?
   → Pas de tests staging avant
   → Action: Déployer staging J-1 demo (jeudi soir)

**Format 3 : Glad/Sad/Mad**

- **Glad** : Content de quoi ?
- **Sad** : Déçu de quoi ?
- **Mad** : Frustré par quoi ?

**Outputs Retro** :

- 2-3 actions concrètes Sprint N+1 (SMART: Specific, Measurable, Atteignable)
- Actions trackées GitHub Issues (label ``retro-action``)
- Review actions précédentes (Done ? Si non, pourquoi ?)

**Facilitation Retro** :

- SM facilite (neutre, pas défensif)
- Timeboxing strict (1h max)
- Parking lot (sujets hors scope → GitHub Discussions)
- Rotation facilitateur (équipe mature → dev peut faciliter)

5. Backlog Refinement (Local)
------------------------------

**Objectif** : Préparer items futurs sprints

**Quand** : Mid-sprint (jeudi semaine 2, avant Nexus Refinement)

**Durée** : 1h

**Participants** : Équipe dev + PO

**Activités** :

1. **Affiner top 10 items backlog** (détails, critères acceptation)
2. **Décomposer epics** en user stories (< 13 points)
3. **Estimer** (Planning Poker, Fibonacci)
4. **Identifier dépendances** techniques

**Exemple Refinement Backend** :

.. code-block:: text

   Item: "Implémenter relances automatisées paiement"

   Questions équipe:
   - Quels niveaux relance ? (PO: 4 niveaux Gentle/Formal/Final/Legal)
   - Fréquence envoi ? (PO: Configurable par syndic, défaut J+15/30/45/60)
   - Email template where ? (PO: DB, éditable syndic)

   Décomposition:
   - #150: Domain PaymentReminder entity (3 pts)
   - #151: Cron job daily check overdue (5 pts)
   - #152: Email template rendering (3 pts)
   - #153: Tests E2E workflow relances (5 pts)

   Total: 16 points (OK pour 1 sprint)

   Dépendances:
   - Cron job → Infra team (tokio-cron-scheduler)
   - Email templates → Design (mockups needed)

   Ready ? ❌ Non (attendre mockups design)
   Action: PO demande mockups, re-refine Sprint 14

**Règles Refinement** :

- Max 10% capacité sprint (1h/sprint 2 semaines = OK)
- Pas d'engagement (estimation ≠ commitment)
- PO décide priorité finale (équipe conseille faisabilité)

Rôles Scrum
===========

Product Owner (PO)
------------------

**Responsabilités** :

1. Définir vision produit (alignée TOGAF ADM)
2. Maintenir Product Backlog (priorisation)
3. Rédiger user stories (critères acceptation)
4. Accepter/rejeter Increment (Sprint Review)
5. Disponible pour équipe (questions, clarifications)

**Chez KoproGo** :

- **PO unique** pour tout KoproGo (cross-équipes)
- Membre CA ASBL (légitimité gouvernance)
- Disponibilité : 20h/semaine minimum
- Rotation possible (vote CA, mandat 1 an)

**Profil PO idéal** :

- Connaissance métier copropriétés (syndic, comptable, ou juriste)
- Compétences techniques (comprendre faisabilité)
- Capacité décision (pas consensus permanent)

Scrum Master (SM)
-----------------

**Responsabilités** :

1. Faciliter cérémonies Scrum
2. Résoudre blocages équipe
3. Protéger équipe (interruptions, scope creep)
4. Coaching Scrum (valeurs, pratiques)
5. Amélioration continue (retros, metrics)

**Chez KoproGo** :

- **SM par équipe** (4 SMs : backend, frontend, infra, IA)
- Contributeur senior (lead dev OK si double casquette)
- Rotation conseillée (6 mois → mentoring)

**Profil SM idéal** :

- Certification Scrum (PSM I minimum, recommandé)
- Expérience dev (crédibilité technique)
- Soft skills (écoute, empathie, facilitation)

Development Team
----------------

**Responsabilités** :

1. Livrer Increment Done chaque sprint
2. Auto-organisation (qui fait quoi)
3. Qualité code (tests, reviews, refactoring)
4. Respect DoD (non négociable)
5. Collaboration (pair programming, mentoring)

**Taille équipe** : 3-5 devs (idéal Scrum)

**Compétences cross-fonctionnelles** :

- Chaque dev = T-shaped (expertise 1 domaine + connaissances larges)
- Exemple Backend : Expert Rust + notions PostgreSQL + notions DevOps

**Chez KoproGo** :

- Contributeurs bénévoles (majorité, phase ASBL)
- Salariés part-time/full-time (future, phase Coopérative)
- Onboarding : Pair 1 semaine avec senior (code + culture)

Definition of Done (DoD)
========================

DoD Équipe (complète DoD Nexus)
--------------------------------

Voir :doc:`/governance/nexus/framework` pour DoD Nexus (cross-équipes).

**DoD Backend** (ajoute à Nexus) :

1. ✅ Architecture hexagonale respectée (Domain → Application → Infrastructure)
2. ✅ Migrations DB testées (rollback + rollforward)
3. ✅ Benchmarks Criterion (si endpoint perf-critical)
4. ✅ Docs Rust (cargo doc comments)

**DoD Frontend** (ajoute à Nexus) :

1. ✅ Responsive mobile (Playwright tests < 768px)
2. ✅ Accessibilité WCAG 2.1 AA (axe-core scans)
3. ✅ PWA offline (ServiceWorker cache)
4. ✅ Storybook components (future)

**DoD Infra** (ajoute à Nexus) :

1. ✅ Terraform plan OK (dry-run, no surprises)
2. ✅ Ansible idempotent (run 2x = same result)
3. ✅ Runbook SRE (incident response doc)
4. ✅ Rollback plan (max 5 min downtime)

**DoD IA/Grid** (ajoute à Nexus) :

1. ✅ Tests edge ARM64 (Raspberry Pi)
2. ✅ Modèle quantized (< 2GB RAM)
3. ✅ Benchmarks inference (< 100ms P99)
4. ✅ Revenus compute trackés (blockchain future)

Métriques Scrum Locales
========================

Vélocité Équipe
---------------

**Définition** : Story points Done par sprint

**Calcul** : Somme points items **Done** (respectant DoD)

**Objectif** : Stabilité (± 10% sprint N vs N-1)

**Exemple Backend Sprints 10-12** :

.. code-block:: text

   Sprint 10: 20 points
   Sprint 11: 18 points
   Sprint 12: 22 points
   Moyenne: 20 points ± 2 (stable ✅)

**Usage** :

- Sprint Planning : Sélectionner ~20 points (basé moyenne)
- PO : Prévoir roadmap (ex: Feature 60 pts = 3 sprints)

**Anti-pattern** :

- ❌ Vélocité = KPI performance équipe (pression → inflation points)
- ✅ Vélocité = outil planification (prévisibilité)

Burndown Chart
--------------

**Définition** : Graphique story points restants vs jours sprint

**Axes** :

- X : Jours sprint (0 à 10)
- Y : Story points restants

**Ligne idéale** : Diagonale (progression linéaire)

**Exemple Sprint 12 Backend** :

.. code-block:: text

   Jour 0:  28 points (Sprint Planning)
   Jour 2:  28 points (pas Done encore)
   Jour 4:  20 points (item #125 Done, 8 pts)
   Jour 6:  15 points (item #126 Done, 5 pts)
   Jour 8:  10 points (item #127 Done, 5 pts)
   Jour 10:  0 points (tous items Done ✅)

**Alarmes** :

- Courbe flat (pas de progrès) → Daily focus blocages
- Courbe monte (scope creep) → PO protège Sprint Goal

Cycle Time
----------

**Définition** : Temps entre "In Progress" et "Done" (1 item)

**Objectif** : < 3 jours (idéal 1-2 jours)

**Exemple** :

.. code-block:: text

   Item #125:
   - Lundi 10h: In Progress
   - Mercredi 16h: Done
   → Cycle time: 2,25 jours ✅

**Alarmes** :

- Cycle time > 5 jours → Item trop gros (décomposer)
- Cycle time > 7 jours → Blocage (SM investigate)

Code Review Time
----------------

**Définition** : Temps entre "PR created" et "PR merged"

**Objectif** : < 24h (1 working day)

**Règles KoproGo** :

- PR < 300 lignes : 2h review max
- PR 300-500 lignes : 4h review max
- PR > 500 lignes : ❌ Refus (split required)

**Exemple** :

.. code-block:: text

   PR #234:
   - Mardi 14h: Created (250 lignes)
   - Mardi 16h: Review Alice (approve)
   - Mardi 17h: Review Bob (request changes)
   - Mercredi 10h: Changes pushed
   - Mercredi 11h: Bob approve
   - Mercredi 11h30: Merged
   → Review time: 21h30 ✅

**Alarmes** :

- Review time > 48h → Reviewer overload (assign 2nd reviewer)

Outils Scrum
============

GitHub Projects
---------------

**Board Kanban** : https://github.com/users/gilmry/projects

**Colonnes** :

1. **Backlog** : Items priorités (PO), triés top → bottom
2. **Sprint N** : Items sprint en cours
3. **In Progress** : WIP (limit 2 items/dev)
4. **Review** : Attente code review
5. **Done** : Merged + déployé staging

**Automation GitHub** :

- Issue created → Backlog
- PR opened → Review
- PR merged → Done (auto)

**Custom Fields** :

- Sprint Goal (texte)
- Story Points (nombre Fibonacci)
- Team (select: backend/frontend/infra/ia)
- Priority (select: P0/P1/P2/P3)

Estimation : Planning Poker
----------------------------

**Échelle Fibonacci** : 1, 2, 3, 5, 8, 13, 21

**Signification** :

- **1 pt** : Trivial (< 2h, ex: typo fix)
- **2 pts** : Simple (< 4h, ex: add DB column)
- **3 pts** : Moyen (1 jour, ex: CRUD endpoint)
- **5 pts** : Complexe (2 jours, ex: workflow feature)
- **8 pts** : Très complexe (3-4 jours, ex: auth system)
- **13 pts** : Epic (1 semaine, **décomposer !**)
- **21+ pts** : ❌ Trop gros (obligatoire split)

**Processus Planning Poker** :

1. PO lit user story + critères acceptation
2. Équipe pose questions (5 min max)
3. Chaque dev vote simultanément (cartes/app)
4. Si consensus (± 1 pt) → Estimation validée
5. Si divergence (ex: 2 vs 8) → Discuss extremes, re-vote

**Exemple** :

.. code-block:: text

   Item: "Implémenter POST /expenses/:id/approve"

   Vote 1:
   - Alice: 3 pts
   - Bob: 8 pts
   - Charlie: 5 pts

   Discussion:
   - Bob: "Il faut workflow state machine + email notification"
   - Alice: "Email déjà fait Infra, juste call API"
   - Charlie: "State machine = pattern Strategy, 1 jour"

   Vote 2:
   - Alice: 5 pts
   - Bob: 5 pts
   - Charlie: 5 pts

   ✅ Consensus: 5 points

Tests & Qualité
===============

Stratégie Pyramid
-----------------

KoproGo applique **Test Pyramid** :

.. code-block:: text

                  E2E (10%)
                 /        \
               /            \
             /   Integration  \
           /       (30%)        \
         /__________________________\
                Unit (60%)

**Répartition temps tests** :

- **60% Unit** : Domain logic, use cases (fast, isolés)
- **30% Integration** : Repositories PostgreSQL (testcontainers)
- **10% E2E** : Full workflow API → UI (Playwright, Cucumber)

**Exemple Backend Sprint** :

.. code-block:: text

   Sprint 12 (5 jours dev):
   - 3 jours: Feature code + unit tests (60%)
   - 1,5 jours: Integration tests PostgreSQL (30%)
   - 0,5 jours: E2E tests Cucumber (10%)

TDD (Test-Driven Development)
------------------------------

**Pratique recommandée** (pas obligatoire) :

1. **Red** : Écrire test qui fail
2. **Green** : Code minimal pour passer test
3. **Refactor** : Améliorer code (tests stay green)

**Exemple TDD Backend** :

.. code-block:: rust

   // 1. RED: Test fail (fonction n'existe pas)
   #[test]
   fn test_submit_expense_for_approval() {
       let expense = Expense::new_draft(...);
       let result = expense.submit_for_approval();
       assert!(result.is_ok());
       assert_eq!(expense.state, ExpenseState::PendingApproval);
   }

   // 2. GREEN: Implémentation minimale
   impl Expense {
       pub fn submit_for_approval(&mut self) -> Result<(), String> {
           self.state = ExpenseState::PendingApproval;
           Ok(())
       }
   }

   // 3. REFACTOR: Ajouter validations
   impl Expense {
       pub fn submit_for_approval(&mut self) -> Result<(), String> {
           if self.state != ExpenseState::Draft {
               return Err("Can only submit Draft expenses".to_string());
           }
           self.state = ExpenseState::PendingApproval;
           self.submitted_at = Some(Utc::now());
           Ok(())
       }
   }

**Avantages TDD** :

- Design code testable (dependency injection forcée)
- Coverage 100% (tests écrits first)
- Refactoring safe (tests = safety net)

BDD (Behavior-Driven Development)
----------------------------------

**Outil** : Cucumber (Gherkin syntax)

**Usage** : Tests E2E user-facing features

**Exemple** :

.. code-block:: gherkin

   # backend/tests/features/expense_workflow.feature

   Feature: Expense Approval Workflow
     As a syndic, I want to submit expenses for approval
     So that conseil syndical can review before payment

   Scenario: Submit expense for approval
     Given a building "Résidence Mozart" exists
     And an expense "Plumbing repair" in Draft state
     When the syndic submits the expense for approval
     Then the expense state is "PendingApproval"
     And an email is sent to conseil syndical members
     And the audit log records "Submitted by syndic@example.com"

**Run tests BDD** :

.. code-block:: bash

   cd backend && cargo test --test bdd

**Avantages BDD** :

- Lisible non-devs (PO, stakeholders)
- Documentation vivante (tests = specs)
- Focus comportement utilisateur (pas implémentation)

Voir aussi
==========

- :doc:`/governance/nexus/framework` : Coordination cross-équipes
- :doc:`/governance/togaf/adm` : Architecture d'entreprise
- :doc:`/ROADMAP_PAR_CAPACITES` : Roadmap par jalons
- :doc:`/E2E_TESTING_GUIDE` : Guide tests E2E Playwright
- :doc:`/MAKEFILE_GUIDE` : Commandes tests (make test, test-bdd, etc.)

---

*Document maintenu par KoproGo ASBL - Scrum adapté pour l'open-source et contributeurs bénévoles*
