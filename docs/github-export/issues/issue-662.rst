==========================================================================================================================
Issue #662: bug: boutons <Button on:click> inopérants (migration Svelte 5 runes incomplète) — bloque création organisation
==========================================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug
:Assignees: Unassigned
:Created: 2026-07-26
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/662>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Test manuel (@gilmry, profil admin) : impossible de créer une organisation depuis `/admin/organizations`. Le bouton "➕ Nouvelle organisation" ne réagit pas — aucune requête réseau, aucune erreur console, le modal ne s'ouvre pas.
   
   ## Cause racine
   
   `Button.svelte` (`frontend/src/components/ui/Button.svelte`) est en mode Svelte 5 runes : il spread `...restProps` sur le `<button>` natif et n'implémente pas `createEventDispatcher`. Sur un **composant** (par opposition à un élément DOM natif), la directive legacy `on:click={...}` (Svelte 4) est traitée comme un forwarding d'événement de composant — pas comme une prop `onclick`. Comme `Button` ne dispatch aucun événement `click`, le handler n'est **jamais appelé**. Silencieux : pas d'erreur, pas de warning.
   
   C'est le même pattern déjà identifié lors de l'audit pré-release v0.1.0 (cf. `docs/cowork/Prompt2026-04-18-22-53.md`, bugs B1/B9) : migration Svelte 4 → 5 incomplète sur les handlers d'événements (`on:click` → `onclick`).
   
   ## Fichiers affectés (6 fichiers, ~18 boutons)
   
   Recherche : `<Button ... on:click={...}>` dans `frontend/src/components/`.
   
   | Fichier | Lignes | Boutons cassés |
   |---|---|---|
   | `OrganizationList.svelte` | 132 | Nouvelle organisation |
   | `BuildingList.svelte` | 116 | Créer un immeuble |
   | `UserListAdmin.svelte` | 173 | Créer un utilisateur |
   | `BuildingDetail.svelte` | 122, 155 | Retour, Modifier |
   | `MeetingDetail.svelte` | 236, 257, 281, 286, 296 | Retour, Terminer, Annuler, Reprogrammer (×2) |
   | `ExpenseDetail.svelte` | 206, 226, 229, 232, 236, 239, 243, 247 | Retour, Marquer payé, Marquer en retard, Annuler, Réactiver, Unpay... |
   
   Les boutons natifs `<button on:click={...}>` (ex. actions Edit/Toggle/Delete dans `OrganizationList.svelte`) restent fonctionnels — `on:click` reste valide sur un élément DOM natif en runes mode. Seul l'usage sur le **composant** `<Button>` est cassé.
   
   ## Fix attendu
   
   Remplacer `on:click={handler}` par `onclick={handler}` sur chaque usage de `<Button ...>` listé ci-dessus. Fix mécanique, pas de changement de comportement attendu au-delà de "le bouton fonctionne enfin".
   
   ## Risque / priorité
   
   Impacte des parcours admin/syndic critiques : création d'organisation (bloquant pour tout nouveau tenant), création d'immeuble, création d'utilisateur, actions sur dépenses (marquer payé/en retard), actions sur réunions (terminer/annuler/reprogrammer). v0.1.0 n'étant pas en prod, pas de crise — mais bloquant pour valider ces parcours en test manuel/E2E.
   
   ## Reproduction (organisation)
   
   1. Login `admin@koprogo.com`
   2. `/admin/organizations`
   3. Clic sur "➕ Nouvelle organisation"
   4. Résultat : rien ne se passe (attendu : modal `OrganizationForm` s'ouvre)
   
   ## Suggestion de test de non-régression
   
   Ajouter un test Vitest component (`@happy`) qui clique le bouton et vérifie l'appel du handler, pour chacun des 6 composants — évite la régression silencieuse si `Button.svelte` évolue encore.

.. raw:: html

   </div>

