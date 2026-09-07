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
