<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../../lib/i18n";
  import Icone from "../ui/Icone.svelte";
  import {
    listAcpsWithMetrics,
    type AcpAvecMetriques,
  } from "../../lib/api/acps";

  /**
   * La table « Mes ACP » du tableau de bord syndic.
   *
   * ── Ce qu'elle montre, et pourquoi ces colonnes-là ──────────────────────
   *
   * Un syndic gère plusieurs copropriétés, et la première question qu'il se
   * pose en ouvrant son tableau de bord est « laquelle a un problème ». Les
   * colonnes sont donc choisies pour qu'un coup d'œil y réponde :
   *
   * - **Le numéro BCE**, en monospace sous le nom. Une ACP est une personne
   *   morale, et son numéro d'entreprise est ce qui la désigne dans un
   *   courrier, un relevé bancaire ou un acte. Il doit figurer sur tous les
   *   documents qui en émanent (Art. 3.86 § 1er al. 4).
   * - **Les lots encodés sur déclarés**, ensemble et jamais séparés. Le
   *   produit affichait ailleurs « 0 lots au total » à côté de « 8 Lots » :
   *   deux nombres exacts qui mesurent des choses différentes, et qui se
   *   contredisent en apparence. La fraction lève l'ambiguïté.
   * - **Les quotités sur le total de l'acte**. C'est le chiffre qui décide de
   *   la validité des appels de fonds : tant que la somme n'atteint pas le
   *   total déclaré, chaque provision porte sur une base incomplète.
   *
   * ── Ce qu'elle ne montre pas encore ─────────────────────────────────────
   *
   * Les impayés par ACP. La remise les demande, et la route ne les sert pas :
   * les afficher à zéro laisserait croire qu'il n'y en a aucun. Une colonne
   * absente se remarque ; une colonne fausse se croit.
   */
  let acps = $state<AcpAvecMetriques[]>([]);
  let chargement = $state(true);
  let erreur = $state<string | null>(null);

  onMount(async () => {
    try {
      acps = await listAcpsWithMetrics();
    } catch (e) {
      erreur = e instanceof Error ? e.message : String(e);
    } finally {
      chargement = false;
    }
  });

  /**
   * Les quotités sont-elles complètes ?
   *
   * Comparaison de CHAÎNES normalisées, jamais de `parseFloat` : une quotité
   * est juridiquement opposable, et la convertir en flottant introduirait une
   * erreur de représentation sur une valeur exacte (ADR-0007/0008).
   *
   * `1000` et `1000.0000` désignent la même quotité : on retire les zéros de
   * queue avant de comparer, sans quoi une ACP parfaitement conforme
   * paraîtrait en écart.
   */
  function quotitesCompletes(somme: string, total: number): boolean {
    const normalisee = somme.includes(".")
      ? somme.replace(/0+$/, "").replace(/\.$/, "")
      : somme;
    return normalisee === String(total);
  }
</script>

<section
  data-testid="table-des-acp"
  class="rounded-card border border-border-soft bg-surface"
>
  <header
    class="flex items-center justify-between border-b border-border-soft px-4 py-3"
  >
    <h2 class="text-[15.5px] font-bold text-ink">
      {$_("dashboards.syndic.myAcps")}
    </h2>
    {#if !chargement && !erreur}
      <span
        class="tabular rounded-chip bg-chip-bg px-2 py-0.5 text-[11.5px] font-bold text-muted-strong"
      >
        {acps.length}
      </span>
    {/if}
  </header>

  {#if chargement}
    <p
      data-testid="table-des-acp-chargement"
      class="px-4 py-6 text-sm text-muted"
    >
      {$_("common.loading")}
    </p>
  {:else if erreur}
    <!--
      Le message porte le code. « Une erreur est survenue » ne permet pas de
      distinguer une route absente d'un serveur en panne, et c'est cette
      distinction qui décide de ce qu'on fait ensuite.
    -->
    <p
      data-testid="table-des-acp-erreur"
      class="m-4 rounded-card border border-warn-border bg-warn-tint px-4 py-3 text-sm text-warn"
    >
      {$_("dashboards.syndic.acpsUnavailable")}
      <span class="font-mono text-[11px]">({erreur})</span>
    </p>
  {:else if acps.length === 0}
    <p data-testid="table-des-acp-vide" class="px-4 py-6 text-sm text-muted">
      {$_("dashboards.syndic.noAcp")}
    </p>
  {:else}
    <table class="w-full">
      <thead>
        <tr class="bg-surface-alt">
          <th
            class="px-4 py-2.5 text-left text-[10.5px] font-bold uppercase tracking-[0.06em] text-muted-strong"
          >
            {$_("dashboards.syndic.acpColumns.name")}
          </th>
          <th
            class="px-4 py-2.5 text-right text-[10.5px] font-bold uppercase tracking-[0.06em] text-muted-strong"
          >
            {$_("dashboards.syndic.acpColumns.buildings")}
          </th>
          <th
            class="px-4 py-2.5 text-right text-[10.5px] font-bold uppercase tracking-[0.06em] text-muted-strong"
          >
            {$_("dashboards.syndic.acpColumns.units")}
          </th>
          <th
            class="px-4 py-2.5 text-right text-[10.5px] font-bold uppercase tracking-[0.06em] text-muted-strong"
          >
            {$_("dashboards.syndic.acpColumns.shares")}
          </th>
        </tr>
      </thead>
      <tbody>
        {#each acps as acp (acp.id)}
          {@const completes = quotitesCompletes(
            acp.quota_sum,
            acp.total_tantiemes,
          )}
          {@const lotsComplets = acp.units_count === acp.declared_units_total}
          <tr
            data-testid="acp-row"
            data-acp-id={acp.id}
            data-acp-name={acp.name}
            data-quotites-completes={completes ? "true" : "false"}
            class="border-t border-line-soft"
          >
            <td class="px-4 py-3">
              <p class="text-[14px] font-semibold text-ink">{acp.name}</p>
              {#if acp.bce_number}
                <!--
                  Le BCE en monospace : c'est un identifiant, et il se lit
                  chiffre par chiffre quand on le recopie sur un virement.
                -->
                <p class="font-mono text-[11.5px] text-muted-strong">
                  {acp.bce_number}
                </p>
              {/if}
            </td>
            <td class="tabular px-4 py-3 text-right text-[14px] text-ink-2">
              {acp.buildings_count}
            </td>
            <td class="tabular px-4 py-3 text-right text-[14px]">
              <!--
                Les deux nombres ENSEMBLE quand ils diffèrent. Afficher le seul
                encodé ferait croire l'immeuble complet ; afficher le seul
                déclaré ferait croire les lots saisis.
              -->
              <span class={lotsComplets ? "text-ink-2" : "text-warn"}>
                {acp.units_count}{lotsComplets
                  ? ""
                  : `/${acp.declared_units_total}`}
              </span>
            </td>
            <td class="tabular px-4 py-3 text-right text-[14px]">
              <span
                data-testid="acp-quotites"
                class="inline-flex items-center gap-1.5 {completes
                  ? 'text-success-text'
                  : 'text-warn'}"
              >
                {#if !completes}
                  <Icone nom="alert" taille={14} class="shrink-0" />
                {/if}
                {acp.quota_sum}/{acp.total_tantiemes}
              </span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>
