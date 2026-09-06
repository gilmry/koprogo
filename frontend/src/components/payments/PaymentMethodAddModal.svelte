<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../../lib/i18n';
  import {
    paymentMethodsApi,
    StoredPaymentMethodType,
    type CreatePaymentMethodDto,
  } from "../../lib/api/payments";
  import { withErrorHandling } from "../../lib/utils/error.utils";
  import { validatePaymentMethod } from "../../lib/validators/payment.validators";
  import { toast } from "../../stores/toast";
  import Modal from "../ui/Modal.svelte";
  import FormInput from "../ui/FormInput.svelte";
  import FormSelect from "../ui/FormSelect.svelte";
  import Button from "../ui/Button.svelte";

  let { open = $bindable(false), ownerId, onadded }: {
    open?: boolean;
    ownerId: string;
    onadded?: () => void;
  } = $props();

  // `stripe_customer_id` et `is_default` sont requis par le serveur et
  // manquaient a ce formulaire : chaque envoi produisait un 400 (#732).
  // `last4` et `brand` n'existent pas cote serveur et ont ete retires.
  let formData: CreatePaymentMethodDto = $state({
    owner_id: "",
    method_type: StoredPaymentMethodType.Card,
    display_label: "",
    stripe_payment_method_id: "",
    stripe_customer_id: "",
    is_default: false,
  });
  // Sync with prop (live value via $effect, not stale initial capture)
  $effect(() => { if (ownerId && !formData.owner_id) formData.owner_id = ownerId; });

  let submitting = $state(false);
  let errors: Record<string, string> = $state({});

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
      setLoading: (v: boolean) => submitting = v,
      successMessage: $_('payments.methodAdded'),
      errorMessage: $_('payments.failedAddMethod'),
    });

    if (result !== undefined) {
      onadded?.();
      handleClose();
    }
  }

  function handleClose() {
    open = false;
    formData = {
      owner_id: ownerId,
      method_type: StoredPaymentMethodType.Card,
      display_label: "",
      stripe_payment_method_id: "",
      stripe_customer_id: "",
      is_default: false,
    };
    errors = {};
  }

  function handleMethodTypeChange() {
    formData.stripe_payment_method_id = "";
  }
</script>

<Modal isOpen={open} onclose={handleClose} title={$_('payments.addMethodTitle')}>
  <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
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
        onchange={handleMethodTypeChange}
        required
        data-testid="method-type-select"
      >
        <!--
          Deux options seulement. « Virement » et « Espèces » figuraient ici
          jusqu'au 2026-09-06 : les choisir produisait un 400, parce que
          `CreatePaymentMethodRequest.method_type` n'a jamais accepté que
          `card` et `sepa_debit`, et que l'endpoint exige de surcroît un
          `stripe_payment_method_id` et un `stripe_customer_id`.

          On n'enregistre pas du liquide. Ce que le produit voudra peut-être
          offrir — qu'un copropriétaire DÉCLARE payer par virement, sans
          instrument conservé — est une fonctionnalité à concevoir, pas deux
          lignes à laisser dans un select. Voir #819.
        -->
        <option value={StoredPaymentMethodType.Card}
          >{$_("payments.typeCard")}</option
        >
        <option value={StoredPaymentMethodType.SepaDebit}
          >{$_("payments.typeSepa")}</option
        >
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
      {#if formData.method_type === StoredPaymentMethodType.Card || formData.method_type === StoredPaymentMethodType.SepaDebit}
        <FormInput
          id="stripe-id"
          label={$_('payments.stripeMethodId')}
          bind:value={formData.stripe_payment_method_id}
          error={errors.stripe_payment_method_id}
          required
          placeholder="pm_xxxxxxxxxxxxx"
          data-testid="stripe-id-input"
        />

        <!--
          `brand` et `last4` etaient saisis ici mais n'existent pas cote
          serveur : ils partaient dans le corps de la requete et n'y servaient
          a rien. Ce que le serveur attend et qui manquait, c'est
          `stripe_customer_id` et `is_default` (#732).

          La marque et les quatre derniers chiffres restent lisibles apres
          coup : le backend les derive du moyen Stripe et les expose dans
          `display_label`.
        -->
        <FormInput
          id="stripe-customer-id"
          label={$_('payments.stripeCustomerId')}
          bind:value={formData.stripe_customer_id}
          error={errors.stripe_customer_id}
          required
          placeholder="cus_xxxxxxxxxxxxx"
          data-testid="stripe-customer-id-input"
        />

        <label class="flex items-center gap-2">
          <input
            type="checkbox"
            bind:checked={formData.is_default}
            data-testid="is-default-input"
          />
          <span>{$_('payments.setAsDefault')}</span>
        </label>
      {/if}

      <!-- Help Text -->

    </div>

    <!-- Actions -->
    <div class="mt-6 flex justify-end space-x-3">
      <Button type="button" variant="outline" onclick={handleClose} data-testid="cancel-btn">
        {$_('common.cancel')}
      </Button>
      <Button type="submit" loading={submitting} data-testid="submit-btn">
        {$_('payments.addMethod')}
      </Button>
    </div>
  </form>
</Modal>
