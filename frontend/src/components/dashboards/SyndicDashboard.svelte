<script lang="ts">
  import Icone from "../ui/Icone.svelte";
  import TableDesAcp from "./TableDesAcp.svelte";
  import EncartIntegriteDonnees from "./EncartIntegriteDonnees.svelte";
  import DecompteLegal from "./DecompteLegal.svelte";
  // Svelte 5 runes mode
  import { _ } from "../../lib/i18n";
  import { authStore } from "../../stores/auth";
  import { api } from "../../lib/api";
  import type { Owner } from "../../lib/types";
  import OwnerEditModal from "../OwnerEditModal.svelte";
  import { ticketsApi } from "../../lib/api/tickets";
  import { notificationsApi } from "../../lib/api/notifications";
  import { formatDateShort } from "../../lib/utils/date.utils";
  import { formatCurrency } from "../../lib/utils/finance.utils";

  let user = $derived($authStore.user);

  interface SyndicStats {
    total_buildings: number;
    total_units: number;
    declared_units: number;
    total_owners: number;
    pending_expenses_count: number;
    pending_expenses_amount: number;
    next_meeting: {
      id: string;
      date: string;
      building_name: string;
    } | null;
  }

  interface UrgentTask {
    task_type: string;
    title: string;
    description: string;
    priority: string;
    building_name: string | null;
    entity_id: string | null;
    due_date: string | null;
    /**
     * L'article qui fonde l'échéance, quand il y en a un.
     *
     * `null` pour les tâches sans origine légale : un retard de paiement est
     * contractuel, une assemblée à venir est un rendez-vous. Seule la
     * transmission du procès-verbal porte aujourd'hui un délai imposé par la
     * loi (Art. 3.87 § 12), et rien ne la suivait avant le 2026-09-10.
     */
    article?: string | null;
    /** Le délai que l'article accorde — le dénominateur du décompte. */
    delai_legal_jours?: number | null;
  }

  let stats = $state<SyndicStats | null>(null);
  let urgentTasks = $state<UrgentTask[]>([]);
  let recentOwners = $state<Owner[]>([]);
  let openTicketsCount = $state(0);
  let unreadNotifCount = $state(0);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Modal state
  let isModalOpen = $state(false);
  let selectedOwner = $state<Owner | null>(null);

  $effect(() => {
    loadDashboardData();
  });

  async function loadDashboardData() {
    try {
      loading = true;
      const [statsData, tasksData, ownersData] = await Promise.all([
        api.get<SyndicStats>("/stats/syndic"),
        api.get<UrgentTask[]>("/stats/syndic/urgent-tasks"),
        api.get<{ data: Owner[] }>("/owners?page=1&per_page=5"),
      ]);
      stats = statsData;
      urgentTasks = tasksData;
      recentOwners = ownersData.data;
      loading = false;

      // Load ticket/notification counts (non-blocking)
      try {
        const [ticketStats, unreadNotifs] = await Promise.all([
          ticketsApi.getStatistics(),
          notificationsApi.getUnread(),
        ]);
        openTicketsCount = (ticketStats as any)?.open_count ?? 0;
        unreadNotifCount = Array.isArray(unreadNotifs)
          ? unreadNotifs.length
          : 0;
      } catch {
        // Non-critical
      }
    } catch (err) {
      error = err instanceof Error ? err.message : $_("common.error.loadStats");
      loading = false;
      console.error("Error fetching stats:", err);
    }
  }

  function openEditModal(owner: Owner) {
    selectedOwner = owner;
    isModalOpen = true;
  }

  function closeModal() {
    isModalOpen = false;
    selectedOwner = null;
  }

  async function handleOwnerSaved() {
    await loadDashboardData();
  }

  function getTaskIcon(taskType: string): string {
    switch (taskType) {
      case "expense":
        return "💰";
      case "meeting":
        return "📄";
      default:
        return "📋";
    }
  }

  function getTaskStyles(priority: string): {
    bg: string;
    border: string;
    text: string;
  } {
    switch (priority) {
      case "urgent":
        return {
          bg: "bg-red-50",
          border: "border-red-200",
          text: "text-red-600",
        };
      case "high":
        return {
          bg: "bg-orange-50",
          border: "border-orange-200",
          text: "text-orange-600",
        };
      default:
        return {
          bg: "bg-yellow-50",
          border: "border-yellow-200",
          text: "text-yellow-600",
        };
    }
  }

  function getPriorityLabel(priority: string): string {
    switch (priority) {
      case "urgent":
        return $_("common.priority.urgent");
      case "high":
        return $_("common.priority.important");
      default:
        return $_("common.priority.toProcess");
    }
  }
</script>

<div data-testid="syndic-dashboard">
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-2">
      {$_("common.welcome")}, {user?.first_name} 👋
    </h1>
    <p class="text-gray-600">
      {$_("dashboards.syndic.title")} - {$_("dashboards.syndic.subtitle")}
    </p>
  </div>

  {#if loading}
    <div class="flex items-center justify-center py-12">
      <div
        class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"
        data-testid="syndic-dashboard-spinner"
      ></div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-8">
      <p class="text-red-800 font-medium">{$_("common.error.title")}</p>
      <p class="text-red-600 text-sm">{error}</p>
    </div>
  {:else if stats}
    <!-- Stats Cards -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
      <a
        href="/buildings"
        data-testid="syndic-kpi-buildings"
        class="block rounded-lg bg-white p-6 shadow transition-colors hover:bg-chip-bg"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium"
            >{$_("dashboards.syndic.stats.buildingsManaged")}</span
          >
          <Icone nom="buildings" taille={20} class="shrink-0 text-muted" />
        </div>
        <p class="text-3xl font-bold text-gray-900">{stats.total_buildings}</p>
        <!-- « 0 lots au total » à côté de « 8 Lots » sur la liste des
             immeubles semblait contradictoire. Les deux nombres sont exacts
             mais ne mesurent pas la même chose : l'un compte les lots
             encodés, l'autre ceux déclarés à l'acte de base. La fraction lève
             l'ambiguïté, et n'est affichée que lorsqu'il y a un écart. -->
        <p class="text-sm text-gray-500 mt-1">
          {stats.declared_units > stats.total_units
            ? $_("dashboards.syndic.stats.unitsEncodedOfDeclared", {
                values: {
                  encoded: stats.total_units,
                  declared: stats.declared_units,
                },
              })
            : `${stats.total_units} ${$_("dashboards.syndic.stats.unitsEncoded")}`}
        </p>
      </a>

      <a
        href="/owners"
        data-testid="syndic-kpi-owners"
        class="block rounded-lg bg-white p-6 shadow transition-colors hover:bg-chip-bg"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium"
            >{$_("dashboards.syndic.stats.owners")}</span
          >
          <Icone nom="owners" taille={20} class="shrink-0 text-muted" />
        </div>
        <p class="text-3xl font-bold text-gray-900">{stats.total_owners}</p>
        <p class="text-sm text-gray-500 mt-1">
          {$_("dashboards.syndic.stats.active")}
        </p>
      </a>

      <a
        href="/expenses"
        data-testid="syndic-kpi-expenses"
        class="block rounded-lg bg-white p-6 shadow transition-colors hover:bg-chip-bg"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium"
            >{$_("dashboards.syndic.stats.pendingExpenses")}</span
          >
          <Icone nom="expenses" taille={20} class="shrink-0 text-muted" />
        </div>
        <p class="text-3xl font-bold text-gray-900">
          {stats.pending_expenses_count}
        </p>
        <p class="text-sm text-orange-600 mt-1">
          {formatCurrency(stats.pending_expenses_amount)}
        </p>
      </a>

      <a
        href="/meetings"
        data-testid="syndic-kpi-meetings"
        class="block rounded-lg bg-white p-6 shadow transition-colors hover:bg-chip-bg"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium"
            >{$_("dashboards.syndic.stats.nextMeeting")}</span
          >
          <Icone nom="meetings" taille={20} class="shrink-0 text-muted" />
        </div>
        {#if stats.next_meeting}
          <p class="text-xl font-bold text-gray-900">
            {formatDateShort(stats.next_meeting.date)}
          </p>
          <p class="text-sm text-gray-500 mt-1">
            {stats.next_meeting.building_name}
          </p>
        {:else}
          <p class="text-lg font-medium text-gray-500">
            {$_("dashboards.syndic.stats.noMeetingsPlanned")}
          </p>
        {/if}
      </a>
    </div>

    <!-- Main Content -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!--
        L'écart entre lots encodés et déclarés, avec sa CONSÉQUENCE.

        La fraction « 23/32 » était déjà affichée, en gris, sous le nombre
        d'immeubles. Le fait était donc connu ; ce qui manquait, c'est ce
        qu'il coûte — les quotités se répartissant sur les lots encodés, les
        appels de fonds portent sur une base incomplète et les copropriétaires
        encodés paient la part des absents.
      -->
      {#if stats}
        <div class="mb-6">
          <EncartIntegriteDonnees
            encodes={stats.total_units}
            declares={stats.declared_units}
          />
        </div>
      {/if}

      <!--
        La table « Mes ACP » : blocs, lots encodés sur déclarés, quotités sur
        le total de l'acte. Un syndic gère plusieurs copropriétés, et sa
        première question en ouvrant l'écran est « laquelle a un problème ».
      -->
      <div class="mb-6">
        <TableDesAcp />
      </div>

      <!-- Urgent Tasks -->
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200">
          <h2 class="text-lg font-semibold text-gray-900">
            {$_("dashboards.syndic.urgentTasks")}
          </h2>
        </div>
        <div class="p-6">
          {#if urgentTasks.length > 0}
            <div class="space-y-4">
              {#each urgentTasks as task}
                {@const styles = getTaskStyles(task.priority)}
                <!--
                  `task-row` et `data-task-kind` : la remise les impose pour
                  que les recettes visent une tâche par sa NATURE plutôt que
                  par sa position dans la liste, qui dépend des données.
                -->
                <div
                  data-testid="task-row"
                  data-task-kind={task.task_type}
                  data-task-priority={task.priority}
                  class="flex items-start space-x-3 p-4 {styles.bg} border {styles.border} rounded-lg"
                >
                  <!--
                    Le décompte d'échéance légale prend la place de l'icône
                    quand la tâche en a une : un article et une marge en jours
                    informent plus qu'un pictogramme de catégorie.
                  -->
                  {#if task.due_date && task.article && task.delai_legal_jours}
                    <DecompteLegal
                      echeance={task.due_date}
                      article={task.article}
                      delaiJours={task.delai_legal_jours}
                    />
                  {:else}
                    <span class="text-2xl">{getTaskIcon(task.task_type)}</span>
                  {/if}
                  <div class="flex-1">
                    <p class="text-sm font-medium text-gray-900">
                      {task.title}
                    </p>
                    <p class="text-sm text-gray-600">{task.description}</p>
                    {#if task.building_name}
                      <p class="text-xs text-gray-500 mt-1">
                        {task.building_name}
                      </p>
                    {/if}
                    <p class="text-xs {styles.text} mt-1">
                      {getPriorityLabel(task.priority)}
                    </p>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-center py-8">
              <p class="text-gray-500">
                {$_("dashboards.syndic.noUrgentTasks")}
              </p>
              <p class="text-sm text-muted mt-2">
                {$_("dashboards.syndic.allUnderControl")}
              </p>
            </div>
          {/if}
        </div>
      </div>

      <!-- Quick Actions -->
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200">
          <h2 class="text-lg font-semibold text-gray-900">
            {$_("dashboards.syndic.quickActions")}
          </h2>
        </div>
        <div class="p-6">
          <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            <a
              data-testid="syndic-buildings-tile"
              href="/buildings"
              class="flex flex-col items-center justify-center p-4 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
            >
              <span class="text-3xl mb-2 group-hover:scale-110 transition"
                >🏢</span
              >
              <span class="text-sm font-medium text-gray-700"
                >{$_("navigation.buildings")}</span
              >
            </a>
            <a
              data-testid="syndic-owners-tile"
              href="/owners"
              class="flex flex-col items-center justify-center p-4 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
            >
              <span class="text-3xl mb-2 group-hover:scale-110 transition"
                >👥</span
              >
              <span class="text-sm font-medium text-gray-700"
                >{$_("navigation.owners")}</span
              >
            </a>
            <a
              data-testid="syndic-expenses-tile"
              href="/expenses"
              class="flex flex-col items-center justify-center p-4 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
            >
              <span class="text-3xl mb-2 group-hover:scale-110 transition"
                >💰</span
              >
              <span class="text-sm font-medium text-gray-700"
                >{$_("navigation.expenses")}</span
              >
            </a>
            <a
              data-testid="syndic-meetings-tile"
              href="/meetings"
              class="flex flex-col items-center justify-center p-4 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
            >
              <span class="text-3xl mb-2 group-hover:scale-110 transition"
                >📅</span
              >
              <span class="text-sm font-medium text-gray-700"
                >{$_("navigation.meetings")}</span
              >
            </a>
            <a
              data-testid="syndic-tickets-tile"
              href="/tickets"
              class="relative flex flex-col items-center justify-center p-4 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
            >
              <span class="text-3xl mb-2 group-hover:scale-110 transition"
                >🎫</span
              >
              <span class="text-sm font-medium text-gray-700"
                >{$_("navigation.tickets")}</span
              >
              {#if openTicketsCount > 0}
                <span
                  class="absolute top-2 right-2 px-1.5 py-0.5 bg-red-500 text-white text-xs font-bold rounded-full"
                  >{openTicketsCount}</span
                >
              {/if}
            </a>
            <a
              data-testid="syndic-convocations-tile"
              href="/convocations"
              class="flex flex-col items-center justify-center p-4 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
            >
              <span class="text-3xl mb-2 group-hover:scale-110 transition"
                >📨</span
              >
              <span class="text-sm font-medium text-gray-700"
                >{$_("navigation.convocations")}</span
              >
            </a>
            <a
              data-testid="syndic-work-reports-tile"
              href="/work-reports"
              class="flex flex-col items-center justify-center p-4 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
            >
              <span class="text-3xl mb-2 group-hover:scale-110 transition"
                >🔧</span
              >
              <span class="text-sm font-medium text-gray-700"
                >{$_("navigation.workReports")}</span
              >
            </a>
            <a
              data-testid="syndic-notifications-tile"
              href="/notifications"
              class="relative flex flex-col items-center justify-center p-4 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
            >
              <span class="text-3xl mb-2 group-hover:scale-110 transition"
                >🔔</span
              >
              <span class="text-sm font-medium text-gray-700"
                >{$_("navigation.notifications")}</span
              >
              {#if unreadNotifCount > 0}
                <span
                  class="absolute top-2 right-2 px-1.5 py-0.5 bg-red-500 text-white text-xs font-bold rounded-full"
                  >{unreadNotifCount}</span
                >
              {/if}
            </a>
          </div>
        </div>
      </div>
    </div>

    <!-- Recent Owners Section -->
    <div class="mt-8">
      <div class="bg-white rounded-lg shadow">
        <div
          class="p-6 border-b border-gray-200 flex justify-between items-center"
        >
          <h2 class="text-lg font-semibold text-gray-900">
            {$_("dashboards.syndic.recentOwners")}
          </h2>
          <a
            data-testid="syndic-owners-all-link"
            href="/owners"
            class="text-sm text-primary-600 hover:text-primary-700 font-medium"
          >
            {$_("common.seeAll")} →
          </a>
        </div>
        <div class="p-6">
          {#if recentOwners.length > 0}
            <div class="space-y-3">
              {#each recentOwners as owner}
                <div
                  class="flex items-center justify-between p-4 bg-gray-50 rounded-lg hover:bg-gray-100 transition"
                >
                  <div class="flex-1">
                    <h3 class="font-medium text-gray-900">
                      {owner.first_name}
                      {owner.last_name}
                    </h3>
                    <p class="text-sm text-gray-600">
                      📧 {owner.email}
                    </p>
                    {#if owner.phone}
                      <p class="text-sm text-gray-500">
                        📞 {owner.phone}
                      </p>
                    {/if}
                  </div>
                  <button
                    data-testid="syndic-owner-edit-button"
                    onclick={() => openEditModal(owner)}
                    class="ml-4 px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 transition"
                  >
                    {$_("common.edit")}
                  </button>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-center py-8">
              <p class="text-gray-500">
                {$_("dashboards.syndic.noOwnersRecorded")}
              </p>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- Owner Edit Modal -->
  <OwnerEditModal
    owner={selectedOwner}
    isOpen={isModalOpen}
    onclose={closeModal}
    onsave={handleOwnerSaved}
  />
</div>
