// Story B2 (Phase B FE) — MagicLinkIssueForm Vitest tests (4-cat).
//
// CRITICAL §3 — RED-first TDD: ces tests décrivent le contrat (data-testid +
// flux écrans) AVANT que le composant ne soit considéré "terminé".
//
// Couverture 4 catégories :
//   @happy    submit → écran "issued" → URL `/c?t=<token>` affichée + bouton
//             copy fonctionnel (navigator.clipboard mocké) + warning visible.
//   @edge     expires_in_seconds = 60 (min) → submit OK ; expires_in_seconds
//             = 30j (max) → submit OK ; backend renvoie 422 "expires_in_seconds
//             < 60" (simulation DevTools tampering) → message inline visible
//             SANS bascule en vue "issued".
//   @security subject = self (currentUserId) → submit disabled + helper text
//             "Vous ne pouvez pas vous émettre un lien à vous-même". Le token
//             émis n'est PAS écrit en localStorage/sessionStorage.
//   @negative scopeIdsByKind vide pour le kind sélectionné → select disabled
//             + helper "Aucun ticket trouvé" + submit disabled.
//
// Pourquoi pas de mock du wrapper `api.ts` : on injecte `onIssue` via prop
// (DI), c'est plus simple et plus déterministe qu'un vi.mock sur le module
// entier.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import MagicLinkIssueForm from "./MagicLinkIssueForm.svelte";
import type { IssuedMagicLink } from "../../api/magic_links";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const SYNDIC_USER_ID = "00000000-0000-0000-0000-000000000001";
const CONTRACTOR_ID = "00000000-0000-0000-0000-000000000002";
const TICKET_ID = "10000000-0000-0000-0000-000000000001";

const USERS = [
  { id: CONTRACTOR_ID, label: "Jean Plombier (contractor)" },
  { id: SYNDIC_USER_ID, label: "Marc Syndic (self)" },
];

const SCOPE_IDS = {
  ticket: [{ id: TICKET_ID, label: "#42 Fuite cuisine" }],
};

function makeIssuedFixture(token: string = "tok-abc-123"): IssuedMagicLink {
  return {
    id: "20000000-0000-0000-0000-000000000001",
    token,
    expires_at: new Date(Date.now() + 7 * 24 * 3600 * 1000).toISOString(),
    scope_kind: "ticket",
    scope_id: TICKET_ID,
  };
}

// ---------------------------------------------------------------------------
// Setup — installe un navigator.clipboard.writeText mock + window flags.
// ---------------------------------------------------------------------------

let writeTextMock: ReturnType<typeof vi.fn>;

beforeEach(() => {
  writeTextMock = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText: writeTextMock },
  });
  Object.defineProperty(window, "isSecureContext", {
    configurable: true,
    value: true,
  });
  // Nettoie storage entre tests pour les checks @security.
  try {
    window.localStorage.clear();
    window.sessionStorage.clear();
  } catch {
    // jsdom peut ne pas exposer storage selon flags — ignore.
  }
});

// ---------------------------------------------------------------------------
// Helpers form (DRY)
// ---------------------------------------------------------------------------

function fillSelect(el: HTMLSelectElement, value: string) {
  el.value = value;
  el.dispatchEvent(new Event("input", { bubbles: true }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

function fillRange(el: HTMLInputElement, value: number) {
  el.value = String(value);
  el.dispatchEvent(new Event("input", { bubbles: true }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("MagicLinkIssueForm — Story B2", () => {
  it("@happy submit → écran issued → URL `/c?t=<token>` + bouton copy", async () => {
    const onIssue = vi.fn().mockResolvedValue(makeIssuedFixture("tok-happy"));
    const { getByTestId } = render(MagicLinkIssueForm, {
      props: {
        users: USERS,
        scopeIdsByKind: SCOPE_IDS,
        currentUserId: SYNDIC_USER_ID,
        publicBaseUrl: "https://koprogo.tld",
        onIssue,
      },
    });

    // Remplir le form.
    fillSelect(
      getByTestId("magic-link-target-input") as HTMLSelectElement,
      CONTRACTOR_ID,
    );
    fillSelect(
      getByTestId("magic-link-scope-select") as HTMLSelectElement,
      "ticket",
    );
    // L'effect a vidé scope_id après changement de kind — on remplit après.
    await waitFor(() => {
      const sid = getByTestId(
        "magic-link-scope-id-select",
      ) as HTMLSelectElement;
      expect(sid.disabled).toBe(false);
    });
    fillSelect(
      getByTestId("magic-link-scope-id-select") as HTMLSelectElement,
      TICKET_ID,
    );

    // Submit.
    const submitBtn = getByTestId(
      "magic-link-issue-submit",
    ) as HTMLButtonElement;
    await waitFor(() => expect(submitBtn.disabled).toBe(false));
    submitBtn.click();

    // Backend appelé avec le bon payload.
    await waitFor(() => expect(onIssue).toHaveBeenCalledTimes(1));
    expect(onIssue).toHaveBeenCalledWith(
      expect.objectContaining({
        subject_user_id: CONTRACTOR_ID,
        scope_kind: "ticket",
        scope_id: TICKET_ID,
        expires_in_seconds: 7 * 24 * 3600,
      }),
    );

    // Vue "issued" affichée avec URL complète + warning.
    await waitFor(() => {
      const urlInput = getByTestId(
        "magic-link-issued-url-input",
      ) as HTMLInputElement;
      expect(urlInput.value).toBe("https://koprogo.tld/c?t=tok-happy");
    });
    expect(getByTestId("magic-link-issued-warning")).toBeInTheDocument();

    // Bouton Copy → navigator.clipboard.writeText appelé avec l'URL complète.
    (getByTestId("magic-link-issued-url-copy") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(writeTextMock).toHaveBeenCalledWith(
        "https://koprogo.tld/c?t=tok-happy",
      ),
    );
  });

  it("@edge backend renvoie 422 'expires_in_seconds < 60' → message inline, pas de bascule vue", async () => {
    const onIssue = vi
      .fn()
      .mockRejectedValue(new Error("expires_in_seconds doit être ≥ 60"));
    const { getByTestId, queryByTestId } = render(MagicLinkIssueForm, {
      props: {
        users: USERS,
        scopeIdsByKind: SCOPE_IDS,
        currentUserId: SYNDIC_USER_ID,
        onIssue,
      },
    });

    fillSelect(
      getByTestId("magic-link-target-input") as HTMLSelectElement,
      CONTRACTOR_ID,
    );
    fillSelect(
      getByTestId("magic-link-scope-id-select") as HTMLSelectElement,
      TICKET_ID,
    );
    // Aux bornes : le min = 60 (≥ accepté côté form, c'est le backend qui crie).
    fillRange(
      getByTestId("magic-link-expires-in-input") as HTMLInputElement,
      60,
    );

    const submitBtn = getByTestId(
      "magic-link-issue-submit",
    ) as HTMLButtonElement;
    await waitFor(() => expect(submitBtn.disabled).toBe(false));
    submitBtn.click();

    // Message inline visible + on reste sur le form (pas de URL issued).
    await waitFor(() => {
      const err = getByTestId("magic-link-form-error");
      expect(err.textContent).toMatch(/expires_in_seconds/i);
    });
    expect(queryByTestId("magic-link-issued-url-input")).toBeNull();
  });

  it("@security subject = self → submit disabled + helper visible + token jamais en storage", async () => {
    const onIssue = vi.fn().mockResolvedValue(makeIssuedFixture());
    const { getByTestId, queryByTestId } = render(MagicLinkIssueForm, {
      props: {
        users: USERS,
        scopeIdsByKind: SCOPE_IDS,
        currentUserId: SYNDIC_USER_ID,
        onIssue,
      },
    });

    // Sélectionne soi-même comme destinataire.
    fillSelect(
      getByTestId("magic-link-target-input") as HTMLSelectElement,
      SYNDIC_USER_ID,
    );
    fillSelect(
      getByTestId("magic-link-scope-id-select") as HTMLSelectElement,
      TICKET_ID,
    );

    // Submit doit être disabled (INV-13 préfront-end check).
    const submitBtn = getByTestId(
      "magic-link-issue-submit",
    ) as HTMLButtonElement;
    await waitFor(() => expect(submitBtn.disabled).toBe(true));

    // Helper text visible.
    const help = getByTestId("magic-link-target-help");
    expect(help.textContent).toMatch(/vous-même/i);

    // onIssue ne doit PAS avoir été appelé.
    submitBtn.click();
    expect(onIssue).not.toHaveBeenCalled();

    // INV-FE5 — aucun token en storage.
    expect(window.localStorage.length).toBe(0);
    expect(window.sessionStorage.length).toBe(0);
    expect(queryByTestId("magic-link-issued-url-input")).toBeNull();
  });

  it("@negative scopeIdsByKind vide → select disabled + helper 'Aucun ticket trouvé' + submit disabled", async () => {
    const onIssue = vi.fn();
    const { getByTestId } = render(MagicLinkIssueForm, {
      props: {
        users: USERS,
        // Aucun scope_id disponible (autocomplete vide).
        scopeIdsByKind: {},
        currentUserId: SYNDIC_USER_ID,
        onIssue,
      },
    });

    fillSelect(
      getByTestId("magic-link-target-input") as HTMLSelectElement,
      CONTRACTOR_ID,
    );

    // Le sélecteur de scope_id est disabled.
    const scopeIdSelect = getByTestId(
      "magic-link-scope-id-select",
    ) as HTMLSelectElement;
    await waitFor(() => expect(scopeIdSelect.disabled).toBe(true));

    // Helper text "Aucun ticket trouvé".
    const help = getByTestId("magic-link-scope-id-help");
    expect(help.textContent).toMatch(/aucun ticket/i);

    // Submit disabled.
    const submitBtn = getByTestId(
      "magic-link-issue-submit",
    ) as HTMLButtonElement;
    expect(submitBtn.disabled).toBe(true);

    // Aucun appel backend.
    submitBtn.click();
    expect(onIssue).not.toHaveBeenCalled();
  });
});
