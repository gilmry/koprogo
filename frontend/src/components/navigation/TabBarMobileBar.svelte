<script lang="ts">
  /**
   * Enrobage Astro de la barre d'onglets.
   *
   * `TabBarMobile` reçoit son rôle en propriété pour rester pur — testable
   * sans store, comme `BuildingSelector`. Cet enrobage consomme `authStore` et
   * le lui passe, selon le même motif que `BuildingSelectorBar`.
   *
   * Il lit aussi le chemin courant. Le frontend est une application Astro
   * multi-page : `window.location.pathname` est donc fiable au montage et ne
   * change pas ensuite — chaque navigation recharge le document. Pas besoin
   * d'écouter quoi que ce soit.
   */
  import { authStore } from "../../stores/auth";
  import TabBarMobile from "./TabBarMobile.svelte";

  let user = $derived($authStore.user);
  let role = $derived(user?.role ?? null);

  const chemin = typeof window === "undefined" ? "" : window.location.pathname;
</script>

{#if user}
  <TabBarMobile {role} {chemin} />
{/if}
