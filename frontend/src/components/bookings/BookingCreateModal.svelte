<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { bookingsApi, type BookableResource } from "../../lib/api/bookings";
  import { toast } from "../../stores/toast";
  import Modal from "../ui/Modal.svelte";

  export let isOpen = false;
  export let resource: BookableResource;
  export let ownerId: string;

  const dispatch = createEventDispatcher();

  function getDefaultStart(): string {
    const d = new Date();
    d.setMinutes(d.getMinutes() >= 30 ? 60 : 30, 0, 0);
    return d.toISOString().slice(0, 16);
  }

  function getDefaultEnd(start: string): string {
    const d = new Date(start);
    d.setHours(d.getHours() + 1);
    return d.toISOString().slice(0, 16);
  }

  let startTime = getDefaultStart();
  let endTime = getDefaultEnd(startTime);
  let purpose = "";
  let attendeesCount: number | undefined = undefined;
  let specialRequests = "";
  let submitting = false;
  let errors: Record<string, string> = {};

  $: if (startTime) endTime = getDefaultEnd(startTime);

  function validate(): boolean {
    errors = {};
    const start = new Date(startTime);
    const end = new Date(endTime);
    const now = new Date();

    if (!startTime) {
      errors.startTime = "L'heure de début est requise";
    } else if (start < now) {
      errors.startTime = "L'heure de début doit être dans le futur";
    }

    if (!endTime) {
      errors.endTime = "L'heure de fin est requise";
    } else if (end <= start) {
      errors.endTime = "L'heure de fin doit être après l'heure de début";
    } else if (resource?.max_booking_duration_hours) {
      const durationHours = (end.getTime() - start.getTime()) / (1000 * 3600);
      if (durationHours > resource.max_booking_duration_hours) {
        errors.endTime = `Durée maximum : ${resource.max_booking_duration_hours}h`;
      }
    }

    return Object.keys(errors).length === 0;
  }

  async function handleSubmit() {
    if (!validate()) return;

    try {
      submitting = true;
      const booking = await bookingsApi.createBooking({
        resource_id: resource.id,
        owner_id: ownerId,
        start_time: new Date(startTime).toISOString(),
        end_time: new Date(endTime).toISOString(),
        purpose: purpose || undefined,
        attendees_count: attendeesCount || undefined,
        special_requests: specialRequests || undefined,
      });
      toast.success("Réservation créée avec succès !");
      dispatch("created", booking);
      handleClose();
    } catch (err: any) {
      toast.error(err.message || "Erreur lors de la création de la réservation");
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

<Modal {isOpen} title="Réserver : {resource?.resource_name ?? ''}" on:close={handleClose}>
  <form on:submit|preventDefault={handleSubmit} class="space-y-4">
    <!-- Dates -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <div>
        <label for="booking-start" class="block text-sm font-medium text-gray-700 mb-1">
          Début <span class="text-red-500">*</span>
        </label>
        <input
          id="booking-start"
          type="datetime-local"
          bind:value={startTime}
          class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 {errors.startTime ? 'border-red-500' : 'border-gray-300'}"
        />
        {#if errors.startTime}
          <p class="text-red-500 text-xs mt-1">{errors.startTime}</p>
        {/if}
      </div>
      <div>
        <label for="booking-end" class="block text-sm font-medium text-gray-700 mb-1">
          Fin <span class="text-red-500">*</span>
        </label>
        <input
          id="booking-end"
          type="datetime-local"
          bind:value={endTime}
          class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 {errors.endTime ? 'border-red-500' : 'border-gray-300'}"
        />
        {#if errors.endTime}
          <p class="text-red-500 text-xs mt-1">{errors.endTime}</p>
        {/if}
      </div>
    </div>

    <p class="text-xs text-gray-500">
      Durée max : {resource?.max_booking_duration_hours}h
      {resource?.hourly_rate_credits ? ` · ${resource.hourly_rate_credits} crédit(s)/heure` : " · Gratuit"}
    </p>

    <!-- Purpose -->
    <div>
      <label for="booking-purpose" class="block text-sm font-medium text-gray-700 mb-1">Objet de la réservation</label>
      <input
        id="booking-purpose"
        type="text"
        bind:value={purpose}
        placeholder="Ex : réunion de copropriété, fête d'anniversaire…"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
      />
    </div>

    <!-- Attendees -->
    {#if resource?.capacity}
      <div>
        <label for="booking-attendees" class="block text-sm font-medium text-gray-700 mb-1">
          Nombre de participants (max {resource.capacity})
        </label>
        <input
          id="booking-attendees"
          type="number"
          bind:value={attendeesCount}
          min="1"
          max={resource.capacity}
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
        />
      </div>
    {/if}

    <!-- Special requests -->
    <div>
      <label for="booking-requests" class="block text-sm font-medium text-gray-700 mb-1">Demandes particulières</label>
      <textarea
        id="booking-requests"
        bind:value={specialRequests}
        rows="3"
        placeholder="Besoin d'équipement spécifique, accessibilité PMR…"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 resize-none"
      ></textarea>
    </div>

    {#if resource?.requires_approval}
      <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-3 text-sm text-yellow-800">
        ⚠️ Cette ressource requiert une validation du syndic avant confirmation.
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
        {submitting ? "Création…" : "Confirmer la réservation"}
      </button>
    </div>
  </form>
</Modal>
