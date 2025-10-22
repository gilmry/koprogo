<script lang="ts">
  import { authStore } from '../stores/auth';
  import { UserRole } from '../lib/types';
  import type { User } from '../lib/types';
  import { apiEndpoint } from '../lib/config';

  let email = '';
  let password = '';
  let error = '';
  let loading = false;

  const handleLogin = async (e: Event) => {
    e.preventDefault();
    error = '';
    loading = true;

    try {
      // Real API call
      const response = await fetch(apiEndpoint('/api/v1/auth/login'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      });

      if (response.ok) {
        const data = await response.json();
        const { token, user } = data;

        // Map backend user format to frontend format
        const mappedUser: User = {
          id: user.id,
          email: user.email,
          firstName: user.first_name,
          lastName: user.last_name,
          role: user.role,
          organizationId: user.organization_id,
          buildingIds: [], // Will be populated from backend later
        };

        // Login with token and initialize sync
        await authStore.login(mappedUser, token);

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

<form on:submit={handleLogin} class="space-y-6">
  {#if error}
    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
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
    />
  </div>

  <div class="flex items-center justify-between">
    <label class="flex items-center">
      <input
        type="checkbox"
        class="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500"
      />
      <span class="ml-2 text-sm text-gray-600">Se souvenir de moi</span>
    </label>
    <a href="/forgot-password" class="text-sm text-primary-600 hover:text-primary-700">
      Mot de passe oublié?
    </a>
  </div>

  <button
    type="submit"
    disabled={loading}
    class="w-full bg-primary-600 text-white py-3 rounded-lg hover:bg-primary-700 transition font-medium disabled:opacity-50 disabled:cursor-not-allowed"
  >
    {loading ? 'Connexion...' : 'Se connecter'}
  </button>

  <div class="mt-6 p-4 bg-gray-50 rounded-lg">
    <p class="text-xs font-semibold text-gray-700 mb-2">Comptes de test:</p>
    <div class="space-y-1 text-xs text-gray-600">
      <p>👑 <strong>SuperAdmin:</strong> admin@koprogo.com / admin123</p>
      <p class="text-gray-500 text-[10px] mt-1">Les données de démonstration peuvent être générées depuis le dashboard SuperAdmin</p>
    </div>
  </div>
</form>
