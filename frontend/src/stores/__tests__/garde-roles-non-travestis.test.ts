import { describe, it, expect } from "vitest";
import { mapUserFromBackend } from "../auth";
import { UserRole } from "../../lib/types";

/**
 * Garde : aucun rôle servi par le backend n'est remplacé par un autre.
 *
 * ── Le défaut ──────────────────────────────────────────────────────────────
 *
 * `normalizeRole` ne connaissait que quatre rôles et terminait par
 * `default: return UserRole.OWNER`. Les dix autres devenaient tous `owner` —
 * mesuré le 2026-09-07 :
 *
 *     entrée : { role: "board_member" }
 *     sortie : { role: "owner", roles: ["owner"] }
 *
 * Deux conséquences. Une **élévation de rôle côté client** : un prestataire ou
 * un concierge se voyait proposer « Mes lots » et « Communauté ». Et la
 * **neutralisation de #814** : `ROLES_SANS_INTERFACE` ne pouvait jamais
 * s'appliquer, puisque le rôle était déjà écrasé quand `Navigation.svelte` le
 * testait.
 *
 * ── Pourquoi `garde-roles` ne l'a pas vu ───────────────────────────────────
 *
 * Elle appelle `canSee("contractor", …)` directement, avec la chaîne du
 * backend. En production, cette chaîne n'arrive jamais jusque-là : le store
 * l'a déjà remplacée. **La garde éprouvait un chemin que le produit
 * n'emprunte pas.**
 *
 * Ce test-ci se place donc une couche plus bas, à l'endroit exact où
 * l'écrasement avait lieu. Les deux sont nécessaires : l'un vérifie la règle,
 * l'autre qu'elle est atteinte.
 *
 * Suivi en #836.
 */

/** Les quatorze rôles de `backend/src/domain/plateforme/user.rs`. */
const ROLES_DU_BACKEND = [
  "superadmin",
  "syndic",
  "accountant",
  "accountant.encodeur",
  "accountant.emetteur",
  "board_member",
  "contractor",
  "owner",
  "community.moderator",
  "lawyer",
  "notary",
  "amo",
  "architect",
  "bet",
  "warden",
];

const utilisateurAvecRole = (role: string) => ({
  id: "u-1",
  email: "test@example.be",
  first_name: "Marcel",
  last_name: "Devos",
  role,
  roles: [{ id: "r-1", role, is_primary: true }],
  active_role: { id: "r-1", role, is_primary: true },
});

describe("le store ne travestit aucun rôle (#836)", () => {
  it("conserve chacun des quatorze rôles servis par le backend", () => {
    const travestis: string[] = [];

    for (const role of ROLES_DU_BACKEND) {
      const u: any = mapUserFromBackend(utilisateurAvecRole(role));
      if (u.role !== role) travestis.push(`${role} → ${u.role}`);
    }

    expect(
      travestis,
      `${travestis.length} rôle(s) remplacés par un autre au passage du store.\n\n` +
        `Un rôle travesti en \`owner\` obtient les menus du copropriétaire : ` +
        `c'est une élévation de rôle côté client, et cela neutralise ` +
        `\`ROLES_SANS_INTERFACE\` — le registre est consulté APRÈS.\n\n` +
        `Si un rôle a été ajouté côté serveur, déclarez-le dans ` +
        `\`UserRole\` (lib/types.ts), puis donnez-lui un menu ou inscrivez-le ` +
        `dans \`ROLES_SANS_INTERFACE\`.\n\n` +
        travestis.join("\n"),
    ).toEqual([]);
  });

  it("conserve le rôle dans `roles[]` autant que dans le rôle actif", () => {
    const u: any = mapUserFromBackend(utilisateurAvecRole("contractor"));
    expect(u.roles?.map((r: any) => r.role)).toEqual(["contractor"]);
  });

  /**
   * Un rôle qu'aucune des deux listes ne connaît ne doit pas devenir un rôle
   * réel. Il traverse tel quel, et `canSee` échouera fermé.
   */
  it("ne promeut pas un rôle inconnu en copropriétaire", () => {
    const u: any = mapUserFromBackend(utilisateurAvecRole("role_invente"));
    expect(u.role).not.toBe(UserRole.OWNER);
  });

  /// Sans ce contrôle, `UserRole` pourrait perdre des valeurs sans que rien
  /// ne le dise : le premier test passerait encore, la chaîne étant renvoyée
  /// telle quelle par le repli.
  it("déclare les quatorze rôles dans `UserRole`", () => {
    const declares = new Set(Object.values(UserRole) as string[]);
    const manquants = ROLES_DU_BACKEND.filter((r) => !declares.has(r));
    expect(
      manquants,
      `\`UserRole\` ne déclare pas : ${manquants.join(", ")}`,
    ).toEqual([]);
  });
});
