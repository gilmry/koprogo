<script lang="ts">
  // Story 2.2 — Wrapper Astro-friendly du BuildingSelector.
  //
  // BuildingSelector accepte `user` en prop pour rester pur (testable
  // sans store). Cette wrapper consomme `authStore` (Svelte legacy store)
  // et le passe au composant — pattern habituel pour les composants
  // « top-bar » qui doivent reagir aux changements d'auth.

  import { authStore } from "../../stores/auth";
  import BuildingSelector from "./BuildingSelector.svelte";

  let user = $derived($authStore.user);

  // ── La réhydratation depuis `?buildingId=` est DÉBRANCHÉE ────────────────
  //
  // `rehydraterDepuisLurl` existe dans `scope.svelte.ts`, elle est testée, et
  // sa propriété de sécurité est vérifiée par témoin : l'identifiant de l'URL
  // n'est jamais cru sur parole, il est validé par le serveur.
  //
  // Mais l'appeler ici a CASSÉ trois specs Playwright, mesuré sur `ba78cd88` :
  //
  //     ticket-complaint.spec.ts:200        page.goto networkidle, 30 s
  //     AccountantJournalEntriesJourney:109 page.goto networkidle, 30 s
  //     AccountantReportsJourney:106        waitForResponse, 10 s
  //
  // Les trois naviguent vers une URL portant `?buildingId=`, et
  // `ticket-complaint` PASSAIT avant. Le symptôme — `networkidle` qui n'arrive
  // jamais — dit que l'activité réseau ne s'arrête plus : un appel qui boucle,
  // ou qui déclenche un rafraîchissement de jeton en cascade.
  //
  // Je débranche plutôt que de laisser une régression sur la branche qui
  // alimente la démo. La fonction reste, avec ses tests ; ce qui manque est le
  // diagnostic de la boucle, pas la fonction. Suivi en #841.
</script>

{#if user}
  <!--
    La barre est EN FLUX NORMAL, sur toutes les tailles.

    ── Ce qu'elle était ──────────────────────────────────────────────────────

    Un overlay `fixed top-2 right-14` sous `lg`, qui flottait entre le logo et
    la cloche du header mobile, et redevenait statique au-dessus. Le
    commentaire qui l'accompagnait décrivait avec soin comment il évitait de
    chevaucher trois éléments — signe qu'il n'aurait pas dû être là.

    ── Pourquoi il n'aurait pas dû être là ───────────────────────────────────

    La remise de design le dit dans son correctif 0.10 : cet overlay
    « n'existe que parce que le mobile a été traité en dernier ». Écrire une
    application desktop-first oblige à replacer les éléments un par un sur
    petit écran, et chaque replacement crée sa propre exception.

    `Layout.astro` réserve déjà une gouttière de 56 px pour le header mobile
    (`h-14 lg:hidden`) et rend cette barre juste après. En flux normal, elle se
    place donc naturellement dessous — sans coordonnées, sans `z-index`, et
    sans qu'il faille connaître la largeur du hamburger.

    Les styles de base sont ceux du mobile ; `lg:` **ajoute** le bureau.
  -->
  <div
    class="flex w-full justify-start border-b border-gray-200 bg-white px-4 py-2 lg:px-6 lg:py-3"
    data-testid="building-selector-bar"
  >
    <BuildingSelector {user} />
  </div>
{/if}
