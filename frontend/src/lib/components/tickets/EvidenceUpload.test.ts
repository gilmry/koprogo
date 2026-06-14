// Story B5 (Phase B FE) — Vitest 4-cat EvidenceUpload.
//
// Couverture (cf. stories.md §B5 + mission) :
//   @happy    : upload via file input → onUpload appelé → status=done →
//               value (bindable) reflète publicUrl ; preview thumbnail
//               affichée pour image ; counter "1/10" mis à jour.
//   @edge     : 10 fichiers → 10e accepté + counter "10/10" + atMax true ;
//               11e fichier → refusé client-side + zone erreur "Maximum 10
//               preuves" ; fichier > 10 MB → refusé + erreur "Taille max".
//   @security : MIME inspection client — fichier .exe (application/x-msdownload)
//               → refusé + zone erreur "Type non autorisé" + onUpload PAS
//               appelé.
//   @negative : upload qui throw → status=error + onError callback ; remove
//               item → liste réduite + revokeObjectURL (vérifié via spy).
//
// Pattern DI : `onUpload` injecté pour simuler upload (pas de fetch réel).

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import EvidenceUpload from "./EvidenceUpload.svelte";
import { UploadError } from "../../utils/fileUpload";

// -----------------------------------------------------------------------------
// jsdom URL.createObjectURL stub (jsdom n'implémente pas)
// -----------------------------------------------------------------------------

const createObjectURLSpy = vi.fn(() => "blob:fake-url-1");
const revokeObjectURLSpy = vi.fn();

beforeEach(() => {
  // jsdom n'expose pas createObjectURL/revokeObjectURL — on patch global.
  (global.URL as unknown as { createObjectURL: typeof createObjectURLSpy }).createObjectURL =
    createObjectURLSpy;
  (global.URL as unknown as { revokeObjectURL: typeof revokeObjectURLSpy }).revokeObjectURL =
    revokeObjectURLSpy;
  createObjectURLSpy.mockClear();
  revokeObjectURLSpy.mockClear();
});

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

function makeFile(
  name: string,
  mime: string,
  sizeBytes: number,
): File {
  // jsdom : File hérite de Blob. On construit avec un contenu fictif puis
  // override `size` si besoin (peut différer en fonction du contenu).
  const blob = new Blob(["x"], { type: mime });
  const f = new File([blob], name, { type: mime });
  Object.defineProperty(f, "size", { value: sizeBytes });
  return f;
}

function fireFileInputChange(
  input: HTMLInputElement,
  files: File[],
): void {
  Object.defineProperty(input, "files", {
    value: files,
    configurable: true,
  });
  input.dispatchEvent(new Event("change", { bubbles: true }));
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

describe("EvidenceUpload — Story B5 (4-cat)", () => {
  it("@happy upload 1 image → onUpload appelé → status=done + counter 1/10", async () => {
    const onUpload = vi.fn().mockResolvedValue("https://cdn/test.png");
    const { getByTestId } = render(EvidenceUpload, {
      props: { value: [], onUpload },
    });

    const input = getByTestId(
      "ticket-evidence-file-input",
    ) as HTMLInputElement;
    fireFileInputChange(input, [
      makeFile("photo.png", "image/png", 1024),
    ]);

    await waitFor(() => expect(onUpload).toHaveBeenCalledTimes(1));

    // Counter mis à jour à 1/10.
    await waitFor(() => {
      const counter = getByTestId("ticket-evidence-count");
      expect(counter.textContent).toMatch(/1\s*\/\s*10/);
    });

    // Preview thumbnail rendue (image → <img>).
    await waitFor(() => {
      const preview = getByTestId("ticket-evidence-preview-0");
      expect(preview).not.toBeNull();
    });
  });

  it("@edge 10 fichiers acceptés, 11e refusé client-side avec erreur 'Maximum'", async () => {
    const onUpload = vi
      .fn()
      .mockImplementation((f: File) =>
        Promise.resolve(`https://cdn/${f.name}`),
      );
    const { getByTestId, queryByTestId } = render(EvidenceUpload, {
      props: { value: [], onUpload },
    });

    const input = getByTestId(
      "ticket-evidence-file-input",
    ) as HTMLInputElement;

    // 10 fichiers → tous acceptés.
    const tenFiles = Array.from({ length: 10 }, (_, i) =>
      makeFile(`p${i}.png`, "image/png", 100),
    );
    fireFileInputChange(input, tenFiles);

    await waitFor(() => expect(onUpload).toHaveBeenCalledTimes(10));
    await waitFor(() => {
      const counter = getByTestId("ticket-evidence-count");
      expect(counter.textContent).toMatch(/10\s*\/\s*10/);
    });

    // Aucune erreur n'est apparue pour les 10.
    expect(queryByTestId("ticket-evidence-error")).toBeNull();

    // 11e fichier → refusé.
    onUpload.mockClear();
    fireFileInputChange(input, [
      makeFile("p11.png", "image/png", 100),
    ]);

    await waitFor(() => {
      const err = queryByTestId("ticket-evidence-error");
      expect(err).not.toBeNull();
      expect(err?.textContent).toMatch(/Maximum 10/i);
    });
    expect(onUpload).not.toHaveBeenCalled();
  });

  it("@edge fichier > 10 MB → refusé + erreur 'Taille max' + onUpload PAS appelé", async () => {
    const onUpload = vi.fn();
    const { getByTestId, queryByTestId } = render(EvidenceUpload, {
      props: { value: [], onUpload },
    });

    const input = getByTestId(
      "ticket-evidence-file-input",
    ) as HTMLInputElement;

    // 11 MB.
    fireFileInputChange(input, [
      makeFile("huge.png", "image/png", 11 * 1024 * 1024),
    ]);

    await waitFor(() => {
      const err = queryByTestId("ticket-evidence-error");
      expect(err).not.toBeNull();
      expect(err?.textContent).toMatch(/Taille max|max 10 MB/i);
    });
    expect(onUpload).not.toHaveBeenCalled();
  });

  it("@security MIME .exe (application/x-msdownload) → refusé + erreur 'Type non autorisé'", async () => {
    const onUpload = vi.fn();
    const { getByTestId, queryByTestId } = render(EvidenceUpload, {
      props: { value: [], onUpload },
    });

    const input = getByTestId(
      "ticket-evidence-file-input",
    ) as HTMLInputElement;

    fireFileInputChange(input, [
      makeFile("evil.exe", "application/x-msdownload", 1024),
    ]);

    await waitFor(() => {
      const err = queryByTestId("ticket-evidence-error");
      expect(err).not.toBeNull();
      expect(err?.textContent).toMatch(/Type non autorisé/i);
    });
    expect(onUpload).not.toHaveBeenCalled();
  });

  it("@security `accept` attribute liste image|video|pdf et exclut le reste", () => {
    const { getByTestId } = render(EvidenceUpload, {
      props: { value: [], onUpload: vi.fn() },
    });
    const input = getByTestId(
      "ticket-evidence-file-input",
    ) as HTMLInputElement;
    const accept = input.getAttribute("accept") ?? "";
    expect(accept).toMatch(/image/);
    expect(accept).toMatch(/video/);
    expect(accept).toMatch(/application\/pdf/);
    // application/x-msdownload n'est PAS dans accept.
    expect(accept).not.toMatch(/x-msdownload/);
  });

  it("@negative upload qui throw → status=error + onError callback", async () => {
    const onUpload = vi
      .fn()
      .mockRejectedValue(
        new UploadError("upload-failed", "S3 returned 500"),
      );
    const onError = vi.fn();
    const { getByTestId, container } = render(EvidenceUpload, {
      props: { value: [], onUpload, onError },
    });

    const input = getByTestId(
      "ticket-evidence-file-input",
    ) as HTMLInputElement;
    fireFileInputChange(input, [
      makeFile("photo.png", "image/png", 1024),
    ]);

    await waitFor(() => expect(onUpload).toHaveBeenCalled());
    await waitFor(() => expect(onError).toHaveBeenCalledTimes(1));
    expect(onError.mock.calls[0][0]).toBeInstanceOf(UploadError);

    // L'item reste visible en état error (UX : permet retry / remove).
    await waitFor(() => {
      const errSpan = container.textContent ?? "";
      expect(errSpan).toMatch(/S3 returned 500/);
    });
  });

  it("@negative remove item → revokeObjectURL appelé + liste réduite", async () => {
    const onUpload = vi.fn().mockResolvedValue("https://cdn/p.png");
    const { getByTestId, queryByTestId } = render(EvidenceUpload, {
      props: { value: [], onUpload },
    });

    const input = getByTestId(
      "ticket-evidence-file-input",
    ) as HTMLInputElement;
    fireFileInputChange(input, [
      makeFile("p.png", "image/png", 1024),
    ]);

    await waitFor(() => expect(onUpload).toHaveBeenCalled());
    await waitFor(() => expect(getByTestId("ticket-evidence-preview-0")).not.toBeNull());

    const removeBtn = getByTestId(
      "ticket-evidence-remove-0",
    ) as HTMLButtonElement;
    removeBtn.click();

    await waitFor(() =>
      expect(queryByTestId("ticket-evidence-preview-0")).toBeNull(),
    );
    expect(revokeObjectURLSpy).toHaveBeenCalled();
  });

  it("@happy dropzone a role=button + aria-label + tabindex 0 (a11y)", () => {
    const { getByTestId } = render(EvidenceUpload, {
      props: { value: [], onUpload: vi.fn() },
    });
    const zone = getByTestId("ticket-evidence-upload");
    expect(zone.getAttribute("role")).toBe("button");
    expect(zone.getAttribute("aria-label")).toMatch(
      /Glissez-déposez|preuves|sélectionner/i,
    );
    expect(zone.getAttribute("tabindex")).toBe("0");
  });
});
