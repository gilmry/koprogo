<script lang="ts">
  import '../lib/i18n';
  import { _ } from 'svelte-i18n';

  export let mandateCount: number = 0;
  export let totalDelegatedPct: number = 0;

  const MAX_MANDATES = 3;
  const MAX_PCT = 10;

  $: isAtLimit = mandateCount >= MAX_MANDATES;
  $: isNearLimit = mandateCount === MAX_MANDATES - 1;
  $: isQuotaAtRisk = totalDelegatedPct > MAX_PCT * 0.8; // warn at 80% of limit
  $: isQuotaExceeded = totalDelegatedPct > MAX_PCT;
</script>

{#if isAtLimit || isQuotaExceeded}
  <div class="rounded-md bg-red-50 border border-red-200 p-3">
    <div class="flex items-start gap-2">
      <svg
        class="h-4 w-4 text-red-500 mt-0.5 flex-shrink-0"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z"
          clip-rule="evenodd"
        />
      </svg>
      <div>
        <p class="text-sm font-semibold text-red-800">{$_('proxy.limitReached')}</p>
        {#if isAtLimit}
          <p class="text-xs text-red-700 mt-0.5">
            {$_('proxy.mandateCount', { values: { count: mandateCount, max: MAX_MANDATES } })}
          </p>
        {/if}
        {#if isQuotaExceeded}
          <p class="text-xs text-red-700 mt-0.5">
            {$_('proxy.quotaExceeded', { values: { pct: totalDelegatedPct.toFixed(1), max: MAX_PCT } })}
          </p>
        {/if}
      </div>
    </div>
  </div>
{:else if isNearLimit || isQuotaAtRisk}
  <div class="rounded-md bg-amber-50 border border-amber-200 p-3">
    <div class="flex items-start gap-2">
      <svg
        class="h-4 w-4 text-amber-500 mt-0.5 flex-shrink-0"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z"
          clip-rule="evenodd"
        />
      </svg>
      <div>
        <p class="text-sm font-semibold text-amber-800">{$_('proxy.warning')}</p>
        {#if isNearLimit}
          <p class="text-xs text-amber-700 mt-0.5">
            {$_('proxy.mandateNearLimit', { values: { count: mandateCount, max: MAX_MANDATES } })}
          </p>
        {/if}
        {#if isQuotaAtRisk && !isQuotaExceeded}
          <p class="text-xs text-amber-700 mt-0.5">
            {$_('proxy.quotaAtRisk', { values: { pct: totalDelegatedPct.toFixed(1), max: MAX_PCT } })}
          </p>
        {/if}
      </div>
    </div>
  </div>
{/if}
