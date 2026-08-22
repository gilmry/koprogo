<script lang="ts">
  // Story 2.2 — BuildingSelector (composant global top-left).
  //
  // ADR-0011 (Portefeuille) + ADR-0012 (Navigation contextualisée).
  //
  // Comportement :
  // - Dropdown + autocomplete débouncé (150ms) + favoris (star) +
  //   portefeuilles équipe (lecture seule, surfacé depuis /portfolios).
  // - Conditionné par rôle : visible si syndic/admin/accountant/superadmin,
  //   masqué pour owner et user null (cf. AC @security).
  // - Sync le store `scope.svelte.ts` (selectedBuildingId / selectedAcpId).
  // - Affiche un état 403 si la sélection est rejetée hors scope.
  //
  // data-testid (contrat stable, i18n-safe — cf. memory data-testid-systematic) :
  //   building-selector-input, -result-{id}, -favorite-{id}, -clear,
  //   -empty, -403
  //
  // Tests : voir `__tests__/BuildingSelector.test.ts` (Vitest 4-cat) et
  // `frontend/tests/e2e/refonte-ux/slice-2-selector-banner/building-selector.spec.ts`
  // (Playwright multi-rôle).

  import { onMount } from "svelte";
  import { _ } from "../../lib/i18n";
  import type { Building, User } from "../../lib/types";
  import { UserRole } from "../../lib/types";
  import {
    scope,
    setBuilding,
    setScopeError,
    resetScope,
  } from "../../stores/scope.svelte";
  import { searchBuildings } from "../../lib/api/buildings";
  import {
    listPortfolios,
    type PortfolioResponseDto,
  } from "../../lib/api/portfolios";

  interface Props {
    user: User | null;
    /**
     * Debounce delay in ms — default 150 (AC @edge). Exposed for tests
     * to optionally lower in unit tests.
     */
    debounceMs?: number;
    /** Max results rendered in the dropdown — default 20 (AC @edge). */
    maxResults?: number;
  }

  let { user, debounceMs = 150, maxResults = 20 }: Props = $props();

  // -------------------------------------------------------------------------
  // RBAC gate — owner / null user → return null (composant invisible).
  // -------------------------------------------------------------------------
  // Listé en haut pour court-circuiter tout rendu / fetch pour les owners.
  // Le test `@security` vérifie qu'AUCUN sous-arbre n'est rendu.
  const visibleForRole = (u: User | null): boolean => {
    if (!u) return false;
    switch (u.role) {
      case UserRole.SYNDIC:
      case UserRole.ACCOUNTANT:
      case UserRole.SUPERADMIN:
        return true;
      case UserRole.OWNER:
        return false;
      default:
        // Conservateur : tout rôle inconnu → caché.
        return false;
    }
  };

  let visible = $derived(visibleForRole(user));

  // -------------------------------------------------------------------------
  // State local
  // -------------------------------------------------------------------------

  let query = $state<string>("");
  let results = $state<Building[]>([]);
  let isOpen = $state<boolean>(false);
  let portfolios = $state<PortfolioResponseDto[]>([]);
  let favorites = $state<Set<string>>(new Set());
  let lastQueryAt = $state<number>(0);
  let debounceHandle: ReturnType<typeof setTimeout> | null = null;

  // -------------------------------------------------------------------------
  // Bootstrap — charge portefeuilles (favoris-first ordering Story 2.5+).
  // -------------------------------------------------------------------------

  onMount(async () => {
    if (!visible) return;
    try {
      portfolios = await listPortfolios();
    } catch {
      // Silencieux — le selector reste utilisable sans portefeuilles.
      portfolios = [];
    }
  });

  // -------------------------------------------------------------------------
  // Recherche débouncée
  // -------------------------------------------------------------------------

  function doSearch(text: string): void {
    const queryAt = Date.now();
    lastQueryAt = queryAt;
    searchBuildings(text, maxResults)
      .then((found) => {
        // Anti race : si une frappe plus récente a déjà mis à jour
        // lastQueryAt, on ignore les résultats obsolètes (cf. mémoire #550
        // équivalent FE : in-flight déduplication).
        if (queryAt !== lastQueryAt) return;
        results = found.slice(0, maxResults);
        isOpen = true;
      })
      .catch(() => {
        if (queryAt !== lastQueryAt) return;
        // Fallback gracieux — vide la liste mais l'input reste utilisable.
        results = [];
        isOpen = true;
      });
  }

  function onInput(event: Event): void {
    const target = event.target as HTMLInputElement;
    query = target.value;
    if (debounceHandle) clearTimeout(debounceHandle);
    debounceHandle = setTimeout(() => {
      doSearch(query);
    }, debounceMs);
  }

  // -------------------------------------------------------------------------
  // Sélection / clear / favoris
  // -------------------------------------------------------------------------

  function onResultClick(b: Building): void {
    setBuilding(b);
    isOpen = false;
    query = b.name;
  }

  function onResultKeydown(event: KeyboardEvent, b: Building): void {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onResultClick(b);
    }
  }

  function onClear(): void {
    query = "";
    results = [];
    isOpen = false;
    resetScope();
  }

  function toggleFavorite(buildingId: string): void {
    if (favorites.has(buildingId)) {
      favorites.delete(buildingId);
    } else {
      favorites.add(buildingId);
    }
    // Reactive trigger — assign back to surface mutation in templates.
    favorites = new Set(favorites);
    // TODO Story 2.5 : POST vers /portfolios/{id}/buildings avec
    // is_favorite=true quand portfolio par défaut sélectionné.
  }
</script>

{#if visible}
  <div
    class="building-selector relative w-full max-w-md"
    data-testid="building-selector-root"
  >
    {#if scope.scopeError === "forbidden"}
      <div
        data-testid="building-selector-403"
        role="alert"
        class="absolute -top-10 left-0 right-0 bg-red-50 border border-red-200 text-red-800 text-sm rounded px-3 py-2"
      >
        {$_("scope.forbidden") || "Accès refusé"}
      </div>
    {/if}

    <div class="relative">
      <input
        type="text"
        data-testid="building-selector-input"
        value={query}
        oninput={onInput}
        onfocus={() => {
          isOpen = results.length > 0;
        }}
        role="combobox"
        aria-autocomplete="list"
        aria-expanded={isOpen}
        aria-controls="building-selector-listbox"
        aria-label={$_("scope.selectBuilding") || "Sélectionner un immeuble"}
        placeholder={$_("scope.searchPlaceholder") || "Rechercher un immeuble…"}
        class="w-full rounded border border-gray-300 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
      />

      {#if scope.selectedBuildingId !== null}
        <button
          type="button"
          data-testid="building-selector-clear"
          onclick={onClear}
          aria-label={$_("scope.clear") || "Effacer la sélection"}
          class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-700"
        >
          ×
        </button>
      {:else}
        <span
          aria-hidden="true"
          class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-xs text-gray-400"
        >
          ▾
        </span>
      {/if}
    </div>

    {#if isOpen}
      <ul
        id="building-selector-listbox"
        role="listbox"
        class="absolute z-50 mt-1 max-h-80 w-full overflow-auto rounded border border-gray-200 bg-white shadow-lg"
      >
        {#if results.length === 0}
          <li
            data-testid="building-selector-empty"
            role="option"
            aria-selected="false"
            class="px-3 py-2 text-sm text-gray-500"
          >
            {$_("scope.noBuildings") ||
              "Aucun immeuble dans votre périmètre"}
          </li>
        {:else}
          {#each results as b (b.id)}
            <li
              data-testid="building-selector-result-{b.id}"
              role="option"
              aria-selected={scope.selectedBuildingId === b.id}
              tabindex="0"
              onclick={() => onResultClick(b)}
              onkeydown={(e) => onResultKeydown(e, b)}
              class="flex cursor-pointer items-center justify-between px-3 py-2 text-sm hover:bg-gray-100 focus:bg-gray-100 focus:outline-none"
            >
              <div class="flex flex-col">
                <span class="font-medium text-gray-900">{b.name}</span>
                <span class="text-xs text-gray-500">{b.city ?? ""}</span>
              </div>
              <button
                type="button"
                data-testid="building-selector-favorite-{b.id}"
                onclick={(e) => {
                  e.stopPropagation();
                  toggleFavorite(b.id);
                }}
                aria-label={favorites.has(b.id)
                  ? $_("scope.unfavorite") || "Retirer des favoris"
                  : $_("scope.favorite") || "Ajouter aux favoris"}
                aria-pressed={favorites.has(b.id)}
                class="ml-2 text-yellow-500 hover:text-yellow-700"
              >
                {favorites.has(b.id) ? "★" : "☆"}
              </button>
            </li>
          {/each}
        {/if}
      </ul>
    {/if}

    {#if portfolios.length > 0}
      <div class="mt-1 text-xs text-gray-500">
        {portfolios.length} {$_("scope.portfolios") || "portefeuilles"}
      </div>
    {/if}
  </div>
{/if}

<style>
  .building-selector {
    min-width: 16rem;
  }
</style>
