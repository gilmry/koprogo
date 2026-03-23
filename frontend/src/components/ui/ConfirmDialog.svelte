<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { _ } from '../../lib/i18n';
  import Modal from './Modal.svelte';
  import Button from './Button.svelte';

  export let isOpen = false;
  export let title = $_('common.confirm');
  export let message = $_('common.sure');
  export let confirmText = $_('common.confirm');
  export let cancelText = $_('common.cancel');
  export let variant: 'danger' | 'primary' = 'primary';
  export let loading = false;

  const dispatch = createEventDispatcher();

  const handleConfirm = () => {
    dispatch('confirm');
  };

  const handleCancel = () => {
    dispatch('cancel');
  };
</script>

<Modal {isOpen} {title} size="sm" showClose={false} on:close={handleCancel}>
  <p class="text-gray-600">
    {message}
  </p>

  <svelte:fragment slot="footer">
    <div class="flex justify-end space-x-3">
      <Button variant="outline" on:click={handleCancel} disabled={loading} data-testid="confirm-dialog-cancel">
        {cancelText}
      </Button>
      <Button {variant} on:click={handleConfirm} {loading} data-testid="confirm-dialog-confirm">
        {confirmText}
      </Button>
    </div>
  </svelte:fragment>
</Modal>
