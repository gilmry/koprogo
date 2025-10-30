<script lang="ts">
  export let id: string;
  export let label: string;
  export let value: string | number = '';
  export let options: Array<{ value: string | number; label: string }> = [];
  export let placeholder = 'Sélectionner...';
  export let required = false;
  export let disabled = false;
  export let error = '';
  export let hint = '';

</script>

<div class="mb-4">
  <label for={id} class="block text-sm font-medium text-gray-700 mb-2">
    {label}
    {#if required}
      <span class="text-red-500">*</span>
    {/if}
  </label>

  <select
    {id}
    {required}
    {disabled}
    bind:value
    {...$$restProps}
    on:blur
    on:focus
    class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 disabled:bg-gray-100 disabled:cursor-not-allowed transition
      {error ? 'border-red-500' : 'border-gray-300'}"
  >
    {#if placeholder}
      <option value="" disabled selected={!value}>{placeholder}</option>
    {/if}
    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>

  {#if error}
    <p class="mt-1 text-sm text-red-600">{error}</p>
  {/if}

  {#if hint && !error}
    <p class="mt-1 text-sm text-gray-500">{hint}</p>
  {/if}
</div>
