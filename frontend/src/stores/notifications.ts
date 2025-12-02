import { writable } from "svelte/store";
import { notificationsApi, type Notification } from "../lib/api/notifications";

interface NotificationStore {
  notifications: Notification[];
  unreadCount: number;
  loading: boolean;
}

function createNotificationStore() {
  const { subscribe, set, update } = writable<NotificationStore>({
    notifications: [],
    unreadCount: 0,
    loading: false,
  });

  return {
    subscribe,

    /**
     * Load unread notifications
     */
    async loadUnread() {
      update((state) => ({ ...state, loading: true }));
      try {
        const notifications = await notificationsApi.getUnread();
        update((state) => ({
          ...state,
          notifications,
          unreadCount: notifications.length,
          loading: false,
        }));
      } catch (err) {
        console.error("Failed to load notifications:", err);
        update((state) => ({ ...state, loading: false }));
      }
    },

    /**
     * Mark notification as read
     */
    async markAsRead(id: string) {
      try {
        await notificationsApi.markAsRead(id);
        update((state) => ({
          ...state,
          notifications: state.notifications.filter((n) => n.id !== id),
          unreadCount: Math.max(0, state.unreadCount - 1),
        }));
      } catch (err) {
        console.error("Failed to mark notification as read:", err);
      }
    },

    /**
     * Mark all notifications as read
     */
    async markAllAsRead() {
      try {
        await notificationsApi.markAllAsRead();
        update((state) => ({
          ...state,
          notifications: [],
          unreadCount: 0,
        }));
      } catch (err) {
        console.error("Failed to mark all notifications as read:", err);
      }
    },

    /**
     * Delete notification
     */
    async delete(id: string) {
      try {
        await notificationsApi.delete(id);
        update((state) => ({
          ...state,
          notifications: state.notifications.filter((n) => n.id !== id),
          unreadCount: Math.max(0, state.unreadCount - 1),
        }));
      } catch (err) {
        console.error("Failed to delete notification:", err);
      }
    },

    /**
     * Add new notification (e.g., from WebSocket)
     */
    addNotification(notification: Notification) {
      update((state) => ({
        ...state,
        notifications: [notification, ...state.notifications],
        unreadCount: state.unreadCount + 1,
      }));
    },

    /**
     * Reset store
     */
    reset() {
      set({
        notifications: [],
        unreadCount: 0,
        loading: false,
      });
    },
  };
}

export const notificationStore = createNotificationStore();
