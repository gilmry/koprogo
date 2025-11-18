/**
 * IndexedDB utilities for offline storage
 * Provides a simple interface for caching data locally
 */

const DB_NAME = 'koprogo-offline';
const DB_VERSION = 1;

// Object store names
export const STORES = {
  BUILDINGS: 'buildings',
  UNITS: 'units',
  OWNERS: 'owners',
  DOCUMENTS: 'documents',
  NOTIFICATIONS: 'notifications',
  USER_PROFILE: 'userProfile',
  PENDING_REQUESTS: 'pendingRequests',
} as const;

export type StoreNames = typeof STORES[keyof typeof STORES];

/**
 * Open IndexedDB database
 */
export async function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;

      // Create object stores if they don't exist
      if (!db.objectStoreNames.contains(STORES.BUILDINGS)) {
        db.createObjectStore(STORES.BUILDINGS, { keyPath: 'id' });
      }

      if (!db.objectStoreNames.contains(STORES.UNITS)) {
        const unitStore = db.createObjectStore(STORES.UNITS, { keyPath: 'id' });
        unitStore.createIndex('building_id', 'building_id', { unique: false });
      }

      if (!db.objectStoreNames.contains(STORES.OWNERS)) {
        db.createObjectStore(STORES.OWNERS, { keyPath: 'id' });
      }

      if (!db.objectStoreNames.contains(STORES.DOCUMENTS)) {
        const docStore = db.createObjectStore(STORES.DOCUMENTS, { keyPath: 'id' });
        docStore.createIndex('created_at', 'created_at', { unique: false });
      }

      if (!db.objectStoreNames.contains(STORES.NOTIFICATIONS)) {
        const notifStore = db.createObjectStore(STORES.NOTIFICATIONS, { keyPath: 'id' });
        notifStore.createIndex('created_at', 'created_at', { unique: false });
        notifStore.createIndex('is_read', 'is_read', { unique: false });
      }

      if (!db.objectStoreNames.contains(STORES.USER_PROFILE)) {
        db.createObjectStore(STORES.USER_PROFILE, { keyPath: 'id' });
      }

      if (!db.objectStoreNames.contains(STORES.PENDING_REQUESTS)) {
        db.createObjectStore(STORES.PENDING_REQUESTS, {
          keyPath: 'id',
          autoIncrement: true,
        });
      }
    };
  });
}

/**
 * Save item to IndexedDB
 */
export async function saveItem<T>(
  storeName: StoreNames,
  item: T
): Promise<void> {
  const db = await openDB();

  return new Promise((resolve, reject) => {
    const transaction = db.transaction([storeName], 'readwrite');
    const store = transaction.objectStore(storeName);
    const request = store.put(item);

    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error);
  });
}

/**
 * Save multiple items to IndexedDB
 */
export async function saveItems<T>(
  storeName: StoreNames,
  items: T[]
): Promise<void> {
  const db = await openDB();

  return new Promise((resolve, reject) => {
    const transaction = db.transaction([storeName], 'readwrite');
    const store = transaction.objectStore(storeName);

    let completed = 0;
    const total = items.length;

    items.forEach((item) => {
      const request = store.put(item);
      request.onsuccess = () => {
        completed++;
        if (completed === total) {
          resolve();
        }
      };
      request.onerror = () => reject(request.error);
    });

    if (total === 0) {
      resolve();
    }
  });
}

/**
 * Get item from IndexedDB by ID
 */
export async function getItem<T>(
  storeName: StoreNames,
  id: string
): Promise<T | undefined> {
  const db = await openDB();

  return new Promise((resolve, reject) => {
    const transaction = db.transaction([storeName], 'readonly');
    const store = transaction.objectStore(storeName);
    const request = store.get(id);

    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

/**
 * Get all items from IndexedDB
 */
export async function getAllItems<T>(
  storeName: StoreNames
): Promise<T[]> {
  const db = await openDB();

  return new Promise((resolve, reject) => {
    const transaction = db.transaction([storeName], 'readonly');
    const store = transaction.objectStore(storeName);
    const request = store.getAll();

    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

/**
 * Delete item from IndexedDB
 */
export async function deleteItem(
  storeName: StoreNames,
  id: string
): Promise<void> {
  const db = await openDB();

  return new Promise((resolve, reject) => {
    const transaction = db.transaction([storeName], 'readwrite');
    const store = transaction.objectStore(storeName);
    const request = store.delete(id);

    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error);
  });
}

/**
 * Clear all items from a store
 */
export async function clearStore(storeName: StoreNames): Promise<void> {
  const db = await openDB();

  return new Promise((resolve, reject) => {
    const transaction = db.transaction([storeName], 'readwrite');
    const store = transaction.objectStore(storeName);
    const request = store.clear();

    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error);
  });
}

/**
 * Queue a request for background sync
 */
export interface PendingRequest {
  url: string;
  method: string;
  headers: Record<string, string>;
  body?: string;
  timestamp: number;
}

export async function queueRequest(request: Omit<PendingRequest, 'timestamp'>): Promise<void> {
  const requestWithTimestamp: PendingRequest = {
    ...request,
    timestamp: Date.now(),
  };

  await saveItem(STORES.PENDING_REQUESTS, requestWithTimestamp);

  // Register background sync if supported
  if ('serviceWorker' in navigator && 'sync' in ServiceWorkerRegistration.prototype) {
    try {
      const registration = await navigator.serviceWorker.ready;
      await registration.sync.register('sync-data');
    } catch (error) {
      console.error('Failed to register background sync:', error);
    }
  }
}

/**
 * Get pending requests count
 */
export async function getPendingRequestsCount(): Promise<number> {
  const requests = await getAllItems<PendingRequest>(STORES.PENDING_REQUESTS);
  return requests.length;
}

/**
 * Clear all pending requests
 */
export async function clearPendingRequests(): Promise<void> {
  await clearStore(STORES.PENDING_REQUESTS);
}
