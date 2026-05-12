<script lang="ts">
  // Svelte 5 runes mode
  let { currentPage, totalPages, totalItems, perPage, onPageChange }: {
    currentPage: number;
    totalPages: number;
    totalItems: number;
    perPage: number;
    onPageChange: (page: number) => void;
  } = $props();

  let startItem = $derived((currentPage - 1) * perPage + 1);
  let endItem = $derived(Math.min(currentPage * perPage, totalItems));

  let pages = $derived.by(() => {
    const maxVisible = 7;
    if (totalPages <= maxVisible) {
      return Array.from({ length: totalPages }, (_, i) => i + 1);
    }

    const p: (number | string)[] = [];

    if (currentPage <= 4) {
      for (let i = 1; i <= 5; i++) p.push(i);
      p.push('...');
      p.push(totalPages);
    } else if (currentPage >= totalPages - 3) {
      p.push(1);
      p.push('...');
      for (let i = totalPages - 4; i <= totalPages; i++) p.push(i);
    } else {
      p.push(1);
      p.push('...');
      for (let i = currentPage - 1; i <= currentPage + 1; i++) p.push(i);
      p.push('...');
      p.push(totalPages);
    }

    return p;
  });

  function goToPage(page: number) {
    if (page < 1 || page > totalPages || page === currentPage) return;
    onPageChange(page);
  }
</script>

<nav class="flex items-center justify-between border-t border-gray-200 px-4 py-3 sm:px-6" aria-label="Pagination">
  <!-- Mobile view -->
  <div class="flex flex-1 justify-between sm:hidden">
    <button
      onclick={() => goToPage(currentPage - 1)}
      disabled={currentPage === 1}
      class="relative inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      Precedent
    </button>
    <button
      onclick={() => goToPage(currentPage + 1)}
      disabled={currentPage === totalPages}
      class="relative ml-3 inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      Suivant
    </button>
  </div>

  <!-- Desktop view -->
  <div class="hidden sm:flex sm:flex-1 sm:items-center sm:justify-between">
    <div>
      <p class="text-sm text-gray-700">
        Affichage de <span class="font-medium">{startItem}</span> à <span class="font-medium">{endItem}</span> sur{' '}
        <span class="font-medium">{totalItems}</span> résultat{totalItems > 1 ? 's' : ''}
      </p>
    </div>
    <div>
      <nav class="isolate inline-flex -space-x-px rounded-md shadow-sm" aria-label="Pagination">
        <button
          onclick={() => goToPage(currentPage - 1)}
          disabled={currentPage === 1}
          class="relative inline-flex items-center rounded-l-md px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span class="sr-only">Precedent</span>
          <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <path fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z" clip-rule="evenodd" />
          </svg>
        </button>

        {#each pages as page}
          {#if page === '...'}
            <span class="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-700 ring-1 ring-inset ring-gray-300 focus:outline-offset-0">
              ...
            </span>
          {:else}
            <button
              onclick={() => goToPage(page as number)}
              class="relative inline-flex items-center px-4 py-2 text-sm font-semibold ring-1 ring-inset ring-gray-300 focus:z-20 focus:outline-offset-0
                {page === currentPage
                  ? 'z-10 bg-primary-600 text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary-600'
                  : 'text-gray-900 hover:bg-gray-50'
                }"
            >
              {page}
            </button>
          {/if}
        {/each}

        <button
          onclick={() => goToPage(currentPage + 1)}
          disabled={currentPage === totalPages}
          class="relative inline-flex items-center rounded-r-md px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span class="sr-only">Suivant</span>
          <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd" />
          </svg>
        </button>
      </nav>
    </div>
  </div>
</nav>
