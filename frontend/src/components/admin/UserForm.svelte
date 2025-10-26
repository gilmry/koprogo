<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { toast } from '../../stores/toast';
  import { api } from '../../lib/api';
  import { UserRole, type User, type Organization } from '../../lib/types';
  import Modal from '../ui/Modal.svelte';
  import FormInput from '../ui/FormInput.svelte';
  import FormSelect from '../ui/FormSelect.svelte';
  import Button from '../ui/Button.svelte';

  export let isOpen = false;
  export let user: User | null = null;
  export let mode: 'create' | 'edit' = 'create';

  const dispatch = createEventDispatcher();

  let formData = {
    email: '',
    password: '',
    confirmPassword: '',
    firstName: '',
    lastName: '',
    role: UserRole.OWNER,
    organizationId: '',
  };

  let errors = {
    email: '',
    password: '',
    confirmPassword: '',
    firstName: '',
    lastName: '',
    organizationId: '',
  };

  let organizations: Organization[] = [];
  let loading = false;
  let loadingOrgs = false;

  const roleOptions = [
    { value: UserRole.OWNER, label: 'Copropriétaire' },
    { value: UserRole.ACCOUNTANT, label: 'Comptable' },
    { value: UserRole.SYNDIC, label: 'Syndic (Gestionnaire)' },
    { value: UserRole.SUPERADMIN, label: 'Super Administrateur' },
  ];

  let organizationOptions: Array<{ value: string; label: string }> = [];

  // Load organizations on mount
  onMount(async () => {
    await loadOrganizations();
  });

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

  // Initialize form with user data if editing
  $: if (user && mode === 'edit') {
    formData = {
      email: user.email,
      password: '',
      confirmPassword: '',
      firstName: user.firstName,
      lastName: user.lastName,
      role: user.role,
      organizationId: user.organizationId || '',
    };
  }

  $: requiresOrganization = formData.role !== UserRole.SUPERADMIN;

  const validateForm = (): boolean => {
    let isValid = true;
    errors = {
      email: '',
      password: '',
      confirmPassword: '',
      firstName: '',
      lastName: '',
      organizationId: '',
    };

    // Email validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!formData.email) {
      errors.email = 'L\'email est requis';
      isValid = false;
    } else if (!emailRegex.test(formData.email)) {
      errors.email = 'Format d\'email invalide';
      isValid = false;
    }

    // Password validation (only for create mode or if password is provided in edit mode)
    if (mode === 'create' || formData.password) {
      if (!formData.password) {
        errors.password = 'Le mot de passe est requis';
        isValid = false;
      } else if (formData.password.length < 6) {
        errors.password = 'Le mot de passe doit contenir au moins 6 caractères';
        isValid = false;
      }

      if (formData.password !== formData.confirmPassword) {
        errors.confirmPassword = 'Les mots de passe ne correspondent pas';
        isValid = false;
      }
    }

    // First name validation
    if (!formData.firstName || formData.firstName.trim().length < 2) {
      errors.firstName = 'Le prénom doit contenir au moins 2 caractères';
      isValid = false;
    }

    // Last name validation
    if (!formData.lastName || formData.lastName.trim().length < 2) {
      errors.lastName = 'Le nom doit contenir au moins 2 caractères';
      isValid = false;
    }

    // Organization validation
    if (requiresOrganization && !formData.organizationId) {
      errors.organizationId = 'L\'organisation est requise pour ce rôle';
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
        email: formData.email.trim(),
        first_name: formData.firstName.trim(),
        last_name: formData.lastName.trim(),
        role: formData.role,
      };

      if (formData.password && mode === 'create') {
        payload.password = formData.password;
      }

      if (requiresOrganization && formData.organizationId) {
        payload.organization_id = formData.organizationId;
      }

      if (mode === 'create') {
        await api.post('/users', payload);
        toast.show('Utilisateur créé avec succès', 'success');
      } else if (user) {
        await api.put(`/users/${user.id}`, payload);
        toast.show('Utilisateur mis à jour avec succès', 'success');
      }

      loading = false;
      handleClose();
      dispatch('success');
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Une erreur est survenue';

      if (errorMessage.includes('email')) {
        errors.email = 'Cet email est déjà utilisé';
      } else {
        toast.show(errorMessage, 'error');
      }
      loading = false;
    }
  };

  const handleClose = () => {
    if (!loading) {
      isOpen = false;
      // Reset form
      formData = {
        email: '',
        password: '',
        confirmPassword: '',
        firstName: '',
        lastName: '',
        role: UserRole.OWNER,
        organizationId: '',
      };
      errors = {
        email: '',
        password: '',
        confirmPassword: '',
        firstName: '',
        lastName: '',
        organizationId: '',
      };
      dispatch('close');
    }
  };
</script>

<Modal
  {isOpen}
  title={mode === 'create' ? 'Nouvel Utilisateur' : 'Modifier l\'Utilisateur'}
  size="md"
  on:close={handleClose}
>
  <form on:submit|preventDefault={handleSubmit} class="space-y-4">
    <div class="grid grid-cols-2 gap-4">
      <FormInput
        id="user-firstName"
        label="Prénom"
        type="text"
        bind:value={formData.firstName}
        error={errors.firstName}
        required
        placeholder="Jean"
      />

      <FormInput
        id="user-lastName"
        label="Nom"
        type="text"
        bind:value={formData.lastName}
        error={errors.lastName}
        required
        placeholder="Dupont"
      />
    </div>

    <FormInput
      id="user-email"
      label="Email"
      type="email"
      bind:value={formData.email}
      error={errors.email}
      required
      placeholder="jean.dupont@example.com"
      autocomplete="email"
    />

    {#if mode === 'create' || formData.password}
      <FormInput
        id="user-password"
        label={mode === 'create' ? 'Mot de passe' : 'Nouveau mot de passe (laisser vide pour ne pas changer)'}
        type="password"
        bind:value={formData.password}
        error={errors.password}
        required={mode === 'create'}
        placeholder="••••••••"
        hint="Au moins 6 caractères"
        autocomplete="new-password"
      />

      {#if formData.password}
        <FormInput
          id="user-confirmPassword"
          label="Confirmer le mot de passe"
          type="password"
          bind:value={formData.confirmPassword}
          error={errors.confirmPassword}
          required={mode === 'create' || !!formData.password}
          placeholder="••••••••"
          autocomplete="new-password"
        />
      {/if}
    {/if}

    <FormSelect
      id="user-role"
      label="Rôle"
      bind:value={formData.role}
      options={roleOptions}
      required
    />

    {#if requiresOrganization}
      <FormSelect
        id="user-organization"
        label="Organisation"
        bind:value={formData.organizationId}
        options={organizationOptions}
        error={errors.organizationId}
        required
        placeholder={loadingOrgs ? 'Chargement...' : 'Sélectionner une organisation'}
        disabled={loadingOrgs}
      />
    {:else}
      <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-3 text-sm">
        <p class="text-yellow-800">
          ℹ️ Les Super Administrateurs n'appartiennent à aucune organisation et ont accès à tout le système.
        </p>
      </div>
    {/if}
  </form>

  <svelte:fragment slot="footer">
    <div class="flex justify-end space-x-3">
      <Button variant="outline" on:click={handleClose} disabled={loading}>
        Annuler
      </Button>
      <Button variant="primary" on:click={handleSubmit} {loading}>
        {mode === 'create' ? 'Créer l\'utilisateur' : 'Enregistrer les modifications'}
      </Button>
    </div>
  </svelte:fragment>
</Modal>
