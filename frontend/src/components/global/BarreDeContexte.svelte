<script lang="ts">
  import { authStore } from "../../stores/auth";
  import BuildingSelector from "./BuildingSelector.svelte";
  import ContextBanner from "./ContextBanner.svelte";

  /**
   * Une seule barre de contexte, au lieu de deux empilées.
   *
   * ── Ce qu'il y avait ────────────────────────────────────────────────────
   *
   * `BuildingSelectorBar` (57 px) et `ContextBanner` (41 px) se suivaient dans
   * `Layout.astro`, soit **environ 100 px de chrome** avant le contenu propre
   * de la page — et sur mobile, avant même le premier mot de l'écran.
   *
   * Et elles disaient la même chose deux fois : l'immeuble courant apparaissait
   * dans le champ du sélecteur ET dans le fil d'Ariane du bandeau.
   *
   * ── Ce que la fusion change, et ce qu'elle ne change pas ────────────────
   *
   * Les deux composants restent : `BuildingSelector` porte le contrôle,
   * `ContextBanner` porte l'indication et la pastille de quotités. Ils sont
   * simplement rendus **côte à côte** dans une seule barre plutôt qu'empilés.
   *
   * C'est la voie prudente que la remise impose pour toute cette refonte :
   * `building-selector-bar`, `context-banner` et les cinq ancrages du bandeau
   * survivent tels quels. Six répertoires de recettes les visent, et une
   * restructuration qui les déplace ne se distingue plus d'une régression.
   *
   * ── Pourquoi le sélecteur À GAUCHE ─────────────────────────────────────
   *
   * Le contrôle avant l'indication : on choisit sa copropriété, puis on lit ce
   * qu'elle est. L'ordre inverse ferait lire un état avant d'avoir le moyen de
   * le changer.
   */
  let user = $derived($authStore.user);
</script>

{#if user}
  <!--
    56 px, hauteur imposée par la remise. En flux normal sur toutes les
    tailles : l'ancien overlay `fixed top-2 right-14` n'existait que parce que
    le mobile avait été traité en dernier.

    `flex-wrap` : sous 640 px, le fil d'Ariane passe à la ligne plutôt que
    d'écraser le sélecteur. Une barre qui déborde est pire qu'une barre qui
    grandit.
  -->
  <div
    data-testid="barre-de-contexte"
    class="flex min-h-[56px] w-full flex-wrap items-center gap-x-4 gap-y-2 border-b border-border-soft bg-surface px-4 py-2 lg:px-6"
  >
    <div data-testid="building-selector-bar" class="min-w-[220px] flex-1">
      <BuildingSelector {user} />
    </div>

    <!--
      Le bandeau se rend nul quand aucun immeuble n'est choisi : il occupe donc
      zéro place tant que le périmètre n'est pas posé, et la barre se réduit au
      seul sélecteur.
    -->
    <ContextBanner />
  </div>
{/if}
