====================================================================================
Issue #344: RFC: Test-Driven Emergence + RACE Adoption (Privacy-First, Graph Social)
====================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: R&D,RFC
:Assignees: Unassigned
:Created: 2026-03-26
:Updated: 2026-03-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/344>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   KoproGo adopte une approche unique : **l'application emerge des tests, pas l'inverse**. Le meme scenario metier se decline a 3 niveaux :
   
   ```
   Scenario metier (source de verite unique, narratif multi-roles)
     | Gherkin
   BDD integration (backend valide le contrat comportemental)
     | meme narratif
   E2E Documentation Vivante (frontend prouve le parcours UI)
     | video generee
   Preuve visuelle (YouTube/stakeholders)
   ```
   
   La **fonctionnalite** et la **conformite** emergent des tests. La **plastique UI** est decouplable. La question : comment optimiser l'**adoption volontaire** des modules communautaires - la pierre angulaire du projet ?
   
   ## Le graph social : enjeu strategique
   
   Les modules obligatoires (vote AG, paiements, convocations) s'imposent d'eux-memes. **L'adoption qui fait ou defait KoproGo est celle des modules communautaires** :
   
   - **SEL** (echanges locaux) - cree du lien entre voisins via le temps partage
   - **Annonces** (notice board) - anime la vie de l'immeuble
   - **Sondages** (polls) - donne une voix entre les AG
   - **Partage d'objets** - reduit les achats redondants
   - **Competences** (skills directory) - valorise les talents des residents
   - **Reservations** (bookings) - mutualise les espaces communs
   - **Gamification** - recompense l'engagement communautaire
   
   Ces modules forment un **graph social de proximite** : chaque echange entre voisins renforce le tissu social de la copropriete. Plus le graph est dense, plus la plateforme a de la valeur. C'est un effet de reseau **hyperlocal** - pas un reseau social generique, mais un outil de solidarite entre voisins qui partagent un bien immobilier commun.
   
   **Sans adoption communautaire, KoproGo n'est qu'un outil administratif de plus. Avec, c'est un catalyseur de vie de quartier.**
   
   ## Proposition : RACE adapte a l'adoption communautaire
   
   | Phase RACE | Objectif KoproGo | KPI |
   |-----------|-----------------|-----|
   | **Reach** | Copropietaires qui **decouvrent** les modules communautaires | % visitant au moins 1 module /mois |
   | **Act** | Copropietaires qui **explorent** (consultent SEL, lisent annonce) | Pages communautaires vues/session |
   | **Convert** | Copropietaires qui **participent** (creent offre, votent, reservent) | % completant 1 action communautaire |
   | **Engage** | Copropietaires qui **reviennent et contribuent** | Retention mensuelle, graph density |
   
   ### Metrique composite : Densite du Graph Social
   
   ```
   Graph Density = (Edges / Maximum Possible Edges) * 100
   - Node = copropietaire actif (1+ action communautaire/mois)
   - Edge = interaction bilaterale (echange SEL, emprunt, evaluation mutuelle)
   - Cible : > 15% = masse critique (effet reseau auto-entretenu)
   ```
   
   ## Privacy-First Analytics : Plausible, pas Hotjar
   
   ### Principes non-negociables
   - **Zero cookies**, zero trackers, zero fingerprinting
   - **GDPR-compliant** sans bandeau de consentement
   - **Self-hostable** - on controle les donnees
   - **Pas de session recording** (pas de Hotjar/surveillance)
   - **Donnees agreggees** uniquement
   
   ### Stack proposee
   
   | Besoin | Outil | Raison |
   |--------|-------|--------|
   | Analytics pages | **Plausible.io** self-hosted | Privacy-first, < 1KB, GDPR natif |
   | Custom events | Plausible Events API | `plausible('SEL Exchange Completed')` |
   | Feature flags | **Unleash** self-hosted | Open source, GDPR, segments par role/batiment |
   | Monitoring | **Prometheus + Grafana** (existant) | RACE dashboards |
   
   ## Integration TOGAF - SAFe - SCRUM - ITIL CSI
   
   ```
   TOGAF ADM Phase H (Architecture Change Management)
     | Building Blocks : modules communautaires + analytics
   SAFe PI Planning
     | Priorise features RACE selon KPIs les plus bas
   SCRUM Sprints
     | BDD/E2E : fonctionnalite emerge des tests
     | A/B variants : CSS/theming uniquement
   Production
     | Plausible mesure RACE KPIs
     | Prometheus mesure performance
   ITIL CSI Review (mensuelle)
     | "Le taux de conversion SEL est 12% (cible 25%)"
     | "Abandon formulaire echange : 40% etape 2"
     | Hypothese UX a tester
   Feature Flag A/B
     | Par batiment (coherence sociale)
     | 2 semaines de mesure
   Decision : rollout variante gagnante
     v retour PI Planning
   ```
   
   ### A/B Testing respectueux
   - Segments par **ROLE** et par **BATIMENT** (pas par individu)
   - Meme variante pour tout un immeuble (coherence entre voisins)
   - **Variantes CSS/theming** - la fonctionnalite est validee par BDD/E2E
   - Rollout canary via ArgoCD (10% -> 50% -> 100% des batiments)
   
   ## OKRs RACE - Adoption Communautaire
   
   ### Q1 : Baseline
   | KR | Cible |
   |----|-------|
   | Copropietaires visitant 1+ module communautaire | > 30% |
   | Copropietaires completant 1+ action communautaire | > 15% |
   | Graph density SEL | > 5% |
   
   ### Q2 : Croissance
   | KR | Cible | Methode |
   |----|-------|---------|
   | Conversion SEL (visit -> action) | > 25% | A/B onboarding |
   | Participation sondages | > 50% invites | A/B notification UX |
   | Graph density | > 15% (masse critique) | Gamification incentives |
   
   ### Q3 : Auto-entretien
   | KR | Cible |
   |----|-------|
   | Retention communautaire mensuelle | > 60% |
   | Note moyenne SEL | > 4.2/5 |
   | Graph density maintenu > 15% | Effet reseau auto-entretenu |
   
   ## Implications techniques
   
   ### Deja en place
   - [x] Prometheus + Grafana + Loki
   - [x] ArgoCD, K3s, Traefik
   - [x] 7 modules communautaires + Gamification
   - [x] BDD/E2E Documentation Vivante
   
   ### A deployer
   - [ ] Plausible self-hosted (1 pod, ~256MB)
   - [ ] Unleash self-hosted (1 pod, ~256MB)
   - [ ] Custom events frontend
   - [ ] Dashboards Grafana RACE (4 KPIs x 7 modules)
   - [ ] Theming decouple (CSS custom properties par variante)
   - [ ] Endpoint /metrics/race agreges (pas de PII)
   
   ## Questions ouvertes
   
   1. Integrer Plausible des le Jalon 2 ou attendre Jalon 3 ?
   2. Seuil de masse critique (15%) a calibrer empiriquement ?
   3. Gamification comme levier RACE (achievements adoption communautaire) ?
   4. Micro-survey in-app optionnel ou analytics passives uniquement ?
   
   ---
   
   *L'application emerge des tests. RACE ajoute la boucle d'adoption pour le graph social hyperlocal. Le tout privacy-first, sans espionnage.*
   
   Generated with [Claude Code](https://claude.com/claude-code)

.. raw:: html

   </div>

