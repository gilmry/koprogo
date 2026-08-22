<script lang="ts">
  // Story B8 (Phase B FE) — composant atomique réutilisable.
  //
  // Radio group 1..=5 (notation type Likert) — réutilisé par :
  //   - B8 ContractorEvaluationForm — 5 instances (quality, timeliness,
  //     communication, cost_compliance, overall).
  //   - Futur : sondages internes (polls SEL), feedback events AG, etc.
  //
  // INV-FE9 (WCAG 2.1 AA + daltoniens) :
  //   - `<fieldset><legend>` (group sémantique radio).
  //   - 5 radios visibles (jamais slider seul → daltonisme + clavier).
  //   - aria-required quand `required={true}`.
  //   - Chaque radio a un label visible + couleur ≠ unique indicateur (texte).
  //   - data-testid composable : `score-input-{score}` (1..5) — exigé stories.md
  //     §B8 ScoreInput atomique.
  //
  // Bornes : impossible de saisir 0 ou 6 via UI (les radios n'existent pas).
  // C'est la défense AC @negative stories.md §B8 ("score = 0 ou 6 → impossible
  // via UI").
  //
  // Pattern DI : `value` + `onChange` callback (cohérent avec
  // SyndicResponseForm B6 et MandateIssueForm B3 — pas de `bind:value` à la
  // racine pour rester contrôlable depuis un parent en `$state`).

  import { SCORE_MIN, SCORE_MAX } from "../../api/contractor_evaluations";

  let {
    /** Identifiant unique du fieldset — sert au DOM + a11y. */
    name,
    /** Label visible (legend du fieldset). */
    label,
    /** Score courant (1..5) ou `null` si pas encore choisi. */
    value,
    /** Callback set value — invoqué quand l'utilisateur clique un radio. */
    onChange,
    /** Si true → un radio doit être coché pour valider le formulaire parent. */
    required = false,
    /** Si true → désactive l'ensemble du fieldset (lecture seule). */
    disabled = false,
    /** Préfixe data-testid pour cibler une instance précise dans un parent
     *  qui contient plusieurs ScoreInput (ex: `quality`, `timeliness`).
     *  Si fourni, les radios deviennent `{testIdPrefix}-score-input-{n}`. */
    testIdPrefix = undefined as string | undefined,
  }: {
    name: string;
    label: string;
    value: number | null;
    onChange: (next: number) => void;
    required?: boolean;
    disabled?: boolean;
    testIdPrefix?: string | undefined;
  } = $props();

  /** Plage statique 1..5 — dérivée des bornes API (SCORE_MIN/MAX). */
  const SCORES: ReadonlyArray<number> = Array.from(
    { length: SCORE_MAX - SCORE_MIN + 1 },
    (_, i) => SCORE_MIN + i,
  );

  function radioTestId(score: number): string {
    return testIdPrefix !== undefined
      ? `${testIdPrefix}-score-input-${score}`
      : `score-input-${score}`;
  }

  function radioId(score: number): string {
    return `${name}-score-${score}`;
  }
</script>

<fieldset
  class="flex flex-col gap-1"
  disabled={disabled}
>
  <legend class="text-sm font-medium text-gray-700">
    {label}
    {#if required}
      <span class="text-red-600" aria-hidden="true">*</span>
    {/if}
  </legend>
  <div
    class="flex items-center gap-3"
    role="radiogroup"
    aria-label={label}
    aria-required={required}
  >
    {#each SCORES as score (score)}
      <label
        for={radioId(score)}
        class={`inline-flex flex-col items-center gap-1 cursor-pointer select-none ${
          disabled ? "opacity-50 cursor-not-allowed" : ""
        }`}
      >
        <input
          id={radioId(score)}
          data-testid={radioTestId(score)}
          type="radio"
          {name}
          value={score}
          checked={value === score}
          {disabled}
          onchange={() => onChange(score)}
          class="h-4 w-4 text-blue-600 border-gray-300 focus:ring-blue-500"
          aria-label={`${label} : ${score} sur ${SCORE_MAX}`}
        />
        <span class="text-xs text-gray-700">{score}</span>
      </label>
    {/each}
  </div>
</fieldset>
