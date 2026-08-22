// Story B1 (Phase B FE) — Vitest 4-cat tests for RoleAssignmentForm.svelte.
//
// CRITICAL §3 — RED-first TDD: ces tests sont écrits AVANT le composant.
//
// Couverture (cf. stories.md §B1 AC 4-cat) :
//   @happy    : admin remplit user_id + role + org → submit → createRoleAssignment
//               appelé avec bon payload + callback `onsuccess` invoqué.
//   @edge     : valid_until = aujourd'hui → ISO 8601 dans payload.
//   @security : backend rejette (403 simulé) → message inline d'erreur,
//               modal reste ouvert, aucune fuite technique.
//   @negative : role custom injecté via DevTools → validation FE bloque
//               la soumission AVANT l'appel `createRoleAssignment`.
//
// Pattern mocks : on stubbe le module `api/role_assignments` (boundary
// réseau du composant). Cf. ContextBanner.test.ts §Mocks pour le pattern.
// Le `api.get` (utilisé pour charger users/orgs) est aussi stubbé via le
// module `api/organizations` (déjà mocké par le projet ailleurs) et un
// fetch direct si nécessaire — ici on mocke api.get sur le module `api`.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, waitFor, fireEvent } from "../../../test-helpers";

vi.mock("../../api/role_assignments", async () => {
  const actual = await vi.importActual<
    typeof import("../../api/role_assignments")
  >("../../api/role_assignments");
  return {
    ...actual,
    createRoleAssignment: vi.fn(),
  };
});

vi.mock("../../api", () => ({
  api: {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
  },
}));

import RoleAssignmentForm from "./RoleAssignmentForm.svelte";
import { createRoleAssignment } from "../../api/role_assignments";
import { api } from "../../api";

const mockedCreate = vi.mocked(createRoleAssignment);
const mockedApiGet = vi.mocked(api.get);

beforeEach(() => {
  mockedCreate.mockReset();
  mockedApiGet.mockReset();

  mockedApiGet.mockImplementation(async (endpoint: string) => {
    if (endpoint.startsWith("/users")) {
      return {
        data: [
          {
            id: "user-pierre",
            email: "pierre.dupont@example.com",
            first_name: "Pierre",
            last_name: "Dupont",
            role: "owner",
          },
          {
            id: "user-marie",
            email: "marie.martin@example.com",
            first_name: "Marie",
            last_name: "Martin",
            role: "owner",
          },
        ],
      };
    }
    if (endpoint.startsWith("/organizations")) {
      return {
        data: [
          { id: "org-acp-a", name: "ACP A", slug: "acp-a" },
          { id: "org-acp-b", name: "ACP B", slug: "acp-b" },
        ],
      };
    }
    return { data: [] };
  });
});

afterEach(() => {
  vi.restoreAllMocks();
});

function defaultProps(overrides: Partial<Record<string, unknown>> = {}) {
  return {
    isOpen: true,
    onclose: vi.fn(),
    onsuccess: vi.fn(),
    ...overrides,
  };
}

describe("RoleAssignmentForm — Story B1 (4-cat)", () => {
  it("@happy admin remplit user/role/org → submit → onsuccess called", async () => {
    mockedCreate.mockResolvedValue({
      id: "assignment-1",
      user_id: "user-pierre",
      role: "accountant.encodeur",
      organization_id: "org-acp-a",
      is_primary: false,
      created_at: "2026-06-14T12:00:00Z",
      updated_at: "2026-06-14T12:00:00Z",
    });

    const props = defaultProps();
    const { getByTestId } = render(RoleAssignmentForm, { props });

    await waitFor(() =>
      expect(getByTestId("role-assignment-user-select")).toBeInTheDocument(),
    );

    // Attendre la fin du chargement des options.
    await waitFor(() => {
      const opt = getByTestId(
        "role-assignment-user-option-user-pierre",
      ) as HTMLOptionElement | null;
      expect(opt).not.toBeNull();
    });

    await fireEvent.change(
      getByTestId("role-assignment-user-select") as HTMLSelectElement,
      { target: { value: "user-pierre" } },
    );
    await fireEvent.change(
      getByTestId("role-assignment-role-select") as HTMLSelectElement,
      { target: { value: "accountant.encodeur" } },
    );
    await fireEvent.change(
      getByTestId("role-assignment-org-select") as HTMLSelectElement,
      { target: { value: "org-acp-a" } },
    );

    const submit = getByTestId("role-assignment-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    await fireEvent.click(submit);

    await waitFor(() => expect(mockedCreate).toHaveBeenCalledTimes(1));
    const [userId, payload] = mockedCreate.mock.calls[0];
    expect(userId).toBe("user-pierre");
    expect(payload).toMatchObject({
      role: "accountant.encodeur",
      organization_id: "org-acp-a",
    });
    expect(props.onsuccess).toHaveBeenCalledTimes(1);
  });

  it("@edge valid_until=aujourd'hui → submit OK, ISO date dans payload", async () => {
    mockedCreate.mockResolvedValue({
      id: "assignment-2",
      user_id: "user-pierre",
      role: "community.moderator",
      organization_id: "org-acp-a",
      is_primary: false,
      created_at: "2026-06-14T12:00:00Z",
      updated_at: "2026-06-14T12:00:00Z",
      valid_until: "2026-06-14T23:59:59Z",
    });

    const props = defaultProps();
    const { getByTestId } = render(RoleAssignmentForm, { props });

    await waitFor(() =>
      expect(getByTestId("role-assignment-user-select")).toBeInTheDocument(),
    );
    await waitFor(() => {
      const opt = getByTestId(
        "role-assignment-user-option-user-pierre",
      ) as HTMLOptionElement | null;
      expect(opt).not.toBeNull();
    });

    await fireEvent.change(
      getByTestId("role-assignment-user-select") as HTMLSelectElement,
      { target: { value: "user-pierre" } },
    );
    await fireEvent.change(
      getByTestId("role-assignment-role-select") as HTMLSelectElement,
      { target: { value: "community.moderator" } },
    );
    await fireEvent.change(
      getByTestId("role-assignment-org-select") as HTMLSelectElement,
      { target: { value: "org-acp-a" } },
    );

    const today = new Date().toISOString().slice(0, 10);
    const validUntilInput = getByTestId(
      "role-assignment-valid-until-input",
    ) as HTMLInputElement;
    await fireEvent.input(validUntilInput, { target: { value: today } });

    await fireEvent.click(
      getByTestId("role-assignment-submit") as HTMLButtonElement,
    );

    await waitFor(() => expect(mockedCreate).toHaveBeenCalledTimes(1));
    const [, payload] = mockedCreate.mock.calls[0];
    const p = payload as { valid_until?: string };
    expect(p.valid_until).toBeTruthy();
    expect(p.valid_until).toMatch(/^\d{4}-\d{2}-\d{2}T/);
  });

  it("@security backend 403 → message inline typé, modal reste ouverte", async () => {
    mockedCreate.mockRejectedValue(
      new Error("Accès refusé. Vous n'êtes pas autorisé."),
    );

    const props = defaultProps();
    const { getByTestId, queryByTestId } = render(RoleAssignmentForm, {
      props,
    });

    await waitFor(() =>
      expect(getByTestId("role-assignment-user-select")).toBeInTheDocument(),
    );
    await waitFor(() => {
      const opt = getByTestId(
        "role-assignment-user-option-user-pierre",
      ) as HTMLOptionElement | null;
      expect(opt).not.toBeNull();
    });

    await fireEvent.change(
      getByTestId("role-assignment-user-select") as HTMLSelectElement,
      { target: { value: "user-pierre" } },
    );
    await fireEvent.change(
      getByTestId("role-assignment-role-select") as HTMLSelectElement,
      { target: { value: "accountant.encodeur" } },
    );
    await fireEvent.change(
      getByTestId("role-assignment-org-select") as HTMLSelectElement,
      { target: { value: "org-acp-b" } },
    );

    await fireEvent.click(
      getByTestId("role-assignment-submit") as HTMLButtonElement,
    );

    await waitFor(() =>
      expect(getByTestId("role-assignment-error-submit")).toBeInTheDocument(),
    );
    expect(props.onsuccess).not.toHaveBeenCalled();
    expect(props.onclose).not.toHaveBeenCalled();
    const errMsg =
      getByTestId("role-assignment-error-submit").textContent ?? "";
    expect(errMsg.toLowerCase()).not.toMatch(/sqlx|postgres|fkey|constraint/);
    expect(queryByTestId("role-assignment-submit")).not.toBeNull();
  });

  it("@negative role invalide via DevTools → validation FE bloque submit", async () => {
    const props = defaultProps();
    const { getByTestId, queryByTestId } = render(RoleAssignmentForm, {
      props,
    });

    await waitFor(() =>
      expect(getByTestId("role-assignment-user-select")).toBeInTheDocument(),
    );
    await waitFor(() => {
      const opt = getByTestId(
        "role-assignment-user-option-user-pierre",
      ) as HTMLOptionElement | null;
      expect(opt).not.toBeNull();
    });

    await fireEvent.change(
      getByTestId("role-assignment-user-select") as HTMLSelectElement,
      { target: { value: "user-pierre" } },
    );

    // Injection DevTools — option custom non-whitelist.
    const roleSelect = getByTestId(
      "role-assignment-role-select",
    ) as HTMLSelectElement;
    const opt = document.createElement("option");
    opt.value = "hacker.role";
    opt.text = "hacker.role";
    roleSelect.appendChild(opt);
    await fireEvent.change(roleSelect, { target: { value: "hacker.role" } });

    await fireEvent.change(
      getByTestId("role-assignment-org-select") as HTMLSelectElement,
      { target: { value: "org-acp-a" } },
    );

    await fireEvent.click(
      getByTestId("role-assignment-submit") as HTMLButtonElement,
    );

    await waitFor(() =>
      expect(getByTestId("role-assignment-error-role")).toBeInTheDocument(),
    );
    expect(mockedCreate).not.toHaveBeenCalled();
    expect(props.onsuccess).not.toHaveBeenCalled();
    expect(queryByTestId("role-assignment-error-role")?.textContent).toMatch(
      /inconnu|invalide/i,
    );
  });
});
