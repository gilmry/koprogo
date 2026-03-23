import { init, getLocaleFromNavigator, addMessages } from "svelte-i18n";
import frMessages from "../locales/fr.json";
import nlMessages from "../locales/nl.json";
import deMessages from "../locales/de.json";
import enMessages from "../locales/en.json";

// Load all locales synchronously to prevent hydration race condition
// on Astro islands (svelte-i18n requires messages before first render)
addMessages("fr", frMessages);
addMessages("nl", nlMessages);
addMessages("de", deMessages);
addMessages("en", enMessages);

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
