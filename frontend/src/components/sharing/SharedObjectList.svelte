<script lang="ts">
  // Svelte 5 runes mode
  import {
    sharingApi,
    type SharedObject,
    ObjectCategory,
  } from "../../lib/api/sharing";
  import SharedObjectCard from "./SharedObjectCard.svelte";
  import { withErrorHandling } from "../../lib/utils/error.utils";

  let { buildingId, showFilters = true }: {
    buildingId: string;
    showFilters?: boolean;
  } = $props();

  let objects = $state<SharedObject[]>([]);
  let filteredObjects = $state<SharedObject[]>([]);
  let loading = $state(true);
  let searchQuery = $state("");
  let selectedCategory = $state<ObjectCategory | "all">("all");
  let selectedAvailability = $state<"available-only" | "all">("available-only");

  $effect(() => {
    loadObjects();
  });

  async function loadObjects() {
    loading = true;
    const result = await withErrorHandling({
      action: () => selectedAvailability === "available-only"
        ? sharingApi.listAvailableObjects(buildingId)
        : sharingApi.listObjectsByBuilding(buildingId),
      errorMessage: "Failed to load shared objects",
    });
    if (result) {
      objects = result;
      applyFilters();
    }
    loading = false;
  }

  function applyFilters() {
    filteredObjects = objects.filter((obj) => {
      const matchesSearch =
        searchQuery === "" ||
        obj.object_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        obj.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        obj.owner_name?.toLowerCase().includes(searchQuery.toLowerCase());

      const matchesCategory =
        selectedCategory === "all" || obj.object_category === selectedCategory;

      return matchesSearch && matchesCategory;
    });
  }

  $effect(() => {
    searchQuery;
    selectedCategory;
    selectedAvailability;
    applyFilters();
  });

  function handleObjectClick(objectId: string) {
    window.location.href = `/sharing-detail?id=${objectId}`;
  }
</script>

<div class="space-y-4" data-testid="shared-object-list">
  {#if showFilters}
    <!-- Filters -->
    <div class="bg-white shadow rounded-lg p-4">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- Search -->
        <div>
          <label for="search" class="block text-sm font-medium text-gray-700 mb-1">
            Search
          </label>
          <input
            type="text"
            id="search"
            bind:value={searchQuery}
            placeholder="Search objects..."
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          />
        </div>

        <!-- Category Filter -->
        <div>
          <label for="category" class="block text-sm font-medium text-gray-700 mb-1">
            Category
          </label>
          <select
            id="category"
            bind:value={selectedCategory}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="all">All Categories</option>
            {#each Object.values(ObjectCategory) as category}
              <option value={category}>{category}</option>
            {/each}
          </select>
        </div>

        <!-- Availability Filter -->
        <div>
          <label for="availability" class="block text-sm font-medium text-gray-700 mb-1">
            Availability
          </label>
          <select
            id="availability"
            bind:value={selectedAvailability}
            onchange={loadObjects}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="available-only">Available Only</option>
            <option value="all">All Objects</option>
          </select>
        </div>
      </div>
    </div>
  {/if}

  <!-- Objects Grid -->
  {#if loading}
    <div class="text-center py-12 text-gray-500">Loading shared objects...</div>
  {:else if filteredObjects.length === 0}
    <div class="bg-white shadow rounded-lg p-12 text-center">
      <p class="text-gray-500">
        No shared objects found.
        {#if searchQuery || selectedCategory !== "all"}
          Try adjusting your filters.
        {/if}
      </p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each filteredObjects as object}
        <SharedObjectCard {object} onClick={() => handleObjectClick(object.id)} />
      {/each}
    </div>

    <!-- Results count -->
    <p class="text-sm text-gray-600 text-center">
      Showing {filteredObjects.length} of {objects.length} shared objects
    </p>
  {/if}
</div>
