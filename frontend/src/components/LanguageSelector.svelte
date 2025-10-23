<script lang="ts">
  import { locale } from 'svelte-i18n';
  import { languages, type LanguageCode } from '../lib/i18n';

  let isOpen = false;

  function selectLanguage(code: LanguageCode) {
    locale.set(code);
    // Save preference to localStorage
    localStorage.setItem('preferred-language', code);
    isOpen = false;
  }

  // Get current language or default to NL
  $: currentLang = languages.find((lang) => lang.code === $locale) || languages[0];
</script>

<div class="relative inline-block text-left">
  <!-- Language Button -->
  <button
    type="button"
    onclick={() => (isOpen = !isOpen)}
    class="inline-flex items-center gap-2 rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50"
    aria-expanded={isOpen}
    aria-haspopup="true"
  >
    <span class="text-xl">{currentLang.flag}</span>
    <span>{currentLang.code.toUpperCase()}</span>
    <svg
      class="-mr-1 h-5 w-5 text-gray-400"
      viewBox="0 0 20 20"
      fill="currentColor"
      aria-hidden="true"
    >
      <path
        fill-rule="evenodd"
        d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
        clip-rule="evenodd"
      />
    </svg>
  </button>

  <!-- Dropdown Menu -->
  {#if isOpen}
    <!-- Overlay to close when clicking outside -->
    <button
      class="fixed inset-0 z-10"
      onclick={() => (isOpen = false)}
      aria-hidden="true"
      tabindex="-1"
    />

    <div
      class="absolute right-0 z-20 mt-2 w-56 origin-top-right rounded-md bg-white shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none"
      role="menu"
      aria-orientation="vertical"
      tabindex="-1"
    >
      <div class="py-1" role="none">
        {#each languages as lang}
          <button
            type="button"
            onclick={() => selectLanguage(lang.code)}
            class="flex w-full items-center gap-3 px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900 {$locale ===
            lang.code
              ? 'bg-gray-50 font-semibold'
              : ''}"
            role="menuitem"
            tabindex="-1"
          >
            <span class="text-xl">{lang.flag}</span>
            <span class="flex-1 text-left">{lang.name}</span>
            {#if $locale === lang.code}
              <svg
                class="h-5 w-5 text-green-600"
                viewBox="0 0 20 20"
                fill="currentColor"
                aria-hidden="true"
              >
                <path
                  fill-rule="evenodd"
                  d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z"
                  clip-rule="evenodd"
                />
              </svg>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  /* Add any additional styles if needed */
</style>
