=============================================================================================
Issue #556: [EPIC] Refonte UX multi-rôle + modèle ACP — pipeline Maury (39 stories, 7 slices)
=============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,priority:high legal-compliance,governance epic,maury track-h-conformite
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-06-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/556>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Refonte UX multi-rôle + modèle ACP — pipeline Maury
   
   > ⚠️ **Correction 2026-05-20** : compte de stories rectifié de 31 → **39** (audit post-création batch slice 0+1). Le total réel par slice est : 1+4+5+9+9+8+3 = 39.
   
   ## Contexte
   
   Refonte FE conséquente issue de la **session live testing 2026-05-20** révélant 5 problèmes structurels :
   
   1. Pas de sélecteur d'immeuble global → erreurs de portée syndic multi-immeubles
   2. Modèle juridique faux — `Building.organization_id` saute le niveau ACP (Art. 3.84 CC)
   3. RBAC Communauté incohérent — conflit d'intérêt syndic participant
   4. Ticketing immature — plaintes/réponses/évaluations Contractor non tracées
   5. Monolithe vs boîte à outils — pas de modularité par ACP
   
   Pilotée par la **Méthode Maury** ([`docs/maury/refonte-ux-multi-role-acp/`](docs/maury/refonte-ux-multi-role-acp/)) avec gates humains de signature à chaque phase.
   
   ## Pipeline Maury — état au 2026-05-20
   
   | Phase | Document | Agent | Statut |
   |---|---|---|---|
   | 1 — Brief | [`brief.md`](docs/maury/refonte-ux-multi-role-acp/brief.md) | Mary (Analyste TOGAF) | ✅ SIGNÉ par @gilmry 2026-05-20 (v1.0) |
   | 2 — PRD | [`prd.md`](docs/maury/refonte-ux-multi-role-acp/prd.md) | John (PM) | ✅ SIGNÉ par @gilmry 2026-05-20 (v1.0) |
   | 3 — Architecture | [`architecture.md`](docs/maury/refonte-ux-multi-role-acp/architecture.md) | Winston (Architecte hexagonal) | ✅ SIGNÉE par @gilmry 2026-05-20 (v1.0) |
   | 4 — Stories | [`stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) | Bob (Scrum Master) | ✅ SIGNÉES par @gilmry 2026-05-20 (v1.0) |
   | 5 — Validation | [`validation.md`](docs/maury/refonte-ux-multi-role-acp/validation.md) | PO @gilmry | ✅ VALIDÉE par @gilmry 2026-05-20 (v1.0) |
   | **6 — Exécution** | (PRs + commits) | dev / qa / release-manager | 🟢 **Ready to start** |
   
   ## Périmètre
   
   - **13 personas** métier (Mathilde Admin, Sylvie Syndic, Pierre/Paul Comptable, Marie Owner, Catherine CdC, Henri Commissaire, Bruno Contractor, Anne AMO, Léa Avocat, Sophie Notaire, Karim Gardien, Architect/BET)
   - **22 capacités** (C1-C22)
   - **27 invariants métier** (INV-1 à INV-27)
   - **45 FRs** en 8 modules (Identity+ACP, Property, Governance, Accounting, Community, Maintenance, Portfolio, Cross-cutting)
   - **6 ADRs inline** dans `architecture.md` (ADR-0010 à ADR-0015) :
     - ADR-0010 ACP comme racine d'agrégat distincte d'Organization
     - ADR-0011 Portefeuille entité backend (vs UI préférence localStorage)
     - ADR-0012 Convention `data-testid="<entity>-<action>"` systématique
     - ADR-0013 Arborescence tests caractérisation + refonte (3 niveaux)
     - ADR-0014 Signature électronique eIDAS (eID belge / itsme / Universign)
     - ADR-0015 Modularité par ACP — module registry + `ModuleGuard` middleware
   
   ## 39 stories en 7 slices
   
   | Slice | Nom | Stories | Effort |
   |---|---|---|---|
   | **0** | Caractérisation FE (régression safety net) | 1 (0.1) | M |
   | **1** | Refacto domaine ACP + migration data + conformité | 4 (1.1-1.4) | L |
   | **2** | Sélecteur global + bannière + Portfolio + #553 fix | 5 (2.1-2.5) | M |
   | **3** | Sous-rôles + Magic Link + PWA + Mandates + Ticketing | 9 (3.1-3.9) | L |
   | **4** | Governance hybride + Commissaire + CdC + signatures eIDAS | 9 (4.1-4.9) | L |
   | **5** | Modularité + onboarding + RBAC Communauté Moderator | 8 (5.1-5.8) | M |
   | **Tx** | Transversal continu (CI gate + helpers shared + log Tier 2) | 3 (Tx.1-Tx.3) | M |
   
   Total = 1+4+5+9+9+8+3 = **39 stories**.
   
   Chaque story = **1 PR atomique** (branch `story/<slice>.<n>-<entity>-<action>`), AC 4-cat condensé Gherkin + `data-testid` listés + files backend/frontend + ADR refs + cluster coord.
   
   ## Coordination cross-épics
   
   | Cluster / Epic | Stories impactées | Convention |
   |---|---|---|
   | #433 Decimal umbrella | 1.4, 3.1, 4.1, 4.8, 4.9 | 1 PR par use-case = 2 migrations atomiques |
   | #555 Result<_, String> epic | 3.1, 3.6, 4.1, 4.5, 4.8, 4.9 | idem, simultané dans la même PR |
   | #553 Building admin UX | 1.4, 2.5 | Closed by ces stories |
   | #554 World-model seed + AG state | 4.5, 4.6 + Tx.1 | Closed by ces stories |
   | #550 Playwright stratification | Tx.2 | Closed by Tx.2 |
   | #48 itsme/eID | 4.2, 4.4 | Promu in-scope |
   
   Story **4.9 méga `[cluster-coord]`** : 4 use-cases (expense, call_for_funds, charge_distribution, etat_date) × 2 migrations atomiques (Decimal + AppError) pour préserver l'invariant validate-before-compute.
   
   ## Intégration WBS go-live v0.1.0
   
   Cf. [`docs/WBS_GO_LIVE_v0.1.0.md`](docs/WBS_GO_LIVE_v0.1.0.md) Track H — WPs ajoutés 2026-05-20 :
   
   - **WP-H0** (slice 0 caractérisation) — pré-requis transverse
   - **WP-H4** (slice 3) — arbitrage v0.1.0 vs v0.2.0
   - **WP-H5** (slice 4) — arbitrage v0.1.0 vs v0.2.0 (Art. 3.87 §4 CC + eIDAS UE 910/2014)
   - **WP-H6** (slice 5) — arbitrage v0.1.0 vs v0.2.0
   - **WP-HTx** (slice transversal) — continuous
   
   Cartographie Maury ↔ WPs Track H existants :
   - WP-H1 ⇔ Story 1.4 + 1.1-1.3 (refacto ACP + #553 fix)
   - WP-H2 ⇔ Story 4.9 méga `[cluster-coord]`
   - WP-H3 ⇔ Story 4.5 (reprise #554)
   - WP-B4 ⇔ Story 1.4 + 2.5
   
   **Bloqueurs légaux v0.1.0** : WP-H0/H1/H2/H3/HTx uniquement. WP-H4/H5/H6 = extension produit à arbitrer.
   
   ## Mémoires d'agent applicables
   
   - `project_admin-publishes-conform-buildings`
   - `project_validate-before-compute`
   - `project_world-model-seed`
   - `project_a11y-wcag-aa-baseline`
   - `project_data-testid-systematic`
   - `project_fe-refactor-test-driven`
   - `feedback_multirole-narrative-scenarios`
   - `project_koprogo-modular-toolbox`
   - `project_no-f64-in-money`
   - `feedback_maury-token-economy`
   
   ## Sous-issues (création par batches sur validation humaine)
   
   | Batch | Stories | Statut |
   |---|---|---|
   | Slice 0 + 1 | 0.1, 1.1, 1.2, 1.3, 1.4 (5 issues) | ✅ #557, #558, #559, #560, #561 |
   | Slice 2 | 2.1-2.5 (5 issues) | ⏳ à créer (sur validation) |
   | Slice 3 | 3.1-3.9 (9 issues) | ⏳ à créer |
   | Slice 4 | 4.1-4.9 (9 issues) | ⏳ à créer |
   | Slice 5 | 5.1-5.8 (8 issues) | ⏳ à créer |
   | Slice Tx | Tx.1, Tx.2, Tx.3 (3 issues) | ⏳ à créer |
   
   Total final = 1 Epic + 39 sous-issues. Toutes les sous-issues référencent cet Epic via `Refs: #556`.
   
   ## Gate d'exécution Phase 6
   
   1. ✅ WBS Track H update — commit `5da52eb`
   2. ✅ Epic GH créé — #556 (cette issue)
   3. ⏳ Sous-issues GH par batch (5/39 créées, slice 0+1 fait)
   4. ⏳ Story 0.1 caractérisation FE (démarrage code)
   5. ⏳ Tx.1/Tx.2 en parallèle (continuous CI gate + helpers shared)
   
   🤖 Epic généré dans le cadre du pipeline Maury — pilotage humain @gilmry à chaque gate.

.. raw:: html

   </div>

