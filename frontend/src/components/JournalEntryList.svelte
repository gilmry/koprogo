<script lang="ts">
  // Svelte 5 runes mode
  //
  // Vue liste du grand livre. La page /journal-entries n'offrait QUE le
  // formulaire de saisie : une fois l'ecriture creee, plus rien ne permettait
  // de la retrouver depuis l'interface (constat F6 du rapport du 2026-09-01).
  // L'API `GET /journal-entries` existait pourtant deja, avec ses filtres et
  // sa pagination — c'est l'ecran qui manquait, pas le service.

  import { _ } from "../lib/i18n";
  import { api } from "../lib/api";
  import { formatDate } from "../lib/utils/date.utils";
  import { formatCurrency } from "../lib/utils/finance.utils";
  import { toNumber } from "../lib/utils/decimal.utils";
  import { SvelteMap } from "svelte/reactivity";

  let { buildingId = null, reloadToken = 0 }: {
    buildingId?: string | null;
    /// Increment par le parent apres une creation pour rafraichir la liste.
    reloadToken?: number;
  } = $props();

  interface JournalEntry {
    id: string;
    building_id: string | null;
    journal_type: string | null;
    entry_date: string;
    description: string | null;
    document_ref: string | null;
    expense_id: string | null;
    contribution_id: string | null;
  }

  interface JournalEntryLine {
    id: string;
    account_code: string;
    // `Decimal` cote Rust, donc CHAINE en JSON (ADR-0008) : passer par
    // `toNumber` avant toute addition, sinon `+` concatene.
    debit: string | number;
    credit: string | number;
    description: string | null;
  }

  let entries = $state<JournalEntry[]>([]);
  let loading = $state(true);
  let error = $state("");

  let filterJournalType = $state("");
  let filterStartDate = $state("");
  let filterEndDate = $state("");
  let page = $state(1);
  const perPage = 20;

  // `SvelteMap` et non `Map` : `$state` rend reactifs les objets et les
  // tableaux, jamais les collections natives — muter une `Map` ne
  // redeclencherait aucun rendu.
  let expanded = new SvelteMap<string, JournalEntryLine[] | "loading" | "error">();

  const journalTypes = $derived([
    { code: "ACH", label: $_("journal.types.ach") },
    { code: "VEN", label: $_("journal.types.ven") },
    { code: "FIN", label: $_("journal.types.fin") },
    { code: "ODS", label: $_("journal.types.ods") },
  ]);

  function journalLabel(code: string | null): string {
    return journalTypes.find((t) => t.code === code)?.label ?? (code || "—");
  }

  async function loadEntries() {
    loading = true;
    error = "";
    try {
      const params = new URLSearchParams({
        page: String(page),
        per_page: String(perPage),
      });
      if (buildingId) params.set("building_id", buildingId);
      if (filterJournalType) params.set("journal_type", filterJournalType);
      // L'API attend du RFC3339, pas la date nue du champ `<input type=date>`.
      if (filterStartDate) params.set("start_date", `${filterStartDate}T00:00:00Z`);
      if (filterEndDate) params.set("end_date", `${filterEndDate}T23:59:59Z`);

      const response = await api.get<{ data: JournalEntry[] }>(
        `/journal-entries?${params.toString()}`,
      );
      entries = response.data ?? [];
      expanded.clear();
    } catch (e) {
      error = e instanceof Error ? e.message : $_("journal.loadError");
      entries = [];
    } finally {
      loading = false;
    }
  }

  // Se declenche au montage, a chaque changement de filtre ou de page, et
  // quand le parent signale une creation.
  $effect(() => {
    void buildingId;
    void filterJournalType;
    void filterStartDate;
    void filterEndDate;
    void page;
    void reloadToken;
    loadEntries();
  });

  async function toggleLines(entryId: string) {
    if (expanded.has(entryId)) {
      expanded.delete(entryId);
      return;
    }
    expanded.set(entryId, "loading");
    try {
      const detail = await api.get<{ lines: JournalEntryLine[] }>(
        `/journal-entries/${entryId}`,
      );
      expanded.set(entryId, detail.lines ?? []);
    } catch {
      expanded.set(entryId, "error");
    }
  }

  function entryTotal(lines: JournalEntryLine[]): number {
    return lines.reduce((sum, l) => sum + toNumber(l.debit), 0);
  }

  function applyFilters() {
    // Tout changement de filtre ramene en page 1 : rester en page 3 d'un
    // resultat qui n'en compte plus qu'une afficherait une liste vide.
    page = 1;
  }
</script>

<div class="bg-white rounded-lg shadow" data-testid="journal-entry-list">
  <div class="border-b border-gray-200 px-6 py-4">
    <h2 class="text-lg font-semibold text-gray-900">{$_("journal.listTitle")}</h2>
    <p class="text-sm text-gray-500">{$_("journal.listSubtitle")}</p>
  </div>

  <div class="border-b border-gray-200 px-6 py-4 grid grid-cols-1 gap-4 sm:grid-cols-4">
    <div>
      <label for="journal-filter-type" class="block text-sm font-medium text-gray-700 mb-1">
        {$_("journal.journalType")}
      </label>
      <select
        id="journal-filter-type"
        bind:value={filterJournalType}
        onchange={applyFilters}
        data-testid="journal-type-filter"
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-primary-500"
      >
        <option value="">{$_("journal.allJournals")}</option>
        {#each journalTypes as t (t.code)}
          <option value={t.code}>{t.code} — {t.label}</option>
        {/each}
      </select>
    </div>
    <div>
      <label for="journal-filter-start" class="block text-sm font-medium text-gray-700 mb-1">
        {$_("journal.from")}
      </label>
      <input
        id="journal-filter-start"
        type="date"
        bind:value={filterStartDate}
        onchange={applyFilters}
        data-testid="journal-start-filter"
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-primary-500"
      />
    </div>
    <div>
      <label for="journal-filter-end" class="block text-sm font-medium text-gray-700 mb-1">
        {$_("journal.to")}
      </label>
      <input
        id="journal-filter-end"
        type="date"
        bind:value={filterEndDate}
        onchange={applyFilters}
        data-testid="journal-end-filter"
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-primary-500"
      />
    </div>
    <div class="flex items-end">
      <button
        type="button"
        onclick={loadEntries}
        data-testid="journal-refresh-button"
        class="w-full px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 font-medium"
      >
        {$_("journal.refresh")}
      </button>
    </div>
  </div>

  <div class="p-6">
    {#if loading}
      <p class="text-gray-500">{$_("common.loading")}</p>
    {:else if error}
      <p class="text-red-600" data-testid="journal-list-error">{error}</p>
    {:else if entries.length === 0}
      <p class="text-gray-500" data-testid="journal-list-empty">{$_("journal.noEntries")}</p>
    {:else}
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_("journal.operationDate")}</th>
              <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_("journal.journalType")}</th>
              <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_("journal.description")}</th>
              <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_("journal.documentRef")}</th>
              <th scope="col" class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">{$_("journal.accountingLines")}</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-200">
            {#each entries as entry (entry.id)}
              <tr data-testid="journal-entry-row">
                <td class="px-4 py-3 whitespace-nowrap text-sm text-gray-900">{formatDate(entry.entry_date)}</td>
                <td class="px-4 py-3 whitespace-nowrap text-sm">
                  <span class="inline-flex items-center rounded-full bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-800">
                    {entry.journal_type || "—"}
                  </span>
                  <span class="ml-2 text-gray-500">{journalLabel(entry.journal_type)}</span>
                </td>
                <td class="px-4 py-3 text-sm text-gray-900">{entry.description || "—"}</td>
                <td class="px-4 py-3 text-sm text-gray-500">{entry.document_ref || $_("journal.noDocumentRef")}</td>
                <td class="px-4 py-3 text-right text-sm">
                  <button
                    type="button"
                    onclick={() => toggleLines(entry.id)}
                    data-testid="toggle-lines-button"
                    class="text-primary-600 hover:text-primary-800 font-medium"
                  >
                    {expanded.has(entry.id) ? $_("journal.hideLines") : $_("journal.showLines")}
                  </button>
                </td>
              </tr>
              {#if expanded.has(entry.id)}
                {@const lines = expanded.get(entry.id)}
                <tr>
                  <td colspan="5" class="bg-gray-50 px-4 py-3" data-testid="journal-entry-lines">
                    {#if lines === "loading"}
                      <p class="text-sm text-gray-500">{$_("common.loading")}</p>
                    {:else if lines === "error"}
                      <p class="text-sm text-red-600">{$_("journal.linesLoadError")}</p>
                    {:else if lines}
                      <table class="min-w-full text-sm">
                        <thead>
                          <tr class="text-left text-xs uppercase text-gray-500">
                            <th scope="col" class="py-1 pr-4">{$_("journal.account")}</th>
                            <th scope="col" class="py-1 pr-4">{$_("journal.description")}</th>
                            <th scope="col" class="py-1 pr-4 text-right">{$_("journal.debit")}</th>
                            <th scope="col" class="py-1 text-right">{$_("journal.credit")}</th>
                          </tr>
                        </thead>
                        <tbody>
                          {#each lines as line (line.id)}
                            <tr data-testid="journal-line-row">
                              <td class="py-1 pr-4 font-mono text-gray-900">{line.account_code}</td>
                              <td class="py-1 pr-4 text-gray-600">{line.description || "—"}</td>
                              <td class="py-1 pr-4 text-right tabular-nums text-gray-900">
                                {toNumber(line.debit) > 0 ? formatCurrency(toNumber(line.debit)) : "—"}
                              </td>
                              <td class="py-1 text-right tabular-nums text-gray-900">
                                {toNumber(line.credit) > 0 ? formatCurrency(toNumber(line.credit)) : "—"}
                              </td>
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                      <p class="mt-2 text-xs text-gray-500" data-testid="journal-entry-total">
                        {$_("journal.entryTotal")}: {formatCurrency(entryTotal(lines))}
                      </p>
                    {/if}
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      </div>

      <div class="mt-4 flex items-center justify-between">
        <button
          type="button"
          disabled={page <= 1}
          onclick={() => (page = Math.max(1, page - 1))}
          data-testid="journal-prev-page"
          class="px-3 py-1.5 text-sm rounded-md border border-gray-300 disabled:opacity-40 disabled:cursor-not-allowed"
        >
          {$_("journal.previous")}
        </button>
        <span class="text-sm text-gray-500">{$_("journal.page", { values: { page: String(page) } })}</span>
        <button
          type="button"
          disabled={entries.length < perPage}
          onclick={() => (page = page + 1)}
          data-testid="journal-next-page"
          class="px-3 py-1.5 text-sm rounded-md border border-gray-300 disabled:opacity-40 disabled:cursor-not-allowed"
        >
          {$_("journal.next")}
        </button>
      </div>
    {/if}
  </div>
</div>
