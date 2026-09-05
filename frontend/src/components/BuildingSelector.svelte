<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../lib/i18n';
  import { api } from "../lib/api";
  import { withLoadingState } from "../lib/utils/error.utils";
  import { extractArray } from "../lib/utils/response.utils";

  let {
    selectedBuildingId = $bindable(""),
    label = "Immeuble",
    required = true,
    disabled = false,
    onSelect,
    onSelectBuilding,
    onSelectAcp,
  }: {
    selectedBuildingId?: string;
    label?: string;
    required?: boolean;
    disabled?: boolean;
    onSelect?: (buildingId: string) => void;
    onSelectBuilding?: (building: Building) => void;
    /**
     * Rappelée avec l'ACP du bloc choisi. C'est elle qui porte le périmètre au
     * sens de la loi : le dossier de gestion appartient à l'ACP, pas au bloc
     * ni au syndic (ADR-0045). Un immeuble n'en est qu'une porte d'entrée.
     */
    onSelectAcp?: (acpId: string) => void;
  } = $props();

  interface Building {
    id: string;
    name: string;
    address: string;
    city?: string;
    postal_code?: string;
    organization_id?: string;
    /** L'ACP dont ce bloc dépend (ADR-0045). */
    acp_id?: string;
  }

  interface Acp {
    id: string;
    name: string;
  }

  let buildings = $state<Building[]>([]);
  let acps = $state<Acp[]>([]);
  let loading = $state(true);
  let error = $state("");

  /**
   * Les blocs groupés par ACP, dans l'ordre des ACP puis des blocs.
   *
   * Une ACP peut compter plusieurs blocs — c'est même le cas visé par
   * l'Art. 3.85 § 1er, « immeuble ou groupe d'immeubles ». Les présenter à
   * plat laissait croire que chaque bloc était une copropriété distincte, ce
   * qui est faux dès qu'un groupe existe, et masquait que les charges se
   * répartissent au niveau de l'ACP.
   *
   * Les blocs dont l'ACP est inconnue sont regroupés à part plutôt que
   * masqués : les taire ferait disparaître des immeubles de l'écran sans que
   * personne comprenne pourquoi.
   */
  let groupes = $derived.by(() => {
    const parAcp = new Map<string, { nom: string; blocs: Building[] }>();
    for (const b of buildings) {
      const cle = b.acp_id ?? "__sans_acp__";
      if (!parAcp.has(cle)) {
        const acp = acps.find((a) => a.id === b.acp_id);
        parAcp.set(cle, {
          nom: acp?.name ?? (b.acp_id ? b.acp_id : $_("buildings.acpUnknown")),
          blocs: [],
        });
      }
      parAcp.get(cle)!.blocs.push(b);
    }
    return [...parAcp.entries()]
      .map(([id, g]) => ({ id, ...g }))
      .sort((x, y) => x.nom.localeCompare(y.nom, "fr"));
  });

  $effect(() => {
    loadBuildings();
    loadAcps();
  });

  /**
   * Les noms d'ACP servent à l'affichage seul. Leur échec ne doit pas empêcher
   * de choisir un immeuble : on retombe alors sur l'identifiant, ce qui reste
   * exploitable, plutôt que de bloquer l'écran.
   */
  async function loadAcps() {
    try {
      const reponse = await api.get("/acps?per_page=200");
      acps = extractArray<Acp>(reponse, "acps");
    } catch {
      acps = [];
    }
  }

  async function loadBuildings() {
    await withLoadingState({
      action: () => api.get("/buildings?per_page=100"),
      setLoading: (v: boolean) => loading = v,
      setError: (v: string) => error = v,
      errorMessage: $_('buildings.loadError'),
      onSuccess: (response) => {
        buildings = extractArray<Building>(response, 'buildings');

        if (buildings.length > 0 && !selectedBuildingId) {
          // Auto-select first building to ensure content loads immediately
          selectedBuildingId = buildings[0].id;
          setTimeout(() => {
            if (onSelect) onSelect(selectedBuildingId);
            if (onSelectBuilding) onSelectBuilding(buildings[0]);
          }, 0);
        } else if (buildings.length > 0 && selectedBuildingId) {
          const selected = buildings.find(b => b.id === selectedBuildingId);
          setTimeout(() => {
            if (onSelect) onSelect(selectedBuildingId);
            if (selected && onSelectBuilding) onSelectBuilding(selected);
          }, 0);
        }
      },
    });
  }
</script>

{#if loading}
  <div class="text-sm text-gray-500 py-2" data-testid="loading-spinner">{$_('buildings.loading')}</div>
{:else if error}
  <div class="p-3 bg-red-50 border border-red-200 rounded-md" data-testid="building-selector-error">
    <p class="text-sm text-red-800">{error}</p>
    <button
      onclick={loadBuildings}
      class="mt-2 text-sm text-red-700 underline hover:text-red-900"
      data-testid="building-selector-retry"
    >
      {$_('common.retry')}
    </button>
  </div>
{:else if buildings.length === 0}
  <div class="p-3 bg-red-50 border border-red-200 rounded-md" data-testid="building-selector-empty">
    <p class="text-sm text-red-800">
      {$_('buildings.noBuildings')}
    </p>
  </div>
{:else if buildings.length === 1}
  <div data-testid="building-selected">
    <span class="block text-sm font-medium text-gray-700">{label}</span>
    <div class="mt-1 px-3 py-2 bg-gray-50 border border-gray-200 rounded-md text-sm text-gray-700">
      {buildings[0].name} — {buildings[0].address}{#if buildings[0].city}, {buildings[0].postal_code} {buildings[0].city}{/if}
    </div>
  </div>
{:else}
  <div>
    <label for="building-selector" class="block text-sm font-medium text-gray-700">
      {label} {#if required}<span class="text-red-500">*</span>{/if}
    </label>
    <select
      id="building-selector"
      bind:value={selectedBuildingId}
      data-testid="building-selector"
      onchange={() => {
        if (selectedBuildingId) {
          if (onSelect) onSelect(selectedBuildingId);
          const selected = buildings.find(b => b.id === selectedBuildingId);
          if (selected && onSelectBuilding) onSelectBuilding(selected);
          if (selected?.acp_id && onSelectAcp) onSelectAcp(selected.acp_id);
        }
      }}
      {required}
      {disabled}
      class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
    >
      <option value="">{$_('buildings.selectBuilding')}</option>
      {#each groupes as groupe (groupe.id)}
        <optgroup label={groupe.nom}>
          {#each groupe.blocs as building (building.id)}
            <option value={building.id}>
              {building.name} — {building.address}{#if building.city}, {building.postal_code} {building.city}{/if}
            </option>
          {/each}
        </optgroup>
      {/each}
    </select>
  </div>
{/if}
