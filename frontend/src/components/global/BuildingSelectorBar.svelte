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
    Positioning rationale :
    - Mobile (< lg) : top-2 RIGHT-14 leaves room for KoproGo logo (left) +
      hamburger button (left:3) + the bell/notifications area. Doesn't
      overlap the mobile header logo at top-left.
    - Desktop (>= lg) : top-3 LEFT-64 (= sidebar width 60 + 4 padding) so
      the selector sits in the content area RIGHT of the sidebar, not on
      top of the sidebar's KoproGo logo.
  -->
  <div
    class="fixed top-2 right-14 z-40 lg:top-3 lg:right-auto lg:left-64"
    data-testid="building-selector-bar"
  >
    <BuildingSelector {user} />
  </div>
{/if}
