=======================================================================
Issue #51: feat: Board tools (Polls, task management, issue reporting)
=======================================================================

:State: **OPEN**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: phase:vps,track:software priority:high
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-13
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/51>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Le conseil de copropriété (Syndic + membres élus) a besoin d'outils pour gérer les décisions courantes entre les AG :
   - Consulter les résidents rapidement (sondages)
   - Suivre les tâches du conseil
   - Gérer les signalements de problèmes
   - Documenter les décisions
   
   **Cas d'usage :**
   - \"Quelle couleur pour repeindre le hall ?\"
   - \"Quel entrepreneur choisir pour la toiture ?\"
   - \"Qui s'occupe d'obtenir 3 devis pour le chauffage ?\"
   - \"Fuite d'eau signalée au 3ème étage\"
   
   ## Features
   
   ### 1. Sondages (Polls)
   
   **Types :**
   - Oui/Non
   - Choix multiples
   - Note (1-5 étoiles)
   - Question ouverte
   
   **Exemple :**
   ```
   Titre: Choix entrepreneur toiture
   Options:
     - Entrepreneur A (€15,000) [PDF devis]
     - Entrepreneur B (€18,000) [PDF devis]
     - Entrepreneur C (€16,500) [PDF devis]
   Fin: 7 jours
   Résultat: 45/60 votes, Entrepreneur B choisi (52%)
   ```
   
   **Entité :**
   ```rust
   pub struct Poll {
       pub id: Uuid,
       pub building_id: Uuid,
       pub title: String,
       pub poll_type: PollType, // YesNo, MultipleChoice, Rating, OpenEnded
       pub options: Vec<PollOption>,
       pub is_anonymous: bool,
       pub ends_at: DateTime<Utc>,
       pub status: PollStatus,
   }
   ```
   
   ### 2. Gestion de Tâches (Board Tasks)
   
   **Tableau Kanban :** À faire | En cours | Terminé
   
   **Tâches typiques :**
   - Obtenir 3 devis toiture → Pierre
   - Réviser police assurance → Marie
   - Organiser AG annuelle → Syndic
   - Suivre retard entrepreneur → Jacques
   
   **Entité :**
   ```rust
   pub struct BoardTask {
       pub id: Uuid,
       pub title: String,
       pub assigned_to: Option<Uuid>,
       pub priority: TaskPriority, // Urgent, High, Normal, Low
       pub status: TaskStatus, // Todo, InProgress, Completed
       pub due_date: Option<DateTime<Utc>>,
   }
   ```
   
   ### 3. Signalements (Issue Reporting)
   
   **Résidents signalent problèmes :**
   - Fuite d'eau
   - Lumière cassée
   - Ascenseur en panne
   - Nuisance sonore
   - Problème parking
   
   **Workflow :**
   1. Résident signale avec photos
   2. Syndic voit notification
   3. Assigne tâche à conseil/entrepreneur
   4. Suit résolution
   5. Notifie résident quand résolu
   
   **Entité :**
   ```rust
   pub struct BuildingIssue {
       pub id: Uuid,
       pub reported_by: Uuid,
       pub title: String,
       pub category: IssueCategory, // Plumbing, Electricity, Heating, Security
       pub severity: IssueSeverity, // Critical, High, Medium, Low
       pub status: IssueStatus, // Reported, InProgress, Resolved
       pub photos: Vec<String>,
   }
   ```
   
   ### 4. Journal Décisions (Decision Log)
   
   **Transparence décisions conseil :**
   - \"Conseil a choisi Entrepreneur X (€5,000)\"
   - \"Nouvelle règle : interdiction fumer espaces communs\"
   - \"Décision reportée à prochaine AG : panneaux solaires\"
   
   **Entité :**
   ```rust
   pub struct BoardDecision {
       pub id: Uuid,
       pub title: String,
       pub description: String,
       pub decision_date: DateTime<Utc>,
       pub vote_summary: String, // \"Unanime\" ou \"4 pour, 1 contre\"
       pub is_public: bool,
   }
   ```
   
   ## Permissions
   
   | Action | Résident | Membre Conseil | Syndic |
   |--------|----------|----------------|--------|
   | Voir sondages | ✅ | ✅ | ✅ |
   | Créer sondages | ❌ | ✅ | ✅ |
   | Voter | ✅ | ✅ | ✅ |
   | Gérer tâches | ❌ | ✅ | ✅ |
   | Signaler problème | ✅ | ✅ | ✅ |
   | Assigner problème | ❌ | ✅ | ✅ |
   
   **Nouveau rôle :** `BoardMember` (membre élu du conseil)
   
   ## API Endpoints
   
   **Polls :**
   - POST /api/v1/buildings/:id/polls
   - GET /api/v1/polls/:id/results
   - POST /api/v1/polls/:id/vote
   
   **Tasks :**
   - POST /api/v1/buildings/:id/board-tasks
   - PUT /api/v1/board-tasks/:id/status
   
   **Issues :**
   - POST /api/v1/buildings/:id/issues
   - PUT /api/v1/issues/:id/resolve
   
   **Decisions :**
   - POST /api/v1/buildings/:id/board-decisions
   - GET /api/v1/buildings/:id/board-decisions
   
   ## Effort
   
   **Large** (8-10 jours)
   
   ## Related
   
   - Complète: #46 (votes AG formels)
   - Dépend: #28 (multi-rôles pour BoardMember)

.. raw:: html

   </div>

