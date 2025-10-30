<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { toast } from '../../stores/toast';
  import { api } from '../../lib/api';
  import { authStore } from '../../stores/auth';
  import type { Building, Organization } from '../../lib/types';
  import Modal from '../ui/Modal.svelte';
  import FormInput from '../ui/FormInput.svelte';
  import FormSelect from '../ui/FormSelect.svelte';
  import Button from '../ui/Button.svelte';

  export let isOpen = false;
  export let building: Building | null = null;
  export let mode: 'create' | 'edit' = 'create';

  const dispatch = createEventDispatcher();

  let formData = {
    name: '',
    address: '',
    city: '',
    postal_code: '',
    country: 'Belgique',
    total_units: 0,
    total_tantiemes: 1000,
    construction_year: null as number | null,
    organization_id: '',
  };

  let organizations: Organization[] = [];
  let organizationOptions: Array<{ value: string; label: string }> = [];
  let loadingOrgs = false;
  let isSuperAdmin = false;

  // Check if user is SuperAdmin
  $: if ($authStore.user) {
    isSuperAdmin = $authStore.user.role === 'superadmin';
  }

  // Load organizations when modal opens and user is SuperAdmin
  $: if (isOpen && isSuperAdmin && organizations.length === 0) {
    loadOrganizations();
  }

  async function loadOrganizations() {
    loadingOrgs = true;
    try {
      const response = await api.get<{ data: Organization[] }>('/organizations?per_page=1000');
      organizations = response.data;
      organizationOptions = organizations.map((org) => ({
        value: org.id,
        label: `${org.name} (${org.subscription_plan})`,
      }));
    } catch (e) {
      console.error('Error loading organizations:', e);
    } finally {
      loadingOrgs = false;
    }
  }

  let errors = {
    name: '',
    address: '',
    city: '',
    postal_code: '',
    total_units: '',
    total_tantiemes: '',
    construction_year: '',
    organization_id: '',
  };

  let loading = false;

  // Initialize form with building data if editing
  $: if (building && mode === 'edit') {
    formData = {
      name: building.name,
      address: building.address,
      city: building.city,
      postal_code: building.postal_code,
      country: building.country || 'Belgique',
      total_units: building.total_units,
      total_tantiemes: building.total_tantiemes || 1000,
      construction_year: building.construction_year,
      organization_id: building.organization_id || '',
    };
  }

  const validateForm = (): boolean => {
    let isValid = true;
    errors = {
      name: '',
      address: '',
      city: '',
      postal_code: '',
      total_units: '',
      total_tantiemes: '',
      construction_year: '',
      organization_id: '',
    };

    // Name validation
    if (!formData.name || formData.name.trim().length < 2) {
      errors.name = 'Le nom doit contenir au moins 2 caractères';
      isValid = false;
    }

    // Address validation
    if (!formData.address || formData.address.trim().length < 3) {
      errors.address = 'L\'adresse doit contenir au moins 3 caractères';
      isValid = false;
    }

    // City validation
    if (!formData.city || formData.city.trim().length < 2) {
      errors.city = 'La ville doit contenir au moins 2 caractères';
      isValid = false;
    }

    // Postal code validation
    if (!formData.postal_code || formData.postal_code.trim().length < 2) {
      errors.postal_code = 'Le code postal est requis';
      isValid = false;
    }

    // Total units validation
    if (formData.total_units < 1) {
      errors.total_units = 'Le nombre de lots doit être au moins 1';
      isValid = false;
    }

    // Total tantièmes validation
    if (formData.total_tantiemes < 1) {
      errors.total_tantiemes = 'Le total des tantièmes doit être au moins 1';
      isValid = false;
    }

    // Construction year validation (optional, but if provided must be valid)
    if (formData.construction_year !== null) {
      const currentYear = new Date().getFullYear();
      if (formData.construction_year < 1800 || formData.construction_year > currentYear + 5) {
        errors.construction_year = `L'année doit être entre 1800 et ${currentYear + 5}`;
        isValid = false;
      }
    }

    // Organization validation (only for SuperAdmin in create mode)
    if (isSuperAdmin && mode === 'create' && !formData.organization_id) {
      errors.organization_id = 'L\'organisation est requise';
      isValid = false;
    }

    return isValid;
  };

  const handleSubmit = async () => {
    if (!validateForm()) {
      return;
    }

    loading = true;

    try {
      const payload: any = {
        name: formData.name.trim(),
        address: formData.address.trim(),
        city: formData.city.trim(),
        postal_code: formData.postal_code.trim(),
        country: formData.country.trim(),
        total_units: formData.total_units,
        total_tantiemes: formData.total_tantiemes,
        construction_year: formData.construction_year,
      };

      // Include organization_id for SuperAdmins
      if (isSuperAdmin && formData.organization_id) {
        payload.organization_id = formData.organization_id;
      }

      if (mode === 'create') {
        await api.post('/buildings', payload);
        toast.show('Immeuble créé avec succès', 'success');
      } else if (building) {
        await api.put(`/buildings/${building.id}`, payload);
        toast.show('Immeuble mis à jour avec succès', 'success');
      }

      // Set loading to false before closing modal
      loading = false;

      // Close modal first
      handleClose();

      // Then dispatch success to reload data
      dispatch('success');
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Une erreur est survenue';
      toast.show(errorMessage, 'error');
      loading = false;
    }
  };

  const handleClose = () => {
    if (!loading) {
      isOpen = false;
      // Reset form
      formData = {
        name: '',
        address: '',
        city: '',
        postal_code: '',
        country: 'Belgique',
        total_units: 0,
        total_tantiemes: 1000,
        construction_year: null,
        organization_id: '',
      };
      errors = {
        name: '',
        address: '',
        city: '',
        postal_code: '',
        total_units: '',
        total_tantiemes: '',
        construction_year: '',
        organization_id: '',
      };
      dispatch('close');
    }
  };
</script>

<Modal
  {isOpen}
  title={mode === 'create' ? 'Nouvel Immeuble' : 'Modifier l\'Immeuble'}
  size="lg"
  on:close={handleClose}
>
  <form
    on:submit|preventDefault={handleSubmit}
    class="space-y-4"
    data-testid="building-form"
  >
    {#if isSuperAdmin}
      <FormSelect
        id="building-organization"
        label="Organisation"
        bind:value={formData.organization_id}
        options={organizationOptions}
        error={errors.organization_id}
        required={mode === 'create'}
        placeholder={loadingOrgs ? 'Chargement...' : 'Sélectionner une organisation'}
        disabled={loadingOrgs}
        data-testid="building-organization-select"
      />
    {/if}

    <FormInput
      id="building-name"
      label="Nom de l'immeuble"
      type="text"
      bind:value={formData.name}
      error={errors.name}
      required
      placeholder="Résidence Les Peupliers"
      data-testid="building-name-input"
    />

    <FormInput
      id="building-address"
      label="Adresse"
      type="text"
      bind:value={formData.address}
      error={errors.address}
      required
      placeholder="123 Rue de la Paix"
      data-testid="building-address-input"
    />

    <div class="grid grid-cols-2 gap-4">
      <FormInput
        id="building-postal-code"
        label="Code postal"
        type="text"
        bind:value={formData.postal_code}
        error={errors.postal_code}
        required
        placeholder="1000"
        data-testid="building-postalcode-input"
      />

      <FormInput
        id="building-city"
        label="Ville"
        type="text"
        bind:value={formData.city}
        error={errors.city}
        required
        placeholder="Bruxelles"
        data-testid="building-city-input"
      />
    </div>

    <FormInput
      id="building-country"
      label="Pays"
      type="text"
      bind:value={formData.country}
      placeholder="Belgique"
      data-testid="building-country-input"
    />

    <div class="grid grid-cols-2 gap-4">
      <FormInput
        id="building-total-units"
        label="Nombre de lots"
        type="number"
        bind:value={formData.total_units}
        error={errors.total_units}
        required
        placeholder="10"
        data-testid="building-totalunits-input"
      />

      <FormInput
        id="building-total-tantiemes"
        label="Total tantièmes (millièmes)"
        type="number"
        bind:value={formData.total_tantiemes}
        error={errors.total_tantiemes}
        required
        placeholder="1000"
        hint="Généralement 1000 en Belgique"
        data-testid="building-totaltantiemes-input"
      />
    </div>

    <FormInput
      id="building-construction-year"
      label="Année de construction"
      type="number"
      bind:value={formData.construction_year}
      error={errors.construction_year}
      placeholder="2000"
      hint="Optionnel"
      on:input={(e) => {
        const val = e.target.value;
        formData.construction_year = val === '' ? null : parseInt(val);
      }}
      data-testid="building-constructionyear-input"
    />
  </form>

  <svelte:fragment slot="footer">
    <div class="flex justify-end space-x-3">
      <Button
        variant="outline"
        on:click={handleClose}
        disabled={loading}
        data-testid="building-cancel-button"
      >
        Annuler
      </Button>
      <Button
        variant="primary"
        on:click={handleSubmit}
        {loading}
        data-testid="building-submit-button"
      >
        {mode === 'create' ? 'Créer l\'immeuble' : 'Enregistrer les modifications'}
      </Button>
    </div>
  </svelte:fragment>
</Modal>
