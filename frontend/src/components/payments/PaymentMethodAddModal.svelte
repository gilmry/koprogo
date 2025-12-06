<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    paymentMethodsApi,
    PaymentMethodType,
    type CreatePaymentMethodDto,
  } from "../../lib/api/payments";
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

  function validate(): boolean {
    errors = {};

    if (!formData.display_label || formData.display_label.trim().length < 3) {
      errors.display_label = "Label must be at least 3 characters";
    }

    if (
      formData.method_type === PaymentMethodType.Card ||
      formData.method_type === PaymentMethodType.SepaDebit
    ) {
      if (
        !formData.stripe_payment_method_id ||
        !formData.stripe_payment_method_id.startsWith("pm_")
      ) {
        errors.stripe_payment_method_id =
          "Valid Stripe Payment Method ID required (starts with pm_)";
      }
    }

    return Object.keys(errors).length === 0;
  }

  async function handleSubmit() {
    if (!validate()) {
      toast.error("Please fix validation errors");
      return;
    }

    try {
      submitting = true;

      await paymentMethodsApi.create({
        ...formData,
        owner_id: ownerId,
      });

      toast.success("Payment method added successfully");

      dispatch("added");
      handleClose();
    } catch (err: any) {
      toast.error(err.message || "Failed to add payment method");
    } finally {
      submitting = false;
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
    // Reset Stripe-specific fields when changing type
    formData.stripe_payment_method_id = "";
    formData.last4 = "";
    formData.brand = "";
  }
</script>

<Modal {open} on:close={handleClose} title="Add Payment Method">
  <form on:submit|preventDefault={handleSubmit}>
    <div class="space-y-4">
      <!-- Info Banner -->
      <div class="bg-blue-50 border border-blue-200 rounded-lg p-3">
        <p class="text-sm text-blue-700">
          💡 <strong>Note:</strong> In a production environment, this form would
          integrate with Stripe Elements to securely collect card details. For now,
          you need a Stripe Payment Method ID (pm_xxx) obtained via Stripe's API.
        </p>
      </div>

      <!-- Method Type -->
      <FormSelect
        label="Payment Method Type"
        bind:value={formData.method_type}
        on:change={handleMethodTypeChange}
        required
      >
        <option value={PaymentMethodType.Card}>💳 Credit/Debit Card</option>
        <option value={PaymentMethodType.SepaDebit}>🏦 SEPA Direct Debit</option>
        <option value={PaymentMethodType.BankTransfer}>🏧 Bank Transfer</option>
        <option value={PaymentMethodType.Cash}>💵 Cash</option>
      </FormSelect>

      <!-- Display Label -->
      <FormInput
        label="Display Name"
        bind:value={formData.display_label}
        error={errors.display_label}
        required
        placeholder="e.g., My Visa Card, Company SEPA Account"
      />

      <!-- Stripe-specific fields for Card and SEPA -->
      {#if formData.method_type === PaymentMethodType.Card || formData.method_type === PaymentMethodType.SepaDebit}
        <FormInput
          label="Stripe Payment Method ID"
          bind:value={formData.stripe_payment_method_id}
          error={errors.stripe_payment_method_id}
          required
          placeholder="pm_xxxxxxxxxxxxx"
        />

        {#if formData.method_type === PaymentMethodType.Card}
          <div class="grid grid-cols-2 gap-4">
            <FormInput
              label="Card Brand (Optional)"
              bind:value={formData.brand}
              placeholder="Visa, Mastercard, etc."
            />
            <FormInput
              label="Last 4 Digits (Optional)"
              bind:value={formData.last4}
              placeholder="1234"
              maxlength={4}
            />
          </div>
        {:else}
          <FormInput
            label="Last 4 IBAN Digits (Optional)"
            bind:value={formData.last4}
            placeholder="1234"
            maxlength={4}
          />
        {/if}
      {/if}

      <!-- Help Text -->
      {#if formData.method_type === PaymentMethodType.BankTransfer || formData.method_type === PaymentMethodType.Cash}
        <div class="text-sm text-gray-600">
          <p>
            {#if formData.method_type === PaymentMethodType.BankTransfer}
              Bank transfers will be processed manually by the syndic.
            {:else}
              Cash payments will be recorded manually by the syndic.
            {/if}
          </p>
        </div>
      {/if}
    </div>

    <!-- Actions -->
    <div class="mt-6 flex justify-end space-x-3">
      <Button type="button" variant="outline" on:click={handleClose}>
        Cancel
      </Button>
      <Button type="submit" loading={submitting}>
        Add Payment Method
      </Button>
    </div>
  </form>
</Modal>
