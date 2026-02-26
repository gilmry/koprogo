import { api } from "../api";

/**
 * Notice Board API Client
 * Community announcements and classified ads
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

export enum NoticeType {
  Announcement = "Announcement",
  Event = "Event",
  LostAndFound = "LostAndFound",
  ClassifiedAd = "ClassifiedAd",
}

export enum NoticeCategory {
  General = "General",
  Maintenance = "Maintenance",
  Social = "Social",
  Security = "Security",
  Environment = "Environment",
  Parking = "Parking",
  Other = "Other",
}

export enum NoticeStatus {
  Draft = "Draft",
  Published = "Published",
  Archived = "Archived",
  Expired = "Expired",
}

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
