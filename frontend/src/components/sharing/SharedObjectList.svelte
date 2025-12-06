<script lang="ts">
  import { onMount } from "svelte";
  import {
    sharingApi,
    type SharedObject,
    ObjectCategory,
    AvailabilityStatus,
  } from "../../lib/api/sharing";
  import { toast } from "../../stores/toast";
  import SharedObjectCard from "./SharedObjectCard.svelte";

  export let buildingId: string;
  export let showFilters = true;

  let objects: SharedObject[] = [];
  let filteredObjects: SharedObject[] = [];
  let loading = true;
  let searchQuery = "";
  let selectedCategory: ObjectCategory | "all" = "all";
  let selectedAvailability: "available-only" | "all" = "available-only";

  onMount(async () => {
    await loadObjects();
  });

  async function loadObjects() {
    try {
      loading = true;
      if (selectedAvailability === "available-only") {
        objects = await sharingApi.listAvailableObjects(buildingId);
      } else {
        objects = await sharingApi.listObjectsByBuilding(buildingId);
      }
      applyFilters();
    } catch (err: any) {
      toast.error(err.message || "Failed to load shared objects");
    } finally {
      loading = false;
    }
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

  $: {
    searchQuery;
    selectedCategory;
    selectedAvailability;
    applyFilters();
  }

  function handleObjectClick(objectId: string) {
    window.location.href = `/sharing-detail?id=${objectId}`;
  }
</script>

<div class="space-y-4">
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
            on:change={loadObjects}
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
