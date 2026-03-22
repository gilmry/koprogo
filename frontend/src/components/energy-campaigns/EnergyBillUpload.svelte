<script lang="ts">
  import { _ } from "svelte-i18n";
  import { createEventDispatcher } from "svelte";
  import {
    energyBillsApi,
    type UploadEnergyBillDto,
    EnergyType,
  } from "../../lib/api/energy-campaigns";

  export let campaignId: string;
  export let unitId: string;

  const dispatch = createEventDispatcher();

  let formData: Partial<UploadEnergyBillDto> = {
    campaign_id: campaignId,
    unit_id: unitId,
    billing_period_start: "",
    billing_period_end: "",
    energy_type: undefined,
    total_kwh: 0,
    consent_signature: "",
  };

  let gdprConsent = false;
  let loading = false;
  let error = "";
  let success = false;

  function generateConsentSignature(): string {
    // Generate a simple signature based on user consent timestamp
    const timestamp = new Date().toISOString();
    const data = `${campaignId}|${unitId}|${timestamp}`;
    // In production, this should be a proper cryptographic signature
    return btoa(data);
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    loading = true;
    error = "";
    success = false;

    try {
      // Validate
      if (!gdprConsent) {
        throw new Error($_("energy.upload.gdprRequired"));
      }
      if (!formData.energy_type) {
        throw new Error($_("energy.upload.typeRequired"));
      }
      if (!formData.total_kwh || formData.total_kwh <= 0) {
        throw new Error($_("energy.upload.consumptionRequired"));
      }
      if (!formData.billing_period_start || !formData.billing_period_end) {
        throw new Error($_("energy.upload.datesRequired"));
      }
      if (formData.billing_period_end! <= formData.billing_period_start!) {
        throw new Error($_("energy.upload.dateInvalid"));
      }

      // Generate GDPR consent signature
      formData.consent_signature = generateConsentSignature();

      const upload = await energyBillsApi.upload(
        formData as UploadEnergyBillDto,
      );
      success = true;
      dispatch("uploaded", upload);

      // Reset form after 2 seconds
      setTimeout(() => {
        formData = {
          campaign_id: campaignId,
          unit_id: unitId,
          billing_period_start: "",
          billing_period_end: "",
          energy_type: undefined,
          total_kwh: 0,
          consent_signature: "",
        };
        gdprConsent = false;
        success = false;
      }, 2000);
    } catch (err: any) {
      error = err.message || $_("energy.upload.uploadError");
      console.error("Failed to upload energy bill:", err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="bg-white shadow-md rounded-lg p-6">
  <h3 class="text-lg font-medium text-gray-900 mb-4">
    📄 {$_("energy.upload.title")}
  </h3>

  <p class="text-sm text-gray-600 mb-6">
    {$_("energy.upload.description")}
  </p>

  {#if success}
    <div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-md">
      <p class="text-sm text-green-800">
        ✅ {$_("energy.upload.successMessage")}
      </p>
    </div>
  {/if}

  {#if error}
    <div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">❌ {error}</p>
    </div>
  {/if}

  <form on:submit={handleSubmit} class="space-y-6">
    <!-- Energy Type -->
    <div>
      <label for="energy_type" class="block text-sm font-medium text-gray-700">
        {$_("energy.upload.energyType")} <span class="text-red-500">*</span>
      </label>
      <select
        id="energy_type"
        bind:value={formData.energy_type}
        required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      >
        <option value="">-- {$_("common.select")} --</option>
        <option value={EnergyType.Electricity}>⚡ {$_("energy.electricity")}</option>
        <option value={EnergyType.Gas}>🔥 {$_("energy.gas")}</option>
        <option value={EnergyType.Heating}>🌡️ {$_("energy.heating")}</option>
      </select>
    </div>

    <!-- Total kWh -->
    <div>
      <label for="total_kwh" class="block text-sm font-medium text-gray-700">
        {$_("energy.upload.totalConsumption")} <span class="text-red-500">*</span>
      </label>
      <input
        type="number"
        id="total_kwh"
        bind:value={formData.total_kwh}
        required
        min="1"
        step="0.01"
        placeholder={$_("energy.upload.consumptionExample")}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
      <p class="mt-1 text-xs text-gray-500">
        {$_("energy.upload.consumptionHelp")}
      </p>
    </div>

    <!-- Billing Period -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label
          for="period_start"
          class="block text-sm font-medium text-gray-700"
        >
          {$_("energy.upload.periodStart")} <span class="text-red-500">*</span>
        </label>
        <input
          type="date"
          id="period_start"
          bind:value={formData.billing_period_start}
          required
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        />
      </div>
      <div>
        <label
          for="period_end"
          class="block text-sm font-medium text-gray-700"
        >
          {$_("energy.upload.periodEnd")} <span class="text-red-500">*</span>
        </label>
        <input
          type="date"
          id="period_end"
          bind:value={formData.billing_period_end}
          required
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        />
      </div>
    </div>
    <p class="text-xs text-gray-500">
      {$_("energy.upload.periodHelp")}
    </p>

    <!-- GDPR Consent -->
    <div class="p-4 bg-blue-50 border-2 border-blue-300 rounded-md">
      <h4 class="text-sm font-medium text-blue-900 mb-3">
        🔒 {$_("energy.upload.gdprTitle")}
      </h4>
      <div class="space-y-2 text-xs text-blue-800 mb-4">
        <p>
          <strong>{$_("energy.upload.yourRights")}:</strong>
        </p>
        <ul class="list-disc list-inside space-y-1">
          <li>
            ✅ <strong>{$_("energy.upload.gdprPoint1Title")}:</strong> {$_("energy.upload.gdprPoint1")}
          </li>
          <li>
            ✅ <strong>{$_("energy.upload.gdprPoint2Title")}:</strong> {$_("energy.upload.gdprPoint2")}
          </li>
          <li>
            ✅ <strong>{$_("energy.upload.gdprPoint3Title")}:</strong> {$_("energy.upload.gdprPoint3")}
          </li>
          <li>
            ✅ <strong>{$_("energy.upload.gdprPoint4Title")}:</strong> {$_("energy.upload.gdprPoint4")}
          </li>
          <li>
            ✅ <strong>{$_("energy.upload.gdprPoint5Title")}:</strong> {$_("energy.upload.gdprPoint5")}
          </li>
          <li>
            ✅ <strong>{$_("energy.upload.gdprPoint6Title")}:</strong> {$_("energy.upload.gdprPoint6")}
          </li>
        </ul>
        <p class="mt-2">
          <strong>{$_("energy.upload.dataUsage")}:</strong> {$_("energy.upload.dataUsageDetails")}
        </p>
      </div>
      <label class="flex items-start">
        <input
          type="checkbox"
          bind:checked={gdprConsent}
          required
          class="mt-0.5 rounded border-blue-300 text-indigo-600 focus:ring-indigo-500"
        />
        <span class="ml-2 text-sm text-blue-900">
          {$_("energy.upload.gdprConsent")}
          <span class="text-red-500">*</span>
        </span>
      </label>
    </div>

    <!-- Security Info -->
    <div class="p-3 bg-green-50 border border-green-200 rounded-md">
      <p class="text-xs text-green-800">
        🔐 {$_("energy.upload.securityTitle")} {$_("energy.upload.securityDetails")}
      </p>
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
        disabled={loading || !gdprConsent}
        class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if loading}
          <span class="inline-block animate-spin mr-2">⏳</span>
          {$_("energy.upload.uploading")}
        {:else}
          🔒 {$_("energy.upload.encryptAndUpload")}
        {/if}
      </button>
    </div>
  </form>
</div>
