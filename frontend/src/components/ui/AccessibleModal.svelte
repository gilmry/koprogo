<script lang="ts">
  /**
   * Accessible Modal Dialog - WCAG 2.1 AA Compliant
   * Features: Focus trap, keyboard navigation, ARIA attributes
   */

  import { onMount, onDestroy } from 'svelte';
  import { trapFocus, FocusManager, announce } from '../../lib/accessibility';

  export let isOpen: boolean = false;
  export let title: string;
  export let description: string | undefined = undefined;
  export let onClose: () => void;
  export let size: 'sm' | 'md' | 'lg' | 'xl' = 'md';

  let dialogEl: HTMLElement;
  let cleanupFocusTrap: (() => void) | undefined;
  const focusManager = new FocusManager();

  const sizeClasses = {
    sm: 'max-w-md',
    md: 'max-w-2xl',
    lg: 'max-w-4xl',
    xl: 'max-w-6xl',
  };

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      handleClose();
    }
  }

  function handleClose() {
    if (onClose) {
      onClose();
    }
  }

  $: if (isOpen && dialogEl) {
    focusManager.save();
    cleanupFocusTrap = trapFocus(dialogEl);
    announce(`Fenêtre modale ouverte: ${title}`, 'polite');
    document.body.style.overflow = 'hidden';
  } else if (!isOpen && cleanupFocusTrap) {
    cleanupFocusTrap();
    focusManager.restore();
    announce('Fenêtre modale fermée', 'polite');
    document.body.style.overflow = '';
  }

  onDestroy(() => {
    if (cleanupFocusTrap) {
      cleanupFocusTrap();
    }
    document.body.style.overflow = '';
  });
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 overflow-y-auto"
    aria-labelledby="modal-title"
    aria-describedby={description ? 'modal-description' : undefined}
    role="dialog"
    aria-modal="true"
    on:click={handleBackdropClick}
  >
    <div
      class="flex min-h-screen items-center justify-center p-4 text-center sm:p-0"
    >
      <!-- Background overlay -->
      <div
        class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
        aria-hidden="true"
      ></div>

      <!-- Modal panel -->
      <div
        bind:this={dialogEl}
        class="relative transform overflow-hidden rounded-lg bg-white text-left shadow-xl transition-all sm:my-8 sm:w-full {sizeClasses[size]}"
      >
        <!-- Header -->
        <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
          <div class="flex items-start justify-between">
            <h2
              id="modal-title"
              class="text-lg font-semibold leading-6 text-gray-900"
            >
              {title}
            </h2>
            <button
              type="button"
              class="rounded-md text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2"
              on:click={handleClose}
              aria-label="Fermer la fenêtre modale"
            >
              <svg
                class="h-6 w-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                aria-hidden="true"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>

          {#if description}
            <p id="modal-description" class="mt-2 text-sm text-gray-500">
              {description}
            </p>
          {/if}
        </div>

        <!-- Content -->
        <div class="bg-white px-4 pb-4 sm:p-6">
          <slot />
        </div>

        <!-- Footer (optional) -->
        {#if $$slots.footer}
          <div class="bg-gray-50 px-4 py-3 sm:flex sm:flex-row-reverse sm:px-6">
            <slot name="footer" />
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
