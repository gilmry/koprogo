<!--
  @deprecated Use Modal; focus trap is now built in (STORY-P7-803).
  This file exists only as a backwards-compatibility shim for existing imports.
  New code should import Modal directly from './Modal.svelte'.
-->
<script lang="ts">
  import Modal from './Modal.svelte';
  import type { Snippet } from 'svelte';

  let {
    isOpen = false,
    title,
    description = undefined,
    onClose,
    size = 'md',
    children,
    footer: footerSnippet,
  }: {
    isOpen?: boolean;
    title: string;
    description?: string | undefined;
    onClose: () => void;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    children?: Snippet;
    footer?: Snippet;
  } = $props();

  // Bridge the legacy prop name `onClose` → Modal's `onclose`
  const handleClose = () => onClose?.();
</script>

<Modal {isOpen} {title} {size} onclose={handleClose}>
  {#if description}
    <p class="mb-4 text-sm text-gray-500">{description}</p>
  {/if}
  {#if children}{@render children()}{/if}
  {#snippet footer()}
    {#if footerSnippet}{@render footerSnippet()}{/if}
  {/snippet}
</Modal>
