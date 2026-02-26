<script lang="ts">
  import { onMount } from "svelte";
  import {
    skillsApi,
    type SkillOffer,
    SkillCategory,
    ExpertiseLevel,
  } from "../../lib/api/skills";
  import { toast } from "../../stores/toast";
  import SkillOfferCard from "./SkillOfferCard.svelte";

  export let buildingId: string;
  export let showFilters = true;

  let offers: SkillOffer[] = [];
  let filteredOffers: SkillOffer[] = [];
  let loading = true;
  let searchQuery = "";
  let selectedCategory: SkillCategory | "all" = "all";
  let selectedExpertise: ExpertiseLevel | "all" = "all";

  onMount(async () => {
    await loadOffers();
  });

  async function loadOffers() {
    try {
      loading = true;
      offers = await skillsApi.listAvailableOffers(buildingId);
      applyFilters();
    } catch (err: any) {
      toast.error(err.message || "Failed to load skill offers");
    } finally {
      loading = false;
    }
  }

  function applyFilters() {
    filteredOffers = offers.filter((offer) => {
      const matchesSearch =
        searchQuery === "" ||
        offer.skill_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        offer.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        offer.owner_name?.toLowerCase().includes(searchQuery.toLowerCase());

      const matchesCategory =
        selectedCategory === "all" || offer.skill_category === selectedCategory;

      const matchesExpertise =
        selectedExpertise === "all" || offer.expertise_level === selectedExpertise;

      return matchesSearch && matchesCategory && matchesExpertise;
    });
  }

  $: {
    searchQuery;
    selectedCategory;
    selectedExpertise;
    applyFilters();
  }

  function handleOfferClick(offerId: string) {
    window.location.href = `/skill-detail?id=${offerId}`;
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
            placeholder="Search skills..."
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
            {#each Object.values(SkillCategory) as category}
              <option value={category}>{category}</option>
            {/each}
          </select>
        </div>

        <!-- Expertise Filter -->
        <div>
          <label for="expertise" class="block text-sm font-medium text-gray-700 mb-1">
            Expertise
          </label>
          <select
            id="expertise"
            bind:value={selectedExpertise}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="all">All Levels</option>
            {#each Object.values(ExpertiseLevel) as level}
              <option value={level}>{level}</option>
            {/each}
          </select>
        </div>
      </div>
    </div>
  {/if}

  <!-- Offers Grid -->
  {#if loading}
    <div class="text-center py-12 text-gray-500">Loading skill offers...</div>
  {:else if filteredOffers.length === 0}
    <div class="bg-white shadow rounded-lg p-12 text-center">
      <p class="text-gray-500">
        No skill offers found.
        {#if searchQuery || selectedCategory !== "all" || selectedExpertise !== "all"}
          Try adjusting your filters.
        {/if}
      </p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each filteredOffers as offer}
        <SkillOfferCard {offer} onClick={() => handleOfferClick(offer.id)} />
      {/each}
    </div>

    <!-- Results count -->
    <p class="text-sm text-gray-600 text-center">
      Showing {filteredOffers.length} of {offers.length} skill offers
    </p>
  {/if}
</div>
