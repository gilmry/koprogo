<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    localExchangesApi,
    type CreateLocalExchangeDto,
    ExchangeType,
    exchangeTypeLabels,
    exchangeTypeIcons,
  } from "../../lib/api/local-exchanges";
  import BuildingSelector from "../BuildingSelector.svelte";
  import { withErrorHandling } from "../../lib/utils/error.utils";

  const dispatch = createEventDispatcher();

  let selectedBuildingId = "";

  let formData: CreateLocalExchangeDto = {
    building_id: "",
    exchange_type: ExchangeType.Service,
    title: "",
    description: "",
    credits: 1,
  };

  $: formData.building_id = selectedBuildingId;

  let loading: boolean = false;
  let error: string | null = null;
  let success: boolean = false;

  async function handleSubmit(e: Event) {
    e.preventDefault();

    // Validation
    if (!formData.building_id) {
      error = $_("exchanges.selectBuilding");
      return;
    }
    if (!formData.title.trim()) {
      error = $_("exchanges.titleRequired");
      return;
    }

    if (!formData.description.trim()) {
      error = $_("exchanges.descriptionRequired");
      return;
    }

    if (formData.credits < 1 || formData.credits > 100) {
      error = $_("exchanges.creditsRange");
      return;
    }

    const result = await withErrorHandling({
      action: () => localExchangesApi.create(formData),
      setLoading: (v) => loading = v,
      errorMessage: $_("exchanges.createError"),
      onSuccess: (exchange) => {
        error = null;
        success = true;
        dispatch("success", exchange);
        setTimeout(() => {
          window.location.href = `/exchange-detail?id=${exchange.id}`;
        }, 1500);
      },
    });
  }
</script>

<form on:submit={handleSubmit} class="space-y-6" data-testid="create-exchange-form">
  <!-- Success Message -->
  {#if success}
    <div
      class="bg-green-50 border border-green-200 rounded-md p-4 flex items-center"
    >
      <svg
        class="h-5 w-5 text-green-400 mr-2"
        fill="currentColor"
        viewBox="0 0 20 20"
      >
        <path
          fill-rule="evenodd"
          d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
          clip-rule="evenodd"
        />
      </svg>
      <p class="text-green-800">
        ✅ {$_("exchanges.successCreate")}
      </p>
    </div>
  {/if}

  <!-- Error Message -->
  {#if error}
    <div class="bg-red-50 border border-red-200 rounded-md p-4">
      <p class="text-red-800">❌ {error}</p>
    </div>
  {/if}

  <!-- Building Selector -->
  <BuildingSelector bind:selectedBuildingId label={$_("exchanges.building")} />

  <!-- Exchange Type -->
  <div>
    <label for="exchange-type" class="block text-sm font-medium text-gray-700 mb-2">
      {$_("exchanges.exchangeType")} <span class="text-red-500">*</span>
    </label>
    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
      {#each Object.values(ExchangeType) as type}
        <label
          class="relative flex items-center p-4 border-2 rounded-lg cursor-pointer hover:bg-gray-50 {formData.exchange_type === type ? 'border-blue-500 bg-blue-50' : 'border-gray-200'}"
        >
          <input
            type="radio"
            name="exchange-type"
            value={type}
            bind:group={formData.exchange_type}
            class="sr-only"
          />
          <span class="text-2xl mr-3">{exchangeTypeIcons[type]}</span>
          <div>
            <p class="font-medium text-gray-900">{exchangeTypeLabels[type]}</p>
            <p class="text-xs text-gray-500">
              {#if type === ExchangeType.Service}
                {$_("exchanges.serviceDescription")}
              {:else if type === ExchangeType.ObjectLoan}
                {$_("exchanges.loanDescription")}
              {:else if type === ExchangeType.SharedPurchase}
                {$_("exchanges.purchaseDescription")}
              {/if}
            </p>
          </div>
        </label>
      {/each}
    </div>
  </div>

  <!-- Title -->
  <div>
    <label for="title" class="block text-sm font-medium text-gray-700 mb-1">
      {$_("exchanges.title")} <span class="text-red-500">*</span>
    </label>
    <input
      id="title"
      type="text"
      bind:value={formData.title}
      placeholder={$_("exchanges.titlePlaceholder")}
      maxlength="100"
      required
      data-testid="exchange-title-input"
      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
    />
    <p class="mt-1 text-xs text-gray-500">
      {formData.title.length}/100 {$_("exchanges.characters")}
    </p>
  </div>

  <!-- Description -->
  <div>
    <label
      for="description"
      class="block text-sm font-medium text-gray-700 mb-1"
    >
      {$_("exchanges.description")} <span class="text-red-500">*</span>
    </label>
    <textarea
      id="description"
      bind:value={formData.description}
      placeholder={$_("exchanges.descriptionPlaceholder")}
      rows="5"
      maxlength="1000"
      required
      data-testid="exchange-description-input"
      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
    ></textarea>
    <p class="mt-1 text-xs text-gray-500">
      {formData.description.length}/1000 {$_("exchanges.characters")}
    </p>
  </div>

  <!-- Credits (Time) -->
  <div>
    <label for="credits" class="block text-sm font-medium text-gray-700 mb-1">
      {$_("exchanges.credits")} <span class="text-red-500">*</span>
    </label>
    <div class="flex items-center gap-4">
      <input
        id="credits"
        type="range"
        min="1"
        max="100"
        step="1"
        bind:value={formData.credits}
        data-testid="exchange-credits-input"
        class="flex-1"
      />
      <div
        class="flex items-center justify-center w-24 px-3 py-2 bg-blue-100 text-blue-800 rounded-md font-semibold"
      >
        {formData.credits}h
      </div>
    </div>
    <p class="mt-2 text-sm text-gray-600">
      ⏱️ {$_("exchanges.estimatedTime")} <strong>{formData.credits} {formData.credits > 1 ? $_("exchanges.hours") : $_("exchanges.hour")}</strong>
      ({$_("exchanges.creditHour")})
    </p>
    <p class="text-xs text-gray-500 mt-1">
      {$_("exchanges.timeCurrency")}
    </p>
  </div>

  <!-- Legal Notice (Belgian Context) -->
  <div class="p-4 bg-blue-50 border-l-4 border-blue-400 text-sm text-blue-800">
    <p>
      {$_("exchanges.legalNotice")}
    </p>
  </div>

  <!-- Submit Button -->
  <div class="flex justify-end gap-3">
    <button
      type="button"
      on:click={() => dispatch("cancel")}
      class="px-6 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
    >
      {$_("common.cancel")}
    </button>
    <button
      type="submit"
      disabled={loading}
      data-testid="exchange-submit-btn"
      class="px-6 py-2 border border-transparent rounded-md text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {loading ? $_("common.creating") : $_("exchanges.createOffer")}
    </button>
  </div>
</form>
