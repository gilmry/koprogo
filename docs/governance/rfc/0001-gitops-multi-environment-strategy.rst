================================================================
RFC 0001: Stratégie GitOps multi-environnement (alternative F)
================================================================

:RFC: 0001
:Auteur: gilmry <gilmry@gmail.com>
:Date: 2026-05-01
:Statut: Draft
:Type: Architecture
:Équipes: infra, devops
:Jalon: 0 (pré-prod, stabilisation v0.1.0)

.. contents:: Table des matières
   :depth: 3
   :local:

Résumé (TL;DR)
==============

Adopter un modèle de branches GitOps **hybride symétrique app/infra** : 4 branches
applicatives (``dev``, ``integration``, ``staging``, ``production``) + 4 branches
infra (``infra-dev``, ``infra-integration``, ``infra-staging``, ``infra-prod``) +
``main`` comme snapshot post-prod consolidé. Backports automatiques selon des
cascades distinctes pour les deux pipelines (apps bottom-up, infra top-down).
Refactor minimal de l'ApplicationSet (les 2 generators existants pointent
chacun sur leur série de branches).

Métadonnées
===========

.. list-table::
   :widths: 30 70
   :header-rows: 0

   * - **RFC**
     - 0001
   * - **Auteur**
     - gilmry <gilmry@gmail.com>
   * - **Date création**
     - 2026-05-01
   * - **Statut**
     - Draft
   * - **Type**
     - Architecture (changement structurel du flow GitOps)
   * - **Équipes impliquées**
     - infra, devops
   * - **Jalon cible**
     - 0 (pré-prod, stabilisation v0.1.0)
   * - **Priorité**
     - P1 (high — bloque la stack GitOps actuelle)
   * - **Effort estimé**
     - 7 PR successives (~2-3 sprints d'effort cumulé)
   * - **Dépendances**
     - PR #465 mergée (gitops bootstrap unblock) ; issue #466 (RFC framing) verrouillée

Contexte et Problème
====================

Problème adressé
----------------

**Qui** : mainteneurs et solo devs de KoproGo, agents IA platform-engineer / devops-engineer.

**Problème** : Suite à la PR #465 (gitops bootstrap unblock), la validation sur cluster
docker-desktop a révélé un *chicken-and-egg structurel* dans le flow GitOps :

- Les ``ApplicationSet``s ArgoCD (``koprogo-infra``, ``koprogo-app``) ciblent les
  branches ``dev``, ``staging``, ``integration``, ``production`` via
  ``targetRevision: {{ .branch }}``.
- Les fichiers d'infra (``cluster-profiles/``, ``kustomize/base/``, ``helm/``,
  ``monosite/k3s/<env>/``) vivent uniquement sur ``feature/dev``.
- Les 8 Applications générées sont en ``SYNC=Unknown`` car ArgoCD checkout
  ``dev``/etc → fichiers absents → manifest generation échoue.

Couplage induit : un fix infra (ex: ``templatePatch`` booleans dans #465) doit
attendre 4 PR successives sur 4 branches d'env. Une feature applicative en
développement peut bloquer un fix infra urgent.

**Impact si non résolu** :

- Aucun déploiement GitOps possible vers les 4 envs (les Applications restent
  ``Unknown``)
- Pas de séparation des responsabilités entre changements app et changements infra
- Coût opérationnel élevé sur chaque modification d'infra (ex: bump cluster-profile)

Contexte organisationnel
-------------------------

- **Jalon actuel** : pré-prod, stabilisation v0.1.0 (cf. mémoire
  ``project_koprogo-current-state.md``)
- **Contrainte CLAUDE.md #10** : v0.1.0 n'est pas en prod, aucun système live ;
  les findings sont des observations d'expérimentation, pas une crise
- **Contrainte CLAUDE.md #2** : pas de prod autonome ; chaque opération sur
  ``production`` ou ``infra-prod`` requiert validation humaine (Tier 1)
- **Contrainte CLAUDE.md #11** : Tier 1 (mutation prod) vs Tier 2 (proposal,
  comments) — toute promotion doit respecter ce modèle

Pourquoi maintenant ?
---------------------

- La stack GitOps est **bloquée** : ``gitops-bootstrap.sh docker-desktop``
  termine sans erreur mais aucune App ne sync. Toute itération sur l'infra K8s
  dépend de la résolution de cette RFC.
- Le diagnostic précis est posé (voir issue #466 commentaires) — il faut
  trancher avant que la dette ne s'accumule (chaque infra fix manuel sur 4
  branches augmente la dette).
- v0.1.0 n'est pas en prod : c'est **le bon moment** pour refondre le modèle
  de branches sans casser un service live.

Solution Proposée
=================

Vue d'ensemble
--------------

Topologie de branches (9 + ``main``) :

.. code-block:: text

   Apps:    feature/X  → dev  → integration  → staging  → production  ──┐
                                                                         ├─→ main
   Infra:   infra/X    → infra-dev → infra-integration → infra-staging → infra-prod ──┘

- **App pipeline** (cycle bottom-up) : features mûrissent dans ``dev``, montent
  vers ``integration``, sont promues vers ``staging``, puis vers ``production``.
- **Infra pipeline** (cycle top-down) : ``infra-prod`` (snapshot stable) est la
  référence ; les envs inférieurs reçoivent un backport descendant pour rester
  alignés.
- ``main`` = snapshot post-release consolidé (app + infra à un instant T en prod).

Cascades de backport automatiques
----------------------------------

Trois cascades, déclenchées par des événements distincts :

.. code-block:: text

   ═══════════════════════════════════════════════════════════════════
    SOURCES                          TARGETS
   ═══════════════════════════════════════════════════════════════════
                                     ┌─ infra-prod
                                     ├─ prod (app)
    main ───────(C1)──────────────►  ├─ infra-staging
                                     └─ staging (app)

                                     ┌─ infra-staging
    infra-prod ───(C2)──────────►    └─ infra-integration

    dev (app) ───(C3)───────────►    integration (app)

    ★ Protégé (jamais de backport entrant) : infra-dev
   ═══════════════════════════════════════════════════════════════════

**C1 — Snapshot post-release** : déclenchée au merge de ``main`` (= post-prod
consolidé). Re-distribue aux 4 envs "released" (prod + staging des deux
pipelines).

**C2 — Hotfix infra** : déclenchée au merge sur ``infra-prod`` (PR directe
hotfix). Propage descendant vers ``infra-staging`` et ``infra-integration``,
**sauf** ``infra-dev`` qui reste l'entrée d'expérimentation.

**C3 — Maturation features apps** : déclenchée au merge sur ``dev``. ``integration``
reçoit le HEAD de ``dev`` pour tester l'intégration des features ensemble.

Branches protégées (jamais de backport entrant) :

- ``infra-dev`` : entrée pipeline infra, expérimentation libre
- ``dev`` : entrée pipeline app (mais ``dev`` est aussi *source* de C3)

Promotions ascendantes (manuelles ou Tier 2 + Tier 1)
------------------------------------------------------

- **App** :

  - ``dev → integration`` : automatique via C3
  - ``integration → staging`` : PR manuelle ou Tier 2, validation Tier 1
  - ``staging → production`` : PR manuelle ou Tier 2, **environment approval GH** Tier 1

- **Infra** :

  - ``infra/X → infra-dev`` : PR Tier 2 ou directe (entrée pipeline)
  - ``infra-dev → infra-integration`` : PR manuelle ou Tier 2
  - ``infra-integration → infra-staging`` : PR manuelle ou Tier 2
  - ``infra-staging → infra-prod`` : PR manuelle ou Tier 2, **environment approval GH** Tier 1

- **Merges vers main** :

  - ``production → main`` : auto-merge déclenché par succès du sync ArgoCD prod (côté app)
  - ``infra-prod → main`` : auto-merge déclenché par succès du sync ArgoCD prod (côté infra)

Hotfix prod
-----------

PR directe autorisée sur ``production`` ou ``infra-prod`` (bypass du gitflow normal).

- Côté infra : la cascade C2 propage automatiquement aux envs inférieurs (``infra-staging``, ``infra-integration``)
- Côté app : back-merge ``production → staging/integration`` à clarifier
  (suggestion : workflow dédié ``backmerge-prod.yml`` qui crée 2 PRs à valider Tier 1)

Couplage app↔infra (release commune)
-------------------------------------

Mécanisme : **tag commun** ``v<X>.<Y>.<Z>`` posé simultanément sur ``production``
et ``infra-prod`` quand les deux pipelines sont synchronisés en prod.

Au niveau PR description : lien obligatoire vers la PR jumelle si une feature
app nécessite un changement infra (ex: nouvelle migration K8s, nouveau secret).
Implémentation via PR template GH avec champ ``Refs-Twin: #<num>``.

Détails techniques
------------------

ApplicationSet refactor
~~~~~~~~~~~~~~~~~~~~~~~

**Avant** (état actuel après PR #465) : les 2 generators pointent sur les mêmes
4 branches (``dev``, ``integration``, ``staging``, ``production``).

**Après** : ``koprogo-app`` reste inchangé ; ``koprogo-infra`` pointe sur les 4
branches infra-* :

.. code-block:: yaml

   # ApplicationSet 1: Infrastructure (Kustomize)
   spec:
     generators:
       - list:
           elements:
             - branch: infra-dev          # ← changé (était "dev")
               environment: dev
               namespace: koprogo-dev
               kustomizePath: infrastructure/monosite/k3s/dev/kustomize
               autoSync: false
               prune: false
             - branch: infra-integration  # ← changé (était "integration")
               environment: integration
               # ...
             - branch: infra-staging      # ← changé
               environment: staging
               # ...
             - branch: infra-prod         # ← changé (était "production")
               environment: production
               # ...

   # ApplicationSet 2: Application (Helm) — INCHANGÉ
   spec:
     generators:
       - list:
           elements:
             - branch: dev                # ← inchangé
             - branch: integration        # ← inchangé
             # ...

C'est l'**effort de refactor le plus minimal possible** parmi les alternatives
de l'issue #466.

Workflow CI infra
~~~~~~~~~~~~~~~~~

Nouveau fichier ``.github/workflows/ci-infra.yml`` :

.. code-block:: yaml

   name: ci-infra
   on:
     pull_request:
       branches: ['main', 'infra-*']
       paths:
         - 'infrastructure/**'
     push:
       branches: ['infra/**']
       paths:
         - 'infrastructure/**'

   jobs:
     kustomize-build:
       strategy:
         matrix:
           env: [dev, integration, staging, production]
       steps:
         - uses: actions/checkout@v4
         - run: kubectl kustomize infrastructure/monosite/k3s/${{ matrix.env }}/kustomize/

     helm-template:
       strategy:
         matrix:
           env: [dev, integration, staging, production]
           profile: [docker-desktop, k3s-self-hosted, k8s-managed]
       steps:
         - uses: actions/checkout@v4
         - run: |
             helm template . \
               -f infrastructure/_shared/cluster-profiles/${{ matrix.profile }}.yaml \
               -f infrastructure/monosite/k3s/${{ matrix.env }}/helm-values.yaml

     kubeconform:
       needs: [kustomize-build, helm-template]
       steps:
         # validation contre schémas K8s (à détailler dans la PR-4)

Targets (12 helm combos + 4 kustomize builds + 1 kubeconform). Estimation :
< 2 min sur runner standard.

Workflows de backport (cascades C1, C2, C3)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

3 workflows GH déclenchés par événements distincts :

- ``backport-c1-main.yml`` : ``on: push: branches: [main]`` → ouvre 4 PRs
  vers ``infra-prod``, ``prod``, ``infra-staging``, ``staging``
- ``backport-c2-infra-prod.yml`` : ``on: push: branches: [infra-prod]`` →
  ouvre 2 PRs vers ``infra-staging`` et ``infra-integration``
- ``backport-c3-dev.yml`` : ``on: push: branches: [dev]`` → ouvre 1 PR vers
  ``integration``

Implémentation suggérée : ``peter-evans/create-pull-request@v6`` (action
maintenue, supporte conflict markers).

Tier 1/2 :

- Le **workflow** (Tier 2) crée les PRs automatiquement
- L'**humain** (Tier 1) review et merge — mais les PRs peuvent être
  auto-mergeable si :ref:`branch-protection-rules` sont satisfaites (CI green +
  required reviewers absent OU bypass label)

Branch protection rules (à appliquer après PR-3)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Branche
     - Protection
   * - ``main``
     - Required PR review (1+) + required checks (ci-infra) + no force-push
   * - ``production``
     - Required PR review (1+) + environment ``production`` (manual approval)
   * - ``infra-prod``
     - Required PR review (1+) + environment ``infra-production`` (manual approval)
   * - ``staging``, ``infra-staging``
     - Required PR review (1+) + required checks (ci-infra)
   * - Autres
     - Required checks uniquement

CODEOWNERS pour ``infrastructure/monosite/k3s/production/`` et
``infrastructure/_shared/`` : assignment automatique aux mainteneurs infra.

Initialisation des nouvelles branches infra-*
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Snapshot initial = état courant de ``main`` (principe backport). Étapes :

.. code-block:: bash

   git checkout main
   git pull
   for env in dev integration staging prod; do
     git checkout -b infra-$env
     git push -u origin infra-$env
   done

Aucun fichier modifié — c'est juste la création des 4 nouveaux pointeurs
qui partent du même commit que ``main``.

Impact et Métriques
===================

Impact technique
----------------

.. list-table::
   :header-rows: 1
   :widths: 30 30 40

   * - Métrique
     - Avant RFC
     - Après RFC
   * - **Apps stuck en ``SYNC=Unknown``**
     - 8 / 8
     - 0 (après PR-2 + PR-3)
   * - **PRs nécessaires pour 1 fix infra**
     - 4 (1 par branche d'env)
     - 1 (PR ``infra/X`` → ``infra-dev``, cascade auto)
   * - **Couplage app/infra dans une PR**
     - Fort (feature/dev embarque tout)
     - Découplé (PRs séparées + tag commun)
   * - **CI temps moyen sur changement infra**
     - ~10 min (suite Rust + Playwright complète)
     - < 2 min (ci-infra dédié, paths-based)
   * - **Audit "what's in prod"**
     - Difficile (4 branches d'env divergentes)
     - ``main`` = snapshot consolidé
   * - **Hotfix prod (bypass gitflow)**
     - N/A
     - PR directe sur ``production`` ou ``infra-prod`` autorisée

Impact métier
-------------

Pas d'impact utilisateur direct (changement infra). Impacts indirects :

- **Time-to-fix infra** : divisé par ~4 (1 PR au lieu de 4)
- **Confiance déploiement** : audits plus simples (``main`` source de vérité
  consolidée)
- **Onboarding nouveau dev/agent** : modèle symétrique app/infra plus lisible

Impact roadmap
--------------

- **Pré-requis** pour activer GitOps en production v0.1.x
- **Bloque** : toute initiative de promotion image automatique (Argo Image
  Updater, release-please) tant que la topologie n'est pas figée
- **Lié** : ADR future sur le secret management (Vault, SealedSecrets,
  ExternalSecrets) — chaque pipeline aura son backend

Alternatives Considérées
========================

Voir l'issue #466 pour le tableau comparatif détaillé. Synthèse rapide :

Alternative A : Status quo amélioré
------------------------------------

**Description** : Garder 4 branches d'env, ajouter sync auto ``feature/dev →
dev → ...``

**Décision** : ❌ Rejetée — couplage infra/apps reste fort.

Alternative B : ``infra/*`` → ``main`` source unique infra
-----------------------------------------------------------

**Description** : Branches ``infra/*`` mergent toutes sur ``main`` ; ApplicationSet
pointe sur ``main``, identité env = path.

**Décision** : ❌ Rejetée — refactor ApplicationSet majeur, branches d'env
restent ambiguës (gardées ? supprimées ?).

Alternative C : ``main`` source unique pour TOUT
-------------------------------------------------

**Description** : Plus de branches d'env, ``main`` fait foi. Promotion = bump
tag d'image dans ``<env>/helm-values.yaml`` via PR.

**Décision** : ❌ Rejetée — refactor le plus lourd, change le review process /
rollback / hotfix flow simultanément.

Alternative D : 2 repos (apps / infra)
---------------------------------------

**Description** : Repo ``koprogo`` (apps) + repo ``koprogo-infra`` (manifests).

**Décision** : ❌ Rejetée — coordination cross-repo trop lourde pour la taille
de KoproGo en v0.1.0.

Alternative E : Ne rien faire
------------------------------

**Description** : Reporter la décision après v0.1.0 stabilisée (CLAUDE.md #10).

**Décision** : ❌ Rejetée — la stack GitOps est bloquée maintenant ; chaque
infra fix manuel sur 4 branches creuse la dette.

Alternative F : Hybride symétrique (RETENUE)
---------------------------------------------

**Décision** : ✅ Retenue (cf. décisions verrouillées sur issue #466).

Justification : refactor ApplicationSet **minimal** (les 2 generators pointent
chacun sur leur série de branches), symétrie cognitive forte (même structure
pour app et infra), compatible solo dev OU mainteneur, ``main`` = audit
consolidé, hotfix prod possible (bypass gitflow autorisé).

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
   * - Désync ``infra-X`` ↔ ``X`` (oubli PR jumelle)
     - Bug en prod app dû à infra obsolète
     - PR template avec champ ``Refs-Twin``, lint CI
     - Moyenne
   * - Backport workflow boucle infinie
     - GH Actions consommé, PRs auto qui s'enchaînent
     - Idempotence ``peter-evans/create-pull-request`` (no-op si rien à backport)
     - Faible
   * - Conflits backport non résolus
     - PR auto bloquée, intervention manuelle
     - Conflit-detection step + assignation automatique au mainteneur
     - Moyenne
   * - Auto-merge sur main avant sync ArgoCD réel
     - Drift entre source de vérité et état déployé
     - Webhook conditionnel : auto-merge uniquement si ArgoCD ``Healthy`` ET ``Synced``
     - Faible

Risques métier
--------------

.. list-table::
   :header-rows: 1
   :widths: 30 30 30 10

   * - Risque
     - Impact
     - Mitigation
     - Probabilité
   * - Solo dev oublie le tag commun
     - Release app ↔ release infra désynchronisée
     - PR template avec section "Twin PR / Tag commun"
     - Moyenne
   * - Mainteneur unique = bottleneck approval
     - Promotions infra-prod / production ralenties
     - CODEOWNERS + multi-mainteneurs (à recruter)
     - Faible (en v0.1.0)
   * - Adoption flow par contributeurs externes
     - Confusion sur "où PR ?"
     - Doc CONTRIBUTING.md mise à jour, issue templates par type
     - Faible

Plan de Rollback
----------------

Si l'alternative F s'avère inadaptée après mise en place (ex: workflows
backport instables), rollback possible :

1. **Revert PR-3** (ApplicationSet refactor) : le generator ``koprogo-infra``
   re-pointe sur ``dev/integration/staging/production``
2. **Désactiver workflows** ``backport-c1/c2/c3.yml`` (commenter le ``on:``)
3. **Conserver les branches** ``infra-*`` (no-op, peuvent être laissées dormantes)
4. **Réouvrir issue #466** pour reconsidérer une alternative (B, C ou D)

Coût rollback : ~2h (1 PR revert + 1 PR désactivation workflows).

Plan d'Implémentation
=====================

Décomposition en 7 PRs successives
-----------------------------------

.. list-table::
   :header-rows: 1
   :widths: 8 50 25 17

   * - PR
     - Contenu
     - Branche
     - Tier
   * - PR-1
     - Kustomize patches (target selector + commonLabels migration)
     - ``chore/fix-gitops-env-branches-targets`` *(en cours)*
     - 2 (proposal)
   * - PR-2
     - Création snapshot initial des 4 branches ``infra-*`` depuis ``main``
     - Opération manuelle ``git push`` (pas de PR formelle)
     - 1 (humain)
   * - PR-3
     - Refactor ApplicationSet (``koprogo-infra`` → ``infra-*``)
     - ``infra/applicationset-infra-branches``
     - 2 (proposal)
   * - PR-4
     - Workflow ``ci-infra.yml`` (kustomize/helm/kubeconform paths-based)
     - ``infra/ci-workflow``
     - 2 (proposal)
   * - PR-5
     - Workflows backport (C1, C2, C3) via ``peter-evans/create-pull-request``
     - ``infra/backport-cascades``
     - 2 (proposal)
   * - PR-6
     - Branch protection rules + environment approvals + CODEOWNERS
     - Configuration GH (settings UI ou ``.github/settings.yml``)
     - 1 (humain ; settings GH critiques)
   * - PR-7
     - Workflow tag commun release + PR template ``Refs-Twin`` field
     - ``infra/release-tag-common``
     - 2 (proposal)

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
     - PR-1 mergée (kustomize fix sur ``feature/dev``)
     - 2026-05-02
     - 🚧 En cours (5 fichiers staged)
   * - **M2**
     - PR-2 (init des 4 branches infra-*)
     - 2026-05-02
     - ⏳ Pending
   * - **M3**
     - PR-3 + PR-4 mergées (ApplicationSet refactor + ci-infra)
     - 2026-05-05
     - ⏳ Pending
   * - **M4**
     - PR-5 mergée (backport cascades) + dry-run validé
     - 2026-05-08
     - ⏳ Pending
   * - **M5**
     - PR-6 + PR-7 mergées (branch protection + tag commun)
     - 2026-05-10
     - ⏳ Pending

Dépendances
-----------

- **PR #465** : ✅ mergée (gitops bootstrap unblock)
- **Issue #466** : ✅ verrouillée (alternative F retenue)
- **Mainteneur(s) GH avec admin rights** : nécessaire pour PR-6 (branch
  protection settings)

Critères d'Acceptation
=======================

Critères fonctionnels
---------------------

1. **GitOps bootstrap end-to-end**

   - **Given** : cluster vide (docker-desktop ou k3s)
   - **When** : ``./infrastructure/_shared/scripts/gitops-bootstrap.sh <type>``
   - **Then** : 8 Applications créées, **toutes** en ``SYNC=Synced`` et
     ``HEALTH=Healthy`` après ~3 min

2. **Cascade C2 (hotfix infra)**

   - **Given** : push direct sur ``infra-prod`` (hotfix)
   - **When** : workflow ``backport-c2-infra-prod.yml`` se déclenche
   - **Then** : 2 PRs créées vers ``infra-staging`` et ``infra-integration``
   - **And** : ces PRs ont le label ``backport`` et un titre ``Backport: <commit
     msg>``

3. **Tag commun release**

   - **Given** : ``production`` et ``infra-prod`` synced en ArgoCD
   - **When** : workflow ``release-tag-common.yml`` déclenché manuellement
   - **Then** : tag ``v<X>.<Y>.<Z>`` posé sur les deux branches simultanément
   - **And** : merges auto vers ``main`` déclenchés (PR auto-merged)

Critères techniques
-------------------

1. ✅ ``ci-infra.yml`` durée < 2 min sur runner GitHub standard
2. ✅ Workflows backport idempotents (no-op si rien à backport)
3. ✅ Conflits backport détectés et assignés à un humain
4. ✅ Branch protection rules configurées sur ``main``, ``production``,
   ``infra-prod``, ``staging``, ``infra-staging``
5. ✅ Environment approval configuré sur ``production`` et ``infra-prod``
6. ✅ Documentation ``CONTRIBUTING.md`` mise à jour (où PR ?)
7. ✅ ADR ``0045-gitops-multi-environment-strategy.md`` accompagne la RFC
   (résumé décision)

Critères non-fonctionnels
--------------------------

1. **Compat Tier 1/2 (CLAUDE.md #11)** : tous les Tier 1 (mutation prod,
   création doc publique, fermeture issue) restent humains ; Tier 2 (workflows
   backport) sont loggés
2. **Pas de ``--no-verify``** (CLAUDE.md ligne rouge) : workflows backport
   honorent les git hooks (skipped uniquement pour les PRs auto, pas pour les
   PRs humaines)
3. **Pas de prod autonome** (CLAUDE.md #2) : auto-merge sur ``main`` n'est
   PAS un déploiement, c'est juste une mise à jour du snapshot — le déploiement
   réel reste piloté par ArgoCD avec sync auto activé seulement sur
   ``staging``/``integration``/``dev`` (pas sur ``production``)

Processus Revue RFC
===================

Soumission
----------

1. ✅ Créer fichier ``docs/governance/rfc/0001-gitops-multi-environment-strategy.rst``
2. ✅ Branch ``chore/fix-gitops-env-branches-targets`` (peut héberger la RFC + PR-1)
3. 🚧 PR vers ``feature/dev``, tag ``rfc``, assigner reviewer (mainteneur)

Revue communauté
----------------

- **Durée** : 7j minimum (CLAUDE.md règle template RFC)
- **Reviewers** : à désigner (idéalement 2+ Tech Leads selon dispo)
- **Critères approval** : approve mainteneur principal + 0 objection majeure non résolue

Décision
--------

À ce stade : Statut ``Draft``. Passe à ``Review`` à l'ouverture de la PR ;
puis ``Accepted`` après merge ; puis ``Implemented`` après PR-7 mergée.

Références
==========

- **PR #465** : https://github.com/gilmry/koprogo/pull/465 (gitops bootstrap unblock)
- **Issue #466** : https://github.com/gilmry/koprogo/issues/466 (RFC framing + décisions verrouillées)
- **CLAUDE.md règles** : #2 (pas de prod autonome), #10 (v0.1.0 pas en prod), #11 (Tier 1/2)
- **ADR existants** : ``docs/adr/0001..0044`` (à renuméroter si besoin pour le futur ADR-0045 d'archivage de cette décision)
- **Template RFC** : ``docs/governance/rfc/template.rst``
- **Activity logs Tier 2** : ``docs/agent-activity/2026-05-01-platform-engineer.md``

Annexes
=======

Annexe A : Diagramme cycle de release
--------------------------------------

.. code-block:: text

   Cycle release v0.X.0 :

   1. Dev iteration:        feature/X → dev (CI app green)
   2. Integration:          dev → integration (cascade C3 auto, tests intégration)
   3. Promotion staging:    integration → staging (PR manuelle/Tier2, validation Tier1)
   4. Promotion prod app:   staging → production (PR manuelle/Tier2, env approval Tier1)
   5. (parallèle) Infra:    infra/X → infra-dev → infra-integration → infra-staging → infra-prod
   6. Sync ArgoCD prod:     ArgoCD applique les manifests des branches "production" et "infra-prod"
   7. Tag commun:           v0.X.0 sur production ET infra-prod (workflow GH auto)
   8. Auto-merge main:      production → main + infra-prod → main (sync ArgoCD success)
   9. Cascade C1:           main → 4 envs (snapshot post-release)
   10. Stabilisation:       toutes les branches "released" alignées sur le snapshot

Annexe B : Compatibilité avec règles CLAUDE.md
-----------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 50 40

   * - Règle
     - Énoncé
     - Application dans la RFC
   * - #1
     - Aucun secret en clair
     - Inchangé. Cluster-profiles + helm-values référencent ExternalSecret /
       SealedSecret selon le profil cluster (déjà en place).
   * - #2
     - Pas de prod autonome
     - Auto-merge ``production → main`` n'est PAS un déploiement. ArgoCD sync
       prod reste pilotable uniquement par humain (auto-sync OFF sur
       ``production``). Cf. ApplicationSet ``autoSync: false`` pour ``prod``.
   * - #6
     - Tout dans GitHub
     - RFC versionnée, issue #466 publique, PRs avec template, agent-activity
       logs commités. Aucune décision hors trace.
   * - #11
     - Tier 1 / Tier 2
     - Workflows backport = Tier 2 (auto, loggé). Merges sur ``production`` /
       ``infra-prod`` / ``main`` = Tier 1 (env approval GH). Conformité
       maintenue.
   * - Ligne rouge ``--no-verify``
     - Jamais
     - Workflows backport honorent git hooks via PR (pas de force-push, pas de
       ``--no-verify``).
   * - Ligne rouge ``:latest``
     - Jamais
     - Tag commun ``v<X>.<Y>.<Z>`` (semver), digest pinning recommandé sur
       ``production`` et ``infra-prod``.

---

**Statut Draft → ouvert aux commentaires sur la PR de cette RFC.**
