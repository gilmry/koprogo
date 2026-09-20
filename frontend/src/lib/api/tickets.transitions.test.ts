/**
 * Les transitions d'un incident envoient ce que le serveur exige (#977).
 *
 * ── Le défaut figé ici ────────────────────────────────────────────────────
 *
 * Trois DTO du backend portent un champ OBLIGATOIRE — `String`, pas
 * `Option<String>` (`application/dto/ticket_dto.rs:143-155`) :
 *
 *     ResolveTicketRequest { resolution_notes: String }
 *     CancelTicketRequest  { reason: String }
 *     ReopenTicketRequest  { reason: String }
 *
 * Le client envoyait `{}` aux trois. Le refus n'arrivait pas au niveau
 * métier : `web::Json<T>` échoue à la DÉSÉRIALISATION, avant que le serveur
 * n'ait regardé le ticket. Mesuré contre la recette le 2026-09-20 :
 *
 *     PUT /tickets/{id}/resolve  {}  -> 400  missing field `resolution_notes`
 *     PUT /tickets/{id}/cancel   {}  -> 400  missing field `reason`
 *     PUT /tickets/{id}/reopen   {}  -> 400  missing field `reason`
 *     PUT /tickets/{id}/resolve  {"resolution_notes":"…"}  -> 200  Resolved
 *
 * Conséquence : un incident assigné ne pouvait JAMAIS être clos depuis
 * l'interface. Le cycle s'arrêtait à `InProgress`, le bouton affichait un
 * toast d'échec, et l'écran restait identique.
 *
 * ── Pourquoi CE test, et pas seulement la recette navigateur ──────────────
 *
 * `incident.journey.ts` prouve le cycle complet, et c'est lui qui a trouvé
 * le défaut. Mais il coûte deux minutes et demie, il lui faut une pile qui
 * tourne, et il ne dit pas QUEL champ manque.
 *
 * Ici le verdict porte sur le corps de la requête, en quelques
 * millisecondes. Les deux se complètent : la recette prouve que ça marche,
 * ce test dit pourquoi le jour où ça cassera.
 */
import { describe, it, expect, vi, beforeEach } from "vitest";

const put = vi.fn().mockResolvedValue({ id: "t-1", status: "Resolved" });

vi.mock("../api", () => ({
  api: {
    put: (...args: unknown[]) => put(...args),
    get: vi.fn(),
    post: vi.fn(),
    delete: vi.fn(),
  },
}));

beforeEach(() => {
  put.mockClear();
});

async function ticketsApi() {
  return (await import("./tickets")).ticketsApi;
}

describe("Transitions d'un incident — le corps porte le champ exigé (#977)", () => {
  it("@happy résoudre transmet la note de résolution", async () => {
    const api = await ticketsApi();
    await api.resolve("t-1", "Joint remplacé, plafond asséché.");

    expect(put).toHaveBeenCalledWith("/tickets/t-1/resolve", {
      resolution_notes: "Joint remplacé, plafond asséché.",
    });
  });

  it("@happy annuler et rouvrir transmettent leur raison", async () => {
    const api = await ticketsApi();
    await api.cancel("t-1", "Doublon du signalement précédent.");
    expect(put).toHaveBeenLastCalledWith("/tickets/t-1/cancel", {
      reason: "Doublon du signalement précédent.",
    });

    await api.reopen("t-1", "La fuite a recommencé.");
    expect(put).toHaveBeenLastCalledWith("/tickets/t-1/reopen", {
      reason: "La fuite a recommencé.",
    });
  });

  it("@negative aucune des trois ne part avec un corps vide", async () => {
    const api = await ticketsApi();
    await api.resolve("t-1", "note");
    await api.cancel("t-1", "raison");
    await api.reopen("t-1", "raison");

    for (const appel of put.mock.calls) {
      const [route, corps] = appel as [string, Record<string, unknown>];
      expect(
        Object.keys(corps).length,
        `${route} part avec un corps vide : le serveur rendra 400 et le ` +
          "bouton ne fera rien, sans que l'écran l'explique.",
      ).toBeGreaterThan(0);
    }
  });

  it("@edge clôturer n'exige rien : c'est la seule des cinq sans corps", async () => {
    // `CloseTicketRequest` n'existe pas côté serveur. Cette transition
    // marchait déjà — mais on ne pouvait pas l'atteindre, puisqu'il faut
    // être `Resolved` pour clôturer et que `resolve` échouait.
    const api = await ticketsApi();
    await api.close("t-1");
    expect(put).toHaveBeenCalledWith("/tickets/t-1/close", {});
  });

  it("@edge démarrer vise `start-work`, la route que le serveur sert vraiment", async () => {
    // `/tickets/{id}/start` rend 404 : le backend déclare
    // `#[put("/tickets/{id}/start-work")]` (`ticket_handlers.rs:585`).
    // Aucun écran n'appelle cette méthode aujourd'hui — c'est du code mort
    // qui attendait son premier appelant pour devenir un défaut.
    const api = await ticketsApi();
    await api.start("t-1");
    expect(put).toHaveBeenCalledWith("/tickets/t-1/start-work", {});
  });
});
