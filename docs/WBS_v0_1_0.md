# WBS v0.1.0 — seule vérité courante

`WBS-v0.1.0-r2` · établi le 2026-09-02 · base `feature/dev` à `50406f09`

> **Ce document remplace** `WBS_GO_LIVE_v0.1.0.md` et les trois WBS de 2026-04-01,
> déplacés dans [`docs/archive/`](archive/README.md) avec leur journal de vérification.
> **Suivi vivant** : issue [#736](https://github.com/gilmry/koprogo/issues/736).
> **Instantané du plan** : `docs/plans/2026-09-02-remise-a-plat-0.1.0-jumeau-juridique.md`.

## Ce qui a changé, et pourquoi le périmètre bouge

Le WBS précédent avait 20 critères sur 26 satisfaits et le tag était à portée. Une
simulation de passation de mandat entre deux cabinets sur une même ACP a montré que
le modèle de données était faux au sens de la loi : le syndic entrant voit zéro
écriture, zéro budget, zéro copropriétaire, alors que l'Art. 3.89 § 5, 7° impose la
transmission de l'ensemble du dossier sous trente jours ; le sortant continue de tout
voir, arriérés compris.

Cause racine : `organization` (notion de plateforme) servait de clé d'accès aux
pièces comptables (notion légale). Le fait le plus volatil — une ACP est gérée par ce
syndic, révisable à chaque AG et au plus tous les trois ans — était gravé dans les
enregistrements les plus durables.

**Arbitrage @gilmry du 2026-09-02** : le recentrage ACP *et* le jumeau juridique
complet entrent dans la 0.1.0. Motif : l'autorisation de faire table rase du schéma
ne vaut que tant qu'aucune vraie copropriété n'a encodé quoi que ce soit. Elle tombe
à la première bêta.

**Conséquence assumée** : la mise en ligne recule. C'est un choix de périmètre, pas
un dérapage.

## Règle d'entrée en 0.1.0

> Entre en 0.1.0 ce qui **porte un invariant du registre RFC-0002** ou **rend le
> modèle irréversible**. Reste en 0.2.0 ce qui est un **mécanisme de mise en œuvre**.

Elle a tranché les stories slice-4 : #576, #581, #582, #583 et la part « signatures
du PV » de #578 passent en 0.1.0 ; #577 (itsme/eID), #579 (adaptateurs eIDAS) et
slice-5 restent en 0.2.0.

## Tracks

### Track J — Jumeau juridique (nouveau, chemin critique)

| Lot | Contenu | État |
|---|---|---|
| J0 | Contextes bornés + `tests/architecture.rs` | **fait** `a17ea6b1` |
| J1 | `PieceDeGestion`, `perimetre_du_mandataire`, `SyndicMandate` | **fait** `2529104c` |
| J2 | `acp_id` sur `budget`, `call_for_funds`, `journal_entry`, `etat_date`, `meeting`, `convocation` | **fait** `380fa2f3` `b517298d` `1126d3cd` `a7b785d3` `fa8f206d` |
| J3 | Puis `owner_contributions`, `payment_reminders` | **fait** `1126d3cd` `5a098e37` |
| J4 | `account_balances` recalculée par ACP | **fait** `2c38da55` |
| J5 | Garde d'écriture sur les routes non protégées — #694, #663 | **fait** `1ea85683` (dette bornée à 69, 5 gardes posées) |
| J6 | Les 11 invariants absents — #737 à #747 | **fait, 11 sur 11** |
| J7 | Les 9 partiels — #748 à #756 | **fait, 9 sur 9** |
| J8 | `registre_legal.rs` exécutable + rapport de conformité pour juriste | **fait** `df2259f6` |

**Couverture côté loi** : **29 sur 29**. Le registre est exécutable : deux tests
d'intégrité vérifient que chaque invariant désigne un module et un test qui
existent encore, si bien qu'il ne peut plus se désynchroniser en silence.

Il ne prouve pas qu'un invariant est *correctement* implémenté — c'est le travail
des tests eux-mêmes — mais qu'il est *encore là*. La distinction est écrite dans le
module, pour qu'on ne lise pas la couverture comme une garantie de justesse.

**Le Track J est terminé.**

Livrés depuis, chacun par la boucle rouge-vert avec son article cité dans le nom
du test :

| Article | Invariant | Issue |
|---|---|---|
| 3.86 § 3 al. 7 | part du fonds de réserve annoncée à l'appel de fonds | #737 |
| 3.86 § 3 al. 8 | solidarité des titulaires en cas d'usufruit | #739 |
| 3.87 § 7 | plafonds de procuration, vérifiés à la clôture du vote | #742 |
| 3.87 § 9 | le prestataire ne vote pas sur sa propre mission | #743 |
| 3.89 § 5, 15° | régime comptable dérivé du décompte légal des lots | #746 |
| 3.85 § 3, 3° | fenêtre statutaire de l'AG ordinaire, et le préavis des propositions | #747 |
| 3.86 § 1er | personnalité juridique aux deux conditions, avec l'asymétrie du § 2 | #740 |
| 3.86 § 3 al. 4 | fonds de réserve exigible à cinq ans, plancher de 5 % | #738 |
| 3.87 § 2 | AG sur requête d'un cinquième des parts, et la sanction du silence | #741 |
| 3.87 § 12 | PV consigné au registre et transmis sous trente jours | #744 |
| 3.89 § 5, 13° | contrat lié au syndic : autorisation **préalable** | #745 |

**Aucun invariant du registre n'est plus absent.** Les neuf partiels restent, et
ce sont eux qui portent le solde vers la cible de 29.

Ce que ces onze itérations ont appris, et qui n'était pas dans le registre :

- **la règle du poids de l'Art. 3.87 § 7 interdit à un majoritaire d'emporter
  un vote seul.** Un copropriétaire à 550/1000 la viole dès qu'il vote ;
- **le décompte de l'Art. 3.89 § 5, 15° n'est pas celui de l'acte de base** :
  quinze appartements, quinze caves et vingt parkings font cinquante lots à
  l'acte et quinze au sens de l'article ;
- **l'asymétrie de l'Art. 3.86 § 2** : une ACP non transcrite ne peut pas
  opposer sa personnalité à un tiers, mais ce tiers peut la lui opposer ;
- **l'antériorité de l'Art. 3.89 § 5, 13°** : une autorisation votée après
  signature ne régularise rien.

Chacune de ces quatre lectures aurait pu passer inaperçue dans une
implémentation qui se contente du sens apparent du texte.

**Le dossier de gestion couvre neuf familles de pièces** : charge, budget, appel de
fonds, quote-part, écriture, assemblée, convocation, état daté, relance. Chacune
porte son ACP, et le scénario de passation
(`le_dossier_de_gestion_suit_lacp_lors_dune_passation`) les fait toutes changer de
mandataire sans qu'aucune ne bouge.

**Deux limites connues, écrites sur place plutôt que découvertes plus tard** :
`accounts` (le plan comptable) reste rattaché à l'organisation alors que l'AR du
12/07/2012 le fixe au niveau de l'ACP ; et le filtre de `meeting_repository_impl`
interpole l'identifiant au lieu de le lier.

### Track K — Dette bloquante reprise du WBS précédent

| Lot | Contenu | Issue |
|---|---|---|
| K1 | Plancher Playwright smoke, borné par l'instabilité CI | #696, #548, #723 |
| K2 | Cascade `Result<_, String>` → `AppError` | #555 (**différé 0.2.0**, non bloquant) |
| K3 | Reliquat `f64` monétaire | **fait** `da711473` — le gate est vert, #433 fermée |
| K4 | Vulnérabilités | **partiel** `d85f80c9` — npm à **zéro**, #674 fermée. #432 : deux alertes restantes, corrigées localement, se fermeront quand le correctif atteindra `main` |
| K5 | Contrat OpenAPI | **fait** `c3f736b6` `09ddca66` — #734 et #732 fermées, cliquet 440 → 425 |
| K6 | Bugs fonctionnels ouverts | **partiel** — #662, #721, #722 fermées ; restent #552, #553, #554, #718, #731 |
| K8 | Observabilité et code mort | **fait** `8aa6b59d` — #719 et #720 fermées |
| K7 | Auto-merge Dependabot sans gate CI | **fait** `7c90d191` — #659 fermée |

### Track F — Ops (repris tel quel)

F1 et F2 sont satisfaits de fait : `koprogo.com` et `api.koprogo.com` répondent 200
avec un certificat valide, déployés en continu par `/etc/cron.d/ecosolva-auto-deploy`.
Reste **F3** : aucun drill de rollback ni de restauration GPG+S3 n'a été joué.

### Track G — Gate humain (Tier 1)

- **G1** — revue humaine fraîche, GO signé. Le rapport du 2026-04-01 est archivé et
  ne sert plus. Cette revue doit porter sur le modèle recentré, pas sur l'ancien.
- **G2** — tag `v0.1.0`, posé par un humain après G1.

Ces deux actes ne sont pas délégables : cf. `docs/governance/RESPONSABILITE.md`.

## Ordre d'exécution

1. J2 → J3 → J4 (propriété ACP complète)
2. J5 (garde d'écriture)
3. J6 → J7 (invariants)
4. J8 (registre exécutable)
5. K1, K4, K5, K6, K7 (dette bloquante, parallélisable)
6. F3 (drills)
7. G1 puis G2

## Méthode

Chaque lot part d'un **test rouge**, et pour Track J d'un test qui **cite son
article**. L'ordre — test, domaine, système, documentation — pré-engage le critère
avant la génération. Voir `docs/governance/RESPONSABILITE.md` pour ce qui compte
comme trace et ce qui n'en est pas.

## Vérification

```bash
~/bin/kcargo test --lib -j 2            # 1724 tests de référence, jamais cargo natif
~/bin/kcargo test --test architecture   # la règle de dépendance entre contextes
cargo sqlx prepare                       # SUR L'HÔTE, contre koprogo-prepare-db:5440
```

**Le scénario qui compte** : créer une ACP, l'affecter au cabinet A, saisir un
dossier complet, clore le mandat, ouvrir celui du cabinet B. B voit tout, A ne voit
plus rien. C'est la traduction technique de l'Art. 3.89 § 5, 7°.

E2E via `PLAYWRIGHT_BASE_URL=http://localhost` — viser la production rejoue le
bannissement CrowdSec malgré l'exception.
