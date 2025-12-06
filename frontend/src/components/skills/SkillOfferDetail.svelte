<script lang="ts">
  import { onMount } from "svelte";
  import {
    skillsApi,
    type SkillOffer,
    type SkillRequest,
    SkillRequestStatus,
  } from "../../lib/api/skills";
  import { toast } from "../../stores/toast";
  import SkillCategoryBadge from "./SkillCategoryBadge.svelte";
  import ProficiencyBadge from "./ProficiencyBadge.svelte";

  export let offerId: string;
  export let currentUserId: string;

  let offer: SkillOffer | null = null;
  let requests: SkillRequest[] = [];
  let loading = true;
  let showRequestModal = false;
  let requestMessage = "";
  let preferredDates: string[] = [""];
  let submitting = false;

  onMount(async () => {
    await loadOfferDetails();
  });

  async function loadOfferDetails() {
    try {
      loading = true;
      offer = await skillsApi.getOfferById(offerId);
      // Load requests if user is the owner
      if (offer.owner_id === currentUserId) {
        requests = await skillsApi.listRequestsByOffer(offerId);
      }
    } catch (err: any) {
      toast.error(err.message || "Failed to load skill offer");
    } finally {
      loading = false;
    }
  }

  async function handleCreateRequest() {
    if (!requestMessage.trim()) {
      toast.error("Please enter a message");
      return;
    }

    try {
      submitting = true;
      await skillsApi.createRequest({
        skill_offer_id: offerId,
        requester_id: currentUserId,
        message: requestMessage,
        preferred_dates: preferredDates.filter((d) => d.trim()),
      });
      toast.success("Request sent successfully");
      showRequestModal = false;
      requestMessage = "";
      preferredDates = [""];
    } catch (err: any) {
      toast.error(err.message || "Failed to send request");
    } finally {
      submitting = false;
    }
  }

  async function handleAcceptRequest(requestId: string) {
    try {
      await skillsApi.acceptRequest(requestId);
      toast.success("Request accepted");
      await loadOfferDetails();
    } catch (err: any) {
      toast.error(err.message || "Failed to accept request");
    }
  }

  async function handleDeclineRequest(requestId: string) {
    try {
      await skillsApi.declineRequest(requestId);
      toast.success("Request declined");
      await loadOfferDetails();
    } catch (err: any) {
      toast.error(err.message || "Failed to decline request");
    }
  }

  function addDateField() {
    preferredDates = [...preferredDates, ""];
  }

  function removeDateField(index: number) {
    preferredDates = preferredDates.filter((_, i) => i !== index);
  }

  function getStatusBadgeClass(status: SkillRequestStatus): string {
    switch (status) {
      case SkillRequestStatus.Pending:
        return "bg-yellow-100 text-yellow-800";
      case SkillRequestStatus.Accepted:
        return "bg-green-100 text-green-800";
      case SkillRequestStatus.Declined:
        return "bg-red-100 text-red-800";
      case SkillRequestStatus.Completed:
        return "bg-blue-100 text-blue-800";
      case SkillRequestStatus.Cancelled:
        return "bg-gray-100 text-gray-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  }

  $: isOwner = offer && offer.owner_id === currentUserId;
</script>

<div class="bg-white shadow rounded-lg overflow-hidden">
  {#if loading}
    <div class="text-center py-12 text-gray-500">Loading...</div>
  {:else if offer}
    <div class="p-6">
      <!-- Header -->
      <div class="flex items-start justify-between mb-4">
        <div class="flex-1">
          <div class="flex items-center gap-2 mb-2">
            <SkillCategoryBadge category={offer.skill_category} />
            <ProficiencyBadge level={offer.proficiency_level} />
          </div>
          <h1 class="text-2xl font-bold text-gray-900">{offer.skill_name}</h1>
          {#if offer.owner_name}
            <p class="text-gray-600 mt-1">Offered by {offer.owner_name}</p>
          {/if}
        </div>

        {#if offer.hourly_rate_credits}
          <div class="bg-green-50 px-4 py-2 rounded-lg">
            <p class="text-lg font-bold text-green-700">
              {offer.hourly_rate_credits} credits/hr
            </p>
          </div>
        {:else}
          <div class="bg-blue-50 px-4 py-2 rounded-lg">
            <p class="text-sm font-semibold text-blue-700">FREE</p>
          </div>
        {/if}
      </div>

      <!-- Stats -->
      <div class="grid grid-cols-3 gap-4 mb-6 bg-gray-50 rounded-lg p-4">
        <div class="text-center">
          <p class="text-2xl font-bold text-gray-900">
            {offer.rating ? offer.rating.toFixed(1) : "N/A"}
          </p>
          <p class="text-sm text-gray-600">⭐ Rating</p>
        </div>
        <div class="text-center">
          <p class="text-2xl font-bold text-gray-900">{offer.total_requests}</p>
          <p class="text-sm text-gray-600">Total Requests</p>
        </div>
        <div class="text-center">
          <p class="text-2xl font-bold text-gray-900">{offer.completed_requests}</p>
          <p class="text-sm text-gray-600">Completed</p>
        </div>
      </div>

      <!-- Description -->
      <div class="mb-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-2">Description</h3>
        <p class="text-gray-700 whitespace-pre-wrap">{offer.description}</p>
      </div>

      <!-- Experience & Certifications -->
      <div class="grid md:grid-cols-2 gap-6 mb-6">
        {#if offer.years_experience}
          <div>
            <h3 class="text-sm font-semibold text-gray-900 mb-2">📅 Experience</h3>
            <p class="text-gray-700">{offer.years_experience} years</p>
          </div>
        {/if}

        {#if offer.certifications && offer.certifications.length > 0}
          <div>
            <h3 class="text-sm font-semibold text-gray-900 mb-2">🏆 Certifications</h3>
            <ul class="list-disc list-inside text-gray-700">
              {#each offer.certifications as cert}
                <li>{cert}</li>
              {/each}
            </ul>
          </div>
        {/if}
      </div>

      <!-- Availability -->
      <div class="mb-6">
        <h3 class="text-sm font-semibold text-gray-900 mb-2">📅 Availability</h3>
        <p class="text-gray-700">{offer.availability}</p>
      </div>

      <!-- Request Button (if not owner) -->
      {#if !isOwner && offer.status === "Available"}
        <button
          on:click={() => (showRequestModal = true)}
          class="w-full bg-blue-600 text-white px-6 py-3 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
        >
          Request This Skill
        </button>
      {/if}

      <!-- Requests (if owner) -->
      {#if isOwner && requests.length > 0}
        <div class="mt-8">
          <h3 class="text-lg font-semibold text-gray-900 mb-4">Skill Requests</h3>
          <div class="space-y-4">
            {#each requests as request}
              <div class="border border-gray-200 rounded-lg p-4">
                <div class="flex items-start justify-between mb-2">
                  <div>
                    <p class="font-semibold text-gray-900">
                      {request.requester_name || "Unknown"}
                    </p>
                    <p class="text-sm text-gray-600">
                      {new Date(request.created_at).toLocaleDateString()}
                    </p>
                  </div>
                  <span
                    class="px-3 py-1 rounded-full text-xs font-medium {getStatusBadgeClass(request.status)}"
                  >
                    {request.status}
                  </span>
                </div>
                <p class="text-gray-700 mb-2">{request.message}</p>
                {#if request.preferred_dates && request.preferred_dates.length > 0}
                  <p class="text-sm text-gray-600">
                    Preferred dates: {request.preferred_dates.join(", ")}
                  </p>
                {/if}
                {#if request.status === SkillRequestStatus.Pending}
                  <div class="flex gap-2 mt-3">
                    <button
                      on:click={() => handleAcceptRequest(request.id)}
                      class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 text-sm"
                    >
                      Accept
                    </button>
                    <button
                      on:click={() => handleDeclineRequest(request.id)}
                      class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 text-sm"
                    >
                      Decline
                    </button>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <div class="text-center py-12 text-gray-500">Skill offer not found</div>
  {/if}
</div>

<!-- Request Modal -->
{#if showRequestModal}
  <div
    class="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50 p-4"
  >
    <div class="bg-white rounded-lg shadow-xl max-w-lg w-full p-6">
      <h2 class="text-xl font-bold text-gray-900 mb-4">Request Skill</h2>

      <form on:submit|preventDefault={handleCreateRequest} class="space-y-4">
        <div>
          <label for="message" class="block text-sm font-medium text-gray-700 mb-1">
            Message <span class="text-red-500">*</span>
          </label>
          <textarea
            id="message"
            bind:value={requestMessage}
            required
            rows="4"
            placeholder="Describe what you need help with..."
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Preferred Dates (optional)
          </label>
          {#each preferredDates as date, index}
            <div class="flex gap-2 mb-2">
              <input
                type="text"
                bind:value={preferredDates[index]}
                placeholder="e.g., Monday 10am"
                class="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
              />
              {#if preferredDates.length > 1}
                <button
                  type="button"
                  on:click={() => removeDateField(index)}
                  class="px-3 py-2 text-red-600 hover:text-red-800"
                >
                  Remove
                </button>
              {/if}
            </div>
          {/each}
          <button
            type="button"
            on:click={addDateField}
            class="text-sm text-blue-600 hover:text-blue-800"
          >
            + Add another date
          </button>
        </div>

        <div class="flex gap-3">
          <button
            type="submit"
            disabled={submitting}
            class="flex-1 bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {submitting ? "Sending..." : "Send Request"}
          </button>
          <button
            type="button"
            on:click={() => (showRequestModal = false)}
            disabled={submitting}
            class="flex-1 bg-gray-200 text-gray-700 px-4 py-2 rounded-md hover:bg-gray-300"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
