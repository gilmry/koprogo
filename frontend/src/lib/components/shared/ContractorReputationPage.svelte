<script lang="ts">
  // Story B8 (Phase B FE) — Wrapper page reputation contractor.
  //
  // Monté par `/contractor-reputation?contractorId=<uuid>`. Astro est en mode
  // static (cf. astro.config.mjs) → impossible d'utiliser un dynamic route
  // `/contractors/[id]/reputation` sans SSR adapter (cf. /c.astro note).
  // On lit le query param côté client puis fetch via API.
  //
  // Pattern cohérent avec `/c.astro` (B0 + magic-link consumption page).
  //
  // INV-24 BE (append-only) : aucun bouton Edit/Delete dans la table — c'est
  // ContractorReputation atomique qui le garantit.

  import { onMount } from "svelte";
  import { api } from "../../api";
  import {
    listForContractor,
    type ContractorEvaluationDto,
  } from "../../api/contractor_evaluations";
  import ContractorReputation from "./ContractorReputation.svelte";

  type UserLike = {
    id: string;
    email: string;
    first_name?: string;
    last_name?: string;
  };

  let loading = $state<boolean>(true);
  let error = $state<string | null>(null);
  let contractorId = $state<string>("");
  let contractorName = $state<string>("Contractor");
  let evaluations = $state<ContractorEvaluationDto[]>([]);

  function readContractorIdFromUrl(): string {
    if (typeof window === "undefined") return "";
    try {
      const url = new URL(window.location.href);
      return url.searchParams.get("contractorId") ?? "";
    } catch {
      return "";
    }
  }

  async function loadInitial(): Promise<void> {
    loading = true;
    error = null;
    const id = readContractorIdFromUrl();
    if (!id) {
      error = "Paramètre contractorId manquant dans l'URL.";
      loading = false;
      return;
    }
    contractorId = id;
    try {
      const [evs, user] = await Promise.all([
        listForContractor(id).catch(() => [] as ContractorEvaluationDto[]),
        api
          .get<UserLike>(`/users/${encodeURIComponent(id)}`, { silent: true })
          .catch(() => null as UserLike | null),
      ]);
      evaluations = evs;
      if (user) {
        contractorName =
          `${user.first_name ?? ""} ${user.last_name ?? ""}`.trim() ||
          user.email ||
          "Contractor";
      } else {
        // Fallback : nom court basé sur l'UUID si user non récupérable.
        contractorName = `Contractor ${id.slice(0, 8)}`;
      }
    } catch {
      error = "Erreur lors du chargement des évaluations.";
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadInitial();
  });
</script>

<div class="flex flex-col gap-4">
  {#if loading}
    <p class="text-sm text-gray-500" role="status" aria-live="polite">
      Chargement de la réputation…
    </p>
  {:else if error}
    <p
      data-testid="contractor-reputation-error"
      class="rounded border border-red-300 bg-red-50 px-3 py-2 text-sm text-red-800"
      role="alert"
    >
      {error}
    </p>
  {:else}
    <ContractorReputation {contractorName} {evaluations} />
  {/if}
</div>
