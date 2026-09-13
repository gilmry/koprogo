import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "../../../test-helpers";
import MenuMobile from "../MenuMobile.svelte";
import { resetScope, setAcp } from "../../../stores/scope.svelte";

vi.mock("../../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: unknown) => void) => {
      fn((cle: string) => cle);
      return () => {};
    },
  },
}));

const utilisateur = vi.hoisted(() => ({ courant: null as unknown }));

vi.mock("../../../stores/auth", () => ({
  authStore: {
    subscribe: (fn: (v: unknown) => void) => {
      fn({ user: utilisateur.courant });
      return () => {};
    },
  },
}));

/**
 * L'écran « Plus » de la barre d'onglets mobile.
 *
 * ── Ce que ces tests tiennent ────────────────────────────────────────────
 *
 * Cet écran est le SEUL chemin vers ce que la barre du bas ne montre pas.
 * Trois façons de le casser, et chacune a une conséquence distincte :
 *
 * 1. **Proposer une destination interdite au rôle.** `RouteGuard` redirige en
 *    SILENCE : l'utilisateur tape, revient d'où il vient, sans message. Le
 *    lien lit comme cassé. Les groupes passent donc par `canSee`.
 * 2. **Répéter ce qui est déjà dans la barre du bas.** Un écran « Plus » qui
 *    montre les mêmes quatre entrées que les onglets ne sert à rien, et fait
 *    douter d'avoir bien tapé.
 * 3. **Rester vide sans le dire.** Un écran blanc se lit comme une panne. Il
 *    doit distinguer « rien de plus à montrer » de « quelque chose a échoué ».
 *
 * ── Le piège de test que la remise signale ──────────────────────────────
 *
 * Les ancrages de cet écran sont en `menu-mobile-*`. Employer
 * `navigation-menu-*` — ceux du sidebar — fausserait tout compte de menus
 * métier, puisque les deux coexistent dans le même document sur mobile.
 */

function connecter(role: string | null) {
  utilisateur.courant = role
    ? {
        id: "u-1",
        first_name: "Ana",
        last_name: "Dubois",
        role,
        email: "a@b.be",
      }
    : null;
}

beforeEach(() => {
  resetScope();
  connecter(null);
});

describe("@happy l'écran Plus montre ce que la barre du bas ne montre pas", () => {
  it("propose au syndic ses groupes métier", () => {
    connecter("syndic");
    render(MenuMobile);

    // `gestion` et `compta` ne sont pas dans les quatre onglets du syndic
    // (Aujourd'hui, ACP, Charges, AG) : ils doivent apparaître ici.
    expect(screen.queryByTestId("menu-mobile-gestion")).toBeTruthy();
    expect(screen.queryByTestId("menu-mobile-compta")).toBeTruthy();
  });

  it("met le compte en tête, avec ses initiales", () => {
    // C'est ce qu'on vient chercher le plus souvent dans un écran « Plus » ;
    // le reléguer en bas obligerait à faire défiler pour se déconnecter.
    connecter("syndic");
    render(MenuMobile);

    const compte = screen.getByTestId("menu-mobile-compte");
    expect(compte).toHaveAttribute("href", "/profile");
    expect(compte.textContent).toContain("AD");
  });
});

describe("@edge aucun groupe autorisé ne disparaît de l'écran", () => {
  it("montre TOUS les groupes que le rôle peut ouvrir", () => {
    // ── Ce que ce test empêche, et que j'ai commis ─────────────────────
    //
    // J'avais d'abord masqué les groupes dont la destination figure déjà
    // dans la barre du bas. Le groupe « Comptabilité » du syndic pointe vers
    // `/expenses`, qui est aussi son onglet « Charges » : le filtre le
    // masquait donc, et privait le syndic de CINQ destinations —
    // `/invoice-workflow`, `/budgets`, `/etats-dates`, `/journal-entries`,
    // `/reports`.
    //
    // Un groupe contient plusieurs pages ; le faire pointer vers la première
    // est un raccourci, et masquer le groupe entier sur cette base est une
    // faute. Mieux vaut un doublon apparent qu'une destination
    // inatteignable.
    connecter("syndic");
    render(MenuMobile);

    for (const groupe of [
      "gestion",
      "compta",
      "gouvernance",
      "ticketing",
      "communaute",
    ]) {
      expect(
        screen.queryByTestId(`menu-mobile-${groupe}`),
        `le groupe ${groupe} manque à l'écran « Plus » du syndic`,
      ).toBeTruthy();
    }
  });
});

describe("@security l'écran ne propose rien que le rôle ne puisse ouvrir", () => {
  it("ne montre pas le menu admin à un syndic", () => {
    connecter("syndic");
    render(MenuMobile);

    // `canSee` réserve `admin` au superadmin. Le proposer ici ferait taper
    // sur un lien que `RouteGuard` refuserait en silence.
    expect(screen.queryByTestId("menu-mobile-admin")).toBeNull();
  });

  it("ne montre pas les groupes métier à un administrateur hors contexte", () => {
    // Superadmin sans périmètre : `canSee` lui donne le menu admin et lui
    // retire les menus métier. C'est le mode « in-context » de
    // `permissions.ts`, et l'écran doit le respecter comme le sidebar.
    connecter("superadmin");
    render(MenuMobile);

    expect(screen.queryByTestId("menu-mobile-admin")).toBeTruthy();
    expect(screen.queryByTestId("menu-mobile-gestion")).toBeNull();
  });

  it("bascule les menus métier quand une ACP est choisie", () => {
    // Et l'inverse : avec un périmètre posé, l'administrateur passe en mode
    // métier. C'est la bascule que le passage du périmètre à l'ACP a rendue
    // possible — avant, il fallait descendre à un immeuble.
    connecter("superadmin");
    setAcp("acp-1");
    render(MenuMobile);

    expect(screen.queryByTestId("menu-mobile-gestion")).toBeTruthy();
    expect(screen.queryByTestId("menu-mobile-admin")).toBeNull();
  });
});

describe("@negative un écran vide se dit, il ne se subit pas", () => {
  it("annonce qu'il n'y a rien de plus quand aucun groupe n'est visible", () => {
    // Rôle inconnu : `canSee` est fail-closed, aucun groupe ne passe. L'écran
    // doit LE DIRE — un blanc se lit comme une panne.
    connecter("role_qui_nexiste_pas");
    render(MenuMobile);

    expect(screen.queryByTestId("menu-mobile-vide")).toBeTruthy();
  });
});

describe("les ancrages n'empiètent pas sur ceux du sidebar", () => {
  it("n'emploie aucun ancrage en navigation-menu-*", () => {
    connecter("syndic");
    const { container } = render(MenuMobile);

    // Les deux composants coexistent dans le même document sur mobile :
    // dupliquer les ancrages du sidebar fausserait tout compte de menus.
    expect(
      container.querySelectorAll('[data-testid^="navigation-menu-"]').length,
    ).toBe(0);
    expect(
      container.querySelectorAll('[data-testid^="menu-mobile-"]').length,
    ).toBeGreaterThan(0);
  });
});
