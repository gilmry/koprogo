import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { rehydraterDepuisLurl, getScope, resetScope } from "../scope.svelte";
import type { Building } from "../../lib/types";

/**
 * Le périmètre se réhydrate depuis l'URL, et le serveur en décide.
 *
 * ── Le défaut ─────────────────────────────────────────────────────────────
 *
 * `scope.svelte.ts` annonce en tête que le périmètre « est dérivable d'un
 * deep-link (?buildingId=...) ou d'un défaut serveur », et que « le rehydrate
 * sur reload sera porté par Story 2.5 ». **Aucun des deux n'existait.**
 *
 * Or le frontend est une application Astro MULTI-PAGE : chaque navigation est
 * un chargement de document complet, et un `$state` de module repart à zéro.
 * Le périmètre était donc nul au premier rendu de chaque page, pour les douze
 * composants qui le lisent.
 *
 * Quatre échecs Playwright venaient de là, et ils avaient été classés
 * « défaut de fixture ». C'en était un — les tests naviguent directement sans
 * cliquer le sélecteur. Mais **un utilisateur qui ouvre un lien fait
 * exactement la même chose**, et obtient le même écran vide. Cf. #841.
 *
 * ── Ce que ces tests tiennent ─────────────────────────────────────────────
 *
 * Le point délicat n'est pas de lire l'URL, c'est de ne pas la croire.
 * L'en-tête du store refuse la persistance parce qu'elle « crée un risque de
 * scope violation post-rotation d'organisation ». Le troisième test est celui
 * qui garde cette propriété : un identifiant refusé par le serveur ne doit
 * RIEN laisser dans le périmètre.
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

describe("la réhydratation du périmètre depuis l'URL (#841)", () => {
  it("ne fait rien quand l'URL ne désigne aucun immeuble", async () => {
    poserLurl("");
    const charger = vi.fn();

    const adopte = await rehydraterDepuisLurl(charger);

    expect(adopte).toBeNull();
    expect(charger).not.toHaveBeenCalled();
    expect(getScope().selectedBuildingId).toBeNull();
  });

  it("adopte l'immeuble que le serveur accepte, et son ACP", async () => {
    poserLurl("?buildingId=b-42");
    const charger = vi.fn().mockResolvedValue(IMMEUBLE);

    const adopte = await rehydraterDepuisLurl(charger);

    expect(charger).toHaveBeenCalledWith("b-42");
    expect(adopte).toEqual(IMMEUBLE);
    expect(getScope().selectedBuildingId).toBe("b-42");
    expect(getScope().selectedAcpId).toBe("acp-7");
  });

  /**
   * LE test. C'est la propriété de sécurité que l'en-tête du store défend, et
   * la raison pour laquelle on ne persiste pas le périmètre : l'identifiant
   * vient de l'extérieur, il n'est jamais cru sur parole.
   */
  it("ne garde rien quand le serveur refuse l'immeuble", async () => {
    poserLurl("?buildingId=celui-dun-autre-cabinet");
    const charger = vi.fn().mockRejectedValue({ status: 403 });

    const adopte = await rehydraterDepuisLurl(charger);

    expect(adopte).toBeNull();
    expect(getScope().selectedBuildingId).toBeNull();
    expect(getScope().selectedBuilding).toBeNull();
    expect(getScope().scopeError).toBe("forbidden");
  });

  it("distingue un immeuble inexistant d'un immeuble interdit", async () => {
    poserLurl("?buildingId=b-inconnu");
    const charger = vi.fn().mockRejectedValue({ status: 404 });

    await rehydraterDepuisLurl(charger);

    expect(getScope().scopeError).toBe("not_found");
  });

  /**
   * `tickets.astro` et `tickets/new.astro` emploient déjà `building_id`. Des
   * liens circulent peut-être sous cette forme : les casser en imposant la
   * forme canonique serait gratuit.
   */
  it("accepte aussi la forme building_id, déjà employée par les tickets", async () => {
    poserLurl("?building_id=b-42");
    const charger = vi.fn().mockResolvedValue(IMMEUBLE);

    await rehydraterDepuisLurl(charger);

    expect(charger).toHaveBeenCalledWith("b-42");
    expect(getScope().selectedBuildingId).toBe("b-42");
  });
});
