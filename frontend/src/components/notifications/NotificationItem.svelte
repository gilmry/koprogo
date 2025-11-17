<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { notificationStore } from "../../stores/notifications";
  import type { Notification, NotificationType } from "../../lib/api/notifications";

  export let notification: Notification;
  export let clickable = true;

  const dispatch = createEventDispatcher();

  const notificationIcons: Record<string, string> = {
    MeetingReminder: "📅",
    PaymentDue: "💰",
    DocumentShared: "📄",
    TicketUpdate: "🔧",
    TicketAssigned: "👷",
    TicketResolved: "✅",
    SystemAlert: "⚠️",
    AccountUpdate: "👤",
    NewMessage: "💬",
    ConvocationSent: "📨",
    ResolutionVoting: "🗳️",
    QuoteReceived: "📋",
    QuoteAccepted: "✔️",
    PaymentSuccess: "✅",
    PaymentFailed: "❌",
    BudgetApproved: "💵",
    EtatDateReady: "📝",
    ExchangeRequested: "🔄",
    ExchangeCompleted: "✔️",
    AchievementEarned: "🏆",
    ChallengeStarted: "🎯",
    ChallengeCompleted: "🌟",
  };

  function getIcon(type: NotificationType): string {
    return notificationIcons[type] || "🔔";
  }

  function formatTime(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return "Just now";
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return date.toLocaleDateString("nl-BE", {
      month: "short",
      day: "numeric",
    });
  }

  async function handleClick() {
    if (clickable && !notification.is_read) {
      await notificationStore.markAsRead(notification.id);
    }

    // Navigate based on notification type and metadata
    if (notification.metadata) {
      const { ticket_id, meeting_id, payment_id, document_id } =
        notification.metadata;

      if (ticket_id) {
        window.location.href = `/ticket-detail?id=${ticket_id}`;
      } else if (meeting_id) {
        window.location.href = `/meeting-detail?id=${meeting_id}`;
      } else if (payment_id) {
        window.location.href = `/payment-detail?id=${payment_id}`;
      } else if (document_id) {
        window.location.href = `/documents?highlight=${document_id}`;
      }
    }

    dispatch("click");
  }

  async function handleDelete(event: MouseEvent) {
    event.stopPropagation();
    await notificationStore.delete(notification.id);
  }
</script>

<div
  class="px-4 py-3 hover:bg-gray-50 transition-colors {clickable
    ? 'cursor-pointer'
    : ''} {!notification.is_read ? 'bg-blue-50' : ''}"
  on:click={handleClick}
  role={clickable ? "button" : "article"}
  tabindex={clickable ? 0 : -1}
>
  <div class="flex items-start space-x-3">
    <!-- Icon -->
    <div class="flex-shrink-0 text-2xl">
      {getIcon(notification.notification_type)}
    </div>

    <!-- Content -->
    <div class="flex-1 min-w-0">
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <p
            class="text-sm font-medium text-gray-900 {!notification.is_read
              ? 'font-semibold'
              : ''}"
          >
            {notification.title}
          </p>
          <p class="mt-1 text-sm text-gray-600 line-clamp-2">
            {notification.message}
          </p>
        </div>

        <!-- Delete button -->
        <button
          on:click={handleDelete}
          class="ml-2 text-gray-400 hover:text-red-600 transition-colors"
          aria-label="Delete notification"
        >
          <svg
            class="h-4 w-4"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fill-rule="evenodd"
              d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
              clip-rule="evenodd"
            />
          </svg>
        </button>
      </div>

      <!-- Time and unread indicator -->
      <div class="mt-1 flex items-center space-x-2">
        <span class="text-xs text-gray-500">
          {formatTime(notification.created_at)}
        </span>
        {#if !notification.is_read}
          <span class="inline-flex h-2 w-2 rounded-full bg-blue-600"></span>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
