/**
 * Le délai de convocation d'une assemblée générale (Art. 3.87 § 3).
 *
 * ── Pourquoi cette règle existe aussi côté navigateur ──────────────────────
 *
 * Le domaine la porte déjà (`domain/copropriete/delai_de_convocation.rs`), et
 * l'API sert le verdict sur chaque assemblée. Mais au moment où un syndic
 * **saisit** une date, l'assemblée n'existe pas encore : il n'y a rien à
 * interroger.
 *
 * Or c'est exactement l'instant où l'avertissement sert. Recette du
 * 2026-09-06 (RN-9) : une assemblée créée pour le 20 septembre, puis
 * « Créer une convocation » qui refuse — la règle est juste, sa temporalité
 * ne l'est pas. Le syndic n'en sortait qu'en supprimant l'assemblée.
 *
 * ── Le risque de cette duplication, et comment il est tenu ─────────────────
 *
 * Une règle écrite deux fois finit par diverger — c'est précisément ce qui a
 * produit #773, où l'écran comptait des têtes pendant que le serveur comptait
 * des voix. `delai-convocation.test.ts` lit donc `minimum_notice_days` **dans
 * la source Rust** et refuse que les deux nombres s'écartent.
 *
 * Voir #780, verrou 1.
 */

/**
 * « La convocation est communiquée quinze jours au moins avant la date de
 * l'assemblée » — Art. 3.87 § 3.
 *
 * Le même délai vaut pour les trois types depuis la loi de 2019, seconde
 * convocation comprise (§ 5).
 */
export const JOURS_DE_PREAVIS = 15;

const MS_PAR_JOUR = 86_400_000;

export type VerdictDelai =
  | { etat: "tenable"; dateLimiteEnvoi: Date }
  | { etat: "trop-court"; dateLimiteEnvoi: Date; joursManquants: number }
  | { etat: "deja-tenue" };

/**
 * Le délai est-il tenable pour une assemblée à cette date ?
 *
 * `maintenant` est un paramètre : une fonction qui lit l'horloge ne se teste
 * qu'en attendant.
 *
 * **Ne dit jamais non.** L'urgence est prévue par le texte lui-même, une
 * assemblée peut être encodée après coup, et une seconde convocation subit la
 * date de l'échec précédent. Ce verdict avertit, il n'interdit pas.
 */
export function evaluerDelai(
  dateAssemblee: Date,
  maintenant: Date = new Date(),
): VerdictDelai {
  if (Number.isNaN(dateAssemblee.getTime())) return { etat: "deja-tenue" };
  if (dateAssemblee.getTime() <= maintenant.getTime()) {
    return { etat: "deja-tenue" };
  }

  const dateLimiteEnvoi = new Date(
    dateAssemblee.getTime() - JOURS_DE_PREAVIS * MS_PAR_JOUR,
  );

  if (maintenant.getTime() <= dateLimiteEnvoi.getTime()) {
    return { etat: "tenable", dateLimiteEnvoi };
  }

  const manqueMs = maintenant.getTime() - dateLimiteEnvoi.getTime();
  const joursManquants = Math.max(1, Math.ceil(manqueMs / MS_PAR_JOUR));
  return { etat: "trop-court", dateLimiteEnvoi, joursManquants };
}
