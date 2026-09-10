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

  it("les entrées épinglées de la colonne latérale ne sont pas des menus", () => {
    // La remise de design met en garde : les deux entrées épinglées
    // — « Aujourd'hui » et la collection principale du rôle — ne doivent pas
    // porter d'ancrage `navigation-menu-*`, sous peine de fausser tout compte
    // de menus métier.
    //
    // Ce dépôt n'a pas d'assertion de compte stricte aujourd'hui, et c'est
    // précisément pour ça que cette garde existe : le jour où quelqu'un en
    // écrit une, elle doit compter des menus, pas des raccourcis.
    //
    // Un ancrage dit ce qu'une chose EST. Une entrée épinglée n'est pas un
    // groupe.
    const source = readFileSync(
      join(RACINE, "src/components/navigation/Navigation.svelte"),
      "utf-8",
    );

    for (const ancrage of ["navigation-link-today", "navigation-link-acps"]) {
      expect(source, `${ancrage} manque`).toContain(`data-testid="${ancrage}"`);
    }

    // Et aucun ancrage de menu ne doit vivre dans le bloc épinglé.
    const debut = source.indexOf('data-testid="navigation-link-today"');
    const fin = source.indexOf('{#if see("admin")}', debut);
    expect(debut).toBeGreaterThan(-1);
    expect(fin).toBeGreaterThan(debut);
    expect(
      source.slice(debut, fin),
      "Une entrée épinglée porte un ancrage `navigation-menu-*` : elle " +
        "serait comptée comme un menu métier.",
    ).not.toContain("navigation-menu-");
  });

  it("aucune entrée épinglée ne double un menu du même rôle", () => {
    // ── Le défaut que ce test empêche, et que j'ai commis ────────────────
    //
    // La remise n'épingle qu'une entrée, et pour le SYNDIC seulement :
    // « Mes ACP ». J'avais étendu le motif aux quatre rôles, et les QUATRE
    // entrées se sont révélées être des doublons de leur propre menu — même
    // destination, même libellé, deux fois dans la même navigation.
    //
    // Une seule a fait échouer la CI : `AdminDashBoard.improved.spec.ts`
    // clique « Organisations » par son nom de lien et en a trouvé deux
    // (« strict mode violation »). Les trois autres attendaient leur tour,
    // faute d'une recette qui clique par nom.
    //
    // Un lien en double n'est pas qu'un ennui de recette : un lecteur d'écran
    // annonce deux fois la même destination, et l'utilisateur ne sait pas
    // laquelle prendre.
    const source = readFileSync(
      join(RACINE, "src/components/navigation/Navigation.svelte"),
      "utf-8",
    );

    // Les destinations épinglées : `libelle:` les distingue des menus, qui
    // emploient `label:`.
    const epinglees = [
      ...source.matchAll(
        /href:\s*"([^"]+)",\s*\n\s*icone:\s*"[^"]+",\s*\n\s*libelle:/g,
      ),
    ].map((m) => m[1]);

    // Les destinations des menus MÉTIER, hors `getAdminItems` : ce dernier est
    // réservé au superadmin par `canSee("admin", …)`, et le syndic ne le voit
    // pas — c'est ce qui rend « Mes ACP » légitime pour lui.
    const finDesMetiers = source.indexOf("const getAdminItems");
    const metiers = new Set(
      [
        ...source
          .slice(0, finDesMetiers > 0 ? finDesMetiers : undefined)
          .matchAll(/\{\s*href:\s*"([^"]+)",\s*label:\s*t\(/g),
      ].map((m) => m[1]),
    );

    const doublons = epinglees.filter((h) => metiers.has(h));
    expect(
      doublons.join("\n"),
      "Ces entrées épinglées visent la même destination qu'un menu métier " +
        "du même rôle. Le lien apparaît deux fois dans la navigation : un " +
        "lecteur d'écran l'annonce deux fois, et `getByRole('link', { name })` " +
        "lève une « strict mode violation ».",
    ).toBe("");

    // Vérification d'aveuglement : si les motifs cessaient de correspondre,
    // les deux listes seraient vides et le test passerait sans rien comparer.
    expect(epinglees.length).toBeGreaterThan(0);
    expect(metiers.size).toBeGreaterThan(10);
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
