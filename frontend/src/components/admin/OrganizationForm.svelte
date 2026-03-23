<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { _ } from '../../lib/i18n';
  import { toast } from '../../stores/toast';
  import { api } from '../../lib/api';
  import type { Organization, SubscriptionPlan } from '../../lib/types';
  import Modal from '../ui/Modal.svelte';
  import FormInput from '../ui/FormInput.svelte';
  import FormSelect from '../ui/FormSelect.svelte';
  import Button from '../ui/Button.svelte';

  export let isOpen = false;
  export let organization: Organization | null = null;
  export let mode: 'create' | 'edit' = 'create';

  const dispatch = createEventDispatcher();

  let formData = {
    name: '',
    slug: '',
    contact_email: '',
    contact_phone: '',
    subscription_plan: 'free' as SubscriptionPlan,
  };

  let errors = {
    name: '',
    slug: '',
    contact_email: '',
    contact_phone: '',
  };

  let loading = false;

  const subscriptionOptions = [
    { value: 'free', label: $_('admin.organization.planFree') },
    { value: 'starter', label: $_('admin.organization.planStarter') },
    { value: 'professional', label: $_('admin.organization.planProfessional') },
    { value: 'enterprise', label: $_('admin.organization.planEnterprise') },
  ];

  // Initialize form with organization data if editing
  $: if (organization && mode === 'edit') {
    formData = {
      name: organization.name,
      slug: organization.slug,
      contact_email: organization.contact_email,
      contact_phone: organization.contact_phone || '',
      subscription_plan: organization.subscription_plan,
    };
  }

  // Auto-generate slug from name
  const generateSlug = () => {
    if (formData.name && !organization) {
      formData.slug = formData.name
        .toLowerCase()
        .normalize('NFD')
        .replace(/[\u0300-\u036f]/g, '')
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '');
    }
  };

  const validateForm = (): boolean => {
    let isValid = true;
    errors = {
      name: '',
      slug: '',
      contact_email: '',
      contact_phone: '',
    };

    // Name validation
    if (!formData.name || formData.name.trim().length < 2) {
      errors.name = $_('admin.organization.nameError');
      isValid = false;
    }

    // Slug validation
    if (!formData.slug || formData.slug.trim().length < 2) {
      errors.slug = $_('admin.organization.slugMinError');
      isValid = false;
    } else if (!/^[a-z0-9-]+$/.test(formData.slug)) {
      errors.slug = $_('admin.organization.slugFormatError');
      isValid = false;
    }

    // Email validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!formData.contact_email) {
      errors.contact_email = $_('admin.organization.emailRequired');
      isValid = false;
    } else if (!emailRegex.test(formData.contact_email)) {
      errors.contact_email = $_('admin.organization.emailFormatError');
      isValid = false;
    }

    // Phone validation (optional but if provided, must be valid)
    if (formData.contact_phone) {
      const phoneRegex = /^\+?[0-9\s\-()]{8,}$/;
      if (!phoneRegex.test(formData.contact_phone)) {
        errors.contact_phone = $_('admin.organization.phoneFormatError');
        isValid = false;
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
      const payload: any = {
        name: formData.name.trim(),
        slug: formData.slug.trim(),
        contact_email: formData.contact_email.trim(),
        subscription_plan: formData.subscription_plan,
      };

      if (formData.contact_phone) {
        payload.contact_phone = formData.contact_phone.trim();
      }

      if (mode === 'create') {
        await api.post('/organizations', payload);
        toast.show($_('admin.organization.createdSuccessfully'), 'success');
      } else if (organization) {
        await api.put(`/organizations/${organization.id}`, payload);
        toast.show($_('admin.organization.updatedSuccessfully'), 'success');
      }

      loading = false;
      handleClose();
      dispatch('success');
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Une erreur est survenue';

      if (errorMessage.includes('slug')) {
        errors.slug = $_('admin.organization.slugAlreadyUsed');
      } else if (errorMessage.includes('email')) {
        errors.contact_email = $_('admin.organization.emailAlreadyUsed');
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
        name: '',
        slug: '',
        contact_email: '',
        contact_phone: '',
        subscription_plan: 'free',
      };
      errors = {
        name: '',
        slug: '',
        contact_email: '',
        contact_phone: '',
      };
      dispatch('close');
    }
  };
</script>

<Modal
  {isOpen}
  title={mode === 'create' ? $_('admin.organization.newOrganization') : $_('admin.organization.editOrganization')}
  size="md"
  on:close={handleClose}
>
  <form on:submit|preventDefault={handleSubmit} class="space-y-4" data-testid="organization-form">
    <FormInput
      id="org-name"
      label={$_('admin.organization.name')}
      type="text"
      bind:value={formData.name}
      on:blur={generateSlug}
      error={errors.name}
      required
      placeholder="Résidence Grand Place SPRL"
      data-testid="organization-name-input"
    />

    <FormInput
      id="org-slug"
      label="Slug (URL)"
      type="text"
      bind:value={formData.slug}
      error={errors.slug}
      required
      placeholder="residence-grand-place"
      hint={$_('admin.organization.slugHint')}
      data-testid="organization-slug-input"
    />

    <FormInput
      id="org-email"
      label={$_('admin.organization.contactEmail')}
      type="email"
      bind:value={formData.contact_email}
      error={errors.contact_email}
      required
      placeholder="contact@residence-grand-place.be"
      data-testid="organization-email-input"
    />

    <FormInput
      id="org-phone"
      label={$_('admin.organization.contactPhone')}
      type="tel"
      bind:value={formData.contact_phone}
      error={errors.contact_phone}
      placeholder="+32 2 123 45 67"
      data-testid="organization-phone-input"
    />

    <FormSelect
      id="org-plan"
      label={$_('admin.organization.subscriptionPlan')}
      bind:value={formData.subscription_plan}
      options={subscriptionOptions}
      required
    />

    <div class="bg-gray-50 p-4 rounded-lg text-sm">
      <p class="font-medium text-gray-700 mb-2">{$_('admin.organization.planLimits')}:</p>
      <ul class="text-gray-600 space-y-1">
        {#if formData.subscription_plan === 'free'}
          <li>• {$_('admin.organization.limitBuildings1')}</li>
          <li>• {$_('admin.organization.limitUsers3')}</li>
        {:else if formData.subscription_plan === 'starter'}
          <li>• {$_('admin.organization.limitBuildings5')}</li>
          <li>• {$_('admin.organization.limitUsers10')}</li>
        {:else if formData.subscription_plan === 'professional'}
          <li>• {$_('admin.organization.limitBuildings20')}</li>
          <li>• {$_('admin.organization.limitUsers50')}</li>
        {:else if formData.subscription_plan === 'enterprise'}
          <li>• {$_('admin.organization.limitBuildingsUnlimited')}</li>
          <li>• {$_('admin.organization.limitUsersUnlimited')}</li>
        {/if}
      </ul>
    </div>
  </form>

  <svelte:fragment slot="footer">
    <div class="flex justify-end space-x-3">
      <Button variant="outline" on:click={handleClose} disabled={loading} data-testid="organization-cancel-button">
        {$_('common.cancel')}
      </Button>
      <Button variant="primary" on:click={handleSubmit} {loading} data-testid="organization-submit-button">
        {mode === 'create' ? $_('admin.organization.createOrganization') : $_('common.saveChanges')}
      </Button>
    </div>
  </svelte:fragment>
</Modal>
