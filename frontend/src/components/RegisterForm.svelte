<script lang="ts">
  import { authStore, mapUserFromBackend } from '../stores/auth';
  import { toast } from '../stores/toast';
  import { UserRole } from '../lib/types';
  import type { User } from '../lib/types';
  import { apiEndpoint } from '../lib/config';
  import FormInput from './ui/FormInput.svelte';
  import FormSelect from './ui/FormSelect.svelte';
  import Button from './ui/Button.svelte';

  let formData = {
    email: '',
    password: '',
    confirmPassword: '',
    first_name: '',
    last_name: '',
    role: UserRole.OWNER,
    organizationId: '',
  };

  let errors = {
    email: '',
    password: '',
    confirmPassword: '',
    first_name: '',
    last_name: '',
  };

  let loading = false;
  let showOrgField = false;

  const roleOptions = [
    { value: UserRole.OWNER, label: 'Copropriétaire' },
    { value: UserRole.SYNDIC, label: 'Syndic' },
    { value: UserRole.ACCOUNTANT, label: 'Comptable' },
  ];

  $: showOrgField = formData.role !== UserRole.OWNER;

  const validateForm = (): boolean => {
    let isValid = true;
    errors = {
      email: '',
      password: '',
      confirmPassword: '',
      first_name: '',
      last_name: '',
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

    // Password validation
    if (!formData.password) {
      errors.password = 'Le mot de passe est requis';
      isValid = false;
    } else if (formData.password.length < 6) {
      errors.password = 'Le mot de passe doit contenir au moins 6 caractères';
      isValid = false;
    }

    // Confirm password validation
    if (!formData.confirmPassword) {
      errors.confirmPassword = 'Veuillez confirmer le mot de passe';
      isValid = false;
    } else if (formData.password !== formData.confirmPassword) {
      errors.confirmPassword = 'Les mots de passe ne correspondent pas';
      isValid = false;
    }

    // First name validation
    if (!formData.first_name || formData.first_name.trim().length < 2) {
      errors.first_name = 'Le prénom doit contenir au moins 2 caractères';
      isValid = false;
    }

    // Last name validation
    if (!formData.last_name || formData.last_name.trim().length < 2) {
      errors.last_name = 'Le nom doit contenir au moins 2 caractères';
      isValid = false;
    }

    return isValid;
  };

  const handleRegister = async (e: Event) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    loading = true;

    try {
      const requestBody: any = {
        email: formData.email,
        password: formData.password,
        first_name: formData.first_name,
        last_name: formData.last_name,
        role: formData.role,
      };

      // Add organization_id if applicable
      if (showOrgField && formData.organizationId) {
        requestBody.organization_id = formData.organizationId;
      }

      const response = await fetch(apiEndpoint('/auth/register'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(requestBody),
      });

      if (response.ok) {
        const data = await response.json();
        const { token, refresh_token, user } = data;

        // Map backend user format to frontend format
        const mappedUser: User = mapUserFromBackend(user);

        // Login with token, refresh token and initialize sync
        await authStore.login(mappedUser, token, refresh_token);

        toast.show('Compte créé avec succès! Bienvenue sur KoproGo.', 'success');

        // Redirect based on role
        const redirectMap = {
          [UserRole.SUPERADMIN]: '/admin',
          [UserRole.SYNDIC]: '/syndic',
          [UserRole.ACCOUNTANT]: '/accountant',
          [UserRole.OWNER]: '/owner',
        };

        setTimeout(() => {
          window.location.href = redirectMap[mappedUser.role] || '/';
        }, 1000);
      } else {
        const errorData = await response.json();
        const errorMessage = errorData.error || 'Une erreur est survenue lors de l\'inscription';

        // Check for specific validation errors
        if (errorMessage.includes('email')) {
          errors.email = 'Cet email est déjà utilisé';
        } else {
          toast.show(errorMessage, 'error');
        }
      }
    } catch (e) {
      console.error('Registration error:', e);
      toast.show(
        'Une erreur est survenue lors de l\'inscription. Vérifiez que le serveur est démarré.',
        'error'
      );
    } finally {
      loading = false;
    }
  };
</script>

<form on:submit={handleRegister} class="space-y-4" data-testid="register-form">
  <div class="grid grid-cols-2 gap-4">
    <FormInput
      id="first_name"
      label="Prénom"
      type="text"
      bind:value={formData.first_name}
      error={errors.first_name}
      required
      placeholder="Jean"
      data-testid="register-first-name"
    />

    <FormInput
      id="last_name"
      label="Nom"
      type="text"
      bind:value={formData.last_name}
      error={errors.last_name}
      required
      placeholder="Dupont"
      data-testid="register-last-name"
    />
  </div>

  <FormInput
    id="email"
    label="Email"
    type="email"
    bind:value={formData.email}
    error={errors.email}
    required
    placeholder="jean.dupont@example.com"
    autocomplete="email"
    data-testid="register-email"
  />

  <FormInput
    id="password"
    label="Mot de passe"
    type="password"
    bind:value={formData.password}
    error={errors.password}
    required
    placeholder="••••••••"
    hint="Au moins 6 caractères"
    autocomplete="new-password"
    data-testid="register-password"
  />

  <FormInput
    id="confirmPassword"
    label="Confirmer le mot de passe"
    type="password"
    bind:value={formData.confirmPassword}
    error={errors.confirmPassword}
    required
    placeholder="••••••••"
    autocomplete="new-password"
    data-testid="register-confirm-password"
  />

  <FormSelect
    id="role"
    label="Type de compte"
    bind:value={formData.role}
    options={roleOptions}
    required
    data-testid="register-role"
  />

  {#if showOrgField}
    <FormInput
      id="organizationId"
      label="ID Organisation (optionnel)"
      type="text"
      bind:value={formData.organizationId}
      hint="Laissez vide si vous créez une nouvelle organisation"
      placeholder="550e8400-e29b-41d4-a716-446655440000"
      data-testid="register-org-id"
    />
  {/if}

  <Button
    type="submit"
    {loading}
    fullWidth
    variant="primary"
    size="lg"
    data-testid="register-submit"
  >
    {loading ? 'Création du compte...' : 'Créer mon compte'}
  </Button>

  <p class="text-xs text-gray-500 text-center mt-4">
    En créant un compte, vous acceptez nos conditions d'utilisation et notre politique de confidentialité.
  </p>
</form>
