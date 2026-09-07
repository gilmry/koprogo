import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * L'écran RGPD ne parle pas anglais à un utilisateur francophone.
 *
 * ── Pourquoi c'est une exigence légale, pas de confort ─────────────────────
 *
 * RGPD Art. 12 § 1 : l'information doit être fournie « d'une façon concise,
 * transparente, compréhensible et aisément accessible, en des termes clairs et
 * simples ». Une page où un copropriétaire lit ses droits dans une langue
 * qu'il ne maîtrise pas ne satisfait pas cette exigence.
 *
 * Recette du 2026-09-06 (RN-4) : quatre intitulés de droits sur cinq et la
 * totalité des boutons d'action étaient en anglais, dont
 * « Request Data Erasure » — qui déclenche un effacement **irréversible**.
 *
 * C'est le seul écran de l'application où la langue engage la conformité.
 *
 * ── Ce que ce test fait ────────────────────────────────────────────────────
 *
 * Il refuse tout texte visible en dur dans le gabarit. Vérification lexicale,
 * adaptée au défaut : un test de rendu passerait aussi bien avec de l'anglais,
 * puisqu'il rend fidèlement ce qu'on lui donne.
 *
 * Voir #774.
 */

const PANNEAU = join(process.cwd(), "src/components/GdprDataPanel.svelte");

/**
 * Des mots qui n'apparaissent en français que par accident.
 *
 * La liste est volontairement étroite : un garde-fou qui crie à tort finit
 * désactivé. On vise les mots-outils, absents du français, plutôt que les
 * termes techniques qui se ressemblent dans les deux langues.
 */
const MOTS_ANGLAIS =
  /\b(your|you|will|the|and|of|to be|have been|cannot|records?|ownerships?|eligibility|successfully|immediately|placeholders?|undone)\b/i;

describe("l'écran RGPD est traduit (#774)", () => {
  const source = readFileSync(PANNEAU, "utf8");

  it("ne laisse aucun texte anglais en dur dans le gabarit", () => {
    // Le gabarit commence après le bloc <script>.
    const gabarit = source.slice(source.indexOf("</script>"));

    const fautes: string[] = [];
    for (const [i, ligne] of gabarit.split("\n").entries()) {
      const nue = ligne.trim();
      if (nue.startsWith("<!--") || nue.startsWith("//")) continue;
      // Le texte entre balises, hors interpolations et hors attributs.
      for (const m of ligne.matchAll(/>([^<>{}\n]{6,})</g)) {
        const texte = m[1].trim();
        if (!texte) continue;
        if (MOTS_ANGLAIS.test(texte)) {
          fautes.push(`ligne ${i + 1} : ${texte.slice(0, 70)}`);
        }
      }
    }

    expect(
      fautes,
      `${fautes.length} texte(s) anglais en dur sur l'écran RGPD.\n\n` +
        `RGPD Art. 12 § 1 : l'information doit être « compréhensible [...] en ` +
        `des termes clairs et simples ». Le bouton d'effacement déclenche une ` +
        `action irréversible ; un utilisateur qui ne comprend pas son ` +
        `intitulé peut le cliquer par curiosité, ou ne jamais exercer un ` +
        `droit qu'il n'a pas compris avoir (#774).\n\n` +
        `Passez par \`$_('gdpr.…')\` et ajoutez la clé aux QUATRE langues.\n\n` +
        fautes.join("\n"),
    ).toEqual([]);
  });

  /// Le gabarit n'est pas le seul endroit où du texte atteint l'utilisateur.
  ///
  /// La première version de ce test ne scannait que le gabarit — ce qui est
  /// entre `>` et `<`. Elle laissait donc passer **dix messages anglais** dans
  /// la partie script, ceux que `withErrorHandling` affiche en toast :
  ///
  ///     successMessage: 'Your personal data has been exported successfully'
  ///     errorMessage: 'Failed to erase data'
  ///
  /// Un utilisateur qui déclenche un effacement irréversible lisait la
  /// confirmation en anglais. Le garde-fou passait, et le défaut restait —
  /// exactement le motif qu'il existe pour empêcher (#774).
  it("ne laisse aucun message de toast en dur", () => {
    const enDur = [
      ...source.matchAll(/(success|error)Message:\s*['"]([A-Z][^'"]{6,})['"]/g),
    ].map((m) => `${m[1]}Message: "${m[2]}"`);

    expect(
      enDur,
      `${enDur.length} message(s) de toast écrits en dur dans le script.\n\n` +
        `Ils atteignent l'utilisateur comme le reste, mais échappent à un ` +
        `contrôle qui ne regarde que le gabarit. Sur cet écran, l'un d'eux ` +
        `confirme un effacement irréversible (#774).\n\n` +
        `Passez par \`$_('gdpr.…')\` et ajoutez la clé aux QUATRE langues.\n\n` +
        enDur.join("\n"),
    ).toEqual([]);
  });

  it("emploie bien l'internationalisation", () => {
    const appels = (source.match(/\$_\(/g) ?? []).length;
    expect(
      appels,
      "L'écran RGPD n'appelle plus la traduction : le texte est probablement " +
        "revenu en dur.",
    ).toBeGreaterThan(30);
  });

  /// Une clé ajoutée dans une seule langue laisse les trois autres afficher
  /// la clé brute — le défaut que #786 a montré avec `notices.draft`.
  it("déclare ses clés dans les quatre langues", () => {
    const langues = ["fr", "en", "nl", "de"].map((l) =>
      JSON.parse(
        readFileSync(join(process.cwd(), `src/locales/${l}.json`), "utf8"),
      ),
    );

    const clesUtilisees = [...source.matchAll(/\$_\('gdpr\.([\w.]+)'/g)].map(
      (m) => m[1],
    );
    expect(clesUtilisees.length).toBeGreaterThan(10);

    const manquantes: string[] = [];
    for (const cle of new Set(clesUtilisees)) {
      for (const [i, dico] of langues.entries()) {
        const valeur = cle
          .split(".")
          .reduce((o: any, k) => (o ? o[k] : undefined), dico.gdpr);
        if (valeur === undefined) {
          manquantes.push(`${["fr", "en", "nl", "de"][i]} : gdpr.${cle}`);
        }
      }
    }

    expect(
      manquantes,
      `Des clés RGPD manquent dans certaines langues. L'utilisateur y verra ` +
        `la clé brute au lieu du texte — c'est ce que #786 a montré avec ` +
        `\`notices.draft\` affiché en clair.\n\n` +
        manquantes.join("\n"),
    ).toEqual([]);
  });
});
