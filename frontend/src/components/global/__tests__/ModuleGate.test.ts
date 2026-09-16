// Story 5.2 — ModuleGate Vitest tests (4-cat).
//
// CRITICAL §3 — RED-first TDD : test rouge AVANT le composant.
//
// Story 5.1 (backend, #585) n'est pas encore mergée : ni migration
// `acp_enabled_modules`, ni endpoint `GET /acps/:id/modules`, ni schéma
// OpenAPI. On mocke donc la frontière réseau (`lib/api/modules`) comme le
// fait déjà ContextBanner.test.ts pour `lib/api/buildings` — la logique de
// rendu conditionnel, elle, reste réelle.
//
// Couverture :
// - @happy    : ACP avec `community` activé → contenu rendu ; ACP sans →
//   fragment vide (pas de placeholder).
// - @edge     : bascule ACP en cours de session → le store re-fetch et
//   l'UI re-render en conséquence.
// - @security : ModuleGate ne fait QUE relayer la décision serveur — un
//   échec réseau (403 ModuleGuard, backend indisponible) masque le
//   contenu par défaut (fail-closed), jamais l'inverse. Ce composant ne
//   remplace pas le middleware backend (`ModuleGuard`, Story 5.1) :
//   c'est de la défense en profondeur côté affichage seulement.
// - @negative : nom de module inconnu → erreur typée en console + fragment
//   vide.
//
// data-testid contractuel (issue #586) : `module-gate-{{module}}`, présent
// ssi le contenu est rendu.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "../../../test-helpers";
import ModuleGate from "../ModuleGate.svelte";
import { setAcp, resetScope } from "../../../stores/scope.svelte";
import { resetEnabledModules } from "../../../stores/enabled_modules.svelte";
import { UnknownModuleError } from "../../../lib/api/modules";

// ---------------------------------------------------------------------------
// Mocks — frontière réseau uniquement
// ---------------------------------------------------------------------------

vi.mock("../../../lib/api/modules", async () => {
  const actual = await vi.importActual<
    typeof import("../../../lib/api/modules")
  >("../../../lib/api/modules");
  return {
    ...actual,
    listEnabledModules: vi.fn(),
  };
});

import { listEnabledModules } from "../../../lib/api/modules";

const mockedListEnabledModules = vi.mocked(listEnabledModules);

const ACP_A = "acp-aaaaaaaa-0000-0000-0000-000000000001";
const ACP_B = "acp-bbbbbbbb-0000-0000-0000-000000000002";

beforeEach(() => {
  resetScope();
  resetEnabledModules();
  vi.clearAllMocks();
});

afterEach(() => {
  resetScope();
  resetEnabledModules();
});

// ===========================================================================
// @happy — module activé → rendu ; module désactivé → fragment vide
// ===========================================================================

describe("ModuleGate @happy", () => {
  it("renders children when the module is enabled for the current ACP", async () => {
    mockedListEnabledModules.mockResolvedValue(["community", "ticketing"]);
    setAcp(ACP_A);

    render(ModuleGate, {
      props: { module: "community" },
    });

    const gate = await screen.findByTestId("module-gate-community");
    expect(gate).toBeInTheDocument();
  });

  it("renders an empty fragment (no placeholder) when the module is disabled", async () => {
    mockedListEnabledModules.mockResolvedValue(["ticketing"]);
    setAcp(ACP_A);

    const { container } = render(ModuleGate, {
      props: { module: "community" },
    });

    await waitFor(() => {
      expect(mockedListEnabledModules).toHaveBeenCalledWith(ACP_A);
    });

    expect(screen.queryByTestId("module-gate-community")).toBeNull();
    // Pas de placeholder résiduel : le fragment est réellement vide.
    expect(container.textContent?.trim() ?? "").toBe("");
  });
});

// ===========================================================================
// @edge — bascule ACP en cours de session → re-fetch + re-render
// ===========================================================================

describe("ModuleGate @edge", () => {
  it("re-fetches enabled modules and re-renders when the ACP scope changes mid-session", async () => {
    mockedListEnabledModules.mockImplementation(async (acpId: string) => {
      return acpId === ACP_A ? ["community"] : [];
    });

    setAcp(ACP_A);
    render(ModuleGate, { props: { module: "community" } });

    await screen.findByTestId("module-gate-community");

    // Bascule d'ACP en cours de session (ex: AcpSelector cliqué par l'utilisateur).
    setAcp(ACP_B);

    await waitFor(() => {
      expect(mockedListEnabledModules).toHaveBeenCalledWith(ACP_B);
    });
    await waitFor(() => {
      expect(screen.queryByTestId("module-gate-community")).toBeNull();
    });
  });
});

// ===========================================================================
// @security — défense en profondeur : le gate ne décide jamais tout seul
// ===========================================================================

describe("ModuleGate @security", () => {
  it("hides content (fail-closed) when the backend module check fails — never grants access on its own", async () => {
    // Simule un ModuleGuard backend qui refuse (403) ou est indisponible :
    // le composant ne doit JAMAIS supposer un accès qu'il ne peut pas
    // vérifier. Le middleware backend (Story 5.1) reste la garde réelle.
    mockedListEnabledModules.mockRejectedValue(new Error("403 forbidden"));
    setAcp(ACP_A);

    render(ModuleGate, { props: { module: "community" } });

    await waitFor(() => {
      expect(mockedListEnabledModules).toHaveBeenCalledWith(ACP_A);
    });
    expect(screen.queryByTestId("module-gate-community")).toBeNull();
  });

  it("always asks the backend (never a hardcoded client-side allowlist) before rendering", async () => {
    mockedListEnabledModules.mockResolvedValue(["community"]);
    setAcp(ACP_A);

    render(ModuleGate, { props: { module: "community" } });

    await screen.findByTestId("module-gate-community");
    expect(mockedListEnabledModules).toHaveBeenCalledTimes(1);
    expect(mockedListEnabledModules).toHaveBeenCalledWith(ACP_A);
  });
});

// ===========================================================================
// @negative — nom de module inconnu → erreur typée console + fragment vide
// ===========================================================================

describe("ModuleGate @negative", () => {
  it("logs a typed error to console and renders nothing for an unknown module name", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});
    setAcp(ACP_A);

    const { container } = render(ModuleGate, {
      props: { module: "foobar" },
    });

    await waitFor(() => {
      const loggedError = consoleErrorSpy.mock.calls
        .map((call) => call[0])
        .find((arg) => arg instanceof UnknownModuleError);
      expect(loggedError).toBeInstanceOf(UnknownModuleError);
    });

    expect(screen.queryByTestId("module-gate-foobar")).toBeNull();
    expect(container.textContent?.trim() ?? "").toBe("");

    consoleErrorSpy.mockRestore();
  });

  it("does not crash and renders nothing when no ACP is selected", () => {
    // Aucun setAcp() — scope.selectedAcpId === null.
    const { container } = render(ModuleGate, {
      props: { module: "community" },
    });

    expect(screen.queryByTestId("module-gate-community")).toBeNull();
    expect(container.textContent?.trim() ?? "").toBe("");
  });
});
