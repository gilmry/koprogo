import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  resoudreLeDefautServeur,
  getScope,
  resetScope,
  setAcp,
} from "../scope.svelte";

/**
 * Le périmètre a un défaut, et c'est le serveur qui le donne.
 *
 * ── Ce que ces tests tiennent ────────────────────────────────────────────
 *
 * Le défaut serveur a été retenu **avant** le lien profond, le 2026-09-10,
 * pour une raison précise : il ne peut pas boucler. Il ne lit rien de ce que
 * l'utilisateur contrôle. Le lien profond, lui, avait cassé trois recettes
 * Playwright avec un `networkidle` qui n'arrivait jamais — une activité
 * réseau qui ne s'arrêtait plus.
 *
 * Les deux propriétés qui comptent, et que la lecture du code ne suffit pas
 * à garantir :
 *
 * 1. **Le défaut comble une absence, il n'arbitre pas.** Un périmètre déjà
 *    posé — par lien profond ou par clic — n'est jamais écrasé.
 * 2. **On ne devine pas entre plusieurs ACP.** Deviner ferait travailler un
 *    syndic dans la mauvaise copropriété sans qu'il l'ait demandé, et les
 *    écritures qu'il y passerait seraient imputées à la mauvaise personne
 *    morale. Ce n'est pas une gêne d'ergonomie, c'est une faute comptable.
 */

const UNE = [{ id: "acp-7" }];
const PLUSIEURS = [{ id: "acp-7" }, { id: "acp-9" }, { id: "acp-11" }];

beforeEach(() => {
  resetScope();
});

describe("@happy le défaut serveur pose le périmètre quand il n'y a qu'une ACP", () => {
  it("adopte l'unique ACP accessible", async () => {
    const adopte = await resoudreLeDefautServeur(async () => UNE);

    expect(adopte).toBe("acp-7");
    expect(getScope().selectedAcpId).toBe("acp-7");
  });

  it("retient combien d'ACP sont accessibles, pour que l'écran sache s'il doit demander", async () => {
    await resoudreLeDefautServeur(async () => PLUSIEURS);

    // Ce n'est pas une donnée d'affichage : c'est ce qui décide si
    // l'affordance de choix doit exister. « A single choice is not a menu. »
    expect(getScope().acpsDisponibles).toBe(3);
  });
});

describe("@edge le défaut ne tranche pas ce qui revient à l'utilisateur", () => {
  it("ne choisit rien quand plusieurs ACP sont accessibles", async () => {
    const adopte = await resoudreLeDefautServeur(async () => PLUSIEURS);

    expect(adopte).toBeNull();
    expect(getScope().selectedAcpId).toBeNull();
  });

  it("ne choisit rien quand aucune ACP n'est accessible", async () => {
    const adopte = await resoudreLeDefautServeur(async () => []);

    expect(adopte).toBeNull();
    expect(getScope().selectedAcpId).toBeNull();
    expect(getScope().acpsDisponibles).toBe(0);
  });
});

describe("@security le défaut n'écrase jamais un périmètre déjà posé", () => {
  it("laisse intact l'ACP choisie, même s'il n'y en avait qu'une autre", async () => {
    // L'utilisateur a déjà désigné son périmètre — par un lien profond validé
    // par le serveur, ou par un clic dans le sélecteur.
    setAcp("acp-choisie-par-lutilisateur");

    const charger = vi.fn(async () => UNE);
    const adopte = await resoudreLeDefautServeur(charger);

    expect(adopte).toBe("acp-choisie-par-lutilisateur");
    expect(getScope().selectedAcpId).toBe("acp-choisie-par-lutilisateur");
    // Et on n'a même pas interrogé le serveur : rien à résoudre.
    expect(charger).not.toHaveBeenCalled();
  });
});

describe("@negative un défaut irrésolu n'est pas un refus de périmètre", () => {
  it("ne pose aucune erreur de périmètre quand le chargement échoue", async () => {
    const adopte = await resoudreLeDefautServeur(async () => {
      throw new Error("réseau indisponible");
    });

    expect(adopte).toBeNull();
    // `scopeError` fait afficher « vous n'avez pas accès à cet immeuble ».
    // Le poser ici accuserait l'utilisateur d'un refus qui n'a pas eu lieu :
    // il n'a rien demandé, c'est nous qui n'avons pas pu répondre.
    expect(getScope().scopeError).toBeNull();
  });

  it("ne laisse pas l'échec remonter à l'appelant", async () => {
    // Un défaut de périmètre non résolu ne doit pas empêcher la page de se
    // rendre : l'écran demandera, et c'est un état légitime.
    await expect(
      resoudreLeDefautServeur(async () => {
        throw new Error("500");
      }),
    ).resolves.toBeNull();
  });
});
