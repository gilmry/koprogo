==============================================================================================
Issue #663: feat(scope): BuildingSelector devrait chercher/sélectionner l'ACP, pas le Building
==============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement
:Assignees: Unassigned
:Created: 2026-07-26
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/663>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Suite au bug report manuel de création d'organisation (`/admin/organizations`), retour @gilmry sur le sélecteur global de périmètre : *« il faut qu'il soit centré sur acp et pas building »*.
   
   Les bugs immédiats (i18n `scope.*` manquante + chevauchement avec le contenu de page) ont été corrigés directement. Ce ticket documente la partie plus large : repenser le sélecteur pour qu'il porte sur l'**ACP** plutôt que sur le **Building**.
   
   ## Pourquoi ce n'est pas un simple fix
   
   Le composant `BuildingSelector.svelte` est explicitement spécifié Building-centric dans une story signée :
   
   - [`docs/maury/refonte-ux-multi-role-acp/stories.md`](../blob/main/docs/maury/refonte-ux-multi-role-acp/stories.md) — **Story 2.2** : *« Composant `BuildingSelector.svelte` + store scope ... Store `scope` réactif (selectedBuildingId/AcpId/PortfolioId) »*.
   
   Changer l'unité de sélection primaire de Building vers ACP touche :
   
   1. **Le store `scope.svelte.ts`** — `selectedAcpId` existe déjà dans `ScopeSnapshot` mais n'est **jamais écrit par aucune UI** actuellement (champ mort/réservé). `selectedBuildingId` est la seule source de vérité active.
   2. **~15 composants consommateurs** de `scope.selectedBuildingId` : `Navigation.svelte`, `lib/auth/permissions.ts` (RBAC), `CallForFundsForm`, `InvoiceForm`, `UnitCreateModal`, `MeetingCreateModal`, `CreatePollForm`, `CreateExchangeForm`, `CreateCampaignForm`, `SyndicContactPanel`, `OwnerUnitList`, `BoardManagement`, `pages/tickets.astro`, etc.
   3. **Le client API** : `frontend/src/lib/api/acps.ts` n'a aujourd'hui que `getAcp(id)` et `listAcps()` (liste complète filtrée par rôle, pas de recherche texte/debounce). `frontend/src/lib/api/buildings.ts` a `searchBuildings(text, maxResults)` — équivalent à construire ou adapter côté ACP (probablement faisable en filtrant `listAcps()` côté client vu la volumétrie attendue par syndic — pas de nouveau endpoint backend nécessaire a priori, à confirmer).
   4. **ADR/Story existants** : `BuildingSelector.svelte` et `scope.svelte.ts` référencent en commentaire "ADR-0011 (Portefeuille)" / "ADR-0012 (Navigation contextualisée)" — ces numéros ne correspondent à AUCUN ADR existant dans `docs/adr/` (0011 = quorum double, 0012 = fonds réserve/roulement, sans rapport). La direction réelle vit dans `docs/maury/refonte-ux-multi-role-acp/`. À clarifier/renommer en même temps si on retouche cette zone (dette de traçabilité).
   
   ## Contexte métier (pourquoi ACP a du sens maintenant)
   
   Le modèle a évolué depuis l'écriture de la Story 2.2 : Track H (#602, migration `units.organization_id → acp_id`, Art. 3.87 CC) fait de l'ACP l'entité de premier niveau (peut englober plusieurs immeubles), alors que Building reste une entité physique. Sélectionner un Building comme périmètre de travail n'est plus forcément le bon niveau si un syndic gère une ACP multi-bâtiments.
   
   ## Ce qui existe déjà côté backend
   
   Domaine ACP complet : `backend/src/domain/entities/acp.rs`, `application/{dto,ports,use_cases}/acp_*`, `infrastructure/web/handlers/acp_handlers.rs` avec `GET/POST/PUT/DELETE /acps`, `GET /acps/{id}`.
   
   ## Proposition de traitement (à valider avant code, cf. CLAUDE.md règle #5)
   
   1. Clarifier avec un mini-brief/story update (pas juste coder) : le sélecteur doit-il remplacer Building par ACP, ou proposer les deux niveaux (ACP en premier, Building en sous-filtre) ?
   2. Si confirmé : ajouter recherche texte côté `acps.ts` (client-side filter sur `listAcps()` suffit probablement), adapter `BuildingSelector.svelte` (le renommer ? `ScopeSelector.svelte` ?), garder `selectedBuildingId` dérivé/synchronisé pour ne pas casser les ~15 consommateurs existants en une seule PR.
   3. Mettre à jour Story 2.2 (ou en créer une 2.2bis) pour tracer la décision, et corriger les références ADR-0011/0012 erronées dans les commentaires.
   
   ## Fichiers clés
   
   - `frontend/src/components/global/BuildingSelector.svelte`
   - `frontend/src/components/global/BuildingSelectorBar.svelte`
   - `frontend/src/stores/scope.svelte.ts`
   - `frontend/src/lib/api/acps.ts`, `frontend/src/lib/api/buildings.ts`
   - `docs/maury/refonte-ux-multi-role-acp/stories.md` (Story 2.2)

.. raw:: html

   </div>

