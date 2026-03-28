<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    skillsApi,
    type SkillOffer,
  } from "../../lib/api/skills";
  import { toast } from "../../stores/toast";
  import SkillCategoryBadge from "./SkillCategoryBadge.svelte";
  import ProficiencyBadge from "./ProficiencyBadge.svelte";
  import { withErrorHandling } from "../../lib/utils/error.utils";

  export let offerId: string;
  export let currentUserId: string;

  let offer: SkillOffer | null = null;
  let loading = true;

  onMount(async () => {
    await loadOfferDetails();
  });

  async function loadOfferDetails() {
    loading = true;
    const result = await withErrorHandling({
      action: () => skillsApi.getOfferById(offerId),
      errorMessage: $_("skills.detail.loadingError"),
    });
    if (result) offer = result;
    loading = false;
  }

  $: isOwner = offer && offer.owner_id === currentUserId;
</script>

<div class="bg-white shadow rounded-lg overflow-hidden" data-testid="skill-offer-detail">
  {#if loading}
    <div class="text-center py-12 text-gray-500">{$_("common.loading")}</div>
  {:else if offer}
    <div class="p-6">
      <!-- Header -->
      <div class="flex items-start justify-between mb-4">
        <div class="flex-1">
          <div class="flex items-center gap-2 mb-2">
            <SkillCategoryBadge category={offer.skill_category} />
            <ProficiencyBadge level={offer.expertise_level} />
          </div>
          <h1 class="text-2xl font-bold text-gray-900">{offer.skill_name}</h1>
          {#if offer.owner_name}
            <p class="text-gray-600 mt-1">{$_("skills.detail.offeredBy")} {offer.owner_name}</p>
          {/if}
        </div>

        {#if offer.is_free}
          <div class="bg-blue-50 px-4 py-2 rounded-lg">
            <p class="text-sm font-semibold text-blue-700">{$_("skills.detail.free")}</p>
          </div>
        {:else if offer.hourly_rate_credits}
          <div class="bg-green-50 px-4 py-2 rounded-lg">
            <p class="text-lg font-bold text-green-700">
              {offer.hourly_rate_credits} {$_("skills.detail.creditsPerHour")}
            </p>
          </div>
        {/if}
      </div>

      <!-- Info -->
      <div class="grid grid-cols-2 gap-4 mb-6 bg-gray-50 rounded-lg p-4">
        <div class="text-center">
          <p class="text-sm font-medium text-gray-900">
            {offer.is_available_for_help ? $_("skills.detail.available") : $_("skills.detail.unavailable")}
          </p>
          <p class="text-sm text-gray-600">{$_("common.status")}</p>
        </div>
        <div class="text-center">
          <p class="text-sm font-medium text-gray-900">
            {offer.is_professional ? $_("skills.detail.professional") : $_("skills.detail.community")}
          </p>
          <p class="text-sm text-gray-600">{$_("common.type")}</p>
        </div>
      </div>

      <!-- Description -->
      <div class="mb-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-2">{$_("common.description")}</h3>
        <p class="text-gray-700 whitespace-pre-wrap">{offer.description}</p>
      </div>

      <!-- Experience & Certifications -->
      <div class="grid md:grid-cols-2 gap-6 mb-6">
        {#if offer.years_of_experience}
          <div>
            <h3 class="text-sm font-semibold text-gray-900 mb-2">{$_("skills.detail.experience")}</h3>
            <p class="text-gray-700">{offer.years_of_experience} {$_("skills.detail.years")}</p>
          </div>
        {/if}

        {#if offer.certifications}
          <div>
            <h3 class="text-sm font-semibold text-gray-900 mb-2">{$_("skills.detail.certifications")}</h3>
            <p class="text-gray-700">{offer.certifications}</p>
          </div>
        {/if}
      </div>

      <!-- Actions -->
      {#if isOwner}
        <div class="flex gap-2">
          {#if offer.is_available_for_help}
            <button
              on:click={async () => { if (offer) { await skillsApi.markUnavailable(offer.id); await loadOfferDetails(); } }}
              class="px-4 py-2 bg-orange-600 text-white rounded hover:bg-orange-700 text-sm"
            >
              {$_("skills.detail.markUnavailable")}
            </button>
          {:else}
            <button
              on:click={async () => { if (offer) { await skillsApi.markAvailable(offer.id); await loadOfferDetails(); } }}
              class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 text-sm"
            >
              {$_("skills.detail.markAvailable")}
            </button>
          {/if}
        </div>
      {/if}
    </div>
  {:else}
    <div class="text-center py-12 text-gray-500">{$_("skills.detail.notFound")}</div>
  {/if}
</div>
