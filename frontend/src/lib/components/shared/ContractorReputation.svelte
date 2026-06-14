<script lang="ts">
  // Story B8 (Phase B FE) — ContractorReputation.
  //
  // Page de reputation contractor publique au sein de l'org. Lecture seule —
  // affiche les moyennes des 5 critères + listing des évaluations (append-only
  // INV-24 BE → AUCUN bouton "Modifier" / "Supprimer" sur une évaluation).
  //
  // Source de vérité : `listForContractor(contractorUserId)` (backend renvoie
  // newest first).
  //
  // INV exposés en UI :
  //   - INV-24 BE (append-only) : pas de bouton Edit/Delete par évaluation.
  //     C'est le contrat — un syndic peut SEULEMENT ajouter une nouvelle
  //     évaluation, jamais réviser une ancienne.
  //
  // INV-FE9 (WCAG 2.1 AA) :
  //   - Chaque moyenne annoncée via `aria-label` complet
  //     (ex: "Moyenne Qualité technique : 4.2 sur 5").
  //   - Table évaluations a `<th scope="col">` + caption sr-only.
  //   - Empty state textuel (pas d'icône-only).
  //
  // data-testid (cf. stories.md §B8 Reputation) :
  //   contractor-reputation-name
  //   contractor-reputation-avg-{quality|timeliness|communication|cost|overall}
  //   contractor-reputation-count
  //   contractor-reputation-eval-row-{id}
  //   contractor-reputation-empty
  //
  // Pattern DI : `evaluations` + `contractorName` fournis par le parent
  // (page Astro) qui fait le fetch via `listForContractor`. Permet de tester
  // sans mocker un fetch.

  import {
    SCORE_DIMENSIONS,
    SCORE_DIMENSION_LABELS_FR,
    averageScore,
    formatAverage,
    type ContractorEvaluationDto,
    type ScoreDimension,
  } from "../../api/contractor_evaluations";

  let {
    /** Nom affiché du contractor (le backend ne renvoie qu'un user_id —
     *  le parent résoud le label via `/users/{id}` ou cache local). */
    contractorName,
    /** Liste des évaluations (newest first côté backend). */
    evaluations = [],
    /** Pour idSuffix data-testid quand on embed le widget dans une autre page. */
    idSuffix = undefined as string | undefined,
  }: {
    contractorName: string;
    evaluations?: readonly ContractorEvaluationDto[];
    idSuffix?: string | undefined;
  } = $props();

  /** Suffixe data-testid pour le `scores-{dim}` — stories.md §B8 utilise
   *  "cost" pour cost_compliance. */
  function scoreTestSuffix(dim: ScoreDimension): string {
    return dim === "cost_compliance" ? "cost" : dim;
  }

  function testId(base: string): string {
    return idSuffix !== undefined ? `${base}-${idSuffix}` : base;
  }

  /** Format date courte pour affichage (FR-Belgique). */
  function formatDate(iso: string): string {
    try {
      return new Date(iso).toISOString().slice(0, 10);
    } catch {
      return iso;
    }
  }
</script>

<section
  class="contractor-reputation flex flex-col gap-4"
  aria-labelledby="contractor-reputation-heading"
>
  <header class="flex flex-col gap-1">
    <h2
      id="contractor-reputation-heading"
      class="text-xl font-semibold text-gray-900"
    >
      Réputation —
      <span data-testid={testId("contractor-reputation-name")}>
        {contractorName}
      </span>
    </h2>
    <p class="text-sm text-gray-500">
      Évaluations cumulées :
      <span
        data-testid={testId("contractor-reputation-count")}
        class="font-semibold text-gray-700"
      >
        {evaluations.length}
      </span>
    </p>
  </header>

  <!-- Moyennes par dimension (5 critères) -->
  <div
    class="grid grid-cols-2 md:grid-cols-5 gap-2"
    role="group"
    aria-label="Moyennes par critère"
  >
    {#each SCORE_DIMENSIONS as dim (dim)}
      {@const avg = averageScore(evaluations, dim)}
      <div
        class="flex flex-col items-center justify-center rounded border border-gray-200 bg-gray-50 px-3 py-2"
      >
        <span class="text-xs font-medium text-gray-600">
          {SCORE_DIMENSION_LABELS_FR[dim]}
        </span>
        <span
          data-testid={testId(
            `contractor-reputation-avg-${scoreTestSuffix(dim)}`,
          )}
          class="text-lg font-semibold text-gray-900"
          aria-label={`Moyenne ${SCORE_DIMENSION_LABELS_FR[dim]} : ${
            avg !== null ? `${avg.toFixed(1)} sur 5` : "non disponible"
          }`}
        >
          {formatAverage(avg)}
        </span>
      </div>
    {/each}
  </div>

  <!-- Listing évaluations (append-only INV-24 — pas de bouton Edit/Delete) -->
  {#if evaluations.length === 0}
    <p
      data-testid={testId("contractor-reputation-empty")}
      class="text-sm text-gray-500"
      role="status"
    >
      Aucune évaluation pour ce contractor pour le moment.
    </p>
  {:else}
    <div class="overflow-x-auto">
      <table
        data-testid={testId("contractor-reputation-list")}
        class="min-w-full divide-y divide-gray-200 text-sm"
      >
        <caption class="sr-only">
          Liste des évaluations de {contractorName} (lecture seule —
          append-only).
        </caption>
        <thead class="bg-gray-50">
          <tr>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Date
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Qualité
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Délais
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Comm.
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Budget
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Globale
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Commentaire
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-100">
          {#each evaluations as ev (ev.id)}
            <tr
              data-testid={testId(`contractor-reputation-eval-row-${ev.id}`)}
            >
              <td class="px-3 py-2 text-gray-600">
                {formatDate(ev.created_at)}
              </td>
              <td class="px-3 py-2 text-gray-900 font-mono">
                {ev.scores.quality}
              </td>
              <td class="px-3 py-2 text-gray-900 font-mono">
                {ev.scores.timeliness}
              </td>
              <td class="px-3 py-2 text-gray-900 font-mono">
                {ev.scores.communication}
              </td>
              <td class="px-3 py-2 text-gray-900 font-mono">
                {ev.scores.cost_compliance}
              </td>
              <td class="px-3 py-2 text-gray-900 font-mono font-semibold">
                {ev.scores.overall}
              </td>
              <td class="px-3 py-2 text-gray-700 max-w-xs truncate">
                {ev.comment}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>
