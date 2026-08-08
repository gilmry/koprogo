<script lang="ts">
  // Story B7 (Phase B FE) — TechnicalSpecVersionTimeline.
  //
  // Ligne de versions d'une TechnicalSpec (chronologique inverse — version
  // courante en tête, Superseded grisées en dessous).
  //
  // Pattern read-only — pas de mutation ici. Le parent (TechnicalSpecDetail)
  // peut cliquer une ligne pour naviguer vers l'historique.
  //
  // INV-FE9 (a11y) :
  //   - role="list" sur la ligne, <li> par version.
  //   - aria-current="true" sur la version active (status != Superseded).
  //   - text-gray-* + opacity-60 sur Superseded (combiné couleur + texte pour
  //     daltoniens — INV-FE9).
  //
  // data-testid (cf. stories.md §B7) :
  //   tech-spec-timeline
  //   tech-spec-version-row-{version}  (ligne par version)

  import type { TechnicalSpecDto } from "../../api/technical_specs";

  // ---------------------------------------------------------------------------
  // Props
  // ---------------------------------------------------------------------------

  let {
    /** Toutes les versions historiques de la spec (ordonnées du backend ou
     *  côté parent). Le composant trie côté client par version semver (desc). */
    versions = [] as TechnicalSpecDto[],
    /** ID de la version active (celle affichée en détail à droite). */
    currentVersionId = undefined as string | undefined,
    /** Callback click sur une ligne — le parent peut router vers la version
     *  historique. Non fournie → ligne non cliquable. */
    onSelect = undefined as
      | undefined
      | ((spec: TechnicalSpecDto) => void),
  }: {
    versions?: TechnicalSpecDto[];
    currentVersionId?: string;
    onSelect?: (spec: TechnicalSpecDto) => void;
  } = $props();

  // ---------------------------------------------------------------------------
  // Dérivations — tri par created_at desc (cohérent avec semver desc en
  // pratique car les versions sont émises chronologiquement).
  // ---------------------------------------------------------------------------

  let sorted = $derived(
    [...versions].sort(
      (a, b) =>
        new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
    ),
  );

  function formatDate(iso: string): string {
    try {
      return new Intl.DateTimeFormat("fr-BE", {
        day: "numeric",
        month: "short",
        year: "numeric",
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  }

  function statusBadge(status: string): {
    label: string;
    classes: string;
  } {
    // Clés en snake_case — le backend sérialise `TechnicalSpecStatus` via
    // son `Display` ("draft"/"pending_signatures"/...), pas le nom d'enum
    // Rust PascalCase (trouvé en investiguant #617 C7, même bug que
    // TechnicalSpecDetail.svelte : ces clés PascalCase ne matchaient jamais).
    const map: Record<string, { label: string; classes: string }> = {
      draft: {
        label: "Brouillon",
        classes: "bg-gray-100 text-gray-700 border-gray-300",
      },
      pending_signatures: {
        label: "Attente signatures",
        classes: "bg-orange-100 text-orange-800 border-orange-300",
      },
      approved: {
        label: "Approuvée",
        classes: "bg-green-100 text-green-800 border-green-300",
      },
      superseded: {
        label: "Remplacée",
        classes: "bg-gray-200 text-gray-500 border-gray-300",
      },
    };
    return (
      map[status] ?? {
        label: status,
        classes: "bg-gray-100 text-gray-700 border-gray-300",
      }
    );
  }
</script>

<section
  class="tech-spec-version-timeline flex flex-col gap-2"
  aria-labelledby="tech-spec-timeline-title"
>
  <h3
    id="tech-spec-timeline-title"
    class="text-sm font-semibold text-gray-800"
  >
    Historique des versions
  </h3>

  {#if sorted.length === 0}
    <p class="text-xs text-gray-500" data-testid="tech-spec-timeline-empty">
      Aucune version pour cette spec.
    </p>
  {:else}
    <ol
      data-testid="tech-spec-timeline"
      role="list"
      class="flex flex-col gap-1 pl-0"
      aria-label="Versions de la fiche technique"
    >
      {#each sorted as spec (spec.id)}
        {@const badge = statusBadge(spec.status)}
        {@const isSuperseded = spec.status === "superseded"}
        {@const isCurrent = spec.id === currentVersionId}
        <li
          data-testid={`tech-spec-version-row-${spec.version}`}
          data-status={spec.status}
          data-current={isCurrent ? "true" : "false"}
          class={`flex items-center gap-3 rounded-md border px-3 py-2 text-sm ${
            isSuperseded
              ? "bg-gray-50 text-gray-500 opacity-70 border-gray-200"
              : "bg-white text-gray-900 border-gray-200"
          } ${isCurrent ? "ring-2 ring-blue-400" : ""}`}
          aria-current={isCurrent ? "true" : "false"}
        >
          <!-- Pastille marker (vide=ronde, plein=courante) -->
          <span
            aria-hidden="true"
            class={`inline-block w-2 h-2 rounded-full ${
              isCurrent
                ? "bg-blue-500"
                : isSuperseded
                  ? "bg-gray-300"
                  : "bg-green-500"
            }`}
          ></span>

          <!-- Version -->
          <span class="font-mono text-sm font-medium">
            v{spec.version}
          </span>

          <!-- Status badge -->
          <span
            class={`inline-flex items-center rounded border px-2 py-0.5 text-xs ${badge.classes}`}
          >
            {badge.label}
          </span>

          <!-- Date -->
          <span class="ml-auto text-xs text-gray-500">
            {formatDate(spec.created_at)}
          </span>

          <!-- Click handler -->
          {#if onSelect}
            <button
              type="button"
              class="min-h-[32px] text-xs text-blue-600 hover:underline"
              onclick={() => onSelect?.(spec)}
              aria-label={`Voir la version ${spec.version}`}
            >
              Voir
            </button>
          {/if}
        </li>
      {/each}
    </ol>
  {/if}
</section>
