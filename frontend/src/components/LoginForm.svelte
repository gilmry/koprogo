<script lang="ts">
  // Svelte 5 runes mode
  import { _, isLoading } from "../lib/i18n";
  import { authStore, mapUserFromBackend } from "../stores/auth";
  import { UserRole } from "../lib/types";
  import type { User } from "../lib/types";
  import { apiEndpoint } from "../lib/config";

  let email = $state("");
  let password = $state("");
  let error = $state("");
  let loading = $state(false);

  $effect(() => {
    // Ensure auth store is initialized before any login attempt
    authStore.init();
  });

  const handleLogin = async (e: Event) => {
    e.preventDefault();
    error = "";
    loading = true;

    try {
      // Real API call
      const response = await fetch(apiEndpoint("/auth/login"), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        // WP-FE1 : credentials inclus pour que le Set-Cookie refresh
        // HttpOnly soit stocké par le navigateur.
        credentials: "include",
        body: JSON.stringify({ email, password }),
      });

      if (response.ok) {
        const data = await response.json();
        // WP-FE1 : plus de refresh_token dans le corps — il est dans le
        // cookie HttpOnly. Seul l'access token (mémoire) transite ici.
        const { token, user } = data;

        // Map backend user format to frontend format
        const mappedUser: User = mapUserFromBackend(user);

        // Access token en mémoire ; refresh = cookie posé par le backend
        await authStore.login(mappedUser, token);

        // Redirect based on role
        const redirectMap: Record<string, string> = {
          [UserRole.SUPERADMIN]: "/admin",
          [UserRole.SYNDIC]: "/syndic",
          [UserRole.ACCOUNTANT]: "/accountant",
          [UserRole.OWNER]: "/owner",
        };

        // Check for ?redirect= query param (set by RouteGuard)
        const urlParams = new URLSearchParams(window.location.search);
        const redirectTo = urlParams.get("redirect");
        const targetUrl = redirectTo || redirectMap[mappedUser.role] || "/";

        // Petit délai pour laisser localStorage se propager avant navigation
        setTimeout(() => {
          window.location.href = targetUrl;
        }, 100);
      } else if (response.status === 429) {
        error =
          $_("auth.tooManyAttempts") ||
          "Trop de tentatives de connexion. Réessayez dans 15 minutes.";
      } else if (response.status === 401) {
        // Le backend renvoie `{"error": "Invalid credentials"}`, en anglais.
        // Reprendre cette chaîne telle quelle affichait « Invalid credentials »
        // à un utilisateur francophone, alors que la traduction existait juste
        // à côté et n'était jamais atteinte : le `||` ne se déclenche que si le
        // serveur ne dit rien. Sur un identifiant erroné, la seule information
        // utile est « c'est faux » ; on l'affiche donc dans la langue de
        // l'interface, sans jamais relayer le texte du serveur.
        error = $_("auth.loginError");
      } else {
        const errorData = await response.json().catch(() => ({}));
        error = errorData.error || $_("auth.loginError");
      }
    } catch (e) {
      console.error("Login error:", e);
      error = $_("auth.connectionError");
    } finally {
      loading = false;
    }
  };
</script>

{#if $isLoading}
  <div class="flex justify-center py-8">
    <div
      class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"
    ></div>
  </div>
{:else}
  <form onsubmit={handleLogin} class="space-y-6" data-testid="login-form">
    {#if error}
      <div
        class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg"
        role="alert"
        data-testid="login-error"
      >
        {error}
      </div>
    {/if}

    <div>
      <label for="email" class="block text-sm font-medium text-gray-700 mb-2">
        {$_("auth.email")}
      </label>
      <input
        id="email"
        type="email"
        bind:value={email}
        required
        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        placeholder={$_("auth.emailPlaceholder")}
        autocomplete="email"
        data-testid="login-email"
      />
    </div>

    <div>
      <label
        for="password"
        class="block text-sm font-medium text-gray-700 mb-2"
      >
        {$_("auth.password")}
      </label>
      <input
        id="password"
        type="password"
        bind:value={password}
        required
        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        placeholder="••••••••"
        autocomplete="current-password"
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
        <span class="ml-2 text-sm text-gray-600">{$_("auth.rememberMe")}</span>
      </label>
      <a
        href="/forgot-password"
        class="text-sm text-primary-600 hover:text-primary-700"
        data-testid="login-forgot-password"
      >
        {$_("auth.forgotPassword")}
      </a>
    </div>

    <button
      type="submit"
      disabled={loading}
      class="w-full bg-primary-600 text-white py-3 rounded-lg hover:bg-primary-700 transition font-medium disabled:opacity-50 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500"
      data-testid="login-submit"
    >
      {loading ? $_("auth.loggingIn") : $_("auth.login")}
    </button>
  </form>
{/if}
