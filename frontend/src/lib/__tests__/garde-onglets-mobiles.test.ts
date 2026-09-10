import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { ongletsPour, rolesAvecOnglets } from "../onglets-mobiles";
import { iconeConnue } from "../icones";
import { UserRole } from "../types";

/**
 * Les onglets du bas ne mènent nulle part d'interdit, et ne cassent aucun
 * compte de menus.
 *
 * ── Le premier risque : un onglet qui rend d'où l'on vient ──────────────
 *
 * `RouteGuard.svelte` redirige **en silence** quand `guards.ts` refuse une
 * route au rôle courant : l'utilisateur tape, et revient d'où il venait, sans
 * message, sans erreur. Le bouton lit comme cassé.
 *
 * C'est exactement le défaut que `garde-tuiles-interdites` traque sur les
 * quatre tableaux de bord. Une barre d'onglets est bien pire : elle est
 * **permanente et à portée de pouce**, donc un onglet mort se tape plusieurs
 * fois par session.
 *
 * ── Le second risque : casser un compte de menus ────────────────────────
 *
 * `Navigation.test.ts @happy` affirme que le syndic voit **exactement cinq**
 * menus métier. La remise de design met en garde : les entrées épinglées ne
 * doivent pas recevoir d'ancrage en `navigation-menu-*`, ou ce compte tombe.
 * La même règle vaut pour les onglets, qui ont leur espace de noms
 * `tabbar-{cle}`.
 *
 * Cette garde le vérifie sur la SOURCE du composant, pas sur son rendu : un
 * ancrage fautif dans une branche rarement prise échapperait au rendu.
 */

const RACINE = process.cwd();

/** Les routes gardées, telles que `guards.ts` les déclare. */
function routesGardees(): Map<string, Set<string>> {
  const source = readFileSync(join(RACINE, "src/lib/guards.ts"), "utf-8");
  const gardes = new Map<string, Set<string>>();
  for (const m of source.matchAll(/"([^"]+)":\s*\[([^\]]*)\]/g)) {
    const roles = new Set(
      [...m[2].matchAll(/UserRole\.(\w+)/g)].map((r) => r[1]),
    );
    if (roles.size > 0) gardes.set(m[1], roles);
  }
  return gardes;
}

/**
 * Le nom du membre d'énumération pour une valeur de rôle.
 *
 * `guards.ts` désigne les rôles par `UserRole.SYNDIC` ; le code, par la
 * valeur `"syndic"`. Les deux doivent être ramenés au même repère avant
 * d'être comparés.
 */
function nomDuMembre(valeur: string): string {
  const trouve = Object.entries(UserRole).find(([, v]) => v === valeur);
  return trouve ? trouve[0] : valeur;
}

describe("les onglets mobiles ne mènent nulle part d'interdit", () => {
  it("ne propose à aucun rôle une route que guards.ts lui refuse", () => {
    const gardes = routesGardees();
    const fautifs: string[] = [];

    for (const role of rolesAvecOnglets()) {
      // `guards.ts` s'écrit `UserRole.SYNDIC` : le motif en extrait le NOM du
      // membre, pas sa valeur. `rolesAvecOnglets()` rend les valeurs
      // (« syndic »). Comparer les deux tels quels donnait trois faux
      // positifs sur des routes parfaitement autorisées — et c'est la
      // vérification d'aveuglement qui l'a montré, pas la lecture.
      const nom = nomDuMembre(role);
      for (const onglet of ongletsPour(role)) {
        const route = onglet.href.replace(/\/+$/, "") || "/";
        const autorises = gardes.get(route);
        if (autorises && !autorises.has(nom)) {
          fautifs.push(
            `  ${role} → ${onglet.cle} (${route}) — réservé à ` +
              `${[...autorises].sort().join(", ")}`,
          );
        }
      }
    }

    expect(
      fautifs.join("\n"),
      "Ces onglets mènent à une route que `guards.ts` interdit au rôle. " +
        "`RouteGuard` redirige en silence : l'utilisateur tape et revient " +
        "d'où il vient, sans message.\n\n" +
        "Une barre d'onglets est permanente et à portée de pouce — un " +
        "onglet mort se tape plusieurs fois par session.",
    ).toBe("");
  });

  it("lit bien guards.ts, et ne juge pas sur une table vide", () => {
    // Vérification d'aveuglement : si le format de `guards.ts` changeait, la
    // table serait vide et le test ci-dessus passerait sans rien vérifier.
    const gardes = routesGardees();
    expect(gardes.size).toBeGreaterThan(10);

    // Et la traduction valeur → nom doit fonctionner, sans quoi TOUTE route
    // gardée paraîtrait interdite à tout le monde.
    expect(nomDuMembre("syndic")).toBe("SYNDIC");
    expect(nomDuMembre("owner")).toBe("OWNER");
  });
});

describe("les onglets ne cassent aucun contrat de test existant", () => {
  it("n'emploie aucun ancrage en navigation-menu-*", () => {
    const source = readFileSync(
      join(RACINE, "src/components/navigation/TabBarMobile.svelte"),
      "utf-8",
    );

    const fautifs = [
      ...source.matchAll(/data-testid="(navigation-menu[^"]*)"/g),
    ]
      .map((m) => `  ${m[1]}`)
      .join("\n");

    expect(
      fautifs,
      "`Navigation.test.ts @happy` compte EXACTEMENT cinq menus métier " +
        "pour le syndic. Un onglet portant `navigation-menu-*` s'ajouterait " +
        "à ce compte et le ferait tomber.\n\n" +
        "Les onglets ont leur espace de noms : `tabbar-{cle}`.",
    ).toBe("");
  });

  it("donne un ancrage tabbar-* à chaque onglet, et une icône qui existe", () => {
    const source = readFileSync(
      join(RACINE, "src/components/navigation/TabBarMobile.svelte"),
      "utf-8",
    );
    expect(source).toContain('data-testid="tabbar-{onglet.cle}"');

    // Une icône absente du jeu ne rend RIEN, en silence : l'onglet perdrait
    // son repère et il ne resterait qu'un libellé de 10,5 px.
    const inconnues: string[] = [];
    for (const role of rolesAvecOnglets()) {
      for (const onglet of ongletsPour(role)) {
        if (!iconeConnue(onglet.icone)) {
          inconnues.push(`  ${role} → ${onglet.cle} : « ${onglet.icone} »`);
        }
      }
    }
    expect(inconnues.join("\n")).toBe("");
  });

  it("tient cinq onglets par rôle, pas davantage", () => {
    // Cinq est le maximum tenable à 390 px sans descendre sous la cible de
    // 44 px. Un sixième n'ajouterait pas une destination : il rendrait les
    // six difficiles à viser.
    for (const role of rolesAvecOnglets()) {
      expect(ongletsPour(role).length, `rôle ${role}`).toBeLessThanOrEqual(5);
    }
  });

  it("ne montre aucune barre à un rôle inconnu", () => {
    // Fail-closed : une barre générique proposerait des destinations que
    // `RouteGuard` refuserait en silence.
    expect(ongletsPour(null)).toEqual([]);
    expect(ongletsPour(undefined)).toEqual([]);
    expect(ongletsPour("")).toEqual([]);
    expect(ongletsPour("ROLE_QUI_NEXISTE_PAS")).toEqual([]);
  });
});
