// Story #798 — le périmètre applicatif bascule de l'immeuble vers l'ACP.
//
// CRITICAL §3 — RED-first TDD : tests rouges AVANT `demanderAcp`.
//
// ── Ce que ces tests tiennent ───────────────────────────────────────────────
//
// `setAcp()` existait déjà (utilisé par les pages ACP-niveau) : il pose
// `selectedAcpId` sans poser de condition, en faisant confiance à l'appelant.
// C'est correct pour un identifiant déjà connu du serveur (ex. la page
// `/acps/{id}` elle-même). Ce n'est PAS suffisant pour `AcpSelector`, qui doit
// tenir la garantie du critère `@negative` de la story #798 : une ACP hors du
// portefeuille de l'utilisateur, une fois demandée, retourne 403 — et le
// périmètre courant reste EXACTEMENT ce qu'il était avant la tentative.
//
// C'est différent de `rehydraterDepuisLurl` (#841) : celui-là répond à un
// premier chargement de page, où le périmètre part toujours de zéro — reset à
// null et "inchangé" sont alors la même chose. `demanderAcp` répond à une
// tentative EN COURS DE SESSION (clic, lien profond après coup...) : un refus
// ne doit pas effacer un périmètre qui fonctionnait déjà.
//
// ── Le second fait que ces tests tiennent ───────────────────────────────────
//
// Le fait juridique qui commande #798 : une ACP peut couvrir plusieurs blocs
// (immeubles), et le droit belge connaît les ACP principales et secondaires.
// La relation ACP -> immeuble n'est donc PAS 1:1. `setAcp()` le respecte déjà
// par ce qu'il NE FAIT PAS : contrairement à `setBuilding()` (qui dérive
// `selectedAcpId` de `building.acp_id`), `setAcp()` ne dérive et ne force
// AUCUN `selectedBuildingId` — l'immeuble reste un filtre secondaire,
// indépendant, qui peut rester vide (l'ACP entière) ou pointer vers N'IMPORTE
// LEQUEL des blocs de l'ACP.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  demanderAcp,
  setAcp,
  setBuilding,
  getScope,
  resetScope,
} from "../scope.svelte";
import type { Building } from "../../lib/types";

function immeuble(overrides: Partial<Building> = {}): Building {
  return {
    id: "b-1",
    acp_id: "acp-principale",
    name: "Bloc A",
    address: "1 Rue Test",
    city: "Bruxelles",
    postal_code: "1000",
    country: "Belgium",
    total_units: 10,
    total_tantiemes: 1000,
    ...overrides,
  };
}

beforeEach(() => {
  resetScope();
  vi.clearAllMocks();
});
afterEach(() => resetScope());

// ===========================================================================
// @happy — le serveur confirme l'ACP demandée
// ===========================================================================

describe("demanderAcp @happy", () => {
  it("pose selectedAcpId quand le serveur confirme l'ACP", async () => {
    const charger = vi.fn().mockResolvedValue({ id: "acp-1" });

    const ok = await demanderAcp("acp-1", charger);

    expect(ok).toBe(true);
    expect(charger).toHaveBeenCalledWith("acp-1");
    expect(getScope().selectedAcpId).toBe("acp-1");
    expect(getScope().scopeError).toBeNull();
  });
});

// ===========================================================================
// @edge — la relation ACP <-> immeuble n'est pas 1:1 (n:n via les lots)
// ===========================================================================

describe("demanderAcp @edge — relation ACP <-> immeuble n:n, pas 1:1", () => {
  it("poser une ACP ne force ni ne dérive aucun immeuble", async () => {
    const charger = vi.fn().mockResolvedValue({ id: "acp-principale" });

    await demanderAcp("acp-principale", charger);

    // Un champ unique fusionnant building+acp serait un modèle faux : l'ACP
    // peut couvrir plusieurs blocs (ACP principale + secondaires, droit
    // belge), donc choisir l'ACP ne peut PAS deviner un immeuble unique.
    expect(getScope().selectedBuildingId).toBeNull();
    expect(getScope().selectedBuilding).toBeNull();
  });

  it("un immeuble déjà sélectionné sous LA MÊME ACP survit au choix de l'ACP", async () => {
    setBuilding(immeuble({ id: "b-bloc-a", acp_id: "acp-principale" }));
    const charger = vi.fn().mockResolvedValue({ id: "acp-principale" });

    await demanderAcp("acp-principale", charger);

    // Le filtre immeuble est secondaire : il n'est pas balayé par la
    // confirmation d'une ACP qu'il désignait déjà.
    expect(getScope().selectedBuildingId).toBe("b-bloc-a");
  });

  it("un second bloc de la même ACP peut être choisi sans re-choisir l'ACP", () => {
    setBuilding(immeuble({ id: "b-bloc-a", acp_id: "acp-principale" }));
    expect(getScope().selectedAcpId).toBe("acp-principale");

    // Bloc B, même ACP (ACP principale couvrant plusieurs blocs) : l'ACP
    // sélectionnée ne bouge pas, la preuve que la relation est 1:n côté ACP,
    // jamais 1:1.
    setBuilding(immeuble({ id: "b-bloc-b", acp_id: "acp-principale" }));

    expect(getScope().selectedBuildingId).toBe("b-bloc-b");
    expect(getScope().selectedAcpId).toBe("acp-principale");
  });
});

// ===========================================================================
// @security — un refus ne doit fuiter aucune information de périmètre
// ===========================================================================

describe("demanderAcp @security", () => {
  it("un refus 403 laisse le périmètre PRÉCÉDENT intact (pas un reset)", async () => {
    setAcp("acp-legitime");
    const charger = vi.fn().mockRejectedValue({ status: 403 });

    const ok = await demanderAcp("acp-dun-autre-cabinet", charger);

    expect(ok).toBe(false);
    // Le point du test : contrairement à rehydraterDepuisLurl (premier
    // chargement, donc reset == inchangé), une tentative EN COURS DE SESSION
    // ne doit PAS effacer un périmètre qui fonctionnait déjà.
    expect(getScope().selectedAcpId).toBe("acp-legitime");
    expect(getScope().scopeError).toBe("forbidden");
  });

  it("un refus 403 sans périmètre préalable ne pose toujours pas l'ACP demandée", async () => {
    const charger = vi.fn().mockRejectedValue({ status: 403 });

    await demanderAcp("acp-hors-portefeuille", charger);

    expect(getScope().selectedAcpId).toBeNull();
  });
});

// ===========================================================================
// @negative — refus, erreurs réseau, distinctions d'erreur
// ===========================================================================

describe("demanderAcp @negative", () => {
  it("ACP hors du portefeuille -> 403, le périmètre n'est pas changé", async () => {
    setAcp("acp-courante");
    setBuilding(immeuble({ id: "b-courant", acp_id: "acp-courante" }));
    const charger = vi.fn().mockRejectedValue({ status: 403 });

    const ok = await demanderAcp("acp-etrangere", charger);

    expect(ok).toBe(false);
    expect(getScope().selectedAcpId).toBe("acp-courante");
    expect(getScope().selectedBuildingId).toBe("b-courant");
    expect(getScope().scopeError).toBe("forbidden");
  });

  it("distingue une ACP inexistante (404) d'une ACP interdite (403)", async () => {
    const charger = vi.fn().mockRejectedValue({ status: 404 });

    await demanderAcp("acp-inconnue", charger);

    expect(getScope().scopeError).toBe("not_found");
  });

  it("ne jette jamais — une erreur sans statut retombe en not_found", async () => {
    const charger = vi.fn().mockRejectedValue(new Error("boom"));

    await expect(demanderAcp("acp-x", charger)).resolves.toBe(false);
    expect(getScope().scopeError).toBe("not_found");
  });
});
