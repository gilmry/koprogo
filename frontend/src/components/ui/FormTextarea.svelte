<script lang="ts">
  export let id: string;
  export let label: string;
  export let value: string = '';
  export let placeholder = '';
  export let required = false;
  export let disabled = false;
  export let error = '';
  export let hint = '';
  export let rows = 4;

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
    on:input={handleInput}
    on:blur
    on:focus
    on:change
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
