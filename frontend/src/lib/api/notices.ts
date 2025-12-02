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
  category: string;
  title: string;
  content: string;
  visibility: NoticeVisibility;
  status: NoticeStatus;
  expires_at?: string;
  contact_info?: string;
  image_urls?: string[];
  view_count: number;
  created_at: string;
  updated_at: string;
}

export enum NoticeType {
  Announcement = "Announcement",
  ForSale = "ForSale",
  WantedToBuy = "WantedToBuy",
  LostAndFound = "LostAndFound",
  Event = "Event",
  Alert = "Alert",
}

export enum NoticeVisibility {
  Public = "Public",
  BuildingOnly = "BuildingOnly",
  OwnersOnly = "OwnersOnly",
}

export enum NoticeStatus {
  Active = "Active",
  Expired = "Expired",
  Archived = "Archived",
  Moderated = "Moderated",
}

export interface CreateNoticeDto {
  building_id: string;
  author_id: string;
  notice_type: NoticeType;
  category: string;
  title: string;
  content: string;
  visibility: NoticeVisibility;
  expires_at?: string;
  contact_info?: string;
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
    return api.get(`/buildings/${buildingId}/notices/active`);
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
    return api.put(`/notices/${id}/archive`, {});
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/notices/${id}`);
  },

  async incrementViewCount(id: string): Promise<void> {
    return api.post(`/notices/${id}/view`, {});
  },

  async getExpiredNotices(buildingId: string): Promise<Notice[]> {
    return api.get(`/buildings/${buildingId}/notices/expired`);
  },
};
