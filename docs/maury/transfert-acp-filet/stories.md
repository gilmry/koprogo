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
