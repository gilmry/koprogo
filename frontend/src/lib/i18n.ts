import { init, getLocaleFromNavigator, addMessages } from "svelte-i18n";
import frMessages from "../locales/fr.json";
import nlMessages from "../locales/nl.json";
import deMessages from "../locales/de.json";
import enMessages from "../locales/en.json";

let initialized = false;

/**
 * Initialize svelte-i18n synchronously. Safe to call multiple times.
 * Must be called before any $_() usage in Svelte components.
 */
export function setupI18n() {
  if (initialized) return;
  initialized = true;

  addMessages("fr", frMessages);
  addMessages("nl", nlMessages);
  addMessages("de", deMessages);
  addMessages("en", enMessages);

  const supportedLocales = ["nl", "fr", "de", "en"];
  const savedLocale =
    typeof localStorage !== "undefined"
      ? localStorage.getItem("preferred-language")
      : null;

  function resolveLocale(): string {
    if (savedLocale && supportedLocales.includes(savedLocale)) {
      return savedLocale;
    }
    const browserLocale = getLocaleFromNavigator()?.split("-")[0];
    if (browserLocale && supportedLocales.includes(browserLocale)) {
      return browserLocale;
    }
    return "fr";
  }

  init({
    fallbackLocale: "fr",
    initialLocale: resolveLocale(),
  });
}

// Auto-init when imported as side-effect (for backward compat)
setupI18n();

// Export language options for selector
export const languages = [
  { code: "nl", name: "Nederlands", flag: "🇧🇪", priority: 1 },
  { code: "fr", name: "Français", flag: "🇧🇪", priority: 2 },
  { code: "de", name: "Deutsch", flag: "🇩🇪", priority: 3 },
  { code: "en", name: "English", flag: "🇬🇧", priority: 4 },
] as const;

export type LanguageCode = (typeof languages)[number]["code"];
