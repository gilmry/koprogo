<script lang="ts">
  // Story B4 (Phase B FE) — Wrapper Svelte qui coordonne :
  //   - `RoleDelegationList` (table + CTA "Nouvelle" + revoke + banner INV-8)
  //   - `RoleDelegationForm` (modal de création)
  //
  // Le wrapper centralise :
  //   - le fetch initial des targets (users délégables) + organizations
  //   - la lecture du `currentUserId` depuis l'authStore (passe à la List
  //     pour la détection non-transitivité INV-8)
  //   - le toggle du modal de création
  //   - le rafraîchissement de la liste après création OK
  //
  // Pourquoi un wrapper séparé : la page Astro `pages/syndic/role-delegations
  // .astro` ne peut pas porter de state Svelte directement — elle monte un
  // seul composant `client:load`. Ce wrapper joue ce rôle d'« entry component ».

  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { _ } from "../../i18n";
  import { authStore } from "../../../stores/auth";
  import {
    listDelegationsOf,
    type RoleDelegationResponse,
  } from "../../api/role_delegations";
  import { listOrganizationUsers } from "../../api/organizations";
  import { api } from "../../api";
  import RoleDelegationList from "./RoleDelegationList.svelte";
  import RoleDelegationForm from "./RoleDelegationForm.svelte";

  let delegations = $state<RoleDelegationResponse[]>([]);
  let targets = $state<Array<{ id: string; label: string }>>([]);
  let organizations = $state<Array<{ id: string; label: string }>>([]);
  let userLabels = $state<Record<string, string>>({});
  let orgLabels = $state<Record<string, string>>({});
  let showForm = $state<boolean>(false);
  let loading = $state<boolean>(true);
  let currentUserId = $state<string | undefined>(undefined);
  let currentOrganizationId = $state<string>("");

  type UserLike = {
    id: string;
    email: string;
    first_name?: string;
    last_name?: string;
    role: string;
  };
  type OrgLike = { id: string; name: string };

  async function loadInitial(): Promise<void> {
    loading = true;
    try {
      // Lit le current user (utilisé par RoleDelegationList pour la détection
      // INV-8 non-transitivité) et son organizationId (endpoint org-scopé —
      // cf. Story S3 docs/maury/syndic-org-users-endpoint ; `organization_id`
      // en fallback pour rester tolérant à la forme brute stockée par
      // authStore). Défensif : authStore peut être en cours de réhydratation
      // au mount (#550 silent-refresh).
      let organizationId = "";
      try {
        const state = get(authStore);
        currentUserId = state.user?.id ?? undefined;
        const authUser = state.user as
          | { organizationId?: string; organization_id?: string }
          | undefined;
        organizationId =
          authUser?.organizationId ?? authUser?.organization_id ?? "";
        currentOrganizationId = organizationId;
      } catch {
        currentUserId = undefined;
      }

      const [d, users, orgsResp] = await Promise.all([
        listDelegationsOf(currentUserId).catch(
          () => [] as RoleDelegationResponse[],
        ),
        organizationId
          ? listOrganizationUsers(organizationId).catch(
              () => ({ data: [] as UserLike[] }),
            )
          : Promise.resolve({ data: [] as UserLike[] }),
        api
          .get<{ data: OrgLike[] }>("/organizations?per_page=1000")
          .catch(() => ({ data: [] as OrgLike[] })),
      ]);

      delegations = d;

      const usrs = users.data ?? [];
      const newTargets: Array<{ id: string; label: string }> = [];
      const newUserLabels: Record<string, string> = {};
      for (const u of usrs) {
        const label =
          `${u.first_name ?? ""} ${u.last_name ?? ""}`.trim() || u.email;
        // On exclut le current user de la liste de cibles (ne peut pas se
        // déléguer à soi-même).
        if (u.id !== currentUserId) {
          newTargets.push({ id: u.id, label });
        }
        newUserLabels[u.id] = label;
      }
      targets = newTargets;
      userLabels = newUserLabels;

      const newOrgs: Array<{ id: string; label: string }> = [];
      const newOrgLabels: Record<string, string> = {};
      for (const o of orgsResp.data ?? []) {
        newOrgs.push({ id: o.id, label: o.name });
        newOrgLabels[o.id] = o.name;
      }
      organizations = newOrgs;
      orgLabels = newOrgLabels;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadInitial();
  });

  function handleCreated(d: RoleDelegationResponse): void {
    // Optimistic prepend — pas de re-fetch coûteux.
    delegations = [d, ...delegations];
    showForm = false;
  }
</script>

<div class="container mx-auto px-4 py-6 flex flex-col gap-6">
  {#if loading}
    <p class="text-sm text-gray-500" role="status" aria-live="polite">
      {$_("common.loading") || "Chargement…"}
    </p>
  {:else}
    <RoleDelegationList
      initialDelegations={delegations}
      {userLabels}
      {orgLabels}
      {currentUserId}
      onNew={() => (showForm = true)}
    />
  {/if}

  {#if showForm}
    <div
      class="fixed inset-0 z-40 flex items-center justify-center bg-black/30 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="role-delegate-modal-title"
    >
      <div
        class="bg-white rounded shadow-lg max-w-2xl w-full max-h-full overflow-y-auto"
      >
        <RoleDelegationForm
          {targets}
          {organizations}
          defaultOrganizationId={currentOrganizationId}
          onSuccess={handleCreated}
          onCancel={() => (showForm = false)}
        />
      </div>
    </div>
  {/if}
</div>
