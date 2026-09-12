<script lang="ts">
  import { ICONES, iconeConnue } from "../../lib/icones";

  /**
   * Une icône du jeu unique du produit.
   *
   * Elle est TOUJOURS décorative : `aria-hidden="true"`, `focusable="false"`.
   * Le nom accessible vit sur le contrôle qui l'entoure — un bouton, un lien,
   * un onglet. C'est la règle de la remise de design, et c'est aussi ce qui
   * laisse `getByRole("button", { name })` continuer de fonctionner dans les
   * recettes : si l'icône portait le nom, le nom accessible du bouton
   * deviendrait « bâtiment Immeubles » au lieu de « Immeubles ».
   *
   * Si le contrôle n'a pas de texte visible, il lui faut un `aria-label`.
   * `garde-icone-sans-nom` le vérifie.
   */
  interface Props {
    /** Nom du tracé dans `lib/icones.ts`. */
    nom: string;
    /** Côté du carré, en pixels. 18–19 en navigation, 15–17 en ligne, 23 en onglet. */
    taille?: number;
    /**
     * Graisse du trait. 1.7 par défaut ; 2.0 à 2.2 pour les chevrons et les
     * icônes d'onglet actif, que la remise veut plus affirmés parce qu'ils
     * portent l'affordance du tap.
     */
    trait?: number;
    class?: string;
  }

  let { nom, taille = 18, trait = 1.7, class: classe = "" }: Props = $props();

  // Une icône inconnue ne rendrait rien du tout, en silence : l'entrée de menu
  // perdrait son repère visuel sans que personne s'en aperçoive. On préfère
  // le dire à la console qu'afficher un trou.
  const traces = $derived(
    iconeConnue(nom)
      ? ICONES[nom]
      : (console.warn(`Icône inconnue : « ${nom} ». Voir lib/icones.ts.`), []),
  );
</script>

<svg
  class={classe}
  width={taille}
  height={taille}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={trait}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  {#each traces as trace}
    <path d={trace} />
  {/each}
</svg>
