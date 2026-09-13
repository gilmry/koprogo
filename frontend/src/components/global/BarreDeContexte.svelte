<script lang="ts">
  import { onMount } from "svelte";
  import { authStore } from "../../stores/auth";
  import { resoudrePerimetreAuChargement } from "../../stores/scope.svelte";
  import { getBuilding } from "../../lib/api/buildings";
  import { listAcps } from "../../lib/api/acps";
  import BuildingSelector from "./BuildingSelector.svelte";
  import ContextBanner from "./ContextBanner.svelte";

  /**
   * Réhydrate le périmètre à CHAQUE chargement de page (#841).
   *
   * ── Pourquoi ici ──────────────────────────────────────────────────────────
   *
   * `BarreDeContexte` est le seul composant garanti présent sur toute page
   * authentifiée (`Layout.astro`), sans la restriction de rôle qui masque
   * `BuildingSelector` pour les owners — et le périmètre est lu par des
   * écrans owner aussi (cf. #841).
   *
   * ── Pourquoi `void` et pas `await` ────────────────────────────────────────
   *
   * Ne bloque pas le montage de la barre : les composants qui lisent le
   * scope (`ContextBanner`, `JournalEntriesPanel`, etc.) réagissent déjà à
   * `$state` quand il change, pas besoin d'attendre ici.
   *
   * ── Ce qui a cassé la première tentative (393c6ee0 → 7d28bb5e) ──────────
   *
   * Un appel équivalent avait fait dépasser trois `networkidle`/`waitForResponse`
   * Playwright de 10 à 30s. Deux propriétés réduisent le risque cette fois :
   * `resoudrePerimetreAuChargement` ne fait JAMAIS deux requêtes séquentielles
   * quand une seule suffit (le défaut n'est même pas tenté si le lien profond a
   * gagné, ni si un `buildingId` était présent et refusé), et le défaut serveur
   * (`GET /acps`, déjà utilisé par `BuildingSelector`) ne s'exécute que quand
   * aucun périmètre n'est encore posé — jamais en boucle, puisqu'il ne
   * redéclenche rien qui le rappelle.
   *
   * Si une régression `networkidle` réapparaît, la première hypothèse à
   * vérifier est la charge concurrente du montage de page (#718 : un seul
   * worker actix, bcrypt bloquant), pas un rappel en boucle de cette fonction
   * — elle ne s'invoque qu'une fois par montage.
   */
  onMount(() => {
    void resoudrePerimetreAuChargement({
      building: getBuilding,
      acps: listAcps,
    });
  });

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
