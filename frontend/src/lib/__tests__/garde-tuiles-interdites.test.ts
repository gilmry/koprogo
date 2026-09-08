import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Un tableau de bord ne propose pas une route interdite à son rôle.
 *
 * `OwnerDashboard.svelte` portait une tuile « Charges » vers `/expenses`, que
 * `guards.ts` réserve au syndic et au comptable. Le copropriétaire qui la
 * cliquait était **redirigé en silence** vers son propre tableau de bord par
 * `RouteGuard.svelte:68` : il revenait d'où il venait, sans message, sans
 * erreur. Le bouton lisait comme cassé.
 *
 * C'est le motif dominant du produit — une capacité offerte à l'écran et
 * inatteignable — sous sa forme la plus trompeuse : ici ce n'est pas la
 * capacité qui manque, c'est le droit d'y aller, et rien ne le dit.
 *
 * Le rapprochement n'est possible que sur les quatre tableaux de bord, parce
 * que ce sont les seuls composants dont le rôle du lecteur est certain. Un
 * lien dans un composant partagé peut viser une route interdite à certains de
 * ses lecteurs sans que l'analyse statique puisse trancher.
 *
 * Le cliquet est à zéro : c'est une interdiction. Il n'y avait qu'un cas.
 */

const TABLEAUX: { fichier: string; role: string }[] = [
  { fichier: "src/components/dashboards/OwnerDashboard.svelte", role: "OWNER" },
  {
    fichier: "src/components/dashboards/SyndicDashboard.svelte",
    role: "SYNDIC",
  },
  {
    fichier: "src/components/dashboards/AccountantDashboard.svelte",
    role: "ACCOUNTANT",
  },
  {
    fichier: "src/components/dashboards/AdminDashboard.svelte",
    role: "SUPERADMIN",
  },
];

/** Les routes gardées, telles que `guards.ts` les déclare. */
function routesGardees(): Map<string, Set<string>> {
  const source = readFileSync(
    join(process.cwd(), "src/lib/guards.ts"),
    "utf-8",
  );
  const gardes = new Map<string, Set<string>>();
  for (const m of source.matchAll(/"([^"]+)":\s*\[([^\]]*)\]/g)) {
    const roles = new Set(
      [...m[2].matchAll(/UserRole\.(\w+)/g)].map((r) => r[1]),
    );
    if (roles.size > 0) gardes.set(m[1], roles);
  }
  return gardes;
}

function tuilesInterdites(): string[] {
  const gardes = routesGardees();
  const fautives: string[] = [];
  for (const { fichier, role } of TABLEAUX) {
    const source = readFileSync(join(process.cwd(), fichier), "utf-8").replace(
      /<!--[\s\S]*?-->/g,
      "",
    );
    for (const m of source.matchAll(/href="(\/[^"{}]*)"/g)) {
      const route = m[1].replace(/\/+$/, "") || "/";
      const autorises = gardes.get(route);
      if (autorises && !autorises.has(role)) {
        const ligne = source.slice(0, m.index).split("\n").length;
        fautives.push(
          `  ${fichier}:${ligne}\n` +
            `      ${route} — réservé à ${[...autorises].sort().join(", ")}, ` +
            `proposé à ${role}`,
        );
      }
    }
  }
  return fautives;
}

describe("les tableaux de bord ne mènent pas à une route interdite", () => {
  it("ne propose aucune route que le rôle ne peut pas ouvrir", () => {
    expect(
      tuilesInterdites().join("\n"),
      "Ces liens mènent à une route que `guards.ts` interdit au rôle du " +
        "tableau de bord. `RouteGuard` redirige en silence : l'utilisateur " +
        "clique et revient d'où il vient, sans message.\n\n" +
        "Visez la route équivalente ouverte au rôle — par exemple " +
        "`/owner/expenses` plutôt que `/expenses`.",
    ).toBe("");
  });

  it("lit encore les gardes et les tableaux", () => {
    // Contrôle d'aveuglement. Il ne porte PAS sur le nombre d'infractions :
    // une garde qui exige des violations punit sa propre réussite.
    expect(
      routesGardees().size,
      "plus aucune route gardée trouvée dans guards.ts : le motif a changé.",
    ).toBeGreaterThan(10);
    const liens = TABLEAUX.map((t) =>
      readFileSync(join(process.cwd(), t.fichier), "utf-8"),
    )
      .join("\n")
      .match(/href="\/[^"{}]*"/g);
    expect(
      liens?.length ?? 0,
      "plus aucun lien littéral dans les tableaux de bord.",
    ).toBeGreaterThan(10);
  });
});
