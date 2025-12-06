<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    paymentMethodsApi,
    PaymentMethodType,
    type PaymentMethod,
  } from "../../lib/api/payments";
  import { toast } from "../../stores/toast";
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
    try {
      actionLoading = true;
      await paymentMethodsApi.setAsDefault(paymentMethod.id);
      toast.success("Payment method set as default");
      dispatch("updated");
    } catch (err: any) {
      toast.error(err.message || "Failed to set as default");
    } finally {
      actionLoading = false;
    }
  }

  async function handleToggleActive() {
    try {
      actionLoading = true;
      if (paymentMethod.is_active) {
        await paymentMethodsApi.deactivate(paymentMethod.id);
        toast.success("Payment method deactivated");
      } else {
        await paymentMethodsApi.reactivate(paymentMethod.id);
        toast.success("Payment method reactivated");
      }
      dispatch("updated");
    } catch (err: any) {
      toast.error(err.message || "Failed to update payment method");
    } finally {
      actionLoading = false;
    }
  }

  async function handleDelete() {
    try {
      actionLoading = true;
      await paymentMethodsApi.delete(paymentMethod.id);
      toast.success("Payment method deleted");
      dispatch("deleted");
    } catch (err: any) {
      toast.error(err.message || "Failed to delete payment method");
    } finally {
      actionLoading = false;
      showDeleteConfirm = false;
    }
  }
</script>

<div
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
        ⭐ Default
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
            {paymentMethod.brand || "Card"} •••• {paymentMethod.last4 || "****"}
          </p>
          {#if paymentMethod.expires_at}
            <p class="text-sm text-gray-500">
              Expires: {formatExpiryDate(paymentMethod.expires_at)}
            </p>
          {/if}
        {:else if paymentMethod.method_type === PaymentMethodType.SepaDebit}
          <p class="text-sm text-gray-600">
            IBAN •••• {paymentMethod.last4 || "****"}
          </p>
        {:else}
          <p class="text-sm text-gray-600">{paymentMethod.method_type}</p>
        {/if}

        <p class="text-xs text-gray-500">
          Added: {new Date(paymentMethod.created_at).toLocaleDateString("nl-BE")}
        </p>
      </div>

      <!-- Actions -->
      {#if canManage}
        <div class="mt-3 flex flex-wrap gap-2">
          {#if !paymentMethod.is_default && paymentMethod.is_active}
            <button
              on:click={handleSetDefault}
              disabled={actionLoading}
              class="text-sm text-blue-600 hover:text-blue-700 font-medium disabled:opacity-50"
            >
              Set as Default
            </button>
          {/if}

          <button
            on:click={handleToggleActive}
            disabled={actionLoading}
            class="text-sm text-gray-600 hover:text-gray-700 font-medium disabled:opacity-50"
          >
            {paymentMethod.is_active ? "Deactivate" : "Reactivate"}
          </button>

          {#if !paymentMethod.is_default}
            <button
              on:click={() => (showDeleteConfirm = true)}
              disabled={actionLoading}
              class="text-sm text-red-600 hover:text-red-700 font-medium disabled:opacity-50"
            >
              Delete
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
  title="Delete Payment Method"
  message="Are you sure you want to delete this payment method? This action cannot be undone."
  confirmText="Delete"
  confirmVariant="danger"
  on:confirm={handleDelete}
  on:cancel={() => (showDeleteConfirm = false)}
/>
