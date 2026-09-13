<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../../lib/i18n";
  import { API_BASE_URL } from "../../lib/api";
  import Icone from "../ui/Icone.svelte";

  /**
   * Le registre des règles de droit belge, lisible par un humain.
   *
   * ── Ce que cette page rend possible ─────────────────────────────────────
   *
   * `CadreLegal` cite un article et propose « Voir la règle → ». Sans page de
   * destination, ce lien serait un 404 : on citerait la loi en demandant de
   * croire sur parole.
   *
   * ── Le code en ancre, et pourquoi ───────────────────────────────────────
   *
   * L'URL porte `?code=`, et la règle correspondante est mise en avant à
   * l'ouverture. Un lien qui dépose au milieu d'une liste de cent règles sans
   * dire laquelle n'aide personne.
   */
  interface RegleLegale {
    code: string;
    category: string;
    roles: string[];
    article: string;
    title: string;
    content: string;
    keywords: string[];
  }

  let regles = $state<RegleLegale[]>([]);
  let chargement = $state(true);
  let erreur = $state<string | null>(null);
  let recherche = $state("");
  let codeVise = $state<string | null>(null);

  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    codeVise = params.get("code");

    try {
      const reponse = await fetch(`${API_BASE_URL}/legal/rules`);
      if (!reponse.ok) {
        // Dire le code, et non « une erreur est survenue » : c'est ce qui
        // permet de distinguer une route absente d'un serveur en panne.
        throw new Error(`HTTP ${reponse.status}`);
      }
      regles = await reponse.json();
    } catch (e) {
      erreur = e instanceof Error ? e.message : String(e);
    } finally {
      chargement = false;
    }
  });

  const filtrees = $derived.by(() => {
    const q = recherche.trim().toLowerCase();
    if (!q) return regles;
    return regles.filter(
      (r) =>
        r.title.toLowerCase().includes(q) ||
        r.article.toLowerCase().includes(q) ||
        r.content.toLowerCase().includes(q) ||
        r.keywords.some((k) => k.toLowerCase().includes(q)),
    );
  });
</script>

<h1 class="text-[27px] font-bold tracking-[-0.02em] text-ink">
  {$_("legal.rulesTitle") || "Règles légales"}
</h1>
<p class="mt-1 text-[12.5px] text-muted">
  {$_("legal.rulesSubtitle") ||
    "Le droit belge de la copropriété, tel que le produit l'applique."}
</p>

<label class="mt-6 block">
  <span class="sr-only"
    >{$_("legal.searchRules") || "Rechercher une règle"}</span
  >
  <input
    type="search"
    data-testid="legal-rules-search"
    bind:value={recherche}
    placeholder={$_("legal.searchRules") || "Rechercher une règle"}
    class="h-[34px] w-full max-w-[300px] rounded-button border border-border-soft bg-surface px-3 text-sm text-ink placeholder:text-muted"
  />
</label>

{#if chargement}
  <p data-testid="legal-rules-loading" class="mt-6 text-sm text-muted">
    {$_("common.loading") || "Chargement…"}
  </p>
{:else if erreur}
  <!--
    Le message porte le code. « Une erreur est survenue » ne permet pas de
    distinguer une route absente d'un serveur en panne, et c'est cette
    distinction qui décide de ce qu'on fait ensuite.
  -->
  <p
    data-testid="legal-rules-error"
    class="mt-6 rounded-card border border-warn-border bg-warn-tint px-4 py-3 text-sm text-warn"
  >
    {$_("legal.rulesUnavailable") ||
      "Les règles ne sont pas accessibles pour le moment."}
    <span class="font-mono text-[11px]">({erreur})</span>
  </p>
{:else if filtrees.length === 0}
  <p data-testid="legal-rules-empty" class="mt-6 text-sm text-muted">
    {$_("legal.noRuleFound") || "Aucune règle ne correspond."}
  </p>
{:else}
  <ul class="mt-6 space-y-3">
    {#each filtrees as regle (regle.code)}
      {@const vise = regle.code === codeVise}
      <li
        data-testid="legal-rule-{regle.code}"
        data-rule-code={regle.code}
        data-targeted={vise ? "true" : "false"}
        class="rounded-card border border-border-soft bg-surface p-4 {vise
          ? 'border-l-[3px] border-l-primary bg-primary-tint'
          : ''}"
      >
        <div class="flex items-start gap-3">
          <Icone
            nom="legalScale"
            taille={17}
            class="mt-0.5 shrink-0 text-primary"
          />
          <div class="min-w-0">
            <p class="text-[14.5px] font-semibold text-ink">{regle.title}</p>
            <p class="mt-0.5 font-mono text-[11px] text-muted-strong">
              {regle.article}
            </p>
            <p class="mt-2 text-[12.5px] leading-relaxed text-ink-3">
              {regle.content}
            </p>
          </div>
        </div>
      </li>
    {/each}
  </ul>
{/if}
