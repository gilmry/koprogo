import { addMessages, init } from "svelte-i18n";

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
// Priority: 1) user preference (localStorage), 2) default "fr"
const supportedLocales = ["nl", "fr", "de", "en"];
const savedLocale =
  typeof localStorage !== "undefined"
    ? localStorage.getItem("preferred-language")
    : null;
const initialLocale =
  (savedLocale && supportedLocales.includes(savedLocale)
    ? savedLocale
    : null) ?? "fr";

init({
  fallbackLocale: "fr",
  initialLocale,
});

// Export language options for selector
export const languages = [
  { code: "nl", name: "Nederlands", flag: "🇧🇪", priority: 1 },
  { code: "fr", name: "Français", flag: "🇧🇪", priority: 2 },
  { code: "de", name: "Deutsch", flag: "🇩🇪", priority: 3 },
  { code: "en", name: "English", flag: "🇬🇧", priority: 4 },
] as const;

export type LanguageCode = (typeof languages)[number]["code"];
