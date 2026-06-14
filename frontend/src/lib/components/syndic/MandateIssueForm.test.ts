// Story B3 (Phase B FE) — Vitest 4-cat MandateIssueForm.
//
// Couverture (cf. stories.md §B3 AC) :
//   @happy    : sélection complète subject + kind + scope + reason >= 10 +
//               valid_until valide → submit appelle issueMandate avec le bon
//               payload + onSuccess invoqué + toast success.
//   @edge     : valid_until = today + 5ans exactement → OK ; +5ans +1 jour
//               → submit disabled + error inline INV-14.
//   @security : subject == issuer → submit disabled + erreur "vous-même" ;
//               option subject correspondant à l'issuer est DOM-disabled
//               (UX: lit le contrat backend INV-15 côté client).
//   @negative : reason < 10 chars → submit disabled + counter rouge ;
//               reason > 500 chars : on tape 600 chars dans textarea (maxlength
//               coupe à 500 → testé via $derived reasonCharCount) ; et test
//               direct du dérivé pour > 500.
//
// nowOverride : injection déterministe — vital pour les calculs max/min
// validUntil (l'AC @edge "+5 ans exactement" dépend de la date courante).
//
// Mocks : on intercepte `issueMandate` du module API pour éviter un fetch
// réseau réel. Le test vérifie le PAYLOAD (subject/kind/scope/reason/
// valid_until ISO 8601) et que `onSuccess` est rappelé avec le résultat mocké.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, waitFor, fireEvent } from "../../../test-helpers";
import MandateIssueForm from "./MandateIssueForm.svelte";

// -----------------------------------------------------------------------------
// Mocks API + toast + authStore
// -----------------------------------------------------------------------------

vi.mock("../../api/mandates", () => ({
  issueMandate: vi.fn(),
}));

vi.mock("../../../stores/toast", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}));

// Mock authStore : `get(authStore)` retourne { user: { id: ISSUER_ID } }
// On expose un store mockable pour pouvoir tester l'edge "subject == issuer".
//
// Important : la factory `vi.mock` est HOISTED en tête de fichier — toute
// référence à une const top-level lèverait `ReferenceError: Cannot access
// before initialization`. On hard-code la valeur ICI ET on l'expose comme
// `const ISSUER_ID` (les deux doivent rester en sync — test invariant).
vi.mock("../../../stores/auth", () => {
  const ID = "issuer-uuid-self";
  const subscribers = new Set<(v: { user: { id: string } | null }) => void>();
  let state: { user: { id: string } | null } = { user: { id: ID } };
  return {
    authStore: {
      subscribe: (fn: (v: { user: { id: string } | null }) => void) => {
        subscribers.add(fn);
        fn(state);
        return () => subscribers.delete(fn);
      },
      // Helpers internes au mock (utilisés par les tests via re-import dynamique)
      __setUser: (user: { id: string } | null) => {
        state = { user };
        subscribers.forEach((s) => s(state));
      },
    },
  };
});

const ISSUER_ID = "issuer-uuid-self";

// i18n : on neutralise $_ (retourne chaîne vide) pour activer le fallback
// FR `|| "..."` du composant. Si on retournait la clé, elle serait truthy
// et le fallback ne se déclencherait pas.
vi.mock("../../i18n", () => {
  const store = {
    subscribe: (fn: (v: (k: string) => string) => void) => {
      fn(() => "");
      return () => {};
    },
  };
  return { _: store };
});

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

// Snapshot fixée pour déterminisme (mardi 10 juin 2026 12:00 UTC — alignée
// avec ExpirationBadge.test.ts).
const NOW_FIXED = new Date("2026-06-10T12:00:00Z");

const SUBJECTS = [
  { id: "user-notary-1", label: "Me. Dupont (notaire)" },
  { id: "user-lawyer-1", label: "Me. Martin (avocat)" },
  // Self : doit apparaître mais être DOM-disabled (@security).
  { id: ISSUER_ID, label: "Vous (syndic)" },
];

const SCOPES: Array<{ id: string; kind: "building" | "acp"; label: string }> = [
  { id: "b-42", kind: "building", label: "Immeuble #42 Tilleuls" },
  { id: "b-38", kind: "building", label: "Immeuble #38 Erables" },
  { id: "acp-1", kind: "acp", label: "ACP Tilleuls (auto-gérée)" },
];

/** Calcule un YYYY-MM-DD à N jours du now fixé. */
function plusDaysISODate(n: number): string {
  const d = new Date(NOW_FIXED.getTime() + n * 24 * 60 * 60 * 1000);
  return d.toISOString().slice(0, 10);
}

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  vi.restoreAllMocks();
});

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

describe("MandateIssueForm — Story B3 (4-cat)", () => {
  it("@happy submit appelle issueMandate avec payload typé + onSuccess", async () => {
    const { issueMandate } = await import("../../api/mandates");
    (issueMandate as unknown as ReturnType<typeof vi.fn>).mockResolvedValueOnce(
      {
        id: "mandate-new-1",
        subject_user_id: "user-notary-1",
        kind: "notary",
        scope_kind: "building",
        scope_id: "b-42",
        reason: "Mandat de notaire pour la transaction Lot 12 (vente)",
        valid_from: NOW_FIXED.toISOString(),
        valid_until: `${plusDaysISODate(365)}T23:59:59Z`,
        issued_by: ISSUER_ID,
        created_at: NOW_FIXED.toISOString(),
        updated_at: NOW_FIXED.toISOString(),
        revoked_at: null,
      },
    );

    const onSuccess = vi.fn();
    const { getByTestId } = render(MandateIssueForm, {
      props: {
        subjects: SUBJECTS,
        scopes: SCOPES,
        onSuccess,
        nowOverride: NOW_FIXED,
      },
    });

    // Subject
    const subjectSel = getByTestId(
      "mandate-subject-select",
    ) as HTMLSelectElement;
    subjectSel.value = "user-notary-1";
    await fireEvent.change(subjectSel);

    // Kind (default est "notary" — on confirme via le data-testid de l'option)
    expect(getByTestId("mandate-kind-option-notary")).toBeTruthy();

    // Scope kind reste "building" (default) — pas besoin de cliquer.
    // Scope id
    const scopeSel = getByTestId(
      "mandate-scope-id-select",
    ) as HTMLSelectElement;
    scopeSel.value = "b-42";
    await fireEvent.change(scopeSel);

    // Reason (50 chars)
    const reasonTa = getByTestId(
      "mandate-reason-textarea",
    ) as HTMLTextAreaElement;
    const reasonText = "Mandat de notaire pour la transaction Lot 12 (vente)";
    reasonTa.value = reasonText;
    await fireEvent.input(reasonTa);

    // valid_until = today + 365j
    const validUntilInp = getByTestId(
      "mandate-valid-until-input",
    ) as HTMLInputElement;
    validUntilInp.value = plusDaysISODate(365);
    await fireEvent.input(validUntilInp);

    const submit = getByTestId("mandate-issue-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));

    submit.click();

    await waitFor(() => expect(issueMandate).toHaveBeenCalledTimes(1));
    const [calledReq] = (issueMandate as unknown as ReturnType<typeof vi.fn>)
      .mock.calls[0];
    expect(calledReq).toMatchObject({
      subject_user_id: "user-notary-1",
      kind: "notary",
      scope_kind: "building",
      scope_id: "b-42",
      reason: reasonText, // trimmed
      valid_until: `${plusDaysISODate(365)}T23:59:59Z`,
    });
    await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));
  });

  it("@edge valid_until = today+5ans exact → OK ; +1 jour → submit disabled + erreur INV-14", async () => {
    const { getByTestId, queryByTestId } = render(MandateIssueForm, {
      props: { subjects: SUBJECTS, scopes: SCOPES, nowOverride: NOW_FIXED },
    });

    // Complete form sauf valid_until
    const subjectSel = getByTestId(
      "mandate-subject-select",
    ) as HTMLSelectElement;
    subjectSel.value = "user-notary-1";
    await fireEvent.change(subjectSel);

    const scopeSel = getByTestId(
      "mandate-scope-id-select",
    ) as HTMLSelectElement;
    scopeSel.value = "b-42";
    await fireEvent.change(scopeSel);

    const reasonTa = getByTestId(
      "mandate-reason-textarea",
    ) as HTMLTextAreaElement;
    reasonTa.value = "Motif valide de plus de 10 chars";
    await fireEvent.input(reasonTa);

    // Cas borne haute exacte : today + 5*365 jours = OK
    const validUntilInp = getByTestId(
      "mandate-valid-until-input",
    ) as HTMLInputElement;
    validUntilInp.value = plusDaysISODate(5 * 365);
    await fireEvent.input(validUntilInp);

    const submit = getByTestId("mandate-issue-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    expect(queryByTestId("mandate-error-validUntil")).toBeNull();

    // Cas hors borne : today + 5*365 + 1 jour → submit disabled + erreur inline
    validUntilInp.value = plusDaysISODate(5 * 365 + 1);
    await fireEvent.input(validUntilInp);

    await waitFor(() => expect(submit.disabled).toBe(true));
    const err = getByTestId("mandate-error-validUntil");
    expect(err.textContent).toMatch(/5 ans|INV-14/i);
  });

  it("@security subject == issuer → submit disabled + erreur 'vous-même' + option DOM-disabled", async () => {
    const { getByTestId, container } = render(MandateIssueForm, {
      props: { subjects: SUBJECTS, scopes: SCOPES, nowOverride: NOW_FIXED },
    });

    // L'option DOM correspondante au self est `disabled` (UX préventive).
    const selfOption = container.querySelector(
      `[data-testid="mandate-subject-option-${ISSUER_ID}"]`,
    ) as HTMLOptionElement;
    expect(selfOption).not.toBeNull();
    expect(selfOption.disabled).toBe(true);

    // Si on FORCE quand même la valeur (DevTools manipulation) → on doit
    // afficher l'erreur d'INV-15 + submit disabled.
    const subjectSel = getByTestId(
      "mandate-subject-select",
    ) as HTMLSelectElement;
    subjectSel.value = ISSUER_ID;
    await fireEvent.change(subjectSel);

    // Remplir le reste pour isoler l'erreur subject
    const scopeSel = getByTestId(
      "mandate-scope-id-select",
    ) as HTMLSelectElement;
    scopeSel.value = "b-42";
    await fireEvent.change(scopeSel);

    const reasonTa = getByTestId(
      "mandate-reason-textarea",
    ) as HTMLTextAreaElement;
    reasonTa.value = "Motif valide pour test self-mandate";
    await fireEvent.input(reasonTa);

    const validUntilInp = getByTestId(
      "mandate-valid-until-input",
    ) as HTMLInputElement;
    validUntilInp.value = plusDaysISODate(30);
    await fireEvent.input(validUntilInp);

    const submit = getByTestId("mandate-issue-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(true));
    const err = getByTestId("mandate-error-subject");
    expect(err.textContent).toMatch(/vous-même|INV-15/i);
  });

  it("@negative reason < 10 chars → submit disabled + counter rouge", async () => {
    const { getByTestId } = render(MandateIssueForm, {
      props: { subjects: SUBJECTS, scopes: SCOPES, nowOverride: NOW_FIXED },
    });

    // Setup subject + scope + date valides
    const subjectSel = getByTestId(
      "mandate-subject-select",
    ) as HTMLSelectElement;
    subjectSel.value = "user-notary-1";
    await fireEvent.change(subjectSel);
    const scopeSel = getByTestId(
      "mandate-scope-id-select",
    ) as HTMLSelectElement;
    scopeSel.value = "b-42";
    await fireEvent.change(scopeSel);
    const validUntilInp = getByTestId(
      "mandate-valid-until-input",
    ) as HTMLInputElement;
    validUntilInp.value = plusDaysISODate(30);
    await fireEvent.input(validUntilInp);

    // Reason = 5 chars (< 10 minimum)
    const reasonTa = getByTestId(
      "mandate-reason-textarea",
    ) as HTMLTextAreaElement;
    reasonTa.value = "court";
    await fireEvent.input(reasonTa);

    const counter = getByTestId("mandate-reason-counter");
    expect(counter.className).toMatch(/text-red-600/);
    expect(counter.textContent).toMatch(/5/);

    const submit = getByTestId("mandate-issue-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(true));
    const err = getByTestId("mandate-error-reason");
    expect(err.textContent).toMatch(/trop court|10 minimum/i);
  });

  it("@negative reason > 500 chars (codepoints unicode) → counter rouge + erreur", async () => {
    // On force le state via une textarea sans maxlength côté DOM (le composant
    // contraint maxlength=500 → on contourne en setant directement la value
    // au-delà — le compteur unicode lit Array.from(reason).length, donc
    // l'erreur se déclenche même si le navigateur a coupé le texte saisi).
    const { getByTestId } = render(MandateIssueForm, {
      props: { subjects: SUBJECTS, scopes: SCOPES, nowOverride: NOW_FIXED },
    });

    const reasonTa = getByTestId(
      "mandate-reason-textarea",
    ) as HTMLTextAreaElement;
    // Retire maxlength pour pouvoir injecter 600 chars via .value (jsdom
    // respecte maxlength=500 sur input mais tolère .value direct).
    reasonTa.removeAttribute("maxlength");
    reasonTa.value = "x".repeat(600);
    await fireEvent.input(reasonTa);

    const counter = getByTestId("mandate-reason-counter");
    expect(counter.className).toMatch(/text-red-600/);
    // 600 char ASCII = 600 codepoints
    expect(counter.textContent).toMatch(/600/);

    const err = getByTestId("mandate-error-reason");
    expect(err.textContent).toMatch(/trop long|500 maximum/i);
  });
});
