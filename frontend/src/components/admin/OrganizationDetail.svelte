<script lang="ts">
  // Svelte 5 runes mode
  import { onMount } from "svelte";
  import { _ } from "../../lib/i18n";
  import { api } from "../../lib/api";
  import { formatDate } from "../../lib/utils/date.utils";

  // Détail d'une organisation, absent jusqu'ici : la liste admin n'offrait
  // aucun moyen d'ouvrir une organisation, seulement de la modifier ou de la
  // supprimer (UX1 de l'audit du 2026-08-30).
  //
  // Les données sont assemblées à partir des listes existantes plutôt que
  // d'un nouvel endpoint agrégé. Il n'existe ni `GET /organizations/{id}`, ni
  // filtre par organisation sur /users, /acps ou /buildings ; en ajouter
  // quatre pour une page d'administration dont les volumes se comptent en
  // dizaines serait disproportionné. Si ces listes grossissent, c'est ici
  // qu'il faudra basculer sur un endpoint dédié.

  interface Organization {
    id: string;
    name: string;
    slug: string;
    contact_email: string;
    contact_phone?: string | null;
    subscription_plan: string;
    max_buildings: number;
    max_users: number;
    is_active: boolean;
    created_at: string;
  }

  interface Acp {
    id: string;
    organization_id: string;
    name: string;
    address_street: string;
    address_postal_code: string;
    address_city: string;
    total_tantiemes: number;
  }

  interface Building {
    id: string;
    acp_id: string;
    name: string;
    address: string;
    city: string;
    total_units: number;
    units_count: number;
  }

  interface User {
    id: string;
    organization_id: string | null;
    email: string;
    first_name: string;
    last_name: string;
    role: string;
    is_active: boolean;
  }

  let organizationId = $state("");
  let organization = $state<Organization | null>(null);
  let acps = $state<Acp[]>([]);
  let buildings = $state<Building[]>([]);
  let users = $state<User[]>([]);
  let loading = $state(true);
  let error = $state("");

  onMount(async () => {
    organizationId = new URLSearchParams(window.location.search).get("id") ?? "";
    if (!organizationId) {
      error = $_("admin.organizations.detail.missingId");
      loading = false;
      return;
    }

    try {
      // Les quatre appels sont indépendants : les enchaîner tripleraient le
      // temps d'affichage sans rien apporter.
      const [orgPage, acpList, buildingPage, userPage] = await Promise.all([
        api.get<{ data: Organization[] }>("/organizations?page=1&per_page=200"),
        api.get<Acp[]>("/acps"),
        api.get<{ data: Building[] }>("/buildings?page=1&per_page=200"),
        api.get<{ data: User[] }>("/users?page=1&per_page=200"),
      ]);

      organization = orgPage.data.find((o) => o.id === organizationId) ?? null;
      if (!organization) {
        error = $_("admin.organizations.detail.notFound");
        return;
      }

      acps = acpList.filter((a) => a.organization_id === organizationId);
      const acpIds = new Set(acps.map((a) => a.id));
      // Les immeubles ne portent pas d'organization_id : le rattachement
      // passe par l'ACP.
      buildings = buildingPage.data.filter((b) => acpIds.has(b.acp_id));
      users = userPage.data.filter((u) => u.organization_id === organizationId);
    } catch (err) {
      error = err instanceof Error ? err.message : $_("common.error");
    } finally {
      loading = false;
    }
  });

  let encodedUnits = $derived(buildings.reduce((n, b) => n + (b.units_count ?? 0), 0));
  let declaredUnits = $derived(buildings.reduce((n, b) => n + (b.total_units ?? 0), 0));
</script>

<div class="container mx-auto px-4 py-8" data-testid="organization-detail">
  <a href="/admin/organizations" class="text-sm text-primary-600 hover:underline">
    ← {$_("admin.organizations.detail.backToList")}
  </a>

  {#if loading}
    <p class="mt-6 text-gray-500">{$_("common.loading")}</p>
  {:else if error}
    <p class="mt-6 text-red-600" data-testid="organization-detail-error">{error}</p>
  {:else if organization}
    <div class="mt-4 flex items-start justify-between">
      <div>
        <h1 class="text-3xl font-bold text-gray-900" data-testid="organization-detail-name">
          {organization.name}
        </h1>
        <p class="mt-1 text-sm text-gray-500">/{organization.slug}</p>
      </div>
      <span
        class="px-3 py-1 rounded-full text-sm font-semibold {organization.is_active
          ? 'bg-green-100 text-green-800'
          : 'bg-red-100 text-red-800'}"
      >
        {organization.is_active ? $_("common.active") : $_("common.inactive")}
      </span>
    </div>

    <!-- Compteurs : ce que l'organisation contient réellement, en regard des
         limites de son plan. C'est l'information qui manquait le plus, la
         liste ne montrant que les plafonds. -->
    <div class="mt-6 grid grid-cols-2 gap-4 sm:grid-cols-4">
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-xs uppercase tracking-wider text-gray-500">
          {$_("admin.organizations.detail.acps")}
        </p>
        <p class="text-2xl font-bold text-gray-900" data-testid="stat-acps">{acps.length}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-xs uppercase tracking-wider text-gray-500">
          {$_("admin.organizations.detail.buildings")}
        </p>
        <p class="text-2xl font-bold text-gray-900" data-testid="stat-buildings">
          {buildings.length} <span class="text-sm font-normal text-gray-500">/ {organization.max_buildings}</span>
        </p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-xs uppercase tracking-wider text-gray-500">
          {$_("admin.organizations.detail.users")}
        </p>
        <p class="text-2xl font-bold text-gray-900" data-testid="stat-users">
          {users.length} <span class="text-sm font-normal text-gray-500">/ {organization.max_users}</span>
        </p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-xs uppercase tracking-wider text-gray-500">
          {$_("admin.organizations.detail.units")}
        </p>
        <p class="text-2xl font-bold text-gray-900" data-testid="stat-units">
          {encodedUnits} <span class="text-sm font-normal text-gray-500">/ {declaredUnits}</span>
        </p>
      </div>
    </div>

    <div class="mt-6 bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900">
        {$_("admin.organizations.detail.contact")}
      </h2>
      <dl class="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-3 text-sm">
        <div>
          <dt class="text-gray-500">{$_("common.email")}</dt>
          <dd class="text-gray-900">{organization.contact_email}</dd>
        </div>
        <div>
          <dt class="text-gray-500">{$_("common.phone")}</dt>
          <dd class="text-gray-900">{organization.contact_phone || "—"}</dd>
        </div>
        <div>
          <dt class="text-gray-500">{$_("admin.organizations.detail.createdAt")}</dt>
          <dd class="text-gray-900">{formatDate(organization.created_at)}</dd>
        </div>
      </dl>
    </div>

    <div class="mt-6 bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900">
        {$_("admin.organizations.detail.acps")}
      </h2>
      {#if acps.length === 0}
        <p class="mt-3 text-sm text-gray-500">{$_("admin.organizations.detail.noAcp")}</p>
      {:else}
        <ul class="mt-3 divide-y divide-gray-200">
          {#each acps as acp (acp.id)}
            <li class="py-3" data-testid="organization-acp">
              <p class="text-sm font-medium text-gray-900">{acp.name}</p>
              <p class="text-sm text-gray-500">
                {acp.address_street}, {acp.address_postal_code} {acp.address_city}
                · {acp.total_tantiemes} {$_("admin.organizations.detail.tantiemes")}
              </p>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <div class="mt-6 bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900">
        {$_("admin.organizations.detail.buildings")}
      </h2>
      {#if buildings.length === 0}
        <p class="mt-3 text-sm text-gray-500">{$_("admin.organizations.detail.noBuilding")}</p>
      {:else}
        <ul class="mt-3 divide-y divide-gray-200">
          {#each buildings as building (building.id)}
            <li class="py-3" data-testid="organization-building">
              <a href={`/building-detail?id=${building.id}`} class="text-sm font-medium text-primary-600 hover:underline">
                {building.name}
              </a>
              <p class="text-sm text-gray-500">
                {building.address}, {building.city}
                · {building.units_count}/{building.total_units} {$_("admin.organizations.detail.unitsEncoded")}
              </p>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <div class="mt-6 bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900">
        {$_("admin.organizations.detail.users")}
      </h2>
      {#if users.length === 0}
        <p class="mt-3 text-sm text-gray-500">{$_("admin.organizations.detail.noUser")}</p>
      {:else}
        <ul class="mt-3 divide-y divide-gray-200">
          {#each users as user (user.id)}
            <li class="py-3 flex items-center justify-between" data-testid="organization-user">
              <div>
                <p class="text-sm font-medium text-gray-900">{user.first_name} {user.last_name}</p>
                <p class="text-sm text-gray-500">{user.email}</p>
              </div>
              <span class="text-xs px-2 py-1 rounded-full bg-gray-100 text-gray-700">{user.role}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>
