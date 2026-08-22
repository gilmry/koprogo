=========================================================================================================================================================================================
Issue #428: Méta — Simulation organisation produit complète multi-cadres (TOGAF + Essential SAFe + Nexus + Scrum + Maury) avec agents IA + backend GH + WBS + traces publiques cohérentes
=========================================================================================================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: documentation,track:infrastructure priority:critical,governance
:Assignees: Unassigned
:Created: 2026-04-29
:Updated: 2026-04-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/428>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte — l'apex de l'expérimentation
   
   Ce dépôt est volontairement une expérimentation d'industrialisation de la production de code par agents IA (#425, #426, #427). **Important : la v0.1.0 n'est pas en production. Aucun système live ne dépend du code actuel.** Cette phase pre-release est précisément le moment qu'on utilise pour câbler les recettes — l'urgence est stratégique (que les recettes soient en place avant la première mise en ligne), pas opérationnelle.
   
   Cette issue acte la **vision apex** : utiliser le backend GitHub (issues, projects, milestones, actions, comments, PRs, releases) comme support d'une **simulation complète d'organisation produit logiciel multi-cadres**, avec agents IA jouant tous les rôles, à des cadences réalistes, avec cérémonies, artefacts, ADRs, RFCs, WBS et deadlines.
   
   **Cadres mobilisés** : **TOGAF ADM** (architecture entreprise) + **Essential SAFe** (single ART, PI cadence) + **Nexus** (intégration multi-équipes) + **Scrum** (équipes) + **Maury** (pipeline par feature, cf. `Maury/Méthode Maury.md`).
   
   **Double critère de réussite** :
   1. **Richesse organisationnelle** : un observateur externe lisant les traces publiques voit l'équivalent qualitatif d'une équipe produit mature de 8-15 personnes — épics découpés, sprints planifiés/revus, PI events, ADRs argumentées, RFCs débattues, rétrospectives honnêtes, demos, releases tagguées avec rapports humains. Indistinguable d'une vraie équipe sur le plan de la **complétude** des artefacts.
   2. **Transparence pédagogique** : un manifeste public (`docs/SIMULATION_MANIFEST.md` + section README) explique la nature simulée + intention expérimentale + recettes utilisées. La simulation **n'est pas une dissimulation** ; c'est un **terrain d'observation publique des recettes d'industrialisation IA**.
   
   ---
   
   ## 1. Enseignements de l'expérimentation à date (2026-04-29)
   
   L'audit qui a déclenché cette issue a produit des observations exploitables. Cette section consigne les **leçons apprises** que les recettes ci-dessous opérationnalisent :
   
   | Pathologie observée | Leçon | Recette qui répond |
   |---|---|---|
   | 1 967 `unwrap()`/`expect()` en prod | Sans `PreToolUse` qui exige un test rouge sur les chemins d'erreur, l'agent optimise pour "compile" et ignore `Result<E>` | #427 partie A (4 catégories + RED-first) |
   | 8 helm values avec `koprogo123` recopié | Sans hook `PreToolUse Edit` détectant secrets en clair, l'agent recopie | #425 (deny + gitleaks) |
   | 921 scénarios BDD mais bugs UX en v0.1.0 | "Nombre de scénarios" ≠ "couverture par catégorie" ; sans matrice 4×N par FR, la métrique est trompeuse | #427 partie A.3 |
   | Composants Svelte > 800 LOC | Sans skill `extract-shared-component` ni story granularité S/M/L imposée, l'agent accumule | #428 (Maury intégré, story-fitting) |
   | `:latest` partout | Sans politique d'imagepolicy + scan supply chain, l'agent prend le défaut paresseux | #425 + future hook |
   | Doc obsolète (`.claude/hooks.md` format ancien) | La doc qui rassure sans protéger est pire que pas de doc ; auto-régénération depuis script + datage obligatoire | #426 + cette issue (auto-doc) |
   | CLAUDE.md à 1 469 lignes / 92 "✅ NOUVEAU" | Toute info chargée à chaque session est une **taxe permanente** sur tous les agents — section §2 ci-dessous | #426 (trim) + cette issue (consolidation tokens) |
   | Audit "claims vs réalité" : ×4.6 inflation LOC | Sans script de mesure auto, le narratif drift indépendamment du code | Cette issue (auto-WBS, auto-velocity, auto-ADR digest) |
   
   **Méta-leçon** : un agent IA optimise mécaniquement pour les métriques mesurées et les chemins de moindre résistance. Si on mesure "compile + lint", il livre cela. Si on mesure "matrice 4×N + ADR signé + RFC accepté + sprint review humain", il livre cela. **La discipline doit être instrumentée, pas demandée.**
   
   ---
   
   ## 2. Stratégie de consolidation des tokens
   
   La méthode Maury et tout ce qui suit n'a de sens qu'avec une **gestion explicite du budget tokens**, sinon l'orchestration multi-cadres explose en consommation. Principes :
   
   ### 2.1 Investir en amont, pas en aval
   - Itérations de directives (Brief → PRD → Architecture → Stories) consomment **avant** la génération de code.
   - Une fois la story signée, l'agent code lit **un seul fichier** (story.md) et n'a **aucune décision** à prendre. Génération mécanique = quasi gratuite.
   - Tech debt = symptôme de directive insuffisante, pas d'agent rapide. Le fix est upstream.
   
   ### 2.2 Curer le contexte chargé à chaque session
   - `CLAUDE.md` cible ≤ 5 000 tokens (aujourd'hui ~33 000) — cf. #426. Chaque session paie ce prix × N agents.
   - `MEMORY.md` index ≤ 500 tokens (sain). Mémoires individuelles ≤ 1 500 tokens chacune.
   - Pas de doublon documentaire (#426 : `CONVOCATIONS_AG.rst` ↔ `CONVOCATIONS_SYSTEM.rst`, etc.) — l'agent qui doit réconcilier paie deux fois.
   
   ### 2.3 Budget par artefact
   | Artefact | Cible (tokens) |
   |---|---|
   | `CLAUDE.md` | ≤ 5 000 |
   | `MEMORY.md` index | ≤ 500 |
   | Brief par feature | ≤ 8 000 |
   | PRD par feature | ≤ 25 000 |
   | Architecture par feature | ≤ 20 000 |
   | Story (1) | ≤ 2 000 |
   | ADR | ≤ 1 500 |
   | RFC | ≤ 4 000 |
   | Persona system prompt | ≤ 2 000 |
   | Persona memory file | ≤ 1 500 |
   | Total contexte ambiant par session code | ≤ 15 000 |
   
   ### 2.4 Templates stricts = zéro devinette
   - Plus le template est strict (sections fixes, frontmatter obligatoire, matrice 4×N imposée), **moins** l'agent consomme : il a la forme à remplir, pas à inventer.
   - Anti-pattern : prompt vague + free-form output. Symptôme : l'agent re-fait 3 fois la même structure dans des fichiers différents.
   
   ### 2.5 Mesure
   - `make token-budget` (à créer) : compte tokens par artefact, alerte si > cible.
   - GH Action weekly : commit `docs/metrics/token-budget-YYYY-MM-DD.md` ; trend visible.
   - Cible : budget ambiant /3 sur 6 mois (de ~50 k → ~15 k tokens chargés à chaque session code).
   
   ### 2.6 Mode "économie" pour tâches simples
   - Pour fixes mineurs (typo, bump version), pas besoin de charger 15 personas. Skill `/quick-fix` qui n'invoque que `dev` + `qa` + hooks gitleaks.
   - Pour features majeures, full pipeline Maury → Stories → Code → Validation.
   
   ---
   
   ## 3. Vision multi-cadres emboîtés
   
   | Cadre | Cadence | Niveau d'abstraction | Artefacts produits |
   |---|---|---|---|
   | **TOGAF ADM** (9 phases A-H + Requirements) | itération annuelle | architecture entreprise | Vision, Capability Map, ADRs, Technology Roadmap, Migration Plan |
   | **Essential SAFe Portfolio** | trimestriel | thèmes stratégiques | OKRs, Lean Budget, Value Streams, Strategic Themes |
   | **Essential SAFe ART / PI** | 8-12 semaines | objectifs et flux release | PI Objectives, System Demo, Inspect & Adapt |
   | **Nexus** | sprint | intégration multi-équipes | Dependency Map, Integration Backlog, Nexus Daily |
   | **Scrum** | 1-2 semaines | incrément livré | Sprint Goal, Increment, Demo, Retrospective |
   | **Maury** (par feature) | scope feature | production-ready discipline | Brief → PRD → Architecture → Stories (cf. `Maury/`) |
   | **Daily** | 1 jour | progrès et impediments | Standup, Daily Integration |
   
   **Pourquoi Essential SAFe (pas full SAFe)** : KoproGo = 1 ART (single value stream copropriété belge). Les couches Large Solution / Portfolio Solution Train du full SAFe ne s'appliquent pas. Essential SAFe = la configuration minimale (1 ART, 1 portfolio, équipes Scrum) — pile bonne taille pour la simulation.
   
   ### 3bis. La méthode Maury elle-même évolue (TOGAF ADM Phase H)
   
   La méthode Maury n'est **pas un dogme figé**. Elle est elle-même un livrable versionné qui doit s'adapter aux enseignements de l'expérimentation. Le mécanisme est emprunté au **TOGAF ADM Phase H — Architecture Change Management**, mais appliqué à la **méthode** (pas seulement à l'architecture du produit) :
   
   | Trigger | Output |
   |---|---|
   | Fin de chaque PI Inspect & Adapt → issue auto `maury-method-review-YYYY-Qx` | Liste des observations PI sur la méthode (templates inadéquats, gates manquants, friction inutile) |
   | Pattern récurrent identifié → RFC `docs/rfc/NNNN-maury-evolution-*.md` | Proposition d'évolution avec contexte / motivation / alternatives / drawbacks |
   | RFC accepté → ADR + nouvelle version `Maury/Méthode_Maury_vX.Y.md` | Changelog explicite, version semver, ADR qui acte le changement |
   | Migration des features en cours | Plan de migration : features actives finissent en méthode v(N), nouvelles features démarrent en v(N+1) |
   
   **Versionnage** :
   - `Maury/Méthode_Maury_v1.0.md` (version initiale, déjà existante en `Maury/Méthode Maury.md`)
   - `Maury/CHANGELOG.md` consigne tous les changements avec lien RFC/ADR
   - Tags Git `maury-v1.0`, `maury-v1.1`, `maury-v2.0` pour figer les versions
   - Chaque feature dans `docs/maury/<feature>/` mentionne dans son frontmatter `maury_version: v1.1` (pour que les agents sachent quel template appliquer)
   
   **Exemples plausibles d'évolutions futures** :
   - *"RFC : ajouter une 5e catégorie de tests `@accessibility` à la matrice 4×N"* (déclenchée si on voit que les bugs A11y passent les 4 catégories actuelles)
   - *"RFC : story XL = trop gros, scinder obligatoirement"* (déclenchée si plusieurs stories XL ratent leur sprint)
   - *"RFC : remplacer Mary+John par un seul agent unifié"* (déclenchée si on voit que les transitions Brief→PRD perdent du contexte)
   - *"RFC : intégrer les ADRs comme output obligatoire de chaque feature Maury"* (déclenchée si les ADRs sont absents pour les features récentes)
   
   **Principe** : la méthode et la production produit évoluent **en symbiose**. Les enseignements remontent au cadre, et le cadre évolue. Cf. §1 (enseignements) qui sera mis à jour à chaque PI I&A.
   
   ---
   
   ## 4. Mapping GitHub backend (tout doit y être tracé)
   
   | Concept agile | Primitive GH | Exemple |
   |---|---|---|
   | Strategic theme / OKR | Project board "Portfolio" + label `togaf:vision` | "Conformité copropriété belge 2026" |
   | Epic (multi-PI) | Issue label `safe:epic`, milestone `pi-2026-Q3+` | "Refonte multi-rôle frontend" |
   | Feature (PI-fitting) | Issue label `safe:feature` + lien `docs/maury/<feature>/` | "Système votes AG" |
   | Story (sprint-fitting) | Issue label `scrum:story` + milestone sprint | "TICKET-007 voter résolution simple maj." |
   | Task | Sub-issue ou checklist dans story | "RED test domain `Vote::cast` rejette double-vote" |
   | Sprint Goal | Milestone description + issue `sprint-goal-WXX` | "Implémenter votes simple majorité bout en bout" |
   | PI Objectives | Issue `pi-objectives-YYYY-Qx` + project board PI | "AG en visio + votes + procurations" |
   | Sprint Backlog | Project board "Sprint WXX" filtré par milestone | tableau Kanban |
   | ART Backlog | Project board "ART" filtré par labels | priorisation cross-équipe |
   | **Daily Standup** | Issue auto-générée `daily-YYYY-MM-DD` | chaque agent commente |
   | **Sprint Review** | Issue `sprint-review-WXX` + lien preview env | démo + métriques |
   | **Sprint Retrospective** | Issue `sprint-retro-WXX` (keep/drop/try) | actions → nouvelles issues |
   | **PI Planning** | Issue `pi-planning-YYYY-Qx` (2 jours simulés) | objectifs + risks + dépendances |
   | **PI System Demo** | Issue `pi-demo-YYYY-Qx` + Cowork report (#427) | démo intégrée 8 sem. |
   | **PI Inspect & Adapt** | Issue `pi-ia-YYYY-Qx` | PMI assessment + actions PI suivant |
   | **ADR** (Architecture Decision Record) | Fichier `docs/adr/NNNN-titre.md` + PR template | "ADR-0042 : SOPS pour secrets IaC" |
   | **RFC** (Request For Comments) | Fichier `docs/rfc/NNNN-titre.md` + PR + commentaires | "RFC-0007 : adopter Essential SAFe" |
   | WBS | Auto-généré `docs/wbs/wbs-YYYY-MM-DD.md` depuis arbre issues | hiérarchie complète |
   | Velocity / Burndown | GH Action génère `docs/metrics/velocity-WXX.md` | par équipe + global |
   | Capability Map (TOGAF) | `docs/togaf/capabilities-map.md` + issues `togaf:capability` | bounded contexts → capacités |
   | Token budget | `docs/metrics/token-budget-YYYY-MM-DD.md` | trend + alerte |
   | **GH Discussions** | Forums catégorisés | "Architecture", "Process", "Decisions Log", "Q&A", "Show & Tell", "Retrospective Themes" |
   
   **Issues vs Discussions** :
   - **GH Issues** = work items avec état (`open`/`closed`) et assignee. Tout ce qui doit aboutir à du code, un artefact, ou un livrable.
   - **GH Discussions** = conversations ouvertes sans état terminal, lieu naturel pour :
     - **Architecture** : débats RFC avant décision (le RFC formel reste un fichier `docs/rfc/NNNN-*.md`, mais la délibération multi-agents se fait en Discussion)
     - **Process** : améliorations méthodo, propositions cross-sprints (peuvent générer une RFC formelle)
     - **Decisions Log** : récap mensuel des décisions importantes (résumé condensé pour le lecteur externe)
     - **Q&A** : questions agent→humain ou agent→agent (une question d'agent au superviseur n'est pas une issue à fermer)
     - **Show & Tell** : démos d'agents, exemples de patterns, ADRs intéressants à mettre en avant
     - **Retrospective Themes** : thèmes émergents inter-sprints qui méritent suivi long-terme (ex: "qualité des PRD signe-t-elle vraiment la fin de l'itération directive ?")
   
   **Règle d'or** : **rien ne se passe hors GitHub**. Toute décision, toute review, toute discussion architecture, toute estimation, toute rétro, toute interrogation = un artefact GH (issue, discussion, comment, PR, fichier, label, milestone). Si ça n'est pas dans GH, ça n'a pas eu lieu.
   
   ---
   
   ## 5. Distinction ADR vs RFC
   
   | | ADR (Architecture Decision Record) | RFC (Request For Comments) |
   |---|---|---|
   | Quand | Décision architecturale **prise** | Proposition à **débattre** avant décision |
   | Forme | Contexte / Options / Décision / Conséquences | Sommaire / Motivation / Détail / Alternatives / Drawbacks / Open questions |
   | Auteur | Architect (TOGAF ou ART ou System) | N'importe quel agent ou humain |
   | Statut | `accepted` / `superseded` / `deprecated` | `draft` / `under-review` / `accepted` / `rejected` |
   | Lieu | `docs/adr/NNNN-*.md` | `docs/rfc/NNNN-*.md` |
   | Granularité | Local à un BC ou cross-cutting | Process, méthode, choix de framework, politique |
   | Exemples | "Utiliser SOPS pour secrets IaC", "Hexagonal pour backend", "Essential SAFe vs full" → décision finale | "RFC : adopter convention de branche `story/<id>`", "RFC : matrice 4×N par FR", "RFC : politique commit messages" |
   
   **Workflow** : RFC → débat (PR + commentaires agents + humain) → si accepté, génère un ou plusieurs ADRs concrets pour les choix architecturaux qui en découlent.
   
   ---
   
   ## 6. Agents et personas (15-20 rôles)
   
   Chaque persona = un fichier `.claude/agents/<role>.md` :
   - system prompt définissant rôle + style + focus
   - style guide (ton, tics, vocabulaire, signature)
   - mémoire `.claude/agents/<role>.memory.md` (décisions passées, cohérence inter-sprints)
   - activité cadencée (cron par persona)
   
   **TOGAF level (architecture entreprise)** :
   - `togaf-chief-architect` — phases B-D, ADRs cross-cutting, capability map
   - `togaf-business-architect` — bounded contexts, value streams
   - `togaf-information-architect` — data architecture, intégrations
   - `togaf-technology-architect` — stack, NFRs, technology roadmap
   
   **Essential SAFe Portfolio level** :
   - `safe-portfolio-manager` — strategic themes, lean budget
   - `safe-epic-owner` — épics multi-PI
   
   **Essential SAFe ART level** (1 ART) :
   - `safe-rte` (Release Train Engineer) — orchestre PI events, dépendances
   - `safe-system-architect` — system of systems pour ART
   - `safe-product-manager` — vision ART, roadmap, features
   
   **Team level (Scrum, 2 équipes parallèles)** :
   - Team A (Backend/Domain) : `scrum-master-A`, `product-owner-A`, `dev-A`, `qa-A`
   - Team B (Frontend/UX) : `scrum-master-B`, `product-owner-B`, `dev-B`, `qa-B`
   - `ux-designer` (cross-équipes)
   
   **Maury level (par feature)** :
   - `maury-mary` — Brief
   - `maury-john` — PRD
   - `maury-winston` — Architecture
   - `maury-bob` — Stories
   
   **Cross-cutting** :
   - `nexus-integration-team` — daily integration check, dependency tracker
   - `release-manager` — gestion releases, lien Cowork (#427)
   - `security-officer` — review sécurité (#425)
   - `documentation-writer` — auto-doc, ADR digest, RFC index, WBS regen
   - `human-supervisor` (Gilles) — single source of human decisions, sign verdicts
   
   ---
   
   ## 7. Cadence des cérémonies (GH Actions cron)
   
   | Cérémonie | Cron | Output |
   |---|---|---|
   | Daily Standup | weekday 09:00 UTC | issue `daily-YYYY-MM-DD` (chaque agent commente) |
   | Daily Nexus integration | weekday 10:00 UTC | issue `nexus-daily-YYYY-MM-DD` |
   | Backlog Refinement | vendredi 14:00 UTC | issue `refinement-WXX` |
   | Sprint Planning | lundi sprint-start 09:00 | issue `sprint-planning-WXX` |
   | Sprint Review | vendredi sprint-end 14:00 | issue `sprint-review-WXX` + demo lien |
   | Sprint Retrospective | vendredi sprint-end 16:00 | issue `sprint-retro-WXX` |
   | PI Planning | 1er lun. de Q (2 jours simulés) | issue `pi-planning-YYYY-Qx` |
   | PI System Demo | dernier ven. de Q | issue `pi-demo-YYYY-Qx` (avec Cowork release report) |
   | PI Inspect & Adapt | lundi suivant PI Demo | issue `pi-ia-YYYY-Qx` |
   | Portfolio Sync | 1er lun. du mois | issue `portfolio-sync-YYYY-MM` |
   | TOGAF ADM iteration | annuel | série issues phases A-H |
   | **Maury Method Review** | fin de chaque PI (Phase H) | issue `maury-method-review-YYYY-Qx` (cf. §3bis) |
   | ADR digest | monthly | `docs/adr/_index-YYYY-MM.md` |
   | RFC digest | monthly | `docs/rfc/_index-YYYY-MM.md` |
   | WBS regeneration | weekly (dim soir) | `docs/wbs/wbs-YYYY-MM-DD.md` |
   | Velocity report | sprint-end | `docs/metrics/velocity-WXX.md` |
   | Token budget report | weekly | `docs/metrics/token-budget-YYYY-MM-DD.md` |
   | Burndown | quotidien sprint | image dans milestone description |
   
   ---
   
   ## 8. Output documentaire systématique
   
   Chaque niveau produit ses artefacts, **tous tracés dans GH** :
   
   | Niveau | Outputs documentaires |
   |---|---|
   | TOGAF | Capability Map, Vision Document, Technology Roadmap, ADRs cross-cutting, Migration Plan |
   | Portfolio | Strategic Themes, OKRs, Lean Budget, Value Stream Map |
   | ART / PI | PI Objectives, PI Planning Minutes, System Demo Notes, I&A Workshop Output, ART Roadmap |
   | Nexus | Dependency Map, Integration Backlog, Daily Nexus Notes |
   | Sprint | Sprint Goal, Sprint Planning Minutes, Daily Standup Logs, Sprint Review Notes, Retro (keep/drop/try) |
   | Feature (Maury) | Brief, PRD, Architecture (avec ADRs), Stories, Validation Report |
   | Decision-level | ADRs (architecture choices) + RFCs (proposals/process changes) |
   | Release | Cowork+human review report (cf. #427), Release Notes, CHANGELOG |
   | Métriques | Velocity, Burndown, Token Budget, ADR/RFC index, WBS |
   | Méthode | `Maury/Méthode_Maury_vX.Y.md` versioned + `Maury/CHANGELOG.md` (cf. §3bis) |
   | Conversations ouvertes | GH Discussions par catégorie (Architecture, Process, Decisions Log, Q&A, Show & Tell, Retrospective Themes) |
   
   **Règle** : tout output existe sous deux formes — (1) fichier markdown dans le repo, (2) référencement dans une issue GH. La traçabilité est complète : on peut remonter d'une ligne de code → story → feature → PI → epic → strategic theme → vision TOGAF.
   
   ### 8bis. Pipeline de mise à jour documentaire pre-release
   
   Une étape automatique régénère/synchronise **toute** la documentation pour qu'elle reflète exactement l'état releasé, **avant** le `git tag`. Évite la dérive doc vs code observée à l'audit (CLAUDE.md à 1469 lignes drifté, claims ×4.6 inflation).
   
   **Trigger** : branche `release/vX.Y.Z` ouverte ET rapport Cowork+humain signé `verdict: GO` (cf. #427).
   
   **Job CI `release-doc-refresh` — 12 étapes** :
   1. **API doc** : `cargo doc --no-deps` + export OpenAPI utoipa → `docs/api/openapi-vX.Y.Z.json` + HTML statique.
   2. **Endpoint listings CLAUDE.md** : section auto-régénérée par script depuis `routes.rs` (remplace les listings manuels actuels qui drift à chaque feature).
   3. **CHANGELOG** : compilé depuis les PRs mergées entre `vX.Y.(Z-1)` et HEAD via `gh pr list --state merged --search "merged:>tag-date"`, groupé `feat/fix/docs/refactor/security`.
   4. **Release notes** : générées depuis CHANGELOG + sprint reviews du PI courant + PI demo notes.
   5. **ADR/RFC snapshot pour la version** : `docs/adr/_index-vX.Y.Z.md` et `docs/rfc/_index-vX.Y.Z.md` figés.
   6. **WBS snapshot release** : `docs/wbs/wbs-release-vX.Y.Z.md` figé pour postérité.
   7. **Velocity history snapshot** : `docs/metrics/velocity-history.md` mis à jour avec la donnée du PI clos.
   8. **README badges** : version, coverage par catégorie (cf. #427), gitleaks status, mutation kill rate, token budget actuel.
   9. **Maury frozen status** : tous les `docs/maury/<feature>/*.md` impliqués dans la release passent leur frontmatter `released_in: vX.Y.Z`.
   10. **CLAUDE.md claims régénérés** : LOC via `tokei`, scénarios BDD via grep tagué, issues open/closed via `gh issue list`, FRs implémentés via traçabilité — élimine la dérive narrative.
   11. **Public manifest** : `docs/SIMULATION_MANIFEST.md` mis à jour avec stats release (PRs mergées, agents actifs, cérémonies tenues, RFCs acceptés/rejetés).
   12. **Sphinx rebuild + publish** : `make docs` puis push GitHub Pages.
   
   **PR auto-créée** : `chore(docs): refresh for vX.Y.Z` agrégeant les 12 outputs. Humain review rapide (dérive ?), merge, **puis seulement** le tag est posé.
   
   **Hook gate** : `PreToolUse Bash(git tag:*)` valide :
   - (a) rapport Cowork signé `verdict: GO` (cf. #427)
   - (b) PR `chore(docs): refresh for vX.Y.Z` mergée
   - (c) tous les `docs/maury/<feature>/*.md` impliqués passés en `released_in: vX.Y.Z`
   
   Si un des trois manque → blocage (`exit 2`).
   
   **Pourquoi avant le tag, pas après** :
   - Tag puis update docs = le tag pointe vers du code dont la doc va dériver = problème actuel.
   - Update avant tag = tag est un snapshot **cohérent** code+doc.
   - `gh release create` peut linker directement les artefacts doc figés pour ce tag.
   
   **Cadence du job** : hors périodique — déclenché uniquement par le flux release. Mais la **doc en cours de PI** continue de se régénérer weekly (WBS, velocity, ADR digest, token budget cf. §7) pour ne pas accumuler la dette doc à la dernière minute.
   
   ---
   
   ## 9. Public trace coherence rules
   
   Pour que la simulation **lise** comme une vraie équipe (richesse, pas dissimulation) :
   
   1. **Persona voice consistency**. Style guide par agent. Mary parle métier (PRD, contexte, personas), Winston parle architecture (ports, adapters, ADRs), Bob parle découpe (stories, vélocité). Pas de mélange.
   2. **Memory continuity**. Memory files consigne décisions, choix, opinions précédentes. Pas de contradiction inter-sprints sans justification.
   3. **Realistic disagreement**. Conflits constructifs documentés (PO vs Dev sur scope, Architect vs RTE sur dépendance, QA vs Dev sur niveau de test). Résolution dans les ceremonies.
   4. **Time distribution**. Cron par persona avec offset.
   5. **Realistic velocity**. Variance 70-110 % de baseline. Retards documentés.
   6. **Realistic blockers**. Blocked stories documentés et reportés.
   7. **Coherent ADRs/RFCs**. Décisions liées aux issues qui les ont déclenchées ; RFCs ont vrai débat en commentaires ; ADRs citent leur RFC parent.
   8. **Public manifest** (`docs/SIMULATION_MANIFEST.md`) :
      - Nature simulée + intention expérimentale
      - Liste personas IA + responsabilités
      - Rôle du superviseur humain (Gilles)
      - Recettes utilisées (#425, #426, #427, cette issue)
      - **Stratégie de consolidation tokens** (cf. §2)
      - Critères de succès expérimentation
      - "Non-deceptive simulation" clearly stated
   9. **README.md racine** mentionne explicitement la simulation et linke le manifesto.
   10. **Issue/PR templates** incluent note "🤖 Cet artefact est produit par l'agent `<role>` dans le cadre de la simulation organisationnelle (cf. SIMULATION_MANIFEST.md)".
   
   ---
   
   ## 10. Critères d'acceptation
   
   - [ ] 15+ agents personas matérialisés `.claude/agents/<role>.md` avec system prompt + style guide.
   - [ ] Memory files par persona (`.claude/agents/<role>.memory.md`).
   - [ ] Cron GH Actions configurés pour les 14+ cérémonies listées.
   - [ ] Templates issue créés pour chaque cérémonie (sprint-planning, sprint-review, sprint-retro, pi-planning, pi-demo, pi-ia, daily-standup, refinement, portfolio-sync).
   - [ ] Templates ADR + RFC + manifesto + simulation note PR.
   - [ ] WBS auto-régénéré weekly + velocity sprint-end + token-budget weekly.
   - [ ] `docs/SIMULATION_MANIFEST.md` publié avec stratégie tokens (§2) explicite.
   - [ ] `docs/adr/` et `docs/rfc/` initialisés avec ADR-0001 (cette simulation) et RFC-0001 (adoption Essential SAFe).
   - [ ] **Sprint pilote complet** (2 sem.) : sprint-planning → 5 daily standups → sprint-review (demo) → sprint-retro → ADRs/RFCs nouvelles → WBS update → velocity report → token-budget report.
   - [ ] **PI pilote complet** (8 sem. = 4 sprints) : PI Planning → 4 sprints intégrés → System Demo (Cowork report cf. #427) → I&A.
   - [ ] Lecteur externe lisant les issues d'un sprint reconstruit l'histoire complète.
   - [ ] Lecteur externe ouvrant le manifesto comprend immédiatement la nature simulée et l'intention.
   - [ ] Token budget global /3 vs baseline (cf. §2.5).
   - [ ] Job CI `release-doc-refresh` (12 étapes §8bis) implémenté et testé sur une release pilote ; PR auto-créée avant tag ; hook tag-gate vérifie a+b+c.
   - [ ] GH Discussions catégorisées créées (Architecture, Process, Decisions Log, Q&A, Show & Tell, Retrospective Themes) ; au moins 1 thread par catégorie peuplé pendant le sprint pilote.
   - [ ] `Maury/CHANGELOG.md` initialisé ; au moins une RFC d'évolution Maury déposée pendant un PI pilote pour valider le cycle Phase H §3bis.
   
   ---
   
   ## 11. Sprints (proposition de phasage)
   
   | Sprint | Durée | Livrables | Effet |
   |---|---|---|---|
   | **S1 — Foundations agents + manifesto + tokens budget** | 4 sem | 15 personas + memory files + manifesto + style guides + ADR-0001 + RFC-0001 + token budget script | L'organisation a son casting et sa charte |
   | **S2 — Cadence quotidienne et sprint** | 4 sem | Daily standup auto, sprint-planning/review/retro auto, WBS weekly, burndown, velocity | Le rythme sprint tourne |
   | **S3 — Cadence PI / Essential SAFe** | 8 sem | PI Planning event, System Demo, I&A, project board PI | Le rythme PI tourne |
   | **S4 — Portfolio + TOGAF** | 8 sem | Portfolio sync mensuel, ADM iteration phases A-H, capability map, ADR/RFC digests | L'enterprise architecture est documentée |
   | **S5 — Pilote sprint complet** | 2 sem | 1 sprint réel piloté par la simulation, audit trace | Validation pilote |
   | **S6 — Pilote PI complet** | 8 sem | 1 PI réel (4 sprints), audit trace, retro globale, leçons consignées | Validation PI |
   
   Total ~32 semaines (8 mois) pour la simulation complète opérationnelle.
   
   ---
   
   ## 12. Liens avec les autres issues
   
   - **#425** (garde-fous IA techniques) — couche sécurité runtime, prérequis. Chaque agent appelle ces hooks.
   - **#426** (cleanup docs) — contexte propre prérequis ; CLAUDE.md trim avant 15 personas (sinon explosion budget tokens cf. §2).
   - **#427** (validation discipline TDD/BDD + Cowork release gate) — chaque sprint review/PI demo passe par le gate Cowork ; chaque story produite par un agent doit avoir matrice 4×N.
   - **Maury method** (`Maury/Méthode Maury.md`) — pipeline feature-level qui s'insère au niveau Scrum. Méthode déjà documentée, on la rend opérationnelle ici.
   
   ---
   
   ## 13. Risques et limites
   
   - **Token cost** : 15+ agents avec memory + activité cadencée = budget significatif. Mitigation §2 (consolidation tokens) ; mesure weekly.
   - **Cohérence inter-sprint** : risque drift de persona. Mitigation : memory files + revue trimestrielle humaine.
   - **Effet "uncanny valley"** : si simulation presque-mais-pas-tout-à-fait réaliste, inconfortable. Mitigation : manifesto qui pose clairement le cadre.
   - **Engagement humain** : malgré l'automatisation, le superviseur reste critique (gates Maury, sign verdicts, retro globale). Pas magique.
   - **GitHub rate limits** : 15+ bots posting régulièrement peuvent toucher les limites. Mitigation : batch + cron offsets.
   - **Maintenance des recettes** : les hooks/skills/agents/templates eux-mêmes deviennent un système à maintenir. Discipline RFC pour les évolutions.
   
   ---
   
   ## Annexe A — Inventaire exhaustif des rituels et artefacts par cadre
   
   Anticipation de tout ce qui peut être agent-automatisé. Sera matérialisé dans `docs/agent-rituals/INVENTORY.md` en S1, mais consigné ici pour que le scope soit fixé.
   
   **Légende** : **Auto** = automatisable agent ; **Semi** = agent prépare/exécute, humain valide aux gates ; **Humain** = décision humaine (l'agent peut produire un draft).
   
   ### A.1 TOGAF ADM (cycle complet)
   
   | Phase | Événement / Artefact | Cadence | Owner agent | A/S/H | Output GH |
   |---|---|---|---|---|---|
   | Preliminary | Architecture Principles | annuel | togaf-chief-architect | Semi | `docs/togaf/principles.md` + RFC |
   | Preliminary | Architecture Governance Charter | annuel | togaf-chief-architect | Semi | `docs/togaf/governance.md` |
   | A | Vision Document | annuel | togaf-business-architect | Semi | `docs/togaf/vision-YYYY.md` |
   | A | Stakeholder Map | annuel | togaf-business-architect | Auto | issue `togaf-stakeholder-map-YYYY` |
   | A | Business Scenarios | annuel | togaf-business-architect | Auto | `docs/togaf/scenarios/` |
   | A | Communication Plan | annuel | release-manager | Auto | `docs/togaf/communication-plan.md` |
   | B | Business Capability Map | annuel | togaf-business-architect | Auto | `docs/togaf/capabilities-map.md` |
   | B | Value Streams | annuel | togaf-business-architect + safe-portfolio-manager | Semi | `docs/togaf/value-streams.md` |
   | B | Organization Map | annuel | togaf-business-architect | Auto | `docs/togaf/org-map.md` |
   | B | Business Process Map | annuel | togaf-business-architect | Auto | `docs/togaf/process-maps/` |
   | C-Data | Data Entities catalog | semi-annuel | togaf-information-architect | Auto | `docs/togaf/data-entities.md` |
   | C-Data | Data Dictionary | continu | togaf-information-architect | Auto | `docs/togaf/data-dictionary.md` |
   | C-Data | Data Lineage | continu | togaf-information-architect | Auto | `docs/togaf/data-lineage/` |
   | C-App | Application Portfolio | annuel | togaf-chief-architect | Auto | `docs/togaf/app-portfolio.md` |
   | C-App | Interface Catalog | continu | togaf-chief-architect | Auto | `docs/togaf/interfaces.md` |
   | D | Technology Standards | annuel | togaf-technology-architect | Semi | `docs/togaf/tech-standards.md` |
   | D | Technology Portfolio | semi-annuel | togaf-technology-architect | Auto | `docs/togaf/tech-portfolio.md` |
   | E | Gaps Matrix | par PI | togaf-chief-architect | Auto | issue `togaf-gaps-YYYY-Qx` |
   | E | Migration Strategy | annuel | togaf-chief-architect | Semi | `docs/togaf/migration-strategy.md` |
   | F | Implementation Plan | par PI | safe-rte | Auto | `docs/togaf/impl-plan-YYYY-Qx.md` |
   | F | Architecture Roadmap | annuel | togaf-chief-architect | Auto | `docs/togaf/roadmap-YYYY.md` |
   | G | Architecture Contracts | par release | togaf-chief-architect | Semi | issue `arch-contract-vX.Y.Z` |
   | G | Compliance Reviews (PR) | continu | togaf-chief-architect (sub-agent) | Auto | comments PR |
   | G | Change Requests | événementiel | n'importe qui | Semi | RFC + issue |
   | H | Maury Method Review | fin de PI | togaf-chief-architect + safe-rte | Auto | issue + RFC évolution méthode (§3bis) |
   | H | Architecture Updates | continu | togaf-chief-architect | Auto | ADRs + capability map updates |
   | Req Mgmt | Requirements Repository | continu | safe-product-manager | Auto | `docs/togaf/requirements.md` |
   | Req Mgmt | Requirements Impact Assessment | par change request | togaf-chief-architect | Semi | comment RFC |
   
   ### A.2 Essential SAFe — Portfolio Level
   
   | Événement / Artefact | Cadence | Owner agent | A/S/H | Output GH |
   |---|---|---|---|---|
   | Portfolio Vision | annuel/Q | safe-portfolio-manager | Semi | `docs/safe/portfolio-vision.md` |
   | Strategic Themes | annuel | safe-portfolio-manager | Humain | issues label `togaf:vision` |
   | Lean Budget allocation | trimestriel | safe-portfolio-manager | Semi | `docs/safe/budget-YYYY-Qx.md` |
   | Value Stream identification | annuel | safe-portfolio-manager | Auto | `docs/togaf/value-streams.md` |
   | Portfolio Kanban | continu | safe-portfolio-manager | Auto | GH Project board "Portfolio" |
   | Portfolio Sync | mensuel | safe-portfolio-manager + safe-epic-owners | Auto | issue `portfolio-sync-YYYY-MM` |
   | Epic Hypothesis Statement | par épic | safe-epic-owner | Semi | issue `safe:epic` body |
   | Lean Business Case | par épic | safe-epic-owner | Semi | document attaché à l'épic |
   | Epic → MMF (Minimum Marketable Feature) decomposition | par épic | safe-epic-owner | Auto | sub-issues features |
   
   ### A.3 Essential SAFe — ART Level
   
   | Événement / Artefact | Cadence | Owner agent | A/S/H | Output GH |
   |---|---|---|---|---|
   | Vision (ART) | par PI | safe-product-manager | Semi | `docs/safe/art-vision-YYYY-Qx.md` |
   | Roadmap (ART) | par PI | safe-product-manager | Auto | `docs/safe/roadmap-YYYY-Qx.md` |
   | ART Backlog | continu | safe-product-manager | Auto | GH Project board "ART" |
   | Solution / System Architecture | continu | safe-system-architect | Auto | ADRs + `docs/safe/sys-arch.md` |
   | Features (PI-fitting) | par PI | safe-product-manager | Auto | issues label `safe:feature` |
   | Enabler Features | par PI | safe-system-architect | Auto | issues label `safe:enabler` |
   | WSJF prioritization | par refinement | safe-product-manager | Auto | scores dans body issue |
   | ART Sync (Scrum of Scrums + PO Sync) | weekly | safe-rte | Auto | issue `art-sync-WXX` |
   | System Demo | sprint-end | safe-rte + qa | Auto | issue `system-demo-WXX` + lien preview |
   | ART Inspect & Adapt | fin de PI | safe-rte + tous | Auto | issue `pi-ia-YYYY-Qx` |
   | Solution Intent | continu | safe-system-architect | Auto | `docs/safe/solution-intent.md` |
   | ART Roadmap update | par PI | safe-rte | Auto | maj `docs/safe/roadmap-YYYY-Qx.md` |
   
   ### A.4 Essential SAFe — PI Planning Event (2 jours simulés)
   
   | Sous-événement | Owner agent | A/S/H | Output GH |
   |---|---|---|---|
   | Business Context | safe-portfolio-manager | Semi | section issue PI Planning |
   | Vision (rappelée) | safe-product-manager | Auto | section issue PI Planning |
   | Architecture Vision (rappelée) | safe-system-architect | Auto | section issue PI Planning |
   | Team Breakouts (planification équipe) | scrum-masters + dev/qa | Auto | sub-issues sprint planning par équipe |
   | Draft Plan Review | safe-rte | Auto | comment issue PI Planning |
   | Management Review | safe-portfolio-manager | Semi | comment issue PI Planning |
   | Risk ROAMing (Resolved/Owned/Accepted/Mitigated) | safe-rte + équipes | Auto | issue `pi-risks-YYYY-Qx` |
   | Final Plan Review | safe-rte | Auto | comment issue PI Planning |
   | Confidence Vote (par équipe) | tous agents (vote simulé) | Auto | poll dans issue PI Planning |
   | PI Planning Retro | safe-rte | Auto | issue `pi-planning-retro-YYYY-Qx` |
   | PI Objectives par équipe | scrum-masters | Auto | issues `pi-objectives-team-X-YYYY-Qx` |
   
   ### A.5 Essential SAFe — CD Pipeline
   
   | Étape | Owner agent | A/S/H | Output |
   |---|---|---|---|
   | Continuous Exploration | safe-product-manager | Auto | issues label `safe:exploration` |
   | Continuous Integration | dev-team + qa | Auto | PRs + CI verts |
   | Continuous Deployment (preview env) | release-manager | Auto | preview env per branch |
   | Release on Demand | release-manager + humain | Humain | tag + GH release (cf. #427 + §8bis) |
   
   ### A.6 Nexus
   
   | Événement / Artefact | Cadence | Owner agent | A/S/H | Output GH |
   |---|---|---|---|---|
   | Nexus Sprint Goal | par sprint | nexus-integration-team | Semi | section issue sprint-planning |
   | Nexus Sprint Backlog | continu sprint | nexus-integration-team | Auto | GH Project board cross-équipe |
   | Nexus Sprint Planning | sprint-start | nexus-integration-team + équipes | Auto | issue `nexus-planning-WXX` |
   | Nexus Daily Scrum | weekday 10:00 UTC | nexus-integration-team | Auto | issue `nexus-daily-YYYY-MM-DD` |
   | Nexus Sprint Review | sprint-end | nexus-integration-team + équipes | Auto | issue `nexus-review-WXX` |
   | Nexus Sprint Retrospective | sprint-end | nexus-integration-team | Auto | issue `nexus-retro-WXX` |
   | Cross-team Refinement | weekly | nexus-integration-team + POs | Auto | issue `nexus-refinement-WXX` |
   | Integrated Increment | sprint-end | nexus-integration-team | Auto | preview env + Cowork verification |
   | Dependency Map | continu | nexus-integration-team | Auto | `docs/nexus/dependency-map.md` (regen weekly) |
   
   ### A.7 Scrum (par équipe — Team A backend, Team B frontend)
   
   | Événement / Artefact | Cadence | Owner agent | A/S/H | Output GH |
   |---|---|---|---|---|
   | Sprint Goal | sprint-start | product-owner-X | Semi | section issue sprint-planning |
   | Product Backlog | continu | product-owner-X | Auto | GH Project board "Backlog Team X" |
   | Sprint Backlog | sprint | scrum-master-X | Auto | GH Project board "Sprint WXX Team X" |
   | Sprint Planning | sprint-start | scrum-master-X + équipe | Auto | issue `sprint-planning-WXX-team-X` |
   | Daily Standup | weekday 09:00 UTC | scrum-master-X (orchestre) | Auto | issue `daily-YYYY-MM-DD` (commentaires par équipier) |
   | Sprint Review | sprint-end | product-owner-X + dev-X + qa-X | Auto | issue `sprint-review-WXX-team-X` + démo |
   | Sprint Retrospective | sprint-end | scrum-master-X | Auto | issue `sprint-retro-WXX-team-X` (keep/drop/try → actions) |
   | Backlog Refinement | weekly | product-owner-X + équipe | Auto | issue `refinement-WXX-team-X` |
   | Definition of Ready (DoR) | continu | product-owner-X | Auto | `.claude/templates/dor.md` + checks issue |
   | Definition of Done (DoD) | continu | qa-X | Auto | `.claude/templates/dod.md` + checks PR |
   | Increment | sprint-end | dev-X | Auto | merged PRs + preview env |
   | Story estimation (planning poker simulé) | refinement | équipe (vote agents) | Auto | poll dans issue refinement |
   | Impediments log | continu | scrum-master-X | Auto | issues label `impediment` |
   | Product Goal | continu | product-owner-X | Semi | description épic ou milestone |
   
   ### A.8 Cross-cutting
   
   | Événement / Artefact | Cadence | Owner agent | A/S/H | Output GH |
   |---|---|---|---|---|
   | ADR | par décision archi | architects | Auto | `docs/adr/NNNN-*.md` + PR |
   | RFC | par proposition | n'importe quel agent | Semi | `docs/rfc/NNNN-*.md` + Discussion + PR |
   | Maury feature pipeline | par feature | maury-mary→john→winston→bob | Auto avec gates humains | `docs/maury/<feature>/*.md` |
   | Code review automated | par PR | qa + security-officer (sous-agent) | Auto | comments PR |
   | Cowork release review | par release | release-manager + humain | Semi | `docs/maury/releases/vX.Y.Z/human-review-report.md` |
   | Doc refresh release | par release | documentation-writer | Auto + PR human-validé | PR `chore(docs): refresh for vX.Y.Z` (cf. §8bis) |
   | WBS regen | weekly | documentation-writer | Auto | `docs/wbs/wbs-YYYY-MM-DD.md` |
   | Velocity report | sprint-end | documentation-writer | Auto | `docs/metrics/velocity-WXX.md` |
   | Token budget report | weekly | documentation-writer | Auto | `docs/metrics/token-budget-YYYY-MM-DD.md` |
   | ADR/RFC digest | monthly | documentation-writer | Auto | `docs/adr/_index-YYYY-MM.md`, `docs/rfc/_index-YYYY-MM.md` |
   | Incident response | événementiel | security-officer | Semi | issue `incident-YYYY-MM-DD-NNN` |
   | Postmortem | post-incident | security-officer + équipes concernées | Auto | `docs/postmortem/YYYY-MM-DD.md` |
   | Decisions Log digest | monthly | documentation-writer | Auto | GH Discussion catégorie "Decisions Log" |
   | GH Discussions modération + digest | weekly | documentation-writer | Auto | digest hebdo dans issue |
   | README / manifesto refresh | par release | documentation-writer | Auto | partie du job §8bis |
   | Maury method changelog update | par RFC accepté Phase H | documentation-writer | Auto | `Maury/CHANGELOG.md` + tag git |
   | Anti-pattern detection | continu (PR) | security-officer + qa | Auto | comments PR + issue si récurrent |
   | Onboarding nouvelles features | par feature | documentation-writer | Auto | `docs/maury/<feature>/onboarding.md` |
   | Risk register update | par PI + événementiel | safe-rte | Auto | `docs/safe/risk-register.md` |
   | OKR tracking | trimestriel | safe-portfolio-manager | Auto | `docs/safe/okr-YYYY-Qx.md` |
   
   **Total** : ~75 rituels/artefacts agent-automatisables. Chacun → script de génération + template + agent owner + cron (si périodique). À matérialiser progressivement S1→S6.
   
   ---
   
   🤖 Issue générée par Claude Opus 4.7 — apex de l'expérimentation : organisation produit complète simulée multi-cadres (TOGAF + Essential SAFe + Nexus + Scrum + Maury) avec agents IA, backend GH, ADR/RFC, WBS, GH Discussions, doc auto-refresh pre-release et traces publiques cohérentes pédagogiques. Les enseignements à date et la stratégie de consolidation tokens sont consignés en §1 et §2 ; l'inventaire exhaustif des rituels en Annexe A. Chaque issue future devra référencer ces sections pour rester alignée.

.. raw:: html

   </div>

