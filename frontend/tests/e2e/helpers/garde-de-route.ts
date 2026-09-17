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
 */
export async function attendreFinDuGardeDeRoute(page: Page): Promise<void> {
  await expect(page.locator(VOILE)).toHaveCount(0, { timeout: 15_000 });
}
