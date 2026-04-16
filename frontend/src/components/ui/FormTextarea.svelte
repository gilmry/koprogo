<script lang="ts">
  // Svelte 5 runes mode — migrated from legacy (STORY-P7-602)
  let {
    id,
    label,
    value = $bindable(''),
    placeholder = '',
    required = false,
    disabled = false,
    error = '',
    hint = '',
    rows = 4,
    ...restProps
  }: {
    id: string;
    label: string;
    value?: string;
    placeholder?: string;
    required?: boolean;
    disabled?: boolean;
    error?: string;
    hint?: string;
    rows?: number;
    [key: string]: any;
  } = $props();

  const handleInput = (e: Event) => {
    const target = e.target as HTMLTextAreaElement;
    value = target.value;
  };
</script>

<div class="mb-4">
  <label for={id} class="block text-sm font-medium text-gray-700 mb-2">
    {label}
    {#if required}
      <span class="text-red-500">*</span>
    {/if}
  </label>

  <textarea
    {id}
    {placeholder}
    {required}
    {disabled}
    {rows}
    {value}
    oninput={handleInput}
    {...restProps}
    class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 disabled:bg-gray-100 disabled:cursor-not-allowed transition resize-y
      {error ? 'border-red-500' : 'border-gray-300'}"
  />

  {#if error}
    <p class="mt-1 text-sm text-red-600">{error}</p>
  {/if}

  {#if hint && !error}
    <p class="mt-1 text-sm text-gray-500">{hint}</p>
  {/if}
</div>
