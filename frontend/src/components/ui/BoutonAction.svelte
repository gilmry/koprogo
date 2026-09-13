<script lang="ts">
  import Icone from "./Icone.svelte";

  /**
   * Un bouton d'action de ligne — 36 px de côté, cible atteignable.
   *
   * ── Ce qu'il remplace ───────────────────────────────────────────────────
   *
   * Des boutons contenant un seul émoji, sans remplissage :
   *
   *     <button class="text-primary-600" aria-label="Modifier">✏️</button>
   *
   * Leur zone de tap effective était d'environ **20 px**. La cible minimale
   * est de 44 px sur mobile, et 36 px sur une ligne de tableau au bureau où
   * le pointeur est précis. Vingt-trois de ces boutons existaient.
   *
   * Le défaut n'était pas seulement la taille. Un émoji dans un bouton est
   * **annoncé par le lecteur d'écran EN PLUS de l'`aria-label`** : le bouton
   * s'appelait « crayon Modifier ». Une icône SVG `aria-hidden` laisse
   * l'`aria-label` seul porter le nom.
   *
   * ── Ce que ce composant n'invente pas ──────────────────────────────────
   *
   * Il exige `ariaLabel`. Un bouton qui ne montre qu'une icône n'a pas de
   * texte visible : sans nom accessible, il est muet pour un lecteur d'écran
   * et introuvable par `getByRole("button", { name })` dans les recettes.
   */
  interface Props {
    /** Nom d'un tracé de `lib/icones.ts` — `edit`, `trash`, etc. */
    nom: string;
    /** Nom accessible. Obligatoire : le bouton n'a aucun texte visible. */
    ariaLabel: string;
    /** Infobulle. Reprend `ariaLabel` par défaut. */
    title?: string;
    /**
     * Teinte. `danger` pour ce qui détruit — la couleur ne porte pas le sens
     * à elle seule, l'`aria-label` le dit, mais elle aide l'œil à hésiter.
     */
    ton?: "neutre" | "danger";
    disabled?: boolean;
    testId?: string;
    onclick?: () => void;
  }

  let {
    nom,
    ariaLabel,
    title,
    ton = "neutre",
    disabled = false,
    testId,
    onclick,
  }: Props = $props();
</script>

<button
  type="button"
  {onclick}
  {disabled}
  aria-label={ariaLabel}
  title={title ?? ariaLabel}
  data-testid={testId}
  class="inline-flex h-9 w-9 items-center justify-center rounded-button transition-colors disabled:cursor-not-allowed disabled:opacity-50 {ton ===
  'danger'
    ? 'text-danger hover:bg-danger-tint'
    : 'text-ink-3 hover:bg-chip-bg'}"
>
  <Icone {nom} taille={17} />
</button>
