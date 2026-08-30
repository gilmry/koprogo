import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, extname } from 'node:path';

// Garde statique contre les clés i18n manquantes.
//
// Un audit du 2026-08-30 a relevé trois clés brutes visibles à l'écran
// (`documents.count`, `gamification.earned_stats`,
// `gamification.no_challenges_filter`). Le balayage systématique en a trouvé
// 124 : le testeur n'avait vu que celles affichées sur les pages qu'il avait
// ouvertes.
//
// Un test e2e serait le mauvais outil ici. Il faudrait visiter chaque page,
// dans chaque état, pour espérer croiser chaque clé — long, fragile, et
// toujours incomplet. Ce contrôle-ci lit le code, trouve toutes les clés
// utilisées, et vérifie qu'elles existent dans les quatre langues. Il tourne
// en une seconde et ne peut rien manquer.

const LOCALES = ['fr', 'nl', 'de', 'en'] as const;
const SRC = join(import.meta.dirname ?? __dirname, '..');

function flatten(obj: Record<string, unknown>, prefix = ''): Set<string> {
  const keys = new Set<string>();
  for (const [k, v] of Object.entries(obj)) {
    const key = `${prefix}${k}`;
    if (v !== null && typeof v === 'object' && !Array.isArray(v)) {
      for (const nested of flatten(v as Record<string, unknown>, `${key}.`)) keys.add(nested);
    } else {
      keys.add(key);
    }
  }
  return keys;
}

function sourceFiles(dir: string, acc: string[] = []): string[] {
  for (const entry of readdirSync(dir)) {
    if (entry === 'node_modules' || entry === 'dist') continue;
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) sourceFiles(full, acc);
    else if (['.svelte', '.astro', '.ts'].includes(extname(entry))) acc.push(full);
  }
  return acc;
}

/// Ne retient que les clés littérales : `$_('a.b')` ou `_('a.b')`. Les clés
/// construites dynamiquement (`$_(\`prefix.${x}\`)`) ne sont pas vérifiables
/// statiquement et sont ignorées à dessein.
function usedKeys(): Map<string, string[]> {
  // La quote fermante DOIT être suivie de `,` ou `)` : sans cela,
  // `$_("notices." + status)` serait pris pour la clé littérale `notices.`.
  const pattern = /\$?_\(\s*['"]([a-zA-Z0-9_.]+)['"]\s*[,)]/g;
  const found = new Map<string, string[]>();
  for (const file of sourceFiles(SRC)) {
    if (file.endsWith('.test.ts')) continue;
    const text = readFileSync(file, 'utf8');
    for (const match of text.matchAll(pattern)) {
      const key = match[1];
      if (!found.has(key)) found.set(key, []);
      found.get(key)!.push(file.slice(SRC.length + 1));
    }
  }
  return found;
}

const catalogs = Object.fromEntries(
  LOCALES.map((l) => [l, flatten(JSON.parse(readFileSync(join(SRC, 'locales', `${l}.json`), 'utf8')))]),
) as Record<(typeof LOCALES)[number], Set<string>>;

// Les valeurs d'énumération sont rendues via une clé construite
// (`$_(\`bookings.resourceType.${type}\`)`), donc invisibles au balayage
// statique ci-dessus. Elles étaient affichées brutes à l'écran
// (« MeetingRoom », « LostAndFound ») avant le 2026-08-30 ; ce contrôle-ci
// interdit qu'une valeur ajoutée à l'enum réapparaisse non traduite.
const DYNAMIC_ENUMS: Record<string, string[]> = {
  'bookings.resourceType': [
    'MeetingRoom', 'PartyRoom', 'Gym', 'SwimmingPool', 'Sauna', 'ParkingSpace',
    'GuestRoom', 'Rooftop', 'Garden', 'LaundryRoom', 'StorageSpace',
    'CoworkingSpace', 'Other',
  ],
  'notices.noticeType': ['Announcement', 'Event', 'LostAndFound', 'ClassifiedAd'],
};

describe('énumérations traduites', () => {
  it.each(LOCALES)('%s traduit chaque valeur d\'énumération', (locale) => {
    const missing: string[] = [];
    for (const [prefix, values] of Object.entries(DYNAMIC_ENUMS)) {
      for (const value of values) {
        if (!catalogs[locale].has(`${prefix}.${value}`)) missing.push(`${prefix}.${value}`);
      }
    }
    expect(missing, `valeurs non traduites en ${locale}`).toEqual([]);
  });

  // Le tableau ci-dessus doit suivre les enums du code : sans ce contrôle,
  // une valeur ajoutée à ResourceType passerait entre les mailles.
  it('la liste de contrôle couvre les enums réels', () => {
    const bookings = readFileSync(join(SRC, 'lib/api/bookings.ts'), 'utf8');
    const declared = [...bookings.matchAll(/^\s{2}(\w+) = "\w+",$/gm)]
      .map((m) => m[1]);
    const resourceTypes = declared.slice(0, DYNAMIC_ENUMS['bookings.resourceType'].length);
    expect(new Set(resourceTypes)).toEqual(new Set(DYNAMIC_ENUMS['bookings.resourceType']));

    const notices = readFileSync(join(SRC, 'lib/api/notices.ts'), 'utf8');
    const noticeBlock = notices.slice(notices.indexOf('export const NoticeType'));
    const noticeValues = [...noticeBlock.slice(0, noticeBlock.indexOf('}')).matchAll(/(\w+): "/g)]
      .map((m) => m[1]);
    expect(new Set(noticeValues)).toEqual(new Set(DYNAMIC_ENUMS['notices.noticeType']));
  });
});

describe('catalogues de traduction', () => {
  it.each(LOCALES)('%s contient toutes les clés utilisées dans le code', (locale) => {
    const missing: string[] = [];
    for (const [key, files] of usedKeys()) {
      if (!catalogs[locale].has(key)) missing.push(`${key}  (${files[0]})`);
    }
    expect(missing, `${missing.length} clé(s) absente(s) de ${locale}.json :\n  ${missing.join('\n  ')}`).toEqual([]);
  });

  // Les quatre catalogues doivent rester alignés : une clé ajoutée en
  // français et oubliée ailleurs produit un affichage anglais isolé, le
  // symptôme exact relevé sur la page Notifications.
  it.each(LOCALES.filter((l) => l !== 'fr'))('%s a exactement les mêmes clés que fr', (locale) => {
    const fr = catalogs.fr;
    const other = catalogs[locale];
    expect([...fr].filter((k) => !other.has(k)), `absentes de ${locale}.json`).toEqual([]);
    expect([...other].filter((k) => !fr.has(k)), `en trop dans ${locale}.json`).toEqual([]);
  });
});
