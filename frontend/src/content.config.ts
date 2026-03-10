import { defineCollection, z } from "astro:content";
import { glob } from "astro/loaders";

const blog = defineCollection({
  loader: glob({ pattern: "**/*.md", base: "./src/content/blog" }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    category: z.enum([
      "syndic",
      "coproprietaire",
      "assemblee-generale",
      "comptabilite",
      "locataire",
      "conseil-copropriete",
      "notaire",
      "rgpd",
      "acp",
    ]),
    tags: z.array(z.string()),
    publishedAt: z.coerce.date(),
    legalRefs: z.array(z.string()).optional(),
  }),
});

export const collections = { blog };
