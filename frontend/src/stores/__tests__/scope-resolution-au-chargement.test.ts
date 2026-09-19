import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  resoudrePerimetreAuChargement,
  getScope,
  resetScope,
  setAcp,
} from "../scope.svelte";
import type { Building } from "../../lib/types";

/**
 * L'orchestrateur qui manquait : les deux replis existent (`rehydraterDepuisLurl`,
 * `resoudreLeDefautServeur`), séparément testés, jamais composés. Story #841 exige
 * les TROIS branches à l'arrivée sur une page :
 *
 *   1. `?buildingId=` présent et accepté → il gagne.
 *   2. absent (ou présent et refusé) → le défaut serveur tente de combler.
 *   3. rien ne résout → périmètre nul, l'écran demande une sélection.
 *
 * Le point délicat de la composition n'est pas l'ordre des deux appels, c'est
 * le refus : un `?buildingId=` que le serveur rejette ne doit PAS retomber sur
 * le défaut serveur — ce serait un repli silencieux sur un immeuble que
 * personne n'a demandé, exactement ce que l'AC @negative interdit.
 */

const IMMEUBLE: Building = {
  id: "b-42",
  name: "Résidence des Érables",
  acp_id: "acp-7",
} as Building;

function poserLurl(recherche: string) {
  Object.defineProperty(window, "location", {
    writable: true,
    value: { ...window.location, search: recherche },
  });
}

beforeEach(() => {
  resetScope();
  vi.clearAllMocks();
});
afterEach(() => resetScope());

describe("@happy le lien profond accepté gagne, le défaut n'est même pas consulté", () => {
  it("adopte l'immeuble de l'URL sans interroger les ACP", async () => {
    poserLurl("?buildingId=b-42");
    const building = vi.fn().mockResolvedValue(IMMEUBLE);
    const acps = vi.fn();

    await resoudrePerimetreAuChargement({ building, acps });

    expect(getScope().selectedBuildingId).toBe("b-42");
    expect(acps).not.toHaveBeenCalled();
  });
});

describe("@edge les trois branches — URL, puis défaut, puis nul", () => {
  it("sans URL et une seule ACP accessible : le défaut serveur pose le périmètre", async () => {
    poserLurl("");
    const building = vi.fn();
    const acps = vi.fn().mockResolvedValue([{ id: "acp-7" }]);

    await resoudrePerimetreAuChargement({ building, acps });

    expect(building).not.toHaveBeenCalled();
    expect(getScope().selectedAcpId).toBe("acp-7");
  });

  it("sans URL et plusieurs ACP : rien n'est deviné, le périmètre reste nul", async () => {
    poserLurl("");
    const acps = vi.fn().mockResolvedValue([{ id: "acp-7" }, { id: "acp-9" }]);

    await resoudrePerimetreAuChargement({ building: vi.fn(), acps });

    expect(getScope().selectedAcpId).toBeNull();
    expect(getScope().acpsDisponibles).toBe(2);
  });

  it("un périmètre déjà posé avant l'appel n'est jamais écrasé par le défaut", async () => {
    poserLurl("");
    setAcp("acp-choisie-par-lutilisateur");
    const acps = vi.fn().mockResolvedValue([{ id: "acp-7" }]);

    await resoudrePerimetreAuChargement({ building: vi.fn(), acps });

    expect(getScope().selectedAcpId).toBe("acp-choisie-par-lutilisateur");
    expect(acps).not.toHaveBeenCalled();
  });
});

describe("@security un buildingId refusé ne retombe JAMAIS sur le défaut serveur", () => {
  it("laisse le périmètre nul et l'erreur posée, sans consulter les ACP", async () => {
    poserLurl("?buildingId=celui-dun-autre-cabinet");
    const building = vi.fn().mockRejectedValue({ status: 403 });
    const acps = vi.fn().mockResolvedValue([{ id: "acp-7" }]);

    await resoudrePerimetreAuChargement({ building, acps });

    expect(getScope().selectedBuildingId).toBeNull();
    expect(getScope().scopeError).toBe("forbidden");
    // Le repli silencieux est exactement ce que l'AC @security interdit : un
    // lien refusé ne doit pas discrètement atterrir sur l'ACP par défaut de
    // l'utilisateur, qui n'a rien demandé de tel.
    expect(acps).not.toHaveBeenCalled();
  });
});

describe("@negative un défaut serveur en échec n'invente pas un refus", () => {
  it("laisse le périmètre nul sans poser scopeError quand /acps échoue", async () => {
    poserLurl("");
    const acps = vi.fn().mockRejectedValue(new Error("réseau indisponible"));

    await resoudrePerimetreAuChargement({ building: vi.fn(), acps });

    expect(getScope().selectedAcpId).toBeNull();
    expect(getScope().scopeError).toBeNull();
  });
});
