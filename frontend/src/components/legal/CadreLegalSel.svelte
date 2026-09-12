<script lang="ts">
  import { _ } from "../../lib/i18n";
  import CadreLegal from "../ui/CadreLegal.svelte";

  /**
   * Le cadre légal des systèmes d'échange local, traduit.
   *
   * Cet encart vivait en dur dans `exchanges.astro`, en français, dans un
   * produit servi en quatre langues. Le passer par `CadreLegal` lui donne à
   * la fois la mise en forme commune et ses traductions.
   */
</script>

<CadreLegal
  titre={$_("localExchanges.selLegalTitle")}
  corps={$_("localExchanges.selLegalBody")}
  testId="sel-cadre-legal"
/>
