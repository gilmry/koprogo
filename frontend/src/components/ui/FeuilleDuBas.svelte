<script lang="ts">
  import type { Snippet } from "svelte";
  import { trapFocus, FocusManager } from "../../lib/accessibility";
  import { _ } from "../../lib/i18n";
  import Icone from "./Icone.svelte";

  /**
   * Une feuille qui monte du bas de l'écran.
   *
   * ── Pourquoi pas `Modal` ────────────────────────────────────────────────
   *
   * `Modal` centre son contenu, et son piège de focus est excellent. Mais un
   * dialogue centré et une feuille du bas ne résolvent pas le même problème :
   *
   * Sur un téléphone tenu d'une main, le pouce atteint le tiers inférieur de
   * l'écran. Un menu déroulant s'ouvre **vers le bas dans le vide** quand son
   * déclencheur est déjà en haut, et un dialogue centré met ses options hors
   * de portée. La feuille monte à la rencontre du pouce.
   *
   * Modifier `Modal` pour lui ajouter une variante changerait toutes les
   * modales du produit. Ce composant réutilise donc le MÊME utilitaire
   * `trapFocus` — une seule implémentation du piège de focus, deux mises en
   * page.
   *
   * ── Ce que la feuille doit faire, et que rien ne rappelle ───────────────
   *
   * Piéger le focus, fermer sur Échap, **rendre le focus au déclencheur** à la
   * fermeture, et verrouiller le défilement du corps. Sans ce dernier point,
   * le fond défile sous la feuille quand on glisse dedans, ce qui est
   * désorientant et fait perdre sa place.
   */
  interface Props {
    ouverte: boolean;
    titre: string;
    /** Sous-titre facultatif : le contexte de ce qu'on choisit. */
    soustitre?: string;
    onfermer: () => void;
    testId?: string;
    children?: Snippet;
  }

  let {
    ouverte = false,
    titre,
    soustitre,
    onfermer,
    testId,
    children,
  }: Props = $props();

  let feuille = $state<HTMLElement | undefined>(undefined);
  const focus = new FocusManager();

  $effect(() => {
    if (!ouverte || !feuille) return;

    // Le focus revient au déclencheur à la fermeture : sans cela, l'utilisateur
    // au clavier repart du haut du document.
    focus.save();
    const relacher = trapFocus(feuille);

    // Le corps ne défile plus sous la feuille. `overflow` est restauré à
    // l'identique, pas mis à « visible » — écraser une valeur qu'on n'a pas
    // posée casserait une page qui la gérait elle-même.
    const defilementAvant = document.body.style.overflow;
    document.body.style.overflow = "hidden";

    const surTouche = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onfermer();
      }
    };
    document.addEventListener("keydown", surTouche);

    return () => {
      relacher();
      document.removeEventListener("keydown", surTouche);
      document.body.style.overflow = defilementAvant;
      focus.restore();
    };
  });
</script>

{#if ouverte}
  <!--
    Le voile : `rgba(28,25,23,.45)` de la remise, une encre chaude plutôt qu'un
    noir pur — le produit n'a aucun gris froid.

    Il est cliquable pour fermer, et c'est un VRAI bouton : un `<div
    role="button">` n'est pas atteignable au clavier, et Svelte le signale à
    juste titre.
  -->
  <button
    type="button"
    data-testid="{testId ?? 'feuille-du-bas'}-voile"
    aria-label={$_("common.close")}
    onclick={onfermer}
    class="fixed inset-0 z-40 bg-[rgba(28,25,23,.45)]"
  ></button>

  <div
    bind:this={feuille}
    data-testid={testId ?? "feuille-du-bas"}
    role="dialog"
    aria-modal="true"
    aria-label={titre}
    class="fixed inset-x-0 bottom-0 z-50 max-h-[85vh] overflow-y-auto rounded-t-sheet bg-surface pb-[env(safe-area-inset-bottom)] shadow-sheet"
  >
    <!--
      La poignée : 38 × 4 px. Elle n'est pas décorative — c'est le signe
      convenu qu'un panneau se ferme en le tirant vers le bas, et son absence
      fait chercher un bouton.
    -->
    <div class="flex justify-center pt-2.5" aria-hidden="true">
      <span class="h-1 w-[38px] rounded-full bg-border-strong"></span>
    </div>

    <header class="flex items-start gap-3 px-4 pb-3 pt-3">
      <div class="min-w-0 flex-1">
        <h2 class="text-[15.5px] font-bold text-ink">{titre}</h2>
        {#if soustitre}
          <p class="mt-0.5 text-[12.5px] text-muted">{soustitre}</p>
        {/if}
      </div>
      <!--
        44 × 44 px : la cible minimale sur mobile. Un bouton de fermeture
        qu'on manque enferme dans le panneau.
      -->
      <button
        type="button"
        data-testid="{testId ?? 'feuille-du-bas'}-fermer"
        aria-label={$_("common.close")}
        onclick={onfermer}
        class="flex h-11 w-11 shrink-0 items-center justify-center rounded-button text-ink-3 hover:bg-chip-bg"
      >
        <Icone nom="chevronDown" taille={20} trait={2.1} />
      </button>
    </header>

    {#if children}{@render children()}{/if}
  </div>
{/if}
