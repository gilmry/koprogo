<script lang="ts">
  // Story B4 (Phase B FE) — Liste des délégations actives (vue syndic).
  //
  // Affiche un tableau scrollable avec colonnes :
  //   Cible (user_id) | Rôle | Source (delegated_from) | Expire (<ExpirationBadge>) | Action
  //
  // Réutilise `ExpirationBadge` (composant atomique partagé B3/B4) — la
  // logique countdown vit dans `lib/utils/dateBadge.ts`, pas ici. Importer,
  // NE PAS recréer (cf. brief Story B4).
  //
  // Banner non-transitivité (cf. AC @security stories.md §B4) :
  //   Si le current user a HÉRITÉ son rôle via une délégation (i.e. il
  //   apparaît comme `user_id` d'une row avec `delegated_from_user_id` set),
  //   on affiche un BANNER persistant ROUGE en tête (role="alert"
  //   aria-live="polite") + le bouton "Nouvelle délégation" est ABSENT du
  //   DOM (pas juste disabled). Backend renvoie 403
  //   `DelegationChainNotAllowed` si bypass DevTools.
  //
  // Détection :
  //   - Prop `currentUserId` (passée par le parent qui lit l'authStore).
  //   - On scan `delegations` : si une row a `user_id === currentUserId` ET
  //     `delegated_from_user_id !== null` → hasInheritedRole = true.
  //   - Si `currentUserId` est `undefined` (auth pas encore hydraté) :
  //     conservative fallback — on affiche le banner avec wording adapté
  //     ("Si vous avez reçu votre rôle par délégation, la re-délégation est
  //     interdite") sans masquer le CTA. Le backend reste source de vérité
  //     (403 si tentative invalide).
  //
  // Pattern fetch : `initialDelegations` pour le rendu build Astro (SSG) /
  // tests, sinon fetch au mount via `listDelegationsOf()`.
  //
  // data-testid (cf. stories.md §B4) :
  //   role-delegation-list
  //   role-delegation-row-{id}
  //   role-delegation-expiration-badge-{id} (via ExpirationBadge idSuffix)
  //   role-delegation-revoke-{id} / role-delegation-revoke-confirm
  //   role-delegate-new-button (CTA — ABSENT si user a hérité)
  //   role-delegate-non-transitive-banner (présent si user a hérité OU
  //                                         fallback conservateur)

  import { _ } from "../../i18n";
  import { toast } from "../../../stores/toast";
  import {
    revokeDelegation,
    listDelegationsOf,
    type RoleDelegationResponse,
  } from "../../api/role_delegations";
  import ExpirationBadge from "../shared/ExpirationBadge.svelte";

  // -------------------------------------------------------------------------
  // Props
  // -------------------------------------------------------------------------

  let {
    /** Liste initiale (server-fetched ou injection test). Si non fournie, le
     *  composant fetche lui-même au mount via `listDelegationsOf()`. */
    initialDelegations = undefined,
    /** Callback émis quand l'utilisateur clique le CTA "Nouvelle délégation" —
     *  le parent ouvre le modal contenant `<RoleDelegationForm>`. */
    onNew = undefined,
    /** Map id → label pour résoudre user_id en nom lisible (fallback UUID
     *  tronqué si non fourni). */
    userLabels = {},
    /** Map id → label pour résoudre organization_id (fallback UUID tronqué). */
    orgLabels = {},
    /** UUID du current user (lu par le parent depuis l'authStore). Si
     *  `undefined` → fallback conservateur (banner toujours affiché). */
    currentUserId = undefined,
    /** Injection clock pour `ExpirationBadge` (tests déterministes). */
    nowOverride = undefined,
  }: {
    initialDelegations?: RoleDelegationResponse[];
    onNew?: () => void;
    userLabels?: Record<string, string>;
    orgLabels?: Record<string, string>;
    currentUserId?: string | undefined;
    nowOverride?: Date | undefined;
  } = $props();

  // -------------------------------------------------------------------------
  // State local
  // -------------------------------------------------------------------------

  let delegations = $state<RoleDelegationResponse[]>([]);
  let loading = $state<boolean>(false);
  /** ID de la délégation en cours de confirmation revoke (null = pas de modal). */
  let pendingRevokeId = $state<string | null>(null);

  // -------------------------------------------------------------------------
  // Fetch initial si pas de liste injectée
  // -------------------------------------------------------------------------

  async function fetchDelegations(): Promise<void> {
    loading = true;
    try {
      delegations = await listDelegationsOf(currentUserId);
    } catch {
      // toast déjà émis par api.ts pour 4xx/5xx
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (initialDelegations !== undefined) {
      delegations = initialDelegations;
    } else {
      void fetchDelegations();
    }
  });

  // -------------------------------------------------------------------------
  // Détection non-transitivité (INV-8)
  //
  // Si le current user a HÉRITÉ son rôle (il est `user_id` d'une row avec
  // `delegated_from_user_id` set), il NE PEUT PAS re-déléguer.
  //
  // Fallback conservateur : si `currentUserId === undefined`, on affiche le
  // banner avec wording adapté mais on garde le CTA (backend décidera).
  // -------------------------------------------------------------------------

  let hasInheritedRole = $derived.by(() => {
    if (currentUserId === undefined) return false; // fallback : pas de masquage
    return delegations.some(
      (d) =>
        d.user_id === currentUserId &&
        d.delegated_from_user_id !== null &&
        d.delegated_from_user_id !== undefined,
    );
  });

  /** Banner toujours affiché sur cette page (avec wording adapté si le user
   *  a hérité, ou avec un wording informatif sinon — pédagogie INV-8). */
  let showBanner = $derived(true);

  /** Le CTA est ABSENT du DOM uniquement si on a la CERTITUDE que l'user a
   *  hérité (hasInheritedRole === true). Si fallback (currentUserId undefined),
   *  le CTA reste — le backend renverra 403 si tentative invalide. */
  let showCta = $derived(!hasInheritedRole);

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
      await revokeDelegation(id);
      toast.success(
        $_("roleDelegation.revoke.success") || "Délégation révoquée.",
      );
      // Optimistic update : retire la row côté client (le backend renvoie
      // 204, pas de re-fetch).
      delegations = delegations.filter((d) => d.id !== id);
    } catch {
      // toast déjà émis par api.ts
    }
  }

  // -------------------------------------------------------------------------
  // Dérivations affichage
  // -------------------------------------------------------------------------

  function labelUser(userId: string): string {
    return userLabels[userId] ?? userId.slice(0, 8);
  }

  function labelOrg(orgId: string | null | undefined): string {
    if (!orgId) return $_("common.global") || "Global";
    return orgLabels[orgId] ?? orgId.slice(0, 8);
  }
</script>

<section
  class="role-delegation-list-section flex flex-col gap-4"
  aria-labelledby="role-delegation-list-title"
>
  <!-- Banner non-transitivité INV-8 — toujours affiché sur cette page,
       wording adapté selon hasInheritedRole. role="alert" aria-live="polite"
       (a11y AC §B4). -->
  {#if showBanner}
    <div
      data-testid="role-delegate-non-transitive-banner"
      role="alert"
      aria-live="polite"
      class={`rounded border px-4 py-3 text-sm ${
        hasInheritedRole
          ? "bg-red-50 border-red-300 text-red-800"
          : "bg-yellow-50 border-yellow-300 text-yellow-900"
      }`}
    >
      {#if hasInheritedRole}
        <strong>
          {$_("roleDelegation.banner.inherited.title") ||
            "Vous avez reçu ce rôle par délégation."}
        </strong>
        <span class="block">
          {$_("roleDelegation.banner.inherited.body") ||
            "Vous ne pouvez pas re-déléguer (non-transitivité INV-8)."}
        </span>
      {:else}
        <strong>
          {$_("roleDelegation.banner.info.title") ||
            "Délégation non transitive (INV-8)."}
        </strong>
        <span class="block">
          {$_("roleDelegation.banner.info.body") ||
            "Si vous avez reçu votre rôle par délégation, la re-délégation est interdite."}
        </span>
      {/if}
    </div>
  {/if}

  <header class="flex items-center justify-between">
    <h2
      id="role-delegation-list-title"
      class="text-lg font-semibold text-gray-900"
    >
      {$_("roleDelegation.list.title") || "Délégations actives"}
    </h2>
    {#if showCta}
      <button
        type="button"
        data-testid="role-delegate-new-button"
        class="min-h-[44px] px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
        onclick={() => onNew?.()}
      >
        {$_("roleDelegation.list.new") || "Nouvelle délégation"}
      </button>
    {/if}
  </header>

  {#if loading && delegations.length === 0}
    <p class="text-sm text-gray-500" role="status" aria-live="polite">
      {$_("common.loading") || "Chargement…"}
    </p>
  {:else if delegations.length === 0}
    <p class="text-sm text-gray-500" data-testid="role-delegation-list-empty">
      {$_("roleDelegation.list.empty") ||
        "Aucune délégation active pour le moment."}
    </p>
  {:else}
    <div class="overflow-x-auto">
      <table
        data-testid="role-delegation-list"
        class="min-w-full divide-y divide-gray-200 text-sm"
      >
        <thead class="bg-gray-50">
          <tr>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("roleDelegation.col.target") || "Cible"}
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("roleDelegation.col.role") || "Rôle"}
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("roleDelegation.col.organization") || "Organisation"}
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("roleDelegation.col.source") || "Source"}
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              {$_("roleDelegation.col.expire") || "Expire"}
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-right font-medium text-gray-700"
            >
              {$_("roleDelegation.col.actions") || "Actions"}
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-100">
          {#each delegations as d (d.id)}
            <tr data-testid={`role-delegation-row-${d.id}`}>
              <td
                class="px-3 py-2 text-gray-900"
                data-testid={`role-delegation-row-target-${d.id}`}
              >
                {labelUser(d.user_id)}
              </td>
              <td
                class="px-3 py-2 text-gray-700"
                data-testid={`role-delegation-row-role-${d.id}`}
              >
                {d.role}
              </td>
              <td
                class="px-3 py-2 text-gray-700"
                data-testid={`role-delegation-row-org-${d.id}`}
              >
                {labelOrg(d.organization_id)}
              </td>
              <td
                class="px-3 py-2 text-gray-600"
                data-testid={`role-delegation-row-source-${d.id}`}
              >
                {d.delegated_from_user_id
                  ? labelUser(d.delegated_from_user_id)
                  : ($_("common.dash") || "—")}
              </td>
              <td class="px-3 py-2">
                {#if d.valid_until}
                  <span
                    data-testid={`role-delegation-expiration-badge-${d.id}`}
                  >
                    <ExpirationBadge
                      validUntil={d.valid_until}
                      idSuffix={`role-delegation-${d.id}`}
                      {nowOverride}
                    />
                  </span>
                {:else}
                  <span class="text-xs text-gray-500">
                    {$_("common.permanent") || "Permanent"}
                  </span>
                {/if}
              </td>
              <td class="px-3 py-2 text-right">
                <button
                  type="button"
                  data-testid={`role-delegation-revoke-${d.id}`}
                  class="text-xs text-red-600 hover:underline"
                  onclick={() => askRevoke(d.id)}
                  aria-label={`Révoquer la délégation ${d.id}`}
                >
                  {$_("roleDelegation.action.revoke") || "Révoquer"}
                </button>
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
      aria-labelledby="role-delegation-revoke-confirm-title"
    >
      <div class="bg-white rounded shadow-lg p-6 max-w-sm w-full">
        <h3
          id="role-delegation-revoke-confirm-title"
          class="text-base font-semibold mb-2"
        >
          {$_("roleDelegation.revoke.confirm.title") ||
            "Confirmer la révocation"}
        </h3>
        <p class="text-sm text-gray-700 mb-4">
          {$_("roleDelegation.revoke.confirm.body") ||
            "Le user perdra immédiatement le rôle délégué."}
        </p>
        <div class="flex justify-end gap-2">
          <button
            type="button"
            class="min-h-[44px] px-3 py-1 text-sm border border-gray-300 rounded text-gray-700"
            onclick={cancelRevoke}
          >
            {$_("common.cancel") || "Annuler"}
          </button>
          <button
            type="button"
            data-testid="role-delegation-revoke-confirm"
            class="min-h-[44px] px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700"
            onclick={confirmRevoke}
          >
            {$_("roleDelegation.action.revoke") || "Révoquer"}
          </button>
        </div>
      </div>
    </div>
  {/if}
</section>
