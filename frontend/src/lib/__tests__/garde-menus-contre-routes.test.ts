/**
 * Cliquet : un rôle ne voit pas un menu vers une route qui le refuse.
 *
 * ── Le défaut que ce cliquet borne ────────────────────────────────────────
 *
 * Deux fichiers décident de ce qu'un rôle peut faire, et **rien ne les
 * confronte** :
 *
 *   - `auth/permissions.ts` pilote la VISIBILITÉ des menus (`canSee`) ;
 *   - `guards.ts` pilote l'ACCÈS aux routes (`canAccessRoute`).
 *
 * Le modérateur communauté l'a payé (#962). `permissions.ts:212` lui accorde
 * le groupe `communaute` « comme un owner » ; les sept routes
 * communautaires de `guards.ts` ne listent que superadmin, syndic,
 * comptable et owner. Mesuré au navigateur :
 *
 *     rôle rendu par le serveur : "community.moderator"
 *     URL après navigation      : /
 *     console : [RouteGuard] Access denied to /notices
 *               for role community.moderator
 *
 * **Le menu le montre ; la route le refuse.** Le rôle voit une porte qu'il
 * ne peut pas franchir — et c'est pire qu'une absence de menu, qui au moins
 * ne promet rien.
 *
 * Aucun test ne l'avait vu parce qu'aucun compte `community.moderator`
 * n'existe dans le monde de recette : le rôle n'avait jamais été exercé.
 * L'écart est apparu en écrivant son parcours filmé.
 *
 * ── Pourquoi un cliquet, et pas un refus ──────────────────────────────────
 *
 * La mise en cohérence demande un ARBITRAGE, pas un correctif : soit le
 * modérateur a bien le périmètre d'un copropriétaire et rejoint les sept
 * routes, soit le rôle n'est pas prêt et quitte `permissions.ts` pour
 * `ROLES_SANS_INTERFACE`. C'est une décision produit (#962), et un test
 * n'en rend pas.
 *
 * Ce cliquet ne tranche donc pas. Il **empêche que l'écart grandisse**, et
 * nomme chaque contradiction pour qu'elle soit traitée plutôt que découverte
 * au navigateur six mois plus tard.
 */
import { describe, it, expect } from "vitest";
import { canSee, type Menu, type Role } from "../auth/permissions";
import { canAccessRoute } from "../guards";
import { UserRole } from "../types";

/**
 * Les routes que chaque menu représente.
 *
 * Écrite ici plutôt que dérivée : `permissions.ts` raisonne en GROUPES de
 * menus, `guards.ts` en routes, et rien ne relie formellement les deux. Cette
 * table EST le lien manquant — la nommer, c'est déjà répondre à la moitié du
 * défaut.
 */
const ROUTES_PAR_MENU: Record<Menu, string[]> = {
  communaute: [
    "/exchanges",
    "/polls",
    "/notices",
    "/bookings",
    "/sharing",
    "/skills",
    "/energy-campaigns",
  ],
  compta: ["/expenses", "/budgets", "/journal-entries"],
  gouvernance: ["/meetings", "/convocations"],
  ticketing: ["/tickets", "/quotes", "/work-reports"],
  "mes-lots": ["/owner/units", "/owner/payments"],
  gestion: ["/buildings", "/owners"],
  admin: ["/admin/users", "/admin/organizations"],
};

const ROLES: Role[] = [
  UserRole.SUPERADMIN,
  UserRole.SYNDIC,
  UserRole.ACCOUNTANT,
  UserRole.OWNER,
  UserRole.COMMUNITY_MODERATOR,
  UserRole.BOARD_MEMBER,
];

/**
 * Un périmètre POSÉ : c'est la situation normale d'un utilisateur au
 * travail. Sans lui, `canSee` cache les menus métier et le cliquet
 * mesurerait un écran de transition plutôt que le produit.
 */
const EN_CONTEXTE = {
  selectedAcpId: "acp-de-reference",
  selectedBuildingId: null,
};

/** Une contradiction : le menu est offert, la route refuse. */
function contradictions(): string[] {
  const trouvees: string[] = [];
  for (const role of ROLES) {
    for (const [menu, routes] of Object.entries(ROUTES_PAR_MENU)) {
      if (!canSee(role, menu as Menu, EN_CONTEXTE)) continue;
      for (const route of routes) {
        if (!canAccessRoute(route, role as UserRole)) {
          trouvees.push(`${role} voit « ${menu} » mais ${route} le refuse`);
        }
      }
    }
  }
  return trouvees.sort();
}

/**
 * Mesuré le 2026-09-20 : **24 contradictions**, sur quatre rôles.
 *
 *   board_member         10
 *   community.moderator   9
 *   superadmin            4
 *   syndic                1
 *
 * ── Ce que le chiffre a appris ────────────────────────────────────────────
 *
 * #962 ne portait que sur le modérateur communauté. L'écart est **quatre
 * fois plus large**, et il touche les rôles principaux :
 *
 *   - `superadmin` voit « compta » alors que `/expenses` et
 *     `/journal-entries` ne listent que syndic et comptable ;
 *   - `syndic` voit « compta » alors que `/journal-entries` n'autorise que
 *     le comptable — il clique « Écritures » et se fait renvoyer ;
 *   - `board_member` cumule dix refus, dont `mes-lots`, alors qu'un membre
 *     du conseil EST copropriétaire (Art. 3.90 § 1er).
 *
 * Ce nombre ne doit que BAISSER. Il est écrit en dur, et pas calculé : un
 * cliquet qui recalcule sa propre référence à chaque exécution vaut toujours
 * sa valeur courante et ne peut jamais mordre. C'est le premier jet de cette
 * garde, corrigé avant d'être commis.
 */
const CONTRADICTIONS_AU_2026_09_20 = 24;

describe("menus et routes ne se contredisent pas (#962)", () => {
  it("ne laisse pas grossir le nombre de rôles qui voient une porte fermée", () => {
    const trouvees = contradictions();

    expect(
      trouvees.length,
      `Un rôle voit désormais un menu vers une route qui le refuse :\n\n` +
        trouvees.map((c) => `  - ${c}`).join("\n") +
        `\n\nCe n'est pas un détail d'affichage. Un menu qui mène à un refus ` +
        `est pire qu'un menu absent : il promet une capacité, et l'utilisateur ` +
        `attribue l'échec à une panne plutôt qu'à une interdiction.\n\n` +
        `Décidez dans quel fichier la règle vit — \`permissions.ts\` pour ce ` +
        `qui se voit, \`guards.ts\` pour ce qui s'ouvre — et alignez l'autre.`,
    ).toBeLessThanOrEqual(CONTRADICTIONS_AU_2026_09_20);
  });

  it("@edge reconnaît encore une contradiction quand on lui en présente une", () => {
    // Un cliquet qui ne détecte plus rien passe au vert pour la mauvaise
    // raison. On vérifie que le détecteur mord : `owner` voit « communaute »,
    // et une route inventée qu'aucun rôle n'autorise doit ressortir.
    expect(canSee(UserRole.OWNER, "communaute", EN_CONTEXTE)).toBe(true);

    // `canAccessRoute` rend `true` pour toute route ABSENTE de son registre
    // (guards.ts:117) — c'est précisément ce qui rend ce cliquet nécessaire :
    // l'essentiel du produit n'a aucune garde déclarée, et ne peut donc pas
    // contredire un menu. Le détecteur ne voit que les routes gardées.
    expect(canAccessRoute("/route-qui-nexiste-pas", UserRole.OWNER)).toBe(true);
    expect(canAccessRoute("/notices", UserRole.COMMUNITY_MODERATOR)).toBe(
      false,
    );
  });
});
