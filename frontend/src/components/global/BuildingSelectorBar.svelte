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
    - Mobile (< lg) : overlay `fixed` top-2 RIGHT-14 laisse la place au logo
      KoproGo (gauche) + bouton hamburger (left:3) + zone cloche/notifs.
      Ne chevauche pas le header mobile.
    - Desktop (>= lg) : plus de `fixed` — la barre est rendue EN FLUX NORMAL
      par `Layout.astro` (juste avant `ContextBanner`, dans le wrapper de
      contenu décalé par la sidebar). Corrige le chevauchement avec le titre
      de page (ex. `<h1>Organisations</h1>`) observé quand le sélecteur était
      `fixed top-3 left-64` par-dessus le contenu sans espace réservé.
  -->
  <div
    class="fixed top-2 right-14 z-40 lg:static lg:top-auto lg:right-auto lg:left-auto lg:z-auto lg:flex lg:w-full lg:justify-start lg:border-b lg:border-gray-200 lg:bg-white lg:px-6 lg:py-3"
    data-testid="building-selector-bar"
  >
    <BuildingSelector {user} />
  </div>
{/if}
