import { api } from "../api";
import type { components } from "../../types/api";

/**
 * Notice Board API Client
 * Community announcements and classified ads
 *
 * Enums are re-exported from auto-generated api.d.ts (STORY-P7-704) —
 * TypeScript will refuse any value that doesn't exist in the Rust enum.
 */

export interface Notice {
  id: string;
  building_id: string;
  author_id: string;
  author_name?: string;
  notice_type: NoticeType;
  category: NoticeCategory;
  title: string;
  content: string;
  status: NoticeStatus;
  is_pinned: boolean;
  published_at?: string;
  expires_at?: string;
  archived_at?: string;
  event_date?: string;
  event_location?: string;
  contact_info?: string;
  is_expired: boolean;
  days_until_event?: number;
  created_at: string;
  updated_at: string;
}

// Re-exported from generated api.d.ts — single source of truth.
export type NoticeType = components["schemas"]["NoticeType"];
export const NoticeType = {
  Announcement: "Announcement" as const,
  Event: "Event" as const,
  LostAndFound: "LostAndFound" as const,
  ClassifiedAd: "ClassifiedAd" as const,
} satisfies Record<string, NoticeType>;

export type NoticeCategory = components["schemas"]["NoticeCategory"];
export const NoticeCategory = {
  General: "General" as const,
  Maintenance: "Maintenance" as const,
  Social: "Social" as const,
  Security: "Security" as const,
  Environment: "Environment" as const,
  Parking: "Parking" as const,
  Other: "Other" as const,
} satisfies Record<string, NoticeCategory>;

export type NoticeStatus = components["schemas"]["NoticeStatus"];
export const NoticeStatus = {
  Draft: "Draft" as const,
  Published: "Published" as const,
  Archived: "Archived" as const,
  Expired: "Expired" as const,
} satisfies Record<string, NoticeStatus>;

export interface CreateNoticeDto {
  building_id: string;
  notice_type: NoticeType;
  category: NoticeCategory;
  title: string;
  content: string;
  event_date?: string;
  event_location?: string;
  contact_info?: string;
  expires_at?: string;
}

export const noticesApi = {
  async create(data: CreateNoticeDto): Promise<Notice> {
    return api.post("/notices", data);
  },

  async getById(id: string): Promise<Notice> {
    return api.get(`/notices/${id}`);
  },

  async listByBuilding(buildingId: string): Promise<Notice[]> {
    return api.get(`/buildings/${buildingId}/notices`);
  },

  async listActive(buildingId: string): Promise<Notice[]> {
    return api.get(`/buildings/${buildingId}/notices/published`);
  },

  async listByType(
    buildingId: string,
    noticeType: NoticeType,
  ): Promise<Notice[]> {
    return api.get(`/buildings/${buildingId}/notices/type/${noticeType}`);
  },

  async listByAuthor(authorId: string): Promise<Notice[]> {
    return api.get(`/owners/${authorId}/notices`);
  },

  async update(id: string, data: Partial<Notice>): Promise<Notice> {
    return api.put(`/notices/${id}`, data);
  },

  async archive(id: string): Promise<Notice> {
    return api.post(`/notices/${id}/archive`, {});
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/notices/${id}`);
  },

  async incrementViewCount(id: string): Promise<void> {
    return api.post(`/notices/${id}/view`, {});
  },

  async getExpiredNotices(buildingId: string): Promise<Notice[]> {
    return api.get(`/buildings/${buildingId}/notices/status/Expired`);
  },
};
