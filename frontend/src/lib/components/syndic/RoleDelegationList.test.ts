// Story B4 (Phase B FE) — Vitest 4-cat RoleDelegationList.
//
// Couverture (cf. stories.md §B4 AC) :
//   @happy    : tableau affiche les rows avec target user + role + source +
//               ExpirationBadge — chaque row a son data-testid stable.
//   @edge     : badge passe au orange (soon ≤ 30j) puis rouge (urgent ≤ 7j).
//               Vérifie le data-level exposé par ExpirationBadge.
//   @security : user qui a HÉRITÉ son rôle (présent comme `user_id` d'une row
//               avec `delegated_from_user_id` set) → banner non-transitivité
//               affiché EN ROUGE + bouton "Nouvelle délégation" ABSENT du
//               DOM (pas juste disabled — cf. AC §B4).
//   @negative : liste vide → message empty + banner d'information toujours
//               présent (fallback conservateur).
//
// L'ExpirationBadge est laissé intact (composant atomique partagé, déjà
// testé dans son propre fichier ExpirationBadge.test.ts). On vérifie ici
// l'INTÉGRATION : la table passe bien validUntil + idSuffix et le badge
// rend les data-* attendus.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import RoleDelegationList from "./RoleDelegationList.svelte";
import type { RoleDelegationResponse } from "../../api/role_delegations";

// -----------------------------------------------------------------------------
// Mocks API
// -----------------------------------------------------------------------------

vi.mock("../../api/role_delegations", () => ({
  listDelegationsOf: vi.fn(),
  revokeDelegation: vi.fn(),
}));

vi.mock("../../../stores/toast", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}));

// $_ retourne "" pour forcer le fallback FR `|| "fallback"` du composant.
vi.mock("../../i18n", () => ({
  _: {
    subscribe: (fn: (v: (k: string) => string) => void) => {
      fn(() => "");
      return () => {};
    },
  },
}));

// -----------------------------------------------------------------------------
// Fixtures
// -----------------------------------------------------------------------------

const NOW_FIXED = new Date("2026-06-10T12:00:00Z");
const ISSUER_ID = "issuer-syndic-uuid";

function mkDelegation(opts: {
  id: string;
  daysUntilExpiry: number;
  userId?: string;
  role?: string;
  organizationId?: string | null;
  delegatedFromUserId?: string | null;
}): RoleDelegationResponse {
  const validUntil = new Date(
    NOW_FIXED.getTime() + opts.daysUntilExpiry * 24 * 60 * 60 * 1000,
  );
  return {
    id: opts.id,
    user_id: opts.userId ?? "user-pierre-1",
    role: opts.role ?? "syndic",
    organization_id: opts.organizationId ?? "org-cabinet-1",
    delegated_from_user_id: opts.delegatedFromUserId ?? ISSUER_ID,
    valid_until: validUntil.toISOString(),
    created_at: NOW_FIXED.toISOString(),
    updated_at: NOW_FIXED.toISOString(),
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

describe("RoleDelegationList — Story B4 (4-cat)", () => {
  it("@happy affiche les rows avec target + role + source + expiration badge", async () => {
    const delegations = [
      mkDelegation({ id: "d-1", daysUntilExpiry: 7 }),
      mkDelegation({
        id: "d-2",
        daysUntilExpiry: 25,
        userId: "user-marie-2",
        role: "community.moderator",
        organizationId: "org-cabinet-1",
      }),
    ];

    const { getByTestId } = render(RoleDelegationList, {
      props: {
        initialDelegations: delegations,
        userLabels: {
          "user-pierre-1": "Pierre Dupont",
          "user-marie-2": "Marie Martin",
          [ISSUER_ID]: "Sophie Syndic",
        },
        orgLabels: {
          "org-cabinet-1": "Cabinet Syndic A",
        },
        // Current user = un autre user (pas un destinataire) → pas de
        // banner inherited rouge, mais CTA présent.
        currentUserId: "user-superadmin-x",
        nowOverride: NOW_FIXED,
      },
    });

    // Liste visible
    await waitFor(() =>
      expect(getByTestId("role-delegation-list")).toBeTruthy(),
    );
    // CTA présent
    expect(getByTestId("role-delegate-new-button")).toBeTruthy();
    // Banner info (jaune) toujours présent — pédagogie INV-8
    expect(getByTestId("role-delegate-non-transitive-banner")).toBeTruthy();

    // Rangée 1 (d-1, 7j → urgent)
    expect(getByTestId("role-delegation-row-d-1")).toBeTruthy();
    expect(getByTestId("role-delegation-row-target-d-1").textContent).toMatch(
      /Pierre Dupont/,
    );
    expect(getByTestId("role-delegation-row-role-d-1").textContent).toMatch(
      /syndic/,
    );
    expect(getByTestId("role-delegation-row-source-d-1").textContent).toMatch(
      /Sophie Syndic/,
    );
    const badge1 = getByTestId("expiration-badge-role-delegation-d-1");
    expect(badge1.getAttribute("data-level")).toBe("urgent");

    // Rangée 2 (d-2, 25j → soon)
    expect(getByTestId("role-delegation-row-target-d-2").textContent).toMatch(
      /Marie Martin/,
    );
    expect(getByTestId("role-delegation-row-role-d-2").textContent).toMatch(
      /community\.moderator/,
    );
    const badge2 = getByTestId("expiration-badge-role-delegation-d-2");
    expect(badge2.getAttribute("data-level")).toBe("soon");
  });

  it("@edge expirations à 3j → urgent (rouge), 25j → soon (orange)", async () => {
    const delegations = [
      mkDelegation({ id: "d-urgent", daysUntilExpiry: 3 }),
      mkDelegation({ id: "d-soon", daysUntilExpiry: 25 }),
    ];

    const { getByTestId } = render(RoleDelegationList, {
      props: {
        initialDelegations: delegations,
        currentUserId: "user-superadmin-x",
        nowOverride: NOW_FIXED,
      },
    });

    await waitFor(() =>
      expect(getByTestId("role-delegation-list")).toBeTruthy(),
    );

    expect(
      getByTestId("expiration-badge-role-delegation-d-urgent").getAttribute(
        "data-level",
      ),
    ).toBe("urgent");
    expect(
      getByTestId("expiration-badge-role-delegation-d-soon").getAttribute(
        "data-level",
      ),
    ).toBe("soon");
  });

  it("@security current user a hérité → banner rouge + CTA ABSENT du DOM", async () => {
    const PIERRE_ID = "user-pierre-1";
    // Pierre est destinataire d'une délégation depuis le syndic (delegated_from
    // set) → il a HÉRITÉ son rôle → re-délégation interdite (INV-8).
    const delegations = [
      mkDelegation({
        id: "d-pierre",
        daysUntilExpiry: 7,
        userId: PIERRE_ID,
        delegatedFromUserId: ISSUER_ID,
      }),
    ];

    const { getByTestId, queryByTestId } = render(RoleDelegationList, {
      props: {
        initialDelegations: delegations,
        currentUserId: PIERRE_ID,
        nowOverride: NOW_FIXED,
      },
    });

    await waitFor(() =>
      expect(getByTestId("role-delegation-list")).toBeTruthy(),
    );

    // Banner présent + ROUGE (wording "vous avez reçu")
    const banner = getByTestId("role-delegate-non-transitive-banner");
    expect(banner).toBeTruthy();
    expect(banner.textContent).toMatch(/reçu|délégation/i);
    expect(banner.className).toMatch(/bg-red-50/);

    // CTA ABSENT du DOM — pas juste disabled (AC @security stories.md §B4)
    expect(queryByTestId("role-delegate-new-button")).toBeNull();
  });

  it("@negative liste vide → message empty + banner d'info toujours présent (fallback)", async () => {
    const { queryByTestId, getByTestId } = render(RoleDelegationList, {
      props: {
        initialDelegations: [],
        // currentUserId === undefined → fallback conservateur : pas de
        // détection inherited mais banner info quand même affiché.
        currentUserId: undefined,
        nowOverride: NOW_FIXED,
      },
    });

    // Pas de table
    expect(queryByTestId("role-delegation-list")).toBeNull();
    // Message empty
    expect(getByTestId("role-delegation-list-empty").textContent).toMatch(
      /Aucune/i,
    );
    // CTA présent (fallback : on ne MASQUE PAS sans certitude)
    expect(getByTestId("role-delegate-new-button")).toBeTruthy();
    // Banner d'info toujours présent (pédagogie INV-8)
    const banner = getByTestId("role-delegate-non-transitive-banner");
    expect(banner).toBeTruthy();
    expect(banner.className).toMatch(/bg-yellow-50/);
  });
});
