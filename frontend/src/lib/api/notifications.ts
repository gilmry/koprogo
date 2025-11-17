import { api } from "../api";

/**
 * Notification API Client
 * Wraps all 11 backend endpoints for notification management
 */

export interface Notification {
  id: string;
  user_id: string;
  title: string;
  message: string;
  notification_type: NotificationType;
  channel: NotificationChannel;
  status: NotificationStatus;
  is_read: boolean;
  metadata?: Record<string, any>;
  sent_at?: string;
  delivered_at?: string;
  failed_at?: string;
  created_at: string;
  updated_at: string;
}

export enum NotificationType {
  MeetingReminder = "MeetingReminder",
  PaymentDue = "PaymentDue",
  DocumentShared = "DocumentShared",
  TicketUpdate = "TicketUpdate",
  TicketAssigned = "TicketAssigned",
  TicketResolved = "TicketResolved",
  SystemAlert = "SystemAlert",
  AccountUpdate = "AccountUpdate",
  NewMessage = "NewMessage",
  ConvocationSent = "ConvocationSent",
  ResolutionVoting = "ResolutionVoting",
  QuoteReceived = "QuoteReceived",
  QuoteAccepted = "QuoteAccepted",
  PaymentSuccess = "PaymentSuccess",
  PaymentFailed = "PaymentFailed",
  BudgetApproved = "BudgetApproved",
  EtatDateReady = "EtatDateReady",
  ExchangeRequested = "ExchangeRequested",
  ExchangeCompleted = "ExchangeCompleted",
  AchievementEarned = "AchievementEarned",
  ChallengeStarted = "ChallengeStarted",
  ChallengeCompleted = "ChallengeCompleted",
}

export enum NotificationChannel {
  Email = "Email",
  Sms = "Sms",
  Push = "Push",
  InApp = "InApp",
}

export enum NotificationStatus {
  Pending = "Pending",
  Sent = "Sent",
  Delivered = "Delivered",
  Failed = "Failed",
}

export interface NotificationPreference {
  id: string;
  user_id: string;
  notification_type: NotificationType;
  enabled: boolean;
  email_enabled: boolean;
  sms_enabled: boolean;
  push_enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface NotificationStats {
  total_notifications: number;
  unread_count: number;
  by_type: Record<NotificationType, number>;
  by_channel: Record<NotificationChannel, number>;
  by_status: Record<NotificationStatus, number>;
}

export interface CreateNotificationDto {
  user_id: string;
  title: string;
  message: string;
  notification_type: NotificationType;
  channel: NotificationChannel;
  metadata?: Record<string, any>;
}

export interface UpdateNotificationPreferenceDto {
  enabled?: boolean;
  email_enabled?: boolean;
  sms_enabled?: boolean;
  push_enabled?: boolean;
}

/**
 * Notifications API functions
 */
export const notificationsApi = {
  /**
   * Create a new notification
   */
  async create(data: CreateNotificationDto): Promise<Notification> {
    return api.post("/notifications", data);
  },

  /**
   * Get notification by ID
   */
  async getById(id: string): Promise<Notification> {
    return api.get(`/notifications/${id}`);
  },

  /**
   * List my notifications
   */
  async listMy(): Promise<Notification[]> {
    return api.get("/notifications/my");
  },

  /**
   * List unread notifications
   */
  async getUnread(): Promise<Notification[]> {
    return api.get("/notifications/unread");
  },

  /**
   * Mark notification as read
   */
  async markAsRead(id: string): Promise<void> {
    return api.put(`/notifications/${id}/read`, {});
  },

  /**
   * Mark all notifications as read
   */
  async markAllAsRead(): Promise<void> {
    return api.put("/notifications/read-all", {});
  },

  /**
   * Delete notification
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/notifications/${id}`);
  },

  /**
   * Get notification statistics
   */
  async getStats(): Promise<NotificationStats> {
    return api.get("/notifications/stats");
  },

  /**
   * Get all notification preferences for user
   */
  async getPreferences(userId: string): Promise<NotificationPreference[]> {
    return api.get(`/notification-preferences/${userId}`);
  },

  /**
   * Get specific notification preference
   */
  async getPreference(
    userId: string,
    notificationType: NotificationType,
  ): Promise<NotificationPreference> {
    return api.get(`/notification-preferences/${userId}/${notificationType}`);
  },

  /**
   * Update notification preference
   */
  async updatePreference(
    userId: string,
    notificationType: NotificationType,
    data: UpdateNotificationPreferenceDto,
  ): Promise<NotificationPreference> {
    return api.put(
      `/notification-preferences/${userId}/${notificationType}`,
      data,
    );
  },
};
