/**
 * L'invariant anti-dette de la documentation vivante.
 *
 * ── Pourquoi cette garde existe ───────────────────────────────────────────
 *
 * `skills/documentation-vivante.md` pose quatre éléments non optionnels. Les
 * trois premiers se voient (un parcours, une cadence, une galerie). Le
 * quatrième est le seul qui empêche les trois autres de pourrir :
 *
 * > « Un invariant structurel oblige **chaque harnais à importer le parcours
 * > partagé** : un harnais de valeur qui ne rejoue plus le parcours (c'est-à-
 * > dire qu'on a recoupé la doc à la main) fait passer la suite au **rouge**.
 * > […] la doc n'est pas "mise à jour", elle est **dérivée** du parcours
 * > rejoué, et on ne peut pas la découpler du test sans déclencher le rouge. »
 *
 * Sans cette garde, rien n'empêche quelqu'un de recopier les étapes dans la
 * vitrine « juste pour cette fois ». Le jour où c'est fait, la doc et le test
 * divergent en silence — et la dette de documentation revient par la porte
 * qu'on croyait murée.
 *
 * ── Ce qu'elle ne prétend pas faire ───────────────────────────────────────
 *
 * Elle lit des **fichiers**, pas du sens. Elle vérifie que les deux harnais
 * importent le même artefact ; elle ne juge pas si le parcours est bon. C'est
 * une borne, pas un verdict — comme toutes les gardes de ce dépôt.
 */
import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { resolve } from "node:path";

const RACINE = resolve(__dirname, "../../..");
const JOURNEYS = resolve(RACINE, "tests/e2e/journeys");

/** Le harnais de correctness et le harnais de valeur. */
// Le gate E2E est un ENSEMBLE de fichiers, pas un fichier.
//
// Il n'en avait qu'un le 2026-09-08 : `parcours.spec.ts`. Depuis, le parcours
// copropriétaire a reçu son propre `coproprietaire.spec.ts`, et la garde a
// conclu que « le gate et la vitrine n'importent pas les mêmes parcours » —
// alors qu'ils les rejouent tous les deux, simplement répartis autrement.
//
// C'est le modèle de la garde qui était périmé, pas le harnais. L'élargir
// n'affaiblit RIEN : l'invariant reste que l'union des parcours rejoués par
// le gate égale celle de la vitrine. Un parcours filmé que le gate ne rejoue
// pas resterait refusé, et c'est tout ce qu'on lui demande.
const HARNAIS_GATE = [
  "parcours.spec.ts",
  "coproprietaire.spec.ts",
  "comptable.spec.ts",
  "administration.spec.ts",
  "conseil.spec.ts",
  "prestataire.spec.ts",
  "moderation.spec.ts",
  "incident.spec.ts",
  // Les quatre balayages tiennent dans UNE spec : ils partagent le même
  // mécanisme (`balayage.ts`) et ne diffèrent que par le rôle. Les répartir
  // en quatre fichiers n'aurait ajouté que de la place où diverger.
  "balayage.spec.ts",
] as const;
const HARNAIS_VITRINE = ["enregistrer-vitrine.mjs"] as const;

const HARNAIS = [
  ...HARNAIS_GATE.map((f) => ["gate E2E", f] as const),
  ...HARNAIS_VITRINE.map((f) => ["vitrine", f] as const),
] as const;

function lire(fichier: string): string {
  return readFileSync(resolve(JOURNEYS, fichier), "utf8");
}

describe("documentation vivante — l'invariant anti-dette", () => {
  it("@happy le dossier des parcours existe et porte au moins un parcours", () => {
    expect(existsSync(JOURNEYS)).toBe(true);
    const parcours = readdirSync(JOURNEYS).filter((f) =>
      f.endsWith(".journey.ts"),
    );
    expect(
      parcours.length,
      "Aucun *.journey.ts : il n'y a pas de source de vérité partagée à " +
        "importer, donc rien à garder.",
    ).toBeGreaterThan(0);
  });

  it.each(HARNAIS)(
    "@negative le harnais « %s » importe le parcours partagé",
    (_nom, fichier) => {
      const source = lire(fichier);
      const importeUnParcours = /\.journey(\.ts)?["']/.test(source);
      expect(
        importeUnParcours,
        `${fichier} n'importe aucun *.journey.ts. Un harnais qui ne rejoue ` +
          "plus le parcours partagé a recopié les étapes à la main : la doc " +
          "et le test vont diverger en silence.",
      ).toBe(true);
    },
  );

  it("@edge les deux harnais rejouent LE MÊME parcours", () => {
    // Dédoublonné : un parcours cité en commentaire ET importé ne doit
    // compter qu'une fois. C'est la garde elle-même qui l'a signalé à sa
    // première exécution — elle comparait deux ensembles dont l'un portait
    // une mention de documentation en plus.
    const nomDuParcours = (source: string) =>
      [
        ...new Set(
          [
            ...source.matchAll(
              /["'`][.\/\w-]*?([\w-]+)\.journey(?:\.ts)?["'`]/g,
            ),
          ].map((m) => m[1]),
        ),
      ].sort();

    const union = (fichiers: readonly string[]) =>
      [...new Set(fichiers.flatMap((f) => nomDuParcours(lire(f))))].sort();
    const gate = union(HARNAIS_GATE);
    const vitrine = union(HARNAIS_VITRINE);
    expect(
      vitrine,
      "Le gate et la vitrine n'importent pas les mêmes parcours. Deux " +
        "lectures d'un même parcours, c'est le principe fondateur ; deux " +
        "parcours différents, c'est la double-maintenance revenue.",
    ).toEqual(gate);
  });

  it("@security la vitrine ne rend aucun verdict", () => {
    const source = lire("enregistrer-vitrine.mjs");
    // Le skill est explicite : « La preuve de valeur n'est PAS un test : elle
    // ne doit pas faire tomber le build. » Un `expect` ici lui confierait la
    // responsabilité qu'on ne peut pas lui confier — et elle finirait coupée
    // du pipeline, comme le skill l'annonce.
    expect(
      /\bexpect\s*\(/.test(source),
      "enregistrer-vitrine.mjs contient une assertion. La preuve de valeur " +
        "n'est pas un test : le verdict appartient au gate E2E.",
    ).toBe(false);
  });

  it("@edge la cadence est une constante nommée, pas un chiffre magique", () => {
    const scene = lire("scene.ts");
    expect(
      /export const CADENCE_MS/.test(scene),
      "La cadence doit être une constante nommée — le skill l'exige " +
        "explicitement : « pas un chiffre magique ».",
    ).toBe(true);
  });
});
