<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../../lib/i18n";
  import { api } from "../../lib/api";
  import { formatCurrency } from "../../lib/utils/finance.utils";

  /**
   * L'encaissement, en une carte et une barre à deux segments.
   *
   * ── Pourquoi UNE carte et non deux ──────────────────────────────────────
   *
   * « Payé » et « En attente » vivaient dans deux cartes séparées. Ce sont
   * pourtant les deux parts d'un même tout : ce qui est encaissé et ce qui ne
   * l'est pas encore. Les séparer oblige à faire l'addition de tête pour
   * savoir où on en est, et la remise de design le dit sans détour — « the
   * 82/18 split is one number, not two cards ».
   *
   * La barre à deux segments rend le rapport visible sans calcul : la part
   * verte est ce qui est rentré, la part ambre ce qui manque.
   *
   * ── Ce que la route sert, et à qui ──────────────────────────────────────
   *
   * `GET /dashboard/accountant/stats`. Elle NE VÉRIFIAIT AUCUN RÔLE avant le
   * 2026-09-10 : tout membre de l'organisation obtenait ces chiffres, y
   * compris `owners_with_overdue` — le nombre de copropriétaires en retard de
   * paiement, qu'un copropriétaire n'a aucun titre à connaître.
   *
   * Elle est désormais réservée au syndic, au comptable et à
   * l'administration. C'est un des 87 cas de #864.
   */
  interface StatsEncaissement {
    total_expenses_current_month: number;
    total_paid: number;
    paid_percentage: number;
    total_pending: number;
    pending_percentage: number;
    owners_with_overdue: number;
  }

  let stats = $state<StatsEncaissement | null>(null);
  let chargement = $state(true);
  let erreur = $state<string | null>(null);

  onMount(async () => {
    try {
      stats = await api.get<StatsEncaissement>("/dashboard/accountant/stats");
    } catch (e) {
      erreur = e instanceof Error ? e.message : String(e);
    } finally {
      chargement = false;
    }
  });

  /**
   * La part encaissée, bornée entre 0 et 100.
   *
   * Le serveur sert déjà un pourcentage, mais une barre qui déborde de sa
   * piste est un défaut d'affichage qu'aucune donnée ne devrait pouvoir
   * provoquer. Un encaissement supérieur au dû — avance, double paiement —
   * remplit la barre, il ne la fait pas sortir.
   */
  const partPayee = $derived(
    stats ? Math.min(100, Math.max(0, stats.paid_percentage)) : 0,
  );
</script>

<section
  data-testid="carte-encaissement"
  class="rounded-card border border-border-soft bg-surface p-4"
>
  <h2 class="text-[12.5px] font-bold uppercase tracking-[0.05em] text-ink-3">
    {$_("dashboards.syndic.collection.title")}
  </h2>

  {#if chargement}
    <p
      data-testid="carte-encaissement-chargement"
      class="mt-3 text-sm text-muted"
    >
      {$_("common.loading")}
    </p>
  {:else if erreur}
    <p
      data-testid="carte-encaissement-erreur"
      class="mt-3 rounded-card border border-warn-border bg-warn-tint px-3 py-2 text-[12.5px] text-warn"
    >
      {$_("dashboards.syndic.collection.unavailable")}
      <span class="font-mono text-[11px]">({erreur})</span>
    </p>
  {:else if stats}
    <div class="mt-2 flex items-baseline justify-between gap-2">
      <p
        data-testid="carte-encaissement-pourcentage"
        data-part-payee={partPayee}
        class="tabular text-[28px] font-bold leading-none tracking-[-0.025em] text-ink"
      >
        {Math.round(partPayee)}<span class="text-[18px] text-muted">%</span>
      </p>
      <p class="tabular text-right text-[12.5px] text-muted">
        {formatCurrency(stats.total_paid)} / {formatCurrency(
          stats.total_paid + stats.total_pending,
        )}
      </p>
    </div>

    <!--
      Une seule piste, deux segments. Le vert est ce qui est rentré, l'ambre ce
      qui manque — et leur somme fait toujours la largeur, ce qui rend le
      rapport lisible sans lire les nombres.
    -->
    <div class="mt-3 flex h-2 w-full overflow-hidden rounded-full bg-line">
      <div
        data-testid="carte-encaissement-segment-paye"
        class="h-full bg-primary"
        style="width: {partPayee}%"
      ></div>
      <div class="h-full flex-1 bg-warn-bar"></div>
    </div>

    <div class="mt-2 flex items-center justify-between text-[11.5px]">
      <span class="flex items-center gap-1.5 text-muted">
        <span class="h-2 w-2 rounded-full bg-primary" aria-hidden="true"></span>
        {$_("dashboards.syndic.collection.paid")}
      </span>
      <span class="flex items-center gap-1.5 text-muted">
        <span class="h-2 w-2 rounded-full bg-warn-bar" aria-hidden="true"
        ></span>
        {$_("dashboards.syndic.collection.pending")}
      </span>
    </div>

    {#if stats.owners_with_overdue > 0}
      <!--
        Le nombre de copropriétaires en retard, et non le seul montant : un
        même total peut venir d'un gros débiteur ou de vingt petits, et les
        deux n'appellent pas la même démarche.
      -->
      <p
        data-testid="carte-encaissement-retardataires"
        data-retardataires={stats.owners_with_overdue}
        class="tabular mt-3 border-t border-line pt-2 text-[12.5px] text-warn"
      >
        {$_("dashboards.syndic.collection.overdueOwners", {
          values: { count: stats.owners_with_overdue },
        })}
      </p>
    {/if}
  {/if}
</section>
