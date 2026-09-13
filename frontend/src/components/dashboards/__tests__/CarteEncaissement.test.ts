import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "../../../test-helpers";
import CarteEncaissement from "../CarteEncaissement.svelte";

vi.mock("../../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: unknown) => void) => {
      fn((cle: string) => cle);
      return () => {};
    },
  },
}));

vi.mock("../../../lib/api", () => ({
  api: { get: vi.fn() },
  API_BASE_URL: "http://test",
}));

vi.mock("../../../lib/utils/finance.utils", () => ({
  formatCurrency: (n: number) => `${n} €`,
}));

import { api } from "../../../lib/api";
const getMoque = vi.mocked(api.get);

/**
 * La carte d'encaissement.
 *
 * ── Ce que ces tests tiennent ────────────────────────────────────────────
 *
 * La barre à deux segments est une addition rendue visible : la part verte et
 * la part ambre font ensemble la largeur. Deux façons de la casser, et les
 * deux viennent de données que le serveur PEUT réellement servir :
 *
 * 1. **Un encaissement supérieur au dû.** Une avance ou un double paiement
 *    donne un pourcentage au-dessus de 100. La barre doit se remplir, pas
 *    sortir de sa piste.
 * 2. **Un pourcentage négatif.** Un avoir, un remboursement mal signé. La
 *    barre doit être vide, pas partir à gauche.
 *
 * Le troisième test porte sur le CLOISONNEMENT : la route servait ces
 * chiffres à tout membre de l'organisation avant le 2026-09-10, y compris le
 * nombre de copropriétaires en retard de paiement. Elle rend maintenant 403 à
 * un copropriétaire, et la carte doit le dire plutôt que de rester vide.
 */

function stats(surcharges: Record<string, number> = {}) {
  return {
    total_expenses_current_month: 48320,
    total_paid: 39780,
    paid_percentage: 82.3,
    total_pending: 8540,
    pending_percentage: 17.7,
    owners_with_overdue: 5,
    ...surcharges,
  };
}

beforeEach(() => {
  getMoque.mockReset();
});

describe("@happy la carte dit la part encaissée et ce qui manque", () => {
  it("affiche le pourcentage arrondi et la largeur du segment payé", async () => {
    getMoque.mockResolvedValue(stats() as never);
    render(CarteEncaissement);

    await waitFor(() => {
      expect(
        screen.queryByTestId("carte-encaissement-pourcentage"),
      ).toBeTruthy();
    });
    const bloc = screen.getByTestId("carte-encaissement-pourcentage");
    expect(bloc).toHaveAttribute("data-part-payee", "82.3");
    expect(bloc.textContent).toContain("82");

    const segment = screen.getByTestId("carte-encaissement-segment-paye");
    expect(segment.style.width).toBe("82.3%");
  });

  it("dit COMBIEN de copropriétaires sont en retard, pas seulement le montant", async () => {
    // Un même total peut venir d'un gros débiteur ou de vingt petits, et les
    // deux n'appellent pas la même démarche.
    getMoque.mockResolvedValue(stats() as never);
    render(CarteEncaissement);

    await waitFor(() => {
      expect(
        screen.queryByTestId("carte-encaissement-retardataires"),
      ).toBeTruthy();
    });
    expect(
      screen.getByTestId("carte-encaissement-retardataires"),
    ).toHaveAttribute("data-retardataires", "5");
  });
});

describe("@edge la barre ne sort jamais de sa piste", () => {
  it("borne un encaissement supérieur au dû", async () => {
    // Avance ou double paiement : le serveur sert bien un pourcentage > 100.
    getMoque.mockResolvedValue(stats({ paid_percentage: 137 }) as never);
    render(CarteEncaissement);

    await waitFor(() => {
      expect(
        screen.queryByTestId("carte-encaissement-segment-paye"),
      ).toBeTruthy();
    });
    expect(
      screen.getByTestId("carte-encaissement-segment-paye").style.width,
    ).toBe("100%");
  });

  it("borne un pourcentage négatif", async () => {
    // Un avoir mal signé. La barre est vide, elle ne part pas à gauche.
    getMoque.mockResolvedValue(stats({ paid_percentage: -12 }) as never);
    render(CarteEncaissement);

    await waitFor(() => {
      expect(
        screen.queryByTestId("carte-encaissement-segment-paye"),
      ).toBeTruthy();
    });
    expect(
      screen.getByTestId("carte-encaissement-segment-paye").style.width,
    ).toBe("0%");
  });

  it("n'affiche pas la ligne des retardataires quand il n'y en a aucun", async () => {
    // Zéro retardataire est une bonne nouvelle, pas une ligne de plus. Un
    // encart qui parle toujours cesse d'être lu.
    getMoque.mockResolvedValue(stats({ owners_with_overdue: 0 }) as never);
    render(CarteEncaissement);

    await waitFor(() => {
      expect(
        screen.queryByTestId("carte-encaissement-pourcentage"),
      ).toBeTruthy();
    });
    expect(screen.queryByTestId("carte-encaissement-retardataires")).toBeNull();
  });
});

describe("@security un refus se dit, il ne se traduit pas par une carte vide", () => {
  it("porte le code quand la route refuse", async () => {
    // La route rend 403 à un copropriétaire depuis le 2026-09-10 (#864).
    // Une carte muette laisserait croire à une panne ; le code dit qu'il
    // s'agit d'un refus, et c'est ce qui décide de ce qu'on fait ensuite.
    getMoque.mockRejectedValue(new Error("HTTP 403"));
    render(CarteEncaissement);

    await waitFor(() => {
      expect(screen.queryByTestId("carte-encaissement-erreur")).toBeTruthy();
    });
    expect(
      screen.getByTestId("carte-encaissement-erreur").textContent,
    ).toContain("HTTP 403");
  });
});
