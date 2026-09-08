import type { APIResponse } from "@playwright/test";

/**
 * Vérifie qu'un appel d'amorçage a réussi, et dit pourquoi sinon.
 *
 * Les douze scénarios construisent leur monde par API dans `beforeAll` :
 * annonces, sondages, devis, échanges, factures, budgets. Le 2026-09-08,
 * **aucune de ces 55 réponses n'était vérifiée**.
 *
 * Ce que cela produit, mesuré sur le run du même jour :
 *
 *     POST /notices  →  400
 *     notice.id      →  undefined
 *     POST /notices/undefined/publish
 *     ... 200 lignes plus loin ...
 *     expect(getByTestId('notice-list-row')).toBeVisible()  →  échec
 *
 * Onze scénarios sur douze échouaient ainsi, tous sur une liste vide, tous
 * en accusant l'affichage. Les captures d'écran montrent une application
 * parfaitement fonctionnelle, le bon immeuble sélectionné, et « Aucun
 * sondage créé ». Le défaut n'était pas là où le message le disait.
 *
 * C'est le motif dominant de ce dépôt — une erreur jetée en amont, un
 * symptôme sans rapport en aval — appliqué cette fois aux recettes
 * elles-mêmes.
 */
export async function amorce(reponse: APIResponse, quoi: string): Promise<any> {
  if (!reponse.ok()) {
    const corps = await reponse.text().catch(() => "<corps illisible>");
    throw new Error(
      `Amorçage « ${quoi} » : ${reponse.status()} ${reponse.statusText()}\n` +
        `${corps.slice(0, 500)}\n\n` +
        `Le scénario ne peut pas se dérouler sans cette donnée. Sans cette ` +
        `vérification, l'échec serait apparu bien plus loin, sur une liste ` +
        `vide, en accusant l'affichage.`,
    );
  }
  return reponse.json().catch(() => ({}));
}

/**
 * Variante pour les appels dont l'échec est acceptable — un nettoyage, ou
 * une donnée qui peut déjà exister. Elle ne lève pas, mais elle TRACE :
 * un amorçage silencieux qui échoue reste la cause d'un symptôme lointain.
 */
export async function amorceToleree(
  reponse: APIResponse,
  quoi: string,
): Promise<any> {
  if (!reponse.ok()) {
    const corps = await reponse.text().catch(() => "<corps illisible>");
    console.warn(
      `[amorçage toléré] « ${quoi} » : ${reponse.status()} — ${corps.slice(0, 200)}`,
    );
    return {};
  }
  return reponse.json().catch(() => ({}));
}
