import { api } from "../api";

/**
 * Skills Directory API Client
 * Resident skills marketplace and offers
 */

export interface SkillOffer {
  id: string;
  building_id: string;
  owner_id: string;
  owner_name?: string;
  skill_category: SkillCategory;
  skill_name: string;
  description: string;
  proficiency_level: ProficiencyLevel;
  hourly_rate_credits?: number;
  availability: string;
  certifications?: string[];
  years_experience?: number;
  status: SkillStatus;
  rating?: number;
  total_requests: number;
  completed_requests: number;
  created_at: string;
  updated_at: string;
}

export enum SkillCategory {
  HomeRepair = "HomeRepair",
  Tutoring = "Tutoring",
  LanguageLessons = "LanguageLessons",
  ITSupport = "ITSupport",
  Cooking = "Cooking",
  Gardening = "Gardening",
  Childcare = "Childcare",
  PetCare = "PetCare",
  Arts = "Arts",
  Music = "Music",
  Sports = "Sports",
  Other = "Other",
}

export enum ProficiencyLevel {
  Beginner = "Beginner",
  Intermediate = "Intermediate",
  Advanced = "Advanced",
  Expert = "Expert",
  Professional = "Professional",
}

export enum SkillStatus {
  Available = "Available",
  Unavailable = "Unavailable",
  Archived = "Archived",
}

export interface SkillRequest {
  id: string;
  skill_offer_id: string;
  requester_id: string;
  requester_name?: string;
  message: string;
  preferred_dates?: string[];
  status: SkillRequestStatus;
  rating?: number;
  feedback?: string;
  created_at: string;
  updated_at: string;
}

export enum SkillRequestStatus {
  Pending = "Pending",
  Accepted = "Accepted",
  Declined = "Declined",
  Completed = "Completed",
  Cancelled = "Cancelled",
}

export interface CreateSkillOfferDto {
  building_id: string;
  owner_id: string;
  skill_category: SkillCategory;
  skill_name: string;
  description: string;
  proficiency_level: ProficiencyLevel;
  hourly_rate_credits?: number;
  availability: string;
  certifications?: string[];
  years_experience?: number;
}

export interface CreateSkillRequestDto {
  skill_offer_id: string;
  requester_id: string;
  message: string;
  preferred_dates?: string[];
}

export const skillsApi = {
  // Skill Offers
  async createOffer(data: CreateSkillOfferDto): Promise<SkillOffer> {
    return api.post("/skill-offers", data);
  },

  async getOfferById(id: string): Promise<SkillOffer> {
    return api.get(`/skill-offers/${id}`);
  },

  async listOffersByBuilding(buildingId: string): Promise<SkillOffer[]> {
    return api.get(`/buildings/${buildingId}/skill-offers`);
  },

  async listAvailableOffers(buildingId: string): Promise<SkillOffer[]> {
    return api.get(`/buildings/${buildingId}/skill-offers/available`);
  },

  async listOffersByCategory(buildingId: string, category: SkillCategory): Promise<SkillOffer[]> {
    return api.get(`/buildings/${buildingId}/skill-offers/category/${category}`);
  },

  async listOffersByOwner(ownerId: string): Promise<SkillOffer[]> {
    return api.get(`/owners/${ownerId}/skill-offers`);
  },

  async updateOffer(id: string, data: Partial<SkillOffer>): Promise<SkillOffer> {
    return api.put(`/skill-offers/${id}`, data);
  },

  async setOfferAvailability(id: string, status: SkillStatus): Promise<SkillOffer> {
    return api.put(`/skill-offers/${id}/status`, { status });
  },

  async deleteOffer(id: string): Promise<void> {
    return api.delete(`/skill-offers/${id}`);
  },

  // Skill Requests
  async createRequest(data: CreateSkillRequestDto): Promise<SkillRequest> {
    return api.post("/skill-requests", data);
  },

  async getRequestById(id: string): Promise<SkillRequest> {
    return api.get(`/skill-requests/${id}`);
  },

  async listRequestsByOffer(offerId: string): Promise<SkillRequest[]> {
    return api.get(`/skill-offers/${offerId}/requests`);
  },

  async listRequestsByRequester(requesterId: string): Promise<SkillRequest[]> {
    return api.get(`/owners/${requesterId}/skill-requests`);
  },

  async acceptRequest(id: string): Promise<SkillRequest> {
    return api.put(`/skill-requests/${id}/accept`, {});
  },

  async declineRequest(id: string): Promise<SkillRequest> {
    return api.put(`/skill-requests/${id}/decline`, {});
  },

  async completeRequest(id: string): Promise<SkillRequest> {
    return api.put(`/skill-requests/${id}/complete`, {});
  },

  async cancelRequest(id: string): Promise<SkillRequest> {
    return api.put(`/skill-requests/${id}/cancel`, {});
  },

  async rateRequest(id: string, rating: number, feedback?: string): Promise<SkillRequest> {
    return api.put(`/skill-requests/${id}/rate`, { rating, feedback });
  },

  // Statistics
  async getOwnerSkillStats(ownerId: string): Promise<any> {
    return api.get(`/owners/${ownerId}/skill-stats`);
  },

  async getBuildingSkillStats(buildingId: string): Promise<any> {
    return api.get(`/buildings/${buildingId}/skill-stats`);
  },
};
