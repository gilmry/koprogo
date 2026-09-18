import { api } from "../api";

/**
 * Recherche d'utilisateurs, côté SERVEUR.
 *
 * ## Pourquoi ce module existe
 *
 * `GET /users` rendait la table entière. Mesuré le 2026-09-18 sur la recette,
 * avec le même jeton et à la suite :
 *
 * ```text
 * GET /organizations   6 190 octets    18,8 ms   (paginée la veille)
 * GET /users       2 399 187 octets   667,0 ms   (4 120 lignes)
 * ```
 *
 * Un facteur 387 en volume, et c'est ce qui a fait dépasser les 30 s à deux
 * specs de la campagne du jour, toutes deux sur `/admin/users` (#953).
 *
 * Le plus trompeur : **quatre des cinq appelants envoyaient déjà
 * `per_page`**, et le serveur l'ignorait. Le code avait l'air correct des
 * deux côtés, et personne ne voyait qu'ils ne se rencontraient pas.
 *
 * Ce module est le seul endroit qui connaît cette forme : si la pagination
 * change encore, elle change ici. Jumeau de `organisations-recherche.ts`.
 */

/** Un utilisateur, réduit à ce qu'une liste ou un sélecteur affiche. */
export interface UtilisateurChoix {
  id: string;
  email: string;
  first_name?: string;
  last_name?: string;
  role?: string;
  organization_id?: string | null;
  is_active?: boolean;
}

export interface PageUtilisateurs {
  elements: UtilisateurChoix[];
  /** Le total CORRESPONDANT à la recherche, pas le total absolu. */
  total: number;
  /** `true` s'il reste des éléments au-delà de cette page. */
  incomplete: boolean;
}

/** Ce qu'un écran ou un sélecteur affiche sans qu'on ait rien tapé. */
export const TAILLE_PAGE_UTILISATEURS = 50;

export interface OptionsRecherche {
  /** Filtre de rôle. `all` et la chaîne vide valent « pas de filtre ». */
  role?: string;
  taille?: number;
}

/**
 * Une page d'utilisateurs, filtrée par `recherche` et par `role`.
 *
 * `recherche` vide ou absente rend la première page, du plus récent au plus
 * ancien. Le serveur cherche dans le courriel, le prénom ET le nom, sans
 * distinction de casse.
 *
 * Le rôle est envoyé au SERVEUR, jamais appliqué ici : filtrer une page de
 * cinquante dans le navigateur montrerait « les syndics parmi les cinquante
 * premiers » en les présentant comme « les syndics ».
 */
export async function chercherUtilisateurs<T = UtilisateurChoix>(
  recherche = "",
  options: OptionsRecherche = {},
): Promise<{ elements: T[]; total: number; incomplete: boolean }> {
  const taille = options.taille ?? TAILLE_PAGE_UTILISATEURS;
  const parametres = new URLSearchParams({
    per_page: String(taille),
    page: "1",
  });
  const terme = recherche.trim();
  if (terme) parametres.set("q", terme);
  const role = options.role?.trim();
  if (role && role !== "all") parametres.set("role", role);

  const reponse = await api.get<{
    data: T[];
    pagination?: { total_items?: number };
  }>(`/users?${parametres.toString()}`);

  const elements = reponse.data ?? [];
  // `total_items` vient du serveur. S'il manque, on ne PRÉTEND pas connaître
  // le total : on rend ce qu'on a et on ne dit pas « complet ».
  const total = reponse.pagination?.total_items ?? elements.length;
  return { elements, total, incomplete: total > elements.length };
}
