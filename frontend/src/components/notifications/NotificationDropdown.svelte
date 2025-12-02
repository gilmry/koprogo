<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { notificationStore } from "../../stores/notifications";
  import NotificationItem from "./NotificationItem.svelte";
  import type { Notification } from "../../lib/api/notifications";

  const dispatch = createEventDispatcher();

  let notifications: Notification[] = [];
  let loading = false;

  notificationStore.subscribe((state) => {
    notifications = state.notifications.slice(0, 10); // Show latest 10
    loading = state.loading;
  });

  async function handleMarkAllRead() {
    await notificationStore.markAllAsRead();
  }

  function handleViewAll() {
    window.location.href = "/notifications";
    dispatch("close");
  }
</script>

<div
  class="w-96 bg-white rounded-lg shadow-lg ring-1 ring-black ring-opacity-5"
  on:click|stopPropagation
>
  <!-- Header -->
  <div
    class="px-4 py-3 border-b border-gray-200 flex items-center justify-between"
  >
    <h3 class="text-lg font-semibold text-gray-900">Notifications</h3>
    {#if notifications.length > 0}
      <button
        on:click={handleMarkAllRead}
        class="text-sm text-blue-600 hover:text-blue-700 font-medium"
      >
        Mark all read
      </button>
    {/if}
  </div>

  <!-- Notifications List -->
  <div class="max-h-96 overflow-y-auto">
    {#if loading}
      <div class="px-4 py-8 text-center text-gray-500">
        <div
          class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"
        ></div>
        <p class="mt-2">Loading notifications...</p>
      </div>
    {:else if notifications.length === 0}
      <div class="px-4 py-8 text-center text-gray-500">
        <svg
          class="mx-auto h-12 w-12 text-gray-400"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
          />
        </svg>
        <p class="mt-2">No new notifications</p>
      </div>
    {:else}
      <div class="divide-y divide-gray-100">
        {#each notifications as notification (notification.id)}
          <NotificationItem
            {notification}
            on:click={() => dispatch("close")}
          />
        {/each}
      </div>
    {/if}
  </div>

  <!-- Footer -->
  {#if notifications.length > 0}
    <div class="px-4 py-3 border-t border-gray-200">
      <button
        on:click={handleViewAll}
        class="w-full text-center text-sm text-blue-600 hover:text-blue-700 font-medium"
      >
        View all notifications
      </button>
    </div>
  {/if}
</div>
