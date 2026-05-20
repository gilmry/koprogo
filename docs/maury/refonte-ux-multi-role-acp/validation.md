---
feature: refonte-ux-multi-role-acp
phase: validation
phase_togaf: F (Migration Planning)
agent_bmad: Product Owner (@gilmry)
authors: [Gilles Maury, Farah Maury]
date: 2026-05-20
version: 1.0
status: Validated by @gilmry 2026-05-20
signed_at: 2026-05-20
signed_by: "@gilmry"
brief_source: brief.md (Mary, v1.0 signé 2026-05-20)
prd_source: prd.md (John, v1.0 signé 2026-05-20)
architecture_source: architecture.md (Winston, v1.0 signé 2026-05-20)
stories_source: stories.md (Bob, v1.0 signé 2026-05-20)
changelog:
  - "1.0 (2026-05-20) — VALIDÉE par @gilmry (PO) en même temps que sign-off Phase 4. Priorisation 6 slices confirmée, intégration WBS Track H autorisée, création issues GitHub débloquée, Phase 6 (Exécution) ouverte."
---

# Validation Product Owner — Refonte UX multi-rôle + modèle ACP

## Méthode Maury — Phase TOGAF F (Migration Planning)

> ✅ **VALIDATION SIGNÉE par @gilmry le 2026-05-20** — Phase 5 verrouillée, Phase 6 (Exécution dev/qa/release-manager) débloquée. Création issues GitHub + intégration WBS go-live v0.1.0 Track H autorisées.

---

## 1. Périmètre validé

Conformité priorisation et budget tokens vis-à-vis :

- **Brief Phase 1** ([brief.md](brief.md)) v1.0 signé 2026-05-20 — 13 personas, 22 capacités, 27 invariants, 19 SC
- **PRD Phase 2** ([prd.md](prd.md)) v1.0 signé 2026-05-20 — 45 FRs en 8 modules, 16 O, 7 NFRs, 5 slices release
- **Architecture Phase 3** ([architecture.md](architecture.md)) v1.0 signée 2026-05-20 — 8 BC DDD, 6 ADRs inline, 9 migrations SQL, 18 endpoints, 7 composants Svelte 5
- **Stories Phase 4** ([stories.md](stories.md)) v1.0 signées 2026-05-20 — 39 stories en 7 slices (0+5+Tx), 1 story = 1 PR

---

## 2. Décisions PO

### 2.1 Priorisation des slices (ordre d'exécution confirmé)

| Ordre | Slice | Raison de la priorisation PO |
|---|---|---|
| **1** | Slice 0 — Caractérisation FE | Pré-requis bloquant : impossible de refactor sans régression safety net |
| **2** | Slice 1 — Refacto ACP + migration data | Bloqueur légal Art. 3.84 CC + dépendance de tous les BC suivants |
| **3** | Slice 2 — Sélecteur global + bannière + Portfolio + #553 fix | Résout les bugs UX live testing 2026-05-20 + débloque l'ergonomie multi-immeubles |
| **4** | Slice 3 — Sous-rôles + Magic Link + Mandates + Ticketing | Périmètre métier complet contractor / comptable / avocat-notaire-AMO |
| **5** | Slice 4 — Governance hybride + signatures eIDAS | Conformité Art. 3.87-3.88 CC + #48 itsme/eID promu in-scope |
| **6** | Slice 5 — Modularité + onboarding + RBAC Communauté Moderator | Dialectique « gardez ce qui marche, prenez ce qui vous manque » (mémoire [[koprogo-modular-toolbox]]) |
| **Tx** | Slice transversal — Tx.1/Tx.2/Tx.3 | Continuous : CI characterization gate + helpers shared + log Tier 2 — démarrage **immédiat** dès slice 0 |

**Aucun ré-ordonnancement par rapport au PRD §8 + Stories §1.1**. PO confirme la séquence proposée.

### 2.2 Intégration WBS go-live v0.1.0

Conformément à la décision humaine 2026-05-20 (« quand ce sera validé, on créera les issues github qu'on intègrera dans le wbs golive 0.1.0 ») :

- ✅ **Slice 1 (refacto ACP)** → **bloqueur Track H Conformité** (cf. commit `7c24150` qui a déjà préparé Track H)
- ✅ **Slice 2 (sélecteur+banner+#553 fix)** → enrichissement WP-D1 + WP-E1 (déjà préparés par commit `7c24150`)
- ✅ **Slice 3-5** → nouveaux WPs sous Track H (à créer dans WBS lors de la phase 6) :
  - WP-H2 Sous-rôles + Magic Link + Mandates + Ticketing
  - WP-H3 Governance hybride + signatures eIDAS
  - WP-H4 Modularité + onboarding + RBAC Moderator
- ✅ **Slice 0 + Tx** → transverses (pas un WP autonome, dépendance de tous les WPs)

### 2.3 Coordination clusters

PO confirme la convention **« 1 PR par use-case = 2 migrations atomiques »** pour les stories `[cluster-coord]` :

- **#433 Decimal umbrella** → stories 1.4, 3.1, 4.1, 4.8, 4.9 migrent Decimal **dans la même PR** que la refonte. Pas de double churn git.
- **#555 Result&lt;_, String&gt; epic** → stories 3.1, 3.6, 4.1, 4.5, 4.8, 4.9 migrent AppError **dans la même PR** que la refonte.
- **Story 4.9 méga-cluster** : 4 use-cases (expense, call_for_funds, charge_distribution, etat_date) × 2 migrations = 1 PR `[cluster-coord]` atomique pour préserver l'invariant validate-before-compute.

### 2.4 Budget tokens (mémoire [[maury-token-economy]])

PO confirme que **l'investissement upstream Maury (briefing → PRD → architecture → stories) est rentable** :

- 4 documents signés = ~95 pages de directives explicites
- 39 stories avec AC 4-cat + data-testid + files + ADR refs = 0 ambiguïté à l'exécution
- Économie attendue : 1 cycle TDD propre par story (RED test → GREEN code → BLUE refactor) sans aller-retour de clarification

### 2.5 Risques résiduels acceptés (cf. PRD §9)

| Risque | Acceptation PO |
|---|---|
| Migration data ACP casse buildings existants | Mitigation backup obligatoire + dry-run + rollback testé — **accepté** |
| Coordination #433/#555 ratée → double churn | Convention `[cluster-coord]` story 4.9 + meta-comment #433 — **accepté** |
| Coût signature eIDAS prestataire | ADR-0014 abstraction port hexagonal + 3 adapters (eID gratuit par défaut) — **accepté** |
| Performance sélecteur 100+ ACPs | Pagination + debounce + cache LRU — **accepté** |
| PWA Contractor adoption faible | UX 3 écrans + SMS onboarding + analytics — **accepté** |
| Cluster #433 traîne → bloque slice 1 | Démarrer slice 0+1 en parallèle de #433, escalade humain >2 semaines — **accepté** |
| Adoption modulaire SC18 < 20% | Analytics dès release + ajustement messaging — **accepté** |

---

## 3. Autorisation Phase 6 (Exécution)

### 3.1 Création issues GitHub autorisée

Conformément à la décision 2026-05-20, PO autorise la création de :

- **1 Epic GitHub** « Refonte UX multi-rôle + modèle ACP — pipeline Maury » avec frontmatter Maury + liens vers les 4 docs signés
- **39 sous-issues** (1 par story), labellées :
  - `epic-refonte-ux-multi-role-acp` (commun)
  - `slice-N` (N=0..5 + Tx)
  - `cluster-coord` si applicable (5 stories Decimal + 6 Result)
  - `area:backend` / `area:frontend` / `area:tests` / `area:ci` selon files
  - `priority:high` pour slices 0-2 (bloqueurs go-live v0.1.0)
  - `priority:medium` pour slices 3-5
  - `track-h-conformite` pour stories WBS Track H
  - `legal-compliance` pour stories touchant droit belge (Art. 3.84-3.89 CC, eIDAS)

⚠️ **Création des issues = action Tier 1** (mutation publique GitHub) — exécutée **par humain ou via workflow_dispatch** avec validation, jamais en autonomie agent.

### 3.2 Intégration WBS autorisée

PO autorise mise à jour [`docs/WBS_GO_LIVE_v0.1.0.md`](../../WBS_GO_LIVE_v0.1.0.md) :

- Track H Conformité enrichi avec slices 1+2 (déjà préparé par commit `7c24150`)
- 3 nouveaux WPs (H2/H3/H4) ajoutés en Track H pour slices 3/4/5
- Slice 0 (caractérisation) + Tx (CI gate + helpers) référencés en pré-requis transverses
- Mise à jour critères GO v0.1.0 si nécessaire (cf. WBS §critères GO)

### 3.3 Démarrage Phase 6 (Exécution)

PO autorise le démarrage de Phase 6 (Exécution) **dans l'ordre des slices** (cf. §2.1) :

- 1 story = 1 branche `story/<slice>.<n>-<entity>-<action>`
- 1 PR par story (sauf `[cluster-coord]`)
- Gate CI par PR : caractérisation VERT + 4-cat VERT + a11y axe-core ≥ 90 + data-testid lint VERT
- Sign-off humain @gilmry par PR (modèle Tier 1)
- Log Tier 2 dans `docs/agent-activity/YYYY-MM-DD-<persona>.md` à chaque session d'exécution

---

## 4. Gate de validation Phase 5 — sign-off humain

> ✅ **Phase 5 VALIDÉE par @gilmry le 2026-05-20** — Phase 6 (Exécution) débloquée.

- [x] Brief / PRD / Architecture / Stories — tous signés v1.0 le 2026-05-20
- [x] Priorisation 6 slices confirmée (§2.1)
- [x] Intégration WBS Track H autorisée (§2.2)
- [x] Convention `[cluster-coord]` #433/#555 acceptée (§2.3)
- [x] Budget tokens upstream Maury validé (§2.4)
- [x] 7 risques résiduels acceptés (§2.5)
- [x] Création issues GitHub autorisée — Tier 1 humain (§3.1)
- [x] Intégration WBS v0.1.0 autorisée (§3.2)
- [x] Démarrage Phase 6 (Exécution) autorisé (§3.3)

**Date validation** : **2026-05-20**
**Signature PO** : **@gilmry** ✅

---

## 5. Phase suivante

Phase 6 (Exécution — dev / qa / release-manager) débloquée. Première action :

1. **Création issues GitHub** (Tier 1 — humain ou workflow_dispatch) : 1 Epic + 39 sous-issues
2. **Mise à jour WBS** Track H (slices 1+2 déjà préparées, ajouter WPs H2/H3/H4 pour slices 3-5)
3. **Démarrage slice 0** (caractérisation FE) — story 0.1 en branche `story/0.1-characterization-suite`
4. **Tx.1/Tx.2** démarrent en parallèle (continuous gate CI + helpers shared)

---

## 6. Liens

- Brief Phase 1 : [`brief.md`](brief.md)
- PRD Phase 2 : [`prd.md`](prd.md)
- Architecture Phase 3 : [`architecture.md`](architecture.md)
- Stories Phase 4 : [`stories.md`](stories.md)
- README pipeline : [`README.md`](README.md)
- WBS go-live v0.1.0 : [`../../WBS_GO_LIVE_v0.1.0.md`](../../WBS_GO_LIVE_v0.1.0.md)

🤖 Validation rédigée par Product Owner persona — signature Tier 1 @gilmry apposée.
