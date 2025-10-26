<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';

  export let isOpen = false;
  export let title = '';
  export let size: 'sm' | 'md' | 'lg' | 'xl' = 'md';
  export let showClose = true;

  const dispatch = createEventDispatcher();

  const sizeClasses = {
    sm: 'max-w-md',
    md: 'max-w-2xl',
    lg: 'max-w-4xl',
    xl: 'max-w-6xl',
  };

  const handleClose = () => {
    dispatch('close');
  };

  const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && isOpen) {
      handleClose();
    }
  };

  onMount(() => {
    document.addEventListener('keydown', handleKeydown);
    return () => {
      document.removeEventListener('keydown', handleKeydown);
    };
  });

  $: if (typeof window !== 'undefined') {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = 'auto';
    }
  }
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black bg-opacity-50 z-40 transition-opacity"
    on:click={handleClose}
    on:keydown={(e) => e.key === 'Enter' && handleClose()}
    role="button"
    tabindex="0"
    aria-label="Close modal"
  ></div>

  <!-- Modal -->
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 overflow-y-auto">
    <div
      class="bg-white rounded-lg shadow-xl w-full {sizeClasses[size]} mx-auto my-8 max-h-[90vh] flex flex-col"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-6 border-b border-gray-200">
        <h2 id="modal-title" class="text-xl font-semibold text-gray-900">
          {title}
        </h2>
        {#if showClose}
          <button
            on:click={handleClose}
            class="text-gray-400 hover:text-gray-600 transition"
            aria-label="Close"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        {/if}
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-6">
        <slot />
      </div>

      <!-- Footer (optional) -->
      {#if $$slots.footer}
        <div class="border-t border-gray-200 p-6 bg-gray-50">
          <slot name="footer" />
        </div>
      {/if}
    </div>
  </div>
{/if}
