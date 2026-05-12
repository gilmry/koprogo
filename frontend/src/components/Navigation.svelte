<script lang="ts">
  // Svelte 5 runes mode
  import { get } from 'svelte/store';
  import { _ } from '../lib/i18n';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { authStore } from '../stores/auth';
  import { UserRole } from '../lib/types';
  import NotificationBell from './notifications/NotificationBell.svelte';

  // --- State ---
  let switchingRole = $state(false);
  let selectedRoleId = $state<string | null>(null);
  let drawerOpen = $state(false);
  let currentPath = $state('');
  let hamburgerButton = $state<HTMLButtonElement | undefined>(undefined);
  let drawerCloseButton = $state<HTMLButtonElement | undefined>(undefined);

  let user = $derived($authStore.user);
  let isAuthenticated = $derived($authStore.isAuthenticated);

  // --- Types ---
  interface NavItem {
    href: string;
    label: string;
    icon: string;
  }

  interface NavGroup {
    id: string;
    label: string;
    items: NavItem[];
  }

  // --- Role helpers (unchanged) ---
  const getRoleLabel = (role: UserRole | undefined) => {
    switch (role) {
      case UserRole.SUPERADMIN: return 'Admin plateforme';
      case UserRole.SYNDIC: return 'Syndic';
      case UserRole.ACCOUNTANT: return 'Comptable';
      case UserRole.OWNER: return 'Copropriétaire';
      default: return 'Rôle';
    }
  };

  const formatRoleOption = (roleId: string | undefined, role: UserRole | undefined, organizationId?: string) => {
    const organizationLabel = organizationId ? `• ${organizationId.slice(0, 8)}` : '• Plateforme';
    return `${getRoleLabel(role)} ${organizationLabel}`;
  };

  const handleRoleChange = async (event: Event) => {
    const target = event.target as HTMLSelectElement;
    const roleId = target.value;
    if (!roleId || roleId === user?.activeRole?.id) return;

    switchingRole = true;
    const success = await authStore.switchRole(roleId);
    switchingRole = false;

    if (!success) {
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

  // --- Drawer controls ---
  const openDrawer = () => {
    drawerOpen = true;
    document.body.style.overflow = 'hidden';
    requestAnimationFrame(() => drawerCloseButton?.focus());
  };

  const closeDrawer = () => {
    drawerOpen = false;
    document.body.style.overflow = '';
    requestAnimationFrame(() => hamburgerButton?.focus());
  };

  const handleNavClick = () => {
    closeDrawer();
  };

  // --- Active link detection ---
  const isActive = (href: string): boolean => {
    if (href === '/') return currentPath === '/';
    return currentPath === href || currentPath.startsWith(href + '/');
  };

  // --- Lifecycle ---
  $effect(() => {
    authStore.init();
    currentPath = window.location.pathname;
  });

  const logout = async () => {
    await authStore.logout();
    window.location.href = '/login';
  };

  // --- Grouped navigation items ---
  const getNavGroups = (role: UserRole | undefined, t: any): NavGroup[] => {
    // Safety check: only return navigation if role is valid and authenticated user has that role
    if (!role) return [];

    // Ensure role matches expected values (lowercase enum)
    const validRoles = [UserRole.SUPERADMIN, UserRole.SYNDIC, UserRole.ACCOUNTANT, UserRole.OWNER];
    if (!validRoles.includes(role)) return [];

    const communityGroup: NavGroup = {
      id: 'communaute',
      label: t('navigation.community'),
      items: [
        { href: '/exchanges', label: t('navigation.sel'), icon: '🔄' },
        { href: '/polls', label: t('navigation.polls'), icon: '📊' },
        { href: '/notices', label: t('navigation.notices'), icon: '📌' },
        { href: '/bookings', label: t('navigation.bookings'), icon: '📅' },
        { href: '/sharing', label: t('navigation.sharing_short'), icon: '🎁' },
        { href: '/skills', label: t('navigation.skills'), icon: '🎓' },
        { href: '/energy-campaigns', label: t('navigation.energy'), icon: '⚡' },
        { href: '/gamification', label: t('navigation.gamification'), icon: '🏆' },
      ],
    };

    // Explicitly handle SUPERADMIN role only - no other role should see admin items
    if (role === UserRole.SUPERADMIN) {
      return [
        {
          id: 'principal',
          label: t('navigation.main'),
          items: [
            { href: '/admin', label: t('navigation.admin'), icon: '⚙️' },
            { href: '/admin/monitoring', label: t('navigation.monitoring'), icon: '📈' },
            { href: '/buildings', label: t('navigation.buildings'), icon: '🏢' },
          ],
        },
        {
          id: 'gestion',
          label: t('navigation.management'),
          items: [
            { href: '/admin/organizations', label: t('navigation.organizations'), icon: '🏛️' },
            { href: '/admin/users', label: t('navigation.users'), icon: '👥' },
            { href: '/admin/board-members', label: t('navigation.council'), icon: '👑' },
            { href: '/admin/gdpr', label: t('navigation.gdpr'), icon: '🔒' },
            { href: '/admin/gamification', label: t('navigation.gamification'), icon: '🏆' },
          ],
        },
        communityGroup,
      ];
    }

    // SYNDIC role - must not include any /admin/* items
    if (role === UserRole.SYNDIC) {
      return [
        {
          id: 'principal',
          label: t('navigation.main'),
          items: [
            { href: '/syndic', label: t('navigation.dashboard'), icon: '📊' },
            { href: '/buildings', label: t('navigation.buildings'), icon: '🏢' },
          ],
        },
        {
          id: 'gestion',
          label: t('navigation.management'),
          items: [
            { href: '/owners', label: t('navigation.owners'), icon: '👤' },
            { href: '/units', label: t('navigation.units'), icon: '🚪' },
            { href: '/expenses', label: t('navigation.expenses'), icon: '💰' },
            { href: '/invoice-workflow', label: t('navigation.invoiceWorkflow'), icon: '✅' },
            { href: '/call-for-funds', label: t('navigation.callForFunds'), icon: '📢' },
            { href: '/owner-contributions', label: t('navigation.contributions'), icon: '💶' },
            { href: '/payment-reminders', label: t('navigation.reminders'), icon: '📧' },
            { href: '/budgets', label: t('navigation.budgets'), icon: '📊' },
            { href: '/etats-dates', label: t('navigation.etatsDates'), icon: '📋' },
            { href: '/gamification', label: t('navigation.gamification'), icon: '🏆' },
          ],
        },
        {
          id: 'gouvernance',
          label: t('navigation.governance'),
          items: [
            { href: '/meetings', label: t('navigation.meetings'), icon: '📅' },
            { href: '/convocations', label: t('navigation.convocations'), icon: '📨' },
            { href: '/tickets', label: t('navigation.tickets'), icon: '🎫' },
            { href: '/quotes', label: t('navigation.quotes'), icon: '📋' },
            { href: '/work-reports', label: t('navigation.works'), icon: '🔧' },
            { href: '/inspections', label: t('navigation.inspections'), icon: '🔍' },
            { href: '/syndic/board-members', label: t('navigation.council'), icon: '👑' },
            { href: '/documents', label: t('navigation.documents'), icon: '📄' },
          ],
        },
        communityGroup,
      ];
    }

    // ACCOUNTANT role
    if (role === UserRole.ACCOUNTANT) {
      return [
        {
          id: 'principal',
          label: t('navigation.main'),
          items: [
            { href: '/accountant', label: t('navigation.dashboard'), icon: '📊' },
            { href: '/buildings', label: t('navigation.buildings'), icon: '🏢' },
          ],
        },
        {
          id: 'comptabilite',
          label: t('navigation.accounting'),
          items: [
            { href: '/expenses', label: t('navigation.expenses'), icon: '💰' },
            { href: '/invoice-workflow', label: t('navigation.invoiceWorkflow'), icon: '✅' },
            { href: '/call-for-funds', label: t('navigation.callForFunds'), icon: '📢' },
            { href: '/owner-contributions', label: t('navigation.contributions'), icon: '💶' },
            { href: '/payment-reminders', label: t('navigation.reminders'), icon: '📧' },
            { href: '/budgets', label: t('navigation.budgets'), icon: '📊' },
            { href: '/etats-dates', label: t('navigation.etatsDates'), icon: '📋' },
            { href: '/journal-entries', label: t('navigation.journalEntries'), icon: '📒' },
            { href: '/reports', label: t('navigation.reportsPcmn'), icon: '📈' },
          ],
        },
        communityGroup,
      ];
    }

    // OWNER role
    if (role === UserRole.OWNER) {
      return [
        {
          id: 'principal',
          label: t('navigation.main'),
          items: [
            { href: '/owner', label: t('navigation.dashboard'), icon: '🏠' },
            { href: '/owner/units', label: t('navigation.units'), icon: '🚪' },
          ],
        },
        {
          id: 'espace',
          label: t('navigation.mySpace'),
          items: [
            { href: '/owner/expenses', label: t('navigation.expenses'), icon: '💰' },
            { href: '/owner/payments', label: t('navigation.payments'), icon: '💳' },
            { href: '/owner/payment-methods', label: t('navigation.paymentMethods'), icon: '🏦' },
            { href: '/owner/tickets', label: t('navigation.myTickets'), icon: '🎫' },
            { href: '/owner/documents', label: t('navigation.documents'), icon: '📄' },
            { href: '/owner/profile', label: t('navigation.profile'), icon: '👤' },
          ],
        },
        communityGroup,
      ];
    }

    // Fallback for any unmapped roles - return minimal navigation
    return [{
      id: 'principal',
      label: t('navigation.main'),
      items: [{ href: '/buildings', label: t('navigation.buildings'), icon: '🏢' }],
    }];
  };

  let navGroups = $derived(getNavGroups(user?.role, $_));
  $effect(() => {
    if (user?.activeRole?.id && user.activeRole.id !== selectedRoleId) {
      selectedRoleId = user.activeRole.id;
    }
  });
</script>

<!-- ============================================================ -->
<!-- DESKTOP SIDEBAR (lg+) - Fixed left, full height              -->
<!-- ============================================================ -->
{#if isAuthenticated}
  <aside
    class="hidden lg:flex lg:flex-col lg:fixed lg:inset-y-0 lg:left-0 lg:w-60 bg-white border-r border-gray-200 z-30"
    role="navigation"
    aria-label="Navigation principale"
    data-testid="sidebar-desktop"
  >
    <!-- Logo + Notification -->
    <div class="flex items-center justify-between h-14 px-4 border-b border-gray-200 shrink-0">
      <a href={`/${user?.role}`} class="text-xl font-bold text-primary-600" data-testid="nav-logo">
        KoproGo
      </a>
      <NotificationBell />
    </div>

    <!-- Scrollable nav groups -->
    <nav class="flex-1 overflow-y-auto py-3 px-3">
      {#each navGroups as group (group.id)}
        <div class="mb-3">
          <h3 class="px-3 mb-1 text-[11px] font-semibold text-gray-400 uppercase tracking-wider">
            {group.label}
          </h3>
          <ul class="space-y-0.5">
            {#each group.items as item (item.href)}
              <li>
                <a
                  href={item.href}
                  class="flex items-center gap-2.5 px-3 py-1.5 rounded-lg text-sm transition-colors
                    {isActive(item.href)
                      ? 'bg-primary-50 text-primary-700 font-semibold'
                      : 'text-gray-700 hover:bg-gray-50 hover:text-primary-600'}"
                  aria-current={isActive(item.href) ? 'page' : undefined}
                  data-testid="nav-link-{item.label.toLowerCase().replace(/\s+/g, '-')}"
                >
                  <span class="text-base shrink-0 w-5 text-center">{item.icon}</span>
                  <span class="truncate">{item.label}</span>
                </a>
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    </nav>

    <!-- User section bottom -->
    <div class="shrink-0 border-t border-gray-200 p-3" data-testid="sidebar-user-section">
      {#if user?.roles && user.roles.length > 1}
        <div class="mb-2">
          <label for="sidebar-role-selector" class="text-[11px] text-gray-400 block mb-1">Rôle actif</label>
          <select
            id="sidebar-role-selector"
            class="w-full px-2 py-1 border border-gray-300 rounded-lg text-xs focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            onchange={handleRoleChange}
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

      <div class="flex items-center gap-2.5 mb-2">
        <div class="w-8 h-8 rounded-full bg-primary-600 text-white flex items-center justify-center font-semibold text-xs shrink-0">
          {user?.first_name?.[0]}{user?.last_name?.[0]}
        </div>
        <div class="min-w-0">
          <p class="text-sm font-medium text-gray-900 truncate">{user?.first_name} {user?.last_name}</p>
          <p class="text-[11px] text-gray-500 truncate">{getRoleLabel(user?.role)}</p>
        </div>
      </div>

      <div class="space-y-0.5">
        <a href="/profile" class="flex items-center gap-2 px-2 py-1 text-xs text-gray-600 hover:bg-gray-50 rounded-lg transition-colors">
          👤 {$_('navigation.profile')}
        </a>
        <a href="/settings" class="flex items-center gap-2 px-2 py-1 text-xs text-gray-600 hover:bg-gray-50 rounded-lg transition-colors">
          ⚙️ Paramètres
        </a>
        <a href="/settings/gdpr" class="flex items-center gap-2 px-2 py-1 text-xs text-gray-600 hover:bg-gray-50 rounded-lg transition-colors">
          🔒 Données RGPD
        </a>
        <button
          onclick={logout}
          class="w-full flex items-center gap-2 px-2 py-1 text-xs text-red-600 hover:bg-red-50 rounded-lg transition-colors"
          data-testid="user-menu-logout"
        >
          🚪 {$_('navigation.logout')}
        </button>
      </div>
    </div>
  </aside>
{/if}

<!-- ============================================================ -->
<!-- MOBILE TOP HEADER (<lg) - Fixed, slim                        -->
<!-- ============================================================ -->
<header
  class="lg:hidden fixed top-0 left-0 right-0 h-14 bg-white border-b border-gray-200 z-40 flex items-center justify-between px-3"
  data-testid="mobile-header"
>
  {#if isAuthenticated}
    <!-- Left: Hamburger -->
    <button
      bind:this={hamburgerButton}
      onclick={openDrawer}
      class="p-2 -ml-1 rounded-lg text-gray-600 hover:bg-gray-100"
      aria-label="Ouvrir le menu"
      aria-expanded={drawerOpen}
      data-testid="hamburger-button"
    >
      <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
      </svg>
    </button>

    <!-- Center: Logo -->
    <a href={`/${user?.role}`} class="text-lg font-bold text-primary-600">KoproGo</a>

    <!-- Right: Notifications + Avatar -->
    <div class="flex items-center gap-1">
      <NotificationBell />
      <a
        href="/profile"
        class="w-8 h-8 rounded-full bg-primary-600 text-white flex items-center justify-center font-semibold text-xs"
        aria-label="Profil"
      >
        {user?.first_name?.[0]}{user?.last_name?.[0]}
      </a>
    </div>
  {:else}
    <!-- Unauthenticated: Logo + Login -->
    <a href="/" class="text-lg font-bold text-primary-600">KoproGo</a>
    <a href="/login" class="px-4 py-1.5 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium" data-testid="nav-login-button">
      Connexion
    </a>
  {/if}
</header>

<!-- ============================================================ -->
<!-- DESKTOP: Unauthenticated sidebar (logo + login)              -->
<!-- ============================================================ -->
{#if !isAuthenticated}
  <aside class="hidden lg:flex lg:flex-col lg:fixed lg:inset-y-0 lg:left-0 lg:w-60 bg-white border-r border-gray-200 z-30 items-center justify-center gap-4">
    <a href="/" class="text-2xl font-bold text-primary-600">KoproGo</a>
    <p class="text-sm text-gray-500 text-center px-6">Plateforme de gestion de copropriété</p>
    <a href="/login" class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium" data-testid="nav-login-button">
      Connexion
    </a>
  </aside>
{/if}

<!-- ============================================================ -->
<!-- MOBILE DRAWER (slide from left)                              -->
<!-- ============================================================ -->
{#if drawerOpen && isAuthenticated}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/40 z-40 lg:hidden"
    transition:fade={{ duration: 200 }}
    onclick={closeDrawer}
    onkeydown={(e) => e.key === 'Escape' && closeDrawer()}
    role="button"
    tabindex="-1"
    aria-label="Fermer le menu"
  ></div>

  <!-- Drawer panel -->
  <aside
    class="fixed inset-y-0 left-0 w-72 bg-white shadow-xl z-50 flex flex-col lg:hidden"
    transition:fly={{ x: -288, duration: 300, easing: cubicOut }}
    role="navigation"
    aria-label="Menu mobile"
    data-testid="mobile-drawer"
  >
    <!-- Header: logo + close -->
    <div class="flex items-center justify-between h-14 px-4 border-b border-gray-200 shrink-0">
      <a href={`/${user?.role}`} class="text-xl font-bold text-primary-600" onclick={handleNavClick}>
        KoproGo
      </a>
      <button
        bind:this={drawerCloseButton}
        onclick={closeDrawer}
        class="p-2 rounded-lg text-gray-500 hover:bg-gray-100"
        aria-label="Fermer le menu"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
        </svg>
      </button>
    </div>

    <!-- Nav groups -->
    <nav class="flex-1 overflow-y-auto py-3 px-3">
      {#each navGroups as group (group.id)}
        <div class="mb-3">
          <h3 class="px-3 mb-1 text-[11px] font-semibold text-gray-400 uppercase tracking-wider">
            {group.label}
          </h3>
          <ul class="space-y-0.5">
            {#each group.items as item (item.href)}
              <li>
                <a
                  href={item.href}
                  onclick={handleNavClick}
                  class="flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm transition-colors
                    {isActive(item.href)
                      ? 'bg-primary-50 text-primary-700 font-semibold'
                      : 'text-gray-700 hover:bg-gray-50 hover:text-primary-600'}"
                  aria-current={isActive(item.href) ? 'page' : undefined}
                  data-testid="nav-link-{item.label.toLowerCase().replace(/\s+/g, '-')}"
                >
                  <span class="text-base shrink-0 w-5 text-center">{item.icon}</span>
                  <span>{item.label}</span>
                </a>
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    </nav>

    <!-- User section -->
    <div class="shrink-0 border-t border-gray-200 p-4">
      {#if user?.roles && user.roles.length > 1}
        <div class="mb-3">
          <label for="drawer-role-selector" class="text-[11px] text-gray-400 block mb-1">Rôle actif</label>
          <select
            id="drawer-role-selector"
            class="w-full px-2 py-1.5 border border-gray-300 rounded-lg text-xs focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            onchange={handleRoleChange}
            disabled={switchingRole}
            bind:value={selectedRoleId}
          >
            {#each user.roles as roleOption}
              <option value={roleOption.id}>
                {formatRoleOption(roleOption.id, roleOption.role, roleOption.organizationId)}
              </option>
            {/each}
          </select>
        </div>
      {/if}

      <div class="flex items-center gap-2.5 mb-3">
        <div class="w-9 h-9 rounded-full bg-primary-600 text-white flex items-center justify-center font-semibold text-sm shrink-0">
          {user?.first_name?.[0]}{user?.last_name?.[0]}
        </div>
        <div class="min-w-0">
          <p class="text-sm font-medium text-gray-900 truncate">{user?.first_name} {user?.last_name}</p>
          <p class="text-xs text-gray-500">{getRoleLabel(user?.role)}</p>
        </div>
      </div>

      <div class="space-y-0.5">
        <a href="/profile" onclick={handleNavClick} class="flex items-center gap-2 px-2 py-1.5 text-xs text-gray-600 hover:bg-gray-50 rounded-lg transition-colors">
          👤 {$_('navigation.profile')}
        </a>
        <a href="/settings" onclick={handleNavClick} class="flex items-center gap-2 px-2 py-1.5 text-xs text-gray-600 hover:bg-gray-50 rounded-lg transition-colors">
          ⚙️ Paramètres
        </a>
        <a href="/settings/gdpr" onclick={handleNavClick} class="flex items-center gap-2 px-2 py-1.5 text-xs text-gray-600 hover:bg-gray-50 rounded-lg transition-colors">
          🔒 Données RGPD
        </a>
        <button
          onclick={logout}
          class="w-full flex items-center gap-2 px-2 py-1.5 text-xs text-red-600 hover:bg-red-50 rounded-lg transition-colors"
          data-testid="mobile-drawer-logout"
        >
          🚪 {$_('navigation.logout')}
        </button>
      </div>
    </div>
  </aside>
{/if}

<!-- Global keyboard handler -->
<svelte:window onkeydown={(e) => e.key === 'Escape' && drawerOpen && closeDrawer()} />
