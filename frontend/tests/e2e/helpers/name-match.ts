/**
 * Correspondance de noms insensible aux accents, avec échec BRUYANT.
 *
 * ## Pourquoi ce module existe
 *
 * Le monde de scénario est semé par le backend sous le nom
 * « Résidence du Parc Royal » (`seed.rs:3133`). Sept scénarios le cherchaient
 * avec `"Residence du Parc"` — SANS accent. En JavaScript :
 *
 *     "Résidence du Parc Royal".includes("Residence du Parc")  // false
 *
 * La comparaison échouait donc systématiquement. Et elle échouait EN SILENCE,
 * ce qui est le vrai défaut :
 *
 *     for (const option of options) {
 *       if (text?.includes("Residence du Parc")) { ... break; }
 *     }
 *     // aucune correspondance -> la boucle se termine normalement,
 *     // `selectOption` n'est jamais appelé, le formulaire reste vide
 *
 *     const building = buildings.find((b) => b.name?.includes("Residence…"));
 *     if (building) { ...toute la préparation des données... }
 *     // pas de building -> le bloc entier est sauté, sans un mot
 *
 * Conséquence observée en CI (trace du run 33250007102) : le formulaire de
 * budget est rempli, `budget-submit-button` est cliqué, mais AUCUN
 * `POST /budgets` ne part — la validation client bloque sur l'immeuble non
 * sélectionné. Le test échoue 15 s plus tard sur `budget-row` absent, en
 * accusant l'affichage. Le vrai message n'est jamais lu.
 *
 * Ces helpers corrigent les deux moitiés du problème : ils normalisent les
 * accents, ET ils lèvent une erreur nommant les candidats disponibles quand
 * rien ne correspond. Un test qui ne trouve pas sa donnée doit le dire —
 * c'est la règle déjà posée par `building.ts` (« fail fast — no silent
 * timeout »).
 */
import type { Locator, Page } from "@playwright/test";

/**
 * Normalise pour comparaison : minuscules, sans diacritiques, espaces réduits.
 *
 * Même transformation NFD que `slugify()` dans `RoleSubmenu.svelte` — les deux
 * côtés du dépôt doivent traiter les accents de la même façon, sous peine de
 * reproduire exactement le bug que ce module corrige.
 */
export function normalizeName(value: string): string {
  return value
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

/** `true` si `haystack` contient `needle`, accents et casse ignorés. */
export function nameContains(haystack: string, needle: string): boolean {
  return normalizeName(haystack).includes(normalizeName(needle));
}

/**
 * Trouve un élément par son nom, accents ignorés. Lève si absent.
 *
 * @param items  collection issue de l'API
 * @param needle fragment de nom recherché
 * @param label  contexte affiché dans l'erreur (nom du scénario, de l'étape…)
 */
export function findByName<T extends { name?: string | null }>(
  items: unknown,
  needle: string,
  label: string,
): T {
  if (!Array.isArray(items)) {
    throw new Error(
      `${label}: réponse inattendue de l'API — tableau attendu, reçu ${typeof items}`,
    );
  }
  const found = (items as T[]).find((it) =>
    it?.name ? nameContains(it.name, needle) : false,
  );
  if (!found) {
    const available = (items as T[])
      .map((it) => it?.name ?? "(sans nom)")
      .join(", ");
    throw new Error(
      `${label}: aucun élément nommé « ${needle} ». ` +
        `Disponibles : ${available || "(collection vide)"}. ` +
        `Le monde de scénario est-il semé ?`,
    );
  }
  return found;
}

/**
 * Sélectionne dans un `<select>` l'option dont le libellé contient `needle`,
 * accents ignorés. Lève si aucune option ne correspond.
 *
 * Remplace la boucle `for…of` + `break` qui, sans correspondance, laissait le
 * `<select>` vide sans rien signaler.
 */
export async function selectOptionByName(
  select: Locator,
  needle: string,
  label: string,
): Promise<string> {
  const options = await select.locator("option").all();
  const seen: string[] = [];

  for (const option of options) {
    const text = (await option.textContent()) ?? "";
    seen.push(text.trim());
    if (text && nameContains(text, needle)) {
      const value = await option.getAttribute("value");
      if (value) {
        await select.selectOption(value);
        return value;
      }
    }
  }

  throw new Error(
    `${label}: aucune option contenant « ${needle} » dans le <select>. ` +
      `Options présentes : ${seen.filter(Boolean).join(", ") || "(aucune)"}. ` +
      `Sans sélection, la validation client bloque la soumission et le test ` +
      `échouera plus loin sur une liste vide, en accusant l'affichage.`,
  );
}

/**
 * Variante tolérante : sélectionne si le `<select>` est présent ET qu'une
 * option correspond, sans lever si le `<select>` lui-même est absent.
 *
 * Réservée aux endroits où le sélecteur d'immeuble est légitimement optionnel
 * (rôle sans scope building). L'absence d'OPTION correspondante reste, elle,
 * une erreur — c'est la distinction que le code d'origine ne faisait pas.
 */
export async function selectOptionByNameIfPresent(
  page: Page,
  select: Locator,
  needle: string,
  label: string,
  timeoutMs = 5000,
): Promise<string | null> {
  if (!(await select.isVisible({ timeout: timeoutMs }).catch(() => false))) {
    return null;
  }
  await select.scrollIntoViewIfNeeded();
  await page.waitForTimeout(200);
  return selectOptionByName(select, needle, label);
}
