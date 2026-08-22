=======================================================================================================
Issue #618: Conformité droit belge — refonte modèle copropriété (acte de base ACP hybride + full sweep)
=======================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,track:software priority:high,legal-compliance governance
:Assignees: Unassigned
:Created: 2026-06-15
:Updated: 2026-07-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/618>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Plan — Mise en conformité droit belge du modèle copropriété (modèle hybride ACP + full sweep)
   
   ## Context
   
   Track H (validate-before-compute) a révélé que `total_tantiemes` / l'acte de base est posé au niveau **building** alors que, juridiquement, il relève de la **copropriété (ACP)**. @gilmry a demandé une **revue complète du modèle domain sur bases légales** (Code civil belge Livre 3, Art. 3.84-3.88 + loi 18/06/2018). La revue (3 audits + vérifs web sourcées) a trouvé 10 divergences/absences au-delà de `total_tantiemes`. Décisions PO : **(D1) modèle hybride** — dénominateur acte de base sur l'**ACP**, sous-totaux de quotités par **building (bloc / association partielle)** ; **(D2) full conformité maintenant**. WIP Track H stashé (`git stash@{0}`, HEAD `3ede509`).
   
   ## Bases légales (sourcées)
   - **Art. 3.84** — copropriété = immeuble OU groupe d'immeubles ; acte de base = statuts de l'ensemble ; fixe la quote-part des communs par lot (valeur respective).
   - **Art. 3.86** — ACP = personne morale de la copropriété (pour un groupe : seul le groupe l'a) ; **fonds de réserve (≥5% charges ordinaires N-1) + fonds de roulement obligatoires** (loi 2019), comptes distincts au nom de l'ACP, réserve renonçable à 4/5.
   - **Quotités** — base 1000/10000 (convention acte de base) ; somme = dénominateur ; charges (valeur/utilité/mixte) + votes.
   - **Art. 3.87 §3** — convocation **15 j toutes AG**, sauf urgence (pas de 8 j).
   - **Art. 3.87 §5** — **DOUBLE quorum** : > moitié des copropriétaires (têtes) ET ≥ moitié des quotités.
   - **Art. 3.87 §1** — un lot peut appartenir à **plusieurs titulaires** (indivision) ou être **démembré** (usufruit/nue-propriété, emphytéose, superficie). Dans ce cas le **droit de vote est SUSPENDU jusqu'à désignation d'un représentant unique** (mandataire commun) qui exercera le vote. Pas de vote commun automatique ; à défaut d'accord usufruitier/nu-propriétaire, le tribunal peut trancher (p. ex. 50/50).
   - **Art. 3.87 §7** — procurations max 3, ou ≤ 10% des voix.
   - **Art. 3.88** — majorités absolue / 2-3 / 4-5 / unanimité.
   
   Sources : [Code civil ejustice](https://www.ejustice.just.fgov.be/img_l/pdf/2020/02/04/2020020347_F.pdf) · [quotités](https://www.choisirunsyndic.be/dossiers/quotites-en-copropriete/) · [personnalité juridique](https://copropriete-ejuris.be/personnalite-juridique/) · [convocation/AG](https://www.droitbelge.be/fiches_detail.asp?idcat=9&id=623) · [fonds réserve](https://vjn-legal.be/copropriete-fonds-de-reserve-et-fonds-de-roulement-desormais-obligatoires/) · [procurations](https://www.choisirunsyndic.be/question/procurations-ag-limite-a-3-mandats-ou-10/)
   
   ## Modèle cible (hybride)
   - **`acps.total_tantiemes`** (nouvelle colonne, acte de base global) = source de vérité du dénominateur. **`buildings.total_tantiemes` conservé mais redéfini** = sous-total quotités du bloc (associations partielles Art. 3.86). Pas de `units.acp_id` (lien via `unit → building → acp`).
   - **Conformité 2 niveaux** : ACP-level (légal, agrégat `SUM(units de tous buildings)==acps.total_tantiemes`) + building-level (H1 conservé, sous-total bloc).
   - **Fonds** : `acps.reserve_fund_balance`, `working_capital_balance`, `reserve_fund_waived` ; `call_for_funds.fund_type`.
   - **Charges** : `distribution_criteria {value|utility|mixed}`.
   - **Multi-titulaires & vote (Art. 3.87 §1)** : un lot a déjà N propriétaires (`unit_owners` M:N existant). Ajout : **type de titularité** (pleine propriété / usufruit / nue-propriété / indivision / emphytéose / superficie) + **représentant de vote désigné** par lot ; **droit de vote suspendu** si lot démembré/indivis sans représentant.
   
   ## Méthode Maury (BMAD) — livrables AVANT code
   
   Refonte structurante → pipeline Maury complet (mémoires `maury-fullstack-first`, `maury-token-economy`, `tdd-bdd-four-categories`). Nouveau dossier **`docs/maury/refonte-ux-multi-role-acp/track-h-conformite-legale/`** (template du dossier sœur signé `track-h-bloqueurs/`), gate de signature @gilmry par doc avant impl :
   - `README.md` (index BMAD + signatures + mapping WBS)
   - `brief.md` (Phase A) — personas (Admin garant acte de base, Syndic, Copropriétaire), capacités légales CB-L1..n, invariants INV-L (acte ACP, conformité 2 niveaux, quorum double, fonds réserve, associations partielles), SCB, **sources légales Art. 3.84-3.88 + loi 2019** (cf. liens ci-dessus)
   - `prd.md` (Phase B) — FR par finding (1-11 + H15/H16), user journeys (groupe d'immeubles, association partielle, AG quorum double), AC 4-cat
   - `architecture.md` (Phase C) — modèle hybride (acps.total_tantiemes + sous-totaux blocs), associations partielles (quotités 2 niveaux), quorum double, migrations réversibles, **mermaid composants + data flow**
   - `stories.md` (Phase D) — H4-H16 self-contained briefables + **Gantt RGRR**
   - `validation.md` (Phase F) — acceptation PO @gilmry
   
   ## Intégration WBS v0.1.0 (`docs/WBS_GO_LIVE_v0.1.0.md`)
   
   Track H enrichi — nouveaux WP mappés aux stories, **statut du déjà-fait pris en compte** :
   - H1 ✅ **FAIT** (commit `6a053a1`) — conservé (sous-total bloc).
   - H2 ✅ mergé (`3ede509`) — **retravaillé par H7** (gate building→ACP).
   - H3 ✅ mergé (`3ede509`) — **étendu par H9** (quorum simple→double + têtes).
   - Nouveaux WP : WP-H7 conformité ACP hybride (H4-H7), WP-H8 gouvernance AG (quorum double + gates votes + représentant/suspension : H9-H10-H17), WP-H9 fonds réserve/roulement + critère utilité (H12-H13), WP-H10 associations partielles (H16), WP-H11 units acp_id (H15), WP-H12 bugs conformité (H8/H11/H14).
   - Régénérer les **2 mermaid** (graph LR deps + Gantt par passe d'agent RGRR) ci-dessous.
   
   ### Mermaid — graph de dépendances (✅ = fait)
   ```mermaid
   graph LR
       H1[H1 BuildingNotConformant<br/>✅ 6a053a1]:::done
       H2[H2 gates building<br/>✅ 3ede509]:::done
       H3[H3 quorum simple<br/>✅ 3ede509]:::done
       ADR[H0 ADR hybride]
       H4[H4 acps.total_tantiemes] --> H5[H5 Acp.assert_conformant] --> H6[H6 AcpRepo metrics] --> H7[H7 gates building→ACP]:::crit
       ADR --> H4
       H5 --> H8[H8 Unit MAX_QUOTA]
       ADR --> H9[H9 quorum double]:::crit --> H10[H10 gates votes] --> H17[H17 représentant vote/suspension]
       ADR --> H11[H11 budget Decimal]
       ADR --> H14[H14 doc convocation]
       H4 --> H15[H15 units acp_id]
       H7 --> H12[H12 DistributionCriteria] --> H16[H16 assoc partielles]:::crit
       H4 --> H13[H13 fonds réserve]
       H1 --> ADR
       H2 -. retravaillé .-> H7
       H3 -. étendu .-> H9
       classDef done fill:#9f9,stroke:#070
       classDef crit fill:#f99,stroke:#900
   ```
   
   ### Mermaid — Gantt par passe d'agent (1 passe = RED-GREEN-REFACTOR-REVIEW ; S=0.5j M=1j L=2j wall-clock)
   ```mermaid
   gantt
       title Track H Conformité légale — Gantt par passe d'agent (RGRR)
       dateFormat YYYY-MM-DD
       axisFormat %d-%m
       section Déjà fait (mergé)
       H1 BuildingNotConformant 6a053a1   :done, h1, 2026-06-15, 1d
       H2 validate-before-compute 3ede509 :done, h2, 2026-06-15, 1d
       H3 Meeting.assert_can_complete     :done, h3, 2026-06-15, 1d
       section Socle conformité ACP (critique)
       H0-ADR acte de base & conformité   :adr, after h3, 1d
       H4 migration acps.total_tantiemes  :h4, after adr, 1d
       H5 Acp.assert_conformant + error   :h5, after h4, 1d
       H6 AcpRepository metrics           :h6, after h5, 1d
       H7 bascule 4 gates building→ACP    :crit, h7, after h6, 2d
       section Bugs conformité (parallèle)
       H8 Unit MAX_QUOTA base 10000       :h8, after h5, 1d
       H11 Budget f64→Decimal            :h11, after adr, 1d
       H14 doc CONVOCATIONS 15j          :h14, after adr, 1d
       H15 units org_id→acp_id           :h15, after h4, 2d
       section Gouvernance AG (parallèle)
       H9 quorum double tête+quotité     :crit, h9, after adr, 2d
       H10 gates votes quorum+proxy      :h10, after h9, 1d
       H17 représentant vote/suspension  :h17, after h10, 2d
       section Charges & fonds
       H12 DistributionCriteria          :h12, after h7, 1d
       H13 fonds réserve/roulement       :h13, after h4, 2d
       section Associations partielles
       H16 partial associations          :crit, h16, after h12, 2d
   ```
   **Chemin critique** : ADR(1) → H4(1) → H5(1) → H6(0.5) → H7(2) → H12(1) → H16(2) ≈ **~8,5 j wall-clock** (compressible avec parallélisme 1 BE + 1 FE, cf. `docker-parallelism-bottleneck`).
   
   ## Découpage en stories (1 story = 1 PR, TDD 4-cat RED-first) — matérialisé dans `stories.md` du dossier Maury
   
   | Story | Objet | Dép. | Note WIP stash |
   |---|---|---|---|
   | **H4** | Migration `acps.total_tantiemes` + backfill (mono: =building ; multi: SUM + `RAISE WARNING`+audit) + `.down` réversible | — | — |
   | **H5** | `Acp::is_conformant/assert_conformant` + `AcpMetrics` + `AcpNotConformantError` + `From<>` AppError(422 `ACP_NOT_CONFORMANT`)/String (pattern `error.rs:486-511`) | H4 | — |
   | **H6** | Port + adapter `AcpRepository::find_by_id_with_metrics` (JOIN units multi-building) | H5 | — |
   | **H7** | **Bascule des 4 gates building→ACP** (expense/call_for_funds/charge_distribution/etat_date) + wiring `main.rs` | H6 | adapte fixes seeds (ACP-level) |
   | **H8** | Retrait `Unit::MAX_QUOTA=dec!(1000)` → borne agrégée (`unit.rs:7,54,81`) | H5 | — |
   | **H9** | **Quorum double** (têtes+quotités) : `validate_quorum` signature, colonnes `meetings`, `MeetingCompletionChecklist`+têtes, `MissingInvariant::HeadCountQuorumNotReached` | — | adapte match `MissingInvariant` |
   | **H10** | Brancher gates votes : `check_quorum_for_voting` + `validate_proxy_mandate` (3/10%) dans use-cases vote | H9 | — |
   | **H11** | `Budget` f64→Decimal (entity/DTO/repo ; DB déjà Decimal) — ADR-0007 | — | — |
   | **H12** | `DistributionCriteria` enum + param + clarif `quota_percentage` (lot) vs `ownership_percentage` (copropriétaire) | H7 | — |
   | **H13** | Fonds réserve(5%)/roulement : colonnes ACP + `call_for_funds.fund_type` + `assert_reserve_fund_compliant` | H4 | — |
   | **H14** | Doc `CONVOCATIONS_AG.rst` → 15 j toutes AG (Art. 3.87 §3), urgence sans seuil | — | — |
   | **H15** | **Migration `units.organization_id` → `acp_id`** (3 étapes : add nullable → backfill depuis `building.acp_id` → NOT NULL + drop `organization_id`) + maj entité `Unit`/DTO/use-cases/handlers (cohérence post-#602, comme buildings) | — | — |
   | **H16** | **Associations partielles + personnalité juridique propre (Art. 3.86)** — voir conception ci-dessous | H4,H5,H7 | — |
   | **H17** | **Multi-titulaires & représentant de vote (Art. 3.87 §1)** — type titularité + représentant désigné + **suspension droit de vote** si démembré/indivis sans désignation — voir conception ci-dessous | H9,H10 | — |
   | **H0-ADR** | ADR(s) `docs/adr/` : acte de base & conformité copropriété (hybride + associations partielles) ; quorum double ; fonds réserve ; représentant de vote/suspension | — | — |
   
   **Ordre** : H0-ADR + H4 → H5 → H6 → H7 (chemin critique) ; H8/H11/H14/**H15** en // ; H9 → H10 → **H17** (gouvernance, // du chemin conformité) ; H12/H13 puis **H16** (le plus profond, après le socle conformité ACP).
   
   ## H16 — Associations partielles (conception)
   
   Art. 3.86 : dans un groupe d'immeubles, l'ACP principale a la personnalité juridique ; une **association partielle** (par bloc/sous-groupe) peut avoir sa **propre personnalité juridique** (seulement si l'ACP principale l'a) OU être créée **sans** personnalité par vote AG **4/5**. Elle gère les **parties communes particulières** (ex. ascenseur/toiture d'un bloc) avec ses propres quotités, AG et charges.
   
   MVP cible :
   - Entité/table **`partial_associations`** : `id`, `acp_id` (parent), `name`, `has_legal_personality bool`, `bce_number` (nullable, si personnalité), `total_tantiemes` (sous-dénominateur des parties communes particulières), timestamps.
   - Rattachement **building → 0..1 partial_association** (`buildings.partial_association_id` nullable) — un bloc appartient à 0 ou 1 association partielle.
   - **Quotités à deux niveaux par lot** (subtilité légale à acter dans l'ADR) : un lot a une quote-part dans les **communs généraux** (dénominateur ACP) ET, s'il dépend d'une association partielle, une quote-part dans les **communs particuliers** (dénominateur PA). → décision de modèle : ajouter `units.particular_quota Decimal` (nullable) OU table `unit_quotas(unit_id, scope, quota)`. Trancher dans H0-ADR.
   - **Scope AG/charges** : `meeting` et `charge_distribution` peuvent cibler une `partial_association` (sous-charges/sous-AG du bloc) au lieu de l'ACP entière. `assert_conformant` PA-level : SUM(particular_quota des units des buildings de la PA) == PA.total_tantiemes.
   - Personnalité juridique : `has_legal_personality=true` interdit si l'ACP parent n'en a pas ; création sans personnalité = trace décision AG 4/5.
   
   ## H17 — Multi-titulaires & représentant de vote (conception)
   
   Art. 3.87 §1 : un lot peut avoir plusieurs titulaires (indivision) ou être démembré (usufruit/nue-propriété, emphytéose, superficie). Le **droit de vote est suspendu** tant que les intéressés n'ont pas **désigné un représentant unique**. Pas de vote commun automatique ; en cas de désaccord usufruitier/nu-propriétaire, le juge peut trancher.
   
   MVP cible :
   - `unit_owners` (M:N existant, `ownership_percentage`, `is_primary_contact`) étendu : **`ownership_type`** enum (`full_owner` | `usufruct` | `bare_owner` | `indivisaire` | `emphyteote` | `superficiaire`) + **`is_voting_representative bool`** (le représentant désigné du lot pour le vote).
   - **`Unit::voting_right_status()`** (domaine pur) → `Active` si lot mono-propriétaire pleine propriété OU un `is_voting_representative` désigné ; `Suspended` si démembré/indivis SANS représentant. Erreur typée `VotingRightSuspended { unit_id }`.
   - **Gate vote (H10)** : enregistrement d'un vote pour un lot `Suspended` → rejet 422 `VOTING_RIGHT_SUSPENDED`.
   - **Quorum (H9)** : un lot `Suspended` ne compte pas comme présent/représenté (ni en têtes ni en quotités). Ajuster le double quorum en conséquence.
   - `is_primary_contact` (existant) reste le contact administratif/facturation — **distinct** du représentant de vote légal.
   
   ## Track H déjà mergé — impact
   - **H1 (`building.rs`) conservé** (sous-total bloc).
   - **H2 retravaillé en H7** : les helpers `assert_building_conformant(building_id)` → `assert_acp_conformant(acp_id)` (résoudre `building.acp_id` puis `acp_repo.find_by_id_with_metrics`). Signatures use-cases inchangées (bridge `?`→String/AppError).
   - **H3 (`Meeting::assert_can_complete`) étendu en H9** (ajout critère têtes).
   
   ## Gestion du stash `git stash@{0}`
   Contient fixes BDD/E2E building-level (filler units, clippy `matches!`, helper e2e seed conforme) + correction `validate_quorum` Decimal déjà appliquée. **Un-stash AVANT H7/H9 puis adapter** (filler units → conformité ACP-level ; match `MissingInvariant` → nouveau variant têtes). Ne pas `drop`. Le bug **400-vs-422** du spec H2 (`validate-before-compute.spec.ts`) sera repris dans H7 (gate ACP).
   
   ## Verification
   - **Gate Maury** : les 5 docs BMAD (`track-h-conformite-legale/`) + ADR(s) signés @gilmry AVANT impl (Tier 1 doc publique) ; WBS v0.1.0 + 2 mermaid mis à jour et validés.
   - `CONVOCATIONS_AG.rst` corrigé relu @gilmry.
   - TDD 4-cat RED-first par story (notamment : conformité ACP multi-building, quorum double tête+quotité, MAX_QUOTA base 10000, budget Decimal, associations partielles quotités 2 niveaux).
   - `make ci` vert sur feature/dev sans nouveau testIgnore/continue-on-error.
   - Migration : `sqlx migrate run` + `.down` testé ; backfill warnings inspectés.
   
   ## Risques
   - **Migration multi-building** : SUM auto peut produire un dénominateur faux si l'acte global diffère → `RAISE WARNING` + audit + validation admin (mémoire `admin-publishes-conform-buildings`).
   - **Rétro-compat seeds** : la bascule ACP-level (H7) casse tous les seeds mono-immeuble qui supposaient building=dénominateur ; CI risque élevé sur `bdd_governance.rs`, `bdd_meeting_complete.rs`, seeds expense/call_for_funds/etat_date.
   - **`present_quotas` DOUBLE PRECISION** (dette ADR-0008) : ne pas aggraver en H9 (têtes = i32, sûr).
   - **Docker/CI** : cascade seeds + testcontainers sérialise (mémoire `docker-parallelism-bottleneck`) → cargo check + CI-as-gate, 1 agent backend à la fois.
   - **H15 (units acp_id)** : migration 3 étapes touchant toute création d'unit + isolation #603 (scope_guard) ; risque cascade tests/handlers — backfill depuis `building.acp_id` (toujours présent post-#602).
   - **H16 (associations partielles)** : story la plus profonde ; subtilité **quotités à deux niveaux par lot** (généraux ACP + particuliers PA) à trancher en ADR avant code ; impact charge_distribution + conformité + AG scoped. Garder MVP, ne pas dériver vers un moteur complet.
   - **H17 (représentant de vote)** : la suspension impacte le double quorum (H9) ET le gate vote (H10) → ordonner après H10 ; migration `unit_owners` + maj seeds (lots multi-titulaires doivent désigner un représentant sinon vote suspendu en test).
   
   ## Hors-scope
   - eIDAS / governance hybride (autres stories Maury — slice 4).
   
   > Note : « associations partielles avec personnalité juridique » (H16) et « migration `units.organization_id`→`acp_id` » (H15), initialement différés, sont **remontés in-scope** par décision @gilmry 2026-06-15.

.. raw:: html

   </div>

