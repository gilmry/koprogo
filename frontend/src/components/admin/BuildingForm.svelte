<script lang="ts">
  // Svelte 5 runes mode
  import type { Snippet } from 'svelte';
  import { _ } from '../../lib/i18n';
  import { toast } from '../../stores/toast';
  import { api } from '../../lib/api';
  import { authStore } from '../../stores/auth';
  import type { Building } from '../../lib/types';
  import { listAcps, type AcpResponseDto } from '../../lib/api/acps';
  import Modal from '../ui/Modal.svelte';
  import FormInput from '../ui/FormInput.svelte';
  import FormSelect from '../ui/FormSelect.svelte';
  import Button from '../ui/Button.svelte';

  let {
    isOpen = false,
    building = null,
    mode = 'create',
    onclose,
    onsuccess,
  }: {
    isOpen?: boolean;
    building?: Building | null;
    mode?: 'create' | 'edit';
    onclose?: () => void;
    onsuccess?: () => void;
  } = $props();

  let formData = $state({
    name: '',
    address: '',
    city: '',
    postal_code: '',
    country: 'Belgique',
    total_units: 0,
    total_tantiemes: 1000,
    construction_year: null as number | null,
    acp_id: '',
  });

  let acps = $state<AcpResponseDto[]>([]);
  let acpOptions = $state<Array<{ value: string; label: string }>>([]);
  let loadingAcps = $state(false);
  let acpsLoadAttempted = $state(false);
  let isSuperAdmin = $state(false);

  // Check if user is SuperAdmin
  $effect(() => {
    if ($authStore.user) {
      isSuperAdmin = $authStore.user.role === 'superadmin';
    }
  });

  // Load ACPs when modal opens and user is SuperAdmin. `acpsLoadAttempted`
  // évite une boucle infinie de requêtes quand la liste d'ACP est
  // légitimement vide (acps.length reste à 0 après un chargement réussi) —
  // sans lui, l'effet se re-déclenche indéfiniment tant qu'aucune ACP
  // n'existe (bug réel, pas seulement un artefact de test).
  $effect(() => {
    if (isOpen && isSuperAdmin && !acpsLoadAttempted) {
      loadAcps();
    }
    if (!isOpen) {
      acpsLoadAttempted = false;
    }
  });

  async function loadAcps() {
    loadingAcps = true;
    acpsLoadAttempted = true;
    try {
      acps = await listAcps();
      acpOptions = acps.map((acp) => ({
        value: acp.id,
        label: `${acp.name} — ${acp.address_street}, ${acp.address_postal_code} ${acp.address_city}`,
      }));
    } catch (e) {
      console.error('Error loading ACPs:', e);
    } finally {
      loadingAcps = false;
    }
  }

  let errors = $state({
    name: '',
    address: '',
    city: '',
    postal_code: '',
    total_units: '',
    total_tantiemes: '',
    construction_year: '',
    acp_id: '',
  });

  let loading = $state(false);

  // Initialize form with building data if editing
  $effect(() => {
    if (building && mode === 'edit') {
      formData = {
        name: building.name,
        address: building.address,
        city: building.city,
        postal_code: building.postal_code,
        country: building.country || 'Belgique',
        total_units: building.total_units,
        total_tantiemes: building.total_tantiemes || 1000,
        construction_year: building.construction_year ?? null,
        acp_id: building.acp_id || '',
      };
    }
  });

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
      acp_id: '',
    };

    // Name validation
    if (!formData.name || formData.name.trim().length < 2) {
      errors.name = $_('admin.building.nameError');
      isValid = false;
    }

    // Address validation
    if (!formData.address || formData.address.trim().length < 3) {
      errors.address = $_('admin.building.addressError');
      isValid = false;
    }

    // City validation
    if (!formData.city || formData.city.trim().length < 2) {
      errors.city = $_('admin.building.cityError');
      isValid = false;
    }

    // Postal code validation
    if (!formData.postal_code || formData.postal_code.trim().length < 2) {
      errors.postal_code = $_('admin.building.postalCodeError');
      isValid = false;
    }

    // Total units validation
    if (formData.total_units < 1) {
      errors.total_units = $_('admin.building.totalUnitsError');
      isValid = false;
    }

    // Total tantiemes validation
    if (formData.total_tantiemes < 1) {
      errors.total_tantiemes = $_('admin.building.totalTantgemesError');
      isValid = false;
    }

    // Construction year validation (optional, but if provided must be valid)
    if (formData.construction_year !== null) {
      const currentYear = new Date().getFullYear();
      if (formData.construction_year < 1800 || formData.construction_year > currentYear + 5) {
        errors.construction_year = $_('admin.building.constructionYearError', { values: { min: 1800, max: currentYear + 5 } });
        isValid = false;
      }
    }

    // ACP validation (only for SuperAdmin in create mode — required by backend)
    if (isSuperAdmin && mode === 'create' && !formData.acp_id) {
      errors.acp_id = $_('admin.building.acpRequired');
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

      // acp_id : requis à la création (SuperAdmin uniquement, cf. backend
      // building_handlers.rs), optionnel en édition (réaffectation d'ACP).
      if (mode === 'create') {
        if (isSuperAdmin && formData.acp_id) {
          payload.acp_id = formData.acp_id;
        }
      } else if (mode === 'edit' && isSuperAdmin && formData.acp_id) {
        payload.acp_id = formData.acp_id;
      }

      if (mode === 'create') {
        await api.post('/buildings', payload);
        toast.show($_('admin.building.createdSuccessfully'), 'success');
      } else if (building) {
        await api.put(`/buildings/${building.id}`, payload);
        toast.show($_('admin.building.updatedSuccessfully'), 'success');
      }

      // Set loading to false before closing modal
      loading = false;

      // Close modal first
      handleClose();

      // Then dispatch success to reload data
      onsuccess?.();
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : $_('common.errorOccurred');
      toast.show(errorMessage, 'error');
      loading = false;
    }
  };

  const handleClose = () => {
    if (!loading) {
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
        acp_id: '',
      };
      errors = {
        name: '',
        address: '',
        city: '',
        postal_code: '',
        total_units: '',
        total_tantiemes: '',
        construction_year: '',
        acp_id: '',
      };
      onclose?.();
    }
  };
</script>

<Modal
  {isOpen}
  title={mode === 'create' ? $_('admin.building.newBuilding') : $_('admin.building.editBuilding')}
  size="lg"
  onclose={handleClose}
>
  <form
    onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}
    class="space-y-4"
    data-testid="building-form"
  >
    {#if isSuperAdmin}
      <FormSelect
        id="building-acp"
        label={$_('admin.building.acp')}
        bind:value={formData.acp_id}
        options={acpOptions}
        error={errors.acp_id}
        required={mode === 'create'}
        placeholder={loadingAcps ? $_('common.loading') : $_('admin.building.selectAcp')}
        hint={!loadingAcps && acpOptions.length === 0 ? $_('admin.building.noAcpAvailable') : ''}
        disabled={loadingAcps}
        data-testid="building-acp-select"
      />
    {/if}

    <FormInput
      id="building-name"
      label={$_('admin.building.name')}
      type="text"
      bind:value={formData.name}
      error={errors.name}
      required
      placeholder="Residence Les Peupliers"
      data-testid="building-name-input"
    />

    <FormInput
      id="building-address"
      label={$_('common.address')}
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
        label={$_('common.postalCode')}
        type="text"
        bind:value={formData.postal_code}
        error={errors.postal_code}
        required
        placeholder="1000"
        data-testid="building-postalcode-input"
      />

      <FormInput
        id="building-city"
        label={$_('common.city')}
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
      label={$_('common.country')}
      type="text"
      bind:value={formData.country}
      placeholder="Belgique"
      data-testid="building-country-input"
    />

    <div class="grid grid-cols-2 gap-4">
      <FormInput
        id="building-total-units"
        label={$_('admin.building.totalUnits')}
        type="number"
        bind:value={formData.total_units}
        error={errors.total_units}
        required
        placeholder="10"
        data-testid="building-totalunits-input"
      />

      <FormInput
        id="building-total-tantiemes"
        label={$_('admin.building.totalTantiemes')}
        type="number"
        bind:value={formData.total_tantiemes}
        error={errors.total_tantiemes}
        required
        placeholder="1000"
        hint={$_('admin.building.tantgemesHint')}
        data-testid="building-totaltantiemes-input"
      />
    </div>

    <FormInput
      id="building-construction-year"
      label={$_('admin.building.constructionYear')}
      type="number"
      bind:value={formData.construction_year}
      error={errors.construction_year}
      placeholder="2000"
      hint={$_('common.optional')}
      oninput={(e: Event) => {
        const val = (e.target as HTMLInputElement).value;
        formData.construction_year = val === '' ? null : parseInt(val);
      }}
      data-testid="building-constructionyear-input"
    />
  </form>

  {#snippet footer()}
    <div class="flex justify-end space-x-3">
      <Button
        variant="outline"
        onclick={handleClose}
        disabled={loading}
        data-testid="building-cancel-button"
      >
        {$_('common.cancel')}
      </Button>
      <Button
        variant="primary"
        onclick={handleSubmit}
        {loading}
        data-testid="building-submit-button"
      >
        {mode === 'create' ? $_('admin.building.createBuilding') : $_('common.saveChanges')}
      </Button>
    </div>
  {/snippet}
</Modal>
