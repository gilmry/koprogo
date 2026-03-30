<script lang="ts">
  import { onMount } from "svelte";
  import { bookingsApi, type BookableResource, ResourceType } from "../../lib/api/bookings";
  import { toast } from "../../stores/toast";
  import { _ } from "../../lib/i18n";
  import ResourceCard from "./ResourceCard.svelte";
  import { withErrorHandling } from "../../lib/utils/error.utils";

  export let buildingId: string;
  export let showFilters = true;

  let resources: BookableResource[] = [];
  let filteredResources: BookableResource[] = [];
  let loading = true;
  let searchQuery = "";
  let selectedType: ResourceType | "all" = "all";
  let selectedAvailability: "available-only" | "all" = "available-only";

  onMount(async () => {
    await loadResources();
  });

  async function loadResources() {
    loading = true;
    const result = await withErrorHandling({
      action: () => selectedAvailability === "available-only"
        ? bookingsApi.listAvailableResources(buildingId)
        : bookingsApi.listResourcesByBuilding(buildingId),
      errorMessage: "Failed to load resources",
    });
    if (result) {
      resources = result;
      applyFilters();
    }
    loading = false;
  }

  function applyFilters() {
    filteredResources = resources.filter((resource) => {
      const matchesSearch =
        searchQuery === "" ||
        resource.resource_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        resource.description.toLowerCase().includes(searchQuery.toLowerCase());

      const matchesType =
        selectedType === "all" || resource.resource_type === selectedType;

      return matchesSearch && matchesType;
    });
  }

  $: {
    searchQuery;
    selectedType;
    selectedAvailability;
    applyFilters();
  }

  function handleResourceClick(resourceId: string) {
    window.location.href = `/booking-detail?id=${resourceId}`;
  }
</script>

<div class="space-y-4" data-testid="resource-list">
  {#if showFilters}
    <!-- Filters -->
    <div class="bg-white shadow rounded-lg p-4">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- Search -->
        <div>
          <label for="search" class="block text-sm font-medium text-gray-700 mb-1">
            {$_('common.search')}
          </label>
          <input
            type="text"
            id="search"
            bind:value={searchQuery}
            placeholder={$_('common.search')}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          />
        </div>

        <!-- Type Filter -->
        <div>
          <label for="type" class="block text-sm font-medium text-gray-700 mb-1">
            {$_('bookings.resource')}
          </label>
          <select
            id="type"
            bind:value={selectedType}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="all">{$_('common.all')}</option>
            {#each Object.values(ResourceType) as type}
              <option value={type}>{type}</option>
            {/each}
          </select>
        </div>

        <!-- Availability Filter -->
        <div>
          <label for="availability" class="block text-sm font-medium text-gray-700 mb-1">
            {$_('bookings.resources')}
          </label>
          <select
            id="availability"
            bind:value={selectedAvailability}
            on:change={loadResources}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="available-only">{$_('bookings.resources')}</option>
            <option value="all">{$_('common.all')}</option>
          </select>
        </div>
      </div>
    </div>
  {/if}

  <!-- Resources Grid -->
  {#if loading}
    <div class="text-center py-12 text-gray-500">{$_('bookings.loadError')}...</div>
  {:else if filteredResources.length === 0}
    <div class="bg-white shadow rounded-lg p-12 text-center">
      <p class="text-gray-500">{$_('bookings.noResources')}</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each filteredResources as resource}
        <ResourceCard {resource} onClick={() => handleResourceClick(resource.id)} />
      {/each}
    </div>

    <!-- Results count -->
    <p class="text-sm text-gray-600 text-center">
      {filteredResources.length} / {resources.length}
    </p>
  {/if}
</div>
