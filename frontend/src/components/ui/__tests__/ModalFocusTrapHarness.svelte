<script lang="ts">
  // Test harness for Modal focus-trap specs (STORY-P7-803)
  import Modal from '../Modal.svelte';

  let { isOpen = false }: { isOpen?: boolean } = $props();
</script>

<Modal {isOpen} title="Focus Trap Test">
  <input data-testid="input-first" type="text" />
  <button data-testid="btn-middle">Middle</button>
  <button data-testid="btn-last">Last</button>
</Modal>
