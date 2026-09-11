import { test, expect } from "@playwright/test";
import { ongletsPour } from "../../../src/lib/onglets-mobiles";
import { largeurDuDocument, ouvreEnTantQue, ROLES, type Role } from "./socle";

/**
 * Les vingt destinations de la barre d'onglets, ouvertes une par une.
 *
 * ── Pourquoi celles-ci, et pourquoi maintenant ─────────────────────────────
 *
 * Ce sont les seuls chemins que le produit propose sur un téléphone. Neuf des
 * dix-sept d'origine ne menaient nulle part — page inexistante, chemin
 * inventé — et c'était de moi. Elles ont été corrigées contre la carte des
 * pages, à la main, ce qui ne garantit rien pour la suivante.
 *
 * Le contrôle importe surtout pour le **copropriétaire** : le rôle le plus
 * nombreux du produit, le seul qui n'ouvrira jamais un bureau, et celui que
 * six recettes navigateur n'ont jamais éprouvé.
 *
 * ── Ce que chaque visite vérifie ───────────────────────────────────────────
 *
 * L'API ne rend que des collections vides. C'est délibéré : l'état vide est
 * l'écran du PREMIER JOUR d'un cabinet, celui que personne ne regarde, et
 * celui où une mise en page tient le moins bien.
 *
 *   1. la page existe et garde sa session — pas de renvoi vers `/login` ;
 *   2. la coquille s'y peint — la barre d'onglets est là ;
 *   3. rien ne déborde en largeur.
 *
 * Le troisième point ne juge PAS les tableaux : ceux de #866 défilent dans
 * leur propre conteneur, ce qui est voulu. Un débordement du DOCUMENT est
 * autre chose — il fait glisser la page entière de côté au moindre geste,
 * pendant que la barre d'onglets, fixe, reste en place.
 */

for (const role of ROLES) {
  const destinations = ongletsPour(role).map((o) => o.href);

  test.describe(`@happy les destinations du rôle ${role}`, () => {
    // Vérification d'aveuglement : une liste vide ferait passer la boucle
    // entière sans rien ouvrir.
    test(`@happy ${role} — la barre propose bien cinq destinations`, () => {
      expect(
        destinations.length,
        `\`ongletsPour("${role}")\` rend ${destinations.length} destinations. ` +
          `Les tests qui suivent n'ouvriraient rien.`,
      ).toBe(5);
    });

    for (const href of destinations) {
      test(`@happy ${role} — ${href} s'ouvre sans déborder`, async ({
        page,
      }) => {
        // La barre oblique finale évite une redirection du serveur de fichiers.
        const chemin = `${href.replace(/\/$/, "")}/`;
        await ouvreEnTantQue(page, role as Role, chemin);

        // Lue APRÈS que la page se soit posée, pas à la peinture de la
        // coquille. `RouteGuard` décide de l'accès dans un effet, donc le
        // rebond arrive une fraction de seconde APRÈS que la barre d'onglets
        // soit visible.
        //
        // La première version lisait l'URL tout de suite et déclarait
        // `/admin/acps` bonne pour le syndic, alors qu'elle le renvoyait sur
        // `/syndic`. C'est l'audit d'accessibilité qui l'a trouvé, en
        // échouant sur « Execution context was destroyed » — un test qui
        // tombe pour une raison qui n'est pas la sienne dit quand même
        // quelque chose.
        await page.waitForLoadState("networkidle");
        await page.waitForTimeout(700);

        expect(
          new URL(page.url()).pathname,
          `${href} a conduit à ${new URL(page.url()).pathname}. Un onglet doit ` +
            `mener où il annonce : soit la page n'existe pas, soit ` +
            `\`canAccessRoute\` la refuse à ce rôle et \`RouteGuard\` renvoie ` +
            `ailleurs. Vérifier que la page EXISTE ne suffit pas.`,
        ).toBe(chemin);

        await expect(page.getByTestId("tabbar")).toBeVisible();

        const appareil = page.viewportSize()!.width;
        const largeur = await largeurDuDocument(page);
        expect(
          largeur,
          `${href} : le document mesure ${largeur} px pour un appareil de ` +
            `${appareil} px. La page se dézoomera pour tout contenir, et ` +
            `chaque texte de l'écran rétrécira d'autant.`,
        ).toBeLessThanOrEqual(appareil);
      });
    }
  });
}
