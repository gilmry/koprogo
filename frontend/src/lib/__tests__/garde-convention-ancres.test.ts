import { describe, expect, it } from "vitest";
import contrat from "./data-testid.contrat.json";

/**
 * La convention de nommage des ancres, relevée puis tenue.
 *
 * ── Pourquoi une convention ───────────────────────────────────────────────
 *
 * #803 la demandait comme deuxième critère de fin : « une convention de
 * nommage écrite, avant d'en poser un seul ». Le dépôt en avait déjà une **de
 * fait** — il suffisait de la constater plutôt que de l'inventer.
 *
 * Relevé sur les 964 ancres du contrat figé :
 *
 * ```
 * forme      <domaine>-<objet>-<rôle>   516 sur 964 en trois segments
 * casse      kebab strict                961 sur 964
 * rôles      -button 99   -input 81   -list 65   -btn 63   -select 39
 * ```
 *
 * ── Ce que cette garde tient ──────────────────────────────────────────────
 *
 * **La casse.** Trois ancres reprennent un nom de champ d'API en casse
 * chameau. C'est défendable — l'ancre désigne l'erreur d'un champ précis — et
 * elles sont donc listées nommément. Une quatrième échouerait.
 *
 * **L'orthographe du rôle « bouton ».** `-button` et `-btn` désignent la même
 * chose, et cette dette oblige quiconque cherche un bouton à essayer les deux
 * formes. `-button` l'emporte au nombre ; le cliquet empêche `-btn` de
 * croître.
 *
 * Elle n'impose PAS le nombre de segments : deux suffisent souvent
 * (`owner-units`, `login-email`) et quatre sont parfois nécessaires
 * (`gdpr-erase-confirm-modal`). Compter les segments punirait la clarté.
 *
 * Suivi en #803.
 */

const ANCRES: string[] = contrat.litteraux;

/** Casse kebab stricte : minuscules, chiffres, tirets simples. */
const KEBAB = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

/**
 * Les trois ancres en casse chameau, et pourquoi.
 *
 * Elles reprennent le nom du champ d'API dont elles signalent l'erreur. Les
 * renommer couperait le lien entre le message et le champ ; les tolérer sans
 * les nommer laisserait la porte ouverte.
 */
const CAMEL_TOLERE = new Set([
  "mandate-error-scopeId",
  "mandate-error-validUntil",
  "tech-spec-create-error-requiredSignatures",
]);

/** Ancres en `-btn` plutôt qu'en `-button`. **Ne doit que BAISSER.** */
const DETTE_BTN_AU_2026_09_08 = 63;

describe("la convention de nommage des ancres (#803)", () => {
  it("écrit les ancres en casse kebab, aux trois exceptions près", () => {
    const hors = ANCRES.filter((a) => !KEBAB.test(a) && !CAMEL_TOLERE.has(a));
    expect(
      hors,
      `Ces ancres ne sont ni en casse kebab, ni parmi les trois exceptions ` +
        `tolérées.\n\n` +
        `La convention relevée sur 961 des 964 ancres est ` +
        `<domaine>-<objet>-<rôle>, en minuscules et tirets.\n\n` +
        `Si l'ancre reprend délibérément un nom de champ d'API, ajoutez-la ` +
        `à CAMEL_TOLERE avec sa raison.\n\n` +
        hors.join("\n"),
    ).toEqual([]);
  });

  it("n'ajoute plus d'ancre en -btn là où -button est la forme majoritaire", () => {
    const btn = ANCRES.filter((a) => a.endsWith("-btn"));
    expect(
      btn.length,
      `${btn.length} ancres se terminent en \`-btn\`, contre ` +
        `${DETTE_BTN_AU_2026_09_08} au 2026-09-08.\n\n` +
        `\`-button\` et \`-btn\` désignent la même chose : la coexistence des ` +
        `deux oblige quiconque cherche un bouton à essayer les deux formes. ` +
        `\`-button\` l'emporte au nombre (99 contre 63).\n\n` +
        `Écrivez \`-button\`.`,
    ).toBeLessThanOrEqual(DETTE_BTN_AU_2026_09_08);
  });

  /**
   * Sans ce contrôle, un contrat vidé rendrait les deux précédents verts
   * faute d'ancres à examiner.
   */
  it("lit encore le contrat figé", () => {
    expect(
      ANCRES.length,
      "moins de 500 ancres dans le contrat : le fichier a changé de forme, " +
        "ou il s'est vidé. Vérifiez avant de vous réjouir.",
    ).toBeGreaterThan(500);
  });
});
