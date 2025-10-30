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

  interface RoleFormEntry {
    id: string;
    role: UserRole;
    organizationId: string;
    isPrimary: boolean;
  }

  const generateId = () =>
    typeof crypto !== 'undefined' && crypto.randomUUID
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.random().toString(16).slice(2)}`;

  const createRoleEntry = (
    role: UserRole = UserRole.OWNER,
    organizationId = '',
    isPrimary = false
  ): RoleFormEntry => ({
    id: generateId(),
    role,
    organizationId,
    isPrimary,
  });

  let formData = {
    email: '',
    password: '',
    confirmPassword: '',
    firstName: '',
    lastName: '',
  };

  let formRoles: RoleFormEntry[] = [createRoleEntry(UserRole.OWNER, '', true)];

  let errors = {
    email: '',
    password: '',
    confirmPassword: '',
    firstName: '',
    lastName: '',
    roles: '',
  };

  let organizations: Organization[] = [];
  let organizationOptions: Array<{ value: string; label: string }> = [];
  let loading = false;
  let loadingOrgs = false;
  let currentUserId: string | null = null;

  const roleOptions = [
    { value: UserRole.OWNER, label: 'Copropriétaire' },
    { value: UserRole.ACCOUNTANT, label: 'Comptable' },
    { value: UserRole.SYNDIC, label: 'Syndic (Gestionnaire)' },
    { value: UserRole.SUPERADMIN, label: 'Super Administrateur' },
  ];

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

  function resetForm() {
    formData = {
      email: '',
      password: '',
      confirmPassword: '',
      firstName: '',
      lastName: '',
    };
    formRoles = [createRoleEntry(UserRole.OWNER, '', true)];
    errors = {
      email: '',
      password: '',
      confirmPassword: '',
      firstName: '',
      lastName: '',
      roles: '',
    };
  }

  function populateFormFromUser(existing: User) {
    formData = {
      email: existing.email,
      password: '',
      confirmPassword: '',
      firstName: existing.firstName,
      lastName: existing.lastName,
    };

    const roles = existing.roles ?? [];
    if (roles.length > 0) {
      formRoles = roles.map((role) =>
        createRoleEntry(
          role.role,
          role.organizationId ?? '',
          role.isPrimary
        )
      );
      ensureSinglePrimary();
    } else {
      const fallbackRole = existing.activeRole ?? {
        id: generateId(),
        role: existing.role,
        organizationId: existing.organizationId ?? '',
        isPrimary: true,
      };
      formRoles = [
        createRoleEntry(
          fallbackRole.role,
          fallbackRole.organizationId ?? '',
          true
        ),
      ];
    }

    errors.roles = '';
  }

  function ensureSinglePrimary() {
    if (!formRoles.some((role) => role.isPrimary) && formRoles.length > 0) {
      formRoles = formRoles.map((role, index) => ({
        ...role,
        isPrimary: index === 0,
      }));
    }
  }

  $: if (isOpen) {
    if (mode === 'edit' && user && currentUserId !== user.id) {
      populateFormFromUser(user);
      currentUserId = user.id;
    } else if (mode === 'create' && currentUserId !== null) {
      resetForm();
      currentUserId = null;
    }
  } else if (!isOpen) {
    currentUserId = mode === 'edit' && user ? user.id : null;
  }

  function setPrimaryRole(index: number) {
    formRoles = formRoles.map((role, idx) => ({
      ...role,
      isPrimary: idx === index,
    }));
  }

  function handleRoleChange(index: number, value: string) {
    const roleValue = normalizeRoleValue(value);
    formRoles = formRoles.map((role, idx) => {
      if (idx !== index) return role;
      return {
        ...role,
        role: roleValue,
        organizationId: roleValue === UserRole.SUPERADMIN ? '' : role.organizationId,
      };
    });
  }

  function handleOrganizationChange(index: number, value: string) {
    formRoles = formRoles.map((role, idx) =>
      idx === index ? { ...role, organizationId: value } : role
    );
  }

  function addRoleEntry() {
    formRoles = [
      ...formRoles,
      createRoleEntry(
        UserRole.OWNER,
        '',
        formRoles.length === 0
      ),
    ];
    ensureSinglePrimary();
  }

  function removeRoleEntry(index: number) {
    if (formRoles.length <= 1) {
      return;
    }
    const removedPrimary = formRoles[index].isPrimary;
    formRoles = formRoles.filter((_, idx) => idx !== index);
    if (removedPrimary) {
      setPrimaryRole(0);
    }
  }

  function normalizeRoleValue(value: string): UserRole {
    switch (value) {
      case UserRole.SUPERADMIN:
        return UserRole.SUPERADMIN;
      case UserRole.SYNDIC:
        return UserRole.SYNDIC;
      case UserRole.ACCOUNTANT:
        return UserRole.ACCOUNTANT;
      case UserRole.OWNER:
      default:
        return UserRole.OWNER;
    }
  }

  const validateForm = (): boolean => {
    let isValid = true;
    errors = {
      email: '',
      password: '',
      confirmPassword: '',
      firstName: '',
      lastName: '',
      roles: '',
    };

    // Email validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!formData.email) {
      errors.email = "L'email est requis";
      isValid = false;
    } else if (!emailRegex.test(formData.email)) {
      errors.email = "Format d'email invalide";
      isValid = false;
    }

    // Password validation
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

    // Roles validation
    if (formRoles.length === 0) {
      errors.roles = 'Au moins un rôle doit être attribué';
      isValid = false;
    } else {
      const seen = new Set<string>();
      let primaryCount = 0;
      for (const entry of formRoles) {
        if (entry.role !== UserRole.SUPERADMIN && !entry.organizationId) {
          errors.roles =
            'Choisissez une organisation pour chaque rôle (sauf SuperAdmin)';
          isValid = false;
          break;
        }
        if (entry.isPrimary) {
          primaryCount += 1;
        }
        const key = `${entry.role}-${entry.organizationId || 'none'}`;
        if (seen.has(key)) {
          errors.roles = 'Doublon de rôle détecté';
          isValid = false;
          break;
        }
        seen.add(key);
      }
      if (isValid) {
        if (primaryCount == 0) {
          errors.roles = 'Sélectionnez un rôle principal';
          isValid = false;
        } else if (primaryCount > 1) {
          errors.roles = 'Un seul rôle peut être défini comme principal';
          isValid = false;
        }
      }
    }

    return isValid;
  };

  const handleSubmit = async () => {
    if (!validateForm()) {
      return;
    }

    loading = true;

    try {
      const primary = formRoles.find((role) => role.isPrimary) ?? formRoles[0];
      const rolesPayload = formRoles.map((entry) => ({
        role: entry.role,
        organization_id:
          entry.role === UserRole.SUPERADMIN
            ? null
            : entry.organizationId || null,
        is_primary: entry.isPrimary,
      }));

      const payload: any = {
        email: formData.email.trim(),
        first_name: formData.firstName.trim(),
        last_name: formData.lastName.trim(),
        roles: rolesPayload,
        role: primary.role,
      };

      if (primary.role !== UserRole.SUPERADMIN) {
        payload.organization_id = primary.organizationId || null;
      } else {
        payload.organization_id = null;
      }

      if (mode === 'create') {
        payload.password = formData.password;
        await api.post('/users', payload);
        toast.show('Utilisateur créé avec succès', 'success');
      } else if (user) {
        if (formData.password) {
          payload.password = formData.password;
        }
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
      resetForm();
    }
  };
</script>

<Modal bind:isOpen onClose={handleClose} size="lg" title={mode === 'create' ? 'Créer un utilisateur' : 'Modifier un utilisateur'}>
  <form
    class="space-y-6"
    data-testid="user-form"
    on:submit|preventDefault={handleSubmit}
  >
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <FormInput
        id="email"
        label="Email"
        type="email"
        required
        bind:value={formData.email}
        error={errors.email}
        data-testid="user-email-input"
      />
      <FormInput
        id="firstName"
        label="Prénom"
        required
        bind:value={formData.firstName}
        error={errors.firstName}
        data-testid="user-firstname-input"
      />
      <FormInput
        id="lastName"
        label="Nom"
        required
        bind:value={formData.lastName}
        error={errors.lastName}
        data-testid="user-lastname-input"
      />
      {#if mode === 'create'}
        <FormInput
          id="password"
          label="Mot de passe"
          type="password"
          required
          bind:value={formData.password}
          error={errors.password}
          data-testid="user-password-input"
        />
        <FormInput
          id="confirmPassword"
          label="Confirmation du mot de passe"
          type="password"
          required
          bind:value={formData.confirmPassword}
          error={errors.confirmPassword}
          data-testid="user-confirmpassword-input"
        />
      {:else}
        <FormInput
          id="password"
          label="Nouveau mot de passe (optionnel)"
          type="password"
          bind:value={formData.password}
          error={errors.password}
          data-testid="user-password-input"
        />
        <FormInput
          id="confirmPassword"
          label="Confirmation du mot de passe"
          type="password"
          bind:value={formData.confirmPassword}
          error={errors.confirmPassword}
          data-testid="user-confirmpassword-input"
        />
      {/if}
    </div>

    <div class="border-t border-gray-200 pt-4">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-gray-900">Rôles attribués</h3>
        <Button
          variant="secondary"
          type="button"
          on:click={addRoleEntry}
          data-testid="user-add-role-button"
        >
          ➕ Ajouter un rôle
        </Button>
      </div>
      <p class="text-sm text-gray-500 mt-1">
        Définissez un ou plusieurs rôles. Un unique rôle doit être marqué comme principal.
      </p>
      {#if errors.roles}
        <p class="text-sm text-red-600 mt-2">{errors.roles}</p>
      {/if}

      <div class="space-y-4 mt-4" data-testid="user-roles-container">
        {#each formRoles as roleEntry, index (roleEntry.id)}
          <div
            class="grid grid-cols-1 md:grid-cols-12 gap-4 items-start bg-gray-50 rounded-lg p-4"
            data-testid="user-role-row"
          >
            <div class="md:col-span-4">
              <label class="block text-sm font-medium text-gray-700 mb-1">
                Rôle <span class="text-red-500">*</span>
              </label>
              <select
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                bind:value={roleEntry.role}
                on:change={(event) =>
                  handleRoleChange(index, (event.target as HTMLSelectElement).value)}
                data-testid="user-role-select"
              >
                {#each roleOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>

            <div class="md:col-span-5">
              {#if roleEntry.role === UserRole.SUPERADMIN}
                <p class="text-sm text-gray-600 mt-8">
                  Aucun rattachement d'organisation pour un SuperAdmin.
                </p>
              {:else}
                <label class="block text-sm font-medium text-gray-700 mb-1">
                  Organisation <span class="text-red-500">*</span>
                </label>
                <FormSelect
                  id={`role-org-${index}`}
                  placeholder="Sélectionner une organisation"
                  options={organizationOptions}
                  bind:value={roleEntry.organizationId}
                  disabled={loadingOrgs}
                  data-testid="user-organization-select"
                />
              {/if}
            </div>

            <div class="md:col-span-2 flex items-center">
              <label class="flex items-center space-x-2 text-sm text-gray-700 mt-6">
                <input
                  type="radio"
                  name="primaryRole"
                  checked={roleEntry.isPrimary}
                  on:change={() => setPrimaryRole(index)}
                  data-testid="user-primary-role-radio"
                />
                <span>Rôle principal</span>
              </label>
            </div>

            <div class="md:col-span-1 flex items-center justify-end mt-6">
              {#if formRoles.length > 1}
                <button
                  type="button"
                  class="text-red-600 hover:text-red-800 text-sm"
                  on:click={() => removeRoleEntry(index)}
                  title="Supprimer ce rôle"
                  data-testid="delete-user-role-button"
                >
                  🗑️
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  </form>

  <div slot="footer" class="flex justify-end space-x-3">
    <Button
      variant="secondary"
      on:click={handleClose}
      disabled={loading}
      data-testid="user-cancel-button"
    >
      Annuler
    </Button>
    <Button
      variant="primary"
      on:click={handleSubmit}
      disabled={loading}
      data-testid="user-submit-button"
    >
      {loading
        ? mode === 'create'
          ? 'Création...'
          : 'Enregistrement...'
        : mode === 'create'
        ? 'Créer'
        : 'Mettre à jour'}
    </Button>
  </div>
</Modal>
