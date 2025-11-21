<script lang="ts">
  import { onMount } from "svelte";
  import { notificationStore } from "../../stores/notifications";
  import NotificationDropdown from "./NotificationDropdown.svelte";

  let showDropdown = false;
  let unreadCount = 0;

  // Subscribe to notification store
  notificationStore.subscribe((state) => {
    unreadCount = state.unreadCount;
  });

  onMount(async () => {
    // Load unread notifications on mount
    await notificationStore.loadUnread();

    // Poll for new notifications every 30 seconds
    // In production, use WebSocket instead
    const interval = setInterval(() => {
      notificationStore.loadUnread();
    }, 30000);

    return () => clearInterval(interval);
  });

  function toggleDropdown() {
    showDropdown = !showDropdown;
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest(".notification-bell")) {
      showDropdown = false;
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="notification-bell relative inline-block">
  <!-- Bell Icon Button -->
  <button
    on:click|stopPropagation={toggleDropdown}
    class="relative p-2 text-gray-400 hover:text-gray-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 rounded-full"
    aria-label="Notifications"
  >
    <!-- Bell SVG Icon -->
    <svg
      class="h-6 w-6"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
      />
    </svg>

    <!-- Unread Badge -->
    {#if unreadCount > 0}
      <span
        class="absolute top-0 right-0 inline-flex items-center justify-center px-2 py-1 text-xs font-bold leading-none text-white transform translate-x-1/2 -translate-y-1/2 bg-red-600 rounded-full"
      >
        {unreadCount > 99 ? "99+" : unreadCount}
      </span>
    {/if}
  </button>

  <!-- Dropdown -->
  {#if showDropdown}
    <div class="absolute right-0 mt-2 z-50">
      <NotificationDropdown on:close={() => (showDropdown = false)} />
    </div>
  {/if}
</div>
