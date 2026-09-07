import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, extname } from "node:path";

/**
 * Cliquet : aucun message d'interface n'est écrit en dur.
 *
 * ── Le défaut ──────────────────────────────────────────────────────────────
 *
 * Un message passé littéralement à un toast ne traverse pas `svelte-i18n` :
 * il s'affiche identique en néerlandais, en allemand et en anglais. Le produit
 * vise quatre langues et sert une clientèle belge — un néerlandophone qui lit
 * « Erreur lors de la mise à jour du profil » n'a pas une traduction
 * approximative, il a un message qu'il ne peut pas lire.
 *
 * ── Pourquoi rien ne le détectait ─────────────────────────────────────────
 *
 * Le code compile, les types passent, le composant rend la chaîne attendue.
 * C'est un défaut lexical, comme les classes Tailwind interpolées : il se voit
 * en lisant le source, jamais en l'exécutant.
 *
 * ── Pourquoi ce cliquet couvre DEUX formes ────────────────────────────────
 *
 * Le premier relevé (#833) ne comptait que `errorMessage:` / `successMessage:`,
 * les options de `withErrorHandling`. Seize occurrences, résorbées. Mais un
 * cliquet posé là aurait affiché zéro tout en laissant passer **vingt** appels
 * directs à `toast.success(...)` / `toast.error(...)`, invisibles pour lui.
 *
 * Une garde verte parce qu'elle regarde à côté vaut moins que pas de garde :
 * elle fait croire que la dette est éteinte. C'est le motif dominant des
 * défauts de ce produit — la capacité écrite, testée et inatteignable — appliqué
 * à la vérification elle-même. Les deux portes sont donc surveillées ensemble.
 *
 * ── Ce qui reste autorisé ─────────────────────────────────────────────────
 *
 * Un littéral sans espace : `toast.error(err.code)` n'est pas une phrase. Et
 * tout ce qui passe par `$_(...)` ou `get(_)(...)`, évidemment.
 *
 * Le compte est à ZÉRO et ne doit pas remonter : contrairement à la dette
 * d'ancrage de #803, celle-ci est éteinte, on n'en tolère donc pas le retour.
 */

/** Messages d'interface écrits en dur. **Ne doit pas remonter au-dessus de 0.** */
const DETTE_AU_2026_09_07 = 0;

const RACINE = join(process.cwd(), "src");
const EXTENSIONS = new Set([".svelte", ".ts", ".astro"]);

/**
 * Les deux portes par lesquelles un message atteint l'écran :
 * l'option de `withErrorHandling`, et l'appel direct au store de toasts.
 *
 * Le littéral doit contenir une espace : c'est ce qui distingue une phrase
 * destinée à un humain d'un identifiant technique.
 */
const MESSAGE_EN_DUR =
  /(?:(?:success|error)Message:|toast\.(?:success|error|info|warning)\()\s*(['"])((?:[^'"\\]|\\.)*\s(?:[^'"\\]|\\.)*)\1/g;

function fichiersDeSource(repertoire: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(repertoire)) {
    if (entree === "node_modules" || entree.startsWith(".")) continue;
    const chemin = join(repertoire, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersDeSource(chemin));
    } else if (
      EXTENSIONS.has(extname(entree)) &&
      !entree.includes(".test.") &&
      !chemin.includes("__tests__")
    ) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

function recenser(): string[] {
  const fautes: string[] = [];
  for (const chemin of fichiersDeSource(RACINE)) {
    const texte = readFileSync(chemin, "utf8");
    for (const m of texte.matchAll(MESSAGE_EN_DUR)) {
      const ligne = texte.slice(0, m.index).split("\n").length;
      fautes.push(
        `${chemin.replace(process.cwd() + "/", "")}:${ligne} — « ${m[2].slice(0, 60)} »`,
      );
    }
  }
  return fautes;
}

describe("aucun message d'interface n'est écrit en dur (#833)", () => {
  it("ne laisse pas revenir de message non traduit", () => {
    const fautes = recenser();

    expect(
      fautes,
      `${fautes.length} message(s) d'interface écrits en dur.\n\n` +
        `Un message littéral ne traverse pas svelte-i18n : il s'affichera en ` +
        `français à un néerlandophone, qui ne pourra pas le lire.\n\n` +
        `Ajoutez la clé dans les QUATRE fichiers de \`src/locales/\` et ` +
        `appelez \`$_("section.cle")\` — ou \`get(_)("section.cle")\` hors ` +
        `composant, comme le fait \`lib/api.ts\`.\n\n` +
        fautes.join("\n"),
    ).toHaveLength(DETTE_AU_2026_09_07);
  });

  /// Sans ce contrôle, renommer le store de toasts rendrait le cliquet
  /// définitivement vert en ne trouvant plus rien à examiner.
  it("reconnaît encore les deux portes qu'il surveille", () => {
    const temoins = [
      `errorMessage: "Failed to load buildings"`,
      `toast.success("Assemblée créée avec succès")`,
      `toast.warning('Session expirée. Veuillez vous reconnecter.')`,
    ];
    for (const temoin of temoins) {
      expect([...temoin.matchAll(MESSAGE_EN_DUR)], temoin).toHaveLength(1);
    }
    // Et ne crie pas sur ce qui est légitime.
    const legitimes = [
      `errorMessage: $_('invoices.loadBuildingsFailed')`,
      `toast.error(get(_)("session.serverError"))`,
      `toast.error(err.code)`,
    ];
    for (const bon of legitimes) {
      expect([...bon.matchAll(MESSAGE_EN_DUR)], bon).toHaveLength(0);
    }
  });
});
