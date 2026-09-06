import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { canSee, ROLES_SANS_INTERFACE, type Menu } from "../permissions";

/**
 * Cliquet : aucun rôle du backend ne reçoit une navigation vide par accident.
 *
 * ── Ce qui a été trouvé ────────────────────────────────────────────────────
 *
 * En cadrant les personas prestataire et conseil de copropriété (#815, #816),
 * le 2026-09-06 : `canSee()` reconnaissait six noms de rôle, le serveur en
 * sérialise quatorze. Cinq tombaient en fail-closed sans que ce soit voulu :
 *
 *     board_member          → aucun menu, alors que 19 routes lui sont dédiées
 *     contractor            → aucun menu, alors que 15 routes lui sont dédiées
 *     accountant.encodeur   → aucun menu, alors qu'il est comptable
 *     accountant.emetteur   → aucun menu, alors qu'il est comptable
 *     community.moderator   → aucun menu, pour un POINT contre un trait d'union
 *
 * `Navigation.svelte` n'a que huit blocs conditionnels, tous pilotés par
 * `see()`. Les huit rendaient `false`. Et le message de secours ne se
 * déclenchait pas : `hasNoRoleAssignment` teste l'ABSENCE de rôle, or ces
 * comptes en ont un. Ils recevaient une barre avec un logo et une
 * déconnexion.
 *
 * ── Pourquoi aucun test ne le voyait ───────────────────────────────────────
 *
 * `permissions.test.ts` testait `canSee("community-moderator", …)` — la même
 * chaîne fautive que le code. Un test qui reprend la constante qu'il garde ne
 * garde rien : il valide la faute deux fois et passe.
 *
 * La seule façon d'attraper cela est de **ne pas écrire les noms de rôle** :
 * il faut les lire là où ils font foi, c'est-à-dire dans le `Display` de
 * `UserRole`, côté serveur.
 *
 * ── Ce que ce test fait ────────────────────────────────────────────────────
 *
 * Pour chaque rôle sérialisé par le backend, il exige une décision explicite :
 * **ou bien il voit au moins un menu, ou bien il figure dans
 * `ROLES_SANS_INTERFACE`.** Un rôle ajouté côté serveur et oublié côté
 * interface fait échouer ce test, au lieu d'offrir un écran vide.
 *
 * Voir #814.
 */

const SOURCE_DES_ROLES = join(
  process.cwd(),
  "../backend/src/domain/plateforme/user.rs",
);

const MENUS: Menu[] = [
  "admin",
  "gestion",
  "compta",
  "gouvernance",
  "communaute",
  "ticketing",
  "mes-lots",
];

/** Les deux scopes possibles : plateforme, et immeuble sélectionné. */
const SCOPES = [null, { selectedBuildingId: "b-1" }] as const;

/**
 * Les rôles tels que le SERVEUR les écrit, lus dans son `impl Display`.
 *
 * On lit le fichier plutôt que de recopier la liste : une liste recopiée
 * diverge, et c'est exactement la divergence que ce test existe pour
 * empêcher.
 */
function rolesDuBackend(): string[] {
  const source = readFileSync(SOURCE_DES_ROLES, "utf8");
  const debut = source.indexOf("impl std::fmt::Display for UserRole");
  expect(
    debut,
    "le `impl Display for UserRole` a changé de forme",
  ).toBeGreaterThan(0);
  const bloc = source.slice(debut, source.indexOf("\n}", debut));
  return [...bloc.matchAll(/write!\(f, "([a-z_.]+)"\)/g)].map((m) => m[1]);
}

describe("aucun rôle du backend ne reçoit une navigation vide (#814)", () => {
  const roles = rolesDuBackend();

  it("lit bien les rôles à la source", () => {
    // Sans ce contrôle, un `Display` remanié rendrait le cliquet
    // silencieusement vert en ne trouvant plus rien à vérifier — le piège
    // déjà posé dans `garde_ecriture`, `garde_lecture` et
    // `garde-couverture-testid`.
    expect(roles.length, `rôles relevés : ${roles.join(", ")}`).toBeGreaterThan(
      10,
    );
    expect(roles).toContain("syndic");
    expect(roles).toContain("board_member");
  });

  it("donne à chaque rôle soit un menu, soit une absence assumée", () => {
    const muets: string[] = [];

    for (const role of roles) {
      const voitQuelqueChose = SCOPES.some((scope) =>
        MENUS.some((menu) => canSee(role, menu, scope)),
      );
      if (!voitQuelqueChose && !ROLES_SANS_INTERFACE.has(role)) {
        muets.push(role);
      }
    }

    expect(
      muets,
      `${muets.length} rôle(s) servis par le backend ne voient AUCUN menu, ` +
        `sans figurer dans \`ROLES_SANS_INTERFACE\`.\n\n` +
        `L'utilisateur obtient une barre de navigation vide : pas même le ` +
        `message « aucun rôle assigné », puisqu'il en a un. C'est ce qui est ` +
        `arrivé à cinq rôles jusqu'au 2026-09-06 (#814).\n\n` +
        `Deux issues possibles, toutes deux explicites :\n` +
        `  — mapper le rôle dans \`canSee()\` ;\n` +
        `  — l'inscrire dans \`ROLES_SANS_INTERFACE\` en disant pourquoi.\n\n` +
        `Rôle(s) : ${muets.join(", ")}`,
    ).toEqual([]);
  });

  it("ne déclare aucun rôle sans interface qui n'existe pas côté serveur", () => {
    // Le registre doit rester une photographie du réel. Un rôle retiré du
    // backend et laissé ici masquerait la prochaine omission.
    const fantomes = [...ROLES_SANS_INTERFACE].filter(
      (r) => !roles.includes(r),
    );
    expect(
      fantomes,
      `\`ROLES_SANS_INTERFACE\` cite des rôles que le backend ne sert plus : ` +
        fantomes.join(", "),
    ).toEqual([]);
  });
});
