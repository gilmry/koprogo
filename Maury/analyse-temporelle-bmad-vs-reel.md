# Analyse Temporelle — Plan BMAD vs Développement Réel KoproGo

## Méthode Maury — Confrontation Sprint Plan vs Historique Git

**Date** : 29/03/2026
**Contexte** : Solo-dev (Gilles Maury), emploi salarié, 6-15h/semaine variable
**Données** :
- Repo principal (`koprogo`) : 1 057 commits, 138k LOC Rust + 10k LOC infra
- Repo infrastructure (`koprogo-infra-restructure`) : 920 commits, 8.4k LOC IaC
- **Total** : ~1 977 commits, du 24/08/2025 au 29/03/2026 (7 mois, 5 jours)
- **14 semaines actives** sur 31 calendaires (55% du temps = 0 commits)
- **L'infra n'est pas encore testée** (pas de tests automatisés IaC)

---

## 1. Chronologie réelle (git log)

### Distribution mensuelle des commits

| Mois | Commits | % total | Phase réelle |
|------|---------|---------|-------------|
| Août 2025 | 1 | 0.1% | Initial commit |
| Oct 2025 | 352 | 33.3% | Sprint 0 + Sprint 1-2 (architecture + core domain + IaC + CI/CD) |
| Nov 2025 | 342 | 32.4% | Sprint 2-5 (AG, comptabilité, paiements, tickets, GDPR, SEL, gamification, polls) |
| Déc 2025 | 39 | 3.7% | Stabilisation, energy campaigns, hotfixes |
| Jan 2026 | 0 | 0% | **Pause complète** |
| Fév 2026 | 53 | 5.0% | Frontend features (budgets, états dates, charge distribution) |
| Mars 2026 | 270 | 25.5% | BDD scenarios, E2E, Documentation Vivante, contractor reports, i18n |

### Phases d'intensité

| Période | Durée | Commits | Rythme | Nature |
|---------|-------|---------|--------|--------|
| **Sprint 0** : 22 oct 2025 | **1 jour** | ~50 | 50/jour | Architecture hexagonale + IaC + CI/CD entier |
| **Burst 1** : 22 oct - 21 nov | **30 jours** | ~694 | 23/jour | Backend complet (60 entités, 559 endpoints) |
| **Ralentissement** : déc 2025 - jan 2026 | **60 jours** | 39 | 0.65/jour | Stabilisation + pause |
| **Burst 2** : fév - 29 mars 2026 | **50 jours** | 323 | 6.5/jour | Frontend, tests, BDD, E2E, i18n, docs |

### Profil du développeur solo — Heures effectives

**Contexte** : Gilles Maury, emploi salarié à temps plein, KoproGo en side-project.
Disponibilité variable : 6h/semaine (semaines light) à 15h+ (semaines burst).
Horaires typiques : soirées (18h-01h) + week-ends. Commits entre minuit et 3h fréquents.

| Semaine | Commits | Profil | Heures estimées | Jours actifs |
|---------|---------|--------|-----------------|--------------|
| W34 (août) | 1 | Initial | ~1h | 1 |
| W43 (oct) | 194 | BURST | ~20h | 5 |
| W44 (oct) | 176 | BURST | ~18h | 7 |
| W45 (nov) | 89 | BURST | ~15h | 4 |
| W46 (nov) | 52 | BURST | ~15h | 6 |
| W47 (nov) | 172 | BURST | ~18h | 5 |
| W48 (nov) | 11 | LIGHT | ~6h | 2 |
| W49 (déc) | 39 | NORMAL | ~10h | 2 |
| W08 (fév) | 33 | NORMAL | ~10h | 3 |
| W09 (fév) | 22 | NORMAL | ~10h | 3 |
| W10 (mars) | 43 | INTENSE | ~12h | 6 |
| W11 (mars) | 54 | BURST | ~15h | 5 |
| W12 (mars) | 21 | NORMAL | ~10h | 4 |
| W13 (mars) | 151 | BURST | ~18h | 6 |
| **TOTAL** | **1 057** | | **~178h** | **59 jours** |

**Heures infra (repo séparé, mêmes semaines)** : ~80h estimées (920 commits, 54 jours actifs)

**Total projet KoproGo** : **~258h** de dev effectif sur 14 semaines actives.
Soit **~18.4h/semaine active** en moyenne (mais 0h les 17 semaines inactives).

**Rythme réel lissé sur les 31 semaines calendaires** : ~8.3h/semaine.

```
PROFIL RÉEL SOLO-DEV AVEC EMPLOI SALARIÉ
═════════════════════════════════════════

Semaines         31 calendaires
Semaines actives 14 (45%)
Semaines à 0h    17 (55%) — emploi salarié, repos, vie perso
Heures totales   ~258h (app ~178h + infra ~80h)
Moyenne/sem act. ~18.4h (burst le soir + WE)
Moyenne lissée   ~8.3h/semaine
Commits/heure    ~7.7 (assisté par IA)

Répartition horaire des commits :
  00h-03h  ████████████████████  23% (nuit)
  04h-08h  ████                   6% (matin tôt)
  09h-12h  ████████              12% (matinée — probablement WE)
  13h-17h  ████████████████      19% (après-midi — WE ou congés)
  18h-23h  ████████████████████████████  40% (soirées — créneau principal)
```

---

## 2. Plan BMAD : estimation Scrum (sprints de 2 semaines)

Le document `epics-and-stories.md` planifie 7+ sprints :

| Sprint BMAD | Stories | Tailles | Durée prévue (2 sem) | Contenu |
|-------------|---------|---------|---------------------|---------|
| Sprint 0 | 8 stories tech | S-M | 2 semaines | Architecture, CI/CD, Docker, IaC, Monitoring |
| Sprint 1 | STORY-001 à 003 | M, L, L | 2 semaines | Buildings, Units/Owners, Auth multi-rôle |
| Sprint 2 | STORY-004, 005, 008, 009 | L, L, L, M | 2 semaines | AG (meetings, votes), PCMN, Budget |
| Sprint 3 | STORY-011 à 014 | L, L, M, M | 2 semaines | Expenses workflow, Payments Stripe/SEPA, Reminders, Call for Funds |
| Sprint 4 | STORY-015 à 017 | L, L, L | 2 semaines | GDPR (5 articles), Tickets, Quotes |
| Sprint 5 | STORY-018 à 021 | L, L, M, L | 2 semaines | SEL, Polls, Notifications, Gamification |
| Sprint 6 | STORY-006, 007, 010, 022, 023 | M, M, M, M, M | 2 semaines | AGE, Visio AG, État date, Board, Documents |
| Sprint 7+ | STORY-024 à 027 | L, M, M, S | 2 semaines | Energy, IoT, SuperAdmin, Dashboard |

**Durée BMAD prévue : 8 sprints × 2 semaines = 16 semaines = 4 mois**

---

## 3. Comparaison directe

### 3.1 Sprint 0 — Fondations

| | Plan BMAD | Réel |
|--|-----------|------|
| **Durée** | 2 semaines | **1 jour** (22 oct 2025) |
| **Contenu** | Architecture Rust+Hexa, BDD, Docker, CI, Astro, IaC, Monitoring | Tout fait en un seul commit massif + CI/CD |
| **Écart** | | **13 jours d'avance** |

**Analyse** : L'agent IA (Claude) a généré le Sprint 0 entier en quelques heures. Un développeur humain solo aurait effectivement mis 2 semaines. Le modèle BMAD calibre pour des humains, pas pour des agents IA.

### 3.2 Sprint 1-2 — Core Domain + AG + Comptabilité

| | Plan BMAD | Réel |
|--|-----------|------|
| **Durée** | 4 semaines (Sprint 1 + 2) | **~16 jours** (22 oct - 7 nov) |
| **Contenu prévu** | Buildings, Owners, Auth, Meetings, Votes, PCMN, Budget | Buildings, Owners, Auth, Board, PCMN, Invoice workflow |
| **Contenu réel additionnel** | — | Business plan, roadmap, governance ASBL, gap analysis |
| **Écart** | | **~12 jours d'avance** |

**Analyse** : Le réel inclut du travail docs/stratégie non prévu par BMAD. Les features core ont été livrées plus vite que prévu, mais l'ordre diffère : Board avant Voting, Invoice avant Meetings.

### 3.3 Sprint 3-5 — AG Voting + Payments + Tickets + GDPR + SEL + Gamification + Polls

| | Plan BMAD | Réel |
|--|-----------|------|
| **Durée** | 6 semaines (Sprint 3 + 4 + 5) | **~11 jours** (7 nov - 18 nov) |
| **Contenu** | Expenses, Payments, Reminders, GDPR, Tickets, SEL, Polls, Notifications, Gamification | **TOUT** livré en 11 jours |
| **Écart** | | **~31 jours d'avance** (!)  |

**Analyse** : C'est l'écart le plus spectaculaire. Le 17 novembre 2025 est le "jour monstre" : Tickets, Notifications, Payments, Convocations, GDPR, Quotes, SEL en un seul jour (~30 commits). Le 18 nov : Polls, 2FA, IoT. BMAD prévoyait 6 semaines pour ce volume — il a été fait en moins de 2 semaines. Vélocité agent IA ≈ 4-5x celle d'un développeur humain.

### 3.4 Sprint 6-7 — Compléments + Post-MVP

| | Plan BMAD | Réel |
|--|-----------|------|
| **Durée** | 4 semaines (Sprint 6 + 7) | **~60 jours** (nov 2025 - mars 2026) avec pauses |
| **Contenu prévu** | AGE, Visio, État date, Board, Documents, Energy, IoT, SuperAdmin | Energy campaigns, frontend, BDD 146 scenarios, E2E 12 scenarios, i18n, contractor reports |
| **Écart** | | **~30 jours de retard** (mais inclut une pause de 30j en janvier) |

**Analyse** : La phase finale (frontend, tests, polish) a pris beaucoup plus longtemps que prévu par BMAD. Raisons :
- **Pause complète en janvier 2026** (0 commits)
- Le frontend est plus lent que le backend (plus de corrections, de cycles debug)
- L'E2E multi-rôles (Documentation Vivante) est un travail itératif non prévu par BMAD
- i18n (648 clés manquantes) et Contractor Reports (BC16) sont des modules émergents

### 3.5 Infrastructure — La couche oubliée par BMAD

| | Plan BMAD (ancien) | Réel |
|--|---------------------|------|
| **Stories prévues** | STORY-T06 (Terraform) + STORY-T07 (Ansible) + STORY-T08 (Monitoring) = **3 stories M** | **Un repo entier** |
| **Estimation BMAD** | ~2-3 jours dans Sprint 0 | **Effort continu sur 5 mois** |
| **Commits infra** | Non comptés | **920 commits** (repo dédié) + 113 commits (repo principal) = **1 033 commits infra** |
| **LOC IaC** | Non estimées | **18 770 LOC** (8 372 repo infra + 10 370 repo principal) |
| **Fichiers IaC** | Non comptés | **236 fichiers** (39 Terraform, 47 Ansible YAML, 21 templates J2, 23 Helm, 23 Kustomize, 36 scripts, 6 workflows, 16 monitoring, 20 Dockerfiles) |

**Décomposition de l'effort infra réel** :

| Catégorie | LOC | Fichiers | Rôles/Modules | Commits estimés |
|-----------|-----|----------|---------------|-----------------|
| Terraform (provisioning) | 989 | 39 + 12 tfvars | 4 modules (ovh-vps, ovh-k3s, ovh-k8s, networking) | ~50 |
| Ansible (security+deploy) | 3 033 | 47 YAML + 21 J2 | 14 rôles (security, monitoring, backup, k3s, argocd, vault, velero, pgo, dns, common, docker, gitops, hardening) | ~200 |
| Helm (packaging) | 949 | 23 | 4 charts (koprogo, monitoring, vault, velero) | ~30 |
| Kustomize (overlays) | 352 | 23 | base + 4 env overlays | ~20 |
| Scripts (automation) | 4 902 | 36 | setup, backup, restore, deploy, load-test | ~100 |
| CI/CD (workflows) | 841 | 6 | ci, security, docker-build, docs | ~50 |
| Monitoring configs | 1 085 | 16 | Prometheus, Grafana dashboards, Loki, Alertmanager, Elasticsearch, Kibana, Filebeat, Elastalert | ~40 |
| Docker (dev+prod) | ~600 | 20 | Dockerfile.dev, Dockerfile.prod, docker-compose multi-env | ~30 |

**Chronologie infra** :
- **22 oct 2025** : Premier IaC (Docker Compose + CI/CD) — même jour que Sprint 0 backend
- **23-26 oct** : VPS monitoring, ArgoCD, Ansible deploy, Terraform (≈50 commits en 4 jours)
- **4 nov** : Stack sécurité production complète (Suricata, CrowdSec, fail2ban, LUKS, audits) — Issue #39-43
- **Nov-Déc 2025** : Stabilisation, corrections Ansible (permissions, git safe dir, health checks)
- **Fév-Mars 2026** : Restructuration infra (repo séparé), Helm charts, Kustomize, multi-env

**Verdict infra** : Le plan BMAD initial sous-estimait **massivement** l'infra. 3 stories M dans Sprint 0 vs un effort qui représente **~52% des commits totaux** (1 033 / 1 977). L'infra est le **plus gros chantier du projet** en volume de commits, mais le plan la traitait comme un appendice.

**Point critique** : L'infra n'est pas encore testée (pas de `terraform validate` en CI, pas de `ansible-lint`, pas de `conftest` pour policy-as-code ISO 27001). C'est une dette technique identifiée.

---

## 4. Synthèse temporelle (CORRIGÉE avec infra)

| Métrique | Plan BMAD (ancien) | Réel | Ratio |
|----------|-----------|------|-------|
| **Durée calendaire** | 16 semaines (4 mois) | 31 semaines (7 mois) | 1.9x plus long |
| **Semaines actives** | 16 (100% temps plein) | **14** (45% du temps) | 0.88x |
| **Heures effectives** | ~640h (16 sem × 40h) | **~258h** (178h app + 80h infra) | **2.5x moins** |
| **Sprint 0 (fondations)** | 80h (2 sem × 40h) | **~20h** (1 jour burst) | **4x moins** |
| **Backend complet** | 400h (10 sem × 40h) | **~100h** (~30 jours × 3.3h/jour) | **4x moins** |
| **Frontend + Tests** | 160h (4 sem × 40h) | **~58h** (~50 jours × 1.2h/jour) | **2.8x moins** |
| **Infrastructure IaC** | ~12h (3 stories M) | **~80h** (920 commits, 54 jours actifs) | **6.7x plus** |
| **Commits totaux** | Non estimé | **1 977** (1 057 app + 920 infra) | — |
| **LOC totales** | ~138k (backend seul) | **~211k** (138k Rust + 18.7k IaC + reste) | +53% non comptés |
| **Tests IaC** | Non prévus | **0** (dette technique) | ∞ |
| **Productivité** | ~0.29 LOC/h (humain classique) | **~0.82 LOC/h** (assisté IA) | **2.8x plus productif** |

### Diagramme temporel comparé

```
PLAN BMAD ANCIEN (16 semaines / 4 mois — infra ignorée)
═══════════════════════════════════════════════════════════════════

Oct         Nov         Déc         Jan         Fév
├─Sprint 0──┤Sprint 1-2─┤Sprint 3-5──┤Sprint 6-7──┤ FIN ✓
  (3 stories infra M, c'est tout)

RÉEL (31 semaines / 7 mois — incluant infra)
═══════════════════════════════════════════════════════════════════

Oct         Nov         Déc         Jan         Fév         Mars
├─S0 (1j)──┤                                                     │
├──────BACKEND BURST (30j, 694 commits)────┤                     │
│           ├─ 17 nov: "jour monstre" (30 commits, 8 modules)   │
│                       ├──PAUSE──┤         ├──PAUSE──┤          │
│                                           ├──FRONTEND + TESTS──┤
│                                           ├──BDD 146 scénarios─┤
│                                           ├──E2E + i18n + docs─┤
│                                                                 FIN ✓
│
├══════════════════ INFRA EN CONTINU (1033 commits) ═════════════┤
│ 22oct: IaC+CI  │ 4nov: Sécurité   │ Fév: Restructure │ Non testé│
│ Docker+Ansible  │ LUKS+Suricata    │ Repo séparé      │ Dette!  │
│ Terraform+ArgoCD│ CrowdSec+fail2ban│ Helm+Kustomize   │         │
└═════════════════════════════════════════════════════════════════┘
  ^-- L'infra est le fil rouge INVISIBLE qui traverse tout le projet
```

---

## 5. Conclusions

### Ce que BMAD prédit correctement

1. **L'ordre des dépendances est correct** : Sprint 0 → Core Domain → AG/Comptabilité → Payments → GDPR → Community. Le réel suit globalement cet ordre (sauf Board avant Voting).

2. **Le découpage en epics/stories est cohérent** : Chaque epic correspond à un bounded context réel, les tailles relatives (L pour Payments, M pour Documents) reflètent la complexité réelle.

3. **Le périmètre fonctionnel est précis à 85%** : Les 27 stories couvrent bien les 60 entités réelles (voir `analyse-bmad-vs-codebase.md`).

### Ce que BMAD ne prédit PAS

1. **L'infrastructure est un projet dans le projet** : BMAD traitait l'IaC comme 3 stories M dans Sprint 0 (~3 jours). La réalité : **1 033 commits**, **18.7k LOC**, **236 fichiers**, 14 rôles Ansible, 4 modules Terraform, 4 Helm charts, un repo entier dédié, et **5 mois de travail continu**. L'infra représente **52% des commits totaux** du projet. C'est la couche la plus sous-estimée. De plus, **l'infra n'est pas encore testée** — pas de terraform validate en CI, pas d'ansible-lint, pas de conftest pour ISO 27001. C'est une dette technique majeure.

2. **La vélocité agent IA** : BMAD calibre pour des sprints humains de 2 semaines. Un agent IA code le Sprint 0 en 1 jour et 3 sprints de backend en 11 jours. Le modèle Scrum 2 semaines est **trop conservateur** pour du dev assisté IA. Il faudrait un coefficient IA (÷3 à ÷5 pour le backend).

3. **Le "long tail" frontend/tests/polish** : BMAD sous-estime massivement la phase finale. Le backend est le "gros œuvre" (rapide avec IA), mais le frontend, les tests E2E multi-rôles, l'i18n, et la Documentation Vivante prennent **3x plus longtemps** que prévu. C'est le travail artisanal que l'IA accélère moins.

4. **Les pauses et le rythme irrégulier** : BMAD suppose un rythme constant (sprint régulier). Le réel montre des bursts intenses (23 commits/jour en octobre-novembre) puis des pauses complètes (janvier). Le modèle Scrum continu ne capture pas la réalité d'un développeur solo avec emploi salarié.

5. **Les modules émergents** : Contractor Reports (BC16), Documentation Vivante, i18n massif, 146 scénarios BDD multi-rôles — tout cela est du travail non planifié qui a émergé du cycle Sprint Review → Rétro → nouveau besoin. C'est exactement la Boucle 1 de la Méthode Maury en action.

6. **Le "jour monstre"** : Le 17 novembre 2025 (Tickets + Notifications + Payments + Convocations + GDPR + Quotes + SEL en un jour) n'est pas modélisable par Scrum. C'est un pattern spécifique au dev IA : l'agent génère des modules entiers en quelques heures quand le pattern hexagonal est maîtrisé.

### Facteur de correction proposé

Pour les futures estimations BMAD avec agents IA :

| Phase | Coefficient IA | Justification |
|-------|---------------|---------------|
| Sprint 0 (fondations) | ÷ 10 | L'IA génère la structure en heures, pas en jours |
| Backend (domain + API) | ÷ 3 à 5 | Pattern hexagonal reproductible, IA très efficace |
| Frontend (composants) | ÷ 1.5 | L'IA aide mais le debug UI reste itératif |
| Infrastructure (IaC) | × 3 à 5 | **L'angle mort majeur** : debug Ansible/Terraform sur VPS réel, permissions, réseau, secrets, SSL — pas automatisable par IA |
| Tests IaC (non encore faits) | **∞** | Pas de tests infra = dette technique non planifiable |
| Tests E2E (Documentation Vivante) | × 2 | Multi-rôles, debugging navigateur, timing |
| i18n / Polish / Docs | × 1.5 | Travail d'intégration que BMAD ne modélise pas |
| **Coefficient global** | **× 1.2** | L'infra annule le gain du backend rapide |

### Formule révisée EN HEURES (pour solo-dev avec IA)

La formule en semaines est trompeuse pour un solo-dev à temps partiel.
**Estimer en heures**, puis diviser par le rythme réel.

```
ÉTAPE 1 : Estimer les heures par couche (BMAD stories → heures)

  Story S = 4h | Story M = 8h | Story L = 16h (baseline humain)
  Puis appliquer les coefficients IA :

  Backend (domain + API) : heures baseline ÷ 3
  Frontend (composants)  : heures baseline ÷ 1.5
  Infrastructure (IaC)   : heures baseline × 1.5 (plus long que prévu, debug VPS réel)
  Tests E2E multi-rôles  : heures baseline × 2
  i18n / Polish / Docs   : heures baseline × 1.5
  + 20% émergence

ÉTAPE 2 : Convertir en durée calendaire

  Heures totales ÷ rythme hebdomadaire moyen = semaines calendaires

  Rythmes solo-dev observés sur KoproGo :
  - Mode burst (emploi + 15h/sem KoproGo) : ~15h/sem
  - Mode normal (emploi + 10h/sem)        : ~10h/sem
  - Mode light (emploi chargé, 6h/sem)    : ~6h/sem
  - Mode pause (vacances, fatigue, vie)    : 0h/sem

  Rythme moyen réaliste lissé : ~8h/semaine
  Taux d'activité : ~45% des semaines (55% à 0 commit)
```

**Application à KoproGo** :

```
Heures estimées BMAD (corrigées avec coefficients IA) :
  Sprint 0 (13 stories S-M)      : ~50h baseline → ÷3 IA    = ~17h  (réel ~20h ✓)
  Backend (20 stories M-L)        : ~240h baseline → ÷3 IA   = ~80h  (réel ~100h, ~OK)
  Frontend (stories implicites)   : ~80h baseline → ÷1.5 IA  = ~53h  (réel ~40h ✓)
  Infra (estimé 3 stories M=24h) : ~24h baseline → ×1.5 infra = ~36h (réel ~80h ✗ ×2.2)
  Tests E2E + BDD                 : ~40h baseline → ×2        = ~80h  (réel ~18h ? sous-estimé car intégré au backend)
  + 20% émergence                                             = +53h

  Total estimé corrigé : ~319h
  Total réel mesuré    : ~258h

  → La formule corrigée est légèrement pessimiste (+24%)
  → L'infra reste sous-estimée (36h estimé vs 80h réel)
  → Coefficient infra devrait être ×3 (pas ×1.5) pour un projet ISO 27001
```

**Formule finale pour les prochains projets solo-dev** :

```
Heures = Σ (stories × baseline_heures × coeff_IA) + (infra × 3) + 20% émergence
Semaines calendaires = Heures ÷ 8 (rythme moyen solo-dev salarié)
Mois calendaires = Semaines ÷ 4.3

Exemple pour un projet équivalent KoproGo :
  ~300h ÷ 8h/sem = ~38 semaines = ~9 mois calendaires
  KoproGo réel : 258h sur 31 semaines (7 mois) → cohérent (les bursts accélèrent)
```

### Répartition réelle de l'effort par couche

```
COMMITS PAR COUCHE (total ~1 977)
═════════════════════════════════

  Infrastructure (IaC+CI/CD+Monitoring+Security)
  ████████████████████████████████████████████████████  52% (1 033)

  Backend (Domain+Application+Infrastructure code)
  ██████████████████████████████████  35% (694)

  Frontend (Astro+Svelte+i18n)
  █████████  9% (~180)

  Tests (BDD+E2E+Documentation Vivante)
  ████  4% (~70)

  → L'infra est le PLUS GROS poste, pas le backend
  → Le plan BMAD original l'estimait à <2% (3 stories sur 48)
```

---

## 6. Verdict final

**Le pipeline BMAD produit un plan fonctionnellement juste (85% du périmètre applicatif) mais ignore la couche infrastructure qui représente 52% de l'effort réel.**

| Dimension | Note | Commentaire |
|-----------|------|-------------|
| Ordre des dépendances | 9/10 | Très bon, l'ordre réel suit le plan à 90% |
| Découpage stories backend | 8/10 | Cohérent, tailles relatives correctes |
| Estimation durée backend | 3/10 | Trop conservateur (÷3 à ÷5 avec IA) |
| Estimation durée frontend/tests | 4/10 | Trop optimiste (× 2 à 3 en réalité) |
| **Estimation durée infra** | **1/10** | **3 stories M vs 1 033 commits réels — angle mort majeur** |
| Couverture tests IaC | 0/10 | Pas de tests infra prévus ni réalisés — dette technique |
| Prédiction du périmètre applicatif | 8.5/10 | 85% capturé, 15% émergent |
| Prédiction du périmètre infra | 2/10 | Mentionné mais sous-dimensionné de 50x |
| **Score global estimation temporelle** | **4/10** | Le plan ignore la moitié du projet |

**Leçon principale** : Dans un projet full-stack ISO 27001, l'infrastructure n'est pas un "appendice" du Sprint 0 — c'est **la moitié du travail**. Le pipeline BMAD v1 était backend-centric. La v2 (corrigée) traite l'IaC, la CI/CD, le monitoring et la sécurité comme des couches à part entière avec des stories dédiées, des sprints dédiés, et des tests dédiés.

**Ce qui manque encore** : des tests IaC (terraform validate, ansible-lint, molecule, conftest pour policy-as-code ISO 27001). Tant que l'infra n'est pas testée, la boucle TDD est incomplète — on a du "code as infrastructure" sans le "test" de TDD.

**La force de BMAD reste le plan (quoi faire), pas le timing (combien de temps).** Et c'est cohérent avec la philosophie de la Méthode Maury : *"We deliver when ready, not according to arbitrary dates."*

---

## 7. Guide d'estimation pour les prochains projets solo-dev

### Contexte type

- Solo-développeur avec emploi salarié
- Assisté par agents IA (Claude Code, ChatGPT pour BMAD)
- Side-project en soirées + week-ends
- Rythme variable : 6-15h/semaine, avec pauses de plusieurs semaines
- Architecture : SOLID + DDD + Hexagonal + BDD + TDD (Méthode Maury)
- Full-stack : Backend + Frontend + IaC + CI/CD + Monitoring + Sécurité

### Grille d'estimation calibrée sur KoproGo

| Couche | Heures/story S | Heures/story M | Heures/story L | Coeff IA |
|--------|---------------|---------------|---------------|----------|
| Backend (domain + API) | 1.5h | 3h | 5h | ÷3 vs humain |
| Frontend (composants + pages) | 3h | 5h | 10h | ÷1.5 vs humain |
| Infrastructure (IaC, Ansible, Terraform, Helm) | 4h | 8h | 20h | ×1 (pas de gain IA, debug VPS réel) |
| CI/CD (workflows, hooks, audit) | 2h | 4h | 8h | ÷2 vs humain |
| Tests E2E (Playwright multi-rôles) | 4h | 8h | 16h | ×1 (pas de gain IA, timing navigateur) |
| Tests BDD (Gherkin + steps) | 1h | 2h | 4h | ÷2 vs humain |
| i18n (traductions, extraction) | 2h | 4h | 8h | ÷1.5 vs humain |
| Documentation | 1h | 2h | 4h | ÷2 vs humain |

### Formule complète

```
1. Lister les stories BMAD par couche
2. Heures = Σ (nb_stories × heures_par_taille)
3. Ajouter +20% émergence (modules découverts en cours de dev)
4. Ajouter +10% stabilisation CI (corrections fmt, clippy, audit, CI rouge)
5. Durée calendaire = Heures ÷ rythme_hebdo_moyen

   Rythmes solo-dev :
   ┌──────────────────────────────────────────────────┐
   │ Mode burst   : 15h/sem (vacances, motivation)    │
   │ Mode normal  : 10h/sem (soirées + 1 jour WE)     │
   │ Mode light   : 6h/sem  (emploi chargé)            │
   │ Mode pause   : 0h/sem  (vie, fatigue, blocage)    │
   │                                                    │
   │ Moyenne lissée réaliste : 8h/semaine               │
   │ Taux d'activité réaliste : 45% des semaines        │
   └──────────────────────────────────────────────────┘
```

### Exemples calibrés

| Taille projet | Stories backend | Stories infra | Heures estimées | Durée calendaire (8h/sem) |
|--------------|----------------|---------------|-----------------|--------------------------|
| **Micro** (1 BC, API simple) | 3 stories M | 2 stories M | ~50h | ~6 semaines (~1.5 mois) |
| **Petit** (3-5 BC, auth, CRUD) | 10 stories M-L | 5 stories M | ~120h | ~15 semaines (~3.5 mois) |
| **Moyen** (10 BC, multi-rôle, paiements) | 20 stories M-L | 8 stories M-L | ~250h | ~31 semaines (~7 mois) ← KoproGo |
| **Grand** (20+ BC, multi-tenant, ISO 27001) | 40 stories M-L | 15 stories M-L | ~500h | ~63 semaines (~15 mois) |

### Pièges identifiés (leçons KoproGo)

1. **L'infra est invisible dans le plan mais consomme 30-50% des heures**
   - Terraform : ça marche en local mais debug OVH API = 3x plus long
   - Ansible : les permissions Linux, git safe dir, health checks = jours entiers
   - Monitoring : Prometheus scrape config, Grafana dashboards = sous-estimé
   - Sécurité : chaque rôle (Suricata, CrowdSec, fail2ban) = 1 jour minimum de debug

2. **Les pauses sont normales et doivent être planifiées**
   - 55% des semaines à 0 commit sur KoproGo. C'est pas de la procrastination, c'est la réalité d'un side-project avec emploi.
   - Multiplier la durée calendaire par 2 pour obtenir la date réaliste.

3. **Le backend va vite, le reste non**
   - L'IA excelle sur le pattern hexagonal reproductible (÷3 à ÷5)
   - Le frontend, les tests E2E, et l'infra ne bénéficient pas du même gain
   - Budget temps réaliste : 35% backend, 15% frontend, 30% infra, 20% tests/docs

4. **Les bursts ne sont pas planifiables**
   - Les semaines à 20h arrivent quand la motivation est là, pas quand le Gantt le dit
   - "We deliver when ready, not according to arbitrary dates"

---

*Analyse réalisée le 29/03/2026 — 1 977 commits analysés (app + infra), ~258h de dev effectif*
