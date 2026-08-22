---
feature: refonte-ux-multi-role-acp/track-h-bloqueurs
phase: validation
phase_togaf: F (Migration Planning)
agent_bmad: Product Owner (@gilmry)
authors: [Gilles Maury]
date: 2026-06-15
version: 1.0
status: Validated by @gilmry 2026-06-15
signed_at: 2026-06-15
signed_by: "@gilmry"
brief_source: brief.md (Mary, v0.1 Draft 2026-06-15)
prd_source: prd.md (John, v0.1 Draft 2026-06-15)
architecture_source: architecture.md (Winston, v0.1 Draft 2026-06-15)
stories_source: stories.md (Bob, v0.1 Draft 2026-06-15)
parent_feature: docs/maury/refonte-ux-multi-role-acp/validation.md (v1.0 signée 2026-05-20)
changelog:
  - "1.0 (2026-06-15) — VALIDÉE par @gilmry (PO) — bug fix `quota_basis` confirmé, 4 stories autorisées en exécution wave par couche (BE puis FE), Phase 6 (Exécution) Track H ouverte."
---

# Validation Product Owner — Track H bloqueurs légaux v0.1.0

## Méthode Maury — Phase TOGAF F (Migration Planning)

> ✅ **VALIDATION SIGNÉE par @gilmry le 2026-06-15** — Phase 5 verrouillée, Phase 6 (Exécution) débloquée. Agents Track H autorisés en wave par couche BE puis FE conformément au Gantt RGRR `stories.md §Gantt`.

---

## 1. Périmètre validé

Conformité priorisation et budget tokens vis-à-vis :

- **Brief Phase 1** ([brief.md](brief.md)) v0.1 signé 2026-06-15 — 3 personas, 9 capacités CB-H1-9, 9 invariants INV-H1-9 (bug fix `quota_basis` inclus), 9 SC, hors-scope explicite v0.1.0
- **PRD Phase 2** ([prd.md](prd.md)) v0.1 signé 2026-06-15 — 4 FRs (H1+H2+H3+B4), user journeys couvrant cas acte 1000 ET 10000, AC business + 6 NFRs transverses
- **Architecture Phase 3** ([architecture.md](architecture.md)) v0.1 signée 2026-06-15 — types Rust (BuildingNotConformantError + MeetingNotCompletableError + MissingInvariant), 2 `From<>` bridges (AppError + String legacy), 1 nouveau port `MeetingCompletionCheckerPort`, FE pattern `<ConformityBanner>` + `<MissingInvariantsList>`, BDD scenario outline 1000/10000
- **Stories Phase 4** ([stories.md](stories.md)) v0.1 signées 2026-06-15 — 4 stories en 3 waves V1/V2/V3, Gantt RGRR ~3.5j critical path, DoD précis incluant audit grep `git grep "CONFORMANT_QUOTA_TOTAL"` → 0 résultat

---

## 2. Décisions PO

### 2.1 Confirmation bug fix domain `quota_basis`

PO confirme le **bug critique courant** identifié 2026-06-15 :
- Constante `CONFORMANT_QUOTA_TOTAL = dec!(1000)` (building.rs:12) → tous les buildings avec acte de base ≠ 1000 (typiquement 10000 pour lots fractionnés finement) étaient mal classifiés non-conformes.
- Cas de l'immeuble @gilmry à 182 lots / 10000 millièmes : aujourd'hui invisible pour syndic même si parfaitement conforme.
- Story H1 inclut explicitement la suppression de la constante + migration vers `self.total_tantiemes`.

**Pas de migration data nécessaire** : `is_conformant` est pure calcul, pas stocké. Juste le code change.

### 2.2 Priorisation waves (ordre d'exécution confirmé)

| Wave | Story | Couche | Raison de la priorisation PO |
|---|---|---|---|
| **V1** | **H1** — BuildingNotConformantError + bug fix + ConformityBanner | BE+FE atomique (0.5j) | Pré-requis bloquant H2 (l'erreur typée et le banner sont consommés par les 4 use-cases H2). Bug fix domain immédiat. |
| **V2 //** | **H2** — validate-before-compute 4 use-cases | BE-heavy (1.5j) + FE 1j | Dépend H1. Critical path principal. Parallèle docker OK avec H3 (1 BE + 1 FE ≤ mémoire docker-parallelism-bottleneck). |
| **V2 //** | **H3** — Meeting.assert_can_complete + MissingInvariantsList | BE-heavy (1.5j) + FE 1j | Indépendant de H1/H2. Lance en parallèle de H2. Couvre Art. 3.87 §3-5 CC. |
| **V3** | **B4** — Regression spec building-edit-modal | FE only (0.25j) | Aucune dépendance ; spec rapide post-stabilisation. |

**Wall-clock total** : ~3.5j critical path. PO confirme l'ordre wave par couche (BE → FE) décidé 2026-06-15.

### 2.3 Coordination wave par couche (rappel mémoire `docker-parallelism-bottleneck`)

PO confirme la stratégie **« BE wave puis FE wave »** plutôt que fullstack par story :

- **V2 BE** : 1 agent BE-H2 séquentiel sur les 4 use-cases + 1 agent BE-H3 en parallèle (max 2 agents BE concurrent docker compose).
- **V2 FE** : après BE wave verte, 1 agent FE-H2 (4 pages) + 1 agent FE-H3 (MeetingDetail integration).
- **Pas de fullstack par story** : caches docker (`target/`, `.sqlx/`, `.vite/`) sérialiseraient quand même les builds simultanés (mémoire `docker-parallelism-bottleneck`).
- **DoD partagé V2 BE wave** : api.d.ts régénéré (gate Contract Types Check) avant FE wave démarre.

### 2.4 Budget tokens (mémoire `feedback_maury-token-economy`)

PO confirme que **l'investissement upstream Maury pour Track H** (5 docs ~2000 lignes ≈ 80k tokens) est rentable :
- 4 stories self-contained briefables agent fresh, sans contexte session
- DoD précis avec audit grep + lint + CI gate
- Bug fix `quota_basis` capturé en stories.md → impossible à oublier par un agent
- Économie attendue : 1 cycle TDD propre par story sans aller-retour de clarification

### 2.5 Risques résiduels acceptés (cf. brief.md §7)

| Risque | Acceptation PO |
|---|---|
| Cascade refactor `Result<_, String>` → `AppError` explose scope | `From<BuildingNotConformantError> for String` + commentaire hérité — **accepté** |
| Tests BDD `validate_before_compute.feature` lents (16+ scénarios) | Réutilisation WorldBuilder + scénarios partagent setup — **accepté** |
| FE banner casse layouts existants | Composant atomique réutilisable + snapshot Vitest — **accepté** |
| Audit `security_incident` volume rows | v0.1.0 bêta fermée ~10 syndics × ~5 tentatives/mois — négligeable, **accepté** |
| H2 cluster-coord 4 use-cases déraille parallélisme | 1 agent BE séquentiel sur les 4 — **accepté** |
| B4 cassé en prod malgré cartographie OK | Regression spec confirme ; si fail réel → issue séparée — **accepté** |

---

## 3. Autorisation Phase 6 (Exécution) Track H

### 3.1 Création issues GitHub (optionnel)

PO **n'exige pas** de créer 1 issue GH par story Track H (vu petit scope : 4 stories). Suffit :
- 1 commentaire sur issue [#553](https://github.com/gilmry/koprogo/issues/553) confirmant que H1+H2 traitent le scope (Bug 2 conformité)
- 1 commentaire sur issue [#554](https://github.com/gilmry/koprogo/issues/554) confirmant que H3 traite le scope (Bug 1 AG can_complete)
- 1 commentaire sur issue [#553](https://github.com/gilmry/koprogo/issues/553) confirmant que B4 est couvert par regression spec

Si exécution révèle un sub-bug isolé → issue séparée à ce moment.

⚠️ **Commentaires GH = action Tier 2** (logué dans `docs/agent-activity/YYYY-MM-DD-<persona>.md`). Fermeture des issues #553 et #554 = Tier 1, humain `@gilmry` exclusif (mémoire `never-close-issues-autonomously`).

### 3.2 Intégration WBS go-live v0.1.0

PO autorise mise à jour [`docs/WBS_GO_LIVE_v0.1.0.md`](../../../WBS_GO_LIVE_v0.1.0.md) Track H :

- WP-B4 → marquer FAIT (cartographie 2026-06-15 confirme code OK) avec note « regression spec couverte par Story B4 ».
- WP-H1 → enrichir avec bug fix `quota_basis` + `BuildingNotConformantError` typée.
- WP-H2 → confirmer 4 use-cases impactés (expense, call_for_funds, charge_distribution, etat_date).
- WP-H3 → préciser invariants Art. 3.87 §3-5 (convocations, votes, présences, quorum, minutes draft).
- Référence Maury Track H : `docs/maury/refonte-ux-multi-role-acp/track-h-bloqueurs/` (5 docs signés v1.0 2026-06-15).

### 3.3 Démarrage Phase 6 Track H (Exécution)

PO autorise le démarrage de Phase 6 (Exécution) **selon le Gantt waves V1/V2/V3** (cf. §2.2 + `stories.md §Gantt`) :

- **Branches** : `story/H1-buildingnotconformant`, `story/H2-validate-before-compute`, `story/H3-meeting-assert-complete`, `story/B4-building-edit-regression`.
- **1 PR par story** (sauf H2 wave qui peut être 1 PR BE + 1 PR FE selon volume).
- **Gate CI par PR** : Lint + Format + cargo test --lib + cargo test --tests + Frontend Check & Build + Playwright project=chromium (incluant nouvelles specs Track H, pas testIgnore).
- **Sign-off humain @gilmry par PR** (modèle Tier 1) **avant merge**.
- **Log Tier 2** dans `docs/agent-activity/2026-06-15-track-h-bloqueurs.md` (à créer si pas déjà) à chaque session d'exécution.
- **Pas de `git push --force`, pas de `--no-verify`, pas de `gh issue close`** (CRITICAL.md lignes rouges).

### 3.4 DoD Track H global → autorisation promotion feature/dev → dev

PO confirme que la promotion `feature/dev → dev` ne s'effectue qu'après :
- [ ] 4 stories Track H DoD atteints (cf. stories.md).
- [ ] Phase C #617 stabilisée OU continue-on-error/testIgnore conservés (décision @gilmry au moment de la promotion).
- [ ] `make ci` GREEN sur feature/dev avec Track H mergé.
- [ ] Audit Tier 1 humain (revue diff complète + checks legal + tests CI).

⚠️ **Promotion feature/dev → dev = action Tier 1** exécutée par @gilmry, jamais en autonomie agent.

---

## 4. Gate de validation Phase 5 — sign-off humain

> ✅ **Phase 5 VALIDÉE par @gilmry le 2026-06-15** — Phase 6 (Exécution) Track H débloquée.

### 4.1 Critères de validation (tous ✅)

- [x] Brief `brief.md` v0.1 cohérent avec parent feature `refonte-ux-multi-role-acp/brief.md` v1.0 signé 2026-05-20.
- [x] PRD `prd.md` v0.1 user journeys couvrent acte 1000 ET 10000.
- [x] Architecture `architecture.md` v0.1 types Rust et SQL queries documentés ; pattern hexagonal respecté.
- [x] Stories `stories.md` v0.1 self-contained briefables agent fresh ; AC 4-cat exhaustives ; Gantt RGRR explicite.
- [x] Bug fix `quota_basis` propagé partout (brief + prd + architecture + stories + 2 mémoires).
- [x] Mémoires liées créées/mises à jour : `project_quota-basis-acte-de-base` (NEW), `project_admin-publishes-conform-buildings` (UPDATED).
- [x] Sanity check : aucun `dec!(1000)` orphelin dans les drafts (audit grep 2026-06-15).
- [x] Risques résiduels acceptés explicitement (§2.5).
- [x] Cohérence avec WBS Track H (lignes 131-148 + 167-170) préservée.

### 4.2 Signature

```
Mary (Brief)         : SIGNED v1.0 par @gilmry 2026-06-15
John (PRD)           : SIGNED v1.0 par @gilmry 2026-06-15
Winston (Arch)       : SIGNED v1.0 par @gilmry 2026-06-15
Bob (Stories)        : SIGNED v1.0 par @gilmry 2026-06-15
PO (Validation)      : SIGNED v1.0 par @gilmry 2026-06-15
```

**→ Agents Track H autorisés à être briefés et lancés selon le Gantt waves V1 → V2 → V3 (cf. `stories.md §Gantt`).**

---

## 5. Refs

- Parent feature : `docs/maury/refonte-ux-multi-role-acp/validation.md` (v1.0 signée 2026-05-20)
- Mémoires Maury appliquées : `feedback_maury-fullstack-first`, `feedback_maury-token-economy`, `feedback_tdd-bdd-four-categories`, `project_admin-publishes-conform-buildings`, `project_quota-basis-acte-de-base`, `project_validate-before-compute`, `docker-parallelism-bottleneck`, `multirole-narrative-scenarios`, `a11y-wcag-aa-baseline`, `data-testid-systematic`, `no-f64-in-money`.
- WBS Track H : `docs/WBS_GO_LIVE_v0.1.0.md` lignes 131-148 + 167-170.
- Issues : [#553](https://github.com/gilmry/koprogo/issues/553) (Bug 1+2), [#554](https://github.com/gilmry/koprogo/issues/554) (AG complete).
