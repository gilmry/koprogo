import { render, waitFor, fireEvent } from "../test-helpers";
import { describe, it, expect, vi } from "vitest";
import MeetingCreateModal from "./MeetingCreateModal.svelte";

vi.mock("../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

vi.mock("../lib/api", () => ({
  api: { post: vi.fn(), get: async () => ({}) },
}));

/**
 * L'avertissement de délai arrive À LA SAISIE, pas au moment de convoquer.
 *
 * Recette du 2026-09-06 (RN-9). Une assemblée créée pour dans cinq jours ne
 * peut plus être convoquée régulièrement (Art. 3.87 § 3, quinze jours de
 * préavis). L'application ne le disait qu'au clic sur « Créer une
 * convocation » — au moment où il ne restait plus qu'à subir. Le syndic n'en
 * sortait qu'en supprimant l'assemblée, le bouton « Reporter » ne lui offrant
 * aucune sortie.
 *
 * La règle était juste, sa temporalité ne l'était pas.
 *
 * **Ce test vérifie aussi qu'on n'INTERDIT pas.** Le texte prévoit lui-même
 * l'urgence, une assemblée peut être encodée après coup, et une seconde
 * convocation subit la date de l'échec précédent. Un avertissement qui
 * bloquerait rendrait ces trois cas impossibles.
 *
 * Voir #780, verrou 1.
 */
describe("MeetingCreateModal — le délai de convocation s'annonce à la saisie (#780)", () => {
  const dansNJours = (n: number) =>
    new Date(Date.now() + n * 86_400_000).toISOString().slice(0, 16);

  const champDeDate = () =>
    document.querySelector(
      '[data-testid="input-meeting-date"]',
    ) as HTMLInputElement | null;

  it("avertit quand la date rend la convocation impossible", async () => {
    render(MeetingCreateModal, { props: {} });

    const champ = champDeDate();
    expect(champ, "le champ de date a disparu").not.toBeNull();

    await fireEvent.input(champ!, { target: { value: dansNJours(5) } });

    await waitFor(
      () =>
        expect(
          document.querySelector(
            '[data-testid="meeting-date-delai-trop-court"]',
          ),
          "Aucun avertissement pour une assemblée à cinq jours. Le syndic " +
            "découvrira l'impasse au moment de convoquer, quand il ne restera " +
            "plus qu'à supprimer l'assemblée (#780).",
        ).not.toBeNull(),
      { timeout: 3000 },
    );
  });

  it("annonce la date limite d'envoi quand le délai tient", async () => {
    render(MeetingCreateModal, { props: {} });

    await fireEvent.input(champDeDate()!, {
      target: { value: dansNJours(40) },
    });

    await waitFor(
      () =>
        expect(
          document.querySelector('[data-testid="meeting-date-delai-tenable"]'),
          "La date limite d'envoi n'est pas annoncée. Le syndic doit la " +
            "calculer de tête alors que le produit la connaît.",
        ).not.toBeNull(),
      { timeout: 3000 },
    );
  });

  it("n'empêche pas de saisir une date trop proche", async () => {
    render(MeetingCreateModal, { props: {} });

    const champ = champDeDate()!;
    await fireEvent.input(champ, { target: { value: dansNJours(2) } });

    await waitFor(() =>
      expect(
        document.querySelector('[data-testid="meeting-date-delai-trop-court"]'),
      ).not.toBeNull(),
    );

    // Le champ garde sa valeur et n'est pas désactivé : l'urgence est prévue
    // par l'Art. 3.87 § 3 lui-même, une assemblée peut être encodée après
    // coup, et une seconde convocation subit la date de l'échec précédent.
    expect(champ.disabled).toBe(false);
  });
});
