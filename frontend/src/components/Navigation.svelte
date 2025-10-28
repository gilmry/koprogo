<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { authStore } from '../stores/auth';
  import { UserRole } from '../lib/types';
  import SyncStatus from './SyncStatus.svelte';
  import LanguageSelector from './LanguageSelector.svelte';

  let showUserMenu = false;

  $: user = $authStore.user;
  $: isAuthenticated = $authStore.isAuthenticated;

  // Initialize auth store from localStorage on mount
  onMount(async () => {
    await authStore.init();
  });

  const logout = async () => {
    await authStore.logout();
    window.location.href = '/login';
  };

  const getNavItems = (role: UserRole | undefined, t: any) => {
    if (!role) return [];

    const commonItems = [
      { href: '/buildings', label: t('navigation.buildings'), icon: '🏢' },
    ];

    switch (role) {
      case UserRole.SUPERADMIN:
        return [
          { href: '/admin', label: t('navigation.admin'), icon: '⚙️' },
          { href: '/admin/monitoring', label: t('navigation.monitoring'), icon: '📈' },
          ...commonItems,
          { href: '/admin/organizations', label: 'Organisations', icon: '🏛️' },
          { href: '/admin/users', label: 'Utilisateurs', icon: '👥' },
        ];

      case UserRole.SYNDIC:
        return [
          { href: '/syndic', label: t('navigation.dashboard'), icon: '📊' },
          ...commonItems,
          { href: '/owners', label: t('navigation.owners'), icon: '👤' },
          { href: '/units', label: t('navigation.units'), icon: '🚪' },
          { href: '/expenses', label: t('navigation.expenses'), icon: '💰' },
          { href: '/meetings', label: t('navigation.meetings'), icon: '📅' },
          { href: '/documents', label: t('navigation.documents'), icon: '📄' },
        ];

      case UserRole.ACCOUNTANT:
        return [
          { href: '/accountant', label: t('navigation.dashboard'), icon: '📊' },
          ...commonItems,
          { href: '/expenses', label: t('navigation.expenses'), icon: '💰' },
          { href: '/reports', label: t('navigation.reports'), icon: '📈' },
        ];

      case UserRole.OWNER:
        return [
          { href: '/owner', label: t('navigation.dashboard'), icon: '🏠' },
          { href: '/owner/units', label: t('navigation.units'), icon: '🚪' },
          { href: '/owner/expenses', label: t('navigation.expenses'), icon: '💰' },
          { href: '/owner/documents', label: t('navigation.documents'), icon: '📄' },
        ];

      default:
        return commonItems;
    }
  };

  $: navItems = getNavItems(user?.role, $_);
</script>

<nav class="bg-white shadow-sm border-b border-gray-200">
  <div class="container mx-auto px-4">
    <div class="flex items-center justify-between h-16">
      <!-- Logo -->
      <a href={isAuthenticated ? `/${user?.role}` : '/'} class="flex items-center space-x-2">
        <span class="text-2xl font-bold text-primary-600">KoproGo</span>
        {#if user?.role}
          <span class="text-sm text-gray-500 hidden md:inline">
            | {user.role === UserRole.SUPERADMIN ? 'Admin' :
               user.role === UserRole.SYNDIC ? 'Syndic' :
               user.role === UserRole.ACCOUNTANT ? 'Comptable' : 'Copropriétaire'}
          </span>
        {/if}
      </a>

      {#if isAuthenticated}
        <!-- Navigation Links -->
        <div class="hidden md:flex items-center space-x-1">
          {#each navItems as item}
            <a
              href={item.href}
              class="px-3 py-2 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-100 hover:text-primary-600 transition"
            >
              <span class="mr-1">{item.icon}</span>
              {item.label}
            </a>
          {/each}
        </div>

        <!-- Right side: Language Selector + Sync Status + User Menu -->
        <div class="flex items-center gap-4">
          <LanguageSelector />
          <SyncStatus />

        <!-- User Menu -->
        <div class="relative">
          <button
            on:click|stopPropagation={() => showUserMenu = !showUserMenu}
            class="flex items-center space-x-2 px-3 py-2 rounded-lg hover:bg-gray-100 transition"
          >
            <div class="w-8 h-8 rounded-full bg-primary-600 text-white flex items-center justify-center font-semibold">
              {user?.firstName?.[0]}{user?.lastName?.[0]}
            </div>
            <span class="hidden md:inline text-sm font-medium text-gray-700">
              {user?.firstName} {user?.lastName}
            </span>
            <svg class="w-4 h-4 text-gray-500" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd"/>
            </svg>
          </button>

          {#if showUserMenu}
            <div class="absolute right-0 mt-2 w-48 bg-white rounded-lg shadow-lg border border-gray-200 py-1 z-50">
              <a
                href="/profile"
                on:click|stopPropagation
                class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
              >
                👤 {$_('navigation.profile')}
              </a>
              <a
                href="/settings"
                on:click|stopPropagation
                class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
              >
                ⚙️ Paramètres
              </a>
              <hr class="my-1" />
              <button
                on:click|stopPropagation={logout}
                class="w-full text-left block px-4 py-2 text-sm text-red-600 hover:bg-gray-100"
              >
                🚪 {$_('navigation.logout')}
              </button>
            </div>
          {/if}
        </div>
        </div>
      {:else}
        <div class="flex items-center gap-4">
          <LanguageSelector />
          <a
            href="/login"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition font-medium"
          >
            Connexion
          </a>
        </div>
      {/if}
    </div>

    {#if isAuthenticated}
      <!-- Mobile Navigation -->
      <div class="md:hidden pb-4 flex flex-wrap gap-2">
        {#each navItems as item}
          <a
            href={item.href}
            class="px-3 py-1.5 rounded-lg text-xs font-medium text-gray-700 bg-gray-50 hover:bg-gray-100 hover:text-primary-600 transition"
          >
            <span class="mr-1">{item.icon}</span>
            {item.label}
          </a>
        {/each}
      </div>
    {/if}
  </div>
</nav>

<svelte:window on:click={() => showUserMenu && (showUserMenu = false)} />
