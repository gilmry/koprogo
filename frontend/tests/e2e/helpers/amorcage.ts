import { expect, type APIResponse, type Page } from "@playwright/test";

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

/**
 * Refuse qu'une écriture pilotée à l'écran échoue en silence.
 *
 * Les scénarios cliquent « soumettre » et enchaînent sans rien vérifier.
 * `budget-workflow` créait ainsi son budget, ne regardait pas le résultat, et
 * échouait quinze lignes plus loin sur `text=2026` introuvable — en accusant
 * la liste, alors que le budget n'avait jamais été enregistré.
 *
 * `ToastContainer.svelte` porte désormais `toast-{type}`, donc `toast-error`
 * pour un échec. On lit son message ET son détail (`toast-details`, où le
 * serveur nomme le champ fautif) pour que l'échec dise ce qui s'est passé.
 *
 * À appeler après chaque geste d'écriture. Ne remplace pas l'assertion sur le
 * résultat attendu : elle explique celle-ci quand elle tombe.
 */
export async function aucuneErreurAffichee(
  page: Page,
  quoi: string,
): Promise<void> {
  const erreur = page.getByTestId("toast-error").first();
  if (await erreur.isVisible({ timeout: 2000 }).catch(() => false)) {
    const message = (await erreur.textContent().catch(() => "")) ?? "";
    const detail =
      (await page
        .getByTestId("toast-details")
        .first()
        .textContent()
        .catch(() => "")) ?? "";
    expect(
      false,
      `« ${quoi} » a échoué à l'écran : ${message.trim()}\n${detail.trim()}\n\n` +
        `Sans cette vérification, le scénario aurait continué et échoué plus ` +
        `loin sur une liste vide, en accusant l'affichage.`,
    ).toBe(true);
  }
}

/**
 * Confirme la boîte de dialogue si l'écran en ouvre une.
 *
 * #844 a remplacé soixante `confirm()` natifs par de vraies modales. Un
 * navigateur piloté SUPPRIME les dialogues natifs — le geste passait donc
 * tout seul, et les scénarios n'ont jamais eu à confirmer quoi que ce soit.
 * Depuis la conversion, la modale reste ouverte et bloque la page.
 *
 * Constaté sur la capture d'écran d'`expense-approval` : « Êtes-vous sûr de
 * vouloir soumettre cette facture pour approbation ? », Annuler / Confirmer,
 * et le scénario qui attend derrière un bouton d'approbation qu'il ne verra
 * jamais.
 *
 * Le clic est fait comme un utilisateur le ferait, pas en forçant l'état :
 * ces scénarios sont enregistrés en vidéo comme documentation vivante, et la
 * confirmation fait partie du parcours réel.
 */
export async function confirmerSiDemande(page: Page): Promise<boolean> {
  const bouton = page.getByTestId("confirm-dialog-confirm");
  if (await bouton.isVisible({ timeout: 2000 }).catch(() => false)) {
    await bouton.click();
    await page.waitForTimeout(300);
    return true;
  }
  return false;
}
