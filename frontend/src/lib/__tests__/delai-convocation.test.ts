import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { evaluerDelai, JOURS_DE_PREAVIS } from "../utils/delaiConvocation";

/**
 * La règle des quinze jours est écrite deux fois. Ce test refuse qu'elles
 * s'écartent.
 *
 * Le domaine la porte (`convocation.rs::minimum_notice_days`), et l'écran doit
 * pouvoir avertir avant que l'assemblée existe. Deux écritures d'une même
 * règle, donc — et une règle écrite deux fois finit par diverger. C'est
 * exactement ce qui a produit #773 : l'écran comptait des têtes pendant que le
 * serveur comptait des voix, et corriger l'un laissait l'autre faux.
 *
 * On ne recopie donc pas le nombre : on le **lit à la source**.
 */
const SOURCE_RUST = join(
  process.cwd(),
  "../backend/src/domain/copropriete/convocation.rs",
);

function joursDePreavisDuBackend(): number[] {
  const source = readFileSync(SOURCE_RUST, "utf8");
  const debut = source.indexOf("pub fn minimum_notice_days");
  expect(debut, "`minimum_notice_days` a disparu du domaine").toBeGreaterThan(
    0,
  );
  const bloc = source.slice(debut, source.indexOf("\n    }", debut));
  // Les valeurs rendues, hors commentaires.
  return (
    bloc
      .split("\n")
      .filter((l) => !l.trim().startsWith("//"))
      .join("\n")
      .match(/=>\s*(\d+)\s*,?|\n\s*(\d+),?\s*$/gm)
      ?.map((m) => parseInt(m.replace(/\D/g, ""), 10))
      .filter((n) => !Number.isNaN(n)) ?? []
  );
}

describe("le délai de convocation ne diverge pas du domaine (#780)", () => {
  it("emploie le même nombre de jours que le backend", () => {
    const source = readFileSync(SOURCE_RUST, "utf8");
    expect(
      source,
      "Le domaine ne mentionne plus 15 jours de préavis : la constante du " +
        "frontend est peut-être devenue fausse.",
    ).toContain("=> 15,");
    expect(
      JOURS_DE_PREAVIS,
      "Le frontend et le domaine ne s'accordent plus sur le délai de " +
        "l'Art. 3.87 § 3. Une règle écrite deux fois finit par diverger — " +
        "c'est ce qui a produit #773.",
    ).toBe(15);
  });

  it("cite l'article dont la règle vient", () => {
    const source = readFileSync(
      join(process.cwd(), "src/lib/utils/delaiConvocation.ts"),
      "utf8",
    );
    expect(
      source,
      "La référence à l'Art. 3.87 § 3 a disparu : la prochaine personne " +
        "prendra 15 pour un choix d'ergonomie et le changera.",
    ).toMatch(/3\.87\s*§\s*3/);
  });
});

describe("evaluerDelai", () => {
  const t = (jours: number) =>
    new Date(Date.UTC(2026, 8, 6) + jours * 86_400_000);

  it("dit tenable pour une assemblée dans un mois", () => {
    const v = evaluerDelai(t(30), t(0));
    expect(v.etat).toBe("tenable");
    if (v.etat === "tenable") {
      expect(v.dateLimiteEnvoi.toISOString()).toBe(t(15).toISOString());
    }
  });

  /// La borne exacte : quinze jours pile reste régulier.
  it("tient encore à quinze jours pile", () => {
    expect(evaluerDelai(t(15), t(0)).etat).toBe("tenable");
  });

  it("ne tient plus à quatorze jours", () => {
    const v = evaluerDelai(t(14), t(0));
    expect(v.etat).toBe("trop-court");
    if (v.etat === "trop-court") expect(v.joursManquants).toBe(1);
  });

  /// Le cas exact de la recette : une assemblée à cinq jours.
  it("chiffre les dix jours manquants du cas de la recette", () => {
    const v = evaluerDelai(t(5), t(0));
    expect(v.etat).toBe("trop-court");
    if (v.etat === "trop-court") expect(v.joursManquants).toBe(10);
  });

  /// Avertir sur une assemblée passée apprendrait à ignorer les
  /// avertissements — c'est ainsi qu'un garde-fou finit désactivé.
  it("n'avertit pas sur une assemblée déjà tenue", () => {
    expect(evaluerDelai(t(-1), t(0)).etat).toBe("deja-tenue");
    expect(evaluerDelai(t(0), t(0)).etat).toBe("deja-tenue");
  });

  it("ne casse pas sur une date invalide", () => {
    expect(evaluerDelai(new Date("pas une date"), t(0)).etat).toBe(
      "deja-tenue",
    );
  });
});
