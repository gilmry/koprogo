<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../stores/auth';

  onMount(async () => {
    // Validate session on component mount
    const isValid = await authStore.validateSession();

    // If session is invalid and we're not on login/register, redirect
    if (!isValid && typeof window !== 'undefined') {
      const currentPath = window.location.pathname;
      const publicPaths = ['/login', '/register', '/forgot-password'];

      if (!publicPaths.includes(currentPath)) {
        window.location.href = '/login';
      }
    }
  });
</script>

<!-- This component has no UI, it just manages session validation -->
