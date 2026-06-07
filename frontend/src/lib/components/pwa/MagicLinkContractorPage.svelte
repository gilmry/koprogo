<script lang="ts">
  // Story 3.3 — PWA Contractor: 3-screen flow (résumé → action → confirmation).
  //
  // Svelte 5 runes only ($state / $derived / $effect / $props). NO classic
  // stores, NO `export let`, NO `$:` reactive blocks (cf. CLAUDE.md §rules).
  //
  // The component is mounted client-side from `/src/pages/c/[token].astro`
  // (`<MagicLinkContractorPage client:load />`) — server-side rendering already
  // resolved the scope payload via `GET /api/v1/c/<token>`. The component:
  //
  //   1. shows screen 1 (summary of the scope object).
  //   2. on "Répondre" click → screen 2 (action form scoped per scope_kind).
  //   3. on submit → POST to `/c/<token>/respond` (placeholder endpoint, see
  //      follow-up). On success → screen 3. On offline → keep draft in IDB.
  //
  // Offline-safe: every keystroke on the draft mutates `$state` AND is
  // persisted to IndexedDB at `magic-link-draft-<token>`. At mount we restore
  // any pre-existing draft for this token. On successful submit we purge.
  //
  // A11y (cf. memory a11y-wcag-aa-baseline):
  //   - Each form field has a <label for> bound to a matching input id.
  //   - The offline banner uses `aria-live="polite"`.
  //   - Buttons have explicit `aria-label` when their icon-only or short.
  //   - Tap targets ≥ 44×44 px (Tailwind `min-h-[44px] min-w-[44px]`).
  //
  // data-testid contract (cf. memory data-testid-systematic):
  //   pwa-screen-1-summary, pwa-screen-2-action, pwa-screen-3-confirm,
  //   pwa-install-prompt, pwa-action-submit, pwa-action-message-input,
  //   pwa-action-amount-input, pwa-offline-indicator.

  import { onMount } from "svelte";
  import { apiEndpoint } from "../../config";

  type ScopeKind = "ticket" | "quote" | "invoice" | "contractor_evaluation";

  type Props = {
    /** Opaque magic-link token. Used as the IDB key and the POST URL path. */
    token: string;
    /** Scope discriminator returned by `GET /api/v1/c/<token>`. */
    scopeKind: ScopeKind;
    /** Server-resolved scope payload (already consumed). */
    scope: Record<string, unknown>;
  };

  let { token, scopeKind, scope }: Props = $props();

  // -------------------------------------------------------------------------
  // Local state — runes only
  // -------------------------------------------------------------------------

  /** Current screen of the flow (1 = summary, 2 = action, 3 = confirm). */
  let screen = $state<1 | 2 | 3>(1);

  /** Draft answer — persisted to IndexedDB at every change. */
  let draft = $state<{ message: string; amount?: number }>({
    message: "",
  });

  /** Whether we have restored a pre-existing draft from IDB. */
  let draftRestored = $state(false);

  /** Whether the browser reports online connectivity. */
  let online = $state(true);

  /** Whether a submit is in flight. */
  let submitting = $state(false);

  /** Server-side error reported during submit (e.g. 403, 500). */
  let submitError = $state<string | null>(null);

  /** beforeinstallprompt event captured for the explicit install button. */
  let installPromptEvent = $state<{
    prompt: () => Promise<void>;
    userChoice: Promise<unknown>;
  } | null>(null);

  // -------------------------------------------------------------------------
  // Derivations
  // -------------------------------------------------------------------------

  let canShowAmount = $derived(
    scopeKind === "quote" || scopeKind === "invoice",
  );

  let scopeTitle = $derived.by(() => {
    switch (scopeKind) {
      case "ticket":
        return "Ticket";
      case "quote":
        return "Devis";
      case "invoice":
        return "Facture";
      case "contractor_evaluation":
        return "Évaluation prestataire";
    }
  });

  let actionVerb = $derived.by(() => {
    switch (scopeKind) {
      case "ticket":
        return "Répondre au ticket";
      case "quote":
        return "Soumettre un devis";
      case "invoice":
        return "Confirmer la facture";
      case "contractor_evaluation":
        return "Envoyer l'évaluation";
    }
  });

  let submitDisabled = $derived(submitting || draft.message.trim() === "");

  // -------------------------------------------------------------------------
  // IndexedDB draft persistence — minimal, dedicated store
  // -------------------------------------------------------------------------

  const IDB_NAME = "koprogo-pwa-contractor";
  const IDB_STORE = "magic-link-drafts";
  const IDB_VERSION = 1;

  function draftKey(): string {
    return `magic-link-draft-${token}`;
  }

  async function openDraftDb(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
      const req = indexedDB.open(IDB_NAME, IDB_VERSION);
      req.onerror = () => reject(req.error);
      req.onupgradeneeded = () => {
        const db = req.result;
        if (!db.objectStoreNames.contains(IDB_STORE)) {
          db.createObjectStore(IDB_STORE);
        }
      };
      req.onsuccess = () => resolve(req.result);
    });
  }

  async function loadDraft(): Promise<typeof draft | null> {
    try {
      const db = await openDraftDb();
      return await new Promise((resolve, reject) => {
        const tx = db.transaction([IDB_STORE], "readonly");
        const store = tx.objectStore(IDB_STORE);
        const req = store.get(draftKey());
        req.onsuccess = () => resolve((req.result as typeof draft) ?? null);
        req.onerror = () => reject(req.error);
      });
    } catch (_err) {
      // IDB unavailable (private browsing, quota) — graceful degradation.
      return null;
    }
  }

  async function saveDraft(value: typeof draft): Promise<void> {
    try {
      const db = await openDraftDb();
      await new Promise<void>((resolve, reject) => {
        const tx = db.transaction([IDB_STORE], "readwrite");
        const store = tx.objectStore(IDB_STORE);
        const req = store.put(value, draftKey());
        req.onsuccess = () => resolve();
        req.onerror = () => reject(req.error);
      });
    } catch (_err) {
      // Best-effort: a failed write is logged in dev but never blocks UX.
    }
  }

  async function purgeDraft(): Promise<void> {
    try {
      const db = await openDraftDb();
      await new Promise<void>((resolve, reject) => {
        const tx = db.transaction([IDB_STORE], "readwrite");
        const store = tx.objectStore(IDB_STORE);
        const req = store.delete(draftKey());
        req.onsuccess = () => resolve();
        req.onerror = () => reject(req.error);
      });
    } catch (_err) {
      /* ignore */
    }
  }

  // -------------------------------------------------------------------------
  // Effects — IDB sync + online/offline + install prompt
  // -------------------------------------------------------------------------

  onMount(() => {
    online =
      typeof navigator !== "undefined" && typeof navigator.onLine === "boolean"
        ? navigator.onLine
        : true;

    const onOnline = () => {
      online = true;
    };
    const onOffline = () => {
      online = false;
    };
    window.addEventListener("online", onOnline);
    window.addEventListener("offline", onOffline);

    // Capture the install prompt event so we can show a dedicated CTA.
    const onBeforeInstall = (e: Event) => {
      e.preventDefault();
      installPromptEvent = e as unknown as typeof installPromptEvent;
    };
    window.addEventListener("beforeinstallprompt", onBeforeInstall);

    // Restore any draft persisted from a previous offline session.
    void loadDraft().then((restored) => {
      if (restored) {
        draft = restored;
      }
      draftRestored = true;
    });

    return () => {
      window.removeEventListener("online", onOnline);
      window.removeEventListener("offline", onOffline);
      window.removeEventListener("beforeinstallprompt", onBeforeInstall);
    };
  });

  // Persist on every draft mutation (but only once we've finished restoring).
  $effect(() => {
    if (!draftRestored) return;
    // Touch reactive props so the effect re-runs.
    const snapshot = { message: draft.message, amount: draft.amount };
    void saveDraft(snapshot);
  });

  // -------------------------------------------------------------------------
  // Actions
  // -------------------------------------------------------------------------

  function goToAction(): void {
    screen = 2;
  }

  async function triggerInstall(): Promise<void> {
    if (!installPromptEvent) return;
    try {
      await installPromptEvent.prompt();
      // Don't await userChoice — we only need to consume the event so the
      // browser doesn't re-fire it.
      installPromptEvent = null;
    } catch (_err) {
      // User dismissed or browser refused — silently swallow.
    }
  }

  async function submit(): Promise<void> {
    if (submitDisabled) return;
    submitting = true;
    submitError = null;

    const payload = {
      message: draft.message.trim(),
      amount: draft.amount,
    };

    try {
      const resp = await fetch(
        apiEndpoint(`/c/${encodeURIComponent(token)}/respond`),
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        },
      );

      if (!resp.ok) {
        // 4xx/5xx — surface a typed error to the user. We don't retry
        // automatically because a 403 (e.g. token already consumed) must
        // not silently re-submit.
        const body = await resp.json().catch(() => ({}));
        submitError =
          (body?.error as string) ??
          `Erreur ${resp.status} — réessayez plus tard.`;
        return;
      }

      // Success — purge draft + advance to confirmation.
      await purgeDraft();
      screen = 3;
    } catch (_err) {
      // Network failure: the draft remains in IDB. The user will see the
      // offline indicator and can retry once back online.
      submitError =
        "Connexion perdue — votre brouillon est conservé. Réessayez à la reconnexion.";
    } finally {
      submitting = false;
    }
  }
</script>

<div
  class="pwa-contractor max-w-2xl mx-auto px-4 py-6"
  data-testid="pwa-contractor-root"
>
  <!-- Offline indicator — aria-live polite so AT announces transitions. -->
  {#if !online}
    <div
      class="mb-4 rounded-lg border border-amber-300 bg-amber-50 px-4 py-3 text-amber-900"
      data-testid="pwa-offline-indicator"
      role="status"
      aria-live="polite"
    >
      <p class="text-sm font-medium">
        Mode hors-ligne — votre brouillon sera synchronisé à la reconnexion.
      </p>
    </div>
  {/if}

  <!-- Install prompt CTA — only visible when the browser fired the event. -->
  {#if installPromptEvent}
    <button
      type="button"
      class="mb-4 w-full min-h-[44px] rounded-lg bg-sky-600 px-4 py-2 text-white font-medium hover:bg-sky-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-700"
      data-testid="pwa-install-prompt"
      aria-label="Installer l'application KoproGo Contractor"
      onclick={triggerInstall}
    >
      Installer l'application
    </button>
  {/if}

  <!-- ===================================================================== -->
  <!-- Screen 1 — Résumé                                                     -->
  <!-- ===================================================================== -->

  {#if screen === 1}
    <section
      data-testid="pwa-screen-1-summary"
      aria-labelledby="pwa-summary-heading"
    >
      <h1
        id="pwa-summary-heading"
        class="text-xl font-semibold text-gray-900 mb-3"
      >
        {scopeTitle}
      </h1>
      <div
        class="rounded-lg border border-gray-200 bg-white p-4 shadow-sm overflow-x-auto"
        data-testid="pwa-summary-content"
      >
        <pre class="text-sm whitespace-pre-wrap text-gray-700">{JSON.stringify(
            scope,
            null,
            2,
          )}</pre>
      </div>

      <div class="mt-5 flex justify-end">
        <button
          type="button"
          class="min-h-[44px] min-w-[44px] rounded-lg bg-sky-600 px-5 py-2 text-white font-medium hover:bg-sky-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-700"
          data-testid="pwa-summary-next"
          aria-label={actionVerb}
          onclick={goToAction}
        >
          {actionVerb}
        </button>
      </div>
    </section>
  {/if}

  <!-- ===================================================================== -->
  <!-- Screen 2 — Action (form)                                              -->
  <!-- ===================================================================== -->

  {#if screen === 2}
    <section
      data-testid="pwa-screen-2-action"
      aria-labelledby="pwa-action-heading"
    >
      <h1
        id="pwa-action-heading"
        class="text-xl font-semibold text-gray-900 mb-3"
      >
        {actionVerb}
      </h1>

      <form
        class="space-y-4"
        onsubmit={(e) => {
          e.preventDefault();
          void submit();
        }}
      >
        <div>
          <label
            for="pwa-action-message"
            class="block text-sm font-medium text-gray-800 mb-1"
          >
            Message
          </label>
          <textarea
            id="pwa-action-message"
            data-testid="pwa-action-message-input"
            class="w-full min-h-[120px] rounded-lg border border-gray-300 px-3 py-2 text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-600"
            bind:value={draft.message}
            required
            aria-required="true"
            placeholder="Décrivez votre intervention, vos disponibilités…"
          ></textarea>
        </div>

        {#if canShowAmount}
          <div>
            <label
              for="pwa-action-amount"
              class="block text-sm font-medium text-gray-800 mb-1"
            >
              Montant (EUR)
            </label>
            <input
              id="pwa-action-amount"
              type="number"
              inputmode="decimal"
              step="0.01"
              min="0"
              data-testid="pwa-action-amount-input"
              class="w-full min-h-[44px] rounded-lg border border-gray-300 px-3 py-2 text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-600"
              bind:value={draft.amount}
              placeholder="0.00"
            />
          </div>
        {/if}

        {#if submitError}
          <div
            class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800"
            data-testid="pwa-action-error"
            role="alert"
          >
            {submitError}
          </div>
        {/if}

        <div class="flex justify-end gap-2">
          <button
            type="button"
            class="min-h-[44px] rounded-lg border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gray-400"
            data-testid="pwa-action-back"
            aria-label="Revenir au résumé"
            onclick={() => (screen = 1)}
          >
            Retour
          </button>
          <button
            type="submit"
            class="min-h-[44px] rounded-lg bg-sky-600 px-5 py-2 text-white font-medium disabled:bg-gray-300 disabled:cursor-not-allowed hover:bg-sky-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-700"
            data-testid="pwa-action-submit"
            disabled={submitDisabled}
            aria-label="Envoyer la réponse au syndic"
          >
            {submitting ? "Envoi…" : "Envoyer"}
          </button>
        </div>
      </form>
    </section>
  {/if}

  <!-- ===================================================================== -->
  <!-- Screen 3 — Confirmation                                               -->
  <!-- ===================================================================== -->

  {#if screen === 3}
    <section
      data-testid="pwa-screen-3-confirm"
      aria-labelledby="pwa-confirm-heading"
    >
      <h1
        id="pwa-confirm-heading"
        class="text-xl font-semibold text-gray-900 mb-3"
      >
        Reçu
      </h1>
      <div
        class="rounded-lg border border-green-200 bg-green-50 p-4 text-green-900"
      >
        <p class="font-medium">
          Votre réponse a bien été transmise. Le syndic sera notifié.
        </p>
        <p class="text-sm mt-2">
          Vous pouvez fermer cette page en toute sécurité.
        </p>
      </div>
      <div class="mt-5 flex justify-end">
        <button
          type="button"
          class="min-h-[44px] rounded-lg border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gray-400"
          data-testid="pwa-confirm-close"
          aria-label="Fermer la page"
          onclick={() => {
            try {
              window.close();
            } catch (_err) {
              /* Browser may refuse to close — fine. */
            }
          }}
        >
          Fermer
        </button>
      </div>
    </section>
  {/if}
</div>

<style>
  .pwa-contractor {
    /* Make sure all tap targets remain accessible on small screens. */
    font-family:
      system-ui,
      -apple-system,
      "Segoe UI",
      sans-serif;
  }
</style>
