import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Garde-fou : `on:evenement` sur un composant qui ne l'emet pas.
 *
 * ── Le defaut, et pourquoi il est invisible ──────────────────────────────
 *
 * En Svelte 5, `on:click` sur un ELEMENT NATIF (`<button on:click>`) est
 * deprecie mais fonctionne. Sur un COMPOSANT (`<Button on:click>`), c'est un
 * abonnement a un evenement de composant : il ne se declenche que si le
 * composant appelle `dispatch('click')`.
 *
 * `Button` est en mode runes et diffuse `...restProps` sur le `<button>`
 * natif. Il n'emet aucun evenement. Les `on:click` poses sur lui n'etaient
 * donc JAMAIS appeles — sans erreur, sans avertissement, sans requete reseau.
 * Un bouton qui ne fait rien et ne dit rien.
 *
 * Constate sur `/admin/organizations` (#662), puis retrouve sur cinq boutons
 * de `MeetingDetail` : cloturer, annuler et reprogrammer une assemblee. Trois
 * actions de gouvernance inertes.
 *
 * ── Ce que ce test distingue, et pourquoi c'est necessaire ───────────────
 *
 * Tout `on:` sur un composant n'est pas casse. `BudgetCreateForm` appelle bien
 * `dispatch('created')` et `dispatch('cancel')` : ses `on:created` et
 * `on:cancel` fonctionnent. Les signaler serait pousser a "corriger" du code
 * juste, ce qui est pire que de ne rien signaler.
 *
 * Le test ne remonte donc que le cas reellement casse : un `on:x` sur un
 * composant dont le fichier ne contient aucun `dispatch('x')`.
 *
 * ── Sa limite, dite plutot que masquee ───────────────────────────────────
 *
 * L'analyse est textuelle. Un composant qui emettrait un evenement au nom
 * calcule passerait entre les mailles, et un `dispatch` en commentaire
 * suffirait a faire taire l'alerte. C'est un filet, pas une preuve.
 */

const SRC = join(import.meta.dirname ?? __dirname, "..", "..");

function fichiersSvelte(racine: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersSvelte(chemin));
    } else if (entree.endsWith(".svelte")) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

/** Le nom de la balise ouvrante qui porte la directive, s'il y en a une. */
function balisePortante(source: string, position: number): string | null {
  const debut = source.lastIndexOf("<", position);
  if (debut < 0) return null;
  const nom = /^<([A-Za-z][\w.]*)/.exec(source.slice(debut));
  return nom ? nom[1] : null;
}

/** Un composant emet-il cet evenement ? Recherche textuelle du dispatch. */
function emetLevenement(cheminComposant: string, evenement: string): boolean {
  try {
    const source = readFileSync(cheminComposant, "utf8");
    return (
      source.includes(`dispatch('${evenement}'`) ||
      source.includes(`dispatch("${evenement}"`)
    );
  } catch {
    // Composant introuvable : on ne peut rien affirmer, on ne signale pas.
    return true;
  }
}

/** Resout `<Nom>` vers son fichier, via l'import qui l'amene. */
function fichierDuComposant(source: string, nom: string, depuis: string): string | null {
  const imp = new RegExp(
    `import\\s+${nom}\\s+from\\s+["']([^"']+)["']`,
  ).exec(source);
  if (!imp) return null;
  const cible = imp[1];
  if (!cible.startsWith(".")) return null;
  const base = join(depuis, "..", cible);
  return base.endsWith(".svelte") ? base : `${base}.svelte`;
}

describe("directives d'evenement Svelte 5", () => {
  it("aucun `on:x` sur un composant qui n'emet pas `x`", () => {
    const casses: string[] = [];

    for (const fichier of fichiersSvelte(join(SRC, "components"))) {
      const source = readFileSync(fichier, "utf8");
      for (const m of source.matchAll(/on:([\w]+)[=|]/g)) {
        const balise = balisePortante(source, m.index!);
        if (!balise || !/^[A-Z]/.test(balise)) continue; // element natif : OK

        const cible = fichierDuComposant(source, balise, fichier);
        if (!cible) continue; // import externe ou dynamique : on s'abstient

        if (!emetLevenement(cible, m[1])) {
          const ligne = source.slice(0, m.index!).split("\n").length;
          casses.push(
            `  ${fichier.replace(SRC, "src")}:${ligne} — <${balise} on:${m[1]}> ` +
              `mais ${balise} n'emet jamais '${m[1]}'`,
          );
        }
      }
    }

    expect(
      casses,
      `Directives d'evenement silencieusement inoperantes :\n${casses.join("\n")}\n\n` +
        `Sur un COMPOSANT, \`on:x\` s'abonne a un evenement de composant. Si le ` +
        `composant ne fait pas \`dispatch('x')\`, le gestionnaire n'est jamais ` +
        `appele — sans erreur, sans avertissement. Utiliser la prop \`onx\` ` +
        `(ex. \`onclick\`), que le composant diffusera via \`...restProps\`.`,
    ).toEqual([]);
  });
});
