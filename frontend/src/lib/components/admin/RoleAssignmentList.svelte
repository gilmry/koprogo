<script lang="ts">
  // Story B1 (Phase B FE) — RoleAssignmentList.
  //
  // Table Svelte 5 (runes) listant les assignments de sous-rôles métier
  // / mandataires. Colonnes : User, Role, Organisation, Expire, Actions.
  //
  // INV-FE1 : `data-testid` stable par row (`role-assignment-row-{id}`) et
  // par action (`role-assignment-revoke-{id}`). Badge expiration réutilise
  // le composant atomique B3 `ExpirationBadge` (palette + label daltonien).
  //
  // INV-FE2 : table sémantique (caption + thead + tbody) + tap targets ≥ 44px
  // sur les boutons révoquer.
  //
  // Filtres : `organizationId` optionnel — si fourni, on appelle
  // `GET /role-assignments?organization_id=…` (superadmin/syndic ; cf. note
  // Story B0bis 8ac5a83). Sinon liste globale superadmin.
  //
  // Optimistic update : à la révocation OK, on retire la row localement
  // sans re-fetch full (cf. memory anti-loading-cascade) ; un échec restore
  // la row + affiche un toast d'erreur (api.ts auto).

  import {
    listAssignments,
    revokeAssignment,
    type RoleAssignment,
  } from "../../api/role_assignments";
  import ExpirationBadge from "../shared/ExpirationBadge.svelte";
  import { expirationStatus } from "../../utils/dateBadge";

  let {
    organizationId = undefined,
    /** Refresh signal — incrément pour forcer un reload (parent peut s'en
     * servir après création depuis le form). */
    refreshTrigger = 0,
  }: {
    organizationId?: string | undefined;
    refreshTrigger?: number;
  } = $props();

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  let assignments = $state<RoleAssignment[]>([]);
  let loading = $state(true);
  let revoking = $state<Record<string, boolean>>({});
  let loadError = $state<string>("");

  // ---------------------------------------------------------------------------
  // Load — `$effect` se déclenche au mount + à chaque changement de dépendance
  // (refreshTrigger, organizationId). Pas besoin d'`onMount` séparé (évite
  // le double-fetch au premier rendu observé en tests).
  // ---------------------------------------------------------------------------

  $effect(() => {
    refreshTrigger;
    organizationId;
    void load();
  });

  async function load(): Promise<void> {
    loading = true;
    loadError = "";
    try {
      const items = await listAssignments(
        organizationId ? { organization_id: organizationId } : undefined,
      );
      assignments = items ?? [];
    } catch (err) {
      loadError =
        err instanceof Error ? err.message : "Erreur de chargement.";
      assignments = [];
    } finally {
      loading = false;
    }
  }

  async function handleRevoke(a: RoleAssignment): Promise<void> {
    if (revoking[a.id]) return;
    revoking = { ...revoking, [a.id]: true };
    // Optimistic — on retire localement, on restore en cas d'échec.
    const previous = assignments;
    assignments = assignments.filter((x) => x.id !== a.id);
    try {
      await revokeAssignment(a.user_id, a.id);
    } catch {
      // Restore — toast déjà géré par api.ts.
      assignments = previous;
    } finally {
      revoking = { ...revoking, [a.id]: false };
    }
  }
</script>

<section aria-labelledby="role-assignment-list-title" class="space-y-4">
  <header class="flex items-center justify-between">
    <h2
      id="role-assignment-list-title"
      class="text-lg font-semibold text-gray-900"
    >
      Assignations de rôles
    </h2>
  </header>

  {#if loading}
    <p
      data-testid="role-assignment-loading"
      class="text-sm text-gray-500"
      aria-live="polite"
    >
      Chargement…
    </p>
  {:else if loadError}
    <p
      data-testid="role-assignment-load-error"
      class="text-sm text-red-600"
      role="alert"
    >
      {loadError}
    </p>
  {:else if assignments.length === 0}
    <p
      data-testid="role-assignment-empty"
      class="text-sm text-gray-500 italic p-4 border border-dashed border-gray-300 rounded-lg"
    >
      Aucune assignation pour le moment.
    </p>
  {:else}
    <div class="overflow-x-auto">
      <table
        data-testid="role-assignment-list"
        class="min-w-full divide-y divide-gray-200 text-sm"
      >
        <caption class="sr-only">Liste des assignations de rôles actives</caption>
        <thead class="bg-gray-50">
          <tr>
            <th
              scope="col"
              class="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >Utilisateur</th
            >
            <th
              scope="col"
              class="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >Rôle</th
            >
            <th
              scope="col"
              class="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >Organisation</th
            >
            <th
              scope="col"
              class="px-3 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >Expire</th
            >
            <th
              scope="col"
              class="px-3 py-2 text-right text-xs font-medium text-gray-500 uppercase tracking-wider"
              >Actions</th
            >
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-100">
          {#each assignments as a (a.id)}
            <tr data-testid={`role-assignment-row-${a.id}`}>
              <td class="px-3 py-2 text-gray-700 font-mono text-xs">
                {a.user_id}
              </td>
              <td class="px-3 py-2 text-gray-900">{a.role}</td>
              <td class="px-3 py-2 text-gray-700 font-mono text-xs">
                {a.organization_id ?? "—"}
              </td>
              <td class="px-3 py-2">
                {#if a.valid_until}
                  <!-- Wrapper `role-assignment-expiration-badge-{id}` (cf.
                       stories.md §B1 data-testid). Le badge interne reste
                       atomique (réutilisable B3/B4/B6) avec son propre
                       `expiration-badge-{idSuffix}`. `data-level` est répliqué
                       sur le wrapper pour les assertions e2e/Vitest. -->
                  <span
                    data-testid={`role-assignment-expiration-badge-${a.id}`}
                    data-level={expirationStatus(a.valid_until).level}
                  >
                    <ExpirationBadge
                      validUntil={a.valid_until}
                      idSuffix={a.id}
                    />
                  </span>
                {:else}
                  <span class="text-gray-400" aria-label="Permanent">∞</span>
                {/if}
              </td>
              <td class="px-3 py-2 text-right">
                <button
                  type="button"
                  data-testid={`role-assignment-revoke-${a.id}`}
                  onclick={() => handleRevoke(a)}
                  disabled={revoking[a.id]}
                  aria-label={`Révoquer l'assignation ${a.role} de ${a.user_id}`}
                  class="min-h-[44px] min-w-[44px] inline-flex items-center justify-center px-3 py-2 rounded-md text-red-600 hover:bg-red-50 disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-500"
                >
                  {revoking[a.id] ? "…" : "Révoquer"}
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>
