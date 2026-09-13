<script lang="ts">
  import { _ } from "../../lib/i18n";

  /**
   * Le titre d'une page, et ce qu'elle fait.
   *
   * ── Pourquoi une île Svelte pour deux lignes de texte ──────────────────
   *
   * Un gabarit Astro fige son texte à la construction, alors que le produit
   * est statique et choisit sa langue DANS le navigateur. Tout ce qui reste
   * dans le gabarit est donc écrit dans une seule langue, définitivement.
   *
   * Mesuré en chargeant chaque écran en `fr` puis en `nl` : quarante pages
   * rendaient leur titre et leur description à l'identique. Un néerlandophone
   * lisait « Gestion des Charges » en haut de chaque écran.
   *
   * ── Ce que le composant apporte en plus de la traduction ───────────────
   *
   * Les quarante en-têtes étaient recopiés à la main, et avaient divergé :
   * `mt-1` ici, `mt-2` là, un `text-3xl` qui est grand pour 393 px. Un seul
   * endroit à corriger le jour où la refonte touchera à cette forme.
   */
  interface Props {
    /** Clé i18n du titre. */
    titre: string;
    /** Clé i18n de la description, facultative. */
    description?: string;
    testId?: string;
  }

  let { titre, description, testId }: Props = $props();
</script>

<div class="mb-6" data-testid={testId ?? "en-tete-de-page"}>
  <!--
    `text-2xl` sous `sm`, `text-3xl` au-delà : 30 px de titre sur un écran de
    393 px mangeaient deux lignes avant le premier mot utile.
  -->
  <h1 class="text-2xl sm:text-3xl font-bold text-ink">{$_(titre)}</h1>
  {#if description}
    <p class="text-muted mt-2 text-sm sm:text-base">{$_(description)}</p>
  {/if}
</div>
