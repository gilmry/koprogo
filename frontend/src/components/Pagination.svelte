<script lang="ts">
  export let currentPage: number;
  export let totalPages: number;
  export let totalItems: number;
  export let perPage: number;
  export let onPageChange: (page: number) => void;

  $: startItem = (currentPage - 1) * perPage + 1;
  $: endItem = Math.min(currentPage * perPage, totalItems);

  // Generate page numbers to display
  $: pages = (() => {
    const maxVisible = 7;
    if (totalPages <= maxVisible) {
      return Array.from({ length: totalPages }, (_, i) => i + 1);
    }

    const pages: (number | string)[] = [];

    if (currentPage <= 4) {
      // Near start: 1 2 3 4 5 ... last
      for (let i = 1; i <= 5; i++) pages.push(i);
      pages.push('...');
      pages.push(totalPages);
    } else if (currentPage >= totalPages - 3) {
      // Near end: 1 ... n-4 n-3 n-2 n-1 n
      pages.push(1);
      pages.push('...');
      for (let i = totalPages - 4; i <= totalPages; i++) pages.push(i);
    } else {
      // Middle: 1 ... current-1 current current+1 ... last
      pages.push(1);
      pages.push('...');
      for (let i = currentPage - 1; i <= currentPage + 1; i++) pages.push(i);
      pages.push('...');
      pages.push(totalPages);
    }

    return pages;
  })();

  function goToPage(page: number) {
    if (page < 1 || page > totalPages || page === currentPage) return;
    onPageChange(page);
  }
</script>

<nav class="flex items-center justify-between border-t border-gray-200 px-4 py-3 sm:px-6" aria-label="Pagination">
  <!-- Mobile view -->
  <div class="flex flex-1 justify-between sm:hidden">
    <button
      on:click={() => goToPage(currentPage - 1)}
      disabled={currentPage === 1}
      class="relative inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      Précédent
    </button>
    <button
      on:click={() => goToPage(currentPage + 1)}
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
        <!-- Previous button -->
        <button
          on:click={() => goToPage(currentPage - 1)}
          disabled={currentPage === 1}
          class="relative inline-flex items-center rounded-l-md px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span class="sr-only">Précédent</span>
          <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <path fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z" clip-rule="evenodd" />
          </svg>
        </button>

        <!-- Page numbers -->
        {#each pages as page}
          {#if page === '...'}
            <span class="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-700 ring-1 ring-inset ring-gray-300 focus:outline-offset-0">
              ...
            </span>
          {:else}
            <button
              on:click={() => goToPage(page as number)}
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

        <!-- Next button -->
        <button
          on:click={() => goToPage(currentPage + 1)}
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
