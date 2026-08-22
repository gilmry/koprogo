<script lang="ts">
  // Story B2 (Phase B FE) — MagicLinkIssueForm (form syndic émission).
  //
  // Parent BE story 3.2 — MagicLink (commit `d08407c`).
  //
  // Le composant a DEUX vues mutuellement exclusives, pilotées par `$state` :
  //   - "form"   → form initial (sélecteur user, scope_kind, scope_id, slider).
  //   - "issued" → écran récap : URL `/c?t=<token>` + bouton Copy + warning
  //                "ne sera plus jamais affiché" + bouton "Émettre un nouveau".
  //
  // INV-FE5 (token sensitivity, cf. CRITICAL.md §B2 Gotcha #2) :
  //   Le token brut ne persiste JAMAIS en `localStorage` / `sessionStorage` —
  //   uniquement dans `issuedToken: $state<string | null>` qui disparaît au
  //   unmount du composant (changer de route OU cliquer "Émettre un nouveau").
  //
  // INV-FE9 (a11y) :
  //   - Slider `expires_in_seconds` : aria-valuemin/max/now + <label for>.
  //   - Alerte warning : role="alert" + aria-live="assertive".
  //   - Input readonly token : aria-label + couleur bg-gray-100 explicite.
  //   - Bouton copy : aria-label="Copier le lien magique".
  //
  // Gotcha #1 (cf. stories.md §B2) : `navigator.clipboard.writeText` requiert
  // HTTPS ou localhost. Fallback `document.execCommand('copy')` quand
  // `!window.isSecureContext` — avec un `console.warn` (pas un crash).
  //
  // data-testid (stables, i18n-safe — cf. memory `data-testid-systematic`) :
  //   Form :   magic-link-{target-input, target-option-{userId},
  //                       scope-select, scope-id-select, scope-id-option-{id},
  //                       expires-in-input, expires-in-display, issue-submit}
  //   Result : magic-link-{issued-url-copy, issued-url-input,
  //                       issued-warning, issue-reset}

  import {
    issueMagicLink,
    MAGIC_LINK_SCOPE_KINDS,
    MAGIC_LINK_MIN_EXPIRES_IN_SECONDS,
    MAGIC_LINK_MAX_EXPIRES_IN_SECONDS,
    MAGIC_LINK_DEFAULT_EXPIRES_IN_SECONDS,
    type MagicLinkScopeKind,
    type IssuedMagicLink,
  } from "../../api/magic_links";
  import { toast } from "../../../stores/toast";

  // ---------------------------------------------------------------------------
  // Props (Svelte 5 runes) — typés pour tests Vitest déterministes.
  // ---------------------------------------------------------------------------

  /** Listing de users sélectionnables comme `subject_user_id` (autocomplete). */
  type UserOption = { id: string; label: string };
  /** Listing de scope IDs filtrés par scope_kind (ex: tickets de l'org). */
  type ScopeIdOption = { id: string; label: string };

  let {
    /** Source des users — injectable en test. Si non fournie, autocomplete vide. */
    users = [] as UserOption[],
    /** Source des scope IDs filtrés par kind — injectable en test. */
    scopeIdsByKind = {} as Partial<Record<MagicLinkScopeKind, ScopeIdOption[]>>,
    /** User ID courant — pour bloquer subject = self (INV-13). */
    currentUserId = "" as string,
    /** Base URL pour rendre l'URL `/c?t=...` (par défaut = origin courant). */
    publicBaseUrl = (typeof window !== "undefined"
      ? window.location.origin
      : "") as string,
    /** Injection de la fonction d'émission — facilite mock Vitest. */
    onIssue = issueMagicLink as (
      req: import("../../api/magic_links").IssueMagicLinkRequest,
    ) => Promise<IssuedMagicLink>,
  }: {
    users?: UserOption[];
    scopeIdsByKind?: Partial<Record<MagicLinkScopeKind, ScopeIdOption[]>>;
    currentUserId?: string;
    publicBaseUrl?: string;
    onIssue?: (
      req: import("../../api/magic_links").IssueMagicLinkRequest,
    ) => Promise<IssuedMagicLink>;
  } = $props();

  // ---------------------------------------------------------------------------
  // State local — TOUT dans `$state`. Le token brut JAMAIS persisté.
  // ---------------------------------------------------------------------------

  /** Vue active : "form" ou "issued". */
  let view = $state<"form" | "issued">("form");

  /** Form fields. */
  let subjectUserId = $state<string>("");
  let scopeKind = $state<MagicLinkScopeKind>("ticket");
  let scopeId = $state<string>("");
  let expiresInSeconds = $state<number>(
    MAGIC_LINK_DEFAULT_EXPIRES_IN_SECONDS,
  );

  /** Erreur backend ou validation — affichée inline sous le form. */
  let errorMessage = $state<string>("");

  /** Soumission en cours — disable submit. */
  let submitting = $state<boolean>(false);

  /** Token brut renvoyé par le backend — uniquement en mémoire `$state`. */
  let issuedToken = $state<string | null>(null);
  let issuedExpiresAt = $state<string | null>(null);

  // ---------------------------------------------------------------------------
  // Derivations
  // ---------------------------------------------------------------------------

  /** Liste filtrée des scope_ids pour le kind sélectionné. */
  let availableScopeIds = $derived<ScopeIdOption[]>(
    scopeIdsByKind[scopeKind] ?? [],
  );

  /** Submit disabled tant que tous les champs ne sont pas valides. */
  let submitDisabled = $derived(
    submitting ||
      subjectUserId === "" ||
      // INV-13 — subject ≠ issuer (frontend pre-check, double-check backend).
      subjectUserId === currentUserId ||
      scopeId === "" ||
      availableScopeIds.length === 0 ||
      expiresInSeconds < MAGIC_LINK_MIN_EXPIRES_IN_SECONDS ||
      expiresInSeconds > MAGIC_LINK_MAX_EXPIRES_IN_SECONDS,
  );

  /** Helper text affiché sous le sélecteur de scope quand vide. */
  let scopeHelperText = $derived(
    availableScopeIds.length === 0
      ? scopeKindLabel(scopeKind, true)
      : "",
  );

  /** Helper text si subject = self (INV-13). */
  let subjectHelperText = $derived(
    subjectUserId !== "" && subjectUserId === currentUserId
      ? "Vous ne pouvez pas vous émettre un lien à vous-même."
      : "",
  );

  /** Label humain pour la valeur du slider. */
  let expiresInDisplay = $derived(humanDuration(expiresInSeconds));

  /** URL publique finale `<base>/c?t=<token>`. */
  let publicUrl = $derived(
    issuedToken
      ? `${publicBaseUrl}/c?t=${encodeURIComponent(issuedToken)}`
      : "",
  );

  // ---------------------------------------------------------------------------
  // Helpers (pures — testables si besoin)
  // ---------------------------------------------------------------------------

  function scopeKindLabel(
    kind: MagicLinkScopeKind,
    asEmptyHelper: boolean = false,
  ): string {
    const labels: Record<MagicLinkScopeKind, [string, string]> = {
      ticket: ["Ticket", "Aucun ticket trouvé"],
      quote: ["Devis", "Aucun devis trouvé"],
      invoice: ["Facture", "Aucune facture trouvée"],
      contractor_evaluation: [
        "Évaluation prestataire",
        "Aucune évaluation trouvée",
      ],
    };
    const [label, empty] = labels[kind];
    return asEmptyHelper ? empty : label;
  }

  function humanDuration(seconds: number): string {
    if (seconds < 60) return `${seconds} sec`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return minutes === 1 ? "1 minute" : `${minutes} minutes`;
    const hours = Math.floor(seconds / 3600);
    if (hours < 24) return hours === 1 ? "1 heure" : `${hours} heures`;
    const days = Math.floor(seconds / 86400);
    return days === 1 ? "1 jour" : `${days} jours`;
  }

  function formatExpiresAt(iso: string): string {
    try {
      const d = new Date(iso);
      // Format FR-BE : "16 juin 2026 à 17h45"
      const dateFmt = new Intl.DateTimeFormat("fr-BE", {
        day: "numeric",
        month: "long",
        year: "numeric",
      }).format(d);
      const timeFmt = new Intl.DateTimeFormat("fr-BE", {
        hour: "2-digit",
        minute: "2-digit",
      }).format(d);
      return `${dateFmt} à ${timeFmt}`;
    } catch {
      return iso;
    }
  }

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  async function submit(): Promise<void> {
    if (submitDisabled) return;
    submitting = true;
    errorMessage = "";
    try {
      const issued = await onIssue({
        subject_user_id: subjectUserId,
        scope_kind: scopeKind,
        scope_id: scopeId,
        expires_in_seconds: expiresInSeconds,
      });
      // INV-FE5 : token reste en $state local UNIQUEMENT.
      issuedToken = issued.token;
      issuedExpiresAt = issued.expires_at;
      view = "issued";
      toast.success("Lien magique émis avec succès.");
    } catch (err) {
      // Le wrapper `api.ts` toast déjà 401/403/429/5xx. On affiche en plus
      // un message inline pour 4xx-de-validation (422 typiquement) — cf. AC
      // @edge "Durée minimale" et @security "subject = self".
      const msg = err instanceof Error ? err.message : String(err);
      errorMessage = msg;
    } finally {
      submitting = false;
    }
  }

  async function copyUrl(): Promise<void> {
    if (!publicUrl) return;
    try {
      if (
        typeof navigator !== "undefined" &&
        navigator.clipboard &&
        typeof window !== "undefined" &&
        window.isSecureContext
      ) {
        await navigator.clipboard.writeText(publicUrl);
        toast.success("Lien copié dans le presse-papier.");
        return;
      }
      // Fallback dev HTTP — execCommand est deprecated mais fonctionne.
      console.warn(
        "[MagicLinkIssueForm] navigator.clipboard indisponible (contexte non sécurisé) — fallback execCommand.",
      );
      const ta = document.createElement("textarea");
      ta.value = publicUrl;
      ta.style.position = "fixed";
      ta.style.left = "-9999px";
      document.body.appendChild(ta);
      ta.select();
      const ok = document.execCommand("copy");
      document.body.removeChild(ta);
      if (ok) {
        toast.success("Lien copié dans le presse-papier.");
      } else {
        toast.error("Impossible de copier — copiez le lien manuellement.");
      }
    } catch (err) {
      console.warn("[MagicLinkIssueForm] copy failed", err);
      toast.error("Impossible de copier — copiez le lien manuellement.");
    }
  }

  function reset(): void {
    // Effacer le token de la mémoire en plus de l'unmount potentiel.
    issuedToken = null;
    issuedExpiresAt = null;
    errorMessage = "";
    view = "form";
    // On garde les autres champs comme préférences UX (subject/scope/slider).
  }

  // ---------------------------------------------------------------------------
  // Réinitialise scope_id quand on change de scope_kind (sinon on garde un
  // id qui pointait vers un ticket alors qu'on est passé à invoice).
  // ---------------------------------------------------------------------------

  $effect(() => {
    // dépend de scopeKind — vide scopeId à chaque changement de kind.
    const _kind = scopeKind;
    scopeId = "";
  });
</script>

<section
  class="magic-link-issue-form max-w-2xl rounded-lg border border-gray-200 bg-white p-6 shadow-sm"
  aria-labelledby="magic-link-form-title"
>
  {#if view === "form"}
    <h2 id="magic-link-form-title" class="mb-4 text-xl font-semibold text-gray-900">
      Émettre un lien magique
    </h2>

    <form
      class="space-y-4"
      onsubmit={(e: SubmitEvent) => {
        e.preventDefault();
        void submit();
      }}
    >
      <!-- Destinataire (subject_user_id) -->
      <div>
        <label for="magic-link-target-input" class="block text-sm font-medium text-gray-700">
          Destinataire
        </label>
        <select
          id="magic-link-target-input"
          data-testid="magic-link-target-input"
          bind:value={subjectUserId}
          class="mt-1 min-h-[44px] w-full rounded-md border border-gray-300 px-3 py-2 focus-visible:outline-2 focus-visible:outline-offset-2"
          aria-describedby={subjectHelperText
            ? "magic-link-target-help"
            : undefined}
        >
          <option value="">— Sélectionner —</option>
          {#each users as user (user.id)}
            <option
              data-testid={`magic-link-target-option-${user.id}`}
              value={user.id}
            >
              {user.label}
            </option>
          {/each}
        </select>
        {#if subjectHelperText}
          <p
            id="magic-link-target-help"
            data-testid="magic-link-target-help"
            class="mt-1 text-sm text-red-600"
            role="alert"
          >
            {subjectHelperText}
          </p>
        {/if}
      </div>

      <!-- Type de ressource (scope_kind) -->
      <div>
        <label for="magic-link-scope-select" class="block text-sm font-medium text-gray-700">
          Type de ressource
        </label>
        <select
          id="magic-link-scope-select"
          data-testid="magic-link-scope-select"
          bind:value={scopeKind}
          class="mt-1 min-h-[44px] w-full rounded-md border border-gray-300 px-3 py-2 focus-visible:outline-2 focus-visible:outline-offset-2"
        >
          {#each MAGIC_LINK_SCOPE_KINDS as kind (kind)}
            <option value={kind}>{scopeKindLabel(kind)}</option>
          {/each}
        </select>
      </div>

      <!-- Ressource (scope_id) -->
      <div>
        <label for="magic-link-scope-id-select" class="block text-sm font-medium text-gray-700">
          Ressource
        </label>
        <select
          id="magic-link-scope-id-select"
          data-testid="magic-link-scope-id-select"
          bind:value={scopeId}
          disabled={availableScopeIds.length === 0}
          class="mt-1 min-h-[44px] w-full rounded-md border border-gray-300 px-3 py-2 disabled:bg-gray-100 disabled:text-gray-500 focus-visible:outline-2 focus-visible:outline-offset-2"
          aria-describedby={scopeHelperText
            ? "magic-link-scope-id-help"
            : undefined}
        >
          <option value="">— Sélectionner —</option>
          {#each availableScopeIds as opt (opt.id)}
            <option
              data-testid={`magic-link-scope-id-option-${opt.id}`}
              value={opt.id}
            >
              {opt.label}
            </option>
          {/each}
        </select>
        {#if scopeHelperText}
          <p
            id="magic-link-scope-id-help"
            data-testid="magic-link-scope-id-help"
            class="mt-1 text-sm text-gray-600"
          >
            {scopeHelperText}
          </p>
        {/if}
      </div>

      <!-- Durée (expires_in_seconds) -->
      <div>
        <label for="magic-link-expires-in-input" class="block text-sm font-medium text-gray-700">
          Validité
        </label>
        <input
          id="magic-link-expires-in-input"
          data-testid="magic-link-expires-in-input"
          type="range"
          bind:value={expiresInSeconds}
          min={MAGIC_LINK_MIN_EXPIRES_IN_SECONDS}
          max={MAGIC_LINK_MAX_EXPIRES_IN_SECONDS}
          step="60"
          class="mt-1 w-full"
          aria-valuemin={MAGIC_LINK_MIN_EXPIRES_IN_SECONDS}
          aria-valuemax={MAGIC_LINK_MAX_EXPIRES_IN_SECONDS}
          aria-valuenow={expiresInSeconds}
          aria-valuetext={expiresInDisplay}
        />
        <div class="mt-1 flex items-center justify-between text-xs text-gray-500">
          <span>1 min</span>
          <span
            data-testid="magic-link-expires-in-display"
            class="text-sm font-semibold text-gray-900"
          >
            {expiresInDisplay}
          </span>
          <span>30 j</span>
        </div>
      </div>

      <!-- Erreur backend (422 typiquement) -->
      {#if errorMessage}
        <p
          data-testid="magic-link-form-error"
          class="rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-700"
          role="alert"
          aria-live="polite"
        >
          {errorMessage}
        </p>
      {/if}

      <!-- Submit -->
      <button
        data-testid="magic-link-issue-submit"
        type="submit"
        disabled={submitDisabled}
        class="min-h-[44px] w-full rounded-md bg-blue-600 px-4 py-2 font-semibold text-white shadow-sm hover:bg-blue-700 focus-visible:outline-2 focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:bg-gray-300"
      >
        {submitting ? "Émission…" : "Émettre"}
      </button>
    </form>
  {:else}
    <!-- view === "issued" — écran récap.  -->
    <h2 id="magic-link-form-title" class="mb-2 text-xl font-semibold text-green-700">
      ✅ Lien émis
    </h2>

    <p
      data-testid="magic-link-issued-warning"
      class="mb-4 rounded-md border border-orange-300 bg-orange-50 p-3 text-sm font-medium text-orange-800"
      role="alert"
      aria-live="assertive"
    >
      ⚠ Ce lien ne sera plus jamais affiché. Copiez-le maintenant et envoyez-le
      au destinataire.
    </p>

    <label for="magic-link-issued-url-input" class="block text-sm font-medium text-gray-700">
      URL d'accès magique
    </label>
    <div class="mt-1 flex items-stretch gap-2">
      <input
        id="magic-link-issued-url-input"
        data-testid="magic-link-issued-url-input"
        type="text"
        readonly
        value={publicUrl}
        class="flex-1 rounded-md border border-gray-300 bg-gray-100 px-3 py-2 font-mono text-sm text-gray-900"
        aria-label="Lien magique à copier — lecture seule"
      />
      <button
        type="button"
        data-testid="magic-link-issued-url-copy"
        onclick={() => void copyUrl()}
        class="min-h-[44px] rounded-md bg-blue-600 px-4 py-2 font-semibold text-white shadow-sm hover:bg-blue-700 focus-visible:outline-2 focus-visible:outline-offset-2"
        aria-label="Copier le lien magique"
      >
        Copier
      </button>
    </div>

    {#if issuedExpiresAt}
      <p class="mt-2 text-sm text-gray-600">
        Expire le {formatExpiresAt(issuedExpiresAt)}.
      </p>
    {/if}

    <button
      type="button"
      data-testid="magic-link-issue-reset"
      onclick={reset}
      class="mt-4 min-h-[44px] w-full rounded-md border border-gray-300 bg-white px-4 py-2 font-semibold text-gray-700 hover:bg-gray-50 focus-visible:outline-2 focus-visible:outline-offset-2"
    >
      Émettre un nouveau lien
    </button>
  {/if}
</section>
