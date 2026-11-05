---
feature: mcp-portee-et-droit
phase: Brief court (lot 0 de la v0.1.0, sécurité immédiate)
status: SIGNED v1.0 par @gilmry 2026-11-04
date: "2026-11-03"
authors: [Claude (drafting)]
related_issues: [1046]
related_adr: [0059]
parent_maury: lettre de mission v0.1.0 « l'ACP au centre », lot 0
---

# Brief — Le MCP applique les mêmes gardes que l'API REST, et ne contredit pas le domaine

## 1. Vision

Le serveur MCP (`infrastructure/web/handlers/mcp_sse_handlers.rs`) est une
deuxième porte d'entrée sur les mêmes données que l'API REST. Il doit en
respecter exactement les règles :

1. **Portée.** Un outil qui reçoit un identifiant (`building_id`,
   `owner_id`, `meeting_id`) ne répond que si l'appelant aurait le droit de
   lire la même ressource par l'API REST. Mêmes gardes, mêmes refus.
2. **Rôle.** `caller_from_role` (`middleware/scope_guard.rs`) ne promeut
   jamais un `admin` sans organisation en SuperAdmin.
3. **Honnêteté.** Un outil qui n'écrit rien ne répond pas « enregistré », et
   un outil qui renvoie des données codées en dur ne se présente pas comme
   une lecture de la base (décision 5, ADR 0059).
4. **Droit.** Les outils juridiques (`legal_search`, `majority_calculator`,
   `travaux_qualifier`) disent ce que dit `domain/copropriete/majorites.rs`,
   qui est la source de vérité : travaux aux parties communes aux 2/3
   (art. 3.88 § 1er, 1°, b), seuil de mise en concurrence fixé par l'AG aux
   2/3 (art. 3.88 § 1er, 1°, c), pas de « trois devis obligatoires ».

## 2. Personas concernés

- **Syndic d'une organisation A** : ne doit voir, par le MCP, que ce qu'il
  voit par l'API REST.
- **Admin sans organisation** : n'a pas les droits d'un SuperAdmin.
- **Copropriétaire** qui interroge l'assistant : reçoit une règle de droit
  exacte.

## 3. Hors périmètre

- Le modèle par mandat (lots 2 et 3) : les gardes utilisées ici sont celles
  qui existent aujourd'hui côté REST (`verify_*_org_access`). Quand le lot 3
  remplacera la garde REST, le MCP suivra par construction, puisqu'il
  appellera la même.
- Toute nouvelle fonction MCP.

## 4. Décisions attendues avant le code

- **Signature de ce brief et des stories.**
- **ADR 0059** (décision 5) : retirer de `tools/list` les outils qui
  n'écrivent rien (`ag_vote`, `appel_de_fonds`) et ceux à données codées en
  dur (`energie_campagne_list`, `alertes_list`), **ou** leur faire renvoyer
  une erreur `not_implemented` explicite. La story S3 couvre les deux
  branches ; une seule sera codée.

**Tranché le 2026-11-04 par @gilmry** : brief et stories signés ; ADR 0059
acceptée avec l'option 1, donc la branche A (retrait de `tools/list`).

## 5. Critère de fin du lot

- `security_mcp_refuse_building_hors_perimetre` vert en CI.
- `security_mcp_ne_contredit_pas_majorites_rs` vert en CI.
- Aucun outil MCP ne répond « enregistré » sans écrire.
