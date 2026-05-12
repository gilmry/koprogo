<script lang="ts">
  // Svelte 5 runes mode — migrated from legacy (STORY-P7-602)
  // Focus trap + focus restoration backported from AccessibleModal (STORY-P7-803)
  import type { Snippet } from 'svelte';
  import { trapFocus, FocusManager, getFocusableElements } from '../../lib/accessibility';

  let {
    isOpen = false,
    title = '',
    size = 'md',
    showClose = true,
    onclose,
    children,
    footer,
  }: {
    isOpen?: boolean;
    title?: string;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    showClose?: boolean;
    onclose?: () => void;
    children?: Snippet;
    footer?: Snippet;
  } = $props();

  const sizeClasses: Record<string, string> = {
    sm: 'max-w-md',
    md: 'max-w-2xl',
    lg: 'max-w-4xl',
    xl: 'max-w-6xl',
  };

  let dialogEl = $state<HTMLElement | undefined>(undefined);

  const handleClose = () => {
    onclose?.();
  };

  // Keyboard: Escape closes the modal
  $effect(() => {
    const handleKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        handleClose();
      }
    };
    document.addEventListener('keydown', handleKeydown);
    return () => {
      document.removeEventListener('keydown', handleKeydown);
    };
  });

  // Body scroll lock
  $effect(() => {
    if (typeof window !== 'undefined') {
      document.body.style.overflow = isOpen ? 'hidden' : 'auto';
    }
  });

  // Focus trap + focus restoration (WCAG 2.1 AA)
  $effect(() => {
    if (!isOpen || !dialogEl) return;

    const focusManager = new FocusManager();
    focusManager.save();

    // Activate focus trap — this also moves focus to the first focusable child
    const cleanupTrap = trapFocus(dialogEl);

    // If trapFocus didn't focus anything (no focusable children), fall back to dialog
    const focusables = getFocusableElements(dialogEl);
    if (focusables.length === 0) {
      dialogEl.focus();
    }

    // Screen-reader announcement is handled implicitly by aria-labelledby
    // pointing at #modal-title when focus enters the dialog.

    return () => {
      cleanupTrap();
      focusManager.restore();
    };
  });
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black bg-opacity-50 z-40 transition-opacity"
    onclick={handleClose}
    aria-hidden="true"
    role="presentation"
  ></div>

  <!-- Modal -->
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 overflow-y-auto">
    <div
      bind:this={dialogEl}
      class="bg-white rounded-lg shadow-xl w-full {sizeClasses[size]} mx-auto my-8 max-h-[90vh] flex flex-col"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
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
            onclick={handleClose}
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
        {#if children}
          {@render children()}
        {/if}
      </div>

      <!-- Footer (optional) -->
      {#if footer}
        <div class="border-t border-gray-200 p-6 bg-gray-50">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}
