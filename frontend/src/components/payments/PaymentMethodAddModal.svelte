<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    paymentMethodsApi,
    PaymentMethodType,
    type CreatePaymentMethodDto,
  } from "../../lib/api/payments";
  import { withErrorHandling } from "../../lib/utils/error.utils";
  import { validatePaymentMethod } from "../../lib/validators/payment.validators";
  import { toast } from "../../stores/toast";
  import Modal from "../ui/Modal.svelte";
  import FormInput from "../ui/FormInput.svelte";
  import FormSelect from "../ui/FormSelect.svelte";
  import Button from "../ui/Button.svelte";

  export let open = false;
  export let ownerId: string;

  const dispatch = createEventDispatcher();

  let formData: CreatePaymentMethodDto = {
    owner_id: ownerId,
    method_type: PaymentMethodType.Card,
    display_label: "",
    stripe_payment_method_id: "",
    last4: "",
    brand: "",
  };

  let submitting = false;
  let errors: Record<string, string> = {};

  async function handleSubmit() {
    errors = validatePaymentMethod(formData, {
      labelMinLength: $_('payments.validation.labelMinLength'),
      stripeIdRequired: $_('payments.validation.stripeIdRequired'),
    });

    if (Object.keys(errors).length > 0) {
      toast.error($_('payments.validation.fixErrors'));
      return;
    }

    const result = await withErrorHandling({
      action: () => paymentMethodsApi.create({
        ...formData,
        owner_id: ownerId,
      }),
      setLoading: (v) => submitting = v,
      successMessage: $_('payments.methodAdded'),
      errorMessage: $_('payments.failedAddMethod'),
    });

    if (result !== undefined) {
      dispatch("added");
      handleClose();
    }
  }

  function handleClose() {
    open = false;
    formData = {
      owner_id: ownerId,
      method_type: PaymentMethodType.Card,
      display_label: "",
      stripe_payment_method_id: "",
      last4: "",
      brand: "",
    };
    errors = {};
    dispatch("close");
  }

  function handleMethodTypeChange() {
    formData.stripe_payment_method_id = "";
    formData.last4 = "";
    formData.brand = "";
  }
</script>

<Modal isOpen={open} on:close={handleClose} title={$_('payments.addMethodTitle')}>
  <form on:submit|preventDefault={handleSubmit}>
    <div class="space-y-4">
      <!-- Info Banner -->
      <div class="bg-blue-50 border border-blue-200 rounded-lg p-3">
        <p class="text-sm text-blue-700">
          {$_('payments.stripeNote')}
        </p>
      </div>

      <!-- Method Type -->
      <FormSelect
        id="method-type"
        label={$_('payments.methodType')}
        bind:value={formData.method_type}
        on:change={handleMethodTypeChange}
        required
        data-testid="method-type-select"
      >
        <option value={PaymentMethodType.Card}>{$_('payments.typeCard')}</option>
        <option value={PaymentMethodType.SepaDebit}>{$_('payments.typeSepa')}</option>
        <option value={PaymentMethodType.BankTransfer}>{$_('payments.typeBankTransfer')}</option>
        <option value={PaymentMethodType.Cash}>{$_('payments.typeCash')}</option>
      </FormSelect>

      <!-- Display Label -->
      <FormInput
        id="display-label"
        label={$_('payments.displayName')}
        bind:value={formData.display_label}
        error={errors.display_label}
        required
        placeholder={$_('payments.displayNamePlaceholder')}
        data-testid="display-label-input"
      />

      <!-- Stripe-specific fields for Card and SEPA -->
      {#if formData.method_type === PaymentMethodType.Card || formData.method_type === PaymentMethodType.SepaDebit}
        <FormInput
          id="stripe-id"
          label={$_('payments.stripeMethodId')}
          bind:value={formData.stripe_payment_method_id}
          error={errors.stripe_payment_method_id}
          required
          placeholder="pm_xxxxxxxxxxxxx"
          data-testid="stripe-id-input"
        />

        {#if formData.method_type === PaymentMethodType.Card}
          <div class="grid grid-cols-2 gap-4">
            <FormInput
              id="brand"
              label={$_('payments.cardBrand')}
              bind:value={formData.brand}
              placeholder={$_('payments.cardBrandPlaceholder')}
              data-testid="brand-input"
            />
            <FormInput
              id="last4"
              label={$_('payments.last4')}
              bind:value={formData.last4}
              placeholder="1234"
              maxlength={4}
              data-testid="last4-input"
            />
          </div>
        {:else}
          <FormInput
            id="last4-iban"
            label={$_('payments.last4Iban')}
            bind:value={formData.last4}
            placeholder="1234"
            maxlength={4}
            data-testid="last4-input"
          />
        {/if}
      {/if}

      <!-- Help Text -->
      {#if formData.method_type === PaymentMethodType.BankTransfer || formData.method_type === PaymentMethodType.Cash}
        <div class="text-sm text-gray-600">
          <p>
            {#if formData.method_type === PaymentMethodType.BankTransfer}
              {$_('payments.bankTransferHelp')}
            {:else}
              {$_('payments.cashHelp')}
            {/if}
          </p>
        </div>
      {/if}
    </div>

    <!-- Actions -->
    <div class="mt-6 flex justify-end space-x-3">
      <Button type="button" variant="outline" on:click={handleClose} data-testid="cancel-btn">
        {$_('common.cancel')}
      </Button>
      <Button type="submit" loading={submitting} data-testid="submit-btn">
        {$_('payments.addMethod')}
      </Button>
    </div>
  </form>
</Modal>
