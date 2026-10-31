# ADR 0059 : un outil MCP qui n'écrit rien n'est pas exposé

- **Status**: Proposed (à trancher par @gilmry)
- **Date**: 2026-10-31
- **Track**: Software / Security
- **Authors**: @gilmry (décision) + Claude (rédaction)
- **Related**: #1046, lot 0 de la lettre de mission du 2026-10-31

## Contexte

Dans `mcp_sse_handlers.rs`, `ag_vote` répond `vote_recorded` sans rien enregistrer et `appel_de_fonds` ne crée rien. `energie_campagne_list` et `alertes_list` renvoient des données codées en dur. Un agent qui les appelle croit avoir agi. Un vote en AG est un acte juridique : le simuler est pire que ne pas l'offrir.

## Options

1. **Retirer ces outils de `tools/list` jusqu'à leur branchement.** L'agent ne les voit pas ; le code reste derrière un drapeau pour le jour du branchement.
2. **Les garder et renvoyer une erreur `not_implemented` explicite.** L'agent les voit, les appelle, et reçoit un refus clair. Plus bruyant, mais la liste des outils reste stable.
3. **Les brancher tout de suite.** Le vote exige l'identité du votant, les quotes-parts et la majorité applicable (`majorites.rs`) ; l'appel de fonds exige le budget et la répartition. Ce n'est pas une tâche de sprint S-B.

## Recommandation

**Option 1 pour les quatre outils.** Ce qui n'existe pas ne s'annonce pas. Le test `security_mcp_ne_contredit_pas_majorites_rs` du lot 0 couvre le reste : tout outil qui parle de droit s'appuie sur le domaine. Quand un outil est branché, il revient dans la liste avec ses tests dans les quatre catégories.

## Réversibilité

Totale : une entrée dans une liste.
