# Agent activity — 2026-10-31 — Six ADR de décision pour « l'ACP au centre »

**Persona :** jarvis (Tier 2 — proposal)

**Contexte :** lettre de mission du PO du 2026-10-31 (KoproGo v0.1.0 : l'ACP au
centre, portabilité et isolation), §3 « décisions réservées à Gilles, à préparer,
pas à trancher ». Sert l'issue #1046.

## Actions Tier 2 effectuées

- Vérification en lecture seule des constats de la lettre sur `dadf78d4` :
  aucune occurrence de `syndic_mandates` dans `migrations/` ni `src/` ;
  `mcp_sse_handlers.rs` sans appel `verify_` ; `20250103000002_disable_rls_policies.sql`
  présent ; `ON DELETE CASCADE` sur `organization_id` dans `budgets`,
  `journal_entries`, `call_for_funds` et `owners` ; 6 fichiers de `backend/src`
  et 43 de `docs/` citent encore « 577- ».
- Rédaction des ADR 0055 à 0060 (statut Proposed), chacune avec contexte,
  options, recommandation et réversibilité.
- Six 🔴 inscrits dans `RELEASE.md`, section « Position courante », avec la
  mise à jour de l'en-tête.
- Issue #1046 ouverte (décision, lots, « Fini quand »), sans détail d'exposition.

## Ce qui n'a PAS été exécuté (hors délégation / hors portée session)

- Aucune décision tranchée : les six ADR restent Proposed jusqu'à la signature
  du PO.
- Aucun code, aucune migration, aucun test : les lots 0 et 1 attendent une
  story signée et le sprint S-B.
- Aucune fusion : la branche `docs/adr-acp-au-centre` est soumise en pull
  request brouillon.

## Vérification

- Relecture des six ADR et de la table des 🔴 (liens relatifs vers
  `docs/adr/`), aucune commande de build lancée : changement de documentation
  seulement.
