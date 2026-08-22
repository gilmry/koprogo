<script lang="ts">
  // Story S2 (docs/maury/syndic-org-users-endpoint) — Wrapper Svelte qui
  // alimente `MagicLinkIssueForm` (composant pur, inchangé) avec :
  //   - `users` : contractors de l'org du syndic connecté (endpoint
  //     org-scopé — cf. Story S1) — pattern ELIGIBLE_ROLES de MandatesPage.
  //   - `scopeIdsByKind.ticket` : tickets de l'org, seul scope kind exercé
  //     par les tests existants (quote/invoice/contractor_evaluation
  //     restent volontairement non câblés, cf. Architecture §3.2).
  //   - `currentUserId` via authStore (pattern MandatesPage/ContractorEvaluationsPage).
  //
  // Pourquoi un wrapper séparé : la page Astro `pages/syndic/magic-links.astro`
  // ne peut pas porter de state Svelte directement — elle monte un seul
  // composant `client:load`. Avant cette story, `MagicLinkIssueForm` était
  // monté nu (sans users/scopes injectés) donc le sélecteur "Destinataire"
  // restait vide et la branche de création n'était jamais exercée (#617 C2).

  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { authStore } from "../../../stores/auth";
  import { listOrganizationUsers } from "../../api/organizations";
  import { ticketsApi } from "../../api/tickets";
  import MagicLinkIssueForm from "./MagicLinkIssueForm.svelte";

  type UserOption = { id: string; label: string };
  type ScopeIdOption = { id: string; label: string };

  let loading = $state<boolean>(true);
  let currentUserId = $state<string>("");
  let users = $state<UserOption[]>([]);
  let scopeIdsByKind = $state<{ ticket: ScopeIdOption[] }>({ ticket: [] });

  async function loadInitial(): Promise<void> {
    loading = true;
    try {
      const authUser = get(authStore).user as
        | { id?: string; organizationId?: string; organization_id?: string }
        | undefined;
      currentUserId = authUser?.id ?? "";
      const organizationId =
        authUser?.organizationId ?? authUser?.organization_id ?? "";

      const [usersResp, tickets] = await Promise.all([
        organizationId
          ? listOrganizationUsers(organizationId).catch(() => ({ data: [] }))
          : Promise.resolve({ data: [] }),
        organizationId
          ? ticketsApi.listByOrganization(organizationId).catch(() => [])
          : Promise.resolve([]),
      ]);

      // Destinataires éligibles = contractors (seul rôle exercé par les
      // tests existants — cf. Architecture §3.2).
      users = (usersResp.data ?? [])
        .filter((u) => u.role === "contractor")
        .map((u) => ({
          id: u.id,
          label:
            `${u.first_name ?? ""} ${u.last_name ?? ""}`.trim() || u.email,
        }));

      scopeIdsByKind = {
        ticket: (tickets ?? []).map((t) => ({
          id: t.id,
          label: t.title || `Ticket ${t.id.slice(0, 8)}`,
        })),
      };
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadInitial();
  });
</script>

{#if loading}
  <p class="text-sm text-gray-500" role="status" aria-live="polite">
    Chargement…
  </p>
{:else}
  <MagicLinkIssueForm {users} {scopeIdsByKind} {currentUserId} />
{/if}
