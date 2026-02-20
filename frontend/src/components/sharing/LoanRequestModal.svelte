<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { sharingApi, type SharedObject } from "../../lib/api/sharing";
  import { toast } from "../../stores/toast";
  import Modal from "../ui/Modal.svelte";

  export let isOpen = false;
  export let object: SharedObject;
  export let borrowerId: string;

  const dispatch = createEventDispatcher();

  function getDefaultStart(): string {
    const d = new Date();
    d.setDate(d.getDate() + 1);
    return d.toISOString().slice(0, 10);
  }

  function getDefaultEnd(start: string, durationDays: number): string {
    const d = new Date(start);
    d.setDate(d.getDate() + durationDays);
    return d.toISOString().slice(0, 10);
  }

  let loanStartDate = getDefaultStart();
  $: loanEndDate = getDefaultEnd(loanStartDate, object?.loan_duration_days ?? 7);
  let notes = "";
  let submitting = false;
  let errors: Record<string, string> = {};

  function validate(): boolean {
    errors = {};
    const start = new Date(loanStartDate);
    const end = new Date(loanEndDate);
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    if (!loanStartDate) {
      errors.startDate = "La date de début est requise";
    } else if (start < today) {
      errors.startDate = "La date de début ne peut pas être dans le passé";
    }

    if (!loanEndDate) {
      errors.endDate = "La date de retour est requise";
    } else if (end <= start) {
      errors.endDate = "La date de retour doit être après la date de début";
    }

    return Object.keys(errors).length === 0;
  }

  async function handleSubmit() {
    if (!validate()) return;

    try {
      submitting = true;
      const loan = await sharingApi.createLoan({
        shared_object_id: object.id,
        borrower_id: borrowerId,
        loan_start_date: loanStartDate,
        loan_end_date: loanEndDate,
        notes: notes || undefined,
      });
      toast.success("Demande d'emprunt envoyée !");
      dispatch("created", loan);
      handleClose();
    } catch (err: any) {
      toast.error(err.message || "Erreur lors de la demande d'emprunt");
    } finally {
      submitting = false;
    }
  }

  function handleClose() {
    isOpen = false;
    errors = {};
    dispatch("close");
  }
</script>

<Modal {isOpen} title="Emprunter : {object?.object_name ?? ''}" on:close={handleClose}>
  <div class="space-y-4">
    <!-- Object summary -->
    <div class="bg-gray-50 rounded-lg p-3 text-sm space-y-1">
      {#if object?.owner_name}
        <p><span class="font-medium">Propriétaire :</span> {object.owner_name}</p>
      {/if}
      <p><span class="font-medium">Durée max :</span> {object?.loan_duration_days} jours</p>
      {#if object?.deposit_required_cents}
        <p>
          <span class="font-medium">Caution :</span>
          €{(object.deposit_required_cents / 100).toFixed(2)}
        </p>
      {/if}
    </div>

    <form on:submit|preventDefault={handleSubmit} class="space-y-4">
      <!-- Dates -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div>
          <label for="loan-start-date" class="block text-sm font-medium text-gray-700 mb-1">
            Date de début <span class="text-red-500">*</span>
          </label>
          <input
            id="loan-start-date"
            type="date"
            bind:value={loanStartDate}
            class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 {errors.startDate ? 'border-red-500' : 'border-gray-300'}"
          />
          {#if errors.startDate}
            <p class="text-red-500 text-xs mt-1">{errors.startDate}</p>
          {/if}
        </div>
        <div>
          <label for="loan-end-date" class="block text-sm font-medium text-gray-700 mb-1">
            Date de retour prévue <span class="text-red-500">*</span>
          </label>
          <input
            id="loan-end-date"
            type="date"
            bind:value={loanEndDate}
            class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 {errors.endDate ? 'border-red-500' : 'border-gray-300'}"
          />
          {#if errors.endDate}
            <p class="text-red-500 text-xs mt-1">{errors.endDate}</p>
          {/if}
        </div>
      </div>

      <!-- Notes -->
      <div>
        <label for="loan-notes" class="block text-sm font-medium text-gray-700 mb-1">Notes (facultatif)</label>
        <textarea
          id="loan-notes"
          bind:value={notes}
          rows="3"
          placeholder="Précisez l'usage prévu, une question au propriétaire…"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 resize-none"
        ></textarea>
      </div>

      {#if object?.usage_instructions}
        <div class="bg-blue-50 border border-blue-200 rounded-lg p-3 text-sm text-blue-800">
          <p class="font-medium mb-1">Instructions d'utilisation :</p>
          <p>{object.usage_instructions}</p>
        </div>
      {/if}

      <!-- Actions -->
      <div class="flex justify-end gap-3 pt-2">
        <button
          type="button"
          on:click={handleClose}
          class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition"
        >
          Annuler
        </button>
        <button
          type="submit"
          disabled={submitting}
          class="px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 disabled:opacity-50 transition"
        >
          {submitting ? "Envoi…" : "Envoyer la demande"}
        </button>
      </div>
    </form>
  </div>
</Modal>
