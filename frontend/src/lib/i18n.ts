import { register, init, getLocaleFromNavigator } from "svelte-i18n";

// Lazy-load locales: each import() creates a separate chunk
// Only the active locale is loaded at startup, others on demand
// Vite code-splits these into ~7 KB gzipped chunks each
register("fr", () => import("../locales/fr.json"));
register("nl", () => import("../locales/nl.json"));
register("de", () => import("../locales/de.json"));
register("en", () => import("../locales/en.json"));

// Priority: 1) user preference (localStorage), 2) browser language, 3) "fr"
const supportedLocales = ["nl", "fr", "de", "en"];
const savedLocale =
  typeof localStorage !== "undefined"
    ? localStorage.getItem("preferred-language")
    : null;

function resolveLocale(): string {
  // 1. User's saved preference
  if (savedLocale && supportedLocales.includes(savedLocale)) {
    return savedLocale;
  }
  // 2. Browser language (e.g. "fr-BE" → "fr")
  const browserLocale = getLocaleFromNavigator()?.split("-")[0];
  if (browserLocale && supportedLocales.includes(browserLocale)) {
    return browserLocale;
  }
  // 3. Default: French (Belgian context)
  return "fr";
}

init({
  fallbackLocale: "fr",
  initialLocale: resolveLocale(),
});

// Export language options for selector
export const languages = [
  { code: "nl", name: "Nederlands", flag: "🇧🇪", priority: 1 },
  { code: "fr", name: "Français", flag: "🇧🇪", priority: 2 },
  { code: "de", name: "Deutsch", flag: "🇩🇪", priority: 3 },
  { code: "en", name: "English", flag: "🇬🇧", priority: 4 },
] as const;

export type LanguageCode = (typeof languages)[number]["code"];
