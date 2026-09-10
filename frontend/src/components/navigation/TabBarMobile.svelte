<script lang="ts">
  import { _ } from "../../lib/i18n";
  import Icone from "../ui/Icone.svelte";
  import { ongletsPour, type OngletMobile } from "../../lib/onglets-mobiles";

  /**
   * La barre d'onglets du bas — ce qu'un pouce atteint sans effort.
   *
   * Elle remplace le tiroir, qui cachait trente-neuf destinations derrière un
   * tap dans le coin supérieur gauche. Le tiroir reste, sous « Plus », mais
   * il cesse d'être le seul chemin.
   *
   * ── Ce que la barre n'a pas le droit de faire ──────────────────────────
   *
   * Recevoir des ancrages en `navigation-menu-*`. `Navigation.test.ts` compte
   * les menus métier du syndic — exactement cinq — et un onglet qui porterait
   * cet ancrage casserait ce compte. Les onglets ont leur propre espace de
   * noms, `tabbar-{cle}`.
   */
  interface Props {
    /** Rôle courant. Aucune barre si le rôle est inconnu. */
    role: string | null | undefined;
    /** Chemin courant, pour marquer l'onglet actif. */
    chemin?: string;
    /**
     * Compteurs par clé d'onglet. Ceux qui ne correspondent à aucun onglet
     * visible sont agrégés sur « Plus » : sans cela, une relance en attente
     * derrière le menu serait invisible.
     */
    compteurs?: Record<string, number>;
  }

  let { role, chemin = "", compteurs = {} }: Props = $props();

  const onglets = $derived(ongletsPour(role));

  /** Un onglet est actif si le chemin courant commence par sa destination. */
  function estActif(onglet: OngletMobile): boolean {
    if (!chemin) return false;
    if (onglet.href === "/") return chemin === "/";
    return chemin === onglet.href || chemin.startsWith(onglet.href + "/");
  }

  /**
   * Le compte porté par « Plus » : tout ce qui n'a pas d'onglet à soi.
   *
   * Un compteur qui n'apparaît nulle part est pire qu'absent : il laisse
   * croire qu'il n'y a rien à faire.
   */
  const compteurDuPlus = $derived.by(() => {
    const visibles = new Set(onglets.map((o) => o.cle));
    return Object.entries(compteurs)
      .filter(([cle]) => !visibles.has(cle) || cle === "plus")
      .reduce((somme, [, n]) => somme + n, 0);
  });

  function compteurDe(onglet: OngletMobile): number {
    if (onglet.cle === "plus") return compteurDuPlus;
    return compteurs[onglet.cle] ?? 0;
  }
</script>

{#if onglets.length > 0}
  <!--
    `lg:hidden` : la barre est mobile, le bureau a sa colonne latérale. C'est
    bien du mobile-first — les styles de base sont ceux du téléphone, et `lg:`
    RETIRE plutôt qu'il n'ajoute, parce que l'élément lui-même n'existe qu'en
    petit.

    74 px : 56 px de cibles utilisables, plus la place de l'indicateur
    d'accueil des téléphones sans bouton.
  -->
  <nav
    data-testid="tabbar"
    aria-label={$_("navigation.mainMobile") || "Navigation principale"}
    class="fixed inset-x-0 bottom-0 z-40 flex h-barre-onglets items-start border-t border-border-soft bg-surface/95 pb-[env(safe-area-inset-bottom,0px)] backdrop-blur lg:hidden"
  >
    {#each onglets as onglet (onglet.cle)}
      {@const actif = estActif(onglet)}
      {@const compte = compteurDe(onglet)}
      <a
        href={onglet.href}
        data-testid="tabbar-{onglet.cle}"
        data-active={actif ? "true" : "false"}
        aria-current={actif ? "page" : undefined}
        class="relative flex h-14 flex-1 flex-col items-center justify-center gap-1 {actif
          ? 'text-primary'
          : 'text-muted'}"
      >
        <!--
          Trait plus affirmé sur l'onglet actif : la remise demande 2.0 à 2.2
          pour l'état actif, contre 1.7 au repos. C'est ce qui distingue les
          deux sans recourir à la couleur seule.
        -->
        <Icone nom={onglet.icone} taille={23} trait={actif ? 2.1 : 1.7} />
        <span
          class="text-[10.5px] leading-none {actif
            ? 'font-bold'
            : 'font-medium'}"
        >
          {$_(onglet.libelle)}
        </span>

        {#if compte > 0}
          <!--
            Le compteur est redondant avec le contenu de l'écran visé : il
            échappe donc à la règle des 12 px minimum, qui vise l'information
            qu'on ne peut lire nulle part ailleurs.
          -->
          <span
            data-testid="tabbar-badge-{onglet.cle}"
            class="tabular absolute right-1/4 top-1 min-w-[17px] rounded-full bg-danger px-1 text-[10px] font-bold leading-[17px] text-white"
            aria-hidden="true"
          >
            {compte > 99 ? "99+" : compte}
          </span>
          <span class="sr-only">
            {compte}
            {$_("navigation.pendingItems") || "en attente"}
          </span>
        {/if}
      </a>
    {/each}
  </nav>
{/if}
