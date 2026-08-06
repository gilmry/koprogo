===============================================================================================
Issue #635: Fonds affectés / thésaurisation : entité Fund dédiée aux travaux d'ampleur (v0.2.0)
===============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement
:Assignees: Unassigned
:Created: 2026-06-27
:Updated: 2026-06-27
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/635>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   H13a a livré le **fonds de réserve** légal sur l'ACP (≥ 5 % charges N-1, renonçable 4/5, Art. 3.86 §3). Mais le modèle binaire roulement/réserve ne suffit pas : en pratique une copropriété **thésaurise** en créant des **fonds idoines dédiés à un travail d'ampleur précis** (réfection toiture, façade, ascenseur…), avec épargne pluriannuelle fléchée. (Décision PO @gilmry 2026-06-25 : différer ce modèle riche hors H13a.)
   
   ## Recherche (bases)
   
   Cadre belge (Art. 3.86 §3 C. civ., loi 18/06/2018) :
   - **Fonds de roulement** : dépenses périodiques (chauffage, ascenseur usage, éclairage communs, gérance). Compte distinct.
   - **Fonds de réserve** : dépenses NON périodiques / gros travaux ; ≥ 5 % charges ordinaires N-1, obligatoire (≤ 5 ans) ; affecté **exclusivement** aux travaux. Compte distinct.
   - **Fonds de travaux prévisionnel / affecté (thésaurisation)** : facultatif, voté en AG ; épargne progressive vers un chantier précis. Gros travaux = majorité **2/3** (Art. 3.88).
   
   Sources : la-copropriete.be (fonds réserve/roulement) · easysyndic.be (travaux & fonds de travaux prévisionnel) · copropriete-ejuris.be · Code civil Livre 3 (IPI).
   
   ## Modèle cible proposé (à acter en ADR)
   
   Entité **`Fund`** (fonds) rattachée à l'ACP :
   - `id`, `acp_id`, `kind` { `working_capital` | `reserve` | `earmarked` (affecté) }
   - `name`, `purpose` / `work_ref` (travail ciblé), `target_amount` (objectif d'épargne), `balance` (solde), comptes distincts.
   - `call_for_funds.fund_id` → rattache un appel de fonds au fonds qu'il alimente (remplace l'enum binaire envisagé en H13b, reverté).
   - Suivi de progression de la thésaurisation (balance / target), reliquat en fin de projet.
   
   ## ⚠️ Contrainte clé (PO @gilmry)
   
   **Une AG peut décider d'imputer un fonds thésaurisé à une autre fin que celle prévue.** L'affectation d'un fonds n'est donc **PAS un verrou immuable** : c'est une **intention par défaut, surchargeable par décision d'AG** (vote). Le modèle doit prévoir :
   - une opération de **réaffectation** d'un fonds (montant ou totalité) vers une autre fin, déclenchée par une **décision d'AG** (lien vers la résolution/meeting) ;
   - un **audit trail** de l'affectation initiale → réaffectation (traçabilité comptable + légale) ;
   - ne jamais bloquer une dépense au seul motif que le fonds était « fléché » ailleurs, dès lors qu'une décision d'AG l'autorise.
   
   ## Critères d'acceptation (esquisse)
   - [ ] ADR « fonds affectés & thésaurisation » (modèle Fund + réaffectation AG).
   - [ ] Entité `Fund` + migration + repo + use-cases (créer/alimenter/réaffecter).
   - [ ] `call_for_funds.fund_id` (rattachement appel ↔ fonds).
   - [ ] Réaffectation par décision d'AG + audit trail.
   - [ ] Tests 4-cat (épargne vers objectif, réaffectation AG, reliquat, garde comptes distincts).
   
   Cible : **v0.2.0**. Refs: #618 (Track H), H13a (commit 58f1731).

.. raw:: html

   </div>

