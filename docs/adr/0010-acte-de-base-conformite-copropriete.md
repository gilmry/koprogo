# ADR 0010: Acte de base au niveau ACP & conformité copropriété à deux niveaux (modèle hybride)

- **Status**: Proposed (acceptation @gilmry au merge)
- **Date**: 2026-06-15
- **Track**: Software / Legal-compliance / Governance
- **Authors**: Claude Opus 4.8 (drafting) + @gilmry sign-off
- **Related**: [ADR 0007](0007-decimal-vs-f64-for-money.md) (Decimal), [ADR 0011](0011-quorum-double-et-suspension-vote.md), [ADR 0012](0012-fonds-reserve-roulement.md) ; issue [#618](https://github.com/gilmry/koprogo/issues/618) ; story CL0/CL1 `docs/maury/refonte-ux-multi-role-acp/track-h-conformite-legale/`

## Context

KoproGo gère des copropriétés belges. La revue domain du 2026-06-15 (issue #618) a confronté le modèle au **Code civil Livre 3** (réforme 2018/2020) :

- **Art. 3.84 CC** — la copropriété forcée s'applique à *« tout immeuble **OU groupe d'immeubles** »*. L'**acte de base** et le règlement de copropriété *« constituent les statuts de l'immeuble ou du groupe d'immeubles »* (acte authentique). L'acte de base **fixe la quote-part des parties communes afférente à chaque partie privative**, selon la valeur respective (superficie nette, affectation, situation).
- **Art. 3.86 CC** — l'**association des copropriétaires (ACP)** est la personne morale de la copropriété. Pour un **groupe d'immeubles**, seul le groupe a la personnalité juridique.
- **Quotités** — exprimées en millièmes (1000) ou dix-millièmes (10000), convention fixée dans l'acte de base ; **leur somme = le dénominateur**.

Sources : [Code civil ejustice](https://www.ejustice.just.fgov.be/img_l/pdf/2020/02/04/2020020347_F.pdf), [quotités choisirunsyndic](https://www.choisirunsyndic.be/dossiers/quotites-en-copropriete/), [personnalité juridique copropriete-ejuris](https://copropriete-ejuris.be/personnalite-juridique/).

**Problème** : le modèle actuel porte le dénominateur des quotités sur `buildings.total_tantiemes`. Or l'acte de base est l'attribut de la **copropriété (ACP)**, pas d'un immeuble isolé. Pour une ACP mono-immeuble c'est sans conséquence, mais pour un **groupe d'immeubles** (1 ACP, 1 acte de base, N blocs), les quotités somment au dénominateur **au niveau du groupe**, pas par bloc. La conformité `SUM(quota)==total_tantiemes` par building est donc structurellement incorrecte pour ce cas.

Le bug `CONFORMANT_QUOTA_TOTAL=dec!(1000)` hard-codé a déjà été corrigé (Story H1, `6a053a1`) en lisant `self.total_tantiemes` — mais cela reste au mauvais niveau (building).

## Decision

Adopter un **modèle hybride** :

1. **`acps.total_tantiemes`** (nouvelle colonne) = dénominateur de l'acte de base, **source de vérité unique** de la copropriété (Art. 3.84).
2. **`buildings.total_tantiemes`** conservé mais **redéfini** = sous-total de quotités du bloc (utile pour les sous-charges / associations partielles).
3. **Conformité à deux niveaux** :
   - **ACP-level** (légal) : `Acp::is_conformant(metrics) := Σ(quota de toutes les units de tous les blocs) == acps.total_tantiemes`. Erreur typée `AcpNotConformantError { acp_id, quota_delta, quota_basis }` → `AppError::AcpNotConformant` (HTTP 422).
   - **Building-level** (sous-total bloc) : `Building::is_conformant` (Story H1) **conservé**.
4. **Gates `validate-before-compute` au niveau ACP** : les 4 use-cases opérationnels (expense, call_for_funds, charge_distribution, etat_date) résolvent `building.acp_id` et vérifient la conformité **ACP** avant tout calcul (retravaille le gate building-level de la Story H2).
5. **`Unit.quota`** non borné à 1000 : la borne effective est `acps.total_tantiemes` (corrige le bug latent `Unit::MAX_QUOTA`).
6. Pas de `units.acp_id` redondant : le lien `unit → building → acp` suffit (cohérence post-#602).

### Hors-scope (différé v0.2.0, décision PO D6 2026-06-15)
Les **associations partielles à personnalité juridique propre** (Art. 3.86) et les **quotités à deux niveaux par lot** (communs généraux ACP + communs particuliers PA, `units.particular_quota`) sont **reportées en v0.2.0**. Le modèle hybride v0.1.0 se limite à ACP (acte de base) + building (sous-total bloc).

## Consequences

**Positives**
- Conformité juridiquement exacte (acte de base = copropriété), supporte le groupe d'immeubles à un acte de base unique.
- Source de vérité unique du dénominateur ; plus de drift inter-blocs.
- Réutilise le pattern éprouvé `BuildingNotConformantError` / `From<>` / 422.

**Négatives / coûts**
- Migration `acps.total_tantiemes` + backfill (mono = building ; multi = `SUM` des sous-totaux + `RAISE WARNING` + validation admin manuelle pour les ACPs multi-blocs dont l'acte global pourrait différer).
- Retravail du gate Story H2 (building→ACP) + reprise des seeds de tests mono-immeuble.
- Rétro-compat : nombreux seeds BDD/E2E supposent building = dénominateur → adaptation (cf. stash WIP).

## Alternatives rejetées

- **Garder `total_tantiemes` sur building** : simple mais juridiquement faux pour les groupes d'immeubles ; rejeté.
- **ACP-only (supprimer building.total_tantiemes)** : perd le sous-total par bloc nécessaire aux associations partielles (v0.2.0) ; rejeté.
- **Hybride (choisi)** : ACP = dénominateur + building = sous-total ; meilleur compromis conformité/évolutivité.
