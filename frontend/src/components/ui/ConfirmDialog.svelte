<script lang="ts">
  // Svelte 5 runes mode — migrated from legacy (STORY-P7-602)
  import { _ } from '../../lib/i18n';
  import Modal from './Modal.svelte';
  import Button from './Button.svelte';

  let {
    isOpen = false,
    title = 'Confirmer',
    message = 'Êtes-vous sûr ?',
    confirmText = 'Confirmer',
    cancelText = 'Annuler',
    variant = 'primary',
    loading = false,
    onconfirm,
    oncancel,
  }: {
    isOpen?: boolean;
    title?: string;
    message?: string;
    confirmText?: string;
    cancelText?: string;
    variant?: 'danger' | 'primary';
    loading?: boolean;
    onconfirm?: () => void;
    oncancel?: () => void;
  } = $props();

  const handleConfirm = () => {
    onconfirm?.();
  };

  const handleCancel = () => {
    oncancel?.();
  };
</script>

<Modal {isOpen} {title} size="sm" showClose={false} onclose={handleCancel}>
  <p class="text-gray-600">
    {message}
  </p>

  {#snippet footer()}
    <div class="flex justify-end space-x-3">
      <Button variant="outline" onclick={handleCancel} disabled={loading} data-testid="confirm-dialog-cancel">
        {cancelText}
      </Button>
      <Button {variant} onclick={handleConfirm} {loading} data-testid="confirm-dialog-confirm">
        {confirmText}
      </Button>
    </div>
  {/snippet}
</Modal>
