<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Modal from './Modal.svelte';
  import Button from './Button.svelte';

  export let isOpen = false;
  export let title = 'Confirmer l\'action';
  export let message = 'Êtes-vous sûr de vouloir continuer?';
  export let confirmText = 'Confirmer';
  export let cancelText = 'Annuler';
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
      <Button variant="outline" on:click={handleCancel} disabled={loading}>
        {cancelText}
      </Button>
      <Button {variant} on:click={handleConfirm} {loading}>
        {confirmText}
      </Button>
    </div>
  </svelte:fragment>
</Modal>
