<script lang="ts">
  // Story 5.7 — OnboardingWizard : mise en service d'une nouvelle ACP en 5
  // étapes (profil → recommandation → activation → démo → confirmation).
  //
  // KPI (SC17/C22) : moins de 5 minutes, mesuré CLIENT-SIDE (pas de round-trip
  // serveur pour la mesure elle-même — `Date.now()` au montage vs à la
  // confirmation).
  //
  // Dépendances déclarées par la story (5.1 `ModuleGuard`/table
  // `acp_enabled_modules`, 5.2 `ModuleGate.svelte`/store `enabled_modules`)
  // n'existent pas encore dans le dépôt à l'écriture de ce composant. L'étape
  // "activation" appelle donc le contrat REST que 5.1 exposera
  // (`PUT /acps/{id}/modules/{module}/enable`) — cohérent avec les
  // data-testid `module-enable-submit` déjà réservés côté 5.1 — mais ne peut
  // pas être vérifiée en E2E réel tant que 5.1 n'est pas mergée. Les tests
  // Vitest de ce fichier mockent la couche `api` et restent donc valides
  // indépendamment de l'ordre d'atterrissage des stories.
  //
  // @security (cf. story) : cet assistant ne s'affiche QUE pour
  // `currentUserRole === "superadmin"`. Ce n'est pas la seule barrière — la
  // page qui monte ce composant vit sous `/admin/*`, déjà couvert par
  // `RouteGuard` + `roleGuards["/admin/*"]` (frontend/src/lib/guards.ts), donc
  // un accès direct par URL sans session superadmin redirige avant même le
  // montage. Le contrôle de rôle ici est une défense en profondeur (même
  // logique que ModuleGate décrite en 5.2 : le client ne remplace jamais le
  // serveur, il évite juste d'afficher ce qu'il ne faut pas).
  //
  // @negative (reprise) : l'état du wizard est journalisé dans IndexedDB à
  // chaque changement (pattern repris de MagicLinkContractorPage.svelte,
  // story 3.3 — DB dédiée, scoped, pas de dépendance runtime ajoutée).
  //
  // data-testid : onboarding-step-{n}, onboarding-module-toggle-{module},
  // onboarding-finish-submit (cf. story) + onboarding-access-denied,
  // onboarding-resumed-banner, onboarding-discard-draft, onboarding-next,
  // onboarding-back, onboarding-skip-recommendation, onboarding-activate-submit,
  // onboarding-demo-next, onboarding-elapsed-time, onboarding-kpi-status.

  import { createAcp, updateAcp, type AcpResponseDto } from "../../api/acps";
  import { api, ApiError } from "../../api";
  import { _ } from "../../i18n";
  import { authStore } from "../../../stores/auth";

  // ---------------------------------------------------------------------
  // Types
  // ---------------------------------------------------------------------

  /**
   * `identity` est le socle toujours actif (jamais désactivable — cf. Story
   * 5.1 @negative "module=identity tentative disable → 403"). Les autres
   * sont les modules métier optionnels de la plateforme.
   */
  export type OnboardingModule =
    | "identity"
    | "community"
    | "accounting"
    | "governance"
    | "ticketing";

  const ALL_MODULES: OnboardingModule[] = [
    "identity",
    "community",
    "accounting",
    "governance",
    "ticketing",
  ];

  /** @edge — modules activés par défaut quand l'utilisateur saute la recommandation. */
  const DEFAULT_MODULES_ON_SKIP: OnboardingModule[] = ["community", "identity"];

  export interface AcpProfileDraft {
    name: string;
    addressStreet: string;
    addressPostalCode: string;
    addressCity: string;
    unitsCount: number;
    hasSharedSpaces: boolean;
  }

  const BLANK_PROFILE: AcpProfileDraft = {
    name: "",
    addressStreet: "",
    addressPostalCode: "",
    addressCity: "",
    unitsCount: 1,
    hasSharedSpaces: false,
  };

  export interface OnboardingResult {
    acpId: string;
    modules: OnboardingModule[];
    elapsedMs: number;
    recommendationSkipped: boolean;
    /**
     * `true` si cette session a repris un brouillon IndexedDB après une
     * interruption. `elapsedMs` inclut alors le temps où le wizard était
     * fermé : il ne mesure plus la performance d'un utilisateur continu et
     * doit être exclu de l'échantillon KPI SC17/C22 (« moins de 5 minutes »),
     * plutôt que compté comme un dépassement d'objectif trompeur.
     */
    wasResumed: boolean;
  }

  export interface OnboardingAnalyticsEvent extends OnboardingResult {
    completedAt: number;
  }

  type Props = {
    /**
     * Rôle actif de l'utilisateur connecté — gate @security. Optionnel : si
     * omis (montage en production depuis la page Astro, qui ne peut pas
     * résoudre le rôle côté serveur dans cette SPA), le composant retombe sur
     * `authStore` (résolu client-side, cf. `stores/auth.ts`). Les tests
     * passent la prop directement pour rester déterministes.
     */
    currentUserRole?: string | null;
    /** Rappelé une fois l'assistant terminé (après purge du brouillon IDB). */
    onFinish?: (result: OnboardingResult) => void;
    /** Mesure client-side du KPI — jamais un round-trip serveur. */
    onAnalytics?: (event: OnboardingAnalyticsEvent) => void;
  };

  let { currentUserRole, onFinish, onAnalytics }: Props = $props();

  // `koprogo_user` (source de `$authStore.user`) est explicitement UN CACHE
  // D'AFFICHAGE, « jamais une preuve d'authentification » (stores/auth.ts) :
  // le rôle n'est fiable que combiné à `isAuthenticated`, confirmé par le
  // silent-refresh. Sans ce garde-fou, une entrée forgée dans localStorage
  // suffirait à afficher l'assistant (le backend refuserait ensuite chaque
  // écriture, mais l'UI l'aurait déjà montré).
  //
  // Tant que la prop est omise ET que le silent-refresh n'a pas conclu
  // (`isLoading`), le rôle n'est PAS résolu : on ne sait ni l'accorder ni le
  // refuser. `authPending` porte cette distinction pour ne jamais afficher
  // « Accès réservé » à un superadmin dont la session est simplement encore
  // en train de se confirmer après un rechargement.
  let authPending = $derived(
    currentUserRole === undefined && $authStore.isLoading,
  );
  let resolvedRole = $derived(
    currentUserRole ??
      ($authStore.isAuthenticated ? ($authStore.user?.role ?? null) : null),
  );
  let accessGranted = $derived(!authPending && resolvedRole === "superadmin");

  // ---------------------------------------------------------------------
  // Recommandation — heuristique pure, testée via le rendu de l'étape 2.
  // ---------------------------------------------------------------------

  function recommendModules(profile: AcpProfileDraft): OnboardingModule[] {
    const modules: OnboardingModule[] = ["identity"];
    if (profile.hasSharedSpaces) modules.push("community");
    if (profile.unitsCount >= 5) modules.push("accounting");
    if (profile.unitsCount >= 10) modules.push("governance");
    return modules;
  }

  // ---------------------------------------------------------------------
  // État — runes uniquement (Svelte 5, cf. CLAUDE.md).
  // ---------------------------------------------------------------------

  let step = $state<1 | 2 | 3 | 4 | 5>(1);
  let profile = $state<AcpProfileDraft>({ ...BLANK_PROFILE });
  let profileError = $state<string | null>(null);

  // Le message d'erreur unique (`profileError`) couvre jusqu'à quatre champs
  // à la fois ; sans ces dérivés, un lecteur d'écran entendrait l'alerte
  // mais jamais lequel des quatre champs est en cause (cf. le pattern par
  // champ de MandateIssueForm.svelte).
  let nameInvalid = $derived(profileError !== null && profile.name.trim() === "");
  let streetInvalid = $derived(
    profileError !== null && profile.addressStreet.trim() === "",
  );
  let postalCodeInvalid = $derived(
    profileError !== null && profile.addressPostalCode.trim() === "",
  );
  let cityInvalid = $derived(
    profileError !== null && profile.addressCity.trim() === "",
  );
  let creatingAcp = $state(false);

  let acpId = $state<string | null>(null);
  let recommendedModules = $state<OnboardingModule[]>([]);
  let selectedModules = $state<OnboardingModule[]>([]);
  let recommendationSkipped = $state(false);

  let activating = $state(false);
  let activationError = $state<string | null>(null);

  let startedAt = $state<number>(Date.now());
  let elapsedMsAtConfirmation = $state<number>(0);

  let draftRestored = $state(false);
  let resumedFromDraft = $state(false);

  const KPI_TARGET_MS = 5 * 60 * 1000;

  // ---------------------------------------------------------------------
  // IndexedDB — reprise après interruption (@negative).
  //
  // DB dédiée, scoped à ce wizard : pas de mélange avec `lib/indexeddb.ts`
  // (offline sync de buildings/units/etc — sémantique différente).
  // ---------------------------------------------------------------------

  const IDB_NAME = "koprogo-onboarding";
  const IDB_STORE = "wizard-drafts";
  const IDB_VERSION = 1;
  const DRAFT_KEY = "current-draft";

  interface WizardDraft {
    step: 1 | 2 | 3 | 4 | 5;
    profile: AcpProfileDraft;
    acpId: string | null;
    recommendedModules: OnboardingModule[];
    selectedModules: OnboardingModule[];
    recommendationSkipped: boolean;
    startedAt: number;
  }

  // Le brouillon est réécrit à chaque frappe (effet de persistance
  // ci-dessous) : ouvrir une connexion neuve à chaque appel multiplierait les
  // handles `IDBDatabase` jamais fermés. On mémorise la promesse d'ouverture
  // pour la partager entre tous les appels de ce montage.
  let dbPromise: Promise<IDBDatabase> | null = null;

  function openDraftDb(): Promise<IDBDatabase> {
    if (!dbPromise) {
      dbPromise = new Promise((resolve, reject) => {
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
    return dbPromise;
  }

  async function loadDraft(): Promise<WizardDraft | null> {
    try {
      const db = await openDraftDb();
      return await new Promise((resolve, reject) => {
        const tx = db.transaction([IDB_STORE], "readonly");
        const store = tx.objectStore(IDB_STORE);
        const req = store.get(DRAFT_KEY);
        req.onsuccess = () => resolve((req.result as WizardDraft) ?? null);
        req.onerror = () => reject(req.error);
      });
    } catch (_err) {
      // IDB indisponible (navigation privée, quota) — dégradation gracieuse :
      // le wizard démarre simplement à l'étape 1, sans reprise.
      return null;
    }
  }

  async function saveDraft(value: WizardDraft): Promise<void> {
    try {
      const db = await openDraftDb();
      await new Promise<void>((resolve, reject) => {
        const tx = db.transaction([IDB_STORE], "readwrite");
        const store = tx.objectStore(IDB_STORE);
        const req = store.put(value, DRAFT_KEY);
        req.onsuccess = () => resolve();
        req.onerror = () => reject(req.error);
      });
    } catch (_err) {
      // Best-effort : une écriture ratée ne doit jamais bloquer l'assistant.
    }
  }

  async function purgeDraft(): Promise<void> {
    try {
      const db = await openDraftDb();
      await new Promise<void>((resolve, reject) => {
        const tx = db.transaction([IDB_STORE], "readwrite");
        const store = tx.objectStore(IDB_STORE);
        const req = store.delete(DRAFT_KEY);
        req.onsuccess = () => resolve();
        req.onerror = () => reject(req.error);
      });
    } catch (_err) {
      /* ignore */
    }
  }

  // Un `$effect` plutôt qu'un `onMount` : `onMount` ne s'exécute qu'UNE fois,
  // immédiatement après le premier rendu — avant que le silent-refresh de
  // `authStore` (asynchrone) n'ait résolu le rôle en production (montage
  // sans prop `currentUserRole`, cf. Props). Il concluait alors
  // `!accessGranted` à tort, marquait `draftRestored = true` sans avoir
  // jamais appelé `loadDraft()`, et la reprise ne se déclenchait plus.
  // L'effet se rejoue quand `authPending`/`accessGranted` changent, et
  // `restoreStarted` l'empêche de relancer `loadDraft()` une seconde fois
  // une fois la tentative faite.
  // Verrou simple (PAS un `$state`) : il ne pilote aucun rendu, seulement
  // l'exécution de l'effet lui-même. En faire un `$state` lu ET écrit par le
  // même effet aurait déclenché une re-exécution supplémentaire — inoffensive
  // ici, mais inutile à raisonner.
  let restoreStarted = false;

  $effect(() => {
    if (authPending) return; // rôle pas encore connu — on attend.
    if (!accessGranted) {
      draftRestored = true;
      return;
    }
    if (restoreStarted) return;
    restoreStarted = true;

    void loadDraft().then((restored) => {
      // Un brouillon "vide" (celui que `discardDraft` réécrit après avoir
      // purgé, cf. l'effet de persistance ci-dessous) ne doit pas déclencher
      // la bannière de reprise pour une session qui vient d'être abandonnée
      // délibérément.
      const estReel =
        restored != null &&
        (restored.acpId != null || restored.profile.name.trim() !== "");
      if (estReel && restored) {
        step = restored.step;
        profile = restored.profile;
        acpId = restored.acpId;
        recommendedModules = restored.recommendedModules;
        selectedModules = restored.selectedModules;
        recommendationSkipped = restored.recommendationSkipped;
        startedAt = restored.startedAt;
        resumedFromDraft = true;
      }
      draftRestored = true;
    });
  });

  // Persiste à chaque changement d'état, une fois la restauration initiale
  // terminée (évite d'écraser un brouillon existant avec l'état vide du
  // premier rendu — même précaution que MagicLinkContractorPage `draftRestored`).
  $effect(() => {
    if (!draftRestored || !accessGranted) return;
    const snapshot: WizardDraft = {
      step,
      profile: { ...profile },
      acpId,
      recommendedModules: [...recommendedModules],
      selectedModules: [...selectedModules],
      recommendationSkipped,
      startedAt,
    };
    void saveDraft(snapshot);
  });

  async function discardDraft(): Promise<void> {
    await purgeDraft();
    step = 1;
    profile = { ...BLANK_PROFILE };
    profileError = null;
    acpId = null;
    recommendedModules = [];
    selectedModules = [];
    recommendationSkipped = false;
    startedAt = Date.now();
    resumedFromDraft = false;
  }

  // ---------------------------------------------------------------------
  // Étape 1 — Profil ACP → création + recommandation.
  // ---------------------------------------------------------------------

  async function goToRecommendation(): Promise<void> {
    profileError = null;

    if (
      profile.name.trim() === "" ||
      profile.addressStreet.trim() === "" ||
      profile.addressPostalCode.trim() === "" ||
      profile.addressCity.trim() === ""
    ) {
      profileError =
        $_("onboarding.profileIncomplete") ||
        "Complétez le nom et l'adresse avant de continuer.";
      return;
    }

    if (!Number.isFinite(profile.unitsCount) || profile.unitsCount < 1) {
      profileError =
        $_("onboarding.unitsCountInvalid") ||
        "Le nombre de lots doit être d'au moins 1.";
      return;
    }

    creatingAcp = true;
    try {
      // `goToRecommendation` peut être rappelée après un "Précédent" depuis
      // l'étape 2 : l'ACP existe déjà, on la met à jour plutôt que d'en
      // créer une seconde en doublon.
      const acp: AcpResponseDto = acpId
        ? await updateAcp(acpId, {
            name: profile.name.trim(),
            address_street: profile.addressStreet.trim(),
            address_postal_code: profile.addressPostalCode.trim(),
            address_city: profile.addressCity.trim(),
            bce_number: null,
          })
        : await createAcp({
            name: profile.name.trim(),
            address_street: profile.addressStreet.trim(),
            address_postal_code: profile.addressPostalCode.trim(),
            address_city: profile.addressCity.trim(),
            organization_id: null,
            bce_number: null,
          });
      acpId = acp.id;
      recommendedModules = recommendModules(profile);
      selectedModules = [...recommendedModules];
      recommendationSkipped = false;
      step = 2;
    } catch (err) {
      profileError =
        err instanceof ApiError
          ? err.message
          : $_("onboarding.acpCreateError") ||
            "Impossible de créer l'ACP. Réessayez.";
    } finally {
      creatingAcp = false;
    }
  }

  // ---------------------------------------------------------------------
  // Étape 2 — Recommandation modules (togglable, ou "passer").
  // ---------------------------------------------------------------------

  function toggleModule(module: OnboardingModule): void {
    if (module === "identity") return; // toujours actif, jamais désactivable.
    selectedModules = selectedModules.includes(module)
      ? selectedModules.filter((m) => m !== module)
      : [...selectedModules, module];
  }

  function skipRecommendation(): void {
    recommendationSkipped = true;
    selectedModules = [...DEFAULT_MODULES_ON_SKIP];
    step = 3;
  }

  function confirmRecommendation(): void {
    step = 3;
  }

  // ---------------------------------------------------------------------
  // Étape 3 — Activation (appel API — contrat Story 5.1, cf. commentaire
  // d'en-tête).
  // ---------------------------------------------------------------------

  async function activateModules(): Promise<void> {
    if (!acpId) return;
    const id = acpId;
    activating = true;
    activationError = null;
    try {
      await Promise.all(
        selectedModules
          .filter((m) => m !== "identity")
          .map((m) =>
            api.put(`/acps/${encodeURIComponent(id)}/modules/${m}/enable`, {}),
          ),
      );
      step = 4;
    } catch (err) {
      activationError =
        err instanceof ApiError
          ? err.message
          : $_("onboarding.activationError") ||
            "Échec de l'activation des modules. Réessayez.";
    } finally {
      activating = false;
    }
  }

  // ---------------------------------------------------------------------
  // Étape 4 — Démo → étape 5 (fige l'horodatage KPI).
  // ---------------------------------------------------------------------

  function finishDemo(): void {
    elapsedMsAtConfirmation = Date.now() - startedAt;
    step = 5;
  }

  // ---------------------------------------------------------------------
  // Étape 5 — Confirmation.
  // ---------------------------------------------------------------------

  async function finish(): Promise<void> {
    if (!acpId) return;
    const result: OnboardingResult = {
      acpId,
      // Copie défensive : `selectedModules` reste un état réactif du
      // composant après l'appel, un appelant qui le muterait (ex. `.sort()`
      // in-place) ne doit pas corrompre le wizard.
      modules: [...selectedModules],
      elapsedMs: elapsedMsAtConfirmation,
      recommendationSkipped,
      wasResumed: resumedFromDraft,
    };
    onAnalytics?.({ ...result, completedAt: Date.now() });
    // Attendu AVANT `onFinish` : si l'appelant navigue (comportement par
    // défaut ci-dessous, ou un `onFinish` fourni qui change de page), une
    // purge non attendue risquerait de ne jamais aboutir — le brouillon
    // survivrait et réapparaîtrait comme une fausse reprise (cf. `discardDraft`).
    await purgeDraft();
    if (onFinish) {
      onFinish(result);
      return;
    }
    // Aucun rappel fourni (montage par défaut depuis `onboarding.astro`, qui
    // ne peut pas sérialiser de fonction à travers l'hydratation d'île
    // Astro) : direction la liste des ACP, où la nouvelle entrée est visible.
    if (typeof window !== "undefined") {
      window.location.href = "/admin/acps";
    }
  }

  function goBack(): void {
    if (step > 1) step = (step - 1) as typeof step;
  }

  function formatElapsed(ms: number): string {
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes} min ${seconds.toString().padStart(2, "0")} s`;
  }

  let underTarget = $derived(elapsedMsAtConfirmation < KPI_TARGET_MS);

  // Gestion du focus au clavier — DoD « accessible clavier seul ».
  //
  // Chaque étape est un bloc `{#if step === N}` distinct : passer à l'étape
  // suivante retire du DOM le bouton qui avait le focus, et le navigateur le
  // reporte sur `<body>`. Un utilisateur clavier ou lecteur d'écran se
  // retrouve alors en tête de document, sans indice sur ce qui vient de se
  // passer — la région `aria-live` ci-dessous annonce le changement, mais
  // n'y ramène pas le focus. On le pose explicitement sur le titre de la
  // nouvelle étape (`tabindex="-1"` : focusable par script, jamais par Tab).
  $effect(() => {
    if (!accessGranted) return;
    // Se relit à chaque changement d'étape.
    const etape = step;
    if (typeof document === "undefined") return;
    const heading = document.getElementById(
      `onboarding-step-${etape}-heading`,
    );
    heading?.focus();
  });
</script>

<div class="onboarding-wizard max-w-2xl mx-auto px-4 py-6" data-testid="onboarding-wizard-root">
  {#if authPending}
    <!-- Rôle pas encore résolu (silent-refresh en cours) : ni le wizard ni
         le refus d'accès ne seraient corrects ici — l'un montrerait une
         capacité pas encore autorisée, l'autre annoncerait à tort un refus
         à un superadmin dont la session est simplement en train de se
         confirmer. -->
    <div
      class="rounded-lg border border-gray-200 bg-white p-4 text-gray-600"
      data-testid="onboarding-checking-access"
      role="status"
      aria-live="polite"
    >
      <p class="text-sm">
        {$_("common.checkingAccess") || "Vérification des accès…"}
      </p>
    </div>
  {:else if !accessGranted}
    <div
      class="rounded-lg border border-red-200 bg-red-50 p-4 text-red-800"
      data-testid="onboarding-access-denied"
      role="alert"
    >
      <p class="font-medium">
        {$_("onboarding.accessDeniedTitle") || "Accès réservé"}
      </p>
      <p class="text-sm mt-1">
        {$_("onboarding.accessDenied") ||
          "Cet assistant est réservé aux administrateurs de la plateforme, lors de la création d'une nouvelle ACP."}
      </p>
    </div>
  {:else}
    <!-- Annonce des changements d'étape pour les lecteurs d'écran. -->
    <p class="sr-only" role="status" aria-live="polite">
      {$_("onboarding.stepOf", { values: { current: step, total: 5 } }) ||
        `Étape ${step} sur 5`}
    </p>

    {#if resumedFromDraft && step !== 5}
      <div
        class="mb-4 rounded-lg border border-amber-300 bg-amber-50 px-4 py-3 text-amber-900 flex items-center justify-between gap-3"
        data-testid="onboarding-resumed-banner"
        role="status"
      >
        <p class="text-sm font-medium">
          {$_("onboarding.resumedBanner") ||
            "Reprise d'une mise en service interrompue."}
        </p>
        <button
          type="button"
          class="min-h-[44px] rounded-lg border border-amber-400 px-3 py-1 text-sm font-medium text-amber-900 hover:bg-amber-100"
          data-testid="onboarding-discard-draft"
          onclick={() => void discardDraft()}
        >
          {$_("onboarding.discardDraft") || "Recommencer à zéro"}
        </button>
      </div>
    {/if}

    {#if step === 1}
      <section data-testid="onboarding-step-1" aria-labelledby="onboarding-step-1-heading">
        <h1
          id="onboarding-step-1-heading"
          tabindex="-1"
          class="text-xl font-semibold text-gray-900 mb-4 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-600"
        >
          {$_("onboarding.step1Title") || "Profil de la copropriété"}
        </h1>

        <div class="space-y-4">
          <div>
            <label for="onboarding-name" class="block text-sm font-medium text-gray-800 mb-1">
              {$_("onboarding.nameLabel") || "Nom de l'ACP"}
            </label>
            <input
              id="onboarding-name"
              type="text"
              data-testid="onboarding-name-input"
              class="w-full min-h-[44px] rounded-lg border border-gray-300 px-3 py-2 text-sm"
              bind:value={profile.name}
              required
              aria-required="true"
              aria-invalid={nameInvalid || undefined}
              aria-describedby={nameInvalid ? "onboarding-profile-error" : undefined}
            />
          </div>

          <div>
            <label for="onboarding-street" class="block text-sm font-medium text-gray-800 mb-1">
              {$_("onboarding.streetLabel") || "Rue et numéro"}
            </label>
            <input
              id="onboarding-street"
              type="text"
              data-testid="onboarding-street-input"
              class="w-full min-h-[44px] rounded-lg border border-gray-300 px-3 py-2 text-sm"
              bind:value={profile.addressStreet}
              required
              aria-required="true"
              aria-invalid={streetInvalid || undefined}
              aria-describedby={streetInvalid ? "onboarding-profile-error" : undefined}
            />
          </div>

          <div class="flex gap-3">
            <div class="flex-1">
              <label for="onboarding-postal-code" class="block text-sm font-medium text-gray-800 mb-1">
                {$_("onboarding.postalCodeLabel") || "Code postal"}
              </label>
              <input
                id="onboarding-postal-code"
                type="text"
                data-testid="onboarding-postal-code-input"
                class="w-full min-h-[44px] rounded-lg border border-gray-300 px-3 py-2 text-sm"
                bind:value={profile.addressPostalCode}
                required
                aria-required="true"
                aria-invalid={postalCodeInvalid || undefined}
                aria-describedby={postalCodeInvalid
                  ? "onboarding-profile-error"
                  : undefined}
              />
            </div>
            <div class="flex-[2]">
              <label for="onboarding-city" class="block text-sm font-medium text-gray-800 mb-1">
                {$_("onboarding.cityLabel") || "Commune"}
              </label>
              <input
                id="onboarding-city"
                type="text"
                data-testid="onboarding-city-input"
                class="w-full min-h-[44px] rounded-lg border border-gray-300 px-3 py-2 text-sm"
                bind:value={profile.addressCity}
                required
                aria-required="true"
                aria-invalid={cityInvalid || undefined}
                aria-describedby={cityInvalid ? "onboarding-profile-error" : undefined}
              />
            </div>
          </div>

          <div>
            <label for="onboarding-units-count" class="block text-sm font-medium text-gray-800 mb-1">
              {$_("onboarding.unitsCountLabel") || "Nombre de lots"}
            </label>
            <input
              id="onboarding-units-count"
              type="number"
              min="1"
              step="1"
              data-testid="onboarding-units-count-input"
              class="w-full min-h-[44px] rounded-lg border border-gray-300 px-3 py-2 text-sm"
              bind:value={profile.unitsCount}
            />
          </div>

          <div class="flex items-center gap-2">
            <input
              id="onboarding-shared-spaces"
              type="checkbox"
              data-testid="onboarding-shared-spaces-checkbox"
              class="min-h-[20px] min-w-[20px]"
              bind:checked={profile.hasSharedSpaces}
            />
            <label for="onboarding-shared-spaces" class="text-sm text-gray-800">
              {$_("onboarding.hasSharedSpacesLabel") ||
                "Dispose d'espaces communs partagés (jardin, buanderie, local vélos…)"}
            </label>
          </div>

          {#if profileError}
            <div
              id="onboarding-profile-error"
              class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800"
              data-testid="onboarding-profile-error"
              role="alert"
            >
              {profileError}
            </div>
          {/if}

          <div class="flex justify-end">
            <button
              type="button"
              class="min-h-[44px] rounded-lg bg-sky-600 px-5 py-2 text-white font-medium disabled:bg-gray-300 disabled:cursor-not-allowed hover:bg-sky-700"
              data-testid="onboarding-next"
              disabled={creatingAcp}
              onclick={() => void goToRecommendation()}
            >
              {creatingAcp
                ? $_("onboarding.creating") || "Création…"
                : $_("onboarding.next") || "Suivant"}
            </button>
          </div>
        </div>
      </section>
    {/if}

    {#if step === 2}
      <section data-testid="onboarding-step-2" aria-labelledby="onboarding-step-2-heading">
        <h1
          id="onboarding-step-2-heading"
          tabindex="-1"
          class="text-xl font-semibold text-gray-900 mb-2 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-600"
        >
          {$_("onboarding.step2Title") || "Modules recommandés"}
        </h1>
        <p class="text-sm text-gray-600 mb-4">
          {$_("onboarding.recommendedIntro") ||
            "Sur base du profil renseigné, ces modules sont recommandés :"}
        </p>

        <ul class="space-y-2 mb-4">
          {#each ALL_MODULES as module (module)}
            <li class="flex items-center gap-3 rounded-lg border border-gray-200 px-3 py-2">
              <input
                id={`onboarding-module-${module}`}
                type="checkbox"
                data-testid={`onboarding-module-toggle-${module}`}
                class="min-h-[20px] min-w-[20px]"
                checked={selectedModules.includes(module)}
                disabled={module === "identity"}
                onchange={() => toggleModule(module)}
              />
              <label for={`onboarding-module-${module}`} class="flex-1 text-sm text-gray-800">
                {$_(`onboarding.module.${module}`) || module}
              </label>
              {#if recommendedModules.includes(module)}
                <span class="text-xs font-medium text-emerald-700">
                  {$_("onboarding.recommendedBadge") || "recommandé"}
                </span>
              {/if}
            </li>
          {/each}
        </ul>

        <div class="flex justify-between">
          <button
            type="button"
            class="min-h-[44px] rounded-lg border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
            data-testid="onboarding-back"
            onclick={goBack}
          >
            {$_("onboarding.back") || "Précédent"}
          </button>
          <div class="flex gap-2">
            <button
              type="button"
              class="min-h-[44px] rounded-lg border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
              data-testid="onboarding-skip-recommendation"
              onclick={skipRecommendation}
            >
              {$_("onboarding.skipRecommendation") || "Passer cette étape"}
            </button>
            <button
              type="button"
              class="min-h-[44px] rounded-lg bg-sky-600 px-5 py-2 text-white font-medium hover:bg-sky-700"
              data-testid="onboarding-next"
              onclick={confirmRecommendation}
            >
              {$_("onboarding.next") || "Suivant"}
            </button>
          </div>
        </div>
      </section>
    {/if}

    {#if step === 3}
      <section data-testid="onboarding-step-3" aria-labelledby="onboarding-step-3-heading">
        <h1
          id="onboarding-step-3-heading"
          tabindex="-1"
          class="text-xl font-semibold text-gray-900 mb-2 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-600"
        >
          {$_("onboarding.step3Title") || "Activation"}
        </h1>
        <ul class="mb-4 list-disc list-inside text-sm text-gray-800">
          {#each selectedModules as module (module)}
            <li>{$_(`onboarding.module.${module}`) || module}</li>
          {/each}
        </ul>

        {#if activationError}
          <div
            class="mb-4 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800"
            data-testid="onboarding-activation-error"
            role="alert"
          >
            {activationError}
          </div>
        {/if}

        <div class="flex justify-between">
          <button
            type="button"
            class="min-h-[44px] rounded-lg border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
            data-testid="onboarding-back"
            onclick={goBack}
          >
            {$_("onboarding.back") || "Précédent"}
          </button>
          <button
            type="button"
            class="min-h-[44px] rounded-lg bg-sky-600 px-5 py-2 text-white font-medium disabled:bg-gray-300 disabled:cursor-not-allowed hover:bg-sky-700"
            data-testid="onboarding-activate-submit"
            disabled={activating}
            onclick={() => void activateModules()}
          >
            {activating
              ? $_("onboarding.activating") || "Activation en cours…"
              : $_("onboarding.activateSubmit") || "Activer les modules sélectionnés"}
          </button>
        </div>
      </section>
    {/if}

    {#if step === 4}
      <section data-testid="onboarding-step-4" aria-labelledby="onboarding-step-4-heading">
        <h1
          id="onboarding-step-4-heading"
          tabindex="-1"
          class="text-xl font-semibold text-gray-900 mb-2 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-600"
        >
          {$_("onboarding.step4Title") || "Démonstration"}
        </h1>
        <p class="text-sm text-gray-600 mb-4">
          {$_("onboarding.demoIntro") || "Aperçu rapide des modules activés."}
        </p>
        <ul class="mb-4 list-disc list-inside text-sm text-gray-800">
          {#each selectedModules as module (module)}
            <li>{$_(`onboarding.module.${module}`) || module}</li>
          {/each}
        </ul>
        <div class="flex justify-end">
          <button
            type="button"
            class="min-h-[44px] rounded-lg bg-sky-600 px-5 py-2 text-white font-medium hover:bg-sky-700"
            data-testid="onboarding-demo-next"
            onclick={finishDemo}
          >
            {$_("onboarding.demoNext") || "Continuer"}
          </button>
        </div>
      </section>
    {/if}

    {#if step === 5}
      <section data-testid="onboarding-step-5" aria-labelledby="onboarding-step-5-heading">
        <h1
          id="onboarding-step-5-heading"
          tabindex="-1"
          class="text-xl font-semibold text-gray-900 mb-2 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-600"
        >
          {$_("onboarding.confirmationTitle") || "Mise en service terminée"}
        </h1>

        <p class="text-sm text-gray-800 mb-1" data-testid="onboarding-elapsed-time">
          {$_("onboarding.elapsedTime") || "Temps écoulé :"}
          {formatElapsed(elapsedMsAtConfirmation)}
        </p>
        <p
          class={`text-sm font-medium mb-4 ${underTarget ? "text-emerald-700" : "text-amber-700"}`}
          data-testid="onboarding-kpi-status"
        >
          {underTarget
            ? $_("onboarding.kpiUnderTarget") || "Objectif atteint (moins de 5 minutes)"
            : $_("onboarding.kpiOverTarget") || "Au-delà de l'objectif de 5 minutes"}
        </p>

        <div class="flex justify-end">
          <button
            type="button"
            class="min-h-[44px] rounded-lg bg-emerald-600 px-5 py-2 text-white font-medium hover:bg-emerald-700"
            data-testid="onboarding-finish-submit"
            onclick={() => void finish()}
          >
            {$_("onboarding.finishSubmit") || "Terminer"}
          </button>
        </div>
      </section>
    {/if}
  {/if}
</div>
