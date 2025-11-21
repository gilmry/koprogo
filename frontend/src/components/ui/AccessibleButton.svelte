<script lang="ts">
  /**
   * Accessible Button Component - WCAG 2.1 AA Compliant
   * Provides proper ARIA attributes, keyboard support, and visual feedback
   */

  export let type: 'button' | 'submit' | 'reset' = 'button';
  export let variant: 'primary' | 'secondary' | 'danger' | 'success' = 'primary';
  export let disabled: boolean = false;
  export let ariaLabel: string | undefined = undefined;
  export let ariaPressed: boolean | undefined = undefined;
  export let ariaExpanded: boolean | undefined = undefined;
  export let loading: boolean = false;
  export let size: 'sm' | 'md' | 'lg' = 'md';

  const variantClasses = {
    primary: 'bg-green-600 hover:bg-green-700 text-white focus:ring-green-500',
    secondary: 'bg-gray-600 hover:bg-gray-700 text-white focus:ring-gray-500',
    danger: 'bg-red-600 hover:bg-red-700 text-white focus:ring-red-500',
    success: 'bg-blue-600 hover:bg-blue-700 text-white focus:ring-blue-500',
  };

  const sizeClasses = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-4 py-2',
    lg: 'px-6 py-3 text-lg',
  };

  const baseClasses =
    'rounded font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed';

  $: classes = `${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]}`;
</script>

<button
  {type}
  class={classes}
  disabled={disabled || loading}
  aria-label={ariaLabel}
  aria-pressed={ariaPressed}
  aria-expanded={ariaExpanded}
  aria-busy={loading}
  on:click
  on:keydown
  on:keyup
  on:focus
  on:blur
>
  {#if loading}
    <span class="flex items-center gap-2">
      <svg
        class="animate-spin h-5 w-5"
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        aria-hidden="true"
      >
        <circle
          class="opacity-25"
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          stroke-width="4"
        ></circle>
        <path
          class="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        ></path>
      </svg>
      <span class="sr-only">Chargement...</span>
      <slot />
    </span>
  {:else}
    <slot />
  {/if}
</button>

<style>
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
  }
</style>
