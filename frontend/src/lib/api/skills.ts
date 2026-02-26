import { api } from "../api";

/**
 * Skills Directory API Client
 * Resident skills marketplace and offers
 */

export interface SkillOffer {
  id: string;
  building_id: string;
  owner_id: string;
  owner_name: string;
  skill_category: SkillCategory;
  skill_name: string;
  expertise_level: ExpertiseLevel;
  description: string;
  is_available_for_help: boolean;
  hourly_rate_credits?: number;
  years_of_experience?: number;
  certifications?: string;
  is_free: boolean;
  is_professional: boolean;
  created_at: string;
  updated_at: string;
}

export enum SkillCategory {
  HomeRepair = "HomeRepair",
  Languages = "Languages",
  Technology = "Technology",
  Education = "Education",
  Arts = "Arts",
  Sports = "Sports",
  Cooking = "Cooking",
  Gardening = "Gardening",
  Health = "Health",
  Legal = "Legal",
  Financial = "Financial",
  PetCare = "PetCare",
  Other = "Other",
}

export enum ExpertiseLevel {
  Beginner = "Beginner",
  Intermediate = "Intermediate",
  Advanced = "Advanced",
  Expert = "Expert",
}

export interface CreateSkillOfferDto {
  building_id: string;
  skill_category: SkillCategory;
  skill_name: string;
  expertise_level: ExpertiseLevel;
  description: string;
  is_available_for_help: boolean;
  hourly_rate_credits?: number;
  years_of_experience?: number;
  certifications?: string;
}

export const skillsApi = {
  // Skill Offers
  async createOffer(data: CreateSkillOfferDto): Promise<SkillOffer> {
    return api.post("/skills", data);
  },

  async getOfferById(id: string): Promise<SkillOffer> {
    return api.get(`/skills/${id}`);
  },

  async listOffersByBuilding(buildingId: string): Promise<SkillOffer[]> {
    return api.get(`/buildings/${buildingId}/skills`);
  },

  async listAvailableOffers(buildingId: string): Promise<SkillOffer[]> {
    return api.get(`/buildings/${buildingId}/skills/available`);
  },

  async listFreeOffers(buildingId: string): Promise<SkillOffer[]> {
    return api.get(`/buildings/${buildingId}/skills/free`);
  },

  async listProfessionalOffers(buildingId: string): Promise<SkillOffer[]> {
    return api.get(`/buildings/${buildingId}/skills/professional`);
  },

  async listOffersByCategory(
    buildingId: string,
    category: SkillCategory,
  ): Promise<SkillOffer[]> {
    return api.get(`/buildings/${buildingId}/skills/category/${category}`);
  },

  async listOffersByExpertise(
    buildingId: string,
    level: ExpertiseLevel,
  ): Promise<SkillOffer[]> {
    return api.get(`/buildings/${buildingId}/skills/expertise/${level}`);
  },

  async listOffersByOwner(ownerId: string): Promise<SkillOffer[]> {
    return api.get(`/owners/${ownerId}/skills`);
  },

  async updateOffer(
    id: string,
    data: Partial<SkillOffer>,
  ): Promise<SkillOffer> {
    return api.put(`/skills/${id}`, data);
  },

  async markAvailable(id: string): Promise<SkillOffer> {
    return api.post(`/skills/${id}/mark-available`, {});
  },

  async markUnavailable(id: string): Promise<SkillOffer> {
    return api.post(`/skills/${id}/mark-unavailable`, {});
  },

  async deleteOffer(id: string): Promise<void> {
    return api.delete(`/skills/${id}`);
  },

  // Statistics
  async getBuildingSkillStats(buildingId: string): Promise<any> {
    return api.get(`/buildings/${buildingId}/skills/statistics`);
  },
};
