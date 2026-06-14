<script lang="ts">
  // Story B3 (Phase B FE) — Liste des mandats émis (vue syndic).
  //
  // Affiche un tableau scrollable avec colonnes :
  //   Subject | Kind | Scope | Émis le | Expire (<ExpirationBadge>) | Action
  //
  // Réutilise `ExpirationBadge` (composant atomique partagé B3/B4/B6) — la
  // logique countdown vit dans `lib/utils/dateBadge.ts`, pas ici (cf. notes
  // anti-pattern stories.md §B3).
  //
  // Pattern fetch : on accepte une prop `initialMandates` pour faciliter le
  // SSR / les tests (pas de fetch synchrone en bord de composant), et on
  // expose `refresh()` via le parent qui peut piloter le rechargement après
  // un POST réussi (callback `onIssued` du `MandateIssueForm`).
  //
  // Confirmation revoke : pattern modal léger inline — pas de nouvelle lib.
  // Le revoke est destructif côté audit (le mandat est marqué `revoked_at`),
  // donc on demande confirmation utilisateur. Backend renvoie 204.
  //
  // data-testid (cf. stories.md §B3) :
  //   mandate-list
  //   mandate-row-{id}
  //   mandate-row-subject-{id}    mandate-row-kind-{id}   mandate-row-scope-{id}
  //   mandate-row-reason-{id}     mandate-expiration-badge-{id} (via ExpirationBadge idSuffix)
  //   mandate-revoke-{id}         mandate-revoke-confirm
  //   mandate-new-button          (CTA — émis depuis cette liste, modal côté parent)

  import { _ } from "../../i18n";
  import { toast } from "../../../stores/toast";
  import {
    revokeMandate,
    listMandates,
    type MandateResponse,
  } from "../../api/mandates";
  import ExpirationBadge from "../shared/ExpirationBadge.svelte";

  // -------------------------------------------------------------------------
  // Props
  // -------------------------------------------------------------------------

  let {
    /** Liste initiale (server-fetched ou injection test). Si non fournie, la
     *  liste fetche elle-même au mount via `listMandates()`. */
    initialMandates = undefined,
    /** Callback émis quand l'utilisateur clique le CTA "Nouveau mandat" — le
     *  parent ouvre le modal contenant `<MandateIssueForm>`. */
    onNew = undefined,
    /** Map id → label pour résoudre subject_user_id en nom lisible (optionnel
     *  — fallback UUID si non fourni). */
    subjectLabels = {},
    /** Map (scope_kind:id) → label pour résoudre scope en libellé immeuble. */
    scopeLabels = {},
    /** Injection clock pour `ExpirationBadge` (tests déterministes). */
    nowOverride = undefined,
  }: {
    initialMandates?: MandateResponse[];
    onNew?: () => void;
    subjectLabels?: Record<string, string>;
    scopeLabels?: Record<string, string>;
    nowOverride?: Date | undefined;
  } = $props();

  // -------------------------------------------------------------------------
  // State local
  // -------------------------------------------------------------------------

  // Initial state — on lit `initialMandates` ICI car $state ne peut pas
  // dépendre directement d'une prop sans capture initiale (Svelte 5 warning
  // `state_referenced_locally`). Le pattern :
  //   - si `initialMandates` est fourni → on l'utilise comme seed
  //   - si `undefined` → on fetche au mount via $effect ci-dessous
  let mandates = $state<MandateResponse[]>([]);
  let loading = $state<boolean>(false);
  /** ID du mandat en cours de confirmation revoke (null = pas de modal). */
  let pendingRevokeId = $state<string | null>(null);

  // -------------------------------------------------------------------------
  // Fetch initial si pas de liste injectée
  // -------------------------------------------------------------------------

  async function fetchMandates(): Promise<void> {
    loading = true;
    try {
      mandates = await listMandates();
    } catch {
      // toast déjà émis par api.ts pour 4xx/5xx
    } finally {
      loading = false;
    }
  }

  // Sync prop → state au mount + à chaque changement de prop. Si la prop
  // est `undefined`, on fetche au lieu de remplacer.
  $effect(() => {
    if (initialMandates !== undefined) {
      mandates = initialMandates;
    } else {
      void fetchMandates();
    }
  });

  // -------------------------------------------------------------------------
  // Actions
  // -------------------------------------------------------------------------

  function askRevoke(id: string): void {
    pendingRevokeId = id;
  }

  function cancelRevoke(): void {
    pendingRevokeId = null;
  }

  async function confirmRevoke(): Promise<void> {
    if (!pendingRevokeId) return;
    const id = pendingRevokeId;
    pendingRevokeId = null;
    try {
      await revokeMandate(id);
      toast.success($_("mandate.revoke.success") || "Mandat révoqué.");
      // Optimistic update : marque revoked_at côté client (le backend renvoie
      // 204, on ne refait pas un GET coûteux).
      mandates = mandates.map((m) =>
        m.id === id ? { ...m, revoked_at: new Date().toISOString() } : m,
      );
    } catch {
      // toast déjà émis par api.ts
    }
  }

  // -------------------------------------------------------------------------
  // Dérivations affichage
  // -------------------------------------------------------------------------

  function labelSubject(m: MandateResponse): string {
    return subjectLabels[m.subject_user_id] ?? m.subject_user_id.slice(0, 8);
  }

  function labelScope(m: MandateResponse): string {
    const key = `${m.scope_kind}:${m.scope_id}`;
    return scopeLabels[key] ?? `${m.scope_kind} ${m.scope_id.slice(0, 8)}`;
  }

  function formatIssued(m: MandateResponse): string {
    const d = new Date(m.created_at);
    return d.toISOString().slice(0, 10);
  }
</script>

<section
  class="mandate-list-section flex flex-col gap-4"
  aria-labelledby="mandate-list-title"
>
  <header class="flex items-center justify-between">
    <h2 id="mandate-list-title" class="text-lg font-semibold text-gray-900">
      {$_("mandate.list.title") || "Mandats émis"}
    </h2>
    <button
      type="button"
      data-testid="mandate-new-button"
      class="px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
      onclick={() => onNew?.()}
    >
      {$_("mandate.list.new") || "Nouveau mandat"}
    </button>
  </header>

  {#if loading && mandates.length === 0}
    <p class="text-sm text-gray-500" role="status" aria-live="polite">
      {$_("common.loading") || "Chargement…"}
    </p>
  {:else if mandates.length === 0}
    <p class="text-sm text-gray-500" data-testid="mandate-list-empty">
      {$_("mandate.list.empty") || "Aucun mandat émis pour le moment."}
    </p>
  {:else}
    <div class="overflow-x-auto">
      <table
        data-testid="mandate-list"
        class="min-w-full divide-y divide-gray-200 text-sm"
      >
        <thead class="bg-gray-50">
          <tr>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("mandate.col.subject") || "Mandataire"}
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("mandate.col.kind") || "Type"}
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("mandate.col.scope") || "Scope"}
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("mandate.col.reason") || "Motif"}
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("mandate.col.issued") || "Émis le"}
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("mandate.col.expire") || "Expire"}
            </th>
            <th scope="col" class="px-3 py-2 text-right font-medium text-gray-700">
              {$_("mandate.col.actions") || "Actions"}
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-100">
          {#each mandates as m (m.id)}
            <tr data-testid={`mandate-row-${m.id}`}>
              <td
                class="px-3 py-2 text-gray-900"
                data-testid={`mandate-row-subject-${m.id}`}
              >
                {labelSubject(m)}
              </td>
              <td
                class="px-3 py-2 text-gray-700"
                data-testid={`mandate-row-kind-${m.id}`}
              >
                {m.kind}
              </td>
              <td
                class="px-3 py-2 text-gray-700"
                data-testid={`mandate-row-scope-${m.id}`}
              >
                {labelScope(m)}
              </td>
              <td
                class="px-3 py-2 text-gray-600 max-w-xs truncate"
                title={m.reason}
                data-testid={`mandate-row-reason-${m.id}`}
              >
                {m.reason}
              </td>
              <td class="px-3 py-2 text-gray-600">{formatIssued(m)}</td>
              <td class="px-3 py-2">
                {#if m.revoked_at}
                  <span
                    class="inline-flex items-center gap-1 rounded border bg-gray-200 text-gray-700 border-gray-300 px-2 py-1 text-xs font-medium"
                    data-testid={`mandate-expiration-badge-${m.id}`}
                    role="status"
                    aria-label="Révoqué"
                  >
                    Révoqué
                  </span>
                {:else}
                  <ExpirationBadge
                    validUntil={m.valid_until}
                    idSuffix={`mandate-${m.id}`}
                    {nowOverride}
                  />
                {/if}
              </td>
              <td class="px-3 py-2 text-right">
                {#if !m.revoked_at}
                  <button
                    type="button"
                    data-testid={`mandate-revoke-${m.id}`}
                    class="text-xs text-red-600 hover:underline"
                    onclick={() => askRevoke(m.id)}
                    aria-label={`Révoquer le mandat ${m.id}`}
                  >
                    {$_("mandate.action.revoke") || "Révoquer"}
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  <!-- Modal de confirmation revoke (inline, pas de lib externe) -->
  {#if pendingRevokeId}
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
      role="dialog"
      aria-modal="true"
      aria-labelledby="mandate-revoke-confirm-title"
    >
      <div class="bg-white rounded shadow-lg p-6 max-w-sm w-full">
        <h3
          id="mandate-revoke-confirm-title"
          class="text-base font-semibold mb-2"
        >
          {$_("mandate.revoke.confirm.title") || "Confirmer la révocation"}
        </h3>
        <p class="text-sm text-gray-700 mb-4">
          {$_("mandate.revoke.confirm.body") ||
            "Cette action est irréversible (audit trail conservé)."}
        </p>
        <div class="flex justify-end gap-2">
          <button
            type="button"
            class="px-3 py-1 text-sm border border-gray-300 rounded text-gray-700"
            onclick={cancelRevoke}
          >
            {$_("common.cancel") || "Annuler"}
          </button>
          <button
            type="button"
            data-testid="mandate-revoke-confirm"
            class="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700"
            onclick={confirmRevoke}
          >
            {$_("mandate.action.revoke") || "Révoquer"}
          </button>
        </div>
      </div>
    </div>
  {/if}
</section>
