<script lang="ts">
  // Story B3 (Phase B FE) — Wrapper Svelte qui coordonne :
  //   - `MandateList` (table + CTA "Nouveau" + revoke)
  //   - `MandateIssueForm` (modal d'émission)
  //
  // Le wrapper centralise :
  //   - le fetch initial des subjects (users mandatables) + scopes (buildings/ACPs)
  //   - le toggle du modal d'émission
  //   - le rafraîchissement de la liste après émission OK
  //
  // Pourquoi un wrapper séparé : la page Astro `pages/syndic/mandates.astro`
  // ne peut pas porter de state Svelte directement — elle monte un seul
  // composant `client:load`. Ce wrapper joue ce rôle d'« entry component ».

  import { onMount } from "svelte";
  import { _ } from "../../i18n";
  import { listMandates, type MandateResponse } from "../../api/mandates";
  import { listBuildings } from "../../api/buildings";
  import { listAcps } from "../../api/acps";
  import { api } from "../../api";
  import MandateList from "./MandateList.svelte";
  import MandateIssueForm from "./MandateIssueForm.svelte";

  let mandates = $state<MandateResponse[]>([]);
  let subjects = $state<Array<{ id: string; label: string }>>([]);
  let scopes = $state<
    Array<{ id: string; kind: "building" | "acp"; label: string }>
  >([]);
  let subjectLabels = $state<Record<string, string>>({});
  let scopeLabels = $state<Record<string, string>>({});
  let showForm = $state<boolean>(false);
  let loading = $state<boolean>(true);

  type UserLike = {
    id: string;
    email: string;
    first_name?: string;
    last_name?: string;
    role: string;
  };

  async function loadInitial(): Promise<void> {
    loading = true;
    try {
      const [m, b, a, users] = await Promise.all([
        listMandates().catch(() => [] as MandateResponse[]),
        listBuildings(1, 100).catch(() => ({ data: [], pagination: {} as never })),
        listAcps().catch(() => []),
        api
          .get<{ data: UserLike[] }>("/users")
          .catch(() => ({ data: [] as UserLike[] })),
      ]);

      mandates = m;

      const usrs = users.data ?? [];
      // Mandataires éligibles (Story 3.4 — sous-ensemble de rôles).
      const ELIGIBLE_ROLES = new Set([
        "lawyer",
        "notary",
        "amo",
        "architect",
        "bet",
        "warden",
      ]);
      const newSubjects: Array<{ id: string; label: string }> = [];
      const newSubjectLabels: Record<string, string> = {};
      for (const u of usrs) {
        if (!ELIGIBLE_ROLES.has(u.role)) continue;
        const label = `${u.first_name ?? ""} ${u.last_name ?? ""}`.trim() || u.email;
        newSubjects.push({ id: u.id, label });
        newSubjectLabels[u.id] = label;
      }
      subjects = newSubjects;
      subjectLabels = newSubjectLabels;

      const newScopes: Array<{
        id: string;
        kind: "building" | "acp";
        label: string;
      }> = [];
      const newScopeLabels: Record<string, string> = {};
      for (const bld of b.data ?? []) {
        newScopes.push({ id: bld.id, kind: "building", label: bld.name });
        newScopeLabels[`building:${bld.id}`] = bld.name;
      }
      for (const acp of a ?? []) {
        newScopes.push({ id: acp.id, kind: "acp", label: acp.name });
        newScopeLabels[`acp:${acp.id}`] = acp.name;
      }
      scopes = newScopes;
      scopeLabels = newScopeLabels;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadInitial();
  });

  function handleIssued(m: MandateResponse): void {
    // Optimistic prepend — pas de re-fetch coûteux.
    mandates = [m, ...mandates];
    showForm = false;
  }
</script>

<div class="container mx-auto px-4 py-6 flex flex-col gap-6">
  {#if loading}
    <p class="text-sm text-gray-500" role="status" aria-live="polite">
      {$_("common.loading") || "Chargement…"}
    </p>
  {:else}
    <MandateList
      initialMandates={mandates}
      {subjectLabels}
      {scopeLabels}
      onNew={() => (showForm = true)}
    />
  {/if}

  {#if showForm}
    <div
      class="fixed inset-0 z-40 flex items-center justify-center bg-black/30 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="mandate-issue-modal-title"
    >
      <div class="bg-white rounded shadow-lg max-w-2xl w-full max-h-full overflow-y-auto">
        <MandateIssueForm
          {subjects}
          {scopes}
          onSuccess={handleIssued}
          onCancel={() => (showForm = false)}
        />
      </div>
    </div>
  {/if}
</div>
