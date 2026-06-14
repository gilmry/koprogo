<script lang="ts">
  // Story B5 (Phase B FE) — WitnessSelector (chip-input + autocomplete).
  //
  // Réutilisé par :
  //   - TicketCreate.svelte (section conditionnelle kind=Complaint).
  //
  // Contrat métier (cf. stories.md §B5 + INV-FE3 numbers) :
  //   - Max 10 témoins.
  //   - Owners du building UNIQUEMENT (filter côté parent via `candidates`).
  //   - witness != self (AC @security §B5 — bouton add disabled sur la row
  //     du current user + helper text).
  //
  // a11y (WCAG 2.1 AA — checklist §B5) :
  //   - Chip-input pattern avec `aria-live="polite"` pour annoncer
  //     ajout/retrait au lecteur d'écran.
  //   - Input de recherche : `aria-autocomplete="list"` + role=combobox.
  //   - Options dans listbox avec role=option + aria-selected.
  //
  // data-testid (cf. stories.md §B5 + mission) :
  //   ticket-witness-search          (input texte de recherche)
  //   ticket-witness-option-{userId} (option dropdown)
  //   ticket-witness-chip-{userId}   (chip sélectionné)
  //   ticket-witness-remove-{userId} (× sur chip)
  //   ticket-witness-count           (compteur "2/10")
  //   ticket-witness-self-warning    (helper text si self-ajout tenté)

  export interface WitnessCandidate {
    id: string;
    /** Nom affiché (e.g. "Marc Dubois — Lot A-3"). */
    label: string;
  }

  let {
    /** UserIds sélectionnés — propagé au parent. */
    value = $bindable<string[]>([]),
    /** Liste des candidats (owners of building) — fournie par le parent. */
    candidates = [] as WitnessCandidate[],
    /** UserId du current user — bloque self-witness. */
    currentUserId = "" as string,
    /** Max témoins (10 par défaut, INV-FE3). */
    max = 10,
  }: {
    value?: string[];
    candidates?: WitnessCandidate[];
    currentUserId?: string;
    max?: number;
  } = $props();

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  let search = $state<string>("");

  // ---------------------------------------------------------------------------
  // Derivations
  // ---------------------------------------------------------------------------

  let count = $derived(value.length);
  let countLabel = $derived(`${count}/${max}`);
  let atMax = $derived(count >= max);

  /** Suggestions filtrées : on retire les déjà sélectionnés, on filtre par
   *  search prefix (case-insensitive), on cap à 10 résultats. */
  let suggestions = $derived(
    candidates
      .filter((c) => !value.includes(c.id))
      .filter((c) =>
        search === ""
          ? true
          : c.label.toLowerCase().includes(search.toLowerCase()),
      )
      .slice(0, 10),
  );

  /** Labels des chips déjà ajoutés (lookup id → label). */
  let chips = $derived(
    value
      .map((id) => candidates.find((c) => c.id === id))
      .filter((c): c is WitnessCandidate => c !== undefined),
  );

  /** Une suggestion correspond au current user → on désactive l'add (§B5
   *  @security) et on affiche un helper. */
  function isSelf(id: string): boolean {
    return id !== "" && id === currentUserId;
  }

  /** Au moins une suggestion = self → on affiche le warning helper. */
  let selfInSuggestions = $derived(
    suggestions.some((s) => isSelf(s.id)),
  );

  // ---------------------------------------------------------------------------
  // Handlers
  // ---------------------------------------------------------------------------

  function addWitness(id: string): void {
    if (atMax) return;
    if (isSelf(id)) return; // INV §B5 @security : self impossible
    if (value.includes(id)) return; // double-add idempotent
    value = [...value, id];
    search = ""; // reset recherche pour UX
  }

  function removeWitness(id: string): void {
    value = value.filter((v) => v !== id);
  }
</script>

<section class="ticket-witness-selector flex flex-col gap-2">
  <div class="flex items-center justify-between">
    <label
      for="ticket-witness-search"
      class="block text-sm font-medium text-gray-700"
    >
      Témoins
    </label>
    <span
      data-testid="ticket-witness-count"
      class="text-xs text-gray-500"
      aria-live="polite"
    >
      {countLabel}
    </span>
  </div>

  <!-- Search input (autocomplete combobox) -->
  <input
    id="ticket-witness-search"
    data-testid="ticket-witness-search"
    type="text"
    bind:value={search}
    placeholder="Rechercher un copropriétaire…"
    autocomplete="off"
    role="combobox"
    aria-autocomplete="list"
    aria-expanded={suggestions.length > 0}
    aria-controls="ticket-witness-listbox"
    aria-disabled={atMax}
    disabled={atMax}
    class="w-full rounded border border-gray-300 px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2 disabled:bg-gray-100"
  />

  <!-- Self warning (helper text quand l'utilisateur apparaît dans les
       suggestions — on rend explicite la règle métier "pas soi-même"). -->
  {#if selfInSuggestions}
    <p
      data-testid="ticket-witness-self-warning"
      class="text-xs text-orange-700"
      role="note"
    >
      Vous ne pouvez pas vous lister comme témoin de votre propre plainte.
    </p>
  {/if}

  <!-- Listbox des suggestions -->
  {#if suggestions.length > 0 && !atMax}
    <ul
      id="ticket-witness-listbox"
      role="listbox"
      class="max-h-48 overflow-y-auto rounded border border-gray-200 bg-white"
    >
      {#each suggestions as s (s.id)}
        {@const disabledOpt = isSelf(s.id)}
        <li role="option" aria-selected="false" class="border-b last:border-b-0">
          <button
            type="button"
            data-testid={`ticket-witness-option-${s.id}`}
            onclick={() => addWitness(s.id)}
            disabled={disabledOpt}
            aria-disabled={disabledOpt}
            class="block w-full px-3 py-2 text-left text-sm hover:bg-blue-50 focus-visible:bg-blue-50 disabled:cursor-not-allowed disabled:text-gray-400"
          >
            {s.label}{disabledOpt ? " (vous-même)" : ""}
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <!-- Chips des témoins ajoutés (aria-live pour annonce a11y) -->
  {#if chips.length > 0}
    <ul
      class="flex flex-wrap gap-2"
      aria-live="polite"
      aria-label={`${chips.length} témoin${chips.length > 1 ? "s" : ""} sélectionné${chips.length > 1 ? "s" : ""}`}
    >
      {#each chips as c (c.id)}
        <li
          data-testid={`ticket-witness-chip-${c.id}`}
          class="inline-flex items-center gap-1 rounded-full border border-blue-300 bg-blue-50 px-3 py-1 text-xs text-blue-800"
        >
          <span>{c.label}</span>
          <button
            type="button"
            data-testid={`ticket-witness-remove-${c.id}`}
            onclick={() => removeWitness(c.id)}
            aria-label={`Retirer ${c.label} de la liste des témoins`}
            class="flex h-5 w-5 items-center justify-center rounded-full bg-white text-blue-800 hover:bg-red-100 hover:text-red-700 focus-visible:outline-2"
          >
            ×
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>
