// Story B7 (Phase B FE) — Vitest 4-cat TechnicalSpecCreate.
//
// Couverture (cf. stories.md §B7 AC) :
//   @happy    : form complet (title + description ≥50 + version 1.0.0 valide +
//               1 deliverable + sig=[syndic,amo]) → submit → onSubmit appelé
//               avec payload typé CreateTechnicalSpecRequest.
//   @edge     : semver strict (v1.0.0 rejeté, 1.0.0-rc1 rejeté, 1.0 rejeté via
//               input number → testé via helper isValidSemver) ;
//               mode="bump" + version <= précédente → submit disabled.
//   @security : couvert par tests TechnicalSpecDetail (gating bouton Signer).
//               Ici on vérifie qu'aucune option "v" préfixe / pre-release n'est
//               saisissable via les inputs séparés (impossibilité par design).
//   @negative : deliverables vide → submit disabled + helper ;
//               description < 50 chars → counter rouge + submit disabled.
//
// Pattern DI : on injecte `onSubmit` via prop — cohérent SyndicResponseForm B6.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import TechnicalSpecCreate from "./TechnicalSpecCreate.svelte";
import {
  isValidSemver,
  type TechnicalSpecDto,
} from "../../api/technical_specs";

vi.mock("../../../stores/toast", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}));

const ACP_ID = "11111111-1111-1111-1111-111111111111";

function fillInput(el: HTMLInputElement | HTMLTextAreaElement, value: string) {
  el.value = value;
  el.dispatchEvent(new Event("input", { bubbles: true }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

function makeFixture(over: Partial<TechnicalSpecDto> = {}): TechnicalSpecDto {
  return {
    id: "spec-uuid-new",
    acp_id: ACP_ID,
    building_id: null,
    title: "Travaux toiture immeuble",
    description:
      "Réfection complète couverture ardoise + zinguerie selon plan archi du 2026-06-09.",
    version: "1.0.0",
    deliverables: ["Démontage couverture existante", "Pose voligeage neuf"],
    required_signatures: ["syndic", "amo"],
    attachments: [],
    status: "Draft",
    created_by: "syndic-uuid",
    previous_version_id: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    ...over,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("TechnicalSpecCreate — Story B7 (4-cat)", () => {
  it("@happy form valide → onSubmit appelé avec payload typé CreateTechnicalSpecRequest", async () => {
    const created = makeFixture();
    const onSubmit = vi.fn().mockResolvedValue(created);

    const { getByTestId } = render(TechnicalSpecCreate, {
      props: { acpId: ACP_ID, onSubmit },
    });

    // Title
    fillInput(
      getByTestId("tech-spec-title-input") as HTMLInputElement,
      "Travaux toiture immeuble",
    );

    // Description ≥ 50 chars
    fillInput(
      getByTestId("tech-spec-description-textarea") as HTMLTextAreaElement,
      "Réfection complète couverture ardoise + zinguerie selon plan archi du 2026-06-09.",
    );

    // Version reste 1.0.0 (defaults) — vérifier preview.
    expect(getByTestId("tech-spec-version-preview").textContent).toMatch(
      /1\.0\.0/,
    );

    // Deliverable 0 (le form en a 1 vide par défaut).
    fillInput(
      getByTestId("tech-spec-deliverable-input-0") as HTMLInputElement,
      "Démontage couverture existante",
    );

    // Sig défaut = ["syndic"] → on coche aussi "amo".
    const amoCb = getByTestId(
      "tech-spec-required-sig-option-amo",
    ) as HTMLInputElement;
    amoCb.click();
    amoCb.dispatchEvent(new Event("change", { bubbles: true }));

    const submit = getByTestId("tech-spec-create-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    submit.click();

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    const [req] = onSubmit.mock.calls[0];
    expect(req).toMatchObject({
      acp_id: ACP_ID,
      title: "Travaux toiture immeuble",
      version: "1.0.0",
      deliverables: ["Démontage couverture existante"],
    });
    expect(req.required_signatures).toEqual(
      expect.arrayContaining(["syndic", "amo"]),
    );
    expect(req.description.length).toBeGreaterThanOrEqual(50);
  });

  it("@edge semver strict — helper isValidSemver rejette v-prefix / pre-release / 2-segments", () => {
    // Le composant utilise inputs number séparés → impossible de taper "v1.0.0"
    // ou "1.0.0-rc1" via UI. Ce test verrouille le contrat invariant CLIENT
    // (cohérent backend Story 3.8 INV semver strict).
    expect(isValidSemver("1.0.0")).toBe(true);
    expect(isValidSemver("v1.0.0")).toBe(false); // prefix v rejeté
    expect(isValidSemver("1.0.0-rc1")).toBe(false); // pre-release rejeté
    expect(isValidSemver("1.0")).toBe(false); // 2 segments rejetés
    expect(isValidSemver("1.0.0+build")).toBe(false); // build metadata rejetée
    expect(isValidSemver("01.0.0")).toBe(false); // leading zero rejeté
    expect(isValidSemver("")).toBe(false);
  });

  it("@edge mode=bump + version <= précédente → submit disabled + erreur version", async () => {
    const previous = makeFixture({ version: "1.5.7" });
    const onSubmit = vi.fn();

    const { getByTestId, queryByTestId } = render(TechnicalSpecCreate, {
      props: {
        acpId: ACP_ID,
        mode: "bump",
        previousVersion: previous,
        onSubmit,
      },
    });

    // Le default en mode bump est major.minor+1.0 → strictement supérieur.
    // On force major=1 minor=0 patch=0 (< 1.5.7) → submit doit être disabled.
    fillInput(
      getByTestId("tech-spec-version-major-input") as HTMLInputElement,
      "1",
    );
    fillInput(
      getByTestId("tech-spec-version-minor-input") as HTMLInputElement,
      "0",
    );
    fillInput(
      getByTestId("tech-spec-version-patch-input") as HTMLInputElement,
      "0",
    );

    const submit = getByTestId("tech-spec-create-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(true));

    // L'erreur version est visible.
    await waitFor(() => {
      const err = queryByTestId("tech-spec-create-error-version");
      expect(err).not.toBeNull();
      expect(err?.textContent).toMatch(/strictement supérieure/i);
    });
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("@security UI inputs séparés → impossible de saisir 'v' prefix / pre-release", () => {
    const onSubmit = vi.fn();
    const { getByTestId } = render(TechnicalSpecCreate, {
      props: { acpId: ACP_ID, onSubmit },
    });

    // Les inputs version sont `type="number"` → DOM rejette les caractères
    // non-numériques (gating natif). On vérifie l'attribut.
    const major = getByTestId(
      "tech-spec-version-major-input",
    ) as HTMLInputElement;
    const minor = getByTestId(
      "tech-spec-version-minor-input",
    ) as HTMLInputElement;
    const patch = getByTestId(
      "tech-spec-version-patch-input",
    ) as HTMLInputElement;
    expect(major.type).toBe("number");
    expect(minor.type).toBe("number");
    expect(patch.type).toBe("number");
    expect(major.min).toBe("0");
    expect(minor.min).toBe("0");
    expect(patch.min).toBe("0");
  });

  it("@negative deliverables vide → submit disabled + helper inline", async () => {
    const onSubmit = vi.fn();
    const { getByTestId, queryByTestId } = render(TechnicalSpecCreate, {
      props: { acpId: ACP_ID, onSubmit },
    });

    // Title + description valides, mais aucun deliverable rempli.
    fillInput(
      getByTestId("tech-spec-title-input") as HTMLInputElement,
      "Titre OK",
    );
    fillInput(
      getByTestId("tech-spec-description-textarea") as HTMLTextAreaElement,
      "Description suffisamment longue pour passer le seuil de 50 chars minimum.",
    );
    // Le deliverable 0 par défaut est vide → on laisse tel quel.

    const submit = getByTestId("tech-spec-create-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(true));

    const err = queryByTestId("tech-spec-create-error-deliverables");
    expect(err).not.toBeNull();
    expect(err?.textContent).toMatch(/au moins 1 deliverable/i);

    submit.click();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("@negative description < 50 chars → counter rouge + submit disabled", async () => {
    const onSubmit = vi.fn();
    const { getByTestId } = render(TechnicalSpecCreate, {
      props: { acpId: ACP_ID, onSubmit },
    });

    fillInput(
      getByTestId("tech-spec-title-input") as HTMLInputElement,
      "Titre OK",
    );

    // 30 chars (< 50 minimum).
    fillInput(
      getByTestId("tech-spec-description-textarea") as HTMLTextAreaElement,
      "Description trop courte 30 ch.",
    );

    const counter = getByTestId("tech-spec-description-counter");
    expect(counter.className).toMatch(/text-red-600/);
    expect(counter.textContent).toMatch(/min\.\s+50/i);

    const submit = getByTestId("tech-spec-create-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(true));
  });

  it("deliverable add/remove dynamique fonctionne", async () => {
    const { getByTestId, queryByTestId } = render(TechnicalSpecCreate, {
      props: { acpId: ACP_ID, onSubmit: vi.fn() },
    });

    // 1 row par défaut.
    expect(queryByTestId("tech-spec-deliverable-row-0")).not.toBeNull();
    expect(queryByTestId("tech-spec-deliverable-row-1")).toBeNull();

    // Ajout
    (getByTestId("tech-spec-deliverable-add") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(queryByTestId("tech-spec-deliverable-row-1")).not.toBeNull(),
    );

    // Remove row 0
    (
      getByTestId("tech-spec-deliverable-remove-0") as HTMLButtonElement
    ).click();
    await waitFor(() =>
      // Après suppression, il reste 1 row (idx 0).
      expect(queryByTestId("tech-spec-deliverable-row-1")).toBeNull(),
    );
  });
});
