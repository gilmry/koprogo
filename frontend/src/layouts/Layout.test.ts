import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, extname } from 'node:path';

// Le titre de page ne doit jamais répéter la marque.
//
// Audit du 2026-08-30 : toutes les pages s'affichaient
// « KoproGo - ... - KoproGo ». Le layout ajoutait « - KoproGo » sans
// condition, alors que 40 des 53 pages passaient déjà un titre contenant la
// marque.
//
// Ce contrôle est statique plutôt qu'e2e : la règle vit dans le layout et
// s'applique aux 115 pages générées. Les vérifier une par une dans un
// navigateur coûterait des minutes pour une propriété qui se lit en une
// ligne.

const SITE_NAME = 'KoproGo';
const PAGES = join(import.meta.dirname ?? __dirname, '..', 'pages');

function pageTitle(raw: string): string {
  return raw.includes(SITE_NAME) ? raw : `${raw} - ${SITE_NAME}`;
}

function astroPages(dir: string, acc: string[] = []): string[] {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) astroPages(full, acc);
    else if (extname(entry) === '.astro') acc.push(full);
  }
  return acc;
}

describe('titre de page', () => {
  it('ajoute la marque quand elle manque', () => {
    expect(pageTitle('Notifications')).toBe('Notifications - KoproGo');
  });

  it('ne la répète pas quand elle est déjà en suffixe', () => {
    expect(pageTitle('Devis - KoproGo')).toBe('Devis - KoproGo');
  });

  it('ne la répète pas quand elle est déjà en préfixe', () => {
    expect(pageTitle('KoproGo - Gestion de Copropriété SaaS')).toBe(
      'KoproGo - Gestion de Copropriété SaaS',
    );
  });

  it('applique la même règle que le layout', () => {
    // Garde-fou : si quelqu'un change la règle dans Layout.astro sans
    // toucher ce test, la divergence est signalée ici.
    const layout = readFileSync(join(PAGES, '..', 'layouts', 'Layout.astro'), 'utf8');
    expect(layout).toContain('title.includes(SITE_NAME) ? title : `${title} - ${SITE_NAME}`');
  });

  it("aucune page ne produit un titre où la marque apparaît deux fois", () => {
    const offenders: string[] = [];
    for (const file of astroPages(PAGES)) {
      const match = readFileSync(file, 'utf8').match(/<Layout[^>]*\btitle=["']([^"']+)["']/);
      if (!match) continue;
      const rendered = pageTitle(match[1]);
      const occurrences = rendered.split(SITE_NAME).length - 1;
      if (occurrences > 1) offenders.push(`${file.slice(PAGES.length + 1)} → ${rendered}`);
    }
    expect(offenders, 'titres contenant deux fois la marque').toEqual([]);
  });
});
