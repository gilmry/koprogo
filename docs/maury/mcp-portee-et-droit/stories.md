---
feature: mcp-portee-et-droit
phase: D (Stories)
status: SIGNED v1.0 par @gilmry 2026-11-04
date: "2026-11-03"
authors: [Claude (drafting)]
depends_on: brief.md (SIGNED v1.0), ADR 0059 (Accepted, option 1)
---

# Stories — Lot 0 : portée, rôle, honnêteté et droit du MCP

## Plan d'exécution

| Story | Objet | Déps | Taille |
|---|---|---|---|
| **S1** — Portée de chaque outil qui reçoit un identifiant | sécurité | aucune | M |
| **S2** — `caller_from_role` : admin sans organisation ≠ SuperAdmin | sécurité | aucune | S |
| **S3** — Outils sans écriture ou à données codées en dur | honnêteté | ADR 0059 | S |
| **S4** — Les outils juridiques s'appuient sur `majorites.rs` | droit | aucune | M |

S1, S2 et S4 sont indépendantes. S3 suit la branche A (ADR 0059 acceptée
le 2026-11-04).
Chaque story : tests rouges d'abord, une PR, aucune fusion par un agent.

## Légende AC

- `@happy` chemin nominal ; `@edge` borne ; `@security` refus hors
  périmètre ; `@negative` défaillance correcte et typée.

---

## Story S1 — Chaque outil qui reçoit un identifiant vérifie la portée

### Goal

Tout outil MCP qui reçoit `building_id`, `owner_id` ou `meeting_id`
appelle, avant toute lecture, la garde REST qui protège la même ressource,
et refuse avec la même sémantique.

### Outils concernés (relevé du 03/11 sur `feature/dev`)

`get_building`, `list_owners`, `list_meetings`, `get_financial_summary`,
`list_tickets`, `get_owner_balance`, `list_pending_expenses`,
`check_quorum`, `get_building_documents`, `list_owners_of_building`,
`ag_quorum_check`, `comptabilite_situation`, `appel_de_fonds`,
`alertes_list`.

### Conception

- Une fonction unique `verifier_portee_mcp(user, cible, state)` qui délègue
  aux gardes existantes (`verify_building_org_access`,
  `verify_acp_org_access`, et l'équivalent pour un propriétaire et une
  réunion). Pas de règle propre au MCP.
- Le refus devient une `RpcError` dont le message ne révèle pas l'existence
  de la ressource (même texte pour « hors périmètre » et « introuvable »).

### AC

#### `@happy`

- AC-S1.h1 — Syndic de A, bâtiment de A : chaque outil répond comme
  aujourd'hui.
- AC-S1.h2 — SuperAdmin : réponse quel que soit le bâtiment.

#### `@edge`

- AC-S1.e1 — Bâtiment de A sans aucune donnée (pas de ticket, pas de
  document) : liste vide, pas d'erreur.

#### `@security`

- AC-S1.s1 — `security_mcp_refuse_building_hors_perimetre` : syndic de A,
  bâtiment de B, sur `get_building`, `get_owner_balance`, `list_tickets`
  et `get_building_documents` → erreur, aucune donnée de B dans la
  réponse. Le même test est paramétré sur **tous** les outils listés
  ci-dessus.
- AC-S1.s2 — Le message d'erreur est identique pour un bâtiment de B et
  pour un UUID inexistant.

#### `@negative`

- AC-S1.n1 — Identifiant non UUID → `INVALID_PARAMS`, comme aujourd'hui.

### Fichiers

- `backend/src/infrastructure/web/handlers/mcp_sse_handlers.rs`
- `backend/src/infrastructure/web/middleware/scope_guard.rs` (lecture,
  ajout éventuel d'une garde propriétaire ou réunion si elle manque)
- `backend/tests/e2e_mcp_sse.rs` ou un nouveau `e2e_mcp_portee.rs`

### DoD

Les tests `@security` sont rouges avant le changement et verts après ;
`make ci` vert ; aucun outil de la liste n'appelle un use case avant la
garde.

---

## Story S2 — Un admin sans organisation n'est pas SuperAdmin

### Goal

`caller_from_role("admin", None, _)` ne renvoie plus
`AcpCaller::SuperAdmin`. Il renvoie l'appelant le moins privilégié
(`Owner { user_id }`), comme pour un syndic sans organisation.

### AC

- `@happy` AC-S2.h1 — `admin` avec organisation → `Admin { organization_id }`
  (inchangé).
- `@edge` AC-S2.e1 — rôle en casse mixte (`Admin`) → même résultat.
- `@security` AC-S2.s1 — `admin` sans organisation → pas SuperAdmin ;
  il ne lit aucune ACP qui n'est pas la sienne par ses lots.
- `@negative` AC-S2.n1 — rôle inconnu → `Owner`, comme aujourd'hui.

### Fichiers

- `backend/src/infrastructure/web/middleware/scope_guard.rs` (+ ses tests)

### Point d'attention

Vérifier qu'aucun compte de recette ou de démo ne dépend de ce
comportement : un admin plateforme légitime doit porter le rôle
`superadmin`.

---

## Story S3 — Pas d'outil qui fait semblant (selon ADR 0059)

### Goal

Selon la décision signée dans l'ADR 0059 :

- **Branche A** (retenue le 2026-11-04) : `ag_vote`, `appel_de_fonds`, `energie_campagne_list` et
  `alertes_list` disparaissent de `tools/list` et leur appel renvoie
  `METHOD_NOT_FOUND`.
- Branche B (écartée) : ils restent listés, et renvoient une erreur explicite
  `not_implemented` au lieu d'une confirmation ou de données fictives.

### AC

- `@happy` AC-S3.h1 — Les autres outils restent listés et fonctionnent.
- `@security` AC-S3.s1 — Aucun outil ne renvoie un texte de confirmation
  (« enregistré », « émis ») sans écriture en base vérifiable dans le même
  test.
- `@negative` AC-S3.n1 — Appel d'un outil retiré ou non implémenté →
  erreur JSON-RPC typée, jamais un succès.
- `@edge` AC-S3.e1 — `tools/list` et le prompt système
  (`mcp_system_prompt.md`) ne citent plus ces outils comme disponibles
  (branche A).

---

## Story S4 — Le droit du MCP vient de `majorites.rs`

### Goal

`legal_search`, `majority_calculator` et `travaux_qualifier` ne portent
plus de seuils écrits à la main : ils dérivent la majorité de
`NatureDeDecision::majorite_requise()`.

### Corrections attendues (relevé du 03/11)

- « Travaux non urgents > 5 000 € : majorité absolue de tous les
  copropriétaires » → travaux aux parties communes : **2/3**
  (art. 3.88 § 1er, 1°, b), majorité absolue seulement pour les travaux
  imposés par la loi, conservatoires et d'administration provisoire.
- « Trois devis obligatoires au-delà de 5 000 € » → l'AG fixe, aux 2/3, le
  montant à partir duquel la mise en concurrence est obligatoire
  (art. 3.88 § 1er, 1°, c).
- Modification des statuts : 4/5 en général, 2/3 pour l'usage des communs,
  unanimité pour les quotes-parts.
- `mcp_system_prompt.md` (l. 33-34, 43) : « majorité simple » et « 4/5 pour
  les travaux au-delà de 50 000 € » → mêmes règles que ci-dessus, sans seuil
  en euros.

### AC

- `@happy` AC-S4.h1 — Pour chaque `NatureDeDecision`, `majority_calculator`
  renvoie la majorité de `majorite_requise()`.
- `@security` AC-S4.s1 — `security_mcp_ne_contredit_pas_majorites_rs` :
  le test parcourt toutes les natures et toutes les entrées de
  `legal_search` qui citent une majorité, et compare au domaine.
- `@edge` AC-S4.e1 — Une nature inconnue en entrée → `DroitCommun`
  (majorité absolue), comme le domaine.
- `@negative` AC-S4.n1 — Aucune réponse ne contient « trois devis » ni
  « 5000 » comme seuil légal.

### Fichiers

- `backend/src/infrastructure/web/handlers/mcp_sse_handlers.rs`
- `backend/src/infrastructure/mcp_system_prompt.md`
- `backend/src/domain/copropriete/majorites.rs` (lecture seule)
