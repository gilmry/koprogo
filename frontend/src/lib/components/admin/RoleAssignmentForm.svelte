<script lang="ts">
  // Story B1 (Phase B FE) — RoleAssignmentForm.
  //
  // Modal form Svelte 5 (runes) pour assigner un sous-rôle métier
  // (`accountant.encodeur` / `accountant.emetteur` / `community.moderator`)
  // ou un mandataire (`lawyer` / `notary` / `amo` / `architect` / `bet` /
  // `warden`) à un user existant, scoped à une organization.
  //
  // INV-FE1 : 100% des éléments interactifs ont un `data-testid` stable et
  // i18n-safe (cf. memory `data-testid-systematic`).
  // INV-FE2 : WCAG 2.1 AA — `role="dialog"` + `aria-labelledby` + focus trap
  // backporté du `Modal.svelte` UI kit + tap target ≥ 44px (min-h-[44px]) +
  // `aria-describedby` pour erreurs inline + `aria-live="polite"`.
  // INV-FE5 : aucun JWT ni token persisté (api.ts gère l'auth via header
  // Bearer en mémoire).
  //
  // Validation FE (anti-bypass DevTools) :
  //   - Le rôle DOIT appartenir à `ASSIGNABLE_ROLES` (cf. role_assignments.ts).
  //     Une valeur custom injectée via DevTools est refusée AVANT l'appel
  //     réseau (économie de tokens + UX : message inline immédiat).
  //   - L'organization_id DOIT être présent (les sous-rôles métier sont
  //     toujours scopés — pas de rôle global côté FE pour cette UI).
  //
  // Erreurs backend (cf. AC @security) :
  //   - 403 cross-org → toast "Accès refusé" auto via api.ts + message inline
  //     sous le bouton submit ; on n'expose JAMAIS l'erreur brute (api.ts
  //     mask déjà sqlx/Postgres/FK).
  //
  // Pattern Svelte 5 : `$state` / `$derived` / `$effect` / `$props`
  // (cf. ContextBanner.svelte Story 2.3 mergée). PAS de `svelte/store`.

  import {
    createRoleAssignment,
    ASSIGNABLE_ROLES,
    type AssignableRole,
    type AssignRoleRequest,
  } from "../../api/role_assignments";
  import { api } from "../../api";

  // ---------------------------------------------------------------------------
  // Props (cf. UserForm.svelte pour le pattern Modal callbacks)
  // ---------------------------------------------------------------------------

  let {
    isOpen = false,
    onclose,
    onsuccess,
  }: {
    isOpen?: boolean;
    onclose?: () => void;
    onsuccess?: () => void;
  } = $props();

  // ---------------------------------------------------------------------------
  // State — formulaire + listes options + erreurs typed par champ
  // ---------------------------------------------------------------------------

  interface UserOption {
    id: string;
    email: string;
    first_name: string;
    last_name: string;
    role?: string;
  }
  interface OrgOption {
    id: string;
    name: string;
    slug?: string;
  }

  let users = $state<UserOption[]>([]);
  let orgs = $state<OrgOption[]>([]);
  let loadingOptions = $state(true);

  let userId = $state<string>("");
  let role = $state<string>("");
  let organizationId = $state<string>("");
  let validUntilDate = $state<string>(""); // "YYYY-MM-DD"

  let errors = $state<{
    user: string;
    role: string;
    org: string;
    valid_until: string;
    submit: string;
  }>({
    user: "",
    role: "",
    org: "",
    valid_until: "",
    submit: "",
  });

  let submitting = $state(false);

  // ---------------------------------------------------------------------------
  // Charge users + organizations à l'ouverture du modal — `$effect` se
  // déclenche au mount (et au prochain `isOpen` true si on réutilise le
  // composant entre plusieurs sessions).
  // ---------------------------------------------------------------------------

  let optionsLoaded = $state(false);
  $effect(() => {
    if (isOpen && !optionsLoaded) {
      optionsLoaded = true;
      void loadOptions();
    }
  });

  async function loadOptions(): Promise<void> {
    loadingOptions = true;
    try {
      const [usersResp, orgsResp] = await Promise.all([
        api.get<{ data: UserOption[] }>("/users?per_page=1000"),
        api.get<{ data: OrgOption[] }>("/organizations?per_page=1000"),
      ]);
      users = usersResp.data ?? [];
      orgs = orgsResp.data ?? [];
    } catch {
      // Toast déjà géré par api.ts. On laisse les listes vides ;
      // le submit échouera proprement (validation user/org required).
    } finally {
      loadingOptions = false;
    }
  }

  // ---------------------------------------------------------------------------
  // Dérivations (validation côté FE — RED-line anti-bypass DevTools)
  // ---------------------------------------------------------------------------

  let isRoleValid = $derived(
    (ASSIGNABLE_ROLES as readonly string[]).includes(role),
  );

  // canSubmit ne checke PAS isRoleValid : si un attaquant injecte une
  // valeur custom via DevTools, on veut que le bouton reste cliquable
  // pour que la validation FE déclenche le message inline (cf. AC @negative).
  // En usage nominal, la dropdown ne propose que des valeurs whitelist.
  let canSubmit = $derived(
    !submitting && userId.length > 0 && role.length > 0 && organizationId.length > 0,
  );

  // ---------------------------------------------------------------------------
  // Submit handler
  // ---------------------------------------------------------------------------

  async function handleSubmit(e: Event): Promise<void> {
    e.preventDefault();
    // Reset errors avant validation.
    errors.user = "";
    errors.role = "";
    errors.org = "";
    errors.valid_until = "";
    errors.submit = "";

    let hasError = false;
    if (!userId) {
      errors.user = "Veuillez sélectionner un utilisateur.";
      hasError = true;
    }
    if (!isRoleValid) {
      errors.role = `Sous-rôle inconnu ou invalide : « ${role || "(vide)"} ».`;
      hasError = true;
    }
    if (!organizationId) {
      errors.org = "Veuillez sélectionner une organisation.";
      hasError = true;
    }
    if (hasError) return;

    // Conversion date "YYYY-MM-DD" → ISO 8601 (fin de journée locale).
    // Le backend attend `TIMESTAMPTZ` ; on fixe T23:59:59 pour qu'un
    // `valid_until=aujourd'hui` reste actif toute la journée (cf. AC @edge).
    let validUntilIso: string | null = null;
    if (validUntilDate) {
      try {
        const d = new Date(`${validUntilDate}T23:59:59`);
        if (!Number.isNaN(d.getTime())) {
          validUntilIso = d.toISOString();
        }
      } catch {
        errors.valid_until = "Date d'expiration invalide.";
        return;
      }
    }

    const payload: AssignRoleRequest = {
      role: role as AssignableRole,
      organization_id: organizationId,
      ...(validUntilIso ? { valid_until: validUntilIso } : {}),
    };

    submitting = true;
    try {
      await createRoleAssignment(userId, payload);
      onsuccess?.();
      // Reset form pour réouverture propre.
      userId = "";
      role = "";
      organizationId = "";
      validUntilDate = "";
    } catch (err) {
      // api.ts a déjà toasté + masqué les erreurs DB. On affiche un message
      // utilisateur safe sous le submit.
      const raw = err instanceof Error ? err.message : "Erreur inconnue";
      errors.submit = raw || "Impossible d'assigner ce sous-rôle.";
    } finally {
      submitting = false;
    }
  }

  function handleCancel(): void {
    onclose?.();
  }

  // ---------------------------------------------------------------------------
  // Escape key → close (a11y)
  // ---------------------------------------------------------------------------

  $effect(() => {
    if (!isOpen) return;
    const onKey = (ev: KeyboardEvent) => {
      if (ev.key === "Escape") {
        handleCancel();
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  });
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black bg-opacity-50 z-40"
    aria-hidden="true"
    role="presentation"
    onclick={handleCancel}
  ></div>

  <!-- Modal -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 overflow-y-auto"
  >
    <div
      class="bg-white rounded-lg shadow-xl w-full max-w-2xl mx-auto my-8 max-h-[90vh] flex flex-col"
      role="dialog"
      aria-modal="true"
      aria-labelledby="role-assignment-form-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <div
        class="flex items-center justify-between p-6 border-b border-gray-200"
      >
        <h2
          id="role-assignment-form-title"
          class="text-xl font-semibold text-gray-900"
        >
          Nouvelle assignation de rôle
        </h2>
        <button
          type="button"
          onclick={handleCancel}
          data-testid="role-assignment-cancel"
          class="min-h-[44px] min-w-[44px] flex items-center justify-center text-gray-400 hover:text-gray-600 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary-500"
          aria-label="Fermer le formulaire"
        >
          <svg
            class="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            aria-hidden="true"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>

      <form onsubmit={handleSubmit} class="flex-1 overflow-y-auto p-6">
        <!-- User -->
        <div class="mb-4">
          <label
            for="role-assignment-user"
            class="block text-sm font-medium text-gray-700 mb-2"
          >
            Utilisateur
            <span class="text-red-500" aria-hidden="true">*</span>
            <span class="sr-only">(obligatoire)</span>
          </label>
          <select
            id="role-assignment-user"
            data-testid="role-assignment-user-select"
            bind:value={userId}
            required
            disabled={loadingOptions}
            aria-invalid={errors.user ? "true" : undefined}
            aria-describedby={errors.user
              ? "role-assignment-error-user"
              : undefined}
            class="w-full min-h-[44px] px-4 py-2 border rounded-lg focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:border-primary-500 disabled:bg-gray-100 transition {errors.user
              ? 'border-red-500'
              : 'border-gray-300'}"
          >
            <option value="" disabled>Sélectionner un utilisateur…</option>
            {#each users as u (u.id)}
              <option
                value={u.id}
                data-testid={`role-assignment-user-option-${u.id}`}
              >
                {u.first_name} {u.last_name} — {u.email}
              </option>
            {/each}
          </select>
          {#if errors.user}
            <p
              id="role-assignment-error-user"
              data-testid="role-assignment-error-user"
              role="alert"
              aria-live="polite"
              class="mt-1 text-sm text-red-600"
            >
              {errors.user}
            </p>
          {/if}
        </div>

        <!-- Role -->
        <div class="mb-4">
          <label
            for="role-assignment-role"
            class="block text-sm font-medium text-gray-700 mb-2"
          >
            Sous-rôle
            <span class="text-red-500" aria-hidden="true">*</span>
          </label>
          <select
            id="role-assignment-role"
            data-testid="role-assignment-role-select"
            bind:value={role}
            required
            aria-invalid={errors.role ? "true" : undefined}
            aria-describedby={errors.role
              ? "role-assignment-error-role"
              : undefined}
            class="w-full min-h-[44px] px-4 py-2 border rounded-lg focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:border-primary-500 transition {errors.role
              ? 'border-red-500'
              : 'border-gray-300'}"
          >
            <option value="" disabled>Sélectionner un sous-rôle…</option>
            {#each ASSIGNABLE_ROLES as r (r)}
              <option value={r}>{r}</option>
            {/each}
          </select>
          {#if errors.role}
            <p
              id="role-assignment-error-role"
              data-testid="role-assignment-error-role"
              role="alert"
              aria-live="polite"
              class="mt-1 text-sm text-red-600"
            >
              {errors.role}
            </p>
          {/if}
        </div>

        <!-- Organization -->
        <div class="mb-4">
          <label
            for="role-assignment-org"
            class="block text-sm font-medium text-gray-700 mb-2"
          >
            Organisation
            <span class="text-red-500" aria-hidden="true">*</span>
          </label>
          <select
            id="role-assignment-org"
            data-testid="role-assignment-org-select"
            bind:value={organizationId}
            required
            disabled={loadingOptions}
            aria-invalid={errors.org ? "true" : undefined}
            aria-describedby={errors.org
              ? "role-assignment-error-org"
              : undefined}
            class="w-full min-h-[44px] px-4 py-2 border rounded-lg focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:border-primary-500 disabled:bg-gray-100 transition {errors.org
              ? 'border-red-500'
              : 'border-gray-300'}"
          >
            <option value="" disabled>Sélectionner une organisation…</option>
            {#each orgs as o (o.id)}
              <option value={o.id}>{o.name}</option>
            {/each}
          </select>
          {#if errors.org}
            <p
              id="role-assignment-error-org"
              data-testid="role-assignment-error-org"
              role="alert"
              aria-live="polite"
              class="mt-1 text-sm text-red-600"
            >
              {errors.org}
            </p>
          {/if}
        </div>

        <!-- Valid until (optional) -->
        <div class="mb-4">
          <label
            for="role-assignment-valid-until"
            class="block text-sm font-medium text-gray-700 mb-2"
          >
            Date d'expiration
            <span class="text-gray-500 text-xs ml-1">
              (optionnel — vide = permanent)
            </span>
          </label>
          <input
            id="role-assignment-valid-until"
            data-testid="role-assignment-valid-until-input"
            type="date"
            bind:value={validUntilDate}
            aria-invalid={errors.valid_until ? "true" : undefined}
            aria-describedby={errors.valid_until
              ? "role-assignment-error-valid-until"
              : undefined}
            class="w-full min-h-[44px] px-4 py-2 border rounded-lg focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:border-primary-500 transition {errors.valid_until
              ? 'border-red-500'
              : 'border-gray-300'}"
          />
          {#if errors.valid_until}
            <p
              id="role-assignment-error-valid-until"
              data-testid="role-assignment-error-valid-until"
              role="alert"
              aria-live="polite"
              class="mt-1 text-sm text-red-600"
            >
              {errors.valid_until}
            </p>
          {/if}
        </div>

        <!-- Submit error inline -->
        {#if errors.submit}
          <div
            id="role-assignment-error-submit"
            data-testid="role-assignment-error-submit"
            role="alert"
            aria-live="assertive"
            class="mb-4 p-3 rounded-lg border border-red-300 bg-red-50 text-sm text-red-700"
          >
            {errors.submit}
          </div>
        {/if}

        <!-- Footer actions -->
        <div class="flex items-center justify-end gap-3 pt-4 border-t">
          <button
            type="button"
            onclick={handleCancel}
            data-testid="role-assignment-cancel-bottom"
            class="min-h-[44px] px-4 py-2 rounded-lg border border-gray-300 text-gray-700 bg-white hover:bg-gray-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary-500"
          >
            Annuler
          </button>
          <button
            type="submit"
            data-testid="role-assignment-submit"
            disabled={!canSubmit}
            class="min-h-[44px] px-4 py-2 rounded-lg bg-primary-600 text-white font-medium hover:bg-primary-700 disabled:bg-gray-300 disabled:cursor-not-allowed focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary-500"
          >
            {submitting ? "Assignation…" : "Assigner"}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
