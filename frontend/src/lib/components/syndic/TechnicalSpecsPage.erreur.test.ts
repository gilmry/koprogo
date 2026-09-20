/**
 * Un refus du serveur se DIT — il ne devient pas une liste vide.
 *
 * ── Le défaut figé ici ────────────────────────────────────────────────────
 *
 * Le balayage du 2026-09-19 a relevé un **400** sur
 * `/syndic/technical-specs` pour 492 caractères rendus (#968). La cause
 * tenait en trois pièces :
 *
 *   1. `acp_id` est obligatoire côté serveur
 *      (`ListTechnicalSpecsQuery.acp_id: Uuid`, non-`Option`) ;
 *   2. la page appelait `listSpecs()` SANS argument, en parallèle du
 *      chargement des ACP — donc avant de savoir de laquelle elle parlait ;
 *   3. `.catch(() => [])` transformait le refus en liste vide.
 *
 * L'écran annonçait alors « aucun cahier des charges » à un syndic qui en
 * avait. **Vide et cassé se ressemblaient**, et seule la console les
 * distinguait — c'est-à-dire personne.
 *
 * ── Pourquoi ce test est ici et pas en Playwright ─────────────────────────
 *
 * Je l'ai d'abord écrit en recette navigateur, en interposant un 400 sur la
 * route. Il ne se déclenchait jamais : le **service worker** de la PWA sert
 * ces appels, et `page.route()` ne l'intercepte pas. L'instrumentation l'a
 * montré — aucune requête visible après navigation, alors que l'état vide
 * s'affichait.
 *
 * Insister aurait produit une recette qui éprouve surtout la couche de
 * cache. Ici, le verdict est déterministe.
 */
import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, waitFor, cleanup } from "../../../test-helpers";

const listSpecs = vi.fn();
const listAcps = vi.fn();

vi.mock("../../api/technical_specs", () => ({
  listSpecs: (...a: unknown[]) => listSpecs(...a),
  createSpec: vi.fn(),
}));
vi.mock("../../api/acps", () => ({
  listAcps: (...a: unknown[]) => listAcps(...a),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function monter() {
  const { default: TechnicalSpecsPage } =
    await import("./TechnicalSpecsPage.svelte");
  return render(TechnicalSpecsPage, {});
}

const UNE_ACP = [{ id: "acp-1", name: "Résidence du Test" }];

describe("Cahiers des charges — l'écran ne déguise pas un refus (#968)", () => {
  it("@negative affiche le message du serveur quand la liste est refusée", async () => {
    listAcps.mockResolvedValue(UNE_ACP);
    listSpecs.mockRejectedValue(
      new Error("Query deserialize error: missing field `acp_id`"),
    );

    await monter();

    await waitFor(() => {
      expect(screen.getByTestId("tech-spec-list-error")).toBeTruthy();
    });

    // Le message porte ce que le SERVEUR a dit. « Une erreur est survenue »
    // ne permettrait pas de distinguer un paramètre manquant d'une panne.
    expect(screen.getByTestId("tech-spec-list-error").textContent).toContain(
      "acp_id",
    );

    // Et surtout : l'écran ne prétend PAS que la liste est vide.
    expect(screen.queryByTestId("tech-spec-list-empty")).toBeNull();
  });

  it("@happy n'interroge la liste qu'une fois l'ACP connue, et avec elle", async () => {
    listAcps.mockResolvedValue(UNE_ACP);
    listSpecs.mockResolvedValue([]);

    await monter();

    await waitFor(() => {
      expect(listSpecs).toHaveBeenCalled();
    });

    // Le cœur du correctif : l'appel porte l'ACP. Sans elle, le serveur
    // refuse toujours — `technical_specs.ts:147` le documentait déjà.
    expect(
      listSpecs,
      "La liste part encore sans `acp_id` : le serveur refusera toujours.",
    ).toHaveBeenCalledWith("acp-1");
  });

  it("@edge sans aucune ACP, l'écran vide est la vérité et rien n'est demandé", async () => {
    listAcps.mockResolvedValue([]);
    listSpecs.mockResolvedValue([]);

    await monter();

    await waitFor(() => {
      expect(screen.queryByTestId("tech-spec-list-empty")).toBeTruthy();
    });

    // Un compte qui ne gère aucune ACP n'a rien à lister : interroger le
    // serveur produirait un 400 et un faux défaut.
    expect(
      listSpecs,
      "La liste est demandée alors qu'aucune ACP n'est connue.",
    ).not.toHaveBeenCalled();
    expect(screen.queryByTestId("tech-spec-list-error")).toBeNull();
  });
});
