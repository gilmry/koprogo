<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from "../../lib/i18n";
  import {
    resolutionsApi,
    type Resolution,
    type Vote,
    VoteChoice,
    ResolutionStatus,
    MajorityType,
  } from "../../lib/api/resolutions";
  import { api } from "../../lib/api";
  import { toast } from "../../stores/toast";
  import ResolutionStatusBadge from "./ResolutionStatusBadge.svelte";
  import { formatDateTime } from "../../lib/utils/date.utils";
  import { withErrorHandling } from "../../lib/utils/error.utils";
  import { toNumber } from "../../lib/utils/decimal.utils";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";

  let {
    resolution,
    meetingStatus = "Scheduled",
    isAdmin = false,
  }: {
    resolution: Resolution;
    meetingStatus?: string;
    isAdmin?: boolean;
  } = $props();

  let votes = $state<Vote[]>([]);

  // L'action en attente de confirmation.
  //
  // Ce composant est en mode RUNES : un `let` simple n'y serait PAS
  // réactif, et l'écran ne se redessinerait jamais (#832).
  //
  // Le `confirm()` remplacé était un dialogue du NAVIGATEUR : un navigateur
  // piloté le supprime, et l'action prend la forme exacte d'une panne (#844).
  //
  // Clôturer un vote fige le résultat d'une résolution d'assemblée.
  let suppressionEnAttente = $state(false);
  let loadingVotes = $state(false);
  let showVotes = $state(false);

  let voteChoice = $state<VoteChoice | null>(null);
  let votingPower = $state(1);
  let proxyOwnerId = $state("");
  let submittingVote = $state(false);
  let closingVoting = $state(false);

  let myOwnerId = $state("");
  let myUnits = $state<Array<{ unit_id: string; unit_number?: string }>>([]);
  let selectedUnitId = $state("");

  $effect(() => {
    (async () => {
      try {
        // `null` = l'utilisateur n'est pas copropriétaire. Cas normal pour un
        // syndic ou un comptable, et non une erreur : la route rendait un 404
        // qui polluait le parcours nominal (voir #766).
        const me = await api.get<{ id: string } | null>("/owners/me");
        if (!me?.id) {
          return;
        }
        myOwnerId = me.id;
        const ownerships = await api.get<Array<{ unit_id: string }>>(
          `/owners/${myOwnerId}/units`,
        );
        const enriched = await Promise.all(
          (Array.isArray(ownerships) ? ownerships : []).map(async (o) => {
            try {
              const u = await api.get<{ unit_number?: string }>(
                `/units/${o.unit_id}`,
              );
              return { unit_id: o.unit_id, unit_number: u.unit_number };
            } catch {
              return { unit_id: o.unit_id };
            }
          }),
        );
        myUnits = enriched;
        if (myUnits.length > 0) selectedUnitId = myUnits[0].unit_id;
      } catch {
        // Non-owner user (e.g. syndic) -- voting not applicable
      }
    })();
  });

  let isOwner = $derived(!!myOwnerId && myUnits.length > 0);
  let canVote = $derived(
    resolution.status === ResolutionStatus.Pending &&
      meetingStatus === "Scheduled" &&
      isOwner,
  );

  /**
   * Le droit de CLORE le scrutin, distinct du droit d'y VOTER.
   *
   * Présider une assemblée n'est pas y participer. Un syndic qui n'est pas
   * copropriétaire ne vote pas — Art. 3.87 § 6, les voix suivent les
   * quotes-parts — mais c'est lui qui tient la séance et en proclame le
   * résultat.
   *
   * Le bouton dépendait de `canVote`, donc de `isOwner` : un syndic non
   * copropriétaire ne le voyait jamais, les résolutions restaient
   * indéfiniment « en attente », et le plafonnement de l'Art. 3.87 § 7 —
   * qui s'applique à la clôture — n'était jamais déclenché. Un invariant
   * légal implémenté mais inatteignable. Constaté en recette le 2026-09-04.
   */
  let canCloseVoting = $derived(
    isAdmin &&
      resolution.status === ResolutionStatus.Pending &&
      meetingStatus === "Scheduled",
  );
  let isClosed = $derived(resolution.status !== ResolutionStatus.Pending);
  let votesPour = $derived(resolution.vote_count_pour ?? 0);
  let votesContre = $derived(resolution.vote_count_contre ?? 0);
  let votesAbstention = $derived(resolution.vote_count_abstention ?? 0);
  let totalVotes = $derived(
    resolution.total_votes ?? votesPour + votesContre + votesAbstention,
  );
  // Les trois totaux arrivent en STRING JSON (`Decimal` cote Rust, cf.
  // ADR-0008 et `docs/api/openapi.json`). L'addition `+` sur des strings
  // CONCATENE : "0.6000" + "0.4000" + "0.0000" donnait "0.60000.40000.0000",
  // dont `> 0` vaut false via NaN — la ligne « pouvoir de vote » ne
  // s'affichait donc JAMAIS. Panne silencieuse sur un decompte de vote
  // d'AG (Art. 3.87). Detectee par `e2e_resolutions`, harnais qui n'avait
  // jamais ete execute.
  let totalVotingPower = $derived(
    toNumber(resolution.total_voting_power_pour) +
      toNumber(resolution.total_voting_power_contre) +
      toNumber(resolution.total_voting_power_abstention),
  );

  function getMajorityLabel(type: MajorityType): string {
    switch (type) {
      case MajorityType.Absolute:
        return $_("resolutions.create.majorityAbsolute");
      case MajorityType.TwoThirds:
        return $_("resolutions.create.majorityTwoThirds");
      case MajorityType.FourFifths:
        return $_("resolutions.create.majorityFourFifths");
      case MajorityType.Unanimity:
        return $_("resolutions.create.majorityUnanimity");
      default:
        return type;
    }
  }

  // Les pourcentages viennent du SERVEUR, jamais d'un calcul local.
  //
  // Ils étaient calculés ici par `count / totalVotes`, c'est-à-dire par
  // NOMBRE DE BULLETINS. L'Art. 3.87 § 6 compte des voix : « chaque
  // copropriétaire dispose d'un nombre de voix correspondant à sa quote-part
  // dans les parties communes ».
  //
  // Constaté en recette le 2026-09-06 (RN-3) : Alice 550 ‰ pour, Bob 250 ‰ et
  // Claire 200 ‰ contre. L'écran affichait « Pour 1 vote (33,3 %) » et
  // « Contre 2 votes (66,7 %) », barre rouge plus longue à l'appui, pendant
  // que l'API renvoyait 55 % et 45 %. Une résolution ADOPTÉE était présentée
  // comme rejetée, sur l'écran même où le syndic rédige son procès-verbal.
  //
  // Le backend avait été corrigé le 2026-09-04 ; le calcul existait en double
  // et corriger l'un laissait l'autre faux, sans qu'aucun test ne le voie.
  //
  // ATTENTION aux dénominateurs, qui diffèrent À DESSEIN et ne somment donc
  // pas à 100 : « pour » et « contre » se rapportent aux voix EXPRIMÉES, les
  // abstentions en étant exclues par l'Art. 3.87 § 8 ; l'abstention se
  // rapporte à TOUTES les voix présentes. Ne pas les ramener à une base
  // commune sous prétexte que le total ne fait pas 100.
  let partPour = $derived(resolution.pour_percentage ?? 0);
  let partContre = $derived(resolution.contre_percentage ?? 0);
  let partAbstention = $derived(resolution.abstention_percentage ?? 0);

  /// Les millièmes retenus, tels que le serveur les compte — plafonnement de
  /// l'Art. 3.87 § 7 compris depuis le 2026-09-06.
  function millièmes(valeur: string | number | undefined): string {
    const n = Number(valeur ?? 0);
    return Number.isFinite(n)
      ? n.toLocaleString("fr-BE", { maximumFractionDigits: 2 })
      : "0";
  }

  async function loadVotes() {
    await withErrorHandling({
      action: async () => {
        const result = await resolutionsApi.getVotes(resolution.id);
        votes = result;
        showVotes = true;
        return result;
      },
      setLoading: (v: boolean) => (loadingVotes = v),
      errorMessage: $_("resolutions.vote.loadVotesError"),
    });
  }

  async function handleVote() {
    if (!voteChoice) {
      toast.error($_("resolutions.vote.selectVote"));
      return;
    }
    if (!myOwnerId || !selectedUnitId) {
      toast.error($_("resolutions.vote.needOwnerAndUnit"));
      return;
    }

    await withErrorHandling({
      action: () =>
        resolutionsApi.castVote(resolution.id, {
          owner_id: myOwnerId,
          unit_id: selectedUnitId,
          choice: voteChoice!,
          voting_power: votingPower,
          proxy_owner_id: proxyOwnerId || undefined,
        }),
      setLoading: (v: boolean) => (submittingVote = v),
      successMessage: $_("resolutions.vote.success"),
      errorMessage: $_("resolutions.vote.error"),
      onSuccess: async () => {
        resolution = await resolutionsApi.getById(resolution.id);
        voteChoice = null;
        if (showVotes) await loadVotes();
      },
    });
  }

  function handleCloseVoting() {
    suppressionEnAttente = true;
  }

  async function executerLaction() {
    suppressionEnAttente = false;

    await withErrorHandling({
      action: () => resolutionsApi.closeVoting(resolution.id),
      setLoading: (v: boolean) => (closingVoting = v),
      errorMessage: $_("resolutions.vote.closeError"),
      onSuccess: async (result) => {
        resolution = result;
        const status =
          resolution.status === ResolutionStatus.Adopted
            ? $_("resolutions.vote.adopted")
            : $_("resolutions.vote.rejected");
        toast.success(
          $_("resolutions.vote.closedMessage", { values: { status } }),
        );
        if (showVotes) await loadVotes();
      },
    });
  }

  function getChoiceLabel(choice: VoteChoice): string {
    switch (choice) {
      case VoteChoice.Pour:
        return $_("resolutions.vote.for");
      case VoteChoice.Contre:
        return $_("resolutions.vote.against");
      case VoteChoice.Abstention:
        return $_("resolutions.vote.abstain");
      default:
        return choice;
    }
  }

  function getChoiceColor(choice: VoteChoice): string {
    switch (choice) {
      case VoteChoice.Pour:
        return "text-green-700 bg-green-100";
      case VoteChoice.Contre:
        return "text-red-700 bg-red-100";
      case VoteChoice.Abstention:
        return "text-gray-700 bg-gray-100";
      default:
        return "text-gray-700 bg-gray-100";
    }
  }
</script>

<div class="border border-gray-200 rounded-lg p-4">
  <div class="flex items-start justify-between mb-3">
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2 mb-1">
        <h4 class="text-sm font-semibold text-gray-900 truncate">
          {resolution.title}
        </h4>
        <ResolutionStatusBadge status={resolution.status} />
      </div>
      {#if resolution.description}
        <p class="text-sm text-gray-600 mt-1">{resolution.description}</p>
      {/if}
      <p class="text-xs text-muted mt-1">
        {getMajorityLabel(resolution.majority_required)}
      </p>
    </div>
  </div>

  <div class="space-y-2 mb-4">
    <div data-testid="vote-progress-pour">
      <div class="flex items-center justify-between text-sm mb-1">
        <span class="text-green-700 font-medium"
          >{$_("resolutions.vote.for")}</span
        >
        <span class="text-gray-600"
          >{votesPour}
          {$_("resolutions.vote.votes", { values: { count: votesPour } })} · {millièmes(
            resolution.total_voting_power_pour,
          )} ‰ ({partPour.toFixed(1)}%)</span
        >
      </div>
      <div class="w-full bg-gray-100 rounded-full h-2.5">
        <div
          class="bg-green-500 h-2.5 rounded-full transition-all"
          style="width: {partPour}%"
        ></div>
      </div>
    </div>

    <div data-testid="vote-progress-contre">
      <div class="flex items-center justify-between text-sm mb-1">
        <span class="text-red-700 font-medium"
          >{$_("resolutions.vote.against")}</span
        >
        <span class="text-gray-600"
          >{votesContre}
          {$_("resolutions.vote.votes", { values: { count: votesContre } })} · {millièmes(
            resolution.total_voting_power_contre,
          )} ‰ ({partContre.toFixed(1)}%)</span
        >
      </div>
      <div class="w-full bg-gray-100 rounded-full h-2.5">
        <div
          class="bg-red-500 h-2.5 rounded-full transition-all"
          style="width: {partContre}%"
        ></div>
      </div>
    </div>

    <div data-testid="vote-progress-abstention">
      <div class="flex items-center justify-between text-sm mb-1">
        <span class="text-gray-700 font-medium"
          >{$_("resolutions.vote.abstain")}</span
        >
        <span class="text-gray-600"
          >{votesAbstention}
          {$_("resolutions.vote.votes", { values: { count: votesAbstention } })} ·
          {millièmes(resolution.total_voting_power_abstention)} ‰ ({partAbstention.toFixed(
            1,
          )}%)</span
        >
      </div>
      <div class="w-full bg-gray-100 rounded-full h-2.5">
        <div
          class="bg-gray-400 h-2.5 rounded-full transition-all"
          style="width: {partAbstention}%"
        ></div>
      </div>
    </div>

    <p class="text-xs text-gray-500 mt-1">
      {$_("resolutions.vote.total")}: {totalVotes}
      {$_("resolutions.vote.votes", { values: { count: totalVotes } })}
      {#if totalVotingPower > 0}
        &middot; {$_("resolutions.vote.votingPower")}: {totalVotingPower}
        {$_("resolutions.vote.thousandths")}
      {/if}
    </p>
  </div>

  {#if canVote}
    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-3">
      <h5 class="text-sm font-semibold text-blue-900 mb-3">
        {$_("resolutions.vote.formTitle")}
      </h5>

      <div class="flex gap-2 mb-3">
        <button
          onclick={() => (voteChoice = VoteChoice.Pour)}
          class="flex-1 py-2 px-3 rounded-lg text-sm font-medium border-2 transition-colors
            {voteChoice === VoteChoice.Pour
            ? 'bg-green-600 text-white border-green-600'
            : 'bg-white text-green-700 border-green-300 hover:bg-green-50'}"
          disabled={submittingVote}
          data-testid="vote-btn-pour"
        >
          {$_("resolutions.vote.for")}
        </button>
        <button
          onclick={() => (voteChoice = VoteChoice.Contre)}
          class="flex-1 py-2 px-3 rounded-lg text-sm font-medium border-2 transition-colors
            {voteChoice === VoteChoice.Contre
            ? 'bg-red-600 text-white border-red-600'
            : 'bg-white text-red-700 border-red-300 hover:bg-red-50'}"
          disabled={submittingVote}
          data-testid="vote-btn-contre"
        >
          {$_("resolutions.vote.against")}
        </button>
        <button
          onclick={() => (voteChoice = VoteChoice.Abstention)}
          class="flex-1 py-2 px-3 rounded-lg text-sm font-medium border-2 transition-colors
            {voteChoice === VoteChoice.Abstention
            ? 'bg-gray-600 text-white border-gray-600'
            : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'}"
          disabled={submittingVote}
          data-testid="vote-btn-abstention"
        >
          {$_("resolutions.vote.abstain")}
        </button>
      </div>

      {#if myUnits.length > 1}
        <div class="mb-3">
          <label
            for="unit-{resolution.id}"
            class="block text-xs font-medium text-gray-700 mb-1"
          >
            {$_("resolutions.vote.unitLabel")}
          </label>
          <select
            id="unit-{resolution.id}"
            bind:value={selectedUnitId}
            class="w-full px-2 py-1.5 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            data-testid="vote-unit-select"
          >
            {#each myUnits as u (u.unit_id)}
              <option value={u.unit_id}
                >{$_("tickets.unit")}
                {u.unit_number || u.unit_id.slice(0, 8)}</option
              >
            {/each}
          </select>
        </div>
      {/if}

      <div class="grid grid-cols-2 gap-3 mb-3">
        <div>
          <label
            for="voting-power-{resolution.id}"
            class="block text-xs font-medium text-gray-700 mb-1"
          >
            {$_("resolutions.vote.votingPowerLabel")}
          </label>
          <input
            id="voting-power-{resolution.id}"
            type="number"
            bind:value={votingPower}
            min="1"
            max="10000"
            class="w-full px-2 py-1.5 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            data-testid="vote-voting-power"
          />
        </div>
        <div>
          <label
            for="proxy-{resolution.id}"
            class="block text-xs font-medium text-gray-700 mb-1"
          >
            {$_("resolutions.vote.proxyLabel")}
          </label>
          <input
            id="proxy-{resolution.id}"
            type="text"
            bind:value={proxyOwnerId}
            placeholder={$_("resolutions.vote.proxyPlaceholder")}
            class="w-full px-2 py-1.5 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            data-testid="vote-proxy-input"
          />
        </div>
      </div>

      <button
        onclick={handleVote}
        data-testid="resolution-vote-submit-button"
        disabled={!voteChoice || submittingVote}
        class="w-full py-2 px-4 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {#if submittingVote}
          {$_("resolutions.vote.submitting")}
        {:else}
          {$_("resolutions.vote.submitButton")}
        {/if}
      </button>
    </div>
  {/if}

  <div class="flex items-center gap-2">
    <button
      onclick={showVotes ? () => (showVotes = false) : loadVotes}
      data-testid="resolution-votes-toggle-button"
      class="text-xs text-indigo-600 hover:text-indigo-800 underline"
      disabled={loadingVotes}
    >
      {#if loadingVotes}
        {$_("common.loading")}
      {:else if showVotes}
        {$_("resolutions.vote.hideVotes")}
      {:else}
        {$_("resolutions.vote.viewVotes", { values: { count: totalVotes } })}
      {/if}
    </button>

    {#if canCloseVoting && totalVotes > 0}
      <button
        onclick={handleCloseVoting}
        disabled={closingVoting}
        class="text-xs text-orange-600 hover:text-orange-800 underline ml-auto"
        data-testid="vote-close-btn"
      >
        {closingVoting
          ? $_("resolutions.vote.closing")
          : $_("resolutions.vote.closeButton")}
      </button>
    {/if}
  </div>

  {#if showVotes && votes.length > 0}
    <div class="mt-3 border-t border-gray-100 pt-3">
      <!--
        Cinq colonnes — votant, lot, choix, puissance, date — dans un panneau
        déjà imbriqué. À 390 px elles se compriment jusqu'à l'illisible (#866).
      -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div class="overflow-x-auto" tabindex="0" role="region">
        <table class="min-w-[440px] w-full text-sm" data-testid="votes-table">
          <thead>
            <tr class="text-left text-xs text-gray-500 uppercase">
              <th scope="col" class="pb-2">{$_("resolutions.vote.voter")}</th>
              <th scope="col" class="pb-2">{$_("resolutions.vote.choice")}</th>
              <th scope="col" class="pb-2 text-right"
                >{$_("resolutions.vote.thousandths")}</th
              >
              <th scope="col" class="pb-2 text-right">{$_("common.date")}</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-50">
            {#each votes as vote}
              <tr>
                <td class="py-1.5">
                  <!-- `owner_name` n'est pas servi par l'API. Afficher huit
                     caractères d'UUID à un syndic ne l'aide en rien : on dit
                     plutôt que le nom n'a pas pu être résolu. Le vrai
                     correctif est d'enrichir le DTO — issues #786 et #765. -->
                  <span class="text-gray-900"
                    >{vote.owner_name ||
                      $_("resolutions.vote.unknownOwner")}</span
                  >
                  {#if vote.proxy_owner_id}
                    <span class="text-xs text-muted ml-1"
                      >({$_("resolutions.vote.proxy")})</span
                    >
                  {/if}
                </td>
                <td class="py-1.5">
                  <span
                    class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {getChoiceColor(
                      vote.vote_choice,
                    )}"
                  >
                    {getChoiceLabel(vote.vote_choice)}
                  </span>
                </td>
                <td class="py-1.5 text-right text-gray-600"
                  >{vote.voting_power}</td
                >
                <td class="py-1.5 text-right text-xs text-muted"
                  >{formatDateTime(vote.voted_at)}</td
                >
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {:else if showVotes && votes.length === 0}
    <p class="mt-3 text-xs text-muted text-center">
      {$_("resolutions.vote.noVotes")}
    </p>
  {/if}
</div>

<!-- Le dialogue qui remplace un `confirm()` natif (#844). -->
<ConfirmDialog
  isOpen={suppressionEnAttente}
  title={$_("common.confirm")}
  message={$_("resolutions.vote.closeConfirm")}
  variant="danger"
  onconfirm={executerLaction}
  oncancel={() => (suppressionEnAttente = false)}
/>
