import { describe, expect, it } from "vitest";
import { ICONES, iconeConnue } from "../icones";

/**
 * Une icône par destination, sans réemploi.
 *
 * ── Ce que le réemploi coûtait ───────────────────────────────────────────
 *
 * Mesuré sur `Navigation.svelte` avant remplacement : **42 entrées de menu
 * pour 33 icônes distinctes**, dont six collisions entre destinations
 * différentes.
 *
 *     📊  budgets / sondages
 *     📅  assemblées / réservations
 *     📋  états datés / devis
 *     📈  rapports PCMN / supervision
 *     🎫  tickets / mes tickets
 *     👤  copropriétaires / profil
 *
 * Deux entrées qui portent le même signe ne se distinguent plus que par leur
 * libellé. L'icône cesse alors d'informer : elle occupe la place d'un repère
 * sans en être un, ce qui est pire que pas d'icône du tout — l'œil s'y fie.
 *
 * La remise de design en fait une règle explicite :
 *
 * > **One distinct icon per nav entry** — no reuse across entries.
 *
 * ── Ce que cette garde vérifie, et ce qu'elle ne peut pas ────────────────
 *
 * Elle vérifie que le JEU d'icônes ne contient pas deux entrées au même
 * tracé. Elle ne peut pas vérifier que deux icônes différentes ne se
 * *ressemblent* pas — ça, c'est l'œil de la revue.
 */

/** Deux icônes sont identiques si leurs tracés le sont, dans le même ordre. */
function empreinte(traces: string[]): string {
  return traces.join("|");
}

describe("le jeu d'icônes n'en réemploie aucune", () => {
  it("ne contient pas deux noms au même tracé", () => {
    const parEmpreinte = new Map<string, string[]>();
    for (const [nom, traces] of Object.entries(ICONES)) {
      const cle = empreinte(traces);
      parEmpreinte.set(cle, [...(parEmpreinte.get(cle) ?? []), nom]);
    }

    const collisions = [...parEmpreinte.values()]
      .filter((noms) => noms.length > 1)
      .map((noms) => `  ${noms.join(" = ")}`);

    expect(
      collisions.join("\n"),
      "Ces noms d'icônes partagent exactement le même tracé. Deux " +
        "destinations qui portent le même signe ne se distinguent plus que " +
        "par leur libellé : l'icône occupe la place d'un repère sans en " +
        "être un.\n\n" +
        "C'est le défaut que le remplacement des émojis corrigeait — six " +
        "collisions sur 42 entrées de menu. Ne le réintroduisez pas.",
    ).toBe("");
  });

  it("donne à chaque icône au moins un tracé", () => {
    const vides = Object.entries(ICONES)
      .filter(
        ([, traces]) => traces.length === 0 || traces.some((t) => !t.trim()),
      )
      .map(([nom]) => `  ${nom}`);

    expect(
      vides.join("\n"),
      "Une icône sans tracé ne rend rien du tout, en silence. L'entrée de " +
        "menu perd son repère et personne ne s'en aperçoit.",
    ).toBe("");
  });

  it("lit bien le jeu, et n'est pas verte par vacuité", () => {
    // Vérification d'aveuglement : sans elle, un jeu vidé — import cassé,
    // export renommé — passerait au vert, et son zéro voudrait dire « je
    // n'ai rien regardé » plutôt que « rien à signaler ».
    expect(Object.keys(ICONES).length).toBeGreaterThan(30);
    expect(iconeConnue("buildings")).toBe(true);
    expect(iconeConnue("ceci-nexiste-pas")).toBe(false);
  });
});
