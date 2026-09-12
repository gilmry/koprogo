<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../../lib/i18n";
  import { api } from "../../lib/api";
  import { formatCurrency } from "../../lib/utils/finance.utils";
  import Icone from "../ui/Icone.svelte";

  /**
   * Ce que le copropriétaire doit, association par association.
   *
   * ── Agréger pour informer, séparer pour agir ────────────────────────────
   *
   * Le total consolidé répond à « combien dois-je ce mois-ci » : il est
   * affiché en grand, parce que c'est la question qu'on se pose en ouvrant
   * l'écran.
   *
   * Mais **chaque ACP est une personne morale distincte, avec son propre
   * compte bancaire** — l'Art. 3.86 § 1er lui donne la personnalité juridique,
   * le § 3 impose des comptes ouverts à son nom. Un virement unique pour le
   * total paierait la mauvaise personne morale pour une partie de la somme :
   * l'argent atterrirait sur le compte de l'ACP A pour des charges dues à
   * l'ACP B, et il faudrait réclamer d'un côté, rembourser de l'autre.
   *
   * Le paiement se fait donc **par association**, un bouton chacune, avec
   * l'explication en clair plutôt que dans une infobulle que personne n'ouvre.
   *
   * ── L'affordance est conditionnelle ─────────────────────────────────────
   *
   * Avec une seule ACP : pas de décomposition, pas de total séparé, une seule
   * ligne. La majorité des copropriétaires n'ont qu'une copropriété, et leur
   * imposer une liste à un élément avec un total qui répète cet élément est du
   * bruit. Un choix unique n'est pas un menu.
   */
  interface DuAupresDuneAcp {
    acp_id: string;
    acp_name: string;
    /** Le numéro d'entreprise, à recopier sur le virement. */
    bce_number: string | null;
    charges_en_attente: number;
    montant: number;
  }

  let dettes = $state<DuAupresDuneAcp[]>([]);
  let chargement = $state(true);
  let erreur = $state<string | null>(null);

  onMount(async () => {
    try {
      dettes = await api.get<DuAupresDuneAcp[]>("/stats/owner/dues-by-acp");
    } catch (e) {
      erreur = e instanceof Error ? e.message : String(e);
    } finally {
      chargement = false;
    }
  });

  const total = $derived(dettes.reduce((s, d) => s + d.montant, 0));
  const plusieurs = $derived(dettes.length > 1);
</script>

<section
  data-testid="dettes-par-acp"
  data-nombre-acp={dettes.length}
  class="rounded-card border border-border-soft bg-surface p-4"
>
  <h2 class="text-[12.5px] font-bold uppercase tracking-[0.05em] text-ink-3">
    {$_("owner.dues.title")}
  </h2>

  {#if chargement}
    <p data-testid="dettes-par-acp-chargement" class="mt-3 text-sm text-muted">
      {$_("common.loading")}
    </p>
  {:else if erreur}
    <p
      data-testid="dettes-par-acp-erreur"
      class="mt-3 rounded-card border border-warn-border bg-warn-tint px-3 py-2 text-[12.5px] text-warn"
    >
      {$_("owner.dues.unavailable")}
      <span class="font-mono text-[11px]">({erreur})</span>
    </p>
  {:else if dettes.length === 0}
    <!--
      Ne rien devoir est une bonne nouvelle, et elle mérite d'être dite. Un
      encart vide se lit comme une panne.
    -->
    <p
      data-testid="dettes-par-acp-rien"
      class="mt-3 flex items-center gap-2 text-[13px] text-success-text"
    >
      <Icone nom="myTickets" taille={16} class="shrink-0" />
      {$_("owner.dues.nothingDue")}
    </p>
  {:else}
    {#if plusieurs}
      <!--
        Le total consolidé, pour SAVOIR. Il n'est affiché que s'il y a
        plusieurs associations : avec une seule, il répéterait la ligne
        unique juste en dessous.
      -->
      <p
        data-testid="dettes-par-acp-total"
        data-total={total}
        class="tabular mt-2 text-[34px] font-bold leading-none tracking-[-0.025em] text-ink"
      >
        {formatCurrency(total)}
      </p>
      <p class="tabular mt-1 text-[12.5px] text-muted">
        {$_("owner.dues.summary", {
          values: {
            charges: dettes.reduce((s, d) => s + d.charges_en_attente, 0),
            acps: dettes.length,
          },
        })}
      </p>
    {/if}

    <ul class="mt-3 space-y-2">
      {#each dettes as due (due.acp_id)}
        <li
          data-testid="dette-acp"
          data-acp-id={due.acp_id}
          class="flex items-center gap-3 rounded-card border border-border-soft p-3"
        >
          <div class="min-w-0 flex-1">
            <p class="truncate text-[14px] font-semibold text-ink">
              {due.acp_name}
            </p>
            {#if due.bce_number}
              <!--
                Le BCE identifie la personne morale créancière. C'est ce
                qu'on recopie sur le virement, et l'Art. 3.86 § 1er al. 4
                impose qu'il figure sur tout document émanant d'elle.
              -->
              <p class="font-mono text-[11px] text-muted-strong">
                {due.bce_number}
              </p>
            {/if}
            <p class="tabular mt-0.5 text-[12.5px] text-muted">
              {$_("owner.dues.chargesCount", {
                values: { count: due.charges_en_attente },
              })}
            </p>
          </div>

          <p class="tabular shrink-0 text-[15px] font-bold text-ink">
            {formatCurrency(due.montant)}
          </p>

          <!--
            Un bouton PAR association. 38 px de haut : au-dessus de la cible
            minimale, parce qu'un bouton de paiement qu'on manque coûte plus
            qu'un autre.
          -->
          <a
            href="/owner/payments?acp={encodeURIComponent(due.acp_id)}"
            data-testid="pay-acp-button"
            data-acp-id={due.acp_id}
            class="flex h-[38px] shrink-0 items-center rounded-button bg-primary px-4 text-[13px] font-semibold text-white hover:bg-primary-hover"
          >
            {$_("owner.dues.pay")}
          </a>
        </li>
      {/each}
    </ul>

    {#if plusieurs}
      <!--
        L'explication est INLINE, pas dans une infobulle : un copropriétaire
        qui voit deux boutons « Payer » se demande pourquoi il ne peut pas
        tout régler d'un coup, et la réponse doit être là où la question se
        pose.
      -->
      <p
        data-testid="dettes-par-acp-explication"
        class="mt-3 rounded-card border border-warn-border bg-warn-tint px-3 py-2 text-[11.5px] leading-relaxed text-ink-3"
      >
        {$_("owner.dues.separateLegalEntities")}
      </p>
    {/if}
  {/if}
</section>
