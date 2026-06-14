// Story B4 (Phase B FE) — Vitest 4-cat RoleDelegationForm.
//
// Couverture (cf. stories.md §B4 AC) :
//   @happy    : sélection complète target + role + valid_until valide →
//               submit appelle delegateRole avec le bon payload + onSuccess
//               invoqué + toast success.
//   @edge     : valid_until = today + 90j exactement → OK ; +1 jour → submit
//               disabled + erreur inline INV-8 ("90 jours").
//   @security : rôle custom injecté via DevTools (sélecteur forcé hors
//               whitelist DELEGABLE_ROLES) → submit disabled + erreur
//               inline ("Rôle inconnu ou non délégable"). Validation FE
//               bloque AVANT toute requête réseau (économie tokens + UX).
//   @negative : valid_until < now → submit disabled + helper "strictement
//               future".
//
// nowOverride : injection déterministe — vital pour les calculs max/min
// validUntil (l'AC @edge "+90j exactement" dépend de la date courante).
//
// Mocks : on intercepte `delegateRole` du module API pour éviter un fetch
// réseau réel. Le test vérifie le PAYLOAD (target/role/org/valid_until ISO
// 8601) et que `onSuccess` est rappelé avec le résultat mocké.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, waitFor, fireEvent } from "../../../test-helpers";
import RoleDelegationForm from "./RoleDelegationForm.svelte";

// -----------------------------------------------------------------------------
// Mocks API + toast
// -----------------------------------------------------------------------------

vi.mock("../../api/role_delegations", async () => {
  // On re-importe le module réel pour garder `DELEGABLE_ROLES` (utilisé pour
  // l'AC @security : on force un rôle HORS de cette liste).
  const actual =
    await vi.importActual<typeof import("../../api/role_delegations")>(
      "../../api/role_delegations",
    );
  return {
    ...actual,
    delegateRole: vi.fn(),
  };
});

vi.mock("../../../stores/toast", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}));

// i18n : on neutralise $_ (retourne chaîne vide) pour activer le fallback
// FR `|| "..."` du composant. Si on retournait la clé, elle serait truthy
// et le fallback ne se déclencherait pas.
vi.mock("../../i18n", () => {
  const store = {
    subscribe: (fn: (v: (k: string) => string) => void) => {
      fn(() => "");
      return () => {};
    },
  };
  return { _: store };
});

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

// Snapshot fixée pour déterminisme (mardi 10 juin 2026 12:00 UTC — alignée
// avec ExpirationBadge.test.ts et MandateIssueForm.test.ts).
const NOW_FIXED = new Date("2026-06-10T12:00:00Z");

const TARGETS = [
  { id: "user-pierre-1", label: "Pierre Dupont (board member)" },
  { id: "user-marie-2", label: "Marie Martin (owner)" },
];

const ORGANIZATIONS = [
  { id: "org-cabinet-1", label: "Cabinet Syndic A" },
  { id: "org-cabinet-2", label: "Cabinet Syndic B" },
];

/** Calcule un YYYY-MM-DD à N jours du now fixé. */
function plusDaysISODate(n: number): string {
  const d = new Date(NOW_FIXED.getTime() + n * 24 * 60 * 60 * 1000);
  return d.toISOString().slice(0, 10);
}

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  vi.restoreAllMocks();
});

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

describe("RoleDelegationForm — Story B4 (4-cat)", () => {
  it("@happy submit appelle delegateRole avec payload typé + onSuccess", async () => {
    const { delegateRole } = await import("../../api/role_delegations");
    (delegateRole as unknown as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      id: "delegation-new-1",
      user_id: "user-pierre-1",
      delegated_from_user_id: "issuer-uuid",
      role: "syndic",
      organization_id: "org-cabinet-1",
      valid_until: `${plusDaysISODate(7)}T23:59:59Z`,
      created_at: NOW_FIXED.toISOString(),
      updated_at: NOW_FIXED.toISOString(),
    });

    const onSuccess = vi.fn();
    const { getByTestId } = render(RoleDelegationForm, {
      props: {
        targets: TARGETS,
        organizations: ORGANIZATIONS,
        onSuccess,
        nowOverride: NOW_FIXED,
      },
    });

    // Target
    const targetSel = getByTestId(
      "role-delegate-target-input",
    ) as HTMLSelectElement;
    targetSel.value = "user-pierre-1";
    await fireEvent.change(targetSel);

    // Role (default = "syndic" — premier de DELEGABLE_ROLES)
    expect(getByTestId("role-delegate-role-option-syndic")).toBeTruthy();

    // Organization
    const orgSel = getByTestId("role-delegate-org-select") as HTMLSelectElement;
    orgSel.value = "org-cabinet-1";
    await fireEvent.change(orgSel);

    // valid_until = today + 7j
    const untilInp = getByTestId(
      "role-delegate-until-input",
    ) as HTMLInputElement;
    untilInp.value = plusDaysISODate(7);
    await fireEvent.input(untilInp);

    const submit = getByTestId("role-delegate-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));

    submit.click();

    await waitFor(() => expect(delegateRole).toHaveBeenCalledTimes(1));
    const [calledReq] = (delegateRole as unknown as ReturnType<typeof vi.fn>)
      .mock.calls[0];
    expect(calledReq).toMatchObject({
      target_user_id: "user-pierre-1",
      role: "syndic",
      organization_id: "org-cabinet-1",
      valid_until: `${plusDaysISODate(7)}T23:59:59Z`,
    });
    await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));
  });

  it("@edge valid_until = today+90j exact → OK ; +1 jour → submit disabled + erreur INV-8", async () => {
    const { getByTestId, queryByTestId } = render(RoleDelegationForm, {
      props: {
        targets: TARGETS,
        organizations: ORGANIZATIONS,
        nowOverride: NOW_FIXED,
      },
    });

    // Complete form sauf valid_until
    const targetSel = getByTestId(
      "role-delegate-target-input",
    ) as HTMLSelectElement;
    targetSel.value = "user-pierre-1";
    await fireEvent.change(targetSel);

    // Cas borne haute exacte : today + 90 jours = OK
    const untilInp = getByTestId(
      "role-delegate-until-input",
    ) as HTMLInputElement;
    untilInp.value = plusDaysISODate(90);
    await fireEvent.input(untilInp);

    const submit = getByTestId("role-delegate-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    expect(queryByTestId("role-delegate-error-until")).toBeNull();

    // Cas hors borne : today + 91 jours → submit disabled + erreur inline
    untilInp.value = plusDaysISODate(91);
    await fireEvent.input(untilInp);

    await waitFor(() => expect(submit.disabled).toBe(true));
    const err = getByTestId("role-delegate-error-until");
    expect(err.textContent).toMatch(/90 jours|INV-8/i);
  });

  it("@security rôle custom hors whitelist (DevTools bypass) → submit disabled + erreur inline", async () => {
    const { getByTestId } = render(RoleDelegationForm, {
      props: {
        targets: TARGETS,
        organizations: ORGANIZATIONS,
        nowOverride: NOW_FIXED,
      },
    });

    // Remplir le reste pour isoler l'erreur role
    const targetSel = getByTestId(
      "role-delegate-target-input",
    ) as HTMLSelectElement;
    targetSel.value = "user-pierre-1";
    await fireEvent.change(targetSel);

    const untilInp = getByTestId(
      "role-delegate-until-input",
    ) as HTMLInputElement;
    untilInp.value = plusDaysISODate(7);
    await fireEvent.input(untilInp);

    // Simule un DevTools-bypass : on injecte une valeur custom HORS whitelist
    // dans le select role. Le composant doit refuser AVANT POST.
    const roleSel = getByTestId(
      "role-delegate-role-select",
    ) as HTMLSelectElement;
    // Ajoute une option custom au DOM puis la sélectionne (ce que ferait un
    // attaquant via DevTools console).
    const customOption = document.createElement("option");
    customOption.value = "superadmin"; // explicitement hors DELEGABLE_ROLES
    customOption.text = "superadmin (injected)";
    roleSel.appendChild(customOption);
    roleSel.value = "superadmin";
    await fireEvent.change(roleSel);

    const submit = getByTestId("role-delegate-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(true));
    const err = getByTestId("role-delegate-error-role");
    expect(err.textContent).toMatch(/inconnu|non délégable|superadmin/i);
  });

  it("@negative valid_until < now → submit disabled + helper 'strictement future'", async () => {
    const { getByTestId } = render(RoleDelegationForm, {
      props: {
        targets: TARGETS,
        organizations: ORGANIZATIONS,
        nowOverride: NOW_FIXED,
      },
    });

    // Setup target + role déjà OK (rôle default = syndic)
    const targetSel = getByTestId(
      "role-delegate-target-input",
    ) as HTMLSelectElement;
    targetSel.value = "user-pierre-1";
    await fireEvent.change(targetSel);

    // valid_until = hier (passé)
    const untilInp = getByTestId(
      "role-delegate-until-input",
    ) as HTMLInputElement;
    untilInp.value = plusDaysISODate(-1);
    await fireEvent.input(untilInp);

    const submit = getByTestId("role-delegate-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(true));
    const err = getByTestId("role-delegate-error-until");
    expect(err.textContent).toMatch(/strictement future|future/i);
  });
});
