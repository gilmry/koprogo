===========================================================================================
Issue #553: Fiche immeuble admin : bouton Modifier KO + drift total_units / quotas (≠ 1000)
===========================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug,javascript track:software,priority:high rust,legal-compliance governance
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-06-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/553>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   Observés en live par l'utilisateur (compte admin) sur la fiche d'un immeuble (`/building-detail?id=...`). 2 bugs distincts à traiter dans cette issue.
   
   ---
   
   ## Bug 1 : Bouton « Modifier » non fonctionnel (UI cassée)
   
   ### Constat
   Sur la fiche d'un immeuble en compte admin, le clic sur le bouton « Modifier » ne déclenche aucune action visible (pas de modal, pas de navigation).
   
   ### Localisation
   - Bouton : [frontend/src/components/BuildingDetail.svelte:111](frontend/src/components/BuildingDetail.svelte#L111) → `<Button variant="primary" on:click={handleEdit}>`
   - Handler : [frontend/src/components/BuildingDetail.svelte:67-69](frontend/src/components/BuildingDetail.svelte#L67-L69)
     ```ts
     const handleEdit = () => {
       showEditModal = true;
     };
     ```
   - Modal cible : `showEditModal` → probablement consommé ailleurs pour rendre un composant `BuildingForm` (cf. ligne ~215 : `onsuccess={handleEditSuccess}`).
   
   ### Cause probable (hypothèses)
   1. Le binding `showEditModal` ne déclenche pas le rendu du modal (modal monté mais pas visible, conflit z-index, opacity 0, etc.)
   2. `on:click` Svelte 5 syntaxe : le code utilise encore `on:click` (Svelte 4) au lieu de `onclick={...}` (Svelte 5 runes). Selon le mode du composant Button, l'event peut ne pas se propager.
   3. Le composant `BuildingForm` enfant attend des props que `BuildingDetail` ne passe pas correctement → erreur silencieuse.
   4. Permissions : RBAC backend refuse l'opération avant rendu (mais ce serait visible en console).
   
   ### Recette
   1. Ouvrir DevTools → Console + Network pendant le clic « Modifier »
   2. Vérifier si `handleEdit` est appelé (ajouter `console.log` temporaire)
   3. Vérifier si `showEditModal` passe à `true` (inspecter le composant Svelte via devtools)
   4. Vérifier que le composant modal est bien rendu (DOM inspector)
   5. Tester le même bouton dans `BuildingList.svelte:185-190` (`data-testid="edit-building-button"`) pour comparer
   6. Ajouter un test Playwright @happy : login admin → ouvrir fiche immeuble → clic Modifier → modal doit s'afficher avec champs pré-remplis
   
   ### Critères d'acceptation
   - [ ] Clic « Modifier » sur fiche immeuble admin → modal d'édition apparaît
   - [ ] Submit du modal → building mis à jour → modal fermé → fiche rechargée avec nouvelles données
   - [ ] Test Playwright `@happy` reproduit le flow + `@negative` (validation form)
   
   ---
   
   ## Bug 2 : Drift entre `total_units` (fiche) et count réel des lots + total des quotas ≠ 1000/10000
   
   ### Constat
   - Sur la fiche immeuble, le champ « nombre de lots » affiche **X** (depuis [BuildingDetail.svelte:141](frontend/src/components/BuildingDetail.svelte#L141) → `{building.total_units}`).
   - Quand on regarde la liste détaillée des lots (units), le **count réel** est différent de X.
   - De plus, le **total des quotas des units** n'est pas égal à 1000 (millièmes — convention belge) ou 10000.
   
   ### Cause probable
   **Drift entre champ déclaratif et données réelles.**
   
   `Building.total_units` est un champ stocké saisi à la création de l'immeuble (cf. backend `CreateBuildingDto`, FE Buildings.spec.ts qui passe `total_units: 10`). Ce champ n'est **pas** dérivé du count des units : si on crée plus ou moins de units que la valeur saisie, la fiche ment.
   
   Pour les quotas : il n'y a pas (visible) de contrainte SQL ni de validation backend qui force `SUM(units.quota) WHERE building_id = X` = 1000.0 (millièmes belges) ou 10000 (dix-millièmes). La somme peut donc dériver à la création/édition d'units sans alerte.
   
   Référence convention : [frontend/src/components/meetings/QuorumPanel.svelte:18](frontend/src/components/meetings/QuorumPanel.svelte#L18) → `let totalQuotas = $state<number>((meeting as any).total_quotas ?? 1000)` → la base utilise bien 1000 millièmes par défaut pour le quorum.
   
   ### Recette
   1. Décider la **source de vérité** :
      - **Option A (recommandée)** : `total_units` devient un champ **dérivé** côté backend (`COUNT(units WHERE building_id = X)`), exposé via une projection / view. Le champ stocké est supprimé ou marqué deprecated.
      - **Option B** : garder `total_units` stocké comme « capacité prévue » et afficher EN PLUS `units.length` comme « lots créés », avec un warning si divergence (ex: `<span class="text-orange-600">10 lots prévus, 8 créés</span>`).
   2. Pour les quotas : ajouter une **contrainte de domaine + validation backend** au moment de create/update d'un Unit :
      ```sql
      -- conceptuel
      CHECK (SUM(quota) OVER (PARTITION BY building_id) <= 1000)
      ```
      OU validation use-case : refuser un Unit qui ferait dépasser 1000 (ou dont l'absence rend le total < 1000 si le building est "complet").
   3. Migration de données : auditer tous les buildings existants pour repérer les drifts actuels et fournir un script de réparation (ou rapport humain à traiter).
   4. ADR à écrire si choix non trivial entre A et B.
   
   ### Critères d'acceptation
   - [ ] La fiche immeuble affiche le **count réel** des units, ou expose explicitement les 2 valeurs si on garde la sémantique « capacité prévue vs créés »
   - [ ] Le total des quotas par building est **garanti = 1000** (ou la base choisie) par contrainte backend
   - [ ] Tentative de créer/éditer un Unit qui ferait dériver le total → erreur typée + message utilisateur clair
   - [ ] Migration data : rapport des buildings en drift actuel + plan de réparation
   - [ ] Test BDD `@happy` + `@negative` (rejet du dépassement)
   
   ---
   
   ## Hors-scope
   
   - Pas lié à #550 (auth refresh — fixé).
   - Pas lié à #552 (work-reports/inspections 400 — autre fiche).
   - Pas un blocker #549 (gate go-live, périmètre différent).
   
   ## Priorité suggérée
   
   - Bug 1 (Modifier cassé) : **high** — empêche un workflow admin courant.
   - Bug 2 (drift quotas) : **high** — risque d'intégrité données et de quorum invalide en AG (impact légal Art. 3.87 CC).

.. raw:: html

   </div>

