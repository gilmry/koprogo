import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Garde-fou : présider une assemblée n'est pas y voter.
 *
 * ── Le défaut ────────────────────────────────────────────────────────────
 *
 * Le bouton de clôture du scrutin s'affichait sous la condition
 * `isAdmin && canVote`, où :
 *
 *   canVote = résolution en attente && séance planifiée && isOwner
 *
 * Un syndic qui n'est pas copropriétaire a donc `isOwner = false`. Le bouton
 * ne lui apparaissait jamais. Or c'est lui qui tient la séance : les
 * résolutions restaient indéfiniment « en attente ».
 *
 * ── Pourquoi c'était grave au-delà de l'ergonomie ────────────────────────
 *
 * Le plafonnement des voix de l'Art. 3.87 § 7 s'applique **à la clôture**,
 * parce qu'il porte sur l'ensemble des bulletins et que le dernier déposé
 * peut faire basculer une séance licite jusque-là.
 *
 * Un bouton de clôture inatteignable rendait donc un invariant légal
 * inatteignable. Le code prétendait à une conformité que l'usage ne
 * produisait jamais — pire qu'une absence de conformité, qui au moins se
 * voit. Constaté en recette le 2026-09-04 : trois résolutions votées,
 * neuf bulletins, aucune clôture possible.
 *
 * ── Ce que ce test vérifie ───────────────────────────────────────────────
 *
 * Que la condition d'affichage du bouton de clôture ne dépend pas du droit
 * de vote. Le test lit la source plutôt que de monter le composant : la
 * règle porte sur une condition, pas sur un rendu, et un test de rendu
 * exigerait de simuler l'authentification, les lots et la séance — trois
 * mécanismes sans rapport avec la règle éprouvée.
 */

const PANNEAU = join(
  process.cwd(),
  "src/components/resolutions/ResolutionVotePanel.svelte",
);

describe("clôture du scrutin", () => {
  const source = readFileSync(PANNEAU, "utf-8");

  it("expose un droit de clore distinct du droit de voter", () => {
    expect(source).toContain("canCloseVoting");
  });

  it("ne subordonne pas la clôture au droit de vote", () => {
    const blocBouton = source.match(
      /\{#if\s+([^}]+)\}\s*<button\s+onclick=\{handleCloseVoting\}/,
    );
    expect(blocBouton, "le bouton de clôture doit être gardé par une condition").not.toBeNull();

    const garde = blocBouton![1];
    expect(
      garde.includes("canVote"),
      `la clôture ne doit pas dépendre de canVote, qui exige isOwner. Garde trouvée : « ${garde} »`,
    ).toBe(false);
    expect(garde).toContain("canCloseVoting");
  });

  it("ne fait pas dépendre canCloseVoting de la qualité de copropriétaire", () => {
    const decl = source.match(/let\s+canCloseVoting\s*=\s*\$derived\(([\s\S]*?)\);/);
    expect(decl, "canCloseVoting doit être un $derived").not.toBeNull();

    const corps = decl![1];
    expect(
      corps.includes("isOwner"),
      "présider une assemblée n'exige pas d'y posséder un lot (Art. 3.87 § 6)",
    ).toBe(false);
    // Mais il faut bien être syndic ou superadmin.
    expect(corps).toContain("isAdmin");
  });
});
