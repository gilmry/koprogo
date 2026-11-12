// WCAG 2.5.3 « Label in Name » : le nom accessible d'un contrôle contient
// son texte visible. C'est la règle `label-content-name-mismatch` d'axe.
//
// Pourquoi une réimplémentation pour les tests : dans jsdom, axe ne peut pas
// trancher cette règle faute de mise en page. Il la rend « incomplete », ni
// passée ni violée (mesuré le 2026-11-05), et un test qui n'exige que
// l'absence de violation reste donc vert quoi qu'il arrive.

/** Texte visible d'un élément : hors sous-arbres `aria-hidden`, sans symboles. */
export function texteVisible(element: Element): string {
  const clone = element.cloneNode(true) as Element;
  clone.querySelectorAll('[aria-hidden="true"]').forEach((n) => n.remove());
  return normaliser(clone.textContent ?? "");
}

/** Nom accessible quand il vient de `aria-label` (le cas des contrôles visés). */
export function nomAccessible(element: Element): string {
  return normaliser(element.getAttribute("aria-label") ?? "");
}

export function libelleDansLeNom(element: Element): boolean {
  return nomAccessible(element).includes(texteVisible(element));
}

function normaliser(texte: string): string {
  return texte
    .replace(/[^\p{L}\p{N}\s]/gu, " ")
    .replace(/\s+/g, " ")
    .trim()
    .toLowerCase();
}
