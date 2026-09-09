<script lang="ts">
  import { authStore } from "../stores/auth";
  import {
    canAccessRoute,
    getDefaultRedirect,
    isPublicRoute,
  } from "../lib/guards";
  import { _ } from "../lib/i18n";

  // Get current route from window.location
  let currentRoute = $state("");
  let isChecking = $state(true);

  $effect(() => {
    let cancelled = false;
    let unsubscribe: (() => void) | undefined;

    (async () => {
      // L'initialisation peut ÉCHOUER, et il faut alors le dire.
      //
      // `await authStore.init()` n'était pas protégé. Si l'appel rejette — un
      // réseau qui hoquette, un backend lent, un rafraîchissement silencieux
      // qui n'aboutit pas — la promesse de cette fonction rejette, `isChecking`
      // reste `true`, et l'écran affiche « Vérification des accès… » POUR
      // TOUJOURS. Ni message, ni redirection, ni délai : la page ne dit rien
      // et n'aboutit jamais.
      //
      // Constaté le 2026-09-09 sur la capture d'écran de
      // `quote-comparison.scenario.ts` : un spinner et ce libellé, vingt
      // secondes durant, sur un scénario vert aux quatre runs précédents.
      //
      // On échoue FERMÉ : une session qu'on n'a pas pu confirmer n'est pas une
      // session. `checkAccess` redirige alors vers `/login` pour toute route
      // protégée, ce qui est le comportement déjà écrit plus bas — et le même
      // choix que `canAccessRoute`, qui se ferme sur un rôle inconnu.
      // Un `try`/`catch` ne suffit pas : `init()` peut ne JAMAIS aboutir,
      // et une promesse qui ne se résout pas ne lève rien. C'est le cas
      // observé — le spinner tournait, sans erreur.
      //
      // Quinze secondes, puis on considère la session absente. C'est un
      // arbitrage : sur un réseau très lent, un rafraîchissement qui aurait
      // fini par réussir renvoie l'utilisateur vers `/login`, où il se
      // reconnecte. L'alternative est un écran qui ne répond plus jamais, ce
      // qui est strictement pire — et il n'a aucun moyen de le savoir.
      const DELAI_INIT_MS = 15_000;
      try {
        await Promise.race([
          authStore.init(),
          new Promise((_, rejeter) =>
            setTimeout(
              () => rejeter(new Error("init() n'a pas abouti en 15 s")),
              DELAI_INIT_MS,
            ),
          ),
        ]);
      } catch (erreur) {
        console.warn(
          "[RouteGuard] init() a échoué ou n'a pas abouti ; session traitée " +
            "comme absente",
          erreur,
        );
      }
      if (cancelled) return;

      currentRoute = window.location.pathname;
      let hasChecked = false;

      // Check if user is authenticated and has access to current route
      const checkAccess = () => {
        // Prevent multiple checks - only check once
        if (hasChecked) {
          return;
        }

        const { user, isAuthenticated, isLoading } = $authStore;

        // Wait until auth store is done loading
        if (isLoading) {
          return;
        }

        // Mark as checked to prevent loops
        hasChecked = true;

        // Public routes are always accessible
        if (isPublicRoute(currentRoute)) {
          isChecking = false;
          return;
        }

        // If not authenticated and trying to access protected route, redirect to login
        if (!isAuthenticated && !isPublicRoute(currentRoute)) {
          window.location.href =
            "/login?redirect=" + encodeURIComponent(currentRoute);
          return;
        }

        // If authenticated but no role, redirect to login (corrupted session)
        if (isAuthenticated && !user?.role) {
          console.warn(
            "[RouteGuard] User authenticated but no role found, logging out",
          );
          authStore.logout();
          window.location.href = "/login";
          return;
        }

        // Check if user has access to current route
        if (isAuthenticated && user?.role) {
          if (!canAccessRoute(currentRoute, user.role)) {
            console.warn(
              `[RouteGuard] Access denied to ${currentRoute} for role ${user.role}`,
            );
            const defaultRoute = getDefaultRedirect(user.role);
            window.location.href = defaultRoute;
            return;
          }
        }

        isChecking = false;
      };

      // Initial check
      checkAccess();

      // Re-check ONLY ONCE on auth store changes (then unsubscribe)
      unsubscribe = authStore.subscribe(() => {
        checkAccess();
      });
    })();

    return () => {
      cancelled = true;
      if (unsubscribe) unsubscribe();
    };
  });
</script>

{#if isChecking}
  <!-- Show loading state while checking access -->
  <div class="fixed inset-0 bg-white z-50 flex items-center justify-center">
    <div class="text-center">
      <div
        class="inline-block h-12 w-12 animate-spin rounded-full border-4 border-solid border-primary-600 border-r-transparent align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]"
        role="status"
      >
        <span class="sr-only">{$_("common.checkingAccess")}</span>
      </div>
      <p class="mt-4 text-gray-600 text-sm">{$_("common.checkingAccess")}</p>
    </div>
  </div>
{/if}
