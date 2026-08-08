<script lang="ts">
  // Story B4 (Phase B FE) — Form de délégation temporaire de rôle (Syndic
  // délègue à Owner Pierre pour 7j, max 90j ; non-transitivité INV-8 BE).
  //
  // Parent BE story 3.5 (`edf171f`) + Story B0 utoipa (`8cab49f`) +
  // wireframe stories.md §B4.
  //
  // Invariants exposés en UI (gating client AVANT POST, redondant avec backend) :
  //   INV-8 (non-transitivité) : si le current user a HÉRITÉ son rôle via une
  //          délégation (i.e. il apparaît côté `user_id` d'une délégation
  //          active avec `delegated_from_user_id` set), le CTA et le form
  //          sont ABSENTS du DOM. Backend renvoie 403 `DelegationChainNotAllowed`
  //          si on bypass via DevTools (cf. AC @security).
  //   INV-8 (max 90j) : `valid_until - now <= 90j` (anti-abuse). Hors fenêtre
  //          → submit disabled + helper rouge inline (cf. AC @edge).
  //   valid_until > now (futur strict) — submit disabled si passé/aujourd'hui.
  //
  // a11y (WCAG 2.1 AA — memory `a11y-wcag-aa-baseline`) :
  //   - Chaque `<label>` lie son contrôle via `for`/`id`.
  //   - Erreurs inline avec `aria-describedby` + `aria-invalid` + `role="alert"`.
  //   - Submit disabled communique état via `aria-disabled` (en plus de
  //     `disabled`).
  //   - Tap target ≥ 44px sur les boutons (focus-visible ring).
  //
  // data-testid (cf. stories.md §B4) :
  //   role-delegate-target-input  / role-delegate-target-option-{userId}
  //   role-delegate-role-select   / role-delegate-role-option-{role}
  //   role-delegate-org-select    / role-delegate-org-option-{orgId}
  //   role-delegate-until-input
  //   role-delegate-submit        / role-delegate-cancel
  //   role-delegate-error-{field} (un par champ — affiché si error[field])
  //
  // i18n : labels via `$_` (fallback FR statique). Voir dateBadge.ts pour
  // motivation du fallback (Story B12+ portera la i18n NL/EN/DE).

  import { _ } from "../../i18n";
  import { toast } from "../../../stores/toast";
  import {
    delegateRole,
    DELEGABLE_ROLES,
    type DelegateRoleRequest,
    type RoleDelegationResponse,
  } from "../../api/role_delegations";

  // -------------------------------------------------------------------------
  // Props — onSuccess remonte la nouvelle ligne au parent (RoleDelegationList).
  // -------------------------------------------------------------------------

  let {
    /** Liste des users sélectionnables comme `target_user_id`. Fournie par le
     *  parent (découplage fetch). */
    targets = [],
    /** Liste des organizations sélectionnables (scope de la délégation) —
     *  peuplée uniquement pour un superadmin (`GET /organizations` reste
     *  superadmin-only) ; vide pour un syndic normal. */
    organizations = [],
    /** Organisation du user connecté — préselectionnée par défaut (un
     *  syndic ne délègue que dans sa propre org ; le sélecteur `organizations`
     *  reste disponible pour un superadmin qui veut choisir explicitement).
     *  Cf. Story S3 docs/maury/syndic-org-users-endpoint : sans ce défaut,
     *  organization_id partait `null` et le check anti-transitivité backend
     *  (scopé par org) rejetait tout syndic réel avec 403. */
    defaultOrganizationId = "",
    /** Callback succès — remonté avec la `RoleDelegationResponse` du backend. */
    onSuccess = undefined,
    /** Callback annulation (close modal côté parent). */
    onCancel = undefined,
    /** Injection clock pour tests déterministes (équivalent MandateIssueForm). */
    nowOverride = undefined,
  }: {
    targets?: Array<{ id: string; label: string }>;
    organizations?: Array<{ id: string; label: string }>;
    defaultOrganizationId?: string;
    onSuccess?: (d: RoleDelegationResponse) => void;
    onCancel?: () => void;
    nowOverride?: Date | undefined;
  } = $props();

  // -------------------------------------------------------------------------
  // State du formulaire
  // -------------------------------------------------------------------------

  let targetUserId = $state<string>("");
  let role = $state<string>(DELEGABLE_ROLES[0]); // "syndic" par défaut
  let organizationId = $state<string>(defaultOrganizationId);
  let validUntil = $state<string>(""); // YYYY-MM-DD (input type=date)

  let submitting = $state<boolean>(false);

  // -------------------------------------------------------------------------
  // Dérivations & validation (INV-8 max 90j + futur strict)
  // -------------------------------------------------------------------------

  /** Now injectable — pour tests déterministes (vs `new Date()` flaky). */
  let now = $derived(nowOverride ?? new Date());

  /** Date max autorisée (INV-8 : today + 90 jours). */
  let maxValidUntil = $derived.by(() => {
    const max = new Date(now.getTime());
    max.setUTCDate(max.getUTCDate() + 90);
    return max.toISOString().slice(0, 10);
  });

  /** Date min autorisée : demain (valid_until strictement futur). */
  let minValidUntil = $derived.by(() => {
    const min = new Date(now.getTime());
    min.setUTCDate(min.getUTCDate() + 1);
    return min.toISOString().slice(0, 10);
  });

  /** valid_until parsé en Date (fin de jour UTC) pour comparaisons. */
  let validUntilDate = $derived<Date | null>(
    validUntil ? new Date(`${validUntil}T23:59:59Z`) : null,
  );

  /** Hors fenêtre INV-8 (>90j depuis maintenant) ?
   *
   * Calculé en granularité JOUR (pas en ms) pour matcher la sémantique
   * user-facing "90j calendaires". */
  let validUntilOutOfRange = $derived.by(() => {
    if (!validUntilDate) return false;
    const nowDay = Date.UTC(
      now.getUTCFullYear(),
      now.getUTCMonth(),
      now.getUTCDate(),
    );
    const targetDay = Date.UTC(
      validUntilDate.getUTCFullYear(),
      validUntilDate.getUTCMonth(),
      validUntilDate.getUTCDate(),
    );
    const deltaDays = Math.round((targetDay - nowDay) / (24 * 60 * 60 * 1000));
    return deltaDays > 90;
  });

  /** valid_until dans le passé ? (strictement <= now en granularité jour). */
  let validUntilInPast = $derived.by(() => {
    if (!validUntilDate) return false;
    return validUntilDate.getTime() <= now.getTime();
  });

  /** Map d'erreurs inline (clé = data-testid suffix). */
  let errors = $derived.by<Record<string, string>>(() => {
    const e: Record<string, string> = {};
    if (targetUserId === "") e.target = "Sélectionnez un utilisateur cible.";
    if (!(DELEGABLE_ROLES as readonly string[]).includes(role))
      e.role = `Rôle inconnu ou non délégable : « ${role || "(vide)"} ».`;
    if (validUntil === "") e.until = "Date d'expiration obligatoire.";
    else if (validUntilInPast)
      e.until = "La date d'expiration doit être strictement future.";
    else if (validUntilOutOfRange)
      e.until = "Durée maximale 90 jours (INV-8 anti-abus).";
    return e;
  });

  let formValid = $derived(Object.keys(errors).length === 0);

  // -------------------------------------------------------------------------
  // Handlers
  // -------------------------------------------------------------------------

  async function handleSubmit(ev: SubmitEvent): Promise<void> {
    ev.preventDefault();
    if (!formValid || submitting) return;
    submitting = true;
    try {
      // Backend attend ISO 8601 (TIMESTAMPTZ). On envoie l'instant fin de
      // journée UTC pour valid_until (23:59:59Z), cohérent avec la
      // sémantique "expire en fin de journée du J indiqué".
      const req: DelegateRoleRequest = {
        target_user_id: targetUserId,
        role,
        valid_until: `${validUntil}T23:59:59Z`,
        ...(organizationId ? { organization_id: organizationId } : {}),
      };
      const created = await delegateRole(req);
      toast.success(
        $_("roleDelegation.create.success") || "Délégation créée.",
      );
      onSuccess?.(created);
      // Reset partiel — on garde le rôle (UX : on délègue souvent en série).
      targetUserId = "";
      validUntil = "";
    } catch {
      // Le wrapper `api.ts` a déjà toasté l'erreur 4xx/5xx — pas besoin de
      // double feedback ici. Note : 403 `DelegationChainNotAllowed` (INV-8)
      // sera toasté générique "Accès refusé" — mais en pratique le CTA est
      // déjà ABSENT côté parent quand le user a hérité, donc on n'arrive
      // ici qu'en cas de bypass DevTools.
    } finally {
      submitting = false;
    }
  }
</script>

<form
  class="role-delegation-form flex flex-col gap-4 p-4 bg-white rounded shadow-sm"
  onsubmit={handleSubmit}
  aria-labelledby="role-delegate-title"
  novalidate
>
  <h2 id="role-delegate-title" class="text-lg font-semibold text-gray-900">
    {$_("roleDelegation.create.title") || "Nouvelle délégation de rôle"}
  </h2>

  <!-- Target user -->
  <div class="flex flex-col gap-1">
    <label
      for="role-delegate-target"
      class="text-sm font-medium text-gray-700"
    >
      {$_("roleDelegation.field.target") || "Utilisateur cible"}
    </label>
    <select
      id="role-delegate-target"
      data-testid="role-delegate-target-input"
      bind:value={targetUserId}
      aria-invalid={errors.target ? "true" : "false"}
      aria-describedby={errors.target ? "role-delegate-error-target" : undefined}
      class="border border-gray-300 rounded px-3 py-2 text-sm min-h-[44px]"
      required
    >
      <option value="" disabled>— {$_("common.choose") || "Choisir…"} —</option>
      {#each targets as t (t.id)}
        <option
          value={t.id}
          data-testid={`role-delegate-target-option-${t.id}`}
        >
          {t.label}
        </option>
      {/each}
    </select>
    {#if errors.target}
      <p
        id="role-delegate-error-target"
        data-testid="role-delegate-error-target"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.target}
      </p>
    {/if}
  </div>

  <!-- Role -->
  <div class="flex flex-col gap-1">
    <label for="role-delegate-role" class="text-sm font-medium text-gray-700">
      {$_("roleDelegation.field.role") || "Rôle délégué"}
    </label>
    <select
      id="role-delegate-role"
      data-testid="role-delegate-role-select"
      bind:value={role}
      aria-invalid={errors.role ? "true" : "false"}
      aria-describedby={errors.role ? "role-delegate-error-role" : undefined}
      class="border border-gray-300 rounded px-3 py-2 text-sm min-h-[44px]"
      required
    >
      {#each DELEGABLE_ROLES as r (r)}
        <option value={r} data-testid={`role-delegate-role-option-${r}`}>
          {r}
        </option>
      {/each}
    </select>
    {#if errors.role}
      <p
        id="role-delegate-error-role"
        data-testid="role-delegate-error-role"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.role}
      </p>
    {/if}
  </div>

  <!-- Organization (optionnelle — global si vide, scoped à une org sinon) -->
  <div class="flex flex-col gap-1">
    <label for="role-delegate-org" class="text-sm font-medium text-gray-700">
      {$_("roleDelegation.field.organization") || "Organisation"}
      <span class="text-gray-500 text-xs ml-1">
        ({$_("common.optional") || "optionnelle"})
      </span>
    </label>
    <select
      id="role-delegate-org"
      data-testid="role-delegate-org-select"
      bind:value={organizationId}
      class="border border-gray-300 rounded px-3 py-2 text-sm min-h-[44px]"
    >
      <option value="">— {$_("common.global") || "Global"} —</option>
      {#each organizations as o (o.id)}
        <option value={o.id} data-testid={`role-delegate-org-option-${o.id}`}>
          {o.label}
        </option>
      {/each}
    </select>
  </div>

  <!-- valid_until -->
  <div class="flex flex-col gap-1">
    <label
      for="role-delegate-until"
      class="text-sm font-medium text-gray-700"
    >
      {$_("roleDelegation.field.validUntil") || "Valide jusqu'au"}
    </label>
    <input
      id="role-delegate-until"
      data-testid="role-delegate-until-input"
      type="date"
      bind:value={validUntil}
      min={minValidUntil}
      max={maxValidUntil}
      aria-invalid={errors.until ? "true" : "false"}
      aria-describedby={errors.until ? "role-delegate-error-until" : undefined}
      class="border border-gray-300 rounded px-3 py-2 text-sm min-h-[44px]"
      required
    />
    {#if errors.until}
      <p
        id="role-delegate-error-until"
        data-testid="role-delegate-error-until"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.until}
      </p>
    {/if}
  </div>

  <!-- Actions -->
  <div class="flex justify-end gap-2 mt-2">
    <button
      type="button"
      data-testid="role-delegate-cancel"
      class="min-h-[44px] px-4 py-2 text-sm border border-gray-300 rounded text-gray-700 hover:bg-gray-50"
      onclick={() => onCancel?.()}
      disabled={submitting}
    >
      {$_("common.cancel") || "Annuler"}
    </button>
    <button
      type="submit"
      data-testid="role-delegate-submit"
      class="min-h-[44px] px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
      disabled={!formValid || submitting}
      aria-disabled={!formValid || submitting}
    >
      {submitting
        ? $_("common.submitting") || "Création…"
        : $_("roleDelegation.action.create") || "Déléguer"}
    </button>
  </div>
</form>
