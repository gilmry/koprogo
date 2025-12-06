<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../stores/auth';
  import { canAccessRoute, getDefaultRedirect, isPublicRoute } from '../lib/guards';
  import type { UserRole } from '../lib/types';

  // Get current route from window.location
  let currentRoute = '';
  let isChecking = true;

  onMount(() => {
    currentRoute = window.location.pathname;

    // Check if user is authenticated and has access to current route
    const checkAccess = () => {
      const { user, isAuthenticated } = $authStore;

      // Public routes are always accessible
      if (isPublicRoute(currentRoute)) {
        isChecking = false;
        return;
      }

      // If not authenticated and trying to access protected route, redirect to login
      if (!isAuthenticated && !isPublicRoute(currentRoute)) {
        window.location.href = '/login?redirect=' + encodeURIComponent(currentRoute);
        return;
      }

      // If authenticated but no role, redirect to login (corrupted session)
      if (isAuthenticated && !user?.role) {
        console.warn('[RouteGuard] User authenticated but no role found, logging out');
        authStore.logout();
        window.location.href = '/login';
        return;
      }

      // Check if user has access to current route
      if (isAuthenticated && user?.role) {
        if (!canAccessRoute(currentRoute, user.role)) {
          console.warn(`[RouteGuard] Access denied to ${currentRoute} for role ${user.role}`);
          const defaultRoute = getDefaultRedirect(user.role);
          window.location.href = defaultRoute;
          return;
        }
      }

      isChecking = false;
    };

    // Initial check
    checkAccess();

    // Re-check on auth store changes
    const unsubscribe = authStore.subscribe(() => {
      checkAccess();
    });

    return () => {
      unsubscribe();
    };
  });
</script>

{#if isChecking}
  <!-- Show loading state while checking access -->
  <div class="fixed inset-0 bg-white z-50 flex items-center justify-center">
    <div class="text-center">
      <div class="inline-block h-12 w-12 animate-spin rounded-full border-4 border-solid border-primary-600 border-r-transparent align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]" role="status">
        <span class="sr-only">Vérification des accès...</span>
      </div>
      <p class="mt-4 text-gray-600 text-sm">Vérification des accès...</p>
    </div>
  </div>
{/if}
