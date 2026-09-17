import { api } from "../api";

/**
 * Recherche d'organisations, côté SERVEUR.
 *
 * ## Pourquoi ce module existe
 *
 * `GET /organizations` rendait la table entière en ignorant `per_page` :
 * 3006 lignes sur la recette au 2026-09-17, d'où **8,2 s** d'écran blanc sur
 * `/admin/acps` contre 1,9 s sur `/admin/users` (#943). Quatre écrans
 * chargeaient tout pour remplir une liste déroulante.
 *
 * La route pagine désormais et accepte `?q=`. Elle répond en **65 ms**.
 *
 * Ce module est le seul endroit qui connaît cette forme : si la pagination
 * change encore, elle change ici.
 */

/** Une organisation, réduite à ce qu'un sélecteur affiche. */
export interface OrganisationChoix {
  id: string;
  name: string;
  slug?: string;
}

export interface PageOrganisations {
  elements: OrganisationChoix[];
  /** Le total CORRESPONDANT à la recherche, pas le total absolu. */
  total: number;
  /** `true` s'il reste des éléments au-delà de cette page. */
  incomplete: boolean;
}

/** Ce qu'un sélecteur affiche sans qu'on ait rien tapé. */
export const TAILLE_PAGE_SELECTEUR = 50;

/**
 * Une page d'organisations, filtrée par `recherche`.
 *
 * `recherche` vide ou absente rend la première page, par nom croissant.
 * Le serveur cherche dans le nom ET le slug, sans distinction de casse.
 */
export async function chercherOrganisations(
  recherche = "",
  taille = TAILLE_PAGE_SELECTEUR,
): Promise<PageOrganisations> {
  const parametres = new URLSearchParams({
    per_page: String(taille),
    page: "1",
  });
  const terme = recherche.trim();
  if (terme) parametres.set("q", terme);

  const reponse = await api.get<{
    data: OrganisationChoix[];
    pagination?: { total_items?: number };
  }>(`/organizations?${parametres.toString()}`);

  const elements = reponse.data ?? [];
  // `total_items` vient du serveur. S'il manque, on ne PRÉTEND pas
  // connaître le total : on rend ce qu'on a et on ne dit pas « complet ».
  const total = reponse.pagination?.total_items ?? elements.length;
  return { elements, total, incomplete: total > elements.length };
}
