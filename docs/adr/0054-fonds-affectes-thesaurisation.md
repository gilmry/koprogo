# ADR 0054: Fonds affectés & thésaurisation (entité `Fund`)

- **Status**: Proposed (draft — en attente de validation @gilmry)
- **Date**: 2026-09-13
- **Track**: Software / Legal-compliance / Finance
- **Authors**: Claude Sonnet 5 (drafting)
- **Related**: [ADR 0012](0012-fonds-reserve-roulement.md) ; issue [#618](https://github.com/gilmry/koprogo/issues/618) (Track H) ; issue [#635](https://github.com/gilmry/koprogo/issues/635) ; story C2.3

## Context

[ADR 0012](0012-fonds-reserve-roulement.md) (Story H13a) a livré le modèle
légal minimal : un fonds de roulement et un fonds de réserve (≥ 5 % des
charges ordinaires N-1, renonçable 4/5), portés par deux colonnes sur
`acps`. Ce modèle binaire ne suffit pas : en pratique une copropriété
**thésaurise** en constituant des **fonds idoines dédiés à un travail
d'ampleur précis** (réfection toiture, façade, ascenseur…), par épargne
pluriannuelle fléchée. Ce troisième cas a été sciemment différé hors H13a
(décision PO @gilmry, 2026-06-25).

Cadre belge (Art. 3.86 §3, Art. 3.88 C. civ., loi du 18/06/2018) :

- **Fonds de roulement** : dépenses périodiques. Compte distinct.
- **Fonds de réserve** : dépenses non périodiques / gros travaux ; ≥ 5 % des
  charges ordinaires N-1, obligatoire (≤ 5 ans), affecté exclusivement aux
  travaux, renonçable à la majorité des 4/5. Compte distinct.
- **Fonds de travaux prévisionnel / affecté** (thésaurisation) : facultatif,
  voté en AG, épargne progressive vers un chantier précis. Les gros travaux
  qu'il finance se votent à la majorité des **2/3** (Art. 3.88 §1, 1°, b).

**Contrainte clé (PO @gilmry)** : l'affectation d'un fonds thésaurisé
**n'est pas un verrou immuable**. Une AG peut décider d'imputer un fonds
affecté à une autre fin que celle prévue à l'origine. Le modèle doit donc
prévoir une réaffectation déclenchée par décision d'AG, avec audit trail, et
ne jamais bloquer une dépense au seul motif que le fonds était fléché
ailleurs dès lors qu'une décision d'AG l'autorise.

## Decision

1. **Entité `Fund`** (`backend/src/domain/comptabilite/fund.rs`), rattachée à
   l'ACP (`acp_id`), à trois natures **exclusives** (`FundKind`) :
   `WorkingCapital`, `Reserve`, `Earmarked` — chacune avec son **propre
   solde** (`balance: Decimal`), donc des comptes distincts au sens du
   modèle (une ligne `Fund` = un compte). Seul `Earmarked` porte `purpose`
   (l'objet du chantier) et `target_amount` (l'objectif d'épargne) ; les
   deux autres natures les refusent (`Fund::new` refuse la combinaison
   invalide, contrainte CHECK dupliquée en base).

2. **Le fonds de réserve légal n'est pas remplacé.** `FundKind::Reserve` ne
   réimplémente pas l'obligation des 5 % / la renonciation 4/5 : ce
   mécanisme reste entièrement porté par `Acp::assert_reserve_fund_compliant`
   (ADR-0012). `Fund` ne fait qu'éviter de confondre un fonds affecté
   facultatif avec l'obligation légale.

3. **Majorité 2/3 à la création d'un fonds affecté.** `Fund::new_earmarked`
   exige que la majorité obtenue satisfasse `MajorityType::TwoThirds`
   (nouvelle méthode `MajorityType::satisfait`, `domain::copropriete::resolution`) —
   Art. 3.88 §1, 1°, b (gros travaux). Créer un fonds fléché à la majorité
   simple contournerait la majorité qui protège les copropriétaires du
   chantier lui-même. Une majorité plus exigeante (4/5, unanimité) satisfait
   aussi le seuil.

4. **Réaffectation, jamais un verrou.** `Fund::reassign(new_purpose, amount,
   resolution_id, resolution_status)` exige `resolution_status ==
   ResolutionStatus::Adopted` et débite `amount` (ou la totalité si `None`)
   du solde. Si le solde retombe à zéro, le fonds sert désormais
   intégralement la nouvelle fin (son `purpose` change) ; sinon le reliquat
   continue de servir l'objet d'origine. L'opération retourne un
   `FundReassignment` (audit trail : `previous_purpose`, `new_purpose`,
   `amount`, `resolution_id`, horodatage), persisté dans
   `fund_reassignments`. Seul un fonds `Earmarked` est réaffectable — la
   réserve légale garde sa propre procédure de renonciation.

5. **Garde-fou dépense / affectation, avec échappatoire d'AG.**
   `Fund::assert_expense_matches_purpose(expense_work_ref, ag_override)`
   refuse une dépense dont l'objet ne correspond pas au `purpose` du fonds
   affecté — sauf si `ag_override == Some(ResolutionStatus::Adopted)`, auquel
   cas la dépense est autorisée : une décision d'AG prime toujours sur le
   fléchage par défaut.

6. **Reliquat.** `Fund::reliquat()` renvoie `balance - target_amount` quand
   le solde dépasse l'objectif (chantier terminé sous budget), `None` sinon.

7. **`call_for_funds.fund_id`** (colonne nullable, `ON DELETE SET NULL`) —
   rattache un appel de fonds au fonds qu'il alimente. Champ optionnel posé
   après construction (`CallForFunds::attach_to_fund`), à l'image de
   `created_by` : aucune signature de constructeur existante n'est modifiée.

## Consequences

**Positives**

- Les trois natures de fonds coexistent sans se confondre (comptes/soldes
  distincts, contrainte CHECK en base).
- L'obligation légale du fonds de réserve (ADR-0012) reste inchangée et
  n'est pas diluée dans le modèle facultatif.
- La majorité qualifiée protège la création d'un fonds affecté comme
  n'importe quel gros travaux.
- La réaffectation reste un geste d'AG tracé, jamais un verrou technique.

**Négatives / coûts**

- MVP : `Fund::reassign` ne modélise qu'un agrégat (le fonds source). Un
  transfert vers un second `Fund` existant (au lieu d'une simple
  réaffectation d'objet sur le même fonds) est un cas d'usage réel mais
  différé — l'audit trail (`fund_reassignments`) suffit pour tracer
  l'opération, l'orchestration inter-fonds est laissée au cas d'usage
  appelant, à enrichir si le besoin se confirme.
- Pas de handlers HTTP / routes / frontend dans cette itération : le
  checklist d'acceptation de la story #635 porte sur Domain + Application +
  Infrastructure (migration + repo + use-cases) ; l'exposition HTTP est un
  incrément suivant, volontairement non inclus ici pour ne pas diluer la
  qualité du socle domaine.

## Alternatives rejetées

- **Un champ `fund_type` supplémentaire sur `call_for_funds` au lieu d'une
  entité dédiée** : rejeté — un appel de fonds est un évènement (l'appel),
  un fonds est un état (le solde cumulé) ; les confondre empêcherait de
  suivre la progression d'une thésaurisation sur plusieurs appels.
- **Réaffectation comme verrou définitif (immutable une fois affecté)** :
  rejeté explicitement par le PO — une AG doit pouvoir réaffecter un fonds
  thésaurisé à une autre fin.
- **Modéliser la réserve légale comme un simple `Fund { kind: Reserve }`
  sans lien avec `Acp::assert_reserve_fund_compliant`** : rejeté — casserait
  la conformité déjà livrée par ADR-0012 ; `Fund::Reserve` documente la
  nature sans se substituer au mécanisme de conformité existant.
