import { api } from "../api";

/**
 * Energy Campaigns API Client
 * Belgian CREG-compliant energy buying groups with GDPR data protection
 * Target: 15-25% savings on energy bills through collective negotiation
 */

export interface EnergyCampaign {
  id: string;
  organization_id: string;
  building_id?: string;
  campaign_name: string;
  status: CampaignStatus;
  energy_types: EnergyType[];
  total_participants: number;
  total_kwh_electricity?: number;
  total_kwh_gas?: number;
  total_kwh_heating?: number;
  offers_received: ProviderOffer[];
  selected_offer_id?: string;
  campaign_start_date: string;
  campaign_end_date: string;
  created_at: string;
  updated_at: string;
}

export enum CampaignStatus {
  Draft = "Draft",
  CollectingData = "CollectingData",
  Negotiating = "Negotiating",
  AwaitingFinalVote = "AwaitingFinalVote",
  Finalized = "Finalized",
  Completed = "Completed",
}

export enum EnergyType {
  Electricity = "Electricity",
  Gas = "Gas",
  Heating = "Heating",
}

export interface ProviderOffer {
  id: string;
  provider_name: string;
  energy_type: EnergyType;
  price_per_kwh_cents: number;
  contract_duration_months: number;
  fixed_monthly_fee_cents?: number;
  green_energy_percentage: number;
  estimated_annual_savings_cents?: number;
  offer_valid_until: string;
  terms_and_conditions_url?: string;
  created_at: string;
}

export interface EnergyBillUpload {
  id: string;
  campaign_id: string;
  unit_id: string;
  building_id: string;
  organization_id: string;
  billing_period_start: string;
  billing_period_end: string;
  energy_type: EnergyType;
  verified: boolean;
  verified_by?: string;
  verified_at?: string;
  consent_timestamp: string;
  consent_signature_hash: string;
  retention_until: string;
  deleted_at?: string;
  created_at: string;
  updated_at: string;
}

export interface CampaignStatistics {
  campaign_id: string;
  total_participants: number;
  total_kwh_aggregated?: number;
  average_kwh_per_unit?: number;
  k_anonymity_compliant: boolean;
  min_participants_required: number;
  offers_count: number;
  best_offer_savings_percentage?: number;
}

export interface CreateCampaignDto {
  organization_id: string;
  building_id?: string;
  campaign_name: string;
  energy_types: EnergyType[];
  campaign_start_date: string;
  campaign_end_date: string;
}

export interface UpdateCampaignStatusDto {
  new_status: CampaignStatus;
}

export interface CreateProviderOfferDto {
  provider_name: string;
  energy_type: EnergyType;
  price_per_kwh_cents: number;
  contract_duration_months: number;
  fixed_monthly_fee_cents?: number;
  green_energy_percentage: number;
  estimated_annual_savings_cents?: number;
  offer_valid_until: string;
  terms_and_conditions_url?: string;
}

export interface SelectOfferDto {
  offer_id: string;
}

export interface UploadEnergyBillDto {
  campaign_id: string;
  unit_id: string;
  billing_period_start: string;
  billing_period_end: string;
  energy_type: EnergyType;
  total_kwh: number;
  consent_signature: string;
}

export interface VerifyUploadDto {
  verified: boolean;
  verified_by: string;
  verification_notes?: string;
}

export interface DecryptedConsumption {
  upload_id: string;
  total_kwh: number;
  billing_period_start: string;
  billing_period_end: string;
  energy_type: EnergyType;
}

/**
 * Energy Campaigns API functions
 */
export const energyCampaignsApi = {
  /**
   * Create a new energy buying campaign
   */
  async create(data: CreateCampaignDto): Promise<EnergyCampaign> {
    return api.post("/energy-campaigns", data);
  },

  /**
   * List all campaigns for organization
   */
  async list(organizationId?: string): Promise<EnergyCampaign[]> {
    let url = "/energy-campaigns";
    if (organizationId) {
      url += `?organization_id=${organizationId}`;
    }
    return api.get(url);
  },

  /**
   * Get campaign by ID
   */
  async getById(id: string): Promise<EnergyCampaign> {
    return api.get(`/energy-campaigns/${id}`);
  },

  /**
   * Get campaign statistics (anonymized, k-anonymity compliant)
   */
  async getStats(id: string): Promise<CampaignStatistics> {
    return api.get(`/energy-campaigns/${id}/stats`);
  },

  /**
   * Update campaign status
   */
  async updateStatus(
    id: string,
    data: UpdateCampaignStatusDto,
  ): Promise<EnergyCampaign> {
    return api.put(`/energy-campaigns/${id}/status`, data);
  },

  /**
   * Delete campaign
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/energy-campaigns/${id}`);
  },

  /**
   * Add provider offer (broker/admin only)
   */
  async addOffer(
    campaignId: string,
    data: CreateProviderOfferDto,
  ): Promise<ProviderOffer> {
    return api.post(`/energy-campaigns/${campaignId}/offers`, data);
  },

  /**
   * List all offers for campaign
   */
  async listOffers(campaignId: string): Promise<ProviderOffer[]> {
    return api.get(`/energy-campaigns/${campaignId}/offers`);
  },

  /**
   * Select winning offer (after vote)
   */
  async selectOffer(
    campaignId: string,
    data: SelectOfferDto,
  ): Promise<EnergyCampaign> {
    return api.post(`/energy-campaigns/${campaignId}/select-offer`, data);
  },

  /**
   * Finalize campaign (after final vote)
   */
  async finalize(campaignId: string): Promise<EnergyCampaign> {
    return api.post(`/energy-campaigns/${campaignId}/finalize`, {});
  },
};

/**
 * Energy Bill Upload API functions
 */
export const energyBillsApi = {
  /**
   * Upload energy bill with GDPR consent
   */
  async upload(data: UploadEnergyBillDto): Promise<EnergyBillUpload> {
    return api.post("/energy-bills/upload", data);
  },

  /**
   * Get my energy bill uploads
   */
  async getMyUploads(): Promise<EnergyBillUpload[]> {
    return api.get("/energy-bills/my-uploads");
  },

  /**
   * Get upload by ID
   */
  async getById(id: string): Promise<EnergyBillUpload> {
    return api.get(`/energy-bills/${id}`);
  },

  /**
   * Decrypt consumption data (owner only)
   */
  async decryptConsumption(id: string): Promise<DecryptedConsumption> {
    return api.get(`/energy-bills/${id}/decrypt`);
  },

  /**
   * Verify upload (admin only)
   */
  async verify(
    id: string,
    data: VerifyUploadDto,
  ): Promise<EnergyBillUpload> {
    return api.put(`/energy-bills/${id}/verify`, data);
  },

  /**
   * Delete upload (GDPR Art. 17 - Right to erasure)
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/energy-bills/${id}`);
  },

  /**
   * Withdraw GDPR consent (Art. 7.3 - Immediate deletion)
   */
  async withdrawConsent(id: string): Promise<void> {
    return api.post(`/energy-bills/${id}/withdraw-consent`, {});
  },

  /**
   * Get all uploads for campaign (admin)
   */
  async getByCampaign(campaignId: string): Promise<EnergyBillUpload[]> {
    return api.get(`/energy-campaigns/${campaignId}/uploads`);
  },
};
