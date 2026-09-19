<script lang="ts">
  // Story #798 — AcpSelector (le périmètre PRINCIPAL, à côté du filtre
  // BuildingSelector.svelte qui survit inchangé).
  //
  // Le fait juridique qui commande ce composant : l'ACP est la personne
  // morale (numéro BCE, compte bancaire, AG, quotités totalisant 1000 PAR
  // ACP) — pas l'immeuble. Une ACP peut couvrir plusieurs blocs (ACP
  // principale + secondaires, droit belge) : la relation ACP <-> immeuble
  // n'est PAS 1:1, donc choisir une ACP ne peut jamais déduire un immeuble
  // unique.
  //
  // Comportement :
  // - Dropdown + filtre client (PAS de debounce réseau : `GET /acps` n'a
  //   aucun paramètre `?search=` côté backend — cf. `acp_handlers::list_acps`
  //   — et le portefeuille ACP d'un utilisateur est une liste courte, pas des
  //   milliers de lignes comme les immeubles). On charge une fois au premier
  //   focus, on filtre en mémoire à chaque frappe.
  // - Conditionné par rôle : visible si syndic/accountant/superadmin, masqué
  //   pour owner et user null (même contrat RBAC que BuildingSelector).
  // - Pose `scope.selectedAcpId` via `setAcp()`. Un immeuble déjà sélectionné
  //   qui n'appartient PAS à l'ACP choisie est effacé (le filtre secondaire
  //   ne doit jamais pointer hors du périmètre principal) ; un immeuble de la
  //   MÊME ACP survit intact.
  // - Affiche `acp-selector-403` quand `scope.scopeError === 'forbidden'`
  //   (posé par `demanderAcp` — cf. `stores/scope.svelte.ts`).
  //
  // data-testid (contrat NEUF — cf. docs/refonte/CONTRAT_DE_TESTS.md §4, ne
  // réemploie jamais `building-selector-*`) :
  //   acp-selector-root, -input, -result-{id}, -favourite-{id}, -clear,
  //   -empty, -403, -listbox
  //
  // Tests : voir `__tests__/AcpSelector.test.ts` (Vitest 4-cat).

  import { _ } from "../../lib/i18n";
  import type { User } from "../../lib/types";
  import { UserRole } from "../../lib/types";
  import { scope, setAcp, setBuilding } from "../../stores/scope.svelte";
  import { listAcps, type AcpResponseDto } from "../../lib/api/acps";

  interface Props {
    user: User | null;
    /** Max results rendered in the dropdown — default 20 (AC @edge). */
    maxResults?: number;
  }

  let { user, maxResults = 20 }: Props = $props();

  // -------------------------------------------------------------------------
  // RBAC gate — owner / null user → return null (composant invisible).
  // -------------------------------------------------------------------------
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
  let allAcps = $state<AcpResponseDto[]>([]);
  let isOpen = $state<boolean>(false);
  let favourites = $state<Set<string>>(new Set());
  let loaded = $state<boolean>(false);

  /**
   * Filtre en mémoire — pas de round-trip réseau par frappe (cf. commentaire
   * de tête : pas de `?search=` côté backend, et la liste est courte).
   */
  let results = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const filtered =
      q === ""
        ? allAcps
        : allAcps.filter((a) => a.name.toLowerCase().includes(q));
    return filtered.slice(0, maxResults);
  });

  // -------------------------------------------------------------------------
  // Bootstrap — charge le portefeuille ACP au premier focus (pas au montage :
  // la barre de périmètre est présente sur chaque page, cf. BuildingSelector).
  // -------------------------------------------------------------------------

  async function chargerPortefeuille(): Promise<void> {
    if (loaded) return;
    loaded = true;
    try {
      allAcps = await listAcps();
    } catch {
      // Fallback gracieux — le sélecteur reste utilisable, liste vide.
      allAcps = [];
    }
  }

  function auPremierFocus(): void {
    isOpen = true;
    void chargerPortefeuille();
  }

  function onInput(event: Event): void {
    const target = event.target as HTMLInputElement;
    query = target.value;
    isOpen = true;
    // Filet de sécurité : si l'utilisateur tape sans avoir déclenché
    // `auPremierFocus` (rare en usage réel — un champ se focus avant de
    // recevoir une frappe — mais possible en test), `chargerPortefeuille`
    // est idempotent (`loaded`) et ne déclenche donc jamais un second appel
    // réseau si le focus l'a déjà fait.
    void chargerPortefeuille();
  }

  // -------------------------------------------------------------------------
  // Sélection / clear / favoris
  // -------------------------------------------------------------------------

  /**
   * Choisir une ACP pose le périmètre principal. Un immeuble déjà sélectionné
   * qui appartenait à une AUTRE ACP est effacé : le laisser pointer vers un
   * bloc hors du nouveau périmètre serait un filtre incohérent. Un immeuble
   * de la MÊME ACP (celle qu'on vient de confirmer) survit intact — c'est la
   * preuve que la relation ACP <-> immeuble n'est pas 1:1 : choisir l'ACP ne
   * décide jamais, à elle seule, d'un bloc précis.
   */
  function onResultClick(acp: AcpResponseDto): void {
    const buildingAcpId = scope.selectedBuilding?.acp_id ?? null;
    setAcp(acp.id);
    if (scope.selectedBuildingId !== null && buildingAcpId !== acp.id) {
      setBuilding(null);
    }
    isOpen = false;
    query = acp.name;
  }

  function onResultKeydown(event: KeyboardEvent, acp: AcpResponseDto): void {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onResultClick(acp);
    }
  }

  function onClear(): void {
    query = "";
    isOpen = false;
    setAcp(null);
    if (scope.selectedBuildingId !== null) {
      setBuilding(null);
    }
  }

  function toggleFavourite(acpId: string): void {
    if (favourites.has(acpId)) {
      favourites.delete(acpId);
    } else {
      favourites.add(acpId);
    }
    favourites = new Set(favourites);
  }
</script>

{#if visible}
  <div
    class="acp-selector relative w-full max-w-md"
    data-testid="acp-selector-root"
  >
    {#if scope.scopeError === "forbidden"}
      <div
        data-testid="acp-selector-403"
        role="alert"
        class="absolute -top-10 left-0 right-0 bg-red-50 border border-red-200 text-red-800 text-sm rounded px-3 py-2"
      >
        {$_("scope.forbidden") || "Accès refusé"}
      </div>
    {/if}

    <div class="relative">
      <input
        type="text"
        data-testid="acp-selector-input"
        value={query}
        oninput={onInput}
        onfocus={auPremierFocus}
        role="combobox"
        aria-autocomplete="list"
        aria-expanded={isOpen}
        aria-controls="acp-selector-listbox"
        aria-label={$_("scope.selectAcp") || "Sélectionner une ACP"}
        placeholder={$_("scope.searchAcpPlaceholder") || "Rechercher une ACP…"}
        class="min-h-11 w-full rounded border border-gray-300 py-2 pl-3 pr-12 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
      />

      {#if scope.selectedAcpId !== null}
        <button
          type="button"
          data-testid="acp-selector-clear"
          onclick={onClear}
          aria-label={$_("scope.clear") || "Effacer la sélection"}
          class="absolute right-0 top-1/2 flex h-11 w-11 -translate-y-1/2 items-center justify-center text-muted hover:text-gray-700"
        >
          ×
        </button>
      {:else}
        <span
          aria-hidden="true"
          class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-xs text-muted"
        >
          ▾
        </span>
      {/if}
    </div>

    {#if isOpen}
      <ul
        id="acp-selector-listbox"
        data-testid="acp-selector-listbox"
        role="listbox"
        class="absolute z-50 mt-1 max-h-80 w-full overflow-auto rounded border border-gray-200 bg-white shadow-lg"
      >
        {#if results.length === 0}
          <li
            data-testid="acp-selector-empty"
            role="option"
            aria-selected="false"
            class="px-3 py-2 text-sm text-gray-500"
          >
            {$_("scope.noAcps") || "Aucune ACP dans votre portefeuille"}
          </li>
        {:else}
          {#each results as acp (acp.id)}
            <li
              data-testid="acp-selector-result-{acp.id}"
              role="option"
              aria-selected={scope.selectedAcpId === acp.id}
              tabindex="0"
              onclick={() => onResultClick(acp)}
              onkeydown={(e) => onResultKeydown(e, acp)}
              class="flex cursor-pointer items-center justify-between px-3 py-2 text-sm hover:bg-gray-100 focus:bg-gray-100 focus:outline-none"
            >
              <div class="flex flex-col">
                <span class="font-medium text-gray-900">{acp.name}</span>
                <span class="text-xs text-gray-500">{acp.address_city}</span>
              </div>
              <button
                type="button"
                data-testid="acp-selector-favourite-{acp.id}"
                onclick={(e) => {
                  e.stopPropagation();
                  toggleFavourite(acp.id);
                }}
                aria-label={favourites.has(acp.id)
                  ? $_("scope.unfavorite") || "Retirer des favoris"
                  : $_("scope.favorite") || "Ajouter aux favoris"}
                aria-pressed={favourites.has(acp.id)}
                class="-mr-1 ml-2 flex h-11 w-11 shrink-0 items-center justify-center rounded text-yellow-500 hover:bg-gray-100 hover:text-yellow-700"
              >
                {favourites.has(acp.id) ? "★" : "☆"}
              </button>
            </li>
          {/each}
        {/if}
      </ul>
    {/if}
  </div>
{/if}

<style>
  .acp-selector {
    min-width: 16rem;
  }
</style>
