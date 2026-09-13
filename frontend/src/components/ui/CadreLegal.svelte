<script lang="ts">
  import { _ } from "../../lib/i18n";
  import Icone from "./Icone.svelte";

  /**
   * L'encart de cadre légal — un seul composant, partout.
   *
   * ── Pourquoi il ne fallait pas les laisser ad hoc ───────────────────────
   *
   * Ces encarts sont, selon la remise de design, « the product's best
   * differentiator » : ils disent quelle règle de droit belge gouverne
   * l'écran qu'on regarde. Aucun concurrent ne le fait.
   *
   * Ils étaient pourtant écrits à la main dans chaque module — bleus ici,
   * jaunes là, avec un émoji ⚖️ dont le rendu change selon le système et que
   * les lecteurs d'écran annoncent. Six occurrences, six mises en forme.
   *
   * Ce qu'un composant unique apporte, au-delà de la cohérence : le lien
   * **« Voir la règle »** vers le registre. Un article cité sans moyen de le
   * lire demande de croire sur parole ; c'est le contraire de ce que ces
   * encarts promettent.
   *
   * ── Une nuance sur les références ───────────────────────────────────────
   *
   * La remise met en garde : les articles cités dans les maquettes sont des
   * **exemples**, et doivent venir du registre du projet, pas d'une chaîne
   * écrite dans un composant. `ruleRef` sert exactement à ça — il désigne une
   * règle du registre, et c'est elle qui porte l'article et son texte.
   *
   * `article` reste accepté pour les cas où la règle n'est pas encore au
   * registre, mais un encart qui n'a que lui ne peut pas offrir le lien : il
   * cite sans permettre de vérifier. C'est mieux que rien, et moins bien que
   * l'autre.
   */
  interface Props {
    /** Titre court : « Cadre légal », « Majorité requise », etc. */
    titre: string;
    /** Le texte de la règle, en clair. */
    corps: string;
    /**
     * Code de la règle au registre (`/legal/rules/{code}`). Quand il est
     * fourni, l'encart offre « Voir la règle → ».
     */
    ruleRef?: string;
    /**
     * Article cité, quand la règle n'est pas encore au registre. Rendu en
     * monospace, comme toutes les références légales du produit.
     */
    article?: string;
    /** Ancre de recette. */
    testId?: string;
  }

  let { titre, corps, ruleRef, article, testId }: Props = $props();
</script>

<!--
  `border-l-[3px]` en `primary` : la même barre d'accent que les lignes de
  tableau et les cartes de tâche. C'est la signature visuelle unique du
  produit, et elle remplace les fonds teintés — un fond coloré transmet l'état
  par la couleur seule, l'accent laisse le texte le porter.
-->
<div
  data-testid={testId ?? "cadre-legal"}
  class="rounded-[10px] border border-border-soft border-l-[3px] border-l-primary bg-surface px-4 py-3.5"
>
  <div class="flex items-start gap-3">
    <!--
      L'icône remplace l'émoji ⚖️. Un émoji ne peut pas être `aria-hidden` :
      il était annoncé, et l'encart s'ouvrait par « balance Cadre légal ».
    -->
    <Icone nom="legalScale" taille={18} class="mt-0.5 shrink-0 text-primary" />

    <div class="min-w-0 flex-1">
      <p class="text-[13px] font-bold text-ink">{titre}</p>
      <p class="mt-1 text-[12.5px] leading-relaxed text-ink-3">{corps}</p>

      {#if article}
        <p
          data-testid="cadre-legal-article"
          class="mt-1.5 font-mono text-[10.5px] text-muted-strong"
        >
          {article}
        </p>
      {/if}
    </div>

    {#if ruleRef}
      <a
        href="/legal-rules?code={encodeURIComponent(ruleRef)}"
        data-testid="cadre-legal-voir-la-regle"
        data-rule-ref={ruleRef}
        class="shrink-0 self-end whitespace-nowrap text-[12.5px] font-semibold text-primary hover:underline"
      >
        {$_("legal.seeRule") || "Voir la règle"}
        <span aria-hidden="true">→</span>
      </a>
    {/if}
  </div>
</div>
