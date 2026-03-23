<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    ticketsApi,
    TicketPriority,
    TicketCategory,
    type CreateTicketDto,
  } from "../../lib/api/tickets";
  import { api } from "../../lib/api";
  import { toast } from "../../stores/toast";
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
    unit_id: unitId,
  };

  let submitting = false;
  let errors: Record<string, string> = {};

  // Charger la liste des immeubles quand le modal s'ouvre
  $: if (open && buildings.length === 0) {
    loadBuildings();
  }

  async function loadBuildings() {
    try {
      loadingBuildings = true;
      const response = await api.get<PageResponse<Building>>('/buildings?per_page=100');
      buildings = response.data;
      // Si pas de buildingId pré-sélectionné et un seul immeuble, le sélectionner
      if (!formData.building_id && buildings.length === 1) {
        formData.building_id = buildings[0].id;
      }
    } catch (e) {
      console.error('Erreur chargement immeubles:', e);
    } finally {
      loadingBuildings = false;
    }
  }

  function validate(): boolean {
    errors = {};

    if (!formData.building_id) {
      errors.building_id = $_('validation.buildingRequired');
    }

    if (!formData.title || formData.title.trim().length < 3) {
      errors.title = $_('validation.titleMinLength');
    }

    if (!formData.description || formData.description.trim().length < 10) {
      errors.description = $_('validation.descriptionMinLength');
    }

    return Object.keys(errors).length === 0;
  }

  async function handleSubmit() {
    if (!validate()) {
      toast.error($_('validation.fixErrors'));
      return;
    }

    try {
      submitting = true;

      const ticket = await ticketsApi.create({
        ...formData,
        requester_id: requesterId,
      });

      toast.success($_('tickets.createSuccess'));

      dispatch("created", ticket);
      handleClose();
    } catch (err: any) {
      toast.error(err.message || $_('tickets.createError'));
    } finally {
      submitting = false;
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
      unit_id: unitId,
    };
    errors = {};
    dispatch("close");
  }
</script>

<Modal {open} on:close={handleClose} title={$_('tickets.createTitle')}>
  <form on:submit|preventDefault={handleSubmit}>
    <div class="space-y-4">
      <!-- Sélecteur d'immeuble -->
      {#if !buildingId}
        <div>
          <label for="building-select" class="block text-sm font-medium text-gray-700 mb-1">
            {$_('buildings.building')} *
          </label>
          {#if loadingBuildings}
            <p class="text-sm text-gray-500">{$_('buildings.loading')}</p>
          {:else}
            <select
              id="building-select"
              bind:value={formData.building_id}
              required
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
        label={$_('common.title')}
        bind:value={formData.title}
        error={errors.title}
        required
        placeholder={$_('tickets.titlePlaceholder')}
      />

      <!-- Description -->
      <FormTextarea
        label={$_('common.description')}
        bind:value={formData.description}
        error={errors.description}
        required
        rows={4}
        placeholder={$_('tickets.descriptionPlaceholder')}
      />

      <!-- Priorité -->
      <FormSelect
        label={$_('tickets.priority')}
        bind:value={formData.priority}
        required
      >
        <option value={TicketPriority.Low}>{$_('tickets.priorities.low')}</option>
        <option value={TicketPriority.Medium}>{$_('tickets.priorities.medium')}</option>
        <option value={TicketPriority.High}>{$_('tickets.priorities.high')}</option>
        <option value={TicketPriority.Urgent}>{$_('tickets.priorities.urgent')}</option>
        <option value={TicketPriority.Critical}>{$_('tickets.priorities.critical')}</option>
      </FormSelect>

      <!-- Catégorie -->
      <FormSelect
        label={$_('tickets.category')}
        bind:value={formData.category}
        required
      >
        <option value={TicketCategory.General}>{$_('tickets.categories.general')}</option>
        <option value={TicketCategory.Plumbing}>{$_('tickets.categories.plumbing')}</option>
        <option value={TicketCategory.Electrical}>{$_('tickets.categories.electrical')}</option>
        <option value={TicketCategory.Heating}>{$_('tickets.categories.heating')}</option>
        <option value={TicketCategory.Cleaning}>{$_('tickets.categories.cleaning')}</option>
        <option value={TicketCategory.Security}>{$_('tickets.categories.security')}</option>
        <option value={TicketCategory.Emergency}>{$_('tickets.categories.emergency')}</option>
      </FormSelect>

      <!-- Lot (optionnel) -->
      {#if !unitId}
        <FormInput
          label={$_('tickets.unitOptional')}
          bind:value={formData.unit_id}
          placeholder={$_('tickets.unitPlaceholder')}
        />
      {/if}
    </div>

    <!-- Actions -->
    <div class="mt-6 flex justify-end space-x-3">
      <Button type="button" variant="outline" on:click={handleClose}>
        {$_('common.cancel')}
      </Button>
      <Button type="submit" loading={submitting}>
        {$_('tickets.create')}
      </Button>
    </div>
  </form>
</Modal>
