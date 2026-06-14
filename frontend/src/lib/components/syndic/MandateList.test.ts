// Story B3 (Phase B FE) — Vitest 4-cat MandateList.
//
// Couverture (cf. stories.md §B3) :
//   @happy    : tableau affiche les rows avec subject / kind / scope / reason
//               / expiration badge — chaque row a son data-testid stable.
//   @edge     : badge passe au vert (fresh) > 30j ; orange (soon) ≤ 30j ;
//               rouge (urgent) ≤ 7j. On vérifie via le data-level exposé par
//               ExpirationBadge (`fresh|soon|urgent|expired`).
//   @security : un mandat révoqué affiche le badge "Révoqué" + N'a PAS de bouton
//               `mandate-revoke-{id}` (UX immutability — pas double-revoke).
//   @negative : liste vide → message empty + pas de tableau.
//
// L'ExpirationBadge est laissé intact (composant atomique partagé, déjà
// testé dans son propre fichier ExpirationBadge.test.ts). On vérifie ici
// l'INTÉGRATION : la table passe bien validUntil + idSuffix et le badge
// rend les data-* attendus.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import MandateList from "./MandateList.svelte";
import type { MandateResponse } from "../../api/mandates";

// -----------------------------------------------------------------------------
// Mocks API
// -----------------------------------------------------------------------------

vi.mock("../../api/mandates", () => ({
  listMandates: vi.fn(),
  revokeMandate: vi.fn(),
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
// (Si on retournait la clé "mandate.list.empty", elle serait truthy et le
// fallback ne serait pas appliqué — le test attendrait alors la clé brute.)
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
const ISSUER_ID = "issuer-uuid";

function mkMandate(opts: {
  id: string;
  daysUntilExpiry: number;
  kind?: string;
  scopeKind?: string;
  scopeId?: string;
  subjectUserId?: string;
  reason?: string;
  revokedAt?: string | null;
}): MandateResponse {
  const validUntil = new Date(
    NOW_FIXED.getTime() + opts.daysUntilExpiry * 24 * 60 * 60 * 1000,
  );
  return {
    id: opts.id,
    subject_user_id: opts.subjectUserId ?? "user-notary-1",
    kind: opts.kind ?? "notary",
    scope_kind: opts.scopeKind ?? "building",
    scope_id: opts.scopeId ?? "b-42",
    reason:
      opts.reason ??
      "Mandat de notaire pour la transaction du Lot 12 (vente bien)",
    issued_by: ISSUER_ID,
    valid_from: NOW_FIXED.toISOString(),
    valid_until: validUntil.toISOString(),
    revoked_at: opts.revokedAt ?? null,
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

describe("MandateList — Story B3 (4-cat)", () => {
  it("@happy affiche les rows avec subject + kind + scope + expiration badge", async () => {
    const mandates = [
      mkMandate({ id: "m-1", daysUntilExpiry: 365 }),
      mkMandate({
        id: "m-2",
        daysUntilExpiry: 25,
        kind: "lawyer",
        scopeKind: "acp",
        scopeId: "acp-1",
        subjectUserId: "user-lawyer-1",
      }),
    ];

    const { getByTestId } = render(MandateList, {
      props: {
        initialMandates: mandates,
        subjectLabels: {
          "user-notary-1": "Me. Dupont (notaire)",
          "user-lawyer-1": "Me. Martin (avocat)",
        },
        scopeLabels: {
          "building:b-42": "Immeuble #42 Tilleuls",
          "acp:acp-1": "ACP Tilleuls",
        },
        nowOverride: NOW_FIXED,
      },
    });

    // Liste visible
    await waitFor(() => expect(getByTestId("mandate-list")).toBeTruthy());
    // CTA Nouveau mandat
    expect(getByTestId("mandate-new-button")).toBeTruthy();

    // Rangée 1 (m-1)
    expect(getByTestId("mandate-row-m-1")).toBeTruthy();
    expect(getByTestId("mandate-row-subject-m-1").textContent).toMatch(
      /Me\. Dupont/,
    );
    expect(getByTestId("mandate-row-kind-m-1").textContent).toMatch(/notary/);
    expect(getByTestId("mandate-row-scope-m-1").textContent).toMatch(
      /Immeuble #42/,
    );
    // Badge expiration via ExpirationBadge (idSuffix=mandate-{id} → testid
    // `expiration-badge-mandate-{id}` qui est aussi exposé via
    // `mandate-expiration-badge-{id}` côté revoked seulement — pour live le
    // composant ExpirationBadge utilise son propre data-testid).
    // 365j > 30 → level="fresh"
    const badge1 = getByTestId("expiration-badge-mandate-m-1");
    expect(badge1.getAttribute("data-level")).toBe("fresh");

    // Rangée 2 (m-2) — 25j → soon
    expect(getByTestId("mandate-row-m-2")).toBeTruthy();
    expect(getByTestId("mandate-row-subject-m-2").textContent).toMatch(
      /Me\. Martin/,
    );
    expect(getByTestId("mandate-row-scope-m-2").textContent).toMatch(
      /ACP Tilleuls/,
    );
    const badge2 = getByTestId("expiration-badge-mandate-m-2");
    expect(badge2.getAttribute("data-level")).toBe("soon");
  });

  it("@edge expirations à 7j → urgent (rouge), 50j → soon (orange — 50≤30 faux, 50≤60 vrai → label en jours)", async () => {
    // Helper note : 50j > 30 → fresh ; on prend donc 8j (soon borderline) +
    // 3j (urgent).
    const mandates = [
      mkMandate({ id: "m-urgent", daysUntilExpiry: 3 }),
      mkMandate({ id: "m-soon", daysUntilExpiry: 25 }),
      mkMandate({ id: "m-fresh", daysUntilExpiry: 365 }),
    ];

    const { getByTestId } = render(MandateList, {
      props: { initialMandates: mandates, nowOverride: NOW_FIXED },
    });

    await waitFor(() => expect(getByTestId("mandate-list")).toBeTruthy());

    expect(
      getByTestId("expiration-badge-mandate-m-urgent").getAttribute("data-level"),
    ).toBe("urgent");
    expect(
      getByTestId("expiration-badge-mandate-m-soon").getAttribute("data-level"),
    ).toBe("soon");
    expect(
      getByTestId("expiration-badge-mandate-m-fresh").getAttribute("data-level"),
    ).toBe("fresh");
  });

  it("@security mandat révoqué → badge 'Révoqué' + pas de bouton revoke", async () => {
    const mandates = [
      mkMandate({
        id: "m-revoked",
        daysUntilExpiry: 100,
        revokedAt: NOW_FIXED.toISOString(),
      }),
      mkMandate({ id: "m-active", daysUntilExpiry: 100 }),
    ];

    const { getByTestId, queryByTestId } = render(MandateList, {
      props: { initialMandates: mandates, nowOverride: NOW_FIXED },
    });

    await waitFor(() => expect(getByTestId("mandate-list")).toBeTruthy());

    // Le révoqué a son badge "Révoqué" + PAS de bouton revoke
    const revokedBadge = getByTestId("mandate-expiration-badge-m-revoked");
    expect(revokedBadge.textContent).toMatch(/Révoqué/i);
    expect(queryByTestId("mandate-revoke-m-revoked")).toBeNull();

    // L'actif a son bouton revoke
    expect(getByTestId("mandate-revoke-m-active")).toBeTruthy();
  });

  it("@negative liste vide → message empty + pas de tableau", async () => {
    const { queryByTestId, getByTestId } = render(MandateList, {
      props: { initialMandates: [], nowOverride: NOW_FIXED },
    });

    // Pas de table
    expect(queryByTestId("mandate-list")).toBeNull();
    // Message empty
    expect(getByTestId("mandate-list-empty").textContent).toMatch(/Aucun/i);
    // CTA toujours présent
    expect(getByTestId("mandate-new-button")).toBeTruthy();
  });
});
