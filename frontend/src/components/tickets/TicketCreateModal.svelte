<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    ticketsApi,
    TicketPriority,
    TicketCategory,
    type CreateTicketDto,
  } from "../../lib/api/tickets";
  import { api } from "../../lib/api";
  import { toast } from "../../stores/toast";
  import { withErrorHandling } from "../../lib/utils/error.utils";
  import { validateCreateTicket } from "../../lib/validators/ticket.validators";
  import type { Building, PageResponse } from "../../lib/types";
  import Modal from "../ui/Modal.svelte";
  import FormInput from "../ui/FormInput.svelte";
  import FormTextarea from "../ui/FormTextarea.svelte";
  import FormSelect from "../ui/FormSelect.svelte";
  import Button from "../ui/Button.svelte";

  export let open = false;
  export let buildingId: string = "";
  export let requesterId: string;
  export let unitId: string | undefined = undefined;

  const dispatch = createEventDispatcher();

  let buildings: Building[] = [];
  let loadingBuildings = false;

  let formData: CreateTicketDto = {
    building_id: buildingId,
    title: "",
    description: "",
    priority: TicketPriority.Medium,
    category: TicketCategory.General,
    requester_id: requesterId,
    unit_id: unitId || undefined,
  };

  let submitting = false;
  let errors: Record<string, string> = {};

  $: if (open && buildings.length === 0) {
    loadBuildings();
  }

  async function loadBuildings() {
    await withErrorHandling({
      action: () => api.get<PageResponse<Building>>('/buildings?per_page=100'),
      setLoading: (v) => loadingBuildings = v,
      onSuccess: (response) => {
        buildings = response.data;
        if (!formData.building_id && buildings.length === 1) {
          formData.building_id = buildings[0].id;
        }
      },
    });
  }

  function validate(): boolean {
    errors = validateCreateTicket(formData, {
      buildingRequired: $_('validation.buildingRequired'),
      titleMinLength: $_('validation.titleMinLength'),
      descriptionMinLength: $_('validation.descriptionMinLength'),
    });
    return Object.keys(errors).length === 0;
  }

  async function handleSubmit() {
    if (!validate()) {
      toast.error($_('validation.fixErrors'));
      return;
    }

    const result = await withErrorHandling({
      action: () => ticketsApi.create({
        ...formData,
        requester_id: requesterId,
        unit_id: formData.unit_id || undefined,
      }),
      setLoading: (v) => submitting = v,
      successMessage: $_('tickets.createSuccess'),
      errorMessage: $_('tickets.createError'),
    });
    if (result) {
      dispatch("created", result);
      handleClose();
    }
  }

  function handleClose() {
    open = false;
    formData = {
      building_id: buildingId,
      title: "",
      description: "",
      priority: TicketPriority.Medium,
      category: TicketCategory.General,
      requester_id: requesterId,
      unit_id: unitId || undefined,
    };
    errors = {};
    dispatch("close");
  }
</script>

<Modal isOpen={open} on:close={handleClose} title={$_('tickets.createTitle')}>
  <form on:submit|preventDefault={handleSubmit} data-testid="ticket-create-form">
    <div class="space-y-4">
      <!-- Sélecteur d'immeuble -->
      {#if !buildingId}
        <div>
          <label for="building-select" class="block text-sm font-medium text-gray-700 mb-1">
            {$_('buildings.building')} *
          </label>
          {#if loadingBuildings}
            <p class="text-sm text-gray-500" data-testid="loading-spinner">{$_('buildings.loading')}</p>
          {:else}
            <select
              id="building-select"
              bind:value={formData.building_id}
              required
              data-testid="ticket-building-select"
              class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              class:border-red-500={errors.building_id}
              class:border-gray-300={!errors.building_id}
            >
              <option value="">{$_('buildings.selectBuilding')}</option>
              {#each buildings as building (building.id)}
                <option value={building.id}>
                  {building.name} - {building.address}, {building.postal_code} {building.city}
                </option>
              {/each}
            </select>
            {#if errors.building_id}
              <p class="text-xs text-red-600 mt-1">{errors.building_id}</p>
            {/if}
          {/if}
        </div>
      {/if}

      <!-- Titre -->
      <FormInput
        id="ticket-title"
        label={$_('common.title')}
        bind:value={formData.title}
        error={errors.title}
        required
        placeholder={$_('tickets.titlePlaceholder')}
        data-testid="ticket-title-input"
      />

      <!-- Description -->
      <FormTextarea
        id="ticket-description"
        label={$_('common.description')}
        bind:value={formData.description}
        error={errors.description}
        required
        rows={4}
        placeholder={$_('tickets.descriptionPlaceholder')}
        data-testid="ticket-description-input"
      />

      <!-- Priorité -->
      <FormSelect
        id="ticket-priority"
        label={$_('tickets.priority')}
        bind:value={formData.priority}
        required
        data-testid="ticket-priority-select"
        options={[
          { value: TicketPriority.Low, label: $_('tickets.priorities.low') },
          { value: TicketPriority.Medium, label: $_('tickets.priorities.medium') },
          { value: TicketPriority.High, label: $_('tickets.priorities.high') },
          { value: TicketPriority.Urgent, label: $_('tickets.priorities.urgent') },
          { value: TicketPriority.Critical, label: $_('tickets.priorities.critical') },
        ]}
      />

      <!-- Catégorie -->
      <FormSelect
        id="ticket-category"
        label={$_('tickets.category')}
        bind:value={formData.category}
        required
        data-testid="ticket-category-select"
        options={[
          { value: TicketCategory.General, label: $_('tickets.categories.general') },
          { value: TicketCategory.Plumbing, label: $_('tickets.categories.plumbing') },
          { value: TicketCategory.Electrical, label: $_('tickets.categories.electrical') },
          { value: TicketCategory.Heating, label: $_('tickets.categories.heating') },
          { value: TicketCategory.Cleaning, label: $_('tickets.categories.cleaning') },
          { value: TicketCategory.Security, label: $_('tickets.categories.security') },
          { value: TicketCategory.Emergency, label: $_('tickets.categories.emergency') },
        ]}
      />

      <!-- Lot (optionnel) -->
      {#if !unitId}
        <FormInput
          id="ticket-unit"
          label={$_('tickets.unitOptional')}
          bind:value={formData.unit_id}
          placeholder={$_('tickets.unitPlaceholder')}
          data-testid="ticket-unit-input"
        />
      {/if}
    </div>

    <!-- Actions -->
    <div class="mt-6 flex justify-end space-x-3">
      <Button type="button" variant="outline" on:click={handleClose} data-testid="ticket-cancel-btn">
        {$_('common.cancel')}
      </Button>
      <Button type="submit" loading={submitting} data-testid="ticket-submit-btn">
        {$_('tickets.create')}
      </Button>
    </div>
  </form>
</Modal>
