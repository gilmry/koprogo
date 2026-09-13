import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "../../../test-helpers";
import DettesParAcp from "../DettesParAcp.svelte";

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
 * Les dettes d'un copropriétaire, ventilées par association.
 *
 * ── Ce que ces tests tiennent, et pourquoi ça n'est pas cosmétique ──────
 *
 * **Chaque ACP est une personne morale distincte, avec son propre compte
 * bancaire** — Art. 3.86 § 1er pour la personnalité juridique, § 3 pour les
 * comptes ouverts à son nom.
 *
 * Un copropriétaire qui verrait un seul montant et ferait un seul virement
 * paierait la mauvaise personne morale pour une partie de la somme : l'argent
 * atterrirait sur le compte de l'ACP A pour des charges dues à l'ACP B. Le
 * syndic de B devrait réclamer, celui de A rembourser.
 *
 * Le test central est donc `@security` : **un bouton de paiement par
 * association, jamais un seul pour le total**. C'est la propriété que le
 * produit ne doit jamais perdre, et un raccourci d'affichage suffirait à la
 * casser.
 *
 * ── Et l'inverse, qui compte autant ────────────────────────────────────
 *
 * Avec une SEULE association, la décomposition est du bruit : le total
 * répéterait la ligne unique, et l'explication répondrait à une question que
 * personne ne se pose. La majorité des copropriétaires n'ont qu'une
 * copropriété. Un choix unique n'est pas un menu.
 */

const DEUX_ACP = [
  {
    acp_id: "acp-1",
    acp_name: "Les Érables",
    bce_number: "0123.456.789",
    charges_en_attente: 2,
    montant: 842.5,
  },
  {
    acp_id: "acp-2",
    acp_name: "Les Glycines",
    bce_number: "0987.654.321",
    charges_en_attente: 1,
    montant: 420,
  },
];

const UNE_ACP = [DEUX_ACP[0]];

beforeEach(() => {
  getMoque.mockReset();
});

describe("@security un paiement par association, jamais un pour le total", () => {
  it("rend un bouton de paiement PAR association", async () => {
    getMoque.mockResolvedValue(DEUX_ACP as never);
    render(DettesParAcp);

    await waitFor(() => {
      expect(screen.queryAllByTestId("pay-acp-button").length).toBe(2);
    });

    // Chaque bouton porte l'identifiant de SA créancière : c'est ce qui
    // garantit qu'un paiement ne peut pas être imputé à une autre.
    const boutons = screen.getAllByTestId("pay-acp-button");
    expect(boutons.map((b) => b.getAttribute("data-acp-id")).sort()).toEqual([
      "acp-1",
      "acp-2",
    ]);
  });

  it("porte le numéro BCE de chaque créancière", async () => {
    // Le BCE identifie la personne morale à qui l'argent est dû ; c'est ce
    // qu'on recopie sur le virement. L'Art. 3.86 § 1er al. 4 impose qu'il
    // figure sur tout document émanant de l'association.
    getMoque.mockResolvedValue(DEUX_ACP as never);
    render(DettesParAcp);

    await waitFor(() => {
      expect(screen.queryAllByTestId("dette-acp").length).toBe(2);
    });
    const texte = screen.getByTestId("dettes-par-acp").textContent ?? "";
    expect(texte).toContain("0123.456.789");
    expect(texte).toContain("0987.654.321");
  });

  it("explique POURQUOI il y a deux boutons, à l'endroit où la question se pose", async () => {
    // Un copropriétaire qui voit deux boutons se demande pourquoi il ne peut
    // pas tout régler d'un coup. La réponse doit être là, pas dans une
    // infobulle que personne n'ouvre.
    getMoque.mockResolvedValue(DEUX_ACP as never);
    render(DettesParAcp);

    await waitFor(() => {
      expect(screen.queryByTestId("dettes-par-acp-explication")).toBeTruthy();
    });
  });
});

describe("@happy le total consolidé répond à « combien dois-je »", () => {
  it("additionne les montants des deux associations", async () => {
    getMoque.mockResolvedValue(DEUX_ACP as never);
    render(DettesParAcp);

    await waitFor(() => {
      expect(screen.queryByTestId("dettes-par-acp-total")).toBeTruthy();
    });
    expect(screen.getByTestId("dettes-par-acp-total")).toHaveAttribute(
      "data-total",
      "1262.5",
    );
  });
});

describe("@edge l'affordance est conditionnelle", () => {
  it("n'affiche ni total ni explication avec une seule association", async () => {
    // Le total répéterait la ligne unique ; l'explication répondrait à une
    // question que personne ne se pose. Un choix unique n'est pas un menu.
    getMoque.mockResolvedValue(UNE_ACP as never);
    render(DettesParAcp);

    await waitFor(() => {
      expect(screen.queryAllByTestId("dette-acp").length).toBe(1);
    });
    expect(screen.queryByTestId("dettes-par-acp-total")).toBeNull();
    expect(screen.queryByTestId("dettes-par-acp-explication")).toBeNull();
    // Mais le bouton de paiement reste, évidemment.
    expect(screen.queryAllByTestId("pay-acp-button").length).toBe(1);
  });
});

describe("@negative les états qui ne sont pas une dette", () => {
  it("dit qu'il n'y a rien à payer, plutôt que de rester vide", async () => {
    // Ne rien devoir est une bonne nouvelle. Un encart vide se lit comme une
    // panne.
    getMoque.mockResolvedValue([] as never);
    render(DettesParAcp);

    await waitFor(() => {
      expect(screen.queryByTestId("dettes-par-acp-rien")).toBeTruthy();
    });
    expect(screen.queryByTestId("dettes-par-acp-erreur")).toBeNull();
  });

  it("porte le code quand la route échoue", async () => {
    getMoque.mockRejectedValue(new Error("HTTP 503"));
    render(DettesParAcp);

    await waitFor(() => {
      expect(screen.queryByTestId("dettes-par-acp-erreur")).toBeTruthy();
    });
    // Le code distingue une panne d'un refus, et c'est ce qui décide de ce
    // qu'on fait ensuite.
    expect(screen.getByTestId("dettes-par-acp-erreur").textContent).toContain(
      "HTTP 503",
    );
  });
});
