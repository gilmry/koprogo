<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { _ } from 'svelte-i18n';
  import { authStore } from '../stores/auth';
  import { UserRole } from '../lib/types';

  let showUserMenu = false;
  let switchingRole = false;
  let selectedRoleId: string | null = null;

  $: user = $authStore.user;
  $: isAuthenticated = $authStore.isAuthenticated;

  const getRoleLabel = (role: UserRole | undefined) => {
    switch (role) {
      case UserRole.SUPERADMIN:
        return 'Admin plateforme';
      case UserRole.SYNDIC:
        return 'Syndic';
      case UserRole.ACCOUNTANT:
        return 'Comptable';
      case UserRole.OWNER:
        return 'Copropriétaire';
      default:
        return 'Rôle';
    }
  };

  const formatRoleOption = (roleId: string | undefined, role: UserRole | undefined, organizationId?: string) => {
    const organizationLabel = organizationId ? `• ${organizationId.slice(0, 8)}` : '• Plateforme';
    return `${getRoleLabel(role)} ${organizationLabel}`;
  };

  const handleRoleChange = async (event: Event) => {
    const target = event.target as HTMLSelectElement;
    const roleId = target.value;
    if (!roleId || roleId === user?.activeRole?.id) {
      return;
    }

    switchingRole = true;
    const success = await authStore.switchRole(roleId);
    switchingRole = false;

    if (!success) {
      // revert selection on failure
      target.value = user?.activeRole?.id ?? '';
    } else {
      selectedRoleId = roleId;
      const nextUser = get(authStore).user;
      if (nextUser?.role) {
        const redirectMap = {
          [UserRole.SUPERADMIN]: '/admin',
          [UserRole.SYNDIC]: '/syndic',
          [UserRole.ACCOUNTANT]: '/accountant',
          [UserRole.OWNER]: '/owner',
        } as const;
        const destination = redirectMap[nextUser.role] ?? '/';
        if (!window.location.pathname.startsWith(destination)) {
          window.location.href = destination;
        }
      }
    }
  };

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
          { href: '/admin/board-members', label: 'Conseil', icon: '👑' },
          { href: '/admin/gdpr', label: 'RGPD', icon: '🔒' },
        ];

      case UserRole.SYNDIC:
        return [
          { href: '/syndic', label: t('navigation.dashboard'), icon: '📊' },
          ...commonItems,
          { href: '/owners', label: t('navigation.owners'), icon: '👤' },
          { href: '/units', label: t('navigation.units'), icon: '🚪' },
          { href: '/expenses', label: t('navigation.expenses'), icon: '💰' },
          { href: '/invoice-workflow', label: 'Workflow factures', icon: '✅' },
          { href: '/call-for-funds', label: 'Appels de fonds', icon: '📢' },
          { href: '/owner-contributions', label: 'Contributions', icon: '💶' },
          { href: '/payment-reminders', label: 'Relances', icon: '📧' },
          { href: '/meetings', label: t('navigation.meetings'), icon: '📅' },
          { href: '/syndic/board-members', label: 'Conseil', icon: '👑' },
          { href: '/documents', label: t('navigation.documents'), icon: '📄' },
        ];

      case UserRole.ACCOUNTANT:
        return [
          { href: '/accountant', label: t('navigation.dashboard'), icon: '📊' },
          ...commonItems,
          { href: '/expenses', label: t('navigation.expenses'), icon: '💰' },
          { href: '/invoice-workflow', label: 'Workflow factures', icon: '✅' },
          { href: '/call-for-funds', label: 'Appels de fonds', icon: '📢' },
          { href: '/owner-contributions', label: 'Contributions', icon: '💶' },
          { href: '/payment-reminders', label: 'Relances', icon: '📧' },
          { href: '/journal-entries', label: 'Écritures comptables', icon: '📒' },
          { href: '/reports', label: 'Rapports PCMN', icon: '📈' },
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
  $: if (user?.activeRole?.id && user.activeRole.id !== selectedRoleId) {
    selectedRoleId = user.activeRole.id;
  }
</script>

<nav class="bg-white shadow-sm border-b border-gray-200" data-testid="navigation">
  <div class="container mx-auto">
    <div class="flex items-center justify-between h-16">
      <!-- Logo -->
      <a
        href={isAuthenticated ? `/${user?.role}` : '/'}
        class="flex items-center space-x-1"
        data-testid="nav-logo"
      >
        <span class="text-xl font-bold text-primary-600">KoproGo</span>
        {#if user?.role}
          <span class="text-xs text-gray-500 hidden lg:inline">
            | {user.role === UserRole.SUPERADMIN ? 'Admin' :
               user.role === UserRole.SYNDIC ? 'Syndic' :
               user.role === UserRole.ACCOUNTANT ? 'Comptable' : 'Copropriétaire'}
          </span>
        {/if}
      </a>

      {#if isAuthenticated}
        <!-- Navigation Links -->
        <div class="hidden md:flex items-center space-x-0.5" data-testid="nav-links">
          {#each navItems as item}
            <a
              href={item.href}
              class="px-2 py-1.5 rounded text-xs font-medium text-gray-700 hover:bg-gray-100 hover:text-primary-600 transition"
              data-testid="nav-link-{item.label.toLowerCase().replace(/\s+/g, '-')}"
            >
              <span class="mr-0.5">{item.icon}</span>
              {item.label}
            </a>
          {/each}
        </div>

        <!-- Right side: User Menu -->
        <div class="flex items-center gap-2">
        <!-- User Menu -->
        <div class="relative" data-testid="user-menu-container">
          <button
            on:click|stopPropagation={() => showUserMenu = !showUserMenu}
            class="flex items-center space-x-2 px-3 py-2 rounded-lg hover:bg-gray-100 transition"
            data-testid="user-menu-button"
          >
            <div class="w-8 h-8 rounded-full bg-primary-600 text-white flex items-center justify-center font-semibold">
              {user?.first_name?.[0]}{user?.last_name?.[0]}
            </div>
            <span class="hidden md:inline text-sm font-medium text-gray-700">
              {user?.first_name} {user?.last_name}
            </span>
            <svg class="w-4 h-4 text-gray-500" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd"/>
            </svg>
          </button>

          {#if showUserMenu}
            <div
              role="menu"
              tabindex="-1"
              class="absolute right-0 mt-2 w-64 bg-white rounded-lg shadow-lg border border-gray-200 py-1 z-50"
              data-testid="user-menu-dropdown"
              on:click|stopPropagation
              on:keydown={(e) => e.key === 'Escape' && (showUserMenu = false)}
            >
              {#if user?.roles && user.roles.length > 1}
                <div class="px-4 py-2 border-b border-gray-200" data-testid="role-switcher">
                  <label for="role-selector" class="text-xs text-gray-500 block mb-1">Changer de rôle</label>
                  <select
                    id="role-selector"
                    class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                    on:change={handleRoleChange}
                    disabled={switchingRole}
                    bind:value={selectedRoleId}
                    data-testid="role-selector"
                  >
                    {#each user.roles as roleOption}
                      <option value={roleOption.id}>
                        {formatRoleOption(roleOption.id, roleOption.role, roleOption.organizationId)}
                      </option>
                    {/each}
                  </select>
                </div>
              {/if}
              <a
                href="/profile"
                on:click|stopPropagation
                class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                data-testid="user-menu-profile"
              >
                👤 {$_('navigation.profile')}
              </a>
              <a
                href="/settings"
                on:click|stopPropagation
                class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                data-testid="user-menu-settings"
              >
                ⚙️ Paramètres
              </a>
              <hr class="my-1" />
              <button
                on:click|stopPropagation={logout}
                class="w-full text-left block px-4 py-2 text-sm text-red-600 hover:bg-gray-100"
                data-testid="user-menu-logout"
              >
                🚪 {$_('navigation.logout')}
              </button>
            </div>
          {/if}
        </div>
        </div>
      {:else}
        <div class="flex items-center gap-4">
          <a
            href="/login"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition font-medium"
            data-testid="nav-login-button"
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
