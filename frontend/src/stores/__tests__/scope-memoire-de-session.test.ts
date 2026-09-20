/**
 * Le périmètre survit à une navigation — sans jamais être cru sur parole.
 *
 * ── Le défaut figé ici ────────────────────────────────────────────────────
 *
 * Signalé par le PO le 2026-09-20, dans ses mots : « quand on sélectionne une
 * ACP, dès qu'on appuie sur un bouton ou qu'on va dans un menu il faut
 * resélectionner ».
 *
 * Le frontend est une application Astro MULTI-PAGE : chaque clic de menu
 * recharge le document, et le `$state` de module repart à zéro. Deux replis
 * existaient, aucun ne couvrait ce cas :
 *
 *   - le lien profond `?buildingId=` répond à « j'arrive par une URL », pas à
 *     « je clique Dépenses » — ce lien ne porte aucune chaîne de requête ;
 *   - le défaut serveur n'adopte une ACP que si l'utilisateur en a
 *     **exactement une** (`resoudreLeDefautServeur`).
 *
 * Un syndic multi-ACP — la prémisse du produit — repartait donc sans
 * périmètre à chaque navigation (#841).
 *
 * ── Ce que ces tests refusent de laisser passer ───────────────────────────
 *
 * La correction pose une mémoire de session. L'en-tête de `scope.svelte.ts`
 * refuse la persistance pour une raison de sécurité qui reste valable : une
 * ACP mémorisée ne doit JAMAIS être adoptée sans que le serveur la confirme.
 * Les cas négatifs ci-dessous valent autant que le cas nominal.
 */
import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  setAcp,
  resetScope,
  getScope,
  reprendreLeChoixDeLaSession,
  resoudrePerimetreAuChargement,
} from "../scope.svelte";

const ACP_A = "aaaaaaaa-1111-4111-8111-aaaaaaaaaaaa";
const ACP_B = "bbbbbbbb-2222-4222-8222-bbbbbbbbbbbb";

beforeEach(() => {
  resetScope();
  window.sessionStorage.clear();
});

describe("Le périmètre survit à une navigation (#841)", () => {
  it("@happy reprend l'ACP choisie quand le serveur la reconnaît encore", async () => {
    setAcp(ACP_A);
    // Ce que fait une navigation : le module est rechargé, l'état en mémoire
    // disparaît. Seule la session survit.
    resetScope();
    window.sessionStorage.setItem("koprogo_perimetre_acp", ACP_A);
    expect(getScope().selectedAcpId).toBeNull();

    const repris = await reprendreLeChoixDeLaSession(async () => [
      { id: ACP_A },
      { id: ACP_B },
    ]);

    expect(repris).toBe(ACP_A);
    expect(getScope().selectedAcpId).toBe(ACP_A);
  });

  it("@security n'adopte PAS une ACP que le serveur ne sert plus, et l'oublie", async () => {
    window.sessionStorage.setItem("koprogo_perimetre_acp", ACP_A);

    // Rotation d'organisation, mandat clos, droit retiré : le serveur ne
    // renvoie plus cette ACP.
    const repris = await reprendreLeChoixDeLaSession(async () => [
      { id: ACP_B },
    ]);

    expect(repris).toBeNull();
    expect(getScope().selectedAcpId).toBeNull();
    expect(
      window.sessionStorage.getItem("koprogo_perimetre_acp"),
      "Une mémoire périmée qui survit resurgit au chargement suivant.",
    ).toBeNull();
  });

  it("@edge un serveur injoignable ne fait pas oublier le choix", async () => {
    window.sessionStorage.setItem("koprogo_perimetre_acp", ACP_A);

    const repris = await reprendreLeChoixDeLaSession(async () => {
      throw new Error("réseau");
    });

    expect(repris).toBeNull();
    expect(
      window.sessionStorage.getItem("koprogo_perimetre_acp"),
      "Une panne de réseau a effacé un choix légitime.",
    ).toBe(ACP_A);
  });

  it("@security une déconnexion efface la mémoire", () => {
    setAcp(ACP_A);
    expect(window.sessionStorage.getItem("koprogo_perimetre_acp")).toBe(ACP_A);

    resetScope();

    expect(window.sessionStorage.getItem("koprogo_perimetre_acp")).toBeNull();
  });

  it("@edge le souvenir ne prime pas sur ce que l'URL demande", async () => {
    window.sessionStorage.setItem("koprogo_perimetre_acp", ACP_A);
    const immeuble = {
      id: "cccccccc-3333-4333-8333-cccccccccccc",
      acp_id: ACP_B,
      name: "Résidence B",
    };
    window.history.replaceState({}, "", `/?buildingId=${immeuble.id}`);

    const acps = vi.fn(async () => [{ id: ACP_A }, { id: ACP_B }]);
    await resoudrePerimetreAuChargement({
      building: async () => immeuble as never,
      acps,
    });

    expect(
      getScope().selectedAcpId,
      "Le souvenir a écrasé un lien profond explicite.",
    ).toBe(ACP_B);
    expect(
      acps,
      "La liste des ACP a été demandée alors que l'URL suffisait.",
    ).not.toHaveBeenCalled();

    window.history.replaceState({}, "", "/");
  });
});
