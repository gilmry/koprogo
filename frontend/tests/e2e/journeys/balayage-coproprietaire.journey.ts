/**
 * Balayage complet du produit, vu par le copropriétaire.
 *
 * Le contenu vit dans `balayage.ts` : un seul mécanisme pour les quatre
 * rôles, parce qu'un balayage recopié quatre fois dériverait trois fois.
 * Ce fichier n'existe que pour donner au parcours un nom de vidéo stable,
 * comme les sept parcours métier.
 */
import { balayagePour } from "./balayage";

export const parcours = balayagePour("copropriétaire");
