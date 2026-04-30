<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from "../../lib/i18n";
  import { notificationStore } from "../../stores/notifications";
  import NotificationItem from "./NotificationItem.svelte";
  import type { Notification } from "../../lib/api/notifications";

  let { onclose }: { onclose?: () => void } = $props();

  let notifications = $state<Notification[]>([]);
  let loading = $state(false);

  let listEl = $state<HTMLDivElement | undefined>();
  let focusIndex = $state(0);

  notificationStore.subscribe((state) => {
    notifications = state.notifications.slice(0, 10); // Show latest 10
    loading = state.loading;
  });

  async function handleMarkAllRead() {
    await notificationStore.markAllAsRead();
  }

  function handleViewAll() {
    window.location.href = "/notifications";
    onclose?.();
  }

  function getMenuItems(): HTMLElement[] {
    if (!listEl) return [];
    return Array.from(
      listEl.querySelectorAll<HTMLElement>('[role="menuitem"]'),
    );
  }

  function handleKey(e: KeyboardEvent) {
    const items = getMenuItems();
    if (items.length === 0) {
      if (e.key === "Escape") {
        e.preventDefault();
        onclose?.();
      }
      return;
    }

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        focusIndex = (focusIndex + 1) % items.length;
        items[focusIndex].focus();
        break;
      case "ArrowUp":
        e.preventDefault();
        focusIndex = focusIndex <= 0 ? items.length - 1 : focusIndex - 1;
        items[focusIndex].focus();
        break;
      case "Home":
        e.preventDefault();
        focusIndex = 0;
        items[0].focus();
        break;
      case "End":
        e.preventDefault();
        focusIndex = items.length - 1;
        items[focusIndex].focus();
        break;
      case "Escape":
        e.preventDefault();
        onclose?.();
        break;
    }
  }

  // Auto-focus first notification item when dropdown opens / list changes
  $effect(() => {
    if (!listEl) return;
    // Re-run when notifications array changes
    const count = notifications.length;
    if (count === 0) return;
    const items = listEl.querySelectorAll<HTMLElement>('[role="menuitem"]');
    if (items.length > 0) {
      focusIndex = 0;
      items[0].focus();
    }
  });
</script>

<div
  bind:this={listEl}
  class="w-96 max-w-[calc(100vw-2rem)] bg-white rounded-lg shadow-lg ring-1 ring-black ring-opacity-5"
  onclick={(e) => e.stopPropagation()}
  onkeydown={handleKey}
  role="presentation"
>
  <!-- Header -->
  <div
    class="px-4 py-3 border-b border-gray-200 flex items-center justify-between"
  >
    <h3 class="text-lg font-semibold text-gray-900">{$_("notifications.title")}</h3>
    {#if notifications.length > 0}
      <button
        onclick={handleMarkAllRead}
        class="text-sm text-blue-600 hover:text-blue-700 font-medium"
      >
        {$_("notifications.markAllRead")}
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
        <p class="mt-2">{$_("common.loading")}</p>
      </div>
    {:else if notifications.length === 0}
      <div class="px-4 py-8 text-center text-gray-500">
        <svg
          class="mx-auto h-12 w-12 text-gray-400"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          aria-hidden="true"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
          />
        </svg>
        <p class="mt-2">{$_("notifications.noNotifications")}</p>
      </div>
    {:else}
      <ul class="divide-y divide-gray-100 list-none m-0 p-0" role="menu" aria-label="Notifications">
        {#each notifications as notification (notification.id)}
          <li role="menuitem" tabindex="-1">
            <NotificationItem
              {notification}
              onclick={() => onclose?.()}
            />
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <!-- Footer -->
  {#if notifications.length > 0}
    <div class="px-4 py-3 border-t border-gray-200">
      <button
        onclick={handleViewAll}
        class="w-full text-center text-sm text-blue-600 hover:text-blue-700 font-medium"
      >
        {$_("notifications.viewAll")}
      </button>
    </div>
  {/if}
</div>
