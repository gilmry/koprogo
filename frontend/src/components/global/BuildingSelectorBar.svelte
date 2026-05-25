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
  <div class="fixed top-2 left-2 z-40 lg:top-3 lg:left-3" data-testid="building-selector-bar">
    <BuildingSelector {user} />
  </div>
{/if}
