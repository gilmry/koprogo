---
feature: transfert-acp-filet
phase: D (Stories)
status: SIGNED v1.0 par @gilmry 2026-11-04
date: "2026-11-03"
authors: [Claude (drafting)]
depends_on: brief.md (SIGNED v1.0) ; option A « rouge attendu strict » (brief § 4)
---

# Stories — Lot 1 : filet de caractérisation du transfert d'ACP

## Plan d'exécution

| Story | Objet | Déps | Taille |
|---|---|---|---|
| **S-1** — Caractérisation du rattachement actuel | filet vert, comportement actuel | aucune | S |
| **S0** — Harnais « rouge attendu » | outillage de test | décision brief § 4 | S |
| **S1** — `acp_transfer.feature` | BDD, 4 catégories | S0 | M |
| **S2** — `e2e_transfert_acp.rs` | e2e API + MCP | S0 | M |

**Règle absolue** : ne jamais modifier un test de caractérisation pour le
faire passer. Un scénario ne change de tag (`@rouge-lot-N` → vert) que dans
la PR du lot qui le fait passer.

## Acteurs des scénarios

ACP « Résidence Les Tilleuls » ; cabinet **A** (syndic sortant) et cabinet
**B** (syndic entrant) ; copropriétaires du lot 3 ; SuperAdmin ; un Admin
d'organisation (pas SuperAdmin).

---

## Story S-1 — Caractérisation du rattachement actuel (ajoutée le 2026-11-11)

Les scénarios S1 et S2 décrivent le comportement **voulu**, rouge d'abord.
Rien ne fige le comportement **actuel** que les lots 2 à 4 vont modifier :
`Acp::set_organization` (`domain/copropriete/acp.rs`), la colonne
`acps.organization_id` et les gardes `verify_*_org_access`. Le rétrofit
Foyer exige ce filet vert avant tout changement de modèle.

**Fichier** : `backend/tests/e2e_caracterisation_rattachement_acp.rs`, cité
dans le bloc e2e de `ci.yml`.

| Cas | Comportement figé |
|---|---|
| C1 | `PUT /acps/{id}` du SuperAdmin avec `organization_id` = B : 200, `acps.organization_id` écrasé, aucune table `syndic_mandates` |
| C2 | Après C1, le syndic de A reçoit 403 sur l'ACP et sur son immeuble, celui de B 200 |
| C3 | Le syndic de A qui déplace l'ACP vers B ou la détache (`null`) : 403, rattachement inchangé |
| C4 | Le SuperAdmin qui envoie `organization_id: null` : 200, rattachement **inchangé** ; une chaîne vide `""` détache |
| C5 | ACP détachée et son immeuble : 403 pour le syndic de A, 200 pour le SuperAdmin |
| C6 | Rattachement à un cabinet inexistant : 400, rattachement inchangé |
| C7 | `DELETE FROM organizations` : l'ACP survit, détachée, son immeuble aussi |

**Règle** : verts sur le code du 2026-11-11, jamais modifiés pour passer.
Le lot qui change délibérément un comportement retire ou réécrit le test
concerné dans sa propre PR, et le dit.

### Registre S-1 : bug ou voulu ?

Figés tels quels par les tests ; la décision revient au PO.

- **R1** — Le code prévoit un Admin de cabinet (`AcpCaller::Admin`) qui
  pourrait détacher l'ACP sans pouvoir la déplacer. Ce rôle est inatteignable :
  `UserRole` refuse `admin`, donc seul le SuperAdmin modifie une ACP (C3). Code
  mort à retirer, ou rôle à créer ? Une ACP détachée devient illisible pour son
  ancien cabinet (C5).
- **R2** — Supprimer un cabinet détache ses ACP (C7). Ce que la suppression
  emporte dans les autres tables qui portent encore un `organization_id` en
  cascade n'est pas caractérisé : à mesurer avant le lot 2 (ADR 0058).
- **R3** — Le transfert ne laisse aucune trace de l'ancien cabinet (C1) ;
  l'accès bascule sur tout l'historique à l'instant (C2). C'est l'écart que
  le lot 2 comble ; noté pour mémoire, pas une question.
- **R4** — `UpdateAcpDto` documente « `null` = détache », mais `null` est
  ignoré : serde lit `null` comme une clé absente sur un `Option<Option<_>>`
  sans `deserialize_with`. La réponse est 200, sans rien changer. Seule une
  chaîne vide détache (C4).

### DoD

Le harnais est vert en local (`KCARGO_DOCKER=1 kcargo test --test
e2e_caracterisation_rattachement_acp`) et cité dans `ci.yml` ; R1 et R2
sont posés au PO.

---

## Story S1 — `acp_transfer.feature`

Chaque scénario : catégorie, lot qui le fera passer.

#### `@happy`

- **H1** (lots 2, 3) — Transfert de A vers B à la date d'effet D : le
  mandat de A est clos à D, celui de B est actif ; budgets, écritures,
  appels de fonds, documents et copropriétaires sont lisibles par B.
- **H2** (lot 3) — Après le transfert, les copropriétaires gardent l'accès
  à leur lot et à leurs appels de fonds.

#### `@security`

- **S1** (lots 3, 4) — L'ancien syndic A reçoit 403 ou 404 sur toutes les
  ressources de l'ACP ; ses listes (tickets, paiements, copropriétaires,
  énergie, tableau de bord) ne contiennent plus aucun identifiant de l'ACP.
- **S2** (lot 3 + lot 0) — Le MCP refuse lui aussi à A tout outil portant
  sur l'ACP.
- **S3** (lot 2, ADR 0058) — Supprimer l'organisation A ne détruit aucune
  donnée de l'ACP.
- **S4** (lot 4) — Le communautaire (SEL, campagnes énergie, défis) est
  visible de B et invisible de A.

#### `@edge`

- **E1** (lot 2) — À la date exacte D, `holder_at(D)` vaut B ; un état daté
  à D-1 cite A.
- **E2** (lot 3) — Une ACP auto-gérée (sans mandat) reste accessible à ses
  copropriétaires.

#### `@negative`

- **N1** (lot 2) — Un mandat de plus de 3 ans est refusé
  (`DUREE_MAXIMALE_JOURS`).
- **N2** (lot 2) — Un Admin non SuperAdmin ne peut pas transférer une ACP.

### DoD

Le fichier existe, chaque scénario est exécuté en CI, chaque scénario rouge
porte son tag de lot, et le harnais S0 fait échouer la CI si un scénario
taggé rouge passe.

---

## Story S2 — `e2e_transfert_acp.rs`

Mêmes scénarios que S1, au niveau HTTP (testcontainers Postgres), plus les
chemins que le BDD ne couvre pas :

- tableau de bord de A : aucun compteur ne compte l'ACP après D ;
- MCP : `get_building`, `list_tickets`, `get_owner_balance`,
  `get_building_documents` appelés par A sur un bâtiment de l'ACP → refus
  (réutilise le test de portée du lot 0).

Fichiers : `backend/tests/e2e_transfert_acp.rs`, helpers existants de
`backend/tests/common/`.

---

## Story S0 — Harnais « rouge attendu »

Option A retenue au brief § 4 (tranché le 2026-11-04) :

- runner BDD : un scénario taggé `@rouge-lot-N` est inversé (échec attendu)
  et la CI échoue s'il passe ;
- e2e : une macro ou un attribut équivalent, même sémantique ;
- un test du harnais lui-même (`@negative` : un scénario rouge qui passe fait
  échouer le job).
