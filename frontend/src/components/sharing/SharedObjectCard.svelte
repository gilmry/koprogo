<script lang="ts">
  import { type SharedObject } from "../../lib/api/sharing";
  import ObjectCategoryBadge from "./ObjectCategoryBadge.svelte";
  import ObjectConditionBadge from "./ObjectConditionBadge.svelte";
  import AvailabilityStatusBadge from "./AvailabilityStatusBadge.svelte";

  export let object: SharedObject;
  export let onClick: (() => void) | undefined = undefined;

  function formatDeposit(cents?: number): string {
    if (!cents) return "No deposit";
    return `€${(cents / 100).toFixed(2)} deposit`;
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
  {#if object.image_urls && object.image_urls.length > 0}
    <img
      src={object.image_urls[0]}
      alt={object.object_name}
      class="w-full h-48 object-cover"
    />
  {:else}
    <div class="w-full h-48 bg-gray-200 flex items-center justify-center">
      <span class="text-4xl text-gray-400">📦</span>
    </div>
  {/if}

  <div class="p-4">
    <div class="flex items-start justify-between mb-2">
      <div class="flex-1">
        <div class="flex items-center gap-2 mb-1">
          <ObjectCategoryBadge category={object.object_category} />
          <ObjectConditionBadge condition={object.condition} />
        </div>
        <h3 class="text-lg font-semibold text-gray-900">{object.object_name}</h3>
        {#if object.owner_name}
          <p class="text-sm text-gray-600">by {object.owner_name}</p>
        {/if}
      </div>
      <AvailabilityStatusBadge status={object.availability_status} />
    </div>

    <p class="text-sm text-gray-700 mb-3 line-clamp-2">{object.description}</p>

    <div class="flex items-center justify-between text-xs text-gray-600">
      <div class="space-y-1">
        <div>📅 {object.loan_duration_days} days max</div>
        <div>{formatDeposit(object.deposit_required_cents)}</div>
      </div>
      <div class="text-right space-y-1">
        <div>{formatRating(object.rating)}</div>
        <div>{object.total_loans} loans</div>
      </div>
    </div>
  </div>
</div>
