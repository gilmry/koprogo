---
feature: transfert-acp-filet
phase: Brief court (lot 1 de la v0.1.0, filet de caractérisation)
status: DRAFT, à signer par @gilmry
date: "2026-11-03"
authors: [Claude (drafting)]
related_issues: [1046]
related_adr: [0053, 0055, 0058]
parent_maury: lettre de mission v0.1.0 « l'ACP au centre », lot 1
---

# Brief — Le transfert d'une ACP d'un syndic à un autre, écrit en tests avant d'être codé

## 1. Vision

La v0.1.0 a trois critères : **transfert sans perte**, **ancien syndic
aveugle sur tous les chemins (MCP compris)**, **communautaire qui suit
l'ACP**. Aucun n'est codé. Le lot 1 les écrit d'abord en tests, rouges,
avant de toucher au modèle (lots 2 à 4). Ces tests sont le contrat : on ne
les modifie jamais pour les faire passer.

## 2. État du code (relevé du 03/11 sur `feature/dev`)

- Le domaine connaît déjà le mandat : `SyndicMandate`, `holder_at(date)`,
  `DUREE_MAXIMALE_JOURS = 3 * 365` (`domain/copropriete/syndic_mandate.rs`)
  et `PassationDeDossier` (`domain/services/dossier_de_gestion.rs`).
- Il n'existe ni table `syndic_mandates`, ni use case de transfert, ni
  endpoint : l'ACP est rattachée par `acps.organization_id`, écrasée par
  `Acp::set_organization` (`domain/copropriete/acp.rs:279`).
- L'accès est contrôlé par organisation (`verify_*_org_access`), pas par
  mandat actif (ADR 0053, non implémentée).

## 3. Livrables

- `backend/tests/features/acp_transfer.feature` (BDD).
- `backend/tests/e2e_transfert_acp.rs` (e2e, testcontainers).
- Chaque scénario porte le lot qui le fera passer, et la CI le lance.

## 4. Décision attendue : comment un test rouge vit en CI

La lettre demande des tests « lancés en CI » et « verts ou rattachés au lot
qui les fera passer ». Trois options :

| Option | Effet | Pour | Contre |
|---|---|---|---|
| **A. Rouge attendu strict** (recommandée) | le scénario tourne ; la CI échoue s'il passe **ou** s'il échoue autrement que prévu | un scénario qui verdit sans qu'on le sache est détecté ; le contrat reste visible | un petit harnais à écrire (tag `@rouge-lot-N` lu par le runner BDD, attribut équivalent côté e2e) |
| B. `#[ignore = "lot N"]` | le test ne tourne pas | aucun outillage | ne répond pas à « lancés en CI » |
| C. Job séparé non bloquant | le job tourne, rouge toléré | simple | c'est exactement le défaut de #1048 : un rouge que personne ne lit |

## 5. Hors périmètre

Le modèle lui-même (lots 2 à 4) et le RLS (lot 5).
