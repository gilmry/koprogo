<!--
Plan de travail exporté depuis la session du 2026-09-02, pour qu'il survive à
une coupure. Validé par @gilmry en trois arbitrages explicites : tout entre
dans la 0.1.0 ; jalons de capacité + labels release ; méthode consignée à la
fois dans la Méthode Foyer et dans la gouvernance koprogo.

Suivi vivant : issue #736. Ce fichier est l'état du plan au moment de son
export, pas un document qui se met à jour tout seul.
-->

# Remise à plat du périmètre 0.1.0 autour du jumeau juridique

## Contexte

Le domaine de KoproGo était une couche plate de 70 entités où trois univers de
règles cohabitaient sans frontière : le Code civil (Livre 3, copropriété), la norme
comptable belge (AR du 12/07/2012), et les fonctions d'économie circulaire.

Ce mélange a produit un défaut de fond, prouvé sur le VPS de démonstration :
`organization` (notion de plateforme SaaS) servait de clé d'accès aux pièces
comptables (notion légale), si bien que le dossier de gestion appartenait au syndic
et non à l'ACP. En simulant une passation de mandat, le syndic entrant voit zéro
écriture, zéro budget, zéro copropriétaire, alors que l'Art. 3.89 § 5, 7° impose la
transmission de l'ensemble du dossier sous trente jours ; le sortant, lui, continue
de tout voir, arriérés et données personnelles compris.

L'objectif dépasse la correction de ce défaut : rendre le domaine **vérifiable
contre son référentiel**, c'est-à-dire pouvoir répondre à « que dit la loi, et où le
code y répond ? » et mesurer la couverture du côté de la loi. Le jumeau est
*augmenté* : chapitre copropriété entier, noyau comptable, économie circulaire.

Le suivi de projet, lui, est à plat au mauvais sens du terme : **cinq documents WBS
concurrents** pour la même 0.1.0, **41 issues ouvertes sur 75 sans jalon ni
version**, un label `release:0.1.0` que **zéro issue ouverte ne porte**, et un WBS
courant qui annonce lui-même « aucune issue/milestone GitHub créée ».

### Décisions prises

- **Tout entre dans la 0.1.0** : le recentrage ACP *et* le jumeau juridique complet.
- **Jalons de capacité** (0 à 7) comme axe de progression, **labels `release:x.y.z`**
  pour dire ce qui part dans quel tag. Chaque issue ouverte reçoit les deux.
- La formulation méthodologique va **dans les deux endroits** : la Méthode Foyer
  (portable) et la gouvernance koprogo (déclinaison locale).

**Conséquence à énoncer une fois** : le WBS actuel n'avait plus que quatre critères
ouverts sur vingt-six et le tag était à portée. Y faire entrer le jumeau complet
recule la mise en ligne de façon substantielle. C'est un choix défendable — le
modèle de données est porteur, et l'arbitrage « table rase sans migration » ne vaut
que tant qu'aucune vraie copropriété n'a encodé quoi que ce soit — mais il doit être
assumé comme tel, pas subi à mi-chemin.

### État à l'instant

Fait et vert, **non commité** (71 fichiers) : quatre contextes bornés, 69 entités
déplacées, façade de transition, `tests/architecture.rs` qui interdit les
dépendances croisées. Vérifié en conteneur : 1724 tests unitaires, 3 tests
d'architecture, zéro échec.

Commité en `2529104c` : ADR-0045, RFC-0002, `PieceDeGestion` et le scénario de
passation.

---

## Phase 0 — Verrouiller l'acquis

Commiter la restructuration en contextes bornés, seule, avec son test d'architecture.
Elle est vérifiée ; la laisser non commitée fait courir le risque de la refaire.

**Fichiers** : `backend/src/domain/{copropriete,comptabilite,economie_circulaire,plateforme}/`,
`backend/src/domain/entities/mod.rs` (façade), `backend/src/domain/mod.rs`,
`backend/tests/architecture.rs`.

---

## Phase 1 — L'issue umbrella et ses enfants

**Une umbrella** « Jumeau numérique du Code civil — recentrage ACP et registre des
invariants », qui référence l'ADR-0045 et le RFC-0002, et rattache les issues
existantes qui en sont les prédécesseurs : **#618** (refonte modèle copropriété,
acte de base ACP hybride), **#694** (scoping user↔ACP absent), **#663**
(BuildingSelector devrait viser l'ACP), **#556** (EPIC refonte UX multi-rôle + ACP).

**Des enfants**, un par invariant absent du registre RFC-0002, chacun portant son
article en titre pour être triable par un juriste autant que par un développeur :

| Article | Invariant |
|---|---|
| 3.86 § 3 al. 7 | part affectée au fonds de réserve communiquée à l'appel de fonds |
| 3.86 § 3 al. 4 | fonds de réserve obligatoire à 5 ans, plancher de 5 % des charges ordinaires |
| 3.86 § 3 al. 8 | usufruit : solidarité des titulaires de droits réels |
| 3.86 § 1er | personnalité juridique aux deux conditions cumulatives |
| 3.87 § 2 | AG sur requête d'un cinquième des parts, convocation sous 30 jours |
| 3.87 § 7 | plafond de trois procurations, sauf ≤ 10 % des voix |
| 3.87 § 9 | conflit d'intérêts : un prestataire ne vote pas sur sa mission |
| 3.87 § 12 | PV au registre et transmis sous 30 jours |
| 3.89 § 5, 13° | contrat ACP↔syndic ou proches : autorisation préalable de l'AG |
| 3.89 § 5, 15° | comptabilité simplifiée sous 20 lots, caves/garages/parkings exclus |
| 3.85 § 3, 3° | fenêtre statutaire de quinze jours de l'AG ordinaire |

Plus les neuf **partiels** en issues distinctes (numéro d'entreprise sur tous les
documents, échéance des cinq ans, mode d'envoi accepté par écrit, signatures du PV,
affectation des majorités de l'Art. 3.88, relevé notaire sous 30 jours, compétence
du conseil de copropriété, arriérés à la mutation).

---

## Phase 2 — Un WBS unique

Écrire `docs/WBS_v0_1_0.md`, **seule vérité courante**, qui absorbe les items encore
ouverts de `WBS_GO_LIVE_v0.1.0.md` (D1 plancher Playwright borné par #696, F3 drills
deploy/rollback et restore GPG+S3, G1 revue humaine fraîche, G2 tag, cascade
`Result<_,String>`→`AppError`, reliquat `f64` monétaire) et y ajoute les tracks du
jumeau.

Déplacer les quatre documents périmés dans `docs/archive/` : `WBS_RELEASE_0_1_0.md`,
`WBS_BUGFIX_UI_v0.1.0.md`, `WBS_CORRECTIONS_v0.1.0.md`, et `WBS_GO_LIVE_v0.1.0.md`
lui-même une fois ses items ouverts repris — en conservant son journal de sessions,
qui est la trace de ce qui a été vérifié et quand.

`ROADMAP_PAR_CAPACITES.rst` et `WBS_PROJET_COMPLET.rst` restent : ce sont l'axe
stratégique et la référence technique, pas des WBS de release.

---

## Phase 3 — Remettre les 75 issues à plat

**La règle d'entrée en 0.1.0**, à énoncer dans le WBS pour qu'elle soit opposable :

> Entre en 0.1.0 ce qui porte un invariant du registre RFC-0002 ou rend le modèle
> irréversible. Reste en 0.2.0 ce qui est un mécanisme de mise en œuvre.

Elle tranche notamment les stories slice-4 aujourd'hui étiquetées `release:0.2.0` :

- **passent en 0.1.0** — #576 (participation à distance, Art. 3.87 § 1er al. 1er),
  #581 (évaluation des contrats, Art. 3.89 § 5, 12°), #582 (conseil de copropriété,
  Art. 3.90), #583 (commissaire aux comptes, Art. 3.91), et la part « signatures du
  PV » de #578 (Art. 3.87 § 10) ;
- **restent en 0.2.0** — #577 (itsme/eID), #579 (adaptateurs eIDAS), la part
  fournisseur qualifié de #578, et tout slice-5 (#585 à #592 : activation de modules,
  RBAC communauté, onboarding), qui relèvent du produit et non de la loi.

Puis, sur les 75 ouvertes : attribuer à chacune un jalon de capacité **et** un label
`release:`, fermer celles que le code a dépassées (à vérifier une par une, pas par
défaut — cf. #433, #443, #661 déjà traitées d'après le WBS), et ordonner le reste
par dépendance plutôt que par date de création.

---

## Phase 4 — Consigner la méthode

**« Human in the loop, AI first execution, human answerable »** précise la première
phrase de la Méthode Foyer (« Tu génères, l'humain valide ») et sa boussole
*répondre-de*. Elle dit trois choses distinctes : l'humain est dans la boucle, pas
après ; l'exécution part de l'IA ; et c'est l'humain qui en répond.

- `/home/ubuntu/jarvis/projects/foyer/Methode-Foyer.md` — la formulation générale,
  dans la section 4 (la boussole), portable vers tous les projets Maury.
- `docs/governance/` de koprogo — sa déclinaison : quels actes exigent une signature
  humaine avant le tag. Les notions Tier 1 / Tier 2 existent déjà, éparpillées dans
  le RFC-0001 et les WBS ; les rassembler là.

---

## Ordre d'exécution vers la 0.1.0

1. Contextes bornés — **fait, à commiter**
2. Propriété ACP : les six entités restantes, puis `payment_reminders`,
   `owner_contributions`, et la vue `account_balances` à recalculer par ACP
3. Garde d'écriture sur les routes non protégées, et #694 / #663 qui en découlent
4. Les 11 invariants absents, chacun par sa boucle rouge-vert
5. Les 9 partiels
6. Registre exécutable (`registre_legal.rs`) et rapport de conformité généré
7. Dette bloquante préexistante : #696, D1, cascade `AppError`, vulns #432/#674
8. Ops : drills F3
9. Gate humain : G1 revue fraîche, puis G2 tag — les seuls actes que l'IA ne pose pas

---

## Vérification

- `~/bin/kcargo test --lib -j 2` — suite complète en conteneur (référence 1724 ;
  jamais `cargo` natif, la racine ne fait que 48 Go).
- `~/bin/kcargo test --test architecture` — la règle de dépendance entre contextes.
- `cargo sqlx prepare` **sur l'hôte** contre `koprogo-prepare-db` (port 5440) après
  toute modification de requête : kcargo ne passe ni `DATABASE_URL` ni le réseau.
- **Le scénario qui compte** : rejouer la passation contre la pile locale. Créer une
  ACP, l'affecter au cabinet A, saisir un dossier complet (budget, appel de fonds,
  écritures, convocation, réunion, état daté), clore le mandat, ouvrir celui du
  cabinet B. B voit tout, A ne voit plus rien. C'est la traduction technique de
  l'Art. 3.89 § 5, 7°.
- **Couverture côté loi** : le rapport généré depuis `registre_legal.rs` doit passer
  de 9 couverts / 9 partiels / 11 absents à 29 couverts.
- E2E via `PLAYWRIGHT_BASE_URL=http://localhost` — viser la production rejoue le
  bannissement CrowdSec malgré l'exception.
