<script lang="ts">
  import { _ } from "svelte-i18n";
  import {
    energyCampaignsApi,
    type CreateCampaignDto,
    EnergyType,
  } from "../../lib/api/energy-campaigns";
  import BuildingSelector from "../BuildingSelector.svelte";

  export let organizationId: string;
  export let onCreated: ((campaign: any) => void) | undefined = undefined;
  export let onCancel: (() => void) | undefined = undefined;

  let selectedBuildingId = "";

  // Default deadline: 3 months from now
  const defaultDeadline = new Date();
  defaultDeadline.setMonth(defaultDeadline.getMonth() + 3);

  let formData: CreateCampaignDto = {
    building_id: undefined,
    campaign_name: "",
    deadline_participation: defaultDeadline.toISOString().split("T")[0],
    energy_types: [],
  };

  $: formData.building_id = selectedBuildingId || undefined;

  let loading = false;
  let error = "";
  let success = false;

  function toggleEnergyType(type: EnergyType) {
    if (formData.energy_types.includes(type)) {
      formData.energy_types = formData.energy_types.filter((t) => t !== type);
    } else {
      formData.energy_types = [...formData.energy_types, type];
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    loading = true;
    error = "";
    success = false;

    try {
      if (!formData.campaign_name.trim()) {
        throw new Error($_("energy.campaign.nameRequired"));
      }
      if (formData.energy_types.length === 0) {
        throw new Error($_("energy.campaign.typeRequired"));
      }
      if (!formData.deadline_participation) {
        throw new Error($_("energy.campaign.deadlineRequired"));
      }
      const today = new Date().toISOString().split("T")[0];
      if (formData.deadline_participation <= today) {
        throw new Error($_("energy.campaign.deadlineMustBeFuture"));
      }

      // Send as ISO datetime for backend DateTime<Utc>
      const payload = {
        ...formData,
        deadline_participation: new Date(formData.deadline_participation).toISOString(),
      };

      const campaign = await energyCampaignsApi.create(payload as any);
      success = true;
      if (onCreated) onCreated(campaign);

      setTimeout(() => {
        formData = {
          building_id: selectedBuildingId || undefined,
          campaign_name: "",
          deadline_participation: defaultDeadline.toISOString().split("T")[0],
          energy_types: [],
        };
        success = false;
      }, 2000);
    } catch (err: any) {
      error = err.message || $_("energy.campaign.createError");
      console.error("Failed to create campaign:", err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="bg-white shadow-md rounded-lg p-6">
  <h3 class="text-lg font-medium text-gray-900 mb-4">
    ➕ {$_("energy.campaign.create")}
  </h3>

  <p class="text-sm text-gray-600 mb-6">
    {$_("energy.campaign.description")}
    <strong>{$_("energy.campaign.savingsExpected")}</strong>
  </p>

  {#if success}
    <div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-md">
      <p class="text-sm text-green-800">
        ✅ {$_("energy.campaign.successRedirect")}
      </p>
    </div>
  {/if}

  {#if error}
    <div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">❌ {error}</p>
    </div>
  {/if}

  <form on:submit={handleSubmit} class="space-y-6">
    <!-- Building Selector -->
    <BuildingSelector bind:selectedBuildingId label={$_("energy.campaign.building")} required={false} />

    <!-- Campaign Name -->
    <div>
      <label
        for="campaign_name"
        class="block text-sm font-medium text-gray-700"
      >
        {$_("energy.campaign.name")} <span class="text-red-500">*</span>
      </label>
      <input
        type="text"
        id="campaign_name"
        bind:value={formData.campaign_name}
        required
        placeholder={$_("energy.campaign.nameExample")}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
      <p class="mt-1 text-xs text-gray-500">
        {$_("energy.campaign.nameHelp")}
      </p>
    </div>

    <!-- Energy Types -->
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-2">
        {$_("energy.campaign.energyTypes")} <span class="text-red-500">*</span>
      </label>
      <div class="space-y-2">
        <label class="flex items-center">
          <input
            type="checkbox"
            checked={formData.energy_types.includes(EnergyType.Electricity)}
            on:change={() => toggleEnergyType(EnergyType.Electricity)}
            class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <span class="ml-2 text-sm text-gray-700">
            ⚡ {$_("energy.campaign.electricity")}
          </span>
        </label>
        <label class="flex items-center">
          <input
            type="checkbox"
            checked={formData.energy_types.includes(EnergyType.Gas)}
            on:change={() => toggleEnergyType(EnergyType.Gas)}
            class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <span class="ml-2 text-sm text-gray-700">
            🔥 {$_("energy.campaign.gas")}
          </span>
        </label>
        <label class="flex items-center">
          <input
            type="checkbox"
            checked={formData.energy_types.includes(EnergyType.Heating)}
            on:change={() => toggleEnergyType(EnergyType.Heating)}
            class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <span class="ml-2 text-sm text-gray-700">
            🌡️ {$_("energy.campaign.heating")}
          </span>
        </label>
      </div>
      <p class="mt-2 text-xs text-gray-500">
        {$_("energy.campaign.energyTypesHelp")}
      </p>
    </div>

    <!-- Deadline -->
    <div>
      <label
        for="deadline_participation"
        class="block text-sm font-medium text-gray-700"
      >
        {$_("energy.campaign.deadline")} <span class="text-red-500">*</span>
      </label>
      <input
        type="date"
        id="deadline_participation"
        bind:value={formData.deadline_participation}
        required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
      <p class="mt-1 text-xs text-gray-500">
        {$_("energy.campaign.deadlineHelp")}
      </p>
    </div>

    <!-- GDPR Notice -->
    <div class="p-4 bg-blue-50 border border-blue-200 rounded-md">
      <h4 class="text-sm font-medium text-blue-900 mb-2">
        🔒 {$_("energy.campaign.gdprTitle")}
      </h4>
      <ul class="text-xs text-blue-800 space-y-1">
        <li>
          ✅ {$_("energy.campaign.gdprPoint1")}
        </li>
        <li>
          ✅ {$_("energy.campaign.gdprPoint2")}
        </li>
        <li>
          ✅ {$_("energy.campaign.gdprPoint3")}
        </li>
        <li>✅ {$_("energy.campaign.gdprPoint4")}}</li>
        <li>
          ✅ {$_("energy.campaign.gdprPoint5")}
        </li>
      </ul>
    </div>

    <!-- Submit Button -->
    <div class="flex justify-end space-x-3">
      <button
        type="button"
        on:click={() => onCancel && onCancel()}
        class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
      >
        {$_("common.cancel")}
      </button>
      <button
        type="submit"
        disabled={loading}
        class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if loading}
          <span class="inline-block animate-spin mr-2">⏳</span>
          {$_("energy.campaign.creating")}
        {:else}
          ✅ {$_("energy.campaign.create")}
        {/if}
      </button>
    </div>
  </form>
</div>
