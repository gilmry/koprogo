import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "../../../test-helpers";
import TableDesAcp from "../TableDesAcp.svelte";

vi.mock("../../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: unknown) => void) => {
      fn((cle: string) => cle);
      return () => {};
    },
  },
}));

vi.mock("../../../lib/api/acps", () => ({
  listAcpsWithMetrics: vi.fn(),
}));

import { listAcpsWithMetrics } from "../../../lib/api/acps";
const listeMoquee = vi.mocked(listAcpsWithMetrics);

/**
 * La table « Mes ACP ».
 *
 * ── Ce que ces tests tiennent ────────────────────────────────────────────
 *
 * Le point délicat n'est pas d'afficher une liste, c'est de dire JUSTE si une
 * copropriété est en écart. Deux façons de se tromper, et les deux ont des
 * conséquences réelles :
 *
 * 1. **Signaler un écart qui n'existe pas.** `1000` et `1000.0000` désignent
 *    la même quotité. Une comparaison de chaînes naïve les distingue, et une
 *    ACP parfaitement conforme apparaîtrait en alerte — le syndic irait
 *    corriger ce qui va bien, et cesserait de croire l'indicateur.
 * 2. **Taire un écart réel.** Convertir la quotité en `number` pour comparer
 *    introduirait une erreur de représentation sur une valeur juridiquement
 *    opposable. Un écart d'un millième passerait inaperçu, et chaque appel de
 *    fonds porterait sur une base fausse.
 */

function acp(surcharges: Record<string, unknown> = {}) {
  return {
    id: "acp-1",
    organization_id: "org-1",
    name: "Les Érables",
    slug: "les-erables",
    legal_status: "acp",
    total_tantiemes: 1000,
    bce_number: "0123.456.789",
    address_street: "Rue de la Loi 1",
    address_postal_code: "1000",
    address_city: "Bruxelles",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    buildings_count: 2,
    units_count: 32,
    declared_units_total: 32,
    quota_sum: "1000",
    ...surcharges,
  };
}

beforeEach(() => {
  listeMoquee.mockReset();
});

describe("@happy la table dit ce qui va et ce qui ne va pas", () => {
  it("affiche une ACP conforme sans alerte", async () => {
    listeMoquee.mockResolvedValue([acp()] as never);
    render(TableDesAcp);

    await waitFor(() => {
      expect(screen.queryByTestId("acp-row")).toBeTruthy();
    });
    expect(screen.getByTestId("acp-row")).toHaveAttribute(
      "data-quotites-completes",
      "true",
    );
  });

  it("signale une ACP dont les quotités n'atteignent pas le total de l'acte", async () => {
    listeMoquee.mockResolvedValue([
      acp({ quota_sum: "742", units_count: 23, declared_units_total: 32 }),
    ] as never);
    render(TableDesAcp);

    await waitFor(() => {
      expect(screen.queryByTestId("acp-row")).toBeTruthy();
    });
    expect(screen.getByTestId("acp-row")).toHaveAttribute(
      "data-quotites-completes",
      "false",
    );
    // Les deux nombres ensemble : afficher le seul encodé ferait croire
    // l'immeuble complet.
    expect(screen.getByTestId("acp-row").textContent).toContain("23/32");
  });
});

describe("@edge la comparaison de quotités ne se laisse pas piéger par la forme", () => {
  it("tient 1000.0000 pour égal à 1000", async () => {
    // Postgres sert un `NUMERIC` avec ses décimales : `SUM(quota::NUMERIC)`
    // peut rendre « 1000.0000 ». Une comparaison de chaînes naïve mettrait
    // cette ACP en alerte alors qu'elle est parfaitement conforme.
    listeMoquee.mockResolvedValue([acp({ quota_sum: "1000.0000" })] as never);
    render(TableDesAcp);

    await waitFor(() => {
      expect(screen.queryByTestId("acp-row")).toBeTruthy();
    });
    expect(screen.getByTestId("acp-row")).toHaveAttribute(
      "data-quotites-completes",
      "true",
    );
  });

  it("ne tient PAS 999.9999 pour égal à 1000", async () => {
    // Un dix-millième d'écart reste un écart : les quotités ne totalisent pas
    // l'acte, et les appels de fonds portent sur une base incomplète. Une
    // conversion en flottant pourrait l'arrondir et le faire disparaître.
    listeMoquee.mockResolvedValue([acp({ quota_sum: "999.9999" })] as never);
    render(TableDesAcp);

    await waitFor(() => {
      expect(screen.queryByTestId("acp-row")).toBeTruthy();
    });
    expect(screen.getByTestId("acp-row")).toHaveAttribute(
      "data-quotites-completes",
      "false",
    );
  });
});

describe("@negative les états qui ne sont pas une liste", () => {
  it("distingue l'absence d'ACP d'une panne", async () => {
    listeMoquee.mockResolvedValue([] as never);
    render(TableDesAcp);

    await waitFor(() => {
      expect(screen.queryByTestId("table-des-acp-vide")).toBeTruthy();
    });
    expect(screen.queryByTestId("table-des-acp-erreur")).toBeNull();
  });

  it("porte le code d'erreur, pas seulement « une erreur est survenue »", async () => {
    listeMoquee.mockRejectedValue(new Error("HTTP 503"));
    render(TableDesAcp);

    await waitFor(() => {
      expect(screen.queryByTestId("table-des-acp-erreur")).toBeTruthy();
    });
    // Le code distingue une route absente d'un serveur en panne, et c'est
    // cette distinction qui décide de ce qu'on fait ensuite.
    expect(screen.getByTestId("table-des-acp-erreur").textContent).toContain(
      "HTTP 503",
    );
  });
});
