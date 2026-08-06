=========================================================================================================================
Issue #661: Quorum AG (Art. 3.87 §5) calculé en f64 sur seuil légal — AgSession vs Meeting/H9 à unifier [bloque ADR-0008]
=========================================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: None
:Assignees: Unassigned
:Created: 2026-07-25
:Updated: 2026-07-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/661>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   > **Bloque la signature de l'amendement ADR-0008** (2026-05-19, WP-A7), dont le §B affirme un invariant que le code viole. Cf. §« Lien ADR » en bas.
   
   ## Constat
   
   Le **quorum d'assemblée générale (Art. 3.87 §5 CC)** est calculé **en `f64`** dans le chemin `AgSession`, et le **seuil légal est comparé en `f64`** — sur un endpoint **exposé**.
   
   [`domain/entities/ag_session.rs:236-244`](backend/src/domain/entities/ag_session.rs#L236-L244) :
   
   ```rust
   pub fn calculate_combined_quorum(
       &self,
       physical_quotas: f64,          // ← quotités
       total_building_quotas: f64,    // ← quotités
   ) -> Result<f64, String> {
       let combined = physical_quotas + self.remote_voting_power;  // f64
       Ok((combined / total_building_quotas) * 100.0)              // f64
   }
   ```
   
   Champs de l'entité concernés : `remote_voting_power: f64` (*« Millièmes représentés par les distanciels »*), `quorum_remote_contribution: f64`.
   
   Comparaison du seuil — [`application/use_cases/ag_session_use_cases.rs:238`](backend/src/application/use_cases/ag_session_use_cases.rs#L238) :
   
   ```rust
   quorum_reached: combined_pct > 50.0     // seuil légal évalué en f64
   ```
   
   **Exposé publiquement** : `GET /ag-sessions/{id}/quorum` ([`routes.rs:647`](backend/src/infrastructure/web/routes.rs#L647) → `get_combined_quorum`). Ce n'est donc **pas du code mort**.
   
   DTO associés également en `f64` : `ag_session_dto.rs` — `voting_power`, `total_building_quotas`, `physical_quotas`, `remote_quotas`, `combined_percentage`.
   
   ## Cause
   
   **Deux implémentations parallèles du même article de loi**, l'une conforme et l'autre non :
   
   | Chemin | Type | Règle implémentée | Statut |
   |---|---|---|---|
   | `Meeting` (Track H, Story H9) | `Decimal` | quorum **double** (têtes > 50% **ET** quotités ≥ 50%, ou > 3/4) — Art. 3.87 §5 | ✅ conforme |
   | `AgSession` (hybride/distanciel) | **`f64`** | quotités seules, `> 50.0` | 🔴 non conforme |
   
   `AgSession` est antérieur à Track H et n'a pas été repris lors de H9. Personne ne l'a vu : l'enforcement prévu par ADR-0008 §A est *« un `code-reviewer` rejette tout nouveau `f64` »* — donc **humain**, jamais outillé. Une liste de carve-outs fermée mais non testée dérive.
   
   *(Même motif structurel que #659 : la règle existe, le garde-fou automatique n'existe pas.)*
   
   ## Impact
   
   - Violation de **ADR-0007** (Decimal pour montants/quotités), de la décision principale **ADR-0008**, et de **ADR-0011** (quorum double, accepté le 2026-07-25).
   - `f64` sur un **seuil juridique** : une AG déclarée en quorum (ou non) sur une comparaison IEEE-754 est attaquable. Un quorum « pile à 50% » est précisément le cas où l'arrondi binaire décide.
   - Le quorum distanciel ignore par ailleurs le volet **têtes** exigé par l'Art. 3.87 §5 (seules les quotités sont testées).
   
   ## Recette proposée
   
   1. **Unifier plutôt que porter.** Ne pas se contenter de convertir `AgSession` en `Decimal` : faire converger le calcul vers l'implémentation `Meeting` (H9) qui porte déjà le quorum double conforme, `AgSession` n'apportant que la **contribution distancielle**. Éviter durablement deux sources de vérité pour Art. 3.87 §5.
   2. **Supprimer `validate_proxy_mandate`** ([`vote.rs:312`](backend/src/domain/entities/vote.rs#L312)) : code mort (référencé uniquement par ses propres tests), en `f64`, et juridiquement faux (Art. 3.87 §7 traité en AND). Le gate réel est `validate_proxy_limit` (Decimal, `resolution_use_cases.rs:326`).
   3. **Trancher `BudgetVarianceResponse`** ([`budget_dto.rs:85-95`](backend/src/application/dto/budget_dto.rs#L85-L95)) : l'entité `Budget` est `Decimal` (Story H11) mais le DTO de variance repasse les montants en `f64` → soit `Decimal`, soit carve-out « DTO d'affichage » explicite.
   4. **Outiller l'invariant** : transformer le §B d'ADR-0008 (« aucun seuil légal ne transite par `f64` ») en **gate CI** (lint/grep sur `f64` monétaire/quotité hors liste fermée, ou test d'architecture). Sans cela la liste redérivera.
   5. **4-cat RED-first** sur le quorum unifié, dont `@edge` **quorum exactement à 50%** (le cas où `f64` et `Decimal` divergent).
   
   ## Critères de sortie
   
   - [ ] Aucun `f64` sur quotité/montant dans le chemin quorum ; seuil comparé en `Decimal`.
   - [ ] Une seule implémentation d'Art. 3.87 §5 (quorum double, têtes **ET** quotités), distanciel inclus.
   - [ ] `validate_proxy_mandate` supprimé.
   - [ ] `BudgetVarianceResponse` tranché et documenté.
   - [ ] Gate CI qui échoue si un nouveau `f64` monétaire/quotité apparaît hors liste.
   - [ ] Test `@edge` quorum à exactement 50% vert.
   
   ## Lien ADR
   
   L'**amendement ADR-0008 du 2026-05-19** reste `Proposed` **tant que ce ticket n'est pas résolu** : son §B affirme *« Pour chaque chemin légal (quorum Art. 3.87…) : assertion qu'aucune valeur de seuil ne transite par f64 »*, ce qui est **factuellement faux** aujourd'hui. Le signer fermerait la liste des carve-outs tout en laissant subsister une violation sur un seuil légal.
   
   Recommandation complémentaire pour cet amendement : **découpler** ses §C (positivité `expenses.amount`) et §D (rotation de clé API) — sans rapport avec la politique `NUMERIC vs DOUBLE PRECISION`, et déjà acquis factuellement.
   
   ## Contexte
   
   Découvert le 2026-07-25 en auditant les carve-outs d'ADR-0008 contre le code courant, à l'occasion de l'acceptation des ADR-0010/0011/0012 (Track H #618).

.. raw:: html

   </div>

