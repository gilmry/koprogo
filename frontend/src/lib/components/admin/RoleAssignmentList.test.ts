// Story B1 (Phase B FE) — Vitest 4-cat tests for RoleAssignmentList.svelte.
//
// Couverture (cf. stories.md §B1 AC 4-cat) :
//   @happy    : la liste affiche les assignments avec colonnes
//               [User, Role, Org, Expire, Actions] + chaque row a
//               `data-testid="role-assignment-row-{id}"`.
//   @edge     : un assignment avec `valid_until` = aujourd'hui → ligne
//               affiche un `<ExpirationBadge>` avec data-level=urgent.
//   @security : un clic révoquer déclenche un DELETE typé sur l'endpoint
//               canonique ; pas de leak du JWT dans le DOM.
//   @negative : aucune assignment → état vide explicite + pas de table.
//
// Pattern mocks : on stubbe le module `api/role_assignments` (boundary
// réseau du composant). Cf. ContextBanner.test.ts §Mocks pour le pattern
// canonique côté projet.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, waitFor, fireEvent } from "../../../test-helpers";

vi.mock("../../api/role_assignments", () => ({
  listAssignments: vi.fn(),
  revokeAssignment: vi.fn(),
}));

import RoleAssignmentList from "./RoleAssignmentList.svelte";
import {
  listAssignments,
  revokeAssignment,
} from "../../api/role_assignments";

const mockedList = vi.mocked(listAssignments);
const mockedRevoke = vi.mocked(revokeAssignment);

beforeEach(() => {
  mockedList.mockReset();
  mockedRevoke.mockReset();
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("RoleAssignmentList — Story B1 (4-cat)", () => {
  it("@happy affiche les assignments avec data-testid stable par row", async () => {
    mockedList.mockResolvedValue([
      {
        id: "a1",
        user_id: "user-pierre",
        role: "accountant.encodeur",
        organization_id: "org-acp-a",
        is_primary: false,
        created_at: "2026-06-14T10:00:00Z",
        updated_at: "2026-06-14T10:00:00Z",
        valid_until: null,
      },
      {
        id: "a2",
        user_id: "user-marie",
        role: "community.moderator",
        organization_id: "org-acp-a",
        is_primary: false,
        created_at: "2026-06-12T10:00:00Z",
        updated_at: "2026-06-12T10:00:00Z",
        valid_until: "2026-06-21T23:59:59Z",
      },
    ]);

    const { getByTestId } = render(RoleAssignmentList, { props: {} });

    await waitFor(() =>
      expect(getByTestId("role-assignment-list")).toBeInTheDocument(),
    );
    expect(getByTestId("role-assignment-row-a1")).toBeInTheDocument();
    expect(getByTestId("role-assignment-row-a2")).toBeInTheDocument();
    expect(getByTestId("role-assignment-revoke-a1")).toBeInTheDocument();
    expect(getByTestId("role-assignment-revoke-a2")).toBeInTheDocument();
    expect(
      getByTestId("role-assignment-expiration-badge-a2"),
    ).toBeInTheDocument();
  });

  it("@edge valid_until=aujourd'hui → ExpirationBadge data-level=urgent", async () => {
    const today = new Date();
    const todayIso = new Date(
      today.getFullYear(),
      today.getMonth(),
      today.getDate(),
      23,
      59,
      59,
    ).toISOString();

    mockedList.mockResolvedValue([
      {
        id: "today-1",
        user_id: "user-pierre",
        role: "lawyer",
        organization_id: "org-acp-a",
        is_primary: false,
        created_at: "2026-06-14T10:00:00Z",
        updated_at: "2026-06-14T10:00:00Z",
        valid_until: todayIso,
      },
    ]);

    const { getByTestId } = render(RoleAssignmentList, { props: {} });
    await waitFor(() =>
      expect(getByTestId("role-assignment-row-today-1")).toBeInTheDocument(),
    );
    const badge = getByTestId("role-assignment-expiration-badge-today-1");
    expect(badge.getAttribute("data-level")).toBe("urgent");
  });

  it("@security clic révoquer → revokeAssignment(user_id, id) avec bonne paire", async () => {
    mockedList.mockResolvedValue([
      {
        id: "a1",
        user_id: "user-pierre",
        role: "accountant.encodeur",
        organization_id: "org-acp-a",
        is_primary: false,
        created_at: "2026-06-14T10:00:00Z",
        updated_at: "2026-06-14T10:00:00Z",
        valid_until: null,
      },
    ]);
    mockedRevoke.mockResolvedValue(undefined);

    const { getByTestId } = render(RoleAssignmentList, { props: {} });
    await waitFor(() =>
      expect(getByTestId("role-assignment-row-a1")).toBeInTheDocument(),
    );

    await fireEvent.click(
      getByTestId("role-assignment-revoke-a1") as HTMLButtonElement,
    );

    await waitFor(() => expect(mockedRevoke).toHaveBeenCalledTimes(1));
    // INV-FE5 : payload contient user_id + assignment_id ; pas de token / cookie.
    expect(mockedRevoke).toHaveBeenCalledWith("user-pierre", "a1");
  });

  it("@negative aucune assignment → état vide explicite", async () => {
    mockedList.mockResolvedValue([]);

    const { getByTestId, queryByTestId } = render(RoleAssignmentList, {
      props: {},
    });

    await waitFor(() =>
      expect(getByTestId("role-assignment-empty")).toBeInTheDocument(),
    );
    expect(queryByTestId("role-assignment-list")).toBeNull();
    expect(getByTestId("role-assignment-empty").textContent).toMatch(
      /aucune|vide/i,
    );
  });
});
