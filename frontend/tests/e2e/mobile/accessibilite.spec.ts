import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { ongletsPour } from "../../../src/lib/onglets-mobiles";
import { ACCUEIL, ouvreEnTantQue, ROLES, type Role } from "./socle";

/**
 * L'audit d'accessibilité, sur des écrans AUTHENTIFIÉS.
 *
 * ── Ce que l'audit existant ne pouvait pas faire ───────────────────────────
 *
 * `tests/e2e/Accessibility.spec.ts` compte dix tests. **Les dix visitent
 * `/login`.** L'écran de connexion est le seul que le produit ait jamais
 * soumis à axe-core : ni tableau, ni navigation, ni tableau de bord, ni le
 * moindre écran derrière une session. C'est #865.
 *
 * La raison n'était pas un oubli : auditer un écran authentifié demandait un
 * compte, donc la démo, donc des identifiants — lesquels ne fonctionnent plus
 * (#870). Le banc mobile lève cette dépendance : il peint une session sans
 * réseau ni base, et peut donc ouvrir n'importe quel écran.
 *
 * ── Pourquoi zéro, et non un cliquet ───────────────────────────────────────
 *
 * Parce que la mesure DIT zéro. Les vingt destinations de la barre d'onglets
 * ont été auditées avant d'écrire ce fichier : dix-huit étaient déjà propres,
 * et deux — `/admin/users` et `/admin/organizations` — portaient un
 * `scrollable-region-focusable`. Elles ont été corrigées, pas tolérées.
 *
 * Un cliquet n'a de sens que si la dette existe. Poser un seuil « au plus N
 * violations » alors que N vaut zéro reviendrait à autoriser d'avance la
 * première.
 *
 * ── Le défaut qu'il a trouvé, et d'où il venait ────────────────────────────
 *
 * `scrollable-region-focusable` dit qu'une zone qui défile doit être
 * atteignable au clavier. Faire défiler les tableaux dans leur propre
 * conteneur (#866) a créé ces zones ; deux d'entre elles n'étaient pas
 * focalisables, donc leurs colonnes de droite étaient hors de portée de qui
 * n'a pas de souris. Le correctif d'un défaut avait produit l'autre, et rien
 * ne pouvait le dire avant qu'un navigateur ne regarde.
 */

const WCAG_AA = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"];

/** Les vingt destinations, plus l'écran « Plus » qui les complète. */
const ECRANS: { role: Role; chemin: string }[] = ROLES.flatMap((role) =>
  ongletsPour(role).map((onglet) => ({
    role: role as Role,
    chemin: `${onglet.href.replace(/\/$/, "")}/`,
  })),
);

test.describe("@security l'audit WCAG AA des écrans authentifiés (#865)", () => {
  for (const { role, chemin } of ECRANS) {
    test(`@security ${role} — ${chemin} ne viole aucune règle AA`, async ({
      page,
    }) => {
      await ouvreEnTantQue(page, role, chemin);

      const resultat = await new AxeBuilder({ page })
        .withTags(WCAG_AA)
        .analyze();

      const detail = resultat.violations
        .map(
          (v) =>
            `  ${v.id} (${v.impact}) × ${v.nodes.length}\n` +
            `    ${v.help}\n` +
            v.nodes
              .slice(0, 3)
              .map((n) => `      ${n.html.slice(0, 110)}`)
              .join("\n"),
        )
        .join("\n");

      expect(
        resultat.violations,
        `${chemin} (${role}) viole ${resultat.violations.length} règle(s) :\n${detail}`,
      ).toEqual([]);
    });
  }

  test("@security axe regarde vraiment la page, et ne rend pas zéro à vide", async ({
    page,
  }) => {
    // Vérification d'aveuglement, et elle n'est pas théorique.
    //
    // Vingt écrans propres du premier coup est un résultat qu'on ne croit pas
    // sans l'éprouver : une mauvaise configuration de tags, une page pas
    // encore hydratée, un sélecteur de contexte mal posé, et l'audit rend
    // zéro en n'ayant rien regardé — exactement le genre de vert que ce
    // chantier passe son temps à défaire.
    await ouvreEnTantQue(page, "syndic", ACCUEIL.syndic);

    const avant = await new AxeBuilder({ page }).withTags(WCAG_AA).analyze();
    expect(
      avant.passes.length,
      "axe ne rapporte AUCUNE règle satisfaite : il n'a pas analysé de contenu.",
    ).toBeGreaterThan(10);

    await page.evaluate(() => {
      const bac = document.createElement("div");
      bac.innerHTML =
        '<img src="/icons/icon-192x192.png"><button></button><a href="#"></a><input type="text">';
      document.getElementById("main-content")?.appendChild(bac);
    });

    const apres = await new AxeBuilder({ page }).withTags(WCAG_AA).analyze();
    const trouvees = apres.violations.map((v) => v.id).sort();
    expect(
      trouvees,
      `Quatre défauts connus ont été injectés — image sans alternative, bouton ` +
        `sans nom, lien sans nom, champ sans étiquette — et axe en a trouvé ` +
        `${trouvees.length}. Il ne voit pas ce qu'il devrait voir, donc son ` +
        `zéro sur les vingt écrans ne vaut rien.`,
    ).toEqual(["button-name", "image-alt", "label", "link-name"]);
  });
});
