<script lang="ts">
  // Story B7 (Phase B FE) — composant atomique réutilisable.
  //
  // Réutilisé par :
  //   - B7 TechnicalSpecSignatureForm (cette story).
  //   - Futur : PV AG, mandats notariés (cf. cluster coord stories.md §B7).
  //
  // Gotcha #2 stories.md §B7 — SignatureForm est l'atomique le plus sensible
  // légalement. Pattern :
  //   1. Checkbox RPGD-style obligatoire "J'ai lu et j'approuve..."
  //   2. Bouton "Signer" disabled tant que la checkbox n'est pas cochée.
  //   3. onSign callback (async) appelé au submit → le parent gère le POST.
  //
  // INV-FE9 (a11y WCAG 2.1 AA) :
  //   - Label associé checkbox via `for`/`id`.
  //   - Button min-h-[44px] cible tactile WCAG 2.5.5.
  //   - aria-disabled cohérent avec disabled (lecteurs d'écran).
  //   - role="status" aria-live="polite" sur l'erreur backend.
  //
  // data-testid (cf. stories.md §B7 atomique) :
  //   signature-confirm-checkbox
  //   signature-sign-button
  //   signature-form-error (si erreur backend)

  // ---------------------------------------------------------------------------
  // Props (Svelte 5 runes)
  // ---------------------------------------------------------------------------

  let {
    /** Texte de la checkbox de confirmation — adapté au contexte
     *  (TechnicalSpec, PV AG, mandat notarié, etc.). */
    confirmLabel = "J'ai lu et j'approuve les éléments ci-dessus.",
    /** Texte du bouton de signature. */
    signLabel = "Signer",
    /** Texte affiché pendant la soumission. */
    signingLabel = "Signature en cours…",
    /** Callback async — le parent gère le POST + erreurs. Si la promise
     *  rejette, on affiche le message inline (errorMessage). */
    onSign,
    /** Désactive le bouton en plus du gating interne (ex: prerequisites
     *  manquants côté parent — mandate inactif, etc.). */
    externallyDisabled = false,
    /** Suffixe data-testid pour permettre plusieurs SignatureForm sur la
     *  même page (rare mais pas exclu). Sans suffix → testid de base. */
    idSuffix = undefined as string | undefined,
    /** Override complet du testid du bouton — utilisé par TechnicalSpecSignatureForm
     *  qui veut exposer `tech-spec-sign-submit` au lieu du pattern auto. */
    buttonTestIdOverride = undefined as string | undefined,
    /** Override complet du testid de la checkbox. */
    checkboxTestIdOverride = undefined as string | undefined,
  }: {
    confirmLabel?: string;
    signLabel?: string;
    signingLabel?: string;
    onSign: () => Promise<void>;
    externallyDisabled?: boolean;
    idSuffix?: string | undefined;
    buttonTestIdOverride?: string | undefined;
    checkboxTestIdOverride?: string | undefined;
  } = $props();

  // ---------------------------------------------------------------------------
  // State local
  // ---------------------------------------------------------------------------

  let confirmed = $state<boolean>(false);
  let submitting = $state<boolean>(false);
  let errorMessage = $state<string>("");

  // ---------------------------------------------------------------------------
  // Derivations
  // ---------------------------------------------------------------------------

  let submitDisabled = $derived(
    submitting || !confirmed || externallyDisabled,
  );

  let checkboxTestId = $derived(
    checkboxTestIdOverride !== undefined
      ? checkboxTestIdOverride
      : idSuffix !== undefined
        ? `signature-confirm-checkbox-${idSuffix}`
        : "signature-confirm-checkbox",
  );

  let buttonTestId = $derived(
    buttonTestIdOverride !== undefined
      ? buttonTestIdOverride
      : idSuffix !== undefined
        ? `signature-sign-button-${idSuffix}`
        : "signature-sign-button",
  );

  let errorTestId = $derived(
    idSuffix !== undefined
      ? `signature-form-error-${idSuffix}`
      : "signature-form-error",
  );

  let checkboxId = $derived(
    idSuffix !== undefined
      ? `signature-confirm-${idSuffix}`
      : "signature-confirm",
  );

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  async function handleSign(): Promise<void> {
    if (submitDisabled) return;
    submitting = true;
    errorMessage = "";
    try {
      await onSign();
      // Reset la checkbox APRÈS succès — le parent décide d'unmount le
      // formulaire ou de le réinitialiser. On reset par cohérence avec le
      // pattern "form de création" (cf. SyndicResponseForm).
      confirmed = false;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      errorMessage = msg;
    } finally {
      submitting = false;
    }
  }
</script>

<div class="signature-form flex flex-col gap-3">
  <!-- Checkbox RPGD-style — gating légal -->
  <label
    for={checkboxId}
    class="flex items-start gap-2 cursor-pointer text-sm text-gray-800"
  >
    <input
      id={checkboxId}
      data-testid={checkboxTestId}
      type="checkbox"
      bind:checked={confirmed}
      class="mt-0.5 h-4 w-4 rounded border-gray-300"
      aria-describedby={errorMessage ? errorTestId : undefined}
    />
    <span>{confirmLabel}</span>
  </label>

  <!-- Bouton de signature -->
  <button
    type="button"
    data-testid={buttonTestId}
    onclick={() => void handleSign()}
    disabled={submitDisabled}
    aria-disabled={submitDisabled}
    class="min-h-[44px] self-start rounded-md bg-blue-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-700 focus-visible:outline-2 focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:bg-gray-300"
  >
    {submitting ? signingLabel : signLabel}
  </button>

  <!-- Erreur backend inline -->
  {#if errorMessage}
    <p
      data-testid={errorTestId}
      class="rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-700"
      role="alert"
      aria-live="polite"
    >
      {errorMessage}
    </p>
  {/if}
</div>
