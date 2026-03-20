/**
 * TestWorld — shared test data for all Playwright E2E specs.
 *
 * Inspired by the BDD OperationsWorld / CommunityWorld / FinancialWorld pattern:
 * a pre-seeded universe with org, users, buildings, units, owners, meetings, etc.
 *
 * Created once in global-setup.ts, stored to disk, loaded by each spec.
 * Eliminates per-test setup (5-10s savings) and field-name bugs.
 */
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const WORLD_PATH = path.join(__dirname, "..", ".test-world.json");

export interface TestWorld {
  /** SuperAdmin token (full access) */
  adminToken: string;

  /** Organization */
  orgId: string;
  orgName: string;

  /** Syndic user */
  syndic: {
    token: string;
    email: string;
    password: string;
    userId: string;
    firstName: string;
    lastName: string;
  };

  /** Owner user */
  owner: {
    token: string;
    email: string;
    password: string;
    userId: string;
    ownerId: string;
    firstName: string;
    lastName: string;
  };

  /** Building with units and owners */
  building: {
    id: string;
    name: string;
    address: string;
    city: string;
  };

  /** Units in the building */
  units: Array<{
    id: string;
    unitNumber: string;
    floor: number;
  }>;

  /** Meeting for governance tests */
  meeting: {
    id: string;
    title: string;
  };

  /** Pre-seeded entities for detail page tests */
  ticket?: { id: string; title: string };
  expense?: { id: string; description: string };
  budget?: { id: string; fiscalYear: number };
  notice?: { id: string; title: string };
  exchange?: { id: string; title: string };
  skill?: { id: string; skillName: string };
  sharedObject?: { id: string; objectName: string };
  poll?: { id: string; title: string };
  quote?: { id: string; projectTitle: string };
  workReport?: { id: string; title: string };
  inspection?: { id: string; inspectorName: string };
  booking?: { id: string; resourceName: string };
  convocation?: { id: string };
  etatDate?: { id: string };
  paymentReminder?: { id: string };

  /** Timestamp for uniqueness */
  createdAt: string;
}

/** Save world to disk (called by global-setup) */
export function saveWorld(world: TestWorld): void {
  fs.writeFileSync(WORLD_PATH, JSON.stringify(world, null, 2));
}

/** Load world from disk (called by specs) */
export function loadWorld(): TestWorld {
  if (!fs.existsSync(WORLD_PATH)) {
    throw new Error(
      "TestWorld not found. Run global-setup first (npx playwright test).",
    );
  }
  return JSON.parse(fs.readFileSync(WORLD_PATH, "utf-8"));
}

/** Clean up world file */
export function cleanWorld(): void {
  if (fs.existsSync(WORLD_PATH)) {
    fs.unlinkSync(WORLD_PATH);
  }
}
