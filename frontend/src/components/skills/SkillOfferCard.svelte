<script lang="ts">
  import { type SkillOffer } from "../../lib/api/skills";
  import SkillCategoryBadge from "./SkillCategoryBadge.svelte";
  import ProficiencyBadge from "./ProficiencyBadge.svelte";

  export let offer: SkillOffer;
  export let onClick: (() => void) | undefined = undefined;
</script>

<div
  class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-lg transition-shadow duration-200 cursor-pointer"
  on:click={onClick}
  on:keydown={(e) => e.key === "Enter" && onClick?.()}
  role="button"
  tabindex="0"
>
  <div class="flex items-start justify-between mb-2">
    <div class="flex-1">
      <div class="flex items-center gap-2 mb-1">
        <SkillCategoryBadge category={offer.skill_category} />
        <ProficiencyBadge level={offer.expertise_level} />
      </div>
      <h3 class="text-lg font-semibold text-gray-900">{offer.skill_name}</h3>
      {#if offer.owner_name}
        <p class="text-sm text-gray-600">by {offer.owner_name}</p>
      {/if}
    </div>
    {#if offer.is_free}
      <div class="bg-blue-50 px-3 py-1 rounded-lg">
        <p class="text-xs font-medium text-blue-700">FREE</p>
      </div>
    {:else if offer.hourly_rate_credits}
      <div class="bg-green-50 px-3 py-1 rounded-lg">
        <p class="text-sm font-semibold text-green-700">
          {offer.hourly_rate_credits} credits/hr
        </p>
      </div>
    {/if}
  </div>

  <p class="text-sm text-gray-700 mb-3 line-clamp-2">{offer.description}</p>

  <div class="flex items-center justify-between text-xs text-gray-500">
    <div class="flex items-center gap-3">
      {#if offer.is_professional}
        <span class="text-purple-600 font-medium">Professional</span>
      {/if}
      {#if !offer.is_available_for_help}
        <span class="text-orange-600 font-medium">Unavailable</span>
      {/if}
    </div>
  </div>

  {#if offer.years_of_experience}
    <div class="mt-2 text-xs text-gray-600">
      {offer.years_of_experience} years experience
    </div>
  {/if}

  {#if offer.certifications}
    <div class="mt-2">
      <span class="inline-block px-2 py-0.5 text-xs bg-gray-100 text-gray-700 rounded">
        {offer.certifications}
      </span>
    </div>
  {/if}
</div>
