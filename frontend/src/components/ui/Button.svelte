<script lang="ts">
  export let type: 'button' | 'submit' | 'reset' = 'button';
  export let variant: 'primary' | 'secondary' | 'danger' | 'success' | 'outline' = 'primary';
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let loading = false;
  export let disabled = false;
  export let fullWidth = false;

  const variantClasses = {
    primary: 'bg-primary-600 text-white hover:bg-primary-700 focus:ring-primary-500',
    secondary: 'bg-gray-600 text-white hover:bg-gray-700 focus:ring-gray-500',
    danger: 'bg-red-600 text-white hover:bg-red-700 focus:ring-red-500',
    success: 'bg-green-600 text-white hover:bg-green-700 focus:ring-green-500',
    outline: 'bg-white text-gray-700 border border-gray-300 hover:bg-gray-50 focus:ring-primary-500',
  };

  const sizeClasses = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-4 py-2',
    lg: 'px-6 py-3 text-lg',
  };

  $: isDisabled = disabled || loading;
</script>

<button
  {type}
  disabled={isDisabled}
  on:click
  {...$$restProps}
  class="inline-flex items-center justify-center font-medium rounded-lg transition focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed
    {variantClasses[variant]}
    {sizeClasses[size]}
    {fullWidth ? 'w-full' : ''}"
>
  {#if loading}
    <svg class="animate-spin -ml-1 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
  {/if}
  <slot />
</button>
