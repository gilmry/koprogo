import { api } from "../api";
import type { components } from "../../types/api";

/**
 * Gamification & Achievements API Client
 * Community engagement system
 *
 * Enums are re-exported from auto-generated api.d.ts (STORY-P7-704) —
 * TypeScript will refuse any value that doesn't exist in the Rust enum.
 */

export interface Achievement {
  id: string;
  organization_id: string;
  category: AchievementCategory;
  tier: AchievementTier;
  name: string;
  description: string;
  icon: string;
  points_value: number;
  requirements: Record<string, any>;
  is_secret: boolean;
  is_repeatable: boolean;
  display_order: number;
  created_at: string;
  updated_at: string;
}

// Re-exported from generated api.d.ts — single source of truth.
export type AchievementCategory = components["schemas"]["AchievementCategory"];
export const AchievementCategory = {
  Community: "Community" as const,
  Sel: "Sel" as const,
  Booking: "Booking" as const,
  Sharing: "Sharing" as const,
  Skills: "Skills" as const,
  Notice: "Notice" as const,
  Governance: "Governance" as const,
  Milestone: "Milestone" as const,
} satisfies Record<string, AchievementCategory>;

export type AchievementTier = components["schemas"]["AchievementTier"];
export const AchievementTier = {
  Bronze: "Bronze" as const,
  Silver: "Silver" as const,
  Gold: "Gold" as const,
  Platinum: "Platinum" as const,
  Diamond: "Diamond" as const,
} satisfies Record<string, AchievementTier>;

export interface UserAchievement {
  id: string;
  user_id: string;
  achievement_id: string;
  achievement?: Achievement;
  earned_at: string;
  times_earned: number;
}

export interface Challenge {
  id: string;
  organization_id: string;
  building_id?: string;
  challenge_type: ChallengeType;
  status: ChallengeStatus;
  title: string;
  description: string;
  icon: string;
  start_date: string;
  end_date: string;
  target_metric: string;
  target_value: number;
  reward_points: number;
  created_at: string;
  updated_at: string;
}

export type ChallengeType = components["schemas"]["ChallengeType"];
export const ChallengeType = {
  Individual: "Individual" as const,
  Team: "Team" as const,
  Building: "Building" as const,
} satisfies Record<string, ChallengeType>;

export type ChallengeStatus = components["schemas"]["ChallengeStatus"];
export const ChallengeStatus = {
  Draft: "Draft" as const,
  Active: "Active" as const,
  Completed: "Completed" as const,
  Cancelled: "Cancelled" as const,
} satisfies Record<string, ChallengeStatus>;

export interface ChallengeProgress {
  id: string;
  challenge_id: string;
  user_id: string;
  current_value: number;
  completed: boolean;
  completed_at?: string;
}

export interface LeaderboardEntry {
  user_id: string;
  user_name: string;
  total_points: number;
  achievements_count: number;
  challenges_completed: number;
  rank: number;
}

export const gamificationApi = {
  // Achievements (7 endpoints)
  async createAchievement(data: any): Promise<Achievement> {
    return api.post("/achievements", data);
  },

  async getAchievement(id: string): Promise<Achievement> {
    return api.get(`/achievements/${id}`);
  },

  async listAchievements(organizationId: string): Promise<Achievement[]> {
    return api.get(`/organizations/${organizationId}/achievements`);
  },

  async listByCategory(
    organizationId: string,
    category: AchievementCategory,
  ): Promise<Achievement[]> {
    return api.get(
      `/organizations/${organizationId}/achievements/category/${category}`,
    );
  },

  async getVisibleAchievements(organizationId: string): Promise<Achievement[]> {
    return api.get(`/organizations/${organizationId}/achievements/visible`);
  },

  async updateAchievement(id: string, data: any): Promise<Achievement> {
    return api.put(`/achievements/${id}`, data);
  },

  async deleteAchievement(id: string): Promise<void> {
    return api.delete(`/achievements/${id}`);
  },

  // User Achievements (3 endpoints)
  async awardAchievement(data: {
    user_id: string;
    achievement_id: string;
  }): Promise<UserAchievement> {
    return api.post("/users/achievements", data);
  },

  async getUserAchievements(_userId?: string): Promise<UserAchievement[]> {
    return api.get(`/users/achievements`);
  },

  async getRecentAchievements(
    _userId?: string,
    limit = 10,
  ): Promise<UserAchievement[]> {
    return api.get(`/users/achievements/recent?limit=${limit}`);
  },

  // Challenges (9 endpoints)
  async createChallenge(data: any): Promise<Challenge> {
    return api.post("/challenges", data);
  },

  async getChallenge(id: string): Promise<Challenge> {
    return api.get(`/challenges/${id}`);
  },

  async listChallenges(organizationId: string): Promise<Challenge[]> {
    return api.get(`/organizations/${organizationId}/challenges`);
  },

  async listByStatus(
    organizationId: string,
    status: ChallengeStatus,
  ): Promise<Challenge[]> {
    return api.get(
      `/organizations/${organizationId}/challenges/status/${status}`,
    );
  },

  async getActiveChallenges(organizationId: string): Promise<Challenge[]> {
    return api.get(`/organizations/${organizationId}/challenges/active`);
  },

  async listBuildingChallenges(buildingId: string): Promise<Challenge[]> {
    return api.get(`/buildings/${buildingId}/challenges`);
  },

  async activateChallenge(id: string): Promise<Challenge> {
    return api.put(`/challenges/${id}/activate`, {});
  },

  async completeChallenge(id: string): Promise<Challenge> {
    return api.put(`/challenges/${id}/complete`, {});
  },

  async cancelChallenge(id: string): Promise<Challenge> {
    return api.put(`/challenges/${id}/cancel`, {});
  },

  async deleteChallenge(id: string): Promise<void> {
    return api.delete(`/challenges/${id}`);
  },

  // Challenge Progress (4 endpoints)
  async getChallengeProgress(
    challengeId: string,
    _userId?: string,
  ): Promise<ChallengeProgress> {
    return api.get(`/challenges/${challengeId}/progress`);
  },

  async listChallengeProgress(
    challengeId: string,
  ): Promise<ChallengeProgress[]> {
    return api.get(`/challenges/${challengeId}/all-progress`);
  },

  async getUserActiveChallenges(
    _userId?: string,
  ): Promise<ChallengeProgress[]> {
    return api.get(`/users/challenges/active`);
  },

  async incrementProgress(
    challengeId: string,
    userId: string,
  ): Promise<ChallengeProgress> {
    return api.post(`/challenges/${challengeId}/progress/increment`, {
      user_id: userId,
    });
  },

  // Statistics (2 endpoints)
  async getUserStats(organizationId: string): Promise<any> {
    return api.get(`/organizations/${organizationId}/gamification/stats`);
  },

  async getLeaderboard(
    organizationId: string,
    buildingId?: string,
    limit = 10,
  ): Promise<LeaderboardEntry[]> {
    let url = `/organizations/${organizationId}/gamification/leaderboard?limit=${limit}`;
    if (buildingId) url += `&building_id=${buildingId}`;
    return api.get(url);
  },
};
