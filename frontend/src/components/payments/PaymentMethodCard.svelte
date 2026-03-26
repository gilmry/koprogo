<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    paymentMethodsApi,
    PaymentMethodType,
    type PaymentMethod,
  } from "../../lib/api/payments";
  import { withErrorHandling } from "../../lib/utils/error.utils";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";

  export let paymentMethod: PaymentMethod;
  export let canManage = true;

  const dispatch = createEventDispatcher();

  let showDeleteConfirm = false;
  let actionLoading = false;

  const methodIcons: Record<PaymentMethodType, string> = {
    [PaymentMethodType.Card]: "💳",
    [PaymentMethodType.SepaDebit]: "🏦",
    [PaymentMethodType.BankTransfer]: "🏧",
    [PaymentMethodType.Cash]: "💵",
  };

  function getIcon(type: PaymentMethodType): string {
    return methodIcons[type] || "💳";
  }

  function formatExpiryDate(expiresAt: string | undefined): string {
    if (!expiresAt) return "";
    const date = new Date(expiresAt);
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const year = String(date.getFullYear()).slice(-2);
    return `${month}/${year}`;
  }

  async function handleSetDefault() {
    const result = await withErrorHandling({
      action: () => paymentMethodsApi.setAsDefault(paymentMethod.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('payments.setDefault'),
      errorMessage: $_('payments.failedSetDefault'),
    });
    if (result !== undefined) dispatch("updated");
  }

  async function handleToggleActive() {
    const result = await withErrorHandling({
      action: () => paymentMethod.is_active
        ? paymentMethodsApi.deactivate(paymentMethod.id)
        : paymentMethodsApi.reactivate(paymentMethod.id),
      setLoading: (v) => actionLoading = v,
      successMessage: paymentMethod.is_active ? $_('payments.deactivated') : $_('payments.reactivated'),
      errorMessage: $_('payments.failedUpdate'),
    });
    if (result !== undefined) dispatch("updated");
  }

  async function handleDelete() {
    const result = await withErrorHandling({
      action: () => paymentMethodsApi.delete(paymentMethod.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('payments.deleted'),
      errorMessage: $_('payments.failedDelete'),
    });
    if (result !== undefined) {
      showDeleteConfirm = false;
      dispatch("deleted");
    } else {
      showDeleteConfirm = false;
    }
  }
</script>

<div
  data-testid="payment-method-card"
  class="relative bg-white border rounded-lg p-4 hover:shadow-md transition-shadow {!paymentMethod.is_active
    ? 'opacity-50'
    : ''}"
>
  <!-- Default badge -->
  {#if paymentMethod.is_default}
    <div class="absolute top-2 right-2">
      <span
        class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
      >
        ⭐ {$_('payments.default')}
      </span>
    </div>
  {/if}

  <!-- Method Icon and Type -->
  <div class="flex items-start space-x-3">
    <div class="text-3xl">{getIcon(paymentMethod.method_type)}</div>

    <div class="flex-1 min-w-0">
      <h3 class="text-lg font-medium text-gray-900">
        {paymentMethod.display_label}
      </h3>

      <div class="mt-1 space-y-1">
        {#if paymentMethod.method_type === PaymentMethodType.Card}
          <p class="text-sm text-gray-600">
            {paymentMethod.brand || $_('payments.card')} •••• {paymentMethod.last4 || "****"}
          </p>
          {#if paymentMethod.expires_at}
            <p class="text-sm text-gray-500" data-testid="payment-method-expiry">
              {$_('payments.expires')}: {formatExpiryDate(paymentMethod.expires_at)}
            </p>
          {/if}
        {:else if paymentMethod.method_type === PaymentMethodType.SepaDebit}
          <p class="text-sm text-gray-600">
            {$_('payments.iban')} •••• {paymentMethod.last4 || "****"}
          </p>
        {:else}
          <p class="text-sm text-gray-600">{paymentMethod.method_type}</p>
        {/if}

        <p class="text-xs text-gray-500">
          {$_('payments.added')}: {new Date(paymentMethod.created_at).toLocaleDateString("nl-BE")}
        </p>
      </div>

      <!-- Actions -->
      {#if canManage}
        <div class="mt-3 flex flex-wrap gap-2">
          {#if !paymentMethod.is_default && paymentMethod.is_active}
            <button
              on:click={handleSetDefault}
              disabled={actionLoading}
              data-testid="set-default-btn"
              class="text-sm text-blue-600 hover:text-blue-700 font-medium disabled:opacity-50"
            >
              {$_('payments.setDefault')}
            </button>
          {/if}

          <button
            on:click={handleToggleActive}
            disabled={actionLoading}
            data-testid="toggle-active-btn"
            class="text-sm text-gray-600 hover:text-gray-700 font-medium disabled:opacity-50"
          >
            {paymentMethod.is_active ? $_('payments.deactivate') : $_('payments.reactivate')}
          </button>

          {#if !paymentMethod.is_default}
            <button
              on:click={() => (showDeleteConfirm = true)}
              disabled={actionLoading}
              data-testid="delete-method-btn"
              class="text-sm text-red-600 hover:text-red-700 font-medium disabled:opacity-50"
            >
              {$_('common.delete')}
            </button>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<!-- Delete Confirmation -->
<ConfirmDialog
  open={showDeleteConfirm}
  title={$_('payments.deleteTitle')}
  message={$_('payments.deleteConfirm')}
  confirmText={$_('common.delete')}
  confirmVariant="danger"
  on:confirm={handleDelete}
  on:cancel={() => (showDeleteConfirm = false)}
/>
