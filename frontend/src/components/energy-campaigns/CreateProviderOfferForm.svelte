<script lang="ts">
  import { _ } from "svelte-i18n";
  import { createEventDispatcher } from "svelte";
  import {
    energyCampaignsApi,
    type CreateProviderOfferDto,
  } from "../../lib/api/energy-campaigns";

  export let campaignId: string;

  const dispatch = createEventDispatcher();

  let formData: CreateProviderOfferDto = {
    provider_name: "",
    price_kwh_electricity: undefined,
    price_kwh_gas: undefined,
    fixed_monthly_fee: 0,
    green_energy_pct: 0,
    contract_duration_months: 12,
    estimated_savings_pct: 0,
    offer_valid_until: "",
  };

  let loading = false;
  let error = "";
  let success = false;

  function setDefaultValidityDate() {
    const date = new Date();
    date.setMonth(date.getMonth() + 1); // Valid for 1 month by default
    formData.offer_valid_until = date.toISOString().split("T")[0];
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    loading = true;
    error = "";
    success = false;

    try {
      // Validate
      if (!formData.provider_name.trim()) {
        throw new Error($_("energy.offer.providerNameRequired"));
      }
      if (!formData.price_kwh_electricity && !formData.price_kwh_gas) {
        throw new Error($_("energy.offer.priceRequired"));
      }
      if (formData.contract_duration_months <= 0) {
        throw new Error($_("energy.offer.durationRequired"));
      }
      if (!formData.offer_valid_until) {
        throw new Error($_("energy.offer.validityRequired"));
      }
      if (formData.green_energy_pct < 0 || formData.green_energy_pct > 100) {
        throw new Error($_("energy.offer.greenPercentageInvalid"));
      }
      if (formData.estimated_savings_pct < 0 || formData.estimated_savings_pct > 100) {
        throw new Error($_("energy.offer.savingsPercentageInvalid"));
      }

      // Send date as ISO datetime for backend DateTime<Utc>
      const payload = {
        ...formData,
        offer_valid_until: new Date(formData.offer_valid_until).toISOString(),
      };

      const offer = await energyCampaignsApi.addOffer(campaignId, payload as any);
      success = true;
      dispatch("created", offer);

      // Reset form after 2 seconds
      setTimeout(() => {
        formData = {
          provider_name: "",
          price_kwh_electricity: undefined,
          price_kwh_gas: undefined,
          fixed_monthly_fee: 0,
          green_energy_pct: 0,
          contract_duration_months: 12,
          estimated_savings_pct: 0,
          offer_valid_until: "",
        };
        success = false;
      }, 2000);
    } catch (err: any) {
      error = err.message || $_("energy.offer.createError");
      console.error("Failed to create provider offer:", err);
    } finally {
      loading = false;
    }
  }

  // Initialize default validity date
  setDefaultValidityDate();
</script>

<div class="bg-white shadow-md rounded-lg p-6">
  <h3 class="text-lg font-medium text-gray-900 mb-4">
    💼 {$_("energy.offer.add")}
  </h3>

  <p class="text-sm text-gray-600 mb-6">
    {$_("energy.offer.description")}
  </p>

  {#if success}
    <div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-md">
      <p class="text-sm text-green-800">✅ {$_("energy.offer.successAdd")}</p>
    </div>
  {/if}

  {#if error}
    <div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">❌ {error}</p>
    </div>
  {/if}

  <form on:submit={handleSubmit} class="space-y-6">
    <!-- Provider Name -->
    <div>
      <label
        for="provider_name"
        class="block text-sm font-medium text-gray-700"
      >
        {$_("energy.offer.providerName")} <span class="text-red-500">*</span>
      </label>
      <input
        type="text"
        id="provider_name"
        bind:value={formData.provider_name}
        required
        placeholder={$_("energy.offer.providerExample")}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
    </div>

    <!-- Pricing -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label
          for="price_kwh_electricity"
          class="block text-sm font-medium text-gray-700"
        >
          {$_("energy.offer.priceElectricity")}
        </label>
        <input
          type="number"
          id="price_kwh_electricity"
          bind:value={formData.price_kwh_electricity}
          min="0"
          step="0.0001"
          placeholder="0.1234"
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        />
        <p class="mt-1 text-xs text-gray-500">
          {$_("energy.offer.priceHelp")}
        </p>
      </div>
      <div>
        <label
          for="price_kwh_gas"
          class="block text-sm font-medium text-gray-700"
        >
          {$_("energy.offer.priceGas")}
        </label>
        <input
          type="number"
          id="price_kwh_gas"
          bind:value={formData.price_kwh_gas}
          min="0"
          step="0.0001"
          placeholder="0.0567"
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        />
        <p class="mt-1 text-xs text-gray-500">
          {$_("energy.offer.priceHelp")}
        </p>
      </div>
    </div>

    <!-- Fixed Fee -->
    <div>
      <label
        for="fixed_fee"
        class="block text-sm font-medium text-gray-700"
      >
        {$_("energy.offer.fixedFee")} <span class="text-red-500">*</span>
      </label>
      <input
        type="number"
        id="fixed_fee"
        bind:value={formData.fixed_monthly_fee}
        required
        min="0"
        step="0.01"
        placeholder="15.00"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
    </div>

    <!-- Contract Duration -->
    <div>
      <label
        for="contract_duration"
        class="block text-sm font-medium text-gray-700"
      >
        {$_("energy.offer.contractDuration")} <span class="text-red-500">*</span>
      </label>
      <input
        type="number"
        id="contract_duration"
        bind:value={formData.contract_duration_months}
        required
        min="1"
        max="60"
        placeholder="12"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
      <p class="mt-1 text-xs text-gray-500">{$_("energy.offer.contractDurationHelp")}</p>
    </div>

    <!-- Green Energy Percentage -->
    <div>
      <label
        for="green_percentage"
        class="block text-sm font-medium text-gray-700"
      >
        {$_("energy.offer.greenPercentage")} <span class="text-red-500">*</span>
      </label>
      <div class="flex items-center space-x-4">
        <input
          type="range"
          id="green_percentage"
          bind:value={formData.green_energy_pct}
          min="0"
          max="100"
          step="1"
          class="flex-1"
        />
        <span class="text-sm font-medium text-gray-900 w-12">
          {formData.green_energy_pct}%
        </span>
      </div>
      <p class="mt-1 text-xs text-gray-500">
        {$_("energy.offer.greenPercentageHelp")}
      </p>
    </div>

    <!-- Estimated Savings -->
    <div>
      <label for="savings" class="block text-sm font-medium text-gray-700">
        {$_("energy.offer.estimatedSavings")} <span class="text-red-500">*</span>
      </label>
      <input
        type="number"
        id="savings"
        bind:value={formData.estimated_savings_pct}
        required
        min="0"
        max="100"
        step="0.1"
        placeholder="15"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
      <p class="mt-1 text-xs text-gray-500">
        {$_("energy.offer.estimatedSavingsHelp")}
      </p>
    </div>

    <!-- Validity Date -->
    <div>
      <label
        for="valid_until"
        class="block text-sm font-medium text-gray-700"
      >
        {$_("energy.offer.validityDate")} <span class="text-red-500">*</span>
      </label>
      <input
        type="date"
        id="valid_until"
        bind:value={formData.offer_valid_until}
        required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
    </div>

    <!-- Submit Button -->
    <div class="flex justify-end space-x-3">
      <button
        type="button"
        on:click={() => dispatch("cancel")}
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
          {$_("energy.offer.adding")}
        {:else}
          ✅ {$_("energy.offer.add")}
        {/if}
      </button>
    </div>
  </form>
</div>
