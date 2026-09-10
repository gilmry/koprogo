import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent } from "../../../test-helpers";
import FeuilleDuBas from "../FeuilleDuBas.svelte";

vi.mock("../../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: unknown) => void) => {
      fn((cle: string) => cle);
      return () => {};
    },
  },
}));

/**
 * La feuille du bas.
 *
 * ── Ce que ces tests tiennent ────────────────────────────────────────────
 *
 * Quatre propriétés qu'on oublie systématiquement en écrivant un panneau
 * modal, et dont chacune a une conséquence concrète :
 *
 * 1. **Le défilement du corps est verrouillé.** Sans cela, glisser dans la
 *    feuille fait défiler la page derrière : on perd sa place, et à la
 *    fermeture on ne reconnaît plus l'écran.
 * 2. **Il est RESTAURÉ à l'identique**, pas mis à « visible ». Écraser une
 *    valeur qu'on n'a pas posée casserait une page qui gérait son propre
 *    défilement.
 * 3. **Échap ferme.** C'est la sortie que tout le monde essaie en premier,
 *    et son absence enferme.
 * 4. **Le voile est un vrai bouton.** Un `<div role="button">` n'est pas
 *    atteignable au clavier : la seule façon de fermer serait le bouton de
 *    l'en-tête, et un utilisateur au clavier qui ne le trouve pas est
 *    coincé.
 */

beforeEach(() => {
  document.body.style.overflow = "";
});

afterEach(() => {
  document.body.style.overflow = "";
});

describe("@happy la feuille s'ouvre et se ferme", () => {
  it("ne rend rien tant qu'elle est fermée", () => {
    render(FeuilleDuBas, {
      props: { ouverte: false, titre: "Choisir une ACP", onfermer: () => {} },
    });
    expect(screen.queryByTestId("feuille-du-bas")).toBeNull();
  });

  it("annonce son titre comme dialogue modal", () => {
    render(FeuilleDuBas, {
      props: { ouverte: true, titre: "Choisir une ACP", onfermer: () => {} },
    });

    const feuille = screen.getByTestId("feuille-du-bas");
    expect(feuille).toHaveAttribute("role", "dialog");
    expect(feuille).toHaveAttribute("aria-modal", "true");
    expect(feuille).toHaveAttribute("aria-label", "Choisir une ACP");
  });
});

describe("@edge le défilement du corps", () => {
  it("verrouille le défilement à l'ouverture", () => {
    render(FeuilleDuBas, {
      props: { ouverte: true, titre: "Titre", onfermer: () => {} },
    });
    expect(document.body.style.overflow).toBe("hidden");
  });

  it("restaure la valeur d'AVANT, pas « visible »", async () => {
    // Une page qui gère son propre défilement — un plan, une liste virtuelle —
    // aurait sa valeur écrasée si on remettait « visible » à l'aveugle.
    document.body.style.overflow = "scroll";

    const { unmount } = render(FeuilleDuBas, {
      props: { ouverte: true, titre: "Titre", onfermer: () => {} },
    });
    expect(document.body.style.overflow).toBe("hidden");

    unmount();
    expect(document.body.style.overflow).toBe("scroll");
  });
});

describe("@happy les deux sorties", () => {
  it("ferme sur Échap", async () => {
    // La sortie que tout le monde essaie en premier.
    const fermer = vi.fn();
    render(FeuilleDuBas, {
      props: { ouverte: true, titre: "Titre", onfermer: fermer },
    });

    await fireEvent.keyDown(document, { key: "Escape" });
    expect(fermer).toHaveBeenCalled();
  });

  it("ferme au clic sur le voile, qui est un vrai bouton", async () => {
    const fermer = vi.fn();
    render(FeuilleDuBas, {
      props: { ouverte: true, titre: "Titre", onfermer: fermer },
    });

    const voile = screen.getByTestId("feuille-du-bas-voile");
    // `<button>` et non `<div role="button">` : un div n'est pas atteignable
    // au clavier, et un utilisateur qui ne trouve pas le bouton de l'en-tête
    // serait coincé.
    expect(voile.tagName.toLowerCase()).toBe("button");

    await fireEvent.click(voile);
    expect(fermer).toHaveBeenCalled();
  });
});

describe("@edge la cible de fermeture est atteignable au pouce", () => {
  it("donne 44 px au bouton de fermeture", () => {
    // Un bouton de fermeture qu'on manque enferme dans le panneau. 44 px est
    // le minimum sur mobile ; `h-11 w-11` vaut exactement 44 px en Tailwind.
    render(FeuilleDuBas, {
      props: { ouverte: true, titre: "Titre", onfermer: () => {} },
    });

    const bouton = screen.getByTestId("feuille-du-bas-fermer");
    expect(bouton.className).toContain("h-11");
    expect(bouton.className).toContain("w-11");
  });
});
