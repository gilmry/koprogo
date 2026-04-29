# docs/archive/

Snapshots historiques préservés pour référence chronologique (cleanup #426, 2026-04-29).

Politique : **les nouveaux documents ne vont PAS ici**. Cette directory contient uniquement des snapshots à un instant T qui ont drifté avec le code (FRONTEND_INVENTORY, IMPLEMENTATION_SUMMARY, etc.).

## Pourquoi archiver et pas supprimer

Ces fichiers gardent une valeur de témoignage historique :
- `2026-04-29-frontend-inventory.md` — inventaire des composants Svelte au moment du big audit qualité.
- `2026-04-29-implementation-summary.md` — état de l'implémentation backend à la même date.
- `2026-04-29-infrastructure-deployment-summary.md` — snapshot infra (avant les recettes #425-#429).
- `2026-04-29-mcp-integration-summary.md` — état MCP integration.

Pour la **vérité actuelle** sur ces sujets, lire les sources :
- Inventaire frontend : `find frontend/src/components -name '*.svelte' | wc -l` ou WBS auto-régénéré (cf. #428).
- Implementation status : issues GitHub ouvertes/fermées + `git log`.
- Infrastructure : `infrastructure/SECURITY.md` + issues #425/#429.
- MCP integration : voir `docker-compose.mcp.yml` + repo MCP servers liés.

## Convention de nommage

`YYYY-MM-DD-<topic>.md` — date du snapshot prefixée pour tri chronologique.
