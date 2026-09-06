# WBS v0.1.0 — seule vérité courante

`WBS-v0.1.0-r3` · établi le 2026-09-02, révisé le 2026-09-04 · base `feature/dev` à `31ff3fc8`

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
| K6 | Bugs fonctionnels ouverts | **fait pour ce qui est corrigeable ici** — #552, #553, #554, #662, #721, #722 fermées. Restent #718 (comportement sous charge) et #731 (DNS du VPS), qui demandent l'environnement réel |
| K8 | Observabilité et code mort | **fait** `8aa6b59d` — #719 et #720 fermées |
| K7 | Auto-merge Dependabot sans gate CI | **fait** `7c90d191` — #659 fermée |

### Track R — Recette navigateur et revue de design, 2026-09-04 au 06

Cinq sessions de recette humaine par Cowork sur la production, plus une revue de design
frontend reçue le 06. **Trente et une issues ouvertes le 2026-09-06**, toutes au périmètre
de la 0.1.0 par décision du 06.

Les rapports ont trouvé de vraies choses et **quatre faux diagnostics**, redressés par la
mesure : le « crash API » était un bannissement CrowdSec, la « régression CORS » le même
bannissement, `onclick: null` est le comportement normal de Svelte 5, et « 7 modules
communautaires sans backend » venait de sondages sur des chemins que personne n'appelle.
Le testeur a lui-même trouvé et corrigé un biais de son outil — une échelle de 1,125 sur
ses coordonnées de clic — qui lui faisait manquer toutes les petites cibles.

#### R1 à R13 — recettes 1 à 4

| Lot | Contenu | Issue | État |
|---|---|---|---|
| R1 | **Fuite inter-organisations sur les routes imbriquées.** Un syndic lisait les bulletins nominatifs d'une autre copropriété. 2 routes fermées, **73 restent sans identité** sur 310 | #772 | **partiel** |
| R2 | Plafonnement Art. 3.87 § 7 appliqué à la lecture, plus seulement à la clôture | #767 | **fait** |
| R3 | L'écran de résultat de vote comptait des têtes quand l'API compte des voix | #773 | **fait** |
| R4 | Modules communautaires : le frontend appelle des chemins par ACP, le serveur sert par immeuble. 111 points d'entrée dorment | #779, #768 | ouvert |
| R5 | `register` reposait la session sur l'appelant authentifié | #769 | **fait** |
| R6 | Page RGPD en anglais, intitulés, boutons et paragraphes | #774 | **fait** |
| R7 | « Clôturer le vote » sans effet : le champ `total_voting_power` était obligatoire et le frontend envoyait `{}` | #776 | **fait** |
| R8 | Annonces : création invisible, filtre en chargement infini, énumérations brutes | #775 | **fait** |
| R9 | Inscription orpheline, « mot de passe oublié » sans backend | #771 | **fait** |
| R10 | Boutons de création d'immeuble et de lot cachés au syndic | #778 | **fait** |
| R11 | CrowdSec bannit les testeurs à cause de nos propres 404 | #766 | **palliatif** — liste blanche posée, la cause tient à R4 |
| R12 | Test intermittent : un UUID aléatoire contenant « 400 » bloquait le déploiement | #777 | **fait** |
| R13 | Contrat OpenAPI absent pour `/expenses` et `/invoices` | #765 | ouvert |

#### R14 à R18 — recette 5

| Lot | Contenu | Issue | État |
|---|---|---|---|
| R14 | **Les erreurs 400 nomment le champ fautif et l'interface le jette.** Trois coupures dans `api.ts` et `error.utils.ts`. Deux extracteurs de `details` existent déjà et **ne peuvent jamais fonctionner** | #782 | ouvert |
| R15 | `acp_id` absent du formulaire d'immeuble pour un syndic. **Régression introduite par R10** : le bouton a été ouvert, pas le champ | #783 | ouvert |
| R16 | L'envoi de convocation exige `recipient_owner_ids` que l'interface ne peut pas constituer | #784 | ouvert |
| R17 | « Reporter » : **faux positif**, le bouton fonctionne. Un `prompt()` rejeté par l'outil de test expliquait tout. Reste à remplacer les dialogues natifs et à corriger le journal d'audit du report | #785 | **partiel** |
| R18 | Le type `Vote` du frontend ne correspond pas au DTO servi ; clé `notices.draft` affichée en clair | #786 | **fait** |
| R22 | **La PWA n'a jamais fonctionné** : le service worker échoue à l'installation depuis novembre 2025, deux icônes du manifeste répondent 404 | #804 | ouvert |

#### R19 à R21 — revue de design frontend, « Part 0 »

Dix défauts de code, **vérifiés un par un**, une issue chacun. La refonte UX elle-même
reste **hors 0.1.0** : son étape principale, le passage du périmètre à l'ACP dans le modèle
de données et `permissions.ts`, touche la même zone que R1, où 73 routes n'ont toujours
aucune identité.

| Lot | Contenu | Issue |
|---|---|---|
| R19 | **Le jeton JWT et le `localStorage` écrits dans la console du navigateur** | #787 |
| R20 | Classes Tailwind interpolées : les styles ne sont jamais générés (tableau comptable, grille du conseil) | #788, #789 |
| R21 | Dette de forme : configuration Tailwind morte, `theme-color` périmé, activité inventée à Paris et Lyon, énumérations brutes, accent manquant, piège de focus absent, desktop-first | #790, #796, #791, #792, #793, #794, #795 |

#### Ce que ce track apprend, et qui dépasse ses lots

**La majorité des défauts trouvés sont des capacités écrites, testées et inatteignables.**
Les modules communautaires, la page d'inscription, les boutons de création, le plafonnement
des voix, la clôture du vote, les deux extracteurs de `details`, la règle de l'Art. 3.87 § 3
al. 3 sur l'accord préalable au courriel : dans chaque cas le code existe, ses tests passent,
et rien n'y mène.

**Nos tests prouvent que le code marche tout en masquant qu'on ne peut pas y arriver.** Ce
motif n'est visible que par un humain devant un navigateur, et cinq recettes l'ont confirmé.

**Une seconde cause commune est apparue en recette 5** : le frontend et le serveur ne
s'accordent pas sur les noms de champs, et rien ne le détecte — `acp_id`,
`recipient_owner_ids`, `total_voting_power`, `vote_choice`, `voted_at`, `content`. C'est
R13 : sans contrat OpenAPI, le frontend écrit ses types à la main et ils dérivent.

**R1, R4, R14, R15 et R19 sont les plus bloquants.** R1 et R19 exposent des données. R14
rend visible tout le reste. R15 empêche de créer un immeuble. R4 fait bannir les testeurs.

### Track U — Refonte UX/UI (revue Claude Design du 2026-09-06)

Entrée au périmètre 0.1.0 sur décision du 06. Neuf lots, dont l'ordre est
contraint : U2 dépend de R1, et tout le reste dépend de U1.

| Lot | Contenu | Issue | Dépend de |
|---|---|---|---|
| U1 | **Jetons de design et jeu d'icônes SVG.** Les émojis collisionnent — 📋 📊 📅 📄 💰 servent chacun deux entrées de menu | #797 | — |
| U2 | **Le périmètre devient l'ACP** dans le modèle de données et `permissions.ts`. Le plus gros lot, et celui dont tout dépend | #798 | **R1 (#772)** |
| U3 | Coquille : barre latérale, barre de contexte unique, navigation mobile, inversion des points de rupture | #799 | U1, U2 |
| U4 | Motif de liste unifié et composant d'encadré légal, alimenté par le registre de règles | #800 | U1 |
| U5 | Tableaux de bord : une file de tâches, pas une navigation dupliquée. Mobile-first par rôle | #801 | U3, U4 |
| U6 | Contrat de tests : ce qui doit survivre, ce qui s'adapte, ce que ce travail doit en plus | #802 | — |

**U2 ne commence pas avant que R1 soit fermée.** Le passage à l'ACP touche
`permissions.ts`, le store de périmètre et les gardes de portée — exactement la
zone où **73 routes imbriquées n'ont aucune identité**. Remanier un
cloisonnement pendant qu'il est percé serait l'ordre inverse du bon.

**Le contrat `data-testid` est figé depuis le 2026-09-06** : 882 identifiants
littéraux et 22 préfixes construits, versionnés dans
`frontend/src/lib/__tests__/data-testid.contrat.json` et gardés par un cliquet.
Il ne peut que grandir. Le code en offre 882 quand les tests n'en interrogent
que ~530 : plus de trois cents existent sans filet et auraient disparu sans un
bruit pendant la refonte.

**Ce que la revue apporte au-delà des écrans.** Elle signale elle-même sa
contradiction avec un test `@security` — sa barre latérale du comptable inclut
un groupe Communauté, que `Navigation.test.ts` interdit — et tranche en faveur
du test. Elle dit aussi que ses références légales et ses codes PCMN sont des
valeurs de maquette, à lire depuis le registre de règles du projet. Sur un
sujet où trois références périmées et une affirmation juridique sans base ont
déjà été trouvées, c'est la bonne règle.

### Track D — Documentation vivante multi-persona (nouveau, 2026-09-06)

Sur le modèle de `gilmry/klaar` : livrables portant leur persona d'auteur en
en-tête, parcours numérotés par persona, organisation par contexte borné.

| Lot | Contenu | Issue |
|---|---|---|
| D1 | Un document par rôle — syndic, copropriétaire, comptable, admin — avec parcours nominal, refus assumés et références légales | #805 |
| D2 | Workflows transverses : cycle de vie d'une AG, circuit d'une facture, entrée d'un copropriétaire | #805 |
| D3 | Cliquet de couverture documentaire, et vérification que les parcours décrits sont **atteignables** | #805 |

**Ce que KoproGo a déjà, et ce qui manque.** Le dépôt compte 86 documents dans
`docs/`, et ils sont bons — PCMN belge, RGPD, convocations, workflow de
facture, gouvernance. Mais ils sont classés **par sujet technique ou
réglementaire, jamais par personne**. Aucun ne répond à la question qu'un
syndic se pose en arrivant : que puis-je faire, dans quel ordre, et pourquoi.

**Pourquoi c'est en 0.1.0.** Même raison que le reste : un financeur ne lit pas
du code. Le produit doit être compréhensible avant la fondation de l'ASBL.

**Ce que « vivante » veut dire ici.** Une documentation qui se met à jour parce
qu'un test la garde, pas parce qu'on y pense. Le dépôt a déjà les deux
mécanismes : le cliquet, employé cinq fois, et le contrat `data-testid` figé
(#802, #803) — un parcours documenté dont une étape n'a pas d'ancrage est un
parcours qu'on ne peut pas prouver.

**Un document qui décrit une capacité inatteignable ment.** C'est le motif
dominant des défauts de ce produit, et c'est pourquoi D1 exige que chaque
document nomme ce qui ne fonctionne pas encore.

### Track F — Ops (repris tel quel)

F1 et F2 sont satisfaits de fait : `koprogo.com` et `api.koprogo.com` répondent 200
avec un certificat valide, déployés en continu par `/etc/cron.d/ecosolva-auto-deploy`.

**F3 est joué le 2026-09-04** — rapport : [`docs/ops/2026-09-04-drill-f3-restauration.md`](ops/2026-09-04-drill-f3-restauration.md).
Son résultat est **négatif sur deux volets sur trois**, et le noter « fait » sans le
dire serait un mensonge :

| Volet | Résultat |
|---|---|
| Rollback de déploiement | **échoue** dès qu'une version a migré — constaté en réel le 2026-09-03, la démo est restée morte |
| Sauvegardes GPG+S3 du runbook | **n'existent pas** sur ecosolva : ni cron, ni clé GPG, ni `s3cmd` |
| Restauration d'un dump | **fonctionne** : 13 s, zéro erreur, 1247 ACP et 11 371 lots retrouvés |

Le drill a aussi **nommé la cause** de l'incident du 2026-09-03, jusque-là attribuée
vaguement à « un conflit avec les données existantes » : deux migrations refusent de
s'appliquer parce que 13 quotes-parts n'ont aucun lot et 9 écritures aucun
rattachement. Elles ont raison de refuser.

`scripts/quarantaine-pieces-sans-acp.sql` lève le blocage **sans rien détruire**, en
déplaçant ces pièces vers une table de quarantaine. Chemin complet vérifié sur la
sauvegarde du 2026-08-31 : restauration → quarantaine → 17 migrations, zéro échec,
données intactes.

**Ce qui reste ouvert au titre de F3** : aucune sauvegarde automatisée n'existe sur
la machine, et le runbook décrit une procédure absente. C'est plus dangereux qu'un
runbook vide, parce qu'on se croit couvert.

### Track G — Gate humain (Tier 1)

- **G1** — revue humaine fraîche, GO signé. Le rapport du 2026-04-01 est archivé et
  ne sert plus. Cette revue doit porter sur le modèle recentré, pas sur l'ancien.
  **Dossier préparé** : [`docs/governance/G1-dossier-de-revue-0.1.0.md`](governance/G1-dossier-de-revue-0.1.0.md)
  — les points à trancher, ce qui reste cassé, et les deux lectures de l'Art. 3.87
  § 7 qui ne sont pas dictées par le texte.
- **G2** — tag `v0.1.0`, posé par un humain après G1.

Ces deux actes ne sont pas délégables : cf. `docs/governance/RESPONSABILITE.md`.

## Ce qui reste, et ce qui le bloque

### ⚠️ Le périmètre a doublé le 2026-09-06, et c'est une décision assumée

**Tout ce qui restait en 0.2.0 entre en 0.1.0, avec la refonte UX/UI.** Le compte passe de
**34 à 77 issues ouvertes** en `release:0.1.0` : les 31 ouvertes le 2026-09-06, les 24
qui étaient en 0.2.0, et les 6 lots de la refonte. Il ne reste plus rien en 0.2.0.

**La raison est stratégique et elle est écrite ici pour qu'on s'en souvienne.** Le produit
doit être bon **avant** la fondation de l'ASBL, parce que c'est sur lui que reposera la
levée de fonds. Une v0.1.0 qui se contenterait de « fonctionner assez pour être montré »
serait suffisante pour une démonstration et insuffisante pour convaincre un financeur.

Ce que « v0.1.0 » signifie change donc deux fois. Ce n'est plus « ce qui fonctionne assez
pour être montré », ni même « ce qu'un syndic peut mener à son terme » : c'est **ce qu'on
peut présenter à quelqu'un qui décide d'y mettre de l'argent**.

Quatre conséquences à assumer :

- **La date recule nettement.** Soixante-dix-sept issues, dont plusieurs de fond : les 73
  routes sans identité (#772), la migration `Result<_, String>` (#555, 1263 occurrences),
  le périmètre ACP (#694, #798), et six lots de refonte.
- **L'ordre compte plus que le compte.** R14 (#782) rend visibles les erreurs et donc tout
  le reste ; R1 (#772) et R19 (#787) exposent des données ; U2 (#798) ne peut pas commencer
  avant R1. Ces dépendances sont dures, pas indicatives.
- **Trois arbitrages produit bloquent** : #770 (conformité et `total_units`), #779
  (périmètre des modules communautaires), #781 (le syndic agissant pour l'ACP). Aucun ne se
  tranche en écrivant du code.
- **G1 devient le vrai jalon.** La revue humaine portera sur un produit complet, ce qui est
  la seule façon d'en tirer un avis qui vaille pour un financeur.

### Les catégories antérieures, au 2026-09-03

**Faisable ici** : #426 (nettoyage de docs), #427 (taxonomie et gate de release),
et les volets restants des stories #576, #581, #582, #583, #663 — dont la part
domaine est livrée. Nouvelles depuis la session du 2026-09-04 : #761, #762, #763.

**Plus bloqué par un push** : la série a été poussée sur `feature/dev` les 3 et 4
septembre, sur autorisation. #660 est **fermée**, preuves d'exécution à l'appui.
#443 a sa part compilation résorbée ; ce qu'elle recouvrait d'autre est mesuré et
commenté sur l'issue.

**Bloqué par l'environnement ou par un humain** : #696, #548 et #723
(l'instabilité Playwright ne s'observe qu'en CI), #718 (comportement sous
rafale), #731 (collision DNS sur le réseau partagé), et **G1 la revue humaine et
G2 le tag**, qui sont Tier 1 par `docs/governance/RESPONSABILITE.md`.

S'y ajoutent les onze harnais BDD, qui exigent le socket Docker et ne
s'exécutent **nulle part** aujourd'hui : ni en local, ni en CI puisque `ci.yml`
exclut `feature/dev`. Mesuré le 2026-09-04, suivi en #540.

### Ce que la branche vérifie désormais avant de déployer

`ci.yml` exclut nommément `feature/dev` — sa pipeline prend ~95 minutes et le
déploiement ne l'attend pas. Le coût de ce choix a été chiffré : **45 régressions
ont vécu sur la branche qui alimente la démo publique** sans qu'aucun signal
n'apparaisse, parce que `cargo test --lib` ne compile pas `tests/`.

Depuis `e337c603`, `vps-feature-dev.yml` porte un **barrage rapide** qui
conditionne la construction des images : gardes ADR-0008 et OpenAPI, `fmt`,
`clippy --all-targets` (la seule étape qui compile les harnais), tests unitaires,
garde d'architecture, cliquet de garde d'écriture, `astro sync`, `svelte-check`,
vitest. Ni e2e, ni BDD, ni Playwright.

Un commit rouge ne produit donc pas d'image et la démo reste sur sa dernière
version saine. Coût mesuré : **11 minutes** à cache froid, contre 1 minute pour
les images. Pour relâcher sans supprimer : retirer le `needs: barrage`.

## Ordre d'exécution

**Le Track R passe devant**, parce qu'il contient ce qui expose des données et ce qui
empêche d'utiliser le produit, et parce que R14 rend observable tout ce qui suit.

1. **R14** (#782) — rendre les erreurs lisibles. Une seule correction d'affichage révèle une
   classe entière de défauts, et réveille deux extracteurs morts. À faire en premier, sans
   discussion : tout le reste se vérifie mieux ensuite.
2. **R19** (#787) et **R1** (#772) — ce qui expose des données : le jeton en console, puis
   les 73 routes imbriquées sans identité.
3. **R15** (#783) et **R16** (#784) — les champs manquants, qui débloquent la création
   d'immeuble et l'envoi de convocation, donc #770 et le cycle de vie d'une AG.
4. **R17** (#785) — trancher l'hypothèse du bundle périmé **avant** d'écrire du code. Si elle
   se confirme, elle remet en cause chaque vérification navigateur faite jusqu'ici.
5. **R4** (#779) — rebrancher les modules communautaires, ce qui referme aussi R11.
6. R18, R20, R21 — le reste du Track R, parallélisable.
7. **U1** (#797) — jetons et icônes : tout le Track U s'appuie dessus.
8. **U6** (#802) — relever le contrat de tests avant d'y toucher. Le contrat
   `data-testid` est déjà figé et gardé par un cliquet.
9. J2 → J3 → J4 (propriété ACP complète)
10. J5 (garde d'écriture)
11. J6 → J7 (invariants)
12. J8 (registre exécutable)
13. **U2** (#798) — le périmètre ACP, **une fois R1 fermée**, et pas avant.
14. **U3 → U4 → U5** — coquille, motif de liste, tableaux de bord.
15. K1, K4, K5, K6, K7 (dette bloquante, parallélisable)
16. **D1 → D2 → D3** — la documentation vivante, **après** que les parcours
    fonctionnent : documenter un parcours qui casse produit un document qui ment.
17. F3 (drills)
18. G1 puis G2

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
