<script lang="ts">
  import { onMount } from "svelte";
  import {
    notificationsApi,
    type Notification,
  } from "../../lib/api/notifications";
  import { notificationStore } from "../../stores/notifications";
  import NotificationItem from "./NotificationItem.svelte";
  import { addToast } from "../../stores/toast";

  let notifications: Notification[] = [];
  let loading = true;
  let filter: "all" | "unread" = "all";

  onMount(async () => {
    await loadNotifications();
  });

  async function loadNotifications() {
    try {
      loading = true;
      if (filter === "unread") {
        notifications = await notificationsApi.getUnread();
      } else {
        notifications = await notificationsApi.listMy();
      }
    } catch (err: any) {
      addToast({
        message: err.message || "Failed to load notifications",
        type: "error",
      });
    } finally {
      loading = false;
    }
  }

  async function handleMarkAllRead() {
    try {
      await notificationStore.markAllAsRead();
      addToast({
        message: "All notifications marked as read",
        type: "success",
      });
      await loadNotifications();
    } catch (err: any) {
      addToast({
        message: err.message || "Failed to mark all as read",
        type: "error",
      });
    }
  }

  $: {
    // Reload when filter changes
    filter;
    loadNotifications();
  }
</script>

<div class="bg-white shadow rounded-lg">
  <!-- Header -->
  <div class="px-6 py-4 border-b border-gray-200">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-semibold text-gray-900">
        Notifications
        <span class="ml-2 text-sm text-gray-500">
          ({notifications.length})
        </span>
      </h2>
      <div class="flex items-center space-x-3">
        {#if notifications.some((n) => !n.is_read)}
          <button
            on:click={handleMarkAllRead}
            class="text-sm text-blue-600 hover:text-blue-700 font-medium"
          >
            Mark all read
          </button>
        {/if}
        <button
          on:click={loadNotifications}
          class="px-3 py-1 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
        >
          Refresh
        </button>
      </div>
    </div>

    <!-- Filter -->
    <div class="flex space-x-2">
      <button
        on:click={() => (filter = "all")}
        class="px-4 py-2 text-sm font-medium rounded-md {filter === 'all'
          ? 'bg-blue-100 text-blue-700'
          : 'text-gray-600 hover:bg-gray-100'}"
      >
        All
      </button>
      <button
        on:click={() => (filter = "unread")}
        class="px-4 py-2 text-sm font-medium rounded-md {filter === 'unread'
          ? 'bg-blue-100 text-blue-700'
          : 'text-gray-600 hover:bg-gray-100'}"
      >
        Unread
      </button>
    </div>
  </div>

  <!-- Notifications -->
  <div class="divide-y divide-gray-200">
    {#if loading}
      <div class="px-6 py-12 text-center text-gray-500">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
        ></div>
        <p class="mt-4">Loading notifications...</p>
      </div>
    {:else if notifications.length === 0}
      <div class="px-6 py-12 text-center text-gray-500">
        <svg
          class="mx-auto h-16 w-16 text-gray-400"
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
        <p class="mt-4 text-lg font-medium">
          {filter === "unread" ? "No unread notifications" : "No notifications"}
        </p>
        <p class="mt-2 text-sm">
          {filter === "unread"
            ? "All caught up! You're all set."
            : "You don't have any notifications yet."}
        </p>
      </div>
    {:else}
      {#each notifications as notification (notification.id)}
        <NotificationItem {notification} on:deleted={loadNotifications} />
      {/each}
    {/if}
  </div>
</div>
