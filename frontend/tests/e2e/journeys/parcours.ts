/**
 * Le parcours de référence — la source de vérité partagée.
 *
 * ── Pourquoi ce fichier existe ────────────────────────────────────────────
 *
 * `skills/documentation-vivante.md` de la Méthode Foyer pose quatre éléments
 * non optionnels de la preuve de valeur. Le premier :
 *
 * > « Une source de vérité partagée — le parcours. Le parcours de référence
 * > est un **artefact unique**, consommé **par les deux harnais**. Pas de
 * > copie du parcours dans le test *et* dans la doc. Si le parcours n'est pas
 * > le même objet des deux côtés, on a recréé la double-maintenance que ce
 * > skill est censé éliminer. »
 *
 * Un `Parcours` est donc une liste d'étapes portant chacune **une description
 * en langage métier**, **une action** et **une assertion**. Deux harnais le
 * rejouent, et lisent la même chose différemment :
 *
 *   - le gate E2E    → à la vitesse, rend vert ou rouge      (correctness)
 *   - la vitrine     → en cadence, narré, rend une galerie   (valeur)
 *
 * ── Ce que ce fichier n'est pas ───────────────────────────────────────────
 *
 * Ce n'est pas un helper de confort. C'est l'artefact dont l'invariant
 * anti-dette (`garde-parcours-partage.test.ts`) vérifie qu'il est bien importé
 * des deux côtés. Un harnais de valeur qui cesserait de l'importer ferait
 * passer la suite au rouge — c'est ce qui rend la dette de documentation
 * structurellement impossible, au lieu de compter sur la discipline.
 */
import type { Page } from "@playwright/test";
import type { Scene } from "./scene";

/** Un rôle du produit, tel qu'un humain le nommerait. */
export type Acteur =
  | "syndic"
  | "copropriétaire"
  | "comptable"
  | "administrateur"
  | "conseil"
  | "prestataire";

export interface Etape {
  /** Identifiant stable, pour retrouver l'étape dans un rapport. */
  readonly id: string;
  /**
   * Ce que l'étape fait, **en langage métier**. C'est ce texte qui est
   * incrusté dans la vidéo et qui devient un chapitre de la galerie : il
   * s'adresse à quelqu'un qui ne lira jamais le code.
   */
  readonly description: string;
  /**
   * Qui agit. Une bascule d'acteur est narrée explicitement — la règle 9 de
   * `CRITICAL.md` exige les bons acteurs, et une vidéo est le seul endroit où
   * cette exigence devient vérifiable d'un regard.
   */
  readonly acteur: Acteur;
  /** L'action, exprimée sur la scène (qui porte la cadence et la narration). */
  readonly action: (scene: Scene) => Promise<void>;
  /**
   * Ce qui doit être vrai après. Facultatif : certaines étapes ne font que
   * mettre en place. Une étape sans assertion ne prouve rien — mais une étape
   * de mise en place qui prétendrait prouver serait pire.
   */
  readonly assertion?: (page: Page) => Promise<void>;
}

export interface Parcours {
  /** Identifiant de fichier et de vidéo. */
  readonly slug: string;
  /** Titre lisible, en tête de la carte de galerie. */
  readonly titre: string;
  /** Ce que ce parcours démontre, et pour qui. */
  readonly propos: string;
  readonly etapes: readonly Etape[];
}

/**
 * Les acteurs distincts d'un parcours, dans l'ordre d'apparition.
 *
 * Sert au contrôle de la règle 9 : un parcours multi-rôles qui n'aurait qu'un
 * seul acteur est un parcours mono-rôle qui s'ignore.
 */
export function acteursDe(parcours: Parcours): Acteur[] {
  const vus: Acteur[] = [];
  for (const e of parcours.etapes) {
    if (!vus.includes(e.acteur)) vus.push(e.acteur);
  }
  return vus;
}
