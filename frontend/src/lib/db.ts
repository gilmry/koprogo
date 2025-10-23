// IndexedDB wrapper for offline storage
import type { User, Building, Owner, Unit, Expense } from "./types";

const DB_NAME = "koprogo_db";
const DB_VERSION = 1;

interface SyncQueue {
  id?: number;
  action: "create" | "update" | "delete";
  entity: string;
  data: any;
  timestamp: number;
  synced: boolean;
}

export class LocalDB {
  private db: IDBDatabase | null = null;

  async init(): Promise<void> {
    // Skip initialization on server side
    if (typeof indexedDB === "undefined") {
      return Promise.resolve();
    }

    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        // Create object stores
        if (!db.objectStoreNames.contains("users")) {
          db.createObjectStore("users", { keyPath: "id" });
        }
        if (!db.objectStoreNames.contains("buildings")) {
          db.createObjectStore("buildings", { keyPath: "id" });
        }
        if (!db.objectStoreNames.contains("owners")) {
          db.createObjectStore("owners", { keyPath: "id" });
        }
        if (!db.objectStoreNames.contains("units")) {
          db.createObjectStore("units", { keyPath: "id" });
        }
        if (!db.objectStoreNames.contains("expenses")) {
          db.createObjectStore("expenses", { keyPath: "id" });
        }
        if (!db.objectStoreNames.contains("sync_queue")) {
          db.createObjectStore("sync_queue", {
            keyPath: "id",
            autoIncrement: true,
          });
        }
      };
    });
  }

  // Generic CRUD operations
  async get<T>(storeName: string, id: string): Promise<T | null> {
    if (!this.db) throw new Error("Database not initialized");

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(storeName, "readonly");
      const store = transaction.objectStore(storeName);
      const request = store.get(id);

      request.onsuccess = () => resolve(request.result || null);
      request.onerror = () => reject(request.error);
    });
  }

  async getAll<T>(storeName: string): Promise<T[]> {
    if (!this.db) throw new Error("Database not initialized");

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(storeName, "readonly");
      const store = transaction.objectStore(storeName);
      const request = store.getAll();

      request.onsuccess = () => resolve(request.result || []);
      request.onerror = () => reject(request.error);
    });
  }

  async put<T>(storeName: string, data: T): Promise<void> {
    if (!this.db) throw new Error("Database not initialized");

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(storeName, "readwrite");
      const store = transaction.objectStore(storeName);
      const request = store.put(data);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async delete(storeName: string, id: string): Promise<void> {
    if (!this.db) throw new Error("Database not initialized");

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(storeName, "readwrite");
      const store = transaction.objectStore(storeName);
      const request = store.delete(id);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async clear(storeName: string): Promise<void> {
    if (!this.db) throw new Error("Database not initialized");

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(storeName, "readwrite");
      const store = transaction.objectStore(storeName);
      const request = store.clear();

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  // Sync queue operations
  async addToSyncQueue(
    action: "create" | "update" | "delete",
    entity: string,
    data: any,
  ): Promise<void> {
    const syncItem: SyncQueue = {
      action,
      entity,
      data,
      timestamp: Date.now(),
      synced: false,
    };

    await this.put("sync_queue", syncItem);
  }

  async getSyncQueue(): Promise<SyncQueue[]> {
    return this.getAll<SyncQueue>("sync_queue");
  }

  async markSynced(id: number): Promise<void> {
    if (!this.db) throw new Error("Database not initialized");

    const item = await this.get<SyncQueue>("sync_queue", id.toString());
    if (item) {
      item.synced = true;
      await this.put("sync_queue", item);
    }
  }

  async clearSyncedItems(): Promise<void> {
    if (!this.db) throw new Error("Database not initialized");

    const items = await this.getSyncQueue();
    const syncedItems = items.filter((item) => item.synced);

    for (const item of syncedItems) {
      if (item.id) {
        await this.delete("sync_queue", item.id.toString());
      }
    }
  }

  // Specific entity operations
  async saveUser(user: User): Promise<void> {
    await this.put("users", user);
  }

  async getUser(id: string): Promise<User | null> {
    return this.get<User>("users", id);
  }

  async saveBuildings(buildings: Building[]): Promise<void> {
    for (const building of buildings) {
      await this.put("buildings", building);
    }
  }

  async getBuildings(): Promise<Building[]> {
    return this.getAll<Building>("buildings");
  }

  async saveOwners(owners: Owner[]): Promise<void> {
    for (const owner of owners) {
      await this.put("owners", owner);
    }
  }

  async getOwners(): Promise<Owner[]> {
    return this.getAll<Owner>("owners");
  }

  async saveUnits(units: Unit[]): Promise<void> {
    for (const unit of units) {
      await this.put("units", unit);
    }
  }

  async getUnits(): Promise<Unit[]> {
    return this.getAll<Unit>("units");
  }

  async saveExpenses(expenses: Expense[]): Promise<void> {
    for (const expense of expenses) {
      await this.put("expenses", expense);
    }
  }

  async getExpenses(): Promise<Expense[]> {
    return this.getAll<Expense>("expenses");
  }
}

// Export singleton instance
export const localDB = new LocalDB();
