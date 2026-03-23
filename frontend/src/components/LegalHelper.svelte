<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { api } from '../lib/api';

  interface LegalRule {
    code: string;
    category: string;
    roles: string[];
    article: string;
    title: string;
    content: string;
    keywords: string[];
  }

  interface AGStep {
    step: number;
    point_odj: string;
    mandatory: boolean;
    majority: string | null;
    notes: string;
  }

  interface MajorityType {
    decision_type: string;
    label: string;
    threshold_description: string;
    article: string;
    examples: string[];
    percentage: number | null;
  }

  let isOpen = false;
  let isLoading = false;
  let currentPath = '';

  let agSequence: AGStep[] = [];
  let legalRules: LegalRule[] = [];
  let majorityTypes: MajorityType[] = [];

  onMount(() => {
    // Get current page path
    currentPath = window.location.pathname;
    loadLegalData();
  });

  async function loadLegalData() {
    isLoading = true;
    try {
      // Load all legal data in parallel
      const [sequenceRes, rulesRes, majoritiesRes] = await Promise.all([
        fetch(`${api.baseUrl}/legal/ag-sequence`),
        fetch(`${api.baseUrl}/legal/rules`),
        fetch(`${api.baseUrl}/legal/majority-for/ordinary`)
      ]);

      if (sequenceRes.ok) {
        agSequence = await sequenceRes.json();
      }

      if (rulesRes.ok) {
        legalRules = await rulesRes.json();
      }

      if (majoritiesRes.ok) {
        majorityTypes = await majoritiesRes.json();
      }
    } catch (error) {
      console.error('Failed to load legal data:', error);
    } finally {
      isLoading = false;
    }
  }

  function togglePanel() {
    isOpen = !isOpen;
  }

  function closePanel() {
    isOpen = false;
  }

  function getContextualContent() {
    // Return contextual legal help based on current URL path
    if (currentPath.includes('/meetings')) {
      return {
        title: $_('legal.meetings.title', { default: 'General Assembly Rules' }),
        content: 'ag_sequence'
      };
    } else if (currentPath.includes('/resolutions') || currentPath.includes('/votes')) {
      return {
        title: $_('legal.resolutions.title', { default: 'Voting & Majorities' }),
        content: 'majorities'
      };
    } else if (currentPath.includes('/expenses') || currentPath.includes('/travaux')) {
      return {
        title: $_('legal.expenses.title', { default: 'Works & Expenses' }),
        content: 'travaux'
      };
    } else {
      return {
        title: $_('legal.default.title', { default: 'Belgian Copropriété Law' }),
        content: 'general'
      };
    }
  }

  function getRelevantRules(category?: string) {
    if (!category) {
      return legalRules.slice(0, 5); // Show first 5 rules as default
    }
    return legalRules.filter(rule => rule.category === category);
  }

  const contextContent = getContextualContent();
</script>

<!-- Floating Help Button -->
<div class="fixed bottom-4 right-4 z-40">
  <button
    on:click={togglePanel}
    class="w-12 h-12 rounded-full bg-blue-600 hover:bg-blue-700 text-white shadow-lg flex items-center justify-center font-bold text-xl transition-all duration-200 hover:shadow-xl"
    aria-label={$_('legal.help.button', { default: 'Legal Help' })}
    title={$_('legal.help.button', { default: 'Legal Help' })}
    data-testid="legal-helper-toggle-btn"
  >
    ?
  </button>
</div>

<!-- Side Panel -->
{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black bg-opacity-50 z-30 transition-opacity"
    on:click={closePanel}
    role="presentation"
  />

  <!-- Panel -->
  <div class="fixed right-0 top-0 h-screen w-96 bg-white shadow-2xl z-40 overflow-y-auto flex flex-col">
    <!-- Header -->
    <div class="sticky top-0 bg-gradient-to-r from-blue-600 to-blue-700 text-white p-4 shadow-md flex items-center justify-between">
      <h2 class="text-lg font-bold">{contextContent.title}</h2>
      <button
        on:click={closePanel}
        class="text-white hover:bg-blue-800 rounded p-1 transition-colors"
        aria-label={$_('legal.close', { default: 'Close' })}
        data-testid="legal-helper-close-btn"
      >
        ✕
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      {#if isLoading}
        <div class="flex items-center justify-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        </div>
      {:else if contextContent.content === 'ag_sequence'}
        <!-- AG Sequence Content -->
        <div class="space-y-3">
          <p class="text-sm text-gray-600 mb-4">
            {$_('legal.ag_sequence.description', { default: 'Mandatory sequence of general assembly agenda items according to Belgian law' })}
          </p>
          {#each agSequence as step (step.step)}
            <div class="border-l-4 border-blue-500 pl-3 py-2 bg-blue-50 rounded">
              <div class="flex items-start justify-between gap-2">
                <div class="flex-1">
                  <p class="font-semibold text-sm text-gray-900">
                    {step.step}. {step.point_odj}
                  </p>
                  {#if step.mandatory}
                    <span class="inline-block mt-1 px-2 py-0.5 bg-red-100 text-red-700 text-xs rounded">
                      {$_('legal.mandatory', { default: 'Mandatory' })}
                    </span>
                  {/if}
                  {#if step.majority}
                    <span class="inline-block mt-1 ml-2 px-2 py-0.5 bg-amber-100 text-amber-700 text-xs rounded">
                      {step.majority}
                    </span>
                  {/if}
                </div>
              </div>
              <p class="text-xs text-gray-600 mt-2">{step.notes}</p>
            </div>
          {/each}
        </div>
      {:else if contextContent.content === 'majorities'}
        <!-- Majorities Content -->
        <div class="space-y-3">
          <p class="text-sm text-gray-600 mb-4">
            {$_('legal.majorities.description', { default: 'Majority rules for different types of decisions' })}
          </p>
          {#each majorityTypes as majority (majority.decision_type)}
            <div class="border rounded-lg p-3 bg-gray-50">
              <p class="font-semibold text-sm text-gray-900">{majority.label}</p>
              <p class="text-xs text-gray-600 mt-1">{majority.threshold_description}</p>
              {#if majority.percentage !== null}
                <div class="mt-2 bg-white p-2 rounded text-xs">
                  <div class="flex justify-between items-center mb-1">
                    <span>{majority.percentage}%</span>
                  </div>
                  <div class="w-full bg-gray-200 rounded-full h-2">
                    <div class="bg-blue-600 h-2 rounded-full" style="width: {majority.percentage}%"></div>
                  </div>
                </div>
              {/if}
              {#if majority.examples && majority.examples.length > 0}
                <p class="text-xs text-gray-700 mt-2">
                  <span class="font-semibold">{$_('legal.examples', { default: 'Examples' })}:</span>
                  {majority.examples.join(', ')}
                </p>
              {/if}
              <p class="text-xs text-gray-500 mt-2 italic">{majority.article}</p>
            </div>
          {/each}
        </div>
      {:else if contextContent.content === 'travaux'}
        <!-- Travaux Rules -->
        <div class="space-y-3">
          <p class="text-sm text-gray-600 mb-4">
            {$_('legal.travaux.description', { default: 'Rules for works and expenses' })}
          </p>
          {#each getRelevantRules('travaux') as rule (rule.code)}
            <div class="border-l-4 border-orange-500 pl-3 py-2 bg-orange-50 rounded">
              <p class="font-semibold text-sm text-gray-900">{rule.code} — {rule.title}</p>
              <p class="text-xs text-gray-600 mt-2">{rule.content}</p>
              <p class="text-xs text-gray-500 mt-2 italic">{rule.article}</p>
            </div>
          {/each}
        </div>
      {:else}
        <!-- General Copropriété Content -->
        <div class="space-y-3">
          <p class="text-sm text-gray-600 mb-4">
            {$_('legal.general.description', { default: 'General information about Belgian copropriété law' })}
          </p>
          {#each getRelevantRules().slice(0, 5) as rule (rule.code)}
            <div class="border-l-4 border-green-500 pl-3 py-2 bg-green-50 rounded">
              <p class="font-semibold text-sm text-gray-900">{rule.code} — {rule.title}</p>
              <p class="text-xs text-gray-600 mt-1">{rule.content.substring(0, 100)}...</p>
              <p class="text-xs text-gray-500 mt-2 italic">{rule.article}</p>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="sticky bottom-0 bg-gray-100 border-t p-3 text-xs text-gray-600">
      <p>
        {$_('legal.footer', { default: 'Based on Belgian Civil Code Articles 3.78-3.100 and IPI Ethics Code' })}
      </p>
    </div>
  </div>
{/if}

<style>
  :global(body) {
    /* Ensure panel doesn't get hidden */
    overflow-x: hidden;
  }
</style>
