<script lang="ts">
  import { type BookableResource } from "../../lib/api/bookings";
  import ResourceTypeBadge from "./ResourceTypeBadge.svelte";

  export let resource: BookableResource;
  export let onClick: (() => void) | undefined = undefined;

  function formatCost(credits?: number): string {
    if (!credits) return "FREE";
    return `${credits} credits/hr`;
  }

  function formatRating(rating?: number): string {
    if (!rating) return "No ratings yet";
    return `⭐ ${rating.toFixed(1)}/5.0`;
  }
</script>

<div
  class="bg-white border border-gray-200 rounded-lg overflow-hidden hover:shadow-lg transition-shadow duration-200 cursor-pointer"
  on:click={onClick}
  on:keydown={(e) => e.key === "Enter" && onClick?.()}
  role="button"
  tabindex="0"
>
  {#if resource.image_urls && resource.image_urls.length > 0}
    <img
      src={resource.image_urls[0]}
      alt={resource.resource_name}
      class="w-full h-48 object-cover"
    />
  {:else}
    <div class="w-full h-48 bg-gradient-to-br from-blue-100 to-blue-200 flex items-center justify-center">
      <span class="text-5xl">🏢</span>
    </div>
  {/if}

  <div class="p-4">
    <div class="flex items-start justify-between mb-2">
      <div class="flex-1">
        <ResourceTypeBadge type={resource.resource_type} />
        <h3 class="text-lg font-semibold text-gray-900 mt-2">{resource.resource_name}</h3>
      </div>
      {#if resource.status === "Available"}
        <span class="bg-green-100 text-green-800 px-2 py-1 rounded text-xs font-medium">
          ✅ Available
        </span>
      {:else}
        <span class="bg-red-100 text-red-800 px-2 py-1 rounded text-xs font-medium">
          ❌ {resource.status}
        </span>
      {/if}
    </div>

    <p class="text-sm text-gray-700 mb-3 line-clamp-2">{resource.description}</p>

    <div class="flex items-center justify-between text-xs text-gray-600 mb-3">
      <div class="space-y-1">
        {#if resource.capacity}
          <div>👥 Capacity: {resource.capacity}</div>
        {/if}
        <div>⏱️ Max {resource.max_booking_duration_hours}h</div>
      </div>
      <div class="text-right space-y-1">
        <div class="font-semibold text-blue-600">{formatCost(resource.hourly_rate_credits)}</div>
        <div>{formatRating(resource.rating)}</div>
      </div>
    </div>

    {#if resource.amenities && resource.amenities.length > 0}
      <div class="flex flex-wrap gap-1">
        {#each resource.amenities.slice(0, 3) as amenity}
          <span class="inline-block px-2 py-0.5 text-xs bg-gray-100 text-gray-700 rounded">
            {amenity}
          </span>
        {/each}
        {#if resource.amenities.length > 3}
          <span class="text-xs text-gray-500">+{resource.amenities.length - 3} more</span>
        {/if}
      </div>
    {/if}

    {#if resource.requires_approval}
      <div class="mt-2 text-xs text-yellow-700 bg-yellow-50 px-2 py-1 rounded">
        ⚠️ Requires approval
      </div>
    {/if}
  </div>
</div>
