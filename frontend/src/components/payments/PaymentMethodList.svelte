<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    paymentMethodsApi,
    type PaymentMethod,
  } from "../../lib/api/payments";
  import { withLoadingState } from "../../lib/utils/error.utils";
  import PaymentMethodCard from "./PaymentMethodCard.svelte";
  import PaymentMethodAddModal from "./PaymentMethodAddModal.svelte";
  import Button from "../ui/Button.svelte";

  export let ownerId: string;
  export let canManage = true;

  let paymentMethods: PaymentMethod[] = [];
  let loading = true;
  let error = "";
  let showAddModal = false;

  onMount(async () => {
    await loadPaymentMethods();
  });

  async function loadPaymentMethods() {
    await withLoadingState({
      action: () => paymentMethodsApi.listByOwner(ownerId),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => paymentMethods = data,
      errorMessage: $_('payments.loadMethodsError'),
    });
  }

  function handleAdded() {
    showAddModal = false;
    loadPaymentMethods();
  }
</script>

<div class="bg-white shadow rounded-lg" data-testid="payment-method-list">
  <!-- Header -->
  <div class="px-6 py-4 border-b border-gray-200">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold text-gray-900">{$_('payments.methodsTitle')}</h2>
        <p class="mt-1 text-sm text-gray-600">
          {$_('payments.methodsDescription')}
        </p>
      </div>
      {#if canManage}
        <Button on:click={() => (showAddModal = true)} data-testid="add-payment-method-btn">
          {$_('payments.addMethod')}
        </Button>
      {/if}
    </div>
  </div>

  <!-- Payment Methods Grid -->
  <div class="p-6">
    {#if loading}
      <div class="text-center py-12 text-gray-500" data-testid="payment-method-list-loading">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
        ></div>
        <p class="mt-4">{$_('common.loading')}</p>
      </div>
    {:else if paymentMethods.length === 0}
      <div class="text-center py-12">
        <svg
          class="mx-auto h-16 w-16 text-gray-400"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z"
          />
        </svg>
        <h3 class="mt-4 text-lg font-medium text-gray-900">
          {$_('payments.noMethods')}
        </h3>
        <p class="mt-2 text-sm text-gray-600">
          {$_('payments.noMethodsDescription')}
        </p>
        {#if canManage}
          <div class="mt-6">
            <Button on:click={() => (showAddModal = true)}>
              {$_('payments.addFirstMethod')}
            </Button>
          </div>
        {/if}
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each paymentMethods as paymentMethod (paymentMethod.id)}
          <PaymentMethodCard
            {paymentMethod}
            {canManage}
            on:updated={loadPaymentMethods}
            on:deleted={loadPaymentMethods}
          />
        {/each}
      </div>

      <!-- Info -->
      <div class="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <div class="flex">
          <div class="flex-shrink-0">
            <svg
              class="h-5 w-5 text-blue-400"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fill-rule="evenodd"
                d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                clip-rule="evenodd"
              />
            </svg>
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-blue-800">
              {$_('payments.secureProcessingTitle')}
            </h3>
            <div class="mt-2 text-sm text-blue-700">
              <p>
                {$_('payments.secureProcessingDescription')}
              </p>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Add Payment Method Modal -->
<PaymentMethodAddModal
  bind:open={showAddModal}
  {ownerId}
  on:added={handleAdded}
/>
