<script lang="ts">
  // Story B5 (Phase B FE) — Wrapper page : bridge URL params + auth + API
  // → mount du TicketCreate avec props prêtes.
  //
  // Pourquoi un wrapper séparé de TicketCreate.svelte ?
  //   - TicketCreate est un composant pur (DI sur onCreate, candidates,
  //     currentUserId) → testable Vitest sans authStore.
  //   - TicketCreatePageWrapper fait l'orchestration runtime : lit
  //     buildingId dans l'URL, charge witnesses du building, lit current user
  //     depuis le authStore, redirige sur succès.
  //
  // Pattern aligné avec MandatesPage.svelte (Story B3).

  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { authStore } from "../../../stores/auth";
  import { api } from "../../api";
  import { toast } from "../../../stores/toast";
  import TicketCreate from "./TicketCreate.svelte";
  import type { WitnessCandidate } from "./WitnessSelector.svelte";
  import type { Ticket } from "../../api/tickets";

  let buildingId = $state<string>("");
  let unitId = $state<string | undefined>(undefined);
  let currentUserId = $state<string>("");
  let witnessCandidates = $state<WitnessCandidate[]>([]);
  let loading = $state<boolean>(true);
  let initError = $state<string>("");

  type OwnerLike = {
    id: string;
    email: string;
    first_name?: string;
    last_name?: string;
    unit_number?: string;
  };

  onMount(() => {
    void initialize();
  });

  async function initialize(): Promise<void> {
    loading = true;
    try {
      // 1. URL params (Astro static).
      const params = new URLSearchParams(window.location.search);
      buildingId = params.get("buildingId") ?? "";
      const u = params.get("unitId");
      unitId = u && u.length > 0 ? u : undefined;

      // 2. Current user via authStore.
      try {
        const auth = get(authStore);
        currentUserId = auth.user?.id ?? "";
      } catch {
        currentUserId = "";
      }

      // 3. Witness candidates : owners du building.
      //    L'endpoint backend liste les owners d'un building (pattern existant).
      if (buildingId) {
        try {
          const owners = await api.get<{ data: OwnerLike[] } | OwnerLike[]>(
            `/buildings/${buildingId}/owners`,
          );
          const list = Array.isArray(owners)
            ? owners
            : (owners.data ?? []);
          witnessCandidates = list
            .filter((o) => o.id !== currentUserId)
            .map((o) => ({
              id: o.id,
              label:
                `${o.first_name ?? ""} ${o.last_name ?? ""}`.trim() +
                (o.unit_number ? ` — Lot ${o.unit_number}` : "") ||
                o.email,
            }));
        } catch {
          // Pas bloquant : le form se rend, juste sans suggestion de témoins.
          witnessCandidates = [];
        }
      }

      if (!buildingId) {
        initError =
          "Aucun immeuble spécifié — ajoutez `?buildingId=…` à l'URL.";
      }
    } finally {
      loading = false;
    }
  }

  function handleCreated(t: Ticket): void {
    toast.success("Ticket créé.");
    // Redirection vers le détail du ticket.
    window.location.href = `/ticket-detail?id=${t.id}`;
  }

  function handleCancel(): void {
    window.history.length > 1
      ? window.history.back()
      : (window.location.href = "/tickets");
  }
</script>

{#if loading}
  <p class="text-sm text-gray-500" role="status" aria-live="polite">
    Chargement…
  </p>
{:else if initError}
  <p
    class="rounded border border-red-300 bg-red-50 p-3 text-sm text-red-700"
    role="alert"
  >
    {initError}
  </p>
{:else}
  <TicketCreate
    {buildingId}
    {unitId}
    {currentUserId}
    {witnessCandidates}
    onCreated={handleCreated}
    onCancel={handleCancel}
  />
{/if}
