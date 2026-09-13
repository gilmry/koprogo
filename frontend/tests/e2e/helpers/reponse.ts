import { expect } from "@playwright/test";
import type { Response } from "@playwright/test";

/**
 * Affirme un code de réponse **en citant le motif du serveur quand il diffère**.
 *
 * ── Pourquoi cet utilitaire existe ─────────────────────────────────────────
 *
 * Le run du 2026-09-07 comptait dix-huit échecs Playwright. Trois d'entre eux
 * ne disaient rien de plus que :
 *
 *     Expected: 201
 *     Received: 400
 *
 * Le serveur, lui, avait répondu précisément — `skill_use_cases.rs:47` renvoie
 * `REFUS_RESERVE_AUX_COPROPRIETAIRES`, un message écrit exprès pour être lu.
 * Le test le jetait.
 *
 * C'est le même défaut que celui des recettes navigateur, retourné contre la
 * vérification : le serveur nomme le champ fautif, et la couche qui l'affiche
 * ne retient que le code. Un échec qui ne dit pas pourquoi coûte un aller-retour
 * complet de CI pour être compris — trente-cinq minutes ici.
 *
 * ── Ce qu'il ne fait pas ───────────────────────────────────────────────────
 *
 * Il ne relâche aucune exigence : le code attendu reste le code attendu. Il ne
 * fait qu'ajouter au message d'échec ce que le serveur avait déjà dit.
 *
 * Le corps est lu de façon tolérante : une réponse sans JSON, vide, ou déjà
 * consommée ne doit pas transformer un échec clair en erreur de l'utilitaire.
 */
export async function attendCode(
  reponse: Response,
  attendu: number,
  contexte?: string,
): Promise<void> {
  const obtenu = reponse.status();
  if (obtenu === attendu) return;

  let motif = "";
  try {
    const corps = await reponse.text();
    if (corps) {
      try {
        const json = JSON.parse(corps);
        motif = json.error ?? json.message ?? corps;
        if (json.details)
          motif += ` | details: ${JSON.stringify(json.details)}`;
      } catch {
        motif = corps;
      }
    }
  } catch {
    motif = "(corps illisible)";
  }

  const ou = contexte ? `${contexte} — ` : "";
  expect(
    obtenu,
    `${ou}${reponse.request().method()} ${reponse.url()}\n` +
      `Le serveur a répondu ${obtenu} au lieu de ${attendu}.\n` +
      `Motif : ${motif.slice(0, 400) || "(aucun corps)"}`,
  ).toBe(attendu);
}
