<script lang="ts">
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
        throw new Error(
          "Vous devez accepter les conditions GDPR pour continuer",
        );
      }
      if (!formData.energy_type) {
        throw new Error("Sélectionnez le type d'énergie");
      }
      if (!formData.total_kwh || formData.total_kwh <= 0) {
        throw new Error(
          "La consommation doit être supérieure à 0 kWh",
        );
      }
      if (!formData.billing_period_start || !formData.billing_period_end) {
        throw new Error("Les dates de facturation sont obligatoires");
      }
      if (formData.billing_period_end! <= formData.billing_period_start!) {
        throw new Error(
          "La date de fin doit être postérieure à la date de début",
        );
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
      error = err.message || "Erreur lors de l'upload de la facture";
      console.error("Failed to upload energy bill:", err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="bg-white shadow-md rounded-lg p-6">
  <h3 class="text-lg font-medium text-gray-900 mb-4">
    📄 Uploader votre facture d'énergie
  </h3>

  <p class="text-sm text-gray-600 mb-6">
    Votre consommation sera <strong>chiffrée (AES-256-GCM)</strong> et
    agrégée de manière anonyme. Les données détaillées ne seront jamais
    partagées avec les fournisseurs.
  </p>

  {#if success}
    <div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-md">
      <p class="text-sm text-green-800">
        ✅ Facture uploadée et chiffrée avec succès ! Merci pour votre
        participation.
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
        Type d'énergie <span class="text-red-500">*</span>
      </label>
      <select
        id="energy_type"
        bind:value={formData.energy_type}
        required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      >
        <option value="">-- Sélectionnez --</option>
        <option value={EnergyType.Electricity}>⚡ Électricité</option>
        <option value={EnergyType.Gas}>🔥 Gaz</option>
        <option value={EnergyType.Heating}>🌡️ Chauffage</option>
      </select>
    </div>

    <!-- Total kWh -->
    <div>
      <label for="total_kwh" class="block text-sm font-medium text-gray-700">
        Consommation totale (kWh) <span class="text-red-500">*</span>
      </label>
      <input
        type="number"
        id="total_kwh"
        bind:value={formData.total_kwh}
        required
        min="1"
        step="0.01"
        placeholder="Ex: 2500"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
      <p class="mt-1 text-xs text-gray-500">
        Trouvez cette valeur sur votre facture d'électricité/gaz sous "Consommation".
      </p>
    </div>

    <!-- Billing Period -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label
          for="period_start"
          class="block text-sm font-medium text-gray-700"
        >
          Début de période <span class="text-red-500">*</span>
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
          Fin de période <span class="text-red-500">*</span>
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
      La période couverte par votre facture (généralement 1-3 mois).
    </p>

    <!-- GDPR Consent -->
    <div class="p-4 bg-blue-50 border-2 border-blue-300 rounded-md">
      <h4 class="text-sm font-medium text-blue-900 mb-3">
        🔒 Consentement GDPR (Obligatoire)
      </h4>
      <div class="space-y-2 text-xs text-blue-800 mb-4">
        <p>
          <strong>Vos droits:</strong>
        </p>
        <ul class="list-disc list-inside space-y-1">
          <li>
            ✅ <strong>Chiffrement total:</strong> Vos données sont chiffrées avec
            AES-256-GCM
          </li>
          <li>
            ✅ <strong>K-anonymité:</strong> Minimum 5 participants avant toute publication
          </li>
          <li>
            ✅ <strong>Données anonymisées:</strong> Seules les statistiques agrégées
            sont partagées
          </li>
          <li>
            ✅ <strong>Droit à l'oubli (Art. 17):</strong> Suppression à tout moment
          </li>
          <li>
            ✅ <strong>Retrait du consentement (Art. 7.3):</strong> Annulez votre
            participation immédiatement
          </li>
          <li>
            ✅ <strong>Rétention limitée:</strong> Données supprimées après 90 jours
          </li>
        </ul>
        <p class="mt-2">
          <strong>Usage des données:</strong> Calculer la consommation totale de
          la copropriété pour négocier avec les fournisseurs d'énergie.
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
          Je consens au traitement de mes données de consommation d'énergie
          selon les conditions GDPR ci-dessus. Je comprends que je peux retirer
          mon consentement à tout moment.
          <span class="text-red-500">*</span>
        </span>
      </label>
    </div>

    <!-- Security Info -->
    <div class="p-3 bg-green-50 border border-green-200 rounded-md">
      <p class="text-xs text-green-800">
        🔐 <strong>Sécurité maximale:</strong> Votre consommation exacte ne sera
        jamais visible par les autres participants ni par les fournisseurs. Seul
        un total agrégé anonymisé sera utilisé pour la négociation.
      </p>
    </div>

    <!-- Submit Button -->
    <div class="flex justify-end space-x-3">
      <button
        type="button"
        on:click={() => dispatch("cancel")}
        class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
      >
        Annuler
      </button>
      <button
        type="submit"
        disabled={loading || !gdprConsent}
        class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if loading}
          <span class="inline-block animate-spin mr-2">⏳</span>
          Chiffrement et upload...
        {:else}
          🔒 Chiffrer et uploader
        {/if}
      </button>
    </div>
  </form>
</div>
