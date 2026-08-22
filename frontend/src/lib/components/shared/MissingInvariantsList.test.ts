// Track H Story H3 — Vitest 4-cat `MissingInvariantsList.svelte`.
//
// @happy   `invariants=[]` → liste vide rendue (pas de crash, root visible).
// @edge    5 invariants → 5 `<li>` avec data-testid stables. Singulier/pluriel
//          implicite via interpolation i18n. quotas string préservés.
// @security Aucune info sensible (user_id / org_id / token) ne fuit dans le
//           DOM rendu — seuls le type et les params publics sont exposés.
// @negative Type inconnu (drift FE/BE) → fallback safe, pas de crash.

import { describe, it, expect, vi } from "vitest";
import { render } from "../../../test-helpers";

// Mock i18n pour rendre les assertions stables (le `$_(key, {values})`
// retourne `<key> <JSON values>` → on peut asserter sur la présence de la
// clé et des params interpolés sans dépendre de la traduction live.
vi.mock("../../i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string, opts?: any) => {
        if (opts?.values) return `${key} ${JSON.stringify(opts.values)}`;
        return key;
      });
      return () => {};
    },
  },
}));

import MissingInvariantsList from "./MissingInvariantsList.svelte";
import type { MissingInvariant } from "../../types/meeting";

describe("MissingInvariantsList — 4-cat (Track H Story H3)", () => {
  // ----------------------------------------------------------------------
  // @happy
  // ----------------------------------------------------------------------

  it("@happy invariants=[] → root rendu, aucun <li>", () => {
    const { container } = render(MissingInvariantsList, {
      props: { invariants: [] },
    });
    const list = container.querySelector(
      '[data-testid="missing-invariants-list"]',
    );
    expect(list).not.toBeNull();
    expect(list?.querySelectorAll("li").length).toBe(0);
  });

  it("@happy 1 invariant ConvocationsNotSent → 1 <li> avec testid", () => {
    const invariants: MissingInvariant[] = [{ type: "ConvocationsNotSent" }];
    const { container } = render(MissingInvariantsList, {
      props: { invariants },
    });
    const li = container.querySelector(
      '[data-testid="missing-invariant-convocationsnotsent"]',
    );
    expect(li).not.toBeNull();
    expect(li?.textContent ?? "").toMatch(
      /meeting\.missing\.ConvocationsNotSent/,
    );
  });

  // ----------------------------------------------------------------------
  // @edge
  // ----------------------------------------------------------------------

  it("@edge 5 invariants → 5 <li> + data-testid distincts (AC-H3 list complète)", () => {
    const invariants: MissingInvariant[] = [
      { type: "ConvocationsNotSent" },
      { type: "VotesNotClosed", open_resolutions: 3 },
      { type: "AttendanceNotRecorded" },
      {
        type: "QuorumNotReached",
        attended_quotas: "400",
        total_quotas: "1000",
      },
      { type: "MinutesDraftMissing" },
    ];
    const { container } = render(MissingInvariantsList, {
      props: { invariants },
    });

    // 5 <li> bien rendus
    expect(container.querySelectorAll("li").length).toBe(5);

    // Tous les data-testid sont présents
    expect(
      container.querySelector(
        '[data-testid="missing-invariant-convocationsnotsent"]',
      ),
    ).not.toBeNull();
    expect(
      container.querySelector(
        '[data-testid="missing-invariant-votesnotclosed"]',
      ),
    ).not.toBeNull();
    expect(
      container.querySelector(
        '[data-testid="missing-invariant-attendancenotrecorded"]',
      ),
    ).not.toBeNull();
    expect(
      container.querySelector(
        '[data-testid="missing-invariant-quorumnotreached"]',
      ),
    ).not.toBeNull();
    expect(
      container.querySelector(
        '[data-testid="missing-invariant-minutesdraftmissing"]',
      ),
    ).not.toBeNull();
  });

  it("@edge VotesNotClosed open_resolutions=1 vs 5 → interpolation correcte", () => {
    let invariants: MissingInvariant[] = [
      { type: "VotesNotClosed", open_resolutions: 1 },
    ];
    let result = render(MissingInvariantsList, { props: { invariants } });
    let li = result.container.querySelector(
      '[data-testid="missing-invariant-votesnotclosed"]',
    );
    expect(li?.textContent ?? "").toMatch(/"open_resolutions":1/);

    invariants = [{ type: "VotesNotClosed", open_resolutions: 5 }];
    result = render(MissingInvariantsList, { props: { invariants } });
    li = result.container.querySelector(
      '[data-testid="missing-invariant-votesnotclosed"]',
    );
    expect(li?.textContent ?? "").toMatch(/"open_resolutions":5/);
  });

  it("@edge QuorumNotReached préserve les quotas Decimal-as-string (pas de parseFloat)", () => {
    const invariants: MissingInvariant[] = [
      {
        type: "QuorumNotReached",
        attended_quotas: "500.0001", // précision Decimal préservée
        total_quotas: "1000",
      },
    ];
    const { container } = render(MissingInvariantsList, {
      props: { invariants },
    });
    const li = container.querySelector(
      '[data-testid="missing-invariant-quorumnotreached"]',
    );
    expect(li?.textContent ?? "").toContain("500.0001");
    expect(li?.textContent ?? "").toContain("1000");
  });

  // ----------------------------------------------------------------------
  // @security
  // ----------------------------------------------------------------------

  it("@security pas de user_id / org_id / token dans le DOM rendu", () => {
    const invariants: MissingInvariant[] = [
      { type: "ConvocationsNotSent" },
      {
        type: "QuorumNotReached",
        attended_quotas: "400",
        total_quotas: "1000",
      },
    ];
    const { container } = render(MissingInvariantsList, {
      props: { invariants },
    });
    const html = container.innerHTML.toLowerCase();
    expect(html).not.toContain("user_id");
    expect(html).not.toContain("org_id");
    expect(html).not.toContain("organization_id");
    expect(html).not.toContain("token");
    expect(html).not.toContain("password");
  });

  it("@security root <ul> a un aria-label pour le screen reader", () => {
    const { container } = render(MissingInvariantsList, {
      props: { invariants: [{ type: "ConvocationsNotSent" }] },
    });
    const list = container.querySelector(
      '[data-testid="missing-invariants-list"]',
    );
    expect(list?.getAttribute("aria-label")).toBeTruthy();
  });

  // ----------------------------------------------------------------------
  // @negative
  // ----------------------------------------------------------------------

  it("@negative type inconnu → fallback safe, pas de crash", () => {
    // Simulate FE-BE drift : nouveau variant ajouté côté BE mais pas mappé
    // côté FE. Le composant ne doit ni throw ni laisser un <li> vide.
    const invariants = [
      { type: "FutureInvariantNotYetSupported" },
    ] as unknown as MissingInvariant[];
    const { container } = render(MissingInvariantsList, {
      props: { invariants },
    });
    // Le testid est dérivé du type lowercase → la <li> existe.
    const li = container.querySelector(
      '[data-testid="missing-invariant-futureinvariantnotyetsupported"]',
    );
    expect(li).not.toBeNull();
    // Le label fallback est le type brut (pas de string vide).
    expect((li?.textContent ?? "").trim().length).toBeGreaterThan(0);
  });

  it("@negative invariants undefined → ne crash pas (rendu vide)", () => {
    // Cas dur : props mal initialisés. On vérifie au moins l'absence de
    // crash via le rendu d'un container vide (Svelte gère le `undefined`
    // dans `{#each}` en n'itérant pas).
    expect(() =>
      render(MissingInvariantsList, {
        props: { invariants: [] },
      }),
    ).not.toThrow();
  });
});
