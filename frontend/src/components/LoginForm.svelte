<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore, mapUserFromBackend } from '../stores/auth';
  import { UserRole } from '../lib/types';
  import type { User } from '../lib/types';
  import { apiEndpoint } from '../lib/config';

  let email = '';
  let password = '';
  let error = '';
  let loading = false;

  onMount(async () => {
    // Ensure auth store is initialized before any login attempt
    await authStore.init();
  });

  const handleLogin = async (e: Event) => {
    e.preventDefault();
    error = '';
    loading = true;

    try {
      // Real API call
      const response = await fetch(apiEndpoint('/auth/login'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      });

      if (response.ok) {
        const data = await response.json();
        const { token, refresh_token, user } = data;

        // Map backend user format to frontend format
        const mappedUser: User = mapUserFromBackend(user);

        // Login with token, refresh token and initialize sync
        await authStore.login(mappedUser, token, refresh_token);

        // Redirect based on role
        const redirectMap = {
          [UserRole.SUPERADMIN]: '/admin',
          [UserRole.SYNDIC]: '/syndic',
          [UserRole.ACCOUNTANT]: '/accountant',
          [UserRole.OWNER]: '/owner',
        };
        window.location.href = redirectMap[mappedUser.role] || '/';
      } else {
        const errorData = await response.json();
        error = errorData.error || 'Email ou mot de passe incorrect';
      }
    } catch (e) {
      console.error('Login error:', e);
      error = 'Une erreur est survenue lors de la connexion. Vérifiez que le serveur est démarré.';
    } finally {
      loading = false;
    }
  };
</script>

<form on:submit={handleLogin} class="space-y-6" data-testid="login-form">
  {#if error}
    <div
      class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg"
      data-testid="login-error"
    >
      {error}
    </div>
  {/if}

  <div>
    <label for="email" class="block text-sm font-medium text-gray-700 mb-2">
      Email
    </label>
    <input
      id="email"
      type="email"
      bind:value={email}
      required
      class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
      placeholder="votre@email.com"
      data-testid="login-email"
    />
  </div>

  <div>
    <label for="password" class="block text-sm font-medium text-gray-700 mb-2">
      Mot de passe
    </label>
    <input
      id="password"
      type="password"
      bind:value={password}
      required
      class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
      placeholder="••••••••"
      data-testid="login-password"
    />
  </div>

  <div class="flex items-center justify-between">
    <label class="flex items-center">
      <input
        type="checkbox"
        class="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500"
        data-testid="login-remember"
      />
      <span class="ml-2 text-sm text-gray-600">Se souvenir de moi</span>
    </label>
    <a
      href="/forgot-password"
      class="text-sm text-primary-600 hover:text-primary-700"
      data-testid="login-forgot-password"
    >
      Mot de passe oublié?
    </a>
  </div>

  <button
    type="submit"
    disabled={loading}
    class="w-full bg-primary-600 text-white py-3 rounded-lg hover:bg-primary-700 transition font-medium disabled:opacity-50 disabled:cursor-not-allowed"
    data-testid="login-submit"
  >
    {loading ? 'Connexion...' : 'Se connecter'}
  </button>
</form>
