import { expect, type Page } from "@playwright/test";

/**
 * Le voile que `RouteGuard.svelte` pose pendant qu'il vérifie l'accès.
 *
 * `fixed inset-0 bg-white z-50` : il couvre TOUTE la page, et rien d'autre
 * dans l'application n'emploie cette combinaison.
 */
const VOILE = "div.fixed.inset-0.bg-white.z-50";

/**
 * Attend que `RouteGuard` ait fini de vérifier l'accès à la route courante.
 *
 * ## Pourquoi ça ne va pas de soi
 *
 * `expect(bouton).toBeVisible()` passe alors que le clic est impossible : la
 * visibilité d'un élément ne tient **pas** compte de ce qui le recouvre.
 * Playwright ne s'en aperçoit qu'au moment du clic, et rend alors
 *
 *     <div class="fixed inset-0 bg-white z-50 …"> from <astro-island …
 *     RouteGuard.svelte …> subtree intercepts pointer events
 *
 * après avoir réessayé pendant tout son délai. L'échec ressemble donc à un
 * bouton mort, alors que c'est un écran de chargement encore là.
 *
 * `waitUntil: "networkidle"` ne suffit pas : le voile se retire quand
 * `isChecking` retombe côté Svelte, ce qui n'est pas lié à l'inactivité
 * réseau.
 *
 * ## Ce que ça n'est pas
 *
 * Ce n'est pas un délai allongé ni une assertion relâchée : c'est la
 * précondition réelle du geste. Un utilisateur non plus ne clique pas à
 * travers un écran de chargement. Si le voile ne se retire jamais, ce
 * helper échoue — et c'est bien ce qu'on veut qu'il dise.
 *
 * ## Mesures (2026-09-17, recette)
 *
 * Retrait du voile : 3935 ms sur `/admin/organizations`, 1922 ms sur
 * `/admin/users`. Sans cette attente, `story1-admin-buttons` rendait
 * 4 échecs sur 27 exécutions ; avec, 1 sur 27 — et ce dernier était un
 * `502` d'amorçage, sans rapport.
 *
 * ## Le délai, et pourquoi il ne peut PAS valoir quinze secondes
 *
 * Première version : `timeout: 15_000`. Elle a échoué sur les cinq parcours
 * de la vitrine au premier run de `vitrine.yml`, le 2026-09-18 — le voile
 * était encore là après quinze secondes.
 *
 * `RouteGuard.svelte:45` porte son PROPRE plafond, et il vaut exactement
 * quinze secondes : `authStore.init()` court contre un `DELAI_INIT_MS` de
 * 15 s, et le voile ne se retire qu'après. Attendre quinze secondes ce qui
 * s'autorise quinze secondes est un tirage au sort — l'attente ne peut
 * gagner que si le composant est rapide, c'est-à-dire précisément quand
 * elle ne sert à rien.
 *
 * Le délai d'un helper d'attente doit donc EXCÉDER celui de ce qu'il
 * attend, marge comprise. D'où 25 s : les 15 s du garde, plus dix pour la
 * redescente et la variance d'un exécuteur froid.
 *
 * Ce n'est pas un délai « allongé pour faire passer un test » : c'est un
 * délai qui, pour la première fois, est cohérent avec le composant observé.
 * Si le voile ne se retire toujours pas au bout de 25 s, il y a un vrai
 * défaut — et le helper le dira.
 */
const PLAFOND_DU_GARDE_MS = 15_000; // RouteGuard.svelte:45 — DELAI_INIT_MS
const MARGE_MS = 10_000;

export async function attendreFinDuGardeDeRoute(page: Page): Promise<void> {
  await expect(page.locator(VOILE)).toHaveCount(0, {
    timeout: PLAFOND_DU_GARDE_MS + MARGE_MS,
  });
}
