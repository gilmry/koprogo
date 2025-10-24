// Synchronization service for offline/online data sync
import { localDB } from "./db";
import type { Building, Owner, Expense } from "./types";
import { API_URL } from "./config";

const API_BASE_URL = API_URL;

export class SyncService {
  private isOnline: boolean =
    typeof navigator !== "undefined" ? navigator.onLine : false;
  private syncInProgress: boolean = false;
  private token: string | null = null;

  constructor() {
    // Only setup event listeners on the client side
    if (typeof window !== "undefined") {
      // Listen to online/offline events
      window.addEventListener("online", () => {
        console.log("🟢 Application is online");
        this.isOnline = true;
        this.sync();
      });

      window.addEventListener("offline", () => {
        console.log("🔴 Application is offline");
        this.isOnline = false;
      });
    }
  }

  setToken(token: string | null) {
    this.token = token;
  }

  getOnlineStatus(): boolean {
    return this.isOnline;
  }

  private async fetchWithAuth(
    url: string,
    options: RequestInit = {},
  ): Promise<Response> {
    const headers = new Headers(options.headers);

    if (this.token) {
      headers.set("Authorization", `Bearer ${this.token}`);
    }
    headers.set("Content-Type", "application/json");

    return fetch(url, {
      ...options,
      headers,
    });
  }

  // Sync all pending changes to the backend
  async sync(): Promise<void> {
    if (!this.isOnline || this.syncInProgress || !this.token) {
      return;
    }

    this.syncInProgress = true;
    console.log("🔄 Starting synchronization...");

    try {
      const queue = await localDB.getSyncQueue();
      const unsyncedItems = queue.filter((item) => !item.synced);

      for (const item of unsyncedItems) {
        try {
          await this.syncItem(item);
          if (item.id) {
            await localDB.markSynced(item.id);
          }
        } catch (error) {
          console.error(`Failed to sync item:`, error);
          // Continue with next item even if one fails
        }
      }

      // Clean up synced items
      await localDB.clearSyncedItems();

      // Fetch fresh data from server
      await this.fetchAllData();

      console.log("✅ Synchronization completed");
    } catch (error) {
      console.error("❌ Synchronization failed:", error);
    } finally {
      this.syncInProgress = false;
    }
  }

  private async syncItem(item: any): Promise<void> {
    const { action, entity, data } = item;
    let url = `${API_BASE_URL}/${entity}`;

    switch (action) {
      case "create":
        await this.fetchWithAuth(url, {
          method: "POST",
          body: JSON.stringify(data),
        });
        break;

      case "update":
        url = `${url}/${data.id}`;
        await this.fetchWithAuth(url, {
          method: "PUT",
          body: JSON.stringify(data),
        });
        break;

      case "delete":
        url = `${url}/${data.id}`;
        await this.fetchWithAuth(url, {
          method: "DELETE",
        });
        break;
    }
  }

  // Fetch all data from backend and store locally
  private async fetchAllData(): Promise<void> {
    if (!this.isOnline || !this.token) return;

    try {
      // Fetch buildings
      const buildingsRes = await this.fetchWithAuth(
        `${API_BASE_URL}/buildings`,
      );
      if (buildingsRes.ok) {
        const response = await buildingsRes.json();
        // API returns {data: [...], pagination: {...}}
        const buildings = response.data || response;
        await localDB.saveBuildings(buildings);
      }

      // Fetch owners
      const ownersRes = await this.fetchWithAuth(`${API_BASE_URL}/owners`);
      if (ownersRes.ok) {
        const response = await ownersRes.json();
        // API returns {data: [...], pagination: {...}}
        const owners = response.data || response;
        await localDB.saveOwners(owners);
      }

      // Note: Units and expenses might need building-specific endpoints
    } catch (error) {
      console.error("Failed to fetch data from server:", error);
    }
  }

  // API methods with offline support
  async getBuildings(): Promise<Building[]> {
    if (this.isOnline && this.token) {
      try {
        const response = await this.fetchWithAuth(`${API_BASE_URL}/buildings`);
        if (response.ok) {
          const result = await response.json();
          // API returns {data: [...], pagination: {...}}
          const buildings = result.data || result;
          await localDB.saveBuildings(buildings);
          return buildings;
        }
      } catch (error) {
        console.log("Falling back to local data");
      }
    }

    // Fallback to local data
    return localDB.getBuildings();
  }

  async createBuilding(building: Partial<Building>): Promise<Building | null> {
    if (this.isOnline && this.token) {
      try {
        const response = await this.fetchWithAuth(`${API_BASE_URL}/buildings`, {
          method: "POST",
          body: JSON.stringify(building),
        });

        if (response.ok) {
          const newBuilding = await response.json();
          await localDB.put("buildings", newBuilding);
          return newBuilding;
        }
      } catch (error) {
        console.log("Offline: queueing building creation");
      }
    }

    // Queue for later sync
    await localDB.addToSyncQueue("create", "buildings", building);

    // Create temporary local record
    const tempBuilding = {
      id: `temp-${Date.now()}`,
      ...building,
      createdAt: new Date().toISOString(),
    } as Building;

    await localDB.put("buildings", tempBuilding);
    return tempBuilding;
  }

  async getOwners(): Promise<Owner[]> {
    if (this.isOnline && this.token) {
      try {
        const response = await this.fetchWithAuth(`${API_BASE_URL}/owners`);
        if (response.ok) {
          const result = await response.json();
          // API returns {data: [...], pagination: {...}}
          const owners = result.data || result;
          await localDB.saveOwners(owners);
          return owners;
        }
      } catch (error) {
        console.log("Falling back to local data");
      }
    }

    return localDB.getOwners();
  }

  async getExpenses(): Promise<Expense[]> {
    if (this.isOnline && this.token) {
      try {
        // Note: This might need to be adjusted based on actual API structure
        const buildings = await this.getBuildings();
        const allExpenses: Expense[] = [];

        for (const building of buildings) {
          const response = await this.fetchWithAuth(
            `${API_BASE_URL}/buildings/${building.id}/expenses`,
          );
          if (response.ok) {
            const result = await response.json();
            // API returns {data: [...], pagination: {...}}
            const expenses = result.data || result;
            allExpenses.push(...expenses);
          }
        }

        await localDB.saveExpenses(allExpenses);
        return allExpenses;
      } catch (error) {
        console.log("Falling back to local data");
      }
    }

    return localDB.getExpenses();
  }

  // Initialize sync on login
  async initialize(token: string): Promise<void> {
    this.setToken(token);
    await localDB.init();

    if (this.isOnline) {
      await this.sync();
    }
  }

  // Clear local data on logout
  async clearLocalData(): Promise<void> {
    this.token = null;
    await localDB.clear("users");
    await localDB.clear("buildings");
    await localDB.clear("owners");
    await localDB.clear("units");
    await localDB.clear("expenses");
    await localDB.clear("sync_queue");
  }
}

// Export singleton instance
export const syncService = new SyncService();
