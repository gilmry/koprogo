/**
 * Les cahiers des charges techniques — la liste dit-elle la vérité ?
 *
 * ── Le défaut que cette recette fige ──────────────────────────────────────
 *
 * Le balayage du 2026-09-19 a relevé sur `/syndic/technical-specs` un
 * **400** en console pour 492 caractères rendus (#968). En remontant :
 *
 *   1. `acp_id` est OBLIGATOIRE côté serveur
 *      (`ListTechnicalSpecsQuery.acp_id: Uuid`, non-`Option`) ;
 *   2. `TechnicalSpecsPage.svelte` appelait `listSpecs()` **sans argument**,
 *      en parallèle du chargement des ACP — donc avant de savoir laquelle ;
 *   3. le refus était avalé par un `.catch(() => [])`.
 *
 * `technical_specs.ts:147` documentait pourtant le piège en toutes lettres :
 * « appeler cette fonction SANS `acpId` échoue toujours ». Le commentaire
 * était juste, l'appel ne l'a jamais suivi.
 *
 * Résultat : l'écran annonçait « aucun cahier des charges » à un syndic qui
 * en avait. **Vide et cassé se ressemblent**, et seule la console les
 * distinguait.
 *
 * ── Ce que la recette vérifie ─────────────────────────────────────────────
 *
 * Que la liste reflète ce que le serveur possède, et surtout : qu'un refus
 * se DISE. Un `catch` qui rend une valeur plausible est plus coûteux qu'une
 * exception, parce qu'il produit un écran crédible et faux.
 */
import { test, expect } from "@playwright/test";
import { loginAsSyndic, ensureAcp } from "./helpers/auth";
import { API_BASE } from "./helpers/adresses";

test.describe("Cahiers des charges techniques (#968)", () => {
  test("@happy la liste se charge sans 400, une fois l'ACP connue", async ({
    page,
  }) => {
    const erreurs: string[] = [];
    page.on("console", (m) => {
      if (m.type() === "error") erreurs.push(m.text());
    });
    page.on("pageerror", (e) => erreurs.push(String(e)));

    const contexte = await loginAsSyndic(page, "cdc");
    await ensureAcp(page, contexte.orgId, contexte.adminToken, "cdc");

    await page.goto("/syndic/technical-specs");

    // Ni l'un ni l'autre ne doit rester : le chargement aboutit.
    await expect(page.getByTestId("tech-spec-new-button")).toBeVisible({
      timeout: 20000,
    });

    // ── Ce que ce test ne vérifie PAS, et pourquoi ───────────────────────
    //
    // Il ne vérifie pas que des cahiers des charges s'affichent. En essayant,
    // j'ai découvert que le syndic amorcé ici **ne voit aucune ACP** :
    // `listAcps()` rend une liste vide alors que `ensureAcp` vient d'en créer
    // une pour son organisation. C'est le défaut de cloisonnement user↔ACP
    // (#694), pas celui que cette recette fige.
    //
    // Exiger une ACP ici ferait échouer ce test pour une cause étrangère, et
    // le rendrait illisible : on croirait la régression revenue alors qu'un
    // autre chantier serait en cause. Le cas peuplé appartient à #694.
    //
    // Ce qui est vérifié tient en une phrase : **l'écran ne part plus
    // chercher les cahiers des charges sans savoir de quelle ACP il parle.**
    // C'est exactement le 400 relevé par le balayage.

    // Le point central : un refus du serveur ne doit JAMAIS se présenter
    // comme une liste vide. Si `tech-spec-list-error` est visible, la recette
    // le dit plutôt que de passer au vert sur un écran crédible et faux.
    const enErreur = await page.getByTestId("tech-spec-list-error").count();
    if (enErreur > 0) {
      const message = await page
        .getByTestId("tech-spec-list-error")
        .innerText();
      throw new Error(
        `La liste a refusé de se charger, et le dit — c'est déjà mieux que ` +
          `de mentir, mais ce n'est pas l'état attendu : ${message}`,
      );
    }

    const quatreCents = erreurs.filter((e) => /\b400\b/.test(e));
    expect(
      quatreCents,
      `La liste part sans \`acp_id\` : ${quatreCents.slice(0, 2).join(" | ")}`,
    ).toEqual([]);
  });

  /**
   * ── Pourquoi le cas de refus n'est PAS ici ───────────────────────────────
   *
   * Je l'y avais écrit : interposer un 400 sur `/technical-specs` et vérifier
   * que l'écran le dit. Il ne se déclenchait jamais.
   *
   * L'instrumentation a tranché — la page ne produit **aucune requête
   * visible** après navigation, alors que son état vide s'affiche, donc
   * `loadInitial()` a bien tourné. Le **service worker** de la PWA sert ces
   * appels, et `page.route()` ne l'intercepte pas.
   *
   * Lutter contre le service worker pour poser un bouchon aurait produit une
   * recette fragile qui éprouve surtout la couche de cache. Le rendu d'un
   * refus se vérifie au niveau du COMPOSANT, où il est déterministe :
   * `TechnicalSpecsPage.erreur.test.ts`.
   *
   * Choisir l'outil qui rend un verdict stable n'est pas renoncer à la
   * preuve — c'est refuser une preuve qui n'en serait pas une.
   */
});
