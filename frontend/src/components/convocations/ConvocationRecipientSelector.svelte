<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from "../../lib/i18n";
  import {
    convocationsApi,
    type EligibleRecipient,
  } from "../../lib/api/convocations";
  import { withLoadingState } from "../../lib/utils/error.utils";

  let {
    buildingId,
    onchange,
  }: {
    buildingId: string;
    onchange: (ownerIds: string[]) => void;
  } = $props();

  let eligibles = $state<EligibleRecipient[]>([]);
  let selected = $state<Set<string>>(new Set());
  let loading = $state(true);
  let error = $state("");

  $effect(() => {
    loadEligibles();
  });

  async function loadEligibles() {
    await withLoadingState({
      action: () => convocationsApi.getEligibleRecipients(buildingId),
      setLoading: (v: boolean) => (loading = v),
      setError: (v: string) => (error = v),
      onSuccess: (data: EligibleRecipient[]) => {
        eligibles = data;
        selected = new Set(data.map((o) => o.owner_id));
        // Une liste vide n'est pas un renoncement : il n'y avait rien à
        // choisir. On ne remonte alors rien au parent, qui omettra le champ
        // et laissera le serveur donner le refus légitime (« cet immeuble
        // n'a pas de lot attribué »), au lieu d'un « sélectionnez au moins
        // un destinataire » trompeur pour un cas où il n'y en avait aucun.
        // Sinon, tout le monde est sélectionné par défaut : c'est ce que le
        // serveur ferait de toute façon si la sélection n'était pas touchée.
        if (data.length > 0) {
          emit();
        }
      },
      errorMessage: $_("convocations.errors.loadingEligibleRecipientsFailed"),
    });
  }

  function emit() {
    onchange(Array.from(selected));
  }

  function toggle(ownerId: string) {
    const next = new Set(selected);
    if (next.has(ownerId)) {
      next.delete(ownerId);
    } else {
      next.add(ownerId);
    }
    selected = next;
    emit();
  }

  function toggleAll() {
    selected =
      selected.size === eligibles.length
        ? new Set()
        : new Set(eligibles.map((o) => o.owner_id));
    emit();
  }
</script>

<div
  class="bg-gray-50 border border-gray-200 rounded-lg p-4"
  data-testid="convocation-recipient-selector"
>
  <div class="flex items-center justify-between mb-2">
    <h4 class="text-sm font-semibold text-gray-900">
      {$_("convocations.recipientSelector.title")}
    </h4>
    {#if !loading && !error && eligibles.length > 0}
      <button
        type="button"
        onclick={toggleAll}
        data-testid="convocation-recipient-selector-toggle-all"
        class="text-xs text-amber-700 hover:text-amber-900 underline"
      >
        {selected.size === eligibles.length
          ? $_("convocations.recipientSelector.selectNone")
          : $_("convocations.recipientSelector.selectAll")}
      </button>
    {/if}
  </div>

  {#if loading}
    <div class="py-4 text-center">
      <div
        class="inline-block animate-spin rounded-full h-5 w-5 border-b-2 border-amber-600"
      ></div>
    </div>
  {:else if error}
    <p class="text-sm text-red-600">{error}</p>
  {:else if eligibles.length === 0}
    <p
      class="text-sm text-amber-800"
      data-testid="convocation-recipient-selector-empty"
    >
      {$_("convocations.recipientSelector.noEligibleOwners")}
    </p>
  {:else}
    <ul class="space-y-1 max-h-48 overflow-y-auto">
      {#each eligibles as owner (owner.owner_id)}
        <li>
          <label class="flex items-center gap-2 text-sm text-gray-800">
            <input
              type="checkbox"
              checked={selected.has(owner.owner_id)}
              onchange={() => toggle(owner.owner_id)}
              data-testid="convocation-recipient-selector-checkbox-{owner.owner_id}"
            />
            <span class="font-medium">{owner.full_name}</span>
            <span class="text-gray-500 text-xs">{owner.email}</span>
          </label>
        </li>
      {/each}
    </ul>
    <p
      class="mt-2 text-xs {selected.size === 0
        ? 'text-red-700 font-medium'
        : 'text-gray-500'}"
      data-testid="convocation-recipient-selector-count"
    >
      {selected.size === 0
        ? $_("convocations.recipientSelector.noneSelectedWarning")
        : $_("convocations.recipientSelector.selectedCount", {
            values: { count: selected.size },
          })}
    </p>
  {/if}
</div>
