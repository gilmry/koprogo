---
feature: refonte-ux-multi-role-acp
phase: architecture
phase_togaf: D (Technology)
agent_bmad: Winston (Architecte hexagonal)
authors: [Gilles Maury, Farah Maury]
date: à compléter
version: 0.0
status: Blocked — waiting Phase 2 (PRD) sign-off
prd_source: prd.md (John, v0.0)
---

# Architecture — Refonte UX multi-rôle + modèle ACP

## Methode Maury — Phase TOGAF D (Technology)

> ⛔ **Phase Blocked** — ce document ne peut être démarré qu'après signature du `prd.md` par @gilmry.

---

## Sections imposées (à remplir en Phase 3)

1. Diagramme d'architecture cible (entités + ports + adapters)
   - Nouvelle entité `ACP` (racine d'agrégat dans BC Identity & Access)
   - Refacto `Building.organization_id` → `Building.acp_id` + `ACP.organization_id?`
   - Nouvelle entité `Portfolio` (BC Portfolio)
   - Pattern `Moderator` rôle pour Communauté (BC Community)
2. ADRs à produire
   - ADR-NNNN : modélisation ACP comme racine d'agrégat séparée d'Organization
   - ADR-NNNN : portefeuille (entité backend vs UI préférence — recommandation entité backend cf. brief section 4 C5)
   - ADR-NNNN : convention `data-testid="<entity>-<action>"` (cf. [[data-testid-systematic]])
   - ADR-NNNN : suite de caractérisation FE (cf. [[fe-refactor-test-driven]])
3. Migrations SQL nécessaires
   - Table `acps` (id, organization_id NULLABLE FK, name, slug, created_at, updated_at)
   - `buildings.acp_id` NULLABLE intermédiaire → backfill → NOT NULL
   - Table `portfolios` (id, name, organization_id FK, owner_user_id FK, building_ids JSONB/M2M)
   - `user_role_assignments.valid_until` (NULLABLE pour rôles permanents)
4. Endpoints API (additions/refacto)
   - CRUD `/acps`
   - CRUD `/portfolios`
   - `list_buildings` filtré par rôle + ACP autorisée
   - `POST /reservations` avec champ `on_behalf_of_acp: bool`
5. Impact frontend (composants Svelte)
   - Nouveau composant `BuildingSelector.svelte` (top-left global)
   - Nouveau composant `ContextBanner.svelte` (bannière 3-niveaux)
   - Refacto `Navigation.svelte` (menus conditionnels par rôle ET sélection)
   - Refacto `BuildingDetail.svelte` (count réel + somme réelle quotas + badge conformité — coordonne avec #553)
6. Stratégie tests architecturale
   - `frontend/tests/e2e/characterization/` (suite régression à figer en Phase 2)
   - `frontend/src/**/__tests__/*.test.ts` Vitest TDD RED-GREEN-BLUE
   - `frontend/tests/e2e/refonte-ux/*.spec.ts` Playwright multi-rôle 4-cat
7. Coordination cross-épics
   - #433 Decimal cluster — toute refacto use-case touche aussi Decimal si applicable (cf. meta-comment #433)
   - #555 Result<_, String> epic — toute refacto use-case migre aussi vers `AppError` (rule 4)
8. Gate de validation Phase 3 — sign-off humain

## Signature & GATE

- [ ] PRD Phase 2 signé (prérequis)
- [ ] Architecture rédigée selon template Maury
- [ ] ADRs publiés sous `docs/adr/`
- [ ] Migrations SQL revues
- [ ] Sign-off humain @gilmry pour ouvrir Phase 4 (Stories)
