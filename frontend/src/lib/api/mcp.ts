// MCP API Client
const API_URL =
  import.meta.env.PUBLIC_API_URL || "http://localhost:8080/api/v1";
const MCP_URL = `${API_URL.replace("/api/v1", "")}/mcp/v1`;

export interface Message {
  role: "system" | "user" | "assistant";
  content: string;
}

export interface ChatRequest {
  model: string;
  messages: Message[];
  context?: string;
  max_tokens?: number;
  temperature?: number;
  stream?: boolean;
}

export interface ChatResponse {
  id: string;
  model: string;
  content: string;
  finish_reason: string;
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
  execution_info: {
    execution_type: string;
    node_id?: string;
    latency_ms: number;
    co2_grams: number;
  };
}

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  context_length: number;
  is_available: boolean;
  edge_compatible: boolean;
}

export interface McpStatistics {
  total_requests: number;
  total_tokens: number;
  total_co2_grams: number;
  co2_saved_grams: number;
  edge_requests: number;
  cloud_requests: number;
  grid_requests: number;
  avg_latency_ms: number;
  models_used: string[];
}

/**
 * Send a chat message to MCP
 */
export async function chat(request: ChatRequest): Promise<ChatResponse> {
  const response = await fetch(`${MCP_URL}/chat`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`MCP error: ${error}`);
  }

  return response.json();
}

/**
 * List available models
 */
export async function listModels(): Promise<ModelInfo[]> {
  const response = await fetch(`${MCP_URL}/models`);

  if (!response.ok) {
    throw new Error("Failed to fetch models");
  }

  const data = await response.json();
  return data.models;
}

/**
 * Get MCP statistics
 */
export async function getStatistics(): Promise<McpStatistics> {
  const response = await fetch(`${MCP_URL}/stats`);

  if (!response.ok) {
    throw new Error("Failed to fetch statistics");
  }

  return response.json();
}

/**
 * Check MCP health
 */
export async function checkHealth(): Promise<{
  is_healthy: boolean;
  edge_nodes_available: number;
  active_tasks: number;
  avg_latency_ms: number;
}> {
  const response = await fetch(`${MCP_URL}/health`);

  if (!response.ok) {
    throw new Error("Failed to check health");
  }

  return response.json();
}

// IndexedDB for offline storage
const DB_NAME = "koprogo_mcp";
const DB_VERSION = 1;
const STORE_NAME = "chat_history";

let db: IDBDatabase | null = null;

async function openDB(): Promise<IDBDatabase> {
  if (db) return db;

  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onerror = () => reject(request.error);
    request.onsuccess = () => {
      db = request.result;
      resolve(db);
    };

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        const store = db.createObjectStore(STORE_NAME, {
          keyPath: "id",
          autoIncrement: true,
        });
        store.createIndex("context", "context", { unique: false });
        store.createIndex("created_at", "created_at", { unique: false });
      }
    };
  });
}

export async function saveChatToLocal(
  messages: Message[],
  response: ChatResponse,
  context?: string,
) {
  const db = await openDB();
  const transaction = db.transaction([STORE_NAME], "readwrite");
  const store = transaction.objectStore(STORE_NAME);

  const record = {
    messages,
    response,
    context,
    created_at: new Date().toISOString(),
  };

  return new Promise((resolve, reject) => {
    const request = store.add(record);
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

export async function loadChatHistory(context?: string): Promise<any[]> {
  const db = await openDB();
  const transaction = db.transaction([STORE_NAME], "readonly");
  const store = transaction.objectStore(STORE_NAME);

  return new Promise((resolve, reject) => {
    let request: IDBRequest;

    if (context) {
      const index = store.index("context");
      request = index.getAll(context);
    } else {
      request = store.getAll();
    }

    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}
