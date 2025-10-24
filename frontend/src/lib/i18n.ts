import { addMessages, init, getLocaleFromNavigator } from "svelte-i18n";

import nl from "../locales/nl.json";
import fr from "../locales/fr.json";
import de from "../locales/de.json";
import en from "../locales/en.json";

// Register all messages
addMessages("nl", nl);
addMessages("fr", fr);
addMessages("de", de);
addMessages("en", en);

// Initialize i18n
init({
  fallbackLocale: "nl", // Dutch is default (60% of Belgium)
  initialLocale: getLocaleFromNavigator(),
});

// Export language options for selector
export const languages = [
  { code: "nl", name: "Nederlands", flag: "🇳🇱", priority: 1 },
  { code: "fr", name: "Français", flag: "🇫🇷", priority: 2 },
  { code: "de", name: "Deutsch", flag: "🇩🇪", priority: 3 },
  { code: "en", name: "English", flag: "🇬🇧", priority: 4 },
] as const;

export type LanguageCode = (typeof languages)[number]["code"];
