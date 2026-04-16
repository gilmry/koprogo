<script lang="ts">
  // Svelte 5 runes mode — migrated from legacy (STORY-P7-602)
  let {
    id,
    label,
    value = $bindable(''),
    type = 'text',
    placeholder = '',
    required = false,
    disabled = false,
    error = '',
    hint = '',
    min = undefined,
    max = undefined,
    step = undefined,
    autocomplete = undefined,
    ...restProps
  }: {
    id: string;
    label: string;
    value?: string | number;
    type?: 'text' | 'email' | 'password' | 'number' | 'tel' | 'url' | 'date';
    placeholder?: string;
    required?: boolean;
    disabled?: boolean;
    error?: string;
    hint?: string;
    min?: string | number;
    max?: string | number;
    step?: string | number;
    autocomplete?: string;
    [key: string]: any;
  } = $props();

  const handleInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (type === 'number') {
      value = target.valueAsNumber;
    } else {
      value = target.value;
    }
  };
</script>

<div class="mb-4">
  <label for={id} class="block text-sm font-medium text-gray-700 mb-2">
    {label}
    {#if required}
      <span class="text-red-500" aria-hidden="true">*</span>
      <span class="sr-only">(obligatoire)</span>
    {/if}
  </label>

  <input
    {id}
    {type}
    {placeholder}
    {required}
    {disabled}
    {min}
    {max}
    {step}
    {autocomplete}
    {...restProps}
    {value}
    oninput={handleInput}
    aria-invalid={error ? 'true' : undefined}
    aria-describedby={error ? `${id}-error` : hint ? `${id}-hint` : undefined}
    class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 disabled:bg-gray-100 disabled:cursor-not-allowed transition
      {error ? 'border-red-500' : 'border-gray-300'}"
  />

  {#if error}
    <p id="{id}-error" class="mt-1 text-sm text-red-600" role="alert">{error}</p>
  {/if}

  {#if hint && !error}
    <p id="{id}-hint" class="mt-1 text-sm text-gray-500">{hint}</p>
  {/if}
</div>
