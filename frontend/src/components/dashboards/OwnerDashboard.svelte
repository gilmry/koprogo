<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import { authStore } from '../../stores/auth';
  import { api } from '../../lib/api';
  import type { Building, Unit, Expense } from '../../lib/types';

  import { ticketsApi, type Ticket } from '../../lib/api/tickets';
  import { notificationsApi, type Notification as AppNotification } from '../../lib/api/notifications';
  import { formatDateShort, formatDate } from "../../lib/utils/date.utils";
  import { formatCurrency } from "../../lib/utils/finance.utils";

  $: user = $authStore.user;

  interface OwnerTicket {
    id: string;
    title: string;
    status: string;
    priority: string;
    category: string;
    created_at: string;
  }

  interface OwnerNotification {
    id: string;
    title: string;
    message: string;
    notification_type: string;
    is_read: boolean;
    created_at: string;
  }

  interface OwnerStats {
    total_buildings: number;
    total_units: number;
    total_owners: number;
    pending_expenses_count: number;
    pending_expenses_amount: number;
    next_meeting: {
      id: string;
      date: string;
      building_name: string;
    } | null;
  }

  interface BoardMandate {
    id: string;
    building_id: string;
    building_name: string;
    building_address: string;
    position: string;
    mandate_start: string;
    mandate_end: string;
    days_remaining: number;
    expires_soon: boolean;
  }

  let stats: OwnerStats | null = null;
  let recentBuildings: Building[] = [];
  let recentUnits: Unit[] = [];
  let boardMandates: BoardMandate[] = [];
  let myTickets: OwnerTicket[] = [];
  let unreadNotifications: OwnerNotification[] = [];
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    await loadDashboardData();
  });

  async function loadDashboardData() {
    try {
      loading = true;
      const [statsData, buildingsData, unitsData, mandatesData] = await Promise.all([
        api.get<OwnerStats>('/stats/owner'),
        api.get<{ data: Building[] }>('/buildings?page=1&per_page=3'),
        api.get<{ data: Unit[] }>('/units?page=1&per_page=5'),
        api.get<{ mandates: BoardMandate[] }>('/board-members/my-mandates'),
      ]);
      stats = statsData;
      recentBuildings = buildingsData.data;
      recentUnits = unitsData.data;
      boardMandates = mandatesData.mandates;

      // Load tickets and notifications (non-blocking)
      try {
        const [ticketsData, notifData] = await Promise.all([
          ticketsApi.listMy(),
          notificationsApi.getUnread(),
        ]);
        myTickets = (ticketsData as OwnerTicket[]).slice(0, 5);
        unreadNotifications = (notifData as OwnerNotification[]).slice(0, 5);
      } catch {
        // Non-critical, ignore errors
      }

      loading = false;
    } catch (err) {
      error = err instanceof Error ? err.message : $_('common.error.loadData');
      loading = false;
      console.error('Error fetching owner dashboard data:', err);
    }
  }

  $: openTicketsCount = myTickets.filter(t => t.status !== 'Closed' && t.status !== 'Cancelled' && t.status !== 'Resolved').length;

  function getUnitTypeIcon(type: string): string {
    const icons: Record<string, string> = {
      'Apartment': '🏠',
      'Parking': '🚗',
      'Storage': '📦'
    };
    return icons[type] || '📋';
  }

  function getUnitTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      'Apartment': $_('common.unitTypes.apartment'),
      'Parking': $_('common.unitTypes.parking'),
      'Storage': $_('common.unitTypes.storage')
    };
    return labels[type] || type;
  }

  function getPositionLabel(position: string): string {
    const labels: Record<string, string> = {
      'president': $_('common.boardPositions.president'),
      'treasurer': $_('common.boardPositions.treasurer'),
      'secretary': $_('common.boardPositions.secretary')
    };
    return labels[position] || position;
  }

  function getPositionIcon(position: string): string {
    const icons: Record<string, string> = {
      'president': '👑',
      'treasurer': '💰',
      'secretary': '📝'
    };
    return icons[position] || '🎯';
  }

  function formatFullDate(dateString: string): string {
    return formatDate(dateString);
  }
</script>

<div data-testid="owner-dashboard">
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-2">
      {$_('common.welcome')}, {user?.first_name} 👋
    </h1>
    <p class="text-gray-600">
      {$_('dashboards.owner.title')}
    </p>
  </div>

  {#if loading}
    <div class="flex items-center justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600" data-testid="owner-dashboard-spinner"></div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-8">
      <p class="text-red-800 font-medium">{$_('common.error.title')}</p>
      <p class="text-red-600 text-sm">{error}</p>
    </div>
  {:else if stats}
    <!-- Stats Cards -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">{$_('dashboards.owner.stats.buildings')}</span>
          <span class="text-2xl">🏢</span>
        </div>
        <p class="text-3xl font-bold text-gray-900">{stats.total_buildings}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.total_units} {$_('dashboards.owner.stats.unitsTotal')}</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">{$_('dashboards.owner.stats.expensesToPay')}</span>
          <span class="text-2xl">💰</span>
        </div>
        <p class="text-3xl font-bold text-orange-600">{formatCurrency(stats.pending_expenses_amount)}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.pending_expenses_count} {$_('dashboards.owner.stats.expensesPending')}</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">{$_('dashboards.owner.stats.nextMeeting')}</span>
          <span class="text-2xl">📅</span>
        </div>
        {#if stats.next_meeting}
          <p class="text-xl font-bold text-gray-900">{formatDateShort(stats.next_meeting.date)}</p>
          <p class="text-sm text-gray-500 mt-1">{stats.next_meeting.building_name}</p>
        {:else}
          <p class="text-lg font-medium text-gray-500">{$_('dashboards.owner.stats.noMeetingsPlanned')}</p>
        {/if}
      </div>
    </div>

    <!-- Board Member Mandates (if applicable) -->
    {#if boardMandates.length > 0}
      <div class="mb-8">
        <div class="bg-gradient-to-r from-primary-50 to-primary-100 border-2 border-primary-300 rounded-lg shadow-lg p-6">
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-3">
              <span class="text-4xl">🎖️</span>
              <div>
                <h2 class="text-2xl font-bold text-gray-900">{$_('dashboards.owner.boardMember.title')}</h2>
                <p class="text-sm text-gray-600">{$_('dashboards.owner.boardMember.subtitle')}</p>
              </div>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
            {#each boardMandates as mandate}
              <div class="bg-white rounded-lg border-2 border-primary-200 p-4 hover:border-primary-400 transition">
                <div class="flex items-start justify-between mb-3">
                  <div class="flex items-center gap-2">
                    <span class="text-3xl">{getPositionIcon(mandate.position)}</span>
                    <div>
                      <h3 class="font-bold text-gray-900">{getPositionLabel(mandate.position)}</h3>
                      <p class="text-sm text-gray-600">{mandate.building_name}</p>
                    </div>
                  </div>
                  {#if mandate.expires_soon}
                    <span class="px-2 py-1 bg-orange-100 text-orange-800 text-xs font-medium rounded">
                      ⚠️ {$_('dashboards.owner.mandate.expiresSoon')}
                    </span>
                  {/if}
                </div>

                <p class="text-xs text-gray-500 mb-3">{mandate.building_address}</p>

                <div class="flex items-center justify-between text-sm mb-3">
                  <span class="text-gray-600">{$_('dashboards.owner.mandate.mandate')}:</span>
                  <span class="font-medium text-gray-900">
                    {formatFullDate(mandate.mandate_start)} - {formatFullDate(mandate.mandate_end)}
                  </span>
                </div>

                <div class="flex items-center justify-between text-sm mb-4">
                  <span class="text-gray-600">{$_('dashboards.owner.mandate.remaining')}:</span>
                  <span class="font-medium {mandate.expires_soon ? 'text-orange-600' : 'text-green-600'}">
                    {mandate.days_remaining} {$_('dashboards.owner.mandate.days')}
                  </span>
                </div>

                <a
                  href="/board-dashboard?building_id={mandate.building_id}"
                  class="block w-full text-center bg-primary-600 hover:bg-primary-700 text-white font-medium py-2 px-4 rounded transition"
                >
                  📊 {$_('dashboards.owner.mandate.boardDashboard')}
                </a>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Main Content -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!-- Buildings -->
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200 flex justify-between items-center">
          <h2 class="text-lg font-semibold text-gray-900">{$_('dashboards.owner.myBuildings')}</h2>
          <a href="/buildings" class="text-sm text-primary-600 hover:text-primary-700 font-medium">
            {$_('common.seeAll')} →
          </a>
        </div>
        <div class="p-6">
          {#if recentBuildings.length > 0}
            <div class="space-y-4">
              {#each recentBuildings as building}
                <div class="p-4 border border-gray-200 rounded-lg hover:border-primary-500 transition">
                  <div class="flex items-center justify-between mb-2">
                    <h3 class="font-semibold text-gray-900">{building.name}</h3>
                    <span class="text-2xl">🏢</span>
                  </div>
                  <p class="text-sm text-gray-600">{building.address}</p>
                  <p class="text-sm text-gray-500 mt-1">{building.city}, {building.postal_code}</p>
                  <p class="text-xs text-gray-400 mt-1">{building.total_units} lots</p>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-center py-8">
              <p class="text-gray-500">{$_('dashboards.owner.noBuildings')}</p>
            </div>
          {/if}
        </div>
      </div>

      <!-- Recent Units -->
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200 flex justify-between items-center">
          <h2 class="text-lg font-semibold text-gray-900">{$_('dashboards.owner.recentUnits')}</h2>
          <a href="/units" class="text-sm text-primary-600 hover:text-primary-700 font-medium">
            {$_('common.seeAll')} →
          </a>
        </div>
        <div class="p-6">
          {#if recentUnits.length > 0}
            <div class="space-y-4">
              {#each recentUnits as unit}
                <div class="p-4 border border-gray-200 rounded-lg hover:border-primary-500 transition">
                  <div class="flex items-center justify-between mb-2">
                    <h3 class="font-semibold text-gray-900">Lot {unit.unit_number}</h3>
                    <span class="text-2xl">{getUnitTypeIcon(unit.unit_type)}</span>
                  </div>
                  <p class="text-sm text-gray-600">{getUnitTypeLabel(unit.unit_type)} - Étage {unit.floor}</p>
                  <p class="text-sm text-gray-500 mt-1">{unit.surface_area} m² • {Math.round(unit.quota)}/1000èmes</p>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-center py-8">
              <p class="text-gray-500">{$_('dashboards.owner.noUnits')}</p>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Tickets & Notifications -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mt-8">
      <!-- My Tickets -->
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200 flex justify-between items-center">
          <div class="flex items-center gap-2">
            <h2 class="text-lg font-semibold text-gray-900">{$_('dashboards.owner.myTickets')}</h2>
            {#if openTicketsCount > 0}
              <span class="px-2 py-0.5 bg-orange-100 text-orange-800 text-xs font-medium rounded-full">{openTicketsCount} {openTicketsCount > 1 ? $_('common.plural.open') : $_('common.singular.open')}</span>
            {/if}
          </div>
          <a href="/owner/tickets" class="text-sm text-primary-600 hover:text-primary-700 font-medium">
            {$_('common.seeAll')} →
          </a>
        </div>
        <div class="p-6">
          {#if myTickets.length > 0}
            <div class="space-y-3">
              {#each myTickets as ticket}
                <a href="/ticket-detail?id={ticket.id}" class="block p-3 border border-gray-200 rounded-lg hover:border-primary-300 transition">
                  <div class="flex items-center justify-between mb-1">
                    <h3 class="text-sm font-medium text-gray-900 truncate">{ticket.title}</h3>
                    <span class="text-xs px-2 py-0.5 rounded-full font-medium
                      {ticket.status === 'Open' ? 'bg-blue-100 text-blue-800' :
                       ticket.status === 'InProgress' ? 'bg-yellow-100 text-yellow-800' :
                       ticket.status === 'Resolved' ? 'bg-green-100 text-green-800' :
                       ticket.status === 'Assigned' ? 'bg-purple-100 text-purple-800' :
                       'bg-gray-100 text-gray-800'}">{ticket.status}</span>
                  </div>
                  <div class="flex items-center gap-2 text-xs text-gray-500">
                    <span>{ticket.category}</span>
                    <span>·</span>
                    <span>{formatDateShort(ticket.created_at)}</span>
                  </div>
                </a>
              {/each}
            </div>
          {:else}
            <div class="text-center py-6">
              <p class="text-gray-500 text-sm">{$_('dashboards.owner.noMaintenanceTickets')}</p>
              <a href="/owner/tickets" class="text-sm text-primary-600 hover:text-primary-700 mt-1 inline-block">{$_('dashboards.owner.createTicket')}</a>
            </div>
          {/if}
        </div>
      </div>

      <!-- Notifications -->
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200 flex justify-between items-center">
          <div class="flex items-center gap-2">
            <h2 class="text-lg font-semibold text-gray-900">{$_('common.notifications')}</h2>
            {#if unreadNotifications.length > 0}
              <span class="px-2 py-0.5 bg-red-100 text-red-800 text-xs font-medium rounded-full">{unreadNotifications.length} {unreadNotifications.length > 1 ? $_('common.plural.unread') : $_('common.singular.unread')}</span>
            {/if}
          </div>
          <a href="/notifications" class="text-sm text-primary-600 hover:text-primary-700 font-medium">
            {$_('common.seeAll')} →
          </a>
        </div>
        <div class="p-6">
          {#if unreadNotifications.length > 0}
            <div class="space-y-3">
              {#each unreadNotifications as notif}
                <div class="p-3 border border-gray-200 rounded-lg bg-blue-50/50">
                  <h3 class="text-sm font-medium text-gray-900">{notif.title}</h3>
                  <p class="text-xs text-gray-600 mt-0.5 line-clamp-2">{notif.message}</p>
                  <p class="text-xs text-gray-400 mt-1">{formatDateShort(notif.created_at)}</p>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-center py-6">
              <p class="text-gray-500 text-sm">{$_('dashboards.owner.noUnreadNotifications')}</p>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="mt-8">
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200">
          <h2 class="text-lg font-semibold text-gray-900">{$_('dashboards.owner.quickActions')}</h2>
        </div>
        <div class="p-6">
          <div class="grid grid-cols-2 md:grid-cols-{boardMandates.length > 0 ? '5' : '4'} gap-4">
            <a href="/buildings" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
              <span class="text-4xl mb-2 group-hover:scale-110 transition">🏢</span>
              <span class="text-sm font-medium text-gray-700">{$_('navigation.buildings')}</span>
            </a>
            <a href="/units" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
              <span class="text-4xl mb-2 group-hover:scale-110 transition">🚪</span>
              <span class="text-sm font-medium text-gray-700">{$_('navigation.units')}</span>
            </a>
            <a href="/expenses" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
              <span class="text-4xl mb-2 group-hover:scale-110 transition">💰</span>
              <span class="text-sm font-medium text-gray-700">{$_('navigation.expenses')}</span>
            </a>
            <a href="/meetings" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
              <span class="text-4xl mb-2 group-hover:scale-110 transition">📅</span>
              <span class="text-sm font-medium text-gray-700">{$_('navigation.meetings')}</span>
            </a>
            <a href="/owner/tickets" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
              <span class="text-4xl mb-2 group-hover:scale-110 transition">🎫</span>
              <span class="text-sm font-medium text-gray-700">{$_('navigation.tickets')}</span>
            </a>
            <a href="/owner/payments" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
              <span class="text-4xl mb-2 group-hover:scale-110 transition">💳</span>
              <span class="text-sm font-medium text-gray-700">{$_('navigation.payments')}</span>
            </a>
            {#if boardMandates.length > 0}
              <a
                href="/board-dashboard?building_id={boardMandates[0].building_id}"
                class="flex flex-col items-center justify-center p-6 border-2 border-primary-300 bg-primary-50 rounded-lg hover:border-primary-500 hover:bg-primary-100 transition group"
              >
                <span class="text-4xl mb-2 group-hover:scale-110 transition">🎖️</span>
                <span class="text-sm font-medium text-primary-700">{$_('navigation.council')}</span>
              </a>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
