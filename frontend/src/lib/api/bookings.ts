import { api } from "../api";

/**
 * Resource Booking Calendar API Client
 * Common area reservations (meeting rooms, parking, gym, etc.)
 */

export interface BookableResource {
  id: string;
  building_id: string;
  resource_name: string;
  resource_type: ResourceType;
  description: string;
  capacity?: number;
  hourly_rate_credits?: number;
  requires_approval: boolean;
  advance_booking_days: number;
  max_booking_duration_hours: number;
  availability_schedule?: Record<string, any>;
  amenities?: string[];
  image_urls?: string[];
  status: ResourceStatus;
  total_bookings: number;
  rating?: number;
  created_at: string;
  updated_at: string;
}

export enum ResourceType {
  MeetingRoom = "MeetingRoom",
  PartyRoom = "PartyRoom",
  Gym = "Gym",
  SwimmingPool = "SwimmingPool",
  Sauna = "Sauna",
  ParkingSpace = "ParkingSpace",
  GuestRoom = "GuestRoom",
  Rooftop = "Rooftop",
  Garden = "Garden",
  LaundryRoom = "LaundryRoom",
  StorageSpace = "StorageSpace",
  CoworkingSpace = "CoworkingSpace",
  Other = "Other",
}

export enum ResourceStatus {
  Available = "Available",
  Unavailable = "Unavailable",
  Maintenance = "Maintenance",
  Retired = "Retired",
}

export interface Booking {
  id: string;
  resource_id: string;
  resource_name?: string;
  owner_id: string;
  owner_name?: string;
  start_time: string;
  end_time: string;
  duration_hours: number;
  status: BookingStatus;
  cost_credits?: number;
  purpose?: string;
  attendees_count?: number;
  special_requests?: string;
  approval_notes?: string;
  cancellation_reason?: string;
  rating?: number;
  created_at: string;
  updated_at: string;
}

export enum BookingStatus {
  Pending = "Pending",
  Approved = "Approved",
  Active = "Active",
  Completed = "Completed",
  Cancelled = "Cancelled",
  Rejected = "Rejected",
}

export interface AvailabilitySlot {
  start_time: string;
  end_time: string;
  is_available: boolean;
  existing_booking_id?: string;
}

export interface CreateBookableResourceDto {
  building_id: string;
  resource_name: string;
  resource_type: ResourceType;
  description: string;
  capacity?: number;
  hourly_rate_credits?: number;
  requires_approval: boolean;
  advance_booking_days: number;
  max_booking_duration_hours: number;
  availability_schedule?: Record<string, any>;
  amenities?: string[];
}

export interface CreateBookingDto {
  resource_id: string;
  owner_id: string;
  start_time: string;
  end_time: string;
  purpose?: string;
  attendees_count?: number;
  special_requests?: string;
}

export const bookingsApi = {
  // Resources
  async createResource(
    data: CreateBookableResourceDto,
  ): Promise<BookableResource> {
    return api.post("/bookable-resources", data);
  },

  async getResourceById(id: string): Promise<BookableResource> {
    return api.get(`/bookable-resources/${id}`);
  },

  async listResourcesByBuilding(
    buildingId: string,
  ): Promise<BookableResource[]> {
    return api.get(`/buildings/${buildingId}/bookable-resources`);
  },

  async listAvailableResources(
    buildingId: string,
  ): Promise<BookableResource[]> {
    return api.get(`/buildings/${buildingId}/bookable-resources/available`);
  },

  async listResourcesByType(
    buildingId: string,
    resourceType: ResourceType,
  ): Promise<BookableResource[]> {
    return api.get(
      `/buildings/${buildingId}/bookable-resources/type/${resourceType}`,
    );
  },

  async updateResource(
    id: string,
    data: Partial<BookableResource>,
  ): Promise<BookableResource> {
    return api.put(`/bookable-resources/${id}`, data);
  },

  async setResourceStatus(
    id: string,
    status: ResourceStatus,
  ): Promise<BookableResource> {
    return api.put(`/bookable-resources/${id}/status`, { status });
  },

  async deleteResource(id: string): Promise<void> {
    return api.delete(`/bookable-resources/${id}`);
  },

  // Bookings
  async createBooking(data: CreateBookingDto): Promise<Booking> {
    return api.post("/bookings", data);
  },

  async getBookingById(id: string): Promise<Booking> {
    return api.get(`/bookings/${id}`);
  },

  async listBookingsByResource(resourceId: string): Promise<Booking[]> {
    return api.get(`/bookable-resources/${resourceId}/bookings`);
  },

  async listBookingsByOwner(ownerId: string): Promise<Booking[]> {
    return api.get(`/owners/${ownerId}/bookings`);
  },

  async listUpcomingBookings(buildingId: string): Promise<Booking[]> {
    return api.get(`/buildings/${buildingId}/bookings/upcoming`);
  },

  async listPendingBookings(buildingId: string): Promise<Booking[]> {
    return api.get(`/buildings/${buildingId}/bookings/pending`);
  },

  async approveBooking(id: string, notes?: string): Promise<Booking> {
    return api.put(`/bookings/${id}/approve`, { notes });
  },

  async rejectBooking(id: string, notes: string): Promise<Booking> {
    return api.put(`/bookings/${id}/reject`, { notes });
  },

  async startBooking(id: string): Promise<Booking> {
    return api.put(`/bookings/${id}/start`, {});
  },

  async completeBooking(id: string): Promise<Booking> {
    return api.put(`/bookings/${id}/complete`, {});
  },

  async cancelBooking(id: string, reason: string): Promise<Booking> {
    return api.put(`/bookings/${id}/cancel`, { reason });
  },

  async rateBooking(id: string, rating: number): Promise<Booking> {
    return api.put(`/bookings/${id}/rate`, { rating });
  },

  // Availability
  async checkAvailability(
    resourceId: string,
    startTime: string,
    endTime: string,
  ): Promise<{ is_available: boolean; conflicting_bookings?: Booking[] }> {
    return api.post(`/bookable-resources/${resourceId}/check-availability`, {
      start_time: startTime,
      end_time: endTime,
    });
  },

  async getAvailabilitySlots(
    resourceId: string,
    date: string,
  ): Promise<AvailabilitySlot[]> {
    return api.get(
      `/bookable-resources/${resourceId}/availability-slots?date=${date}`,
    );
  },

  // Statistics
  async getResourceStats(resourceId: string): Promise<any> {
    return api.get(`/bookable-resources/${resourceId}/stats`);
  },

  async getBuildingBookingStats(buildingId: string): Promise<any> {
    return api.get(`/buildings/${buildingId}/booking-stats`);
  },
};
