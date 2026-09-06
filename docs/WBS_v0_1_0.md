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

#### R1 — état au 2026-09-06 : 73 routes gardées sur 81 (#772)

Le relevé initial comptait 73 routes imbriquées sans aucune identité. Il en
reste **8**, et aucune ne se règle en ajoutant un garde : chacune demande une
décision.

Quatre gardes nouvelles sont venues compléter les quatre existantes —
`verify_owner_org_access`, `verify_document_org_access`,
`verify_unit_org_access`, `verify_convocation_org_access`. Les quatorze routes
portées par un copropriétaire méritent d'être signalées à part : elles
servaient **nominativement** ce qu'une personne doit et ce qu'elle a payé.

**Et le cliquet lui-même était faux.** Il comptait la présence du paramètre
`AuthenticatedUser` — nécessaire, pas suffisant. Un script d'insertion a posé
ce paramètre sans la garde qui l'emploie ; le cliquet l'aurait comptée
protégée, et seul l'avertissement `unused variable` du compilateur l'a trahie.
Un second cliquet mesure désormais l'effet : **109 routes prennent l'identité
sans jamais l'employer**, majorant à trier.

**Le seul arbitrage restant** porte sur les quatre routes de campagne d'énergie :
un achat groupé s'adresse peut-être à des personnes qui ne sont copropriétaires
de rien, mais `consent` et `consumption` touchent à un consentement RGPD.

#### R23 — cinq rôles sur quatorze n'ont aucune navigation (#814)

Trouvé en cadrant les personas prestataire et conseil de copropriété.
`canSee()` reconnaît six noms de rôle ; le serveur en sérialise quatorze.
`board_member`, `contractor`, `accountant.encodeur` et `accountant.emetteur`
tombent en fail-closed — et `community.moderator` aussi, parce que l'interface
compare à `community-moderator`, avec un trait d'union là où le serveur écrit
un point.

Les huit blocs de menu de `Navigation.svelte` rendent alors `false`, et le
message de secours ne se déclenche pas : il teste l'**absence** de rôle, or ces
comptes en ont un. Ils reçoivent une barre avec un logo et un bouton de
déconnexion.

Ces rôles ne sont pas décoratifs : neuf routes `/board-decisions`, dix
`/board-members`, vingt-et-une `/tickets`, quinze `/contractor-reports`, et les
écrans qui vont avec. **Sixième occurrence du motif dominant** — écrit, testé,
inatteignable.

**Bloque D1e, D1f et D2d** : deux de leurs étapes ne peuvent pas être jouées au
navigateur tant que la barre est vide.

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

#### R24 — deux énumérations, un seul nom de schéma (#819)

Trouvé en réparant la CI. `payment_method.rs` et `payment.rs` déclarent chacun
un `PaymentMethodType`, l'un à deux variantes, l'autre à quatre. utoipa ne
publie qu'un schéma par nom : le contrat annonçait `["card", "sepa_debit"]`
partout, **y compris pour le corps de `CreatePaymentRequest`**. Sur le papier,
aucun client engendré depuis le contrat ne pouvait enregistrer un paiement en
espèces ni par virement manuel.

Une fois le contrat rendu honnête, `svelte-check` a sorti cinq erreurs qu'il ne
pouvait pas voir : la modale d'ajout offrait « Virement » et « Espèces », deux
options que le serveur rejette en 400 depuis toujours, avec leurs textes d'aide
traduits en quatre langues. **Septième occurrence du motif dominant.**

**R1, R4, R14, R15 et R19 sont les plus bloquants.** R1 et R19 exposent des données. R14
rend visible tout le reste. R15 empêche de créer un immeuble. R4 fait bannir les testeurs.

### Track U — Refonte UX/UI (revue Claude Design du 2026-09-06)

Entrée au périmètre 0.1.0 sur décision du 06. Dix-huit lots, dont l'ordre est
contraint : U2 dépend de R1, et tout le reste dépend de U1.

**Neuf maquettes Claude Design fournissent la référence visuelle.** Les lots
U1 à U6 décrivent ce qu'il faut changer ; les lots U0 apportent à quoi cela
doit ressembler. Chacun se lance depuis une session où `/design-login` est
possible — le connecteur `claude_design` demande une authentification
interactive, qu'une exécution automatisée ne peut pas mener.

**L'ordre entre elles n'est pas libre.** U0a pose la coquille et les jetons ;
U0g pose le socle mobile ; U0b, U0c, U0d et U0f en déclinent les rôles ; U0e
touche la barre latérale, donc le contrôle d'accès, et attend U2 — elle-même
bloquée par R1 (#772). C'est la chaîne de dépendances la plus longue du
périmètre, et la remonter à l'envers reviendrait à redessiner un périmètre
applicatif qu'on n'a pas fini de cloisonner.

| Lot | Contenu | Issue | Dépend de |
|---|---|---|---|
| U0a | **Maquette « Koprogo Review »** — la refonte d'ensemble | #818 | #803 |
| U0b | **Maquette « Roles Mobile First »** — ce que voient le prestataire et le conseil (#815, #816), et le passage au mobile-first (#795) | #820 | #803, U0a |
| U0c | **Maquette « Admin Dashboard (modernisé) »** — l'écran où la revue a trouvé le plus à reprendre : jeton en console (#787), activité inventée à Paris et Lyon (#791) | #821 | #803, U0a |
| U0d | **Maquette « Accountant Dashboard (modernisé) »** — l'écran dont les transactions s'affichent sans couleur depuis toujours, classes Tailwind assemblées à l'exécution (#788) | #822 | #803, U0a |
| U0e | **Maquette « KoproSidebarV2 »** — la barre latérale, c'est-à-dire l'endroit exact où se décide qui voit quoi (#814, #797, #794) | #823 | #803, U0a, **U2** |
| U0f | **Maquette « Owner Mobile First »** — le rôle le plus nombreux, et le seul que six recettes n'ont jamais éprouvé (#807, #789) | #824 | #803, U0a, U0g |
| U0g | **Maquette « Mobile First (modernisé) »** — le socle mobile commun : points de rupture, piège de focus, dialogues natifs, PWA (#795, #794, #785, #804) | #825 | #803, U0a |
| U0h | **Maquette « Syndic Dashboard (modernisé) »** — le rôle central, et le seul endroit où le produit s'arrête net : le cycle de vie d'une AG (#780, #806) | #826 | #803, U0a |
| U0i | **Maquette « Lists (modernisé) »** — l'écran le plus répété du produit, et six défauts d'affichage y logent (#786, #792, #793, #775). **Recouvre U4 (#800)** | #827 | #803, U0a |
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
| D0 | Cadre d'ensemble : en-têtes de persona, registre de règles, cliquet de couverture | #805 |
| D1a | **Syndic** — 15 étapes, du premier écran à la clôture d'une assemblée | #806 |
| D1b | **Copropriétaire** — 10 étapes. Le rôle le plus nombreux en base, et le seul que cinq recettes n'ont jamais éprouvé | #807 |
| D1c | **Comptable** — 10 étapes, dont le cas dégradé de l'immeuble non conforme | #808 |
| D1d | **Administrateur** — 7 étapes, et surtout ce qu'il ne doit pas pouvoir faire | #809 |
| D1e | **Prestataire** — 12 étapes, du ticket reçu au rapport validé qui déclenche le paiement. Le seul persona extérieur à la copropriété | #815 |
| D1f | **Conseil de copropriété** — 11 étapes. Organe de surveillance (Art. 3.90 § 1er) : lecture large, presque aucune écriture | #816 |
| D2a | **Cycle de vie d'une AG** — 12 étapes, 3 rôles, 6 articles du Code civil. Le parcours qui porte le risque juridique | #810 |
| D2b | **Circuit d'une facture** — 11 étapes, du fournisseur au copropriétaire qui paie | #811 |
| D2c | **Naissance d'une copropriété** — 10 étapes, de l'organisation à la première connexion d'un copropriétaire | #812 |
| D2d | **Le ticket jusqu'au paiement** — 14 étapes, 4 rôles. Le seul parcours qui boucle : celui qui signale est celui qui paie | #817 |
| D3 | **Restructurer les 100 specs e2e par persona** pour que les vidéos racontent le produit | #813 |

**Ce que KoproGo a déjà, et ce qui manque.** Le dépôt compte 86 documents dans
`docs/`, et ils sont bons — PCMN belge, RGPD, convocations, workflow de
facture, gouvernance. Mais ils sont classés **par sujet technique ou
réglementaire, jamais par personne**. Aucun ne répond à la question qu'un
syndic se pose en arrivant : que puis-je faire, dans quel ordre, et pourquoi.

**Pourquoi c'est en 0.1.0.** Même raison que le reste : un financeur ne lit pas
du code. Le produit doit être compréhensible avant la fondation de l'ASBL.

**Ce que « vivante » veut dire ici.** Une documentation qui se met à jour parce
qu'un test la garde, pas parce qu'on y pense. Le dépôt a déjà les deux
mécanismes : le cliquet, employé six fois, et le contrat `data-testid` figé
(#802, #803) — un parcours documenté dont une étape n'a pas d'ancrage est un
parcours qu'on ne peut pas prouver.

**Les vidéos existent déjà, mal rangées.** `playwright.config.ts` enregistre
toutes les exécutions en 1280×720, sous le commentaire « DOCUMENTATION
VIVANTE ! », `generate-video-rst.py` construit une galerie et `docs.yml` la
publie. Mais les **cent specs sont organisées par module**, donc par sujet
technique : une vidéo de `Convocations.spec.ts` montre qu'une convocation se
crée, jamais pourquoi ni qui la reçoit. Quinze vidéos numérotées suivant un
syndic du premier écran à la clôture racontent le produit ; cent vidéos par
module ne racontent rien.

**D3 vient après U et après #803**, et l'ordre n'est pas négociable : filmer
des écrans qui vont être refaits produit une documentation périmée le jour de
sa livraison, et restructurer des tests avant que les ancrages existent oblige
à cibler par texte ou par position — ce qui a fait manquer des cibles à la
recette pendant deux sessions.

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

## Inventaire complet du périmètre 0.1.0

**88 issues ouvertes** portent l'étiquette `release:0.1.0`. Elles sont
toutes ci-dessous, sans exception : une issue du périmètre absente du WBS est
une issue que personne ne planifie.

Le tableau est engendré depuis GitHub par `scripts/inventaire-wbs.py`, et un
test refuse qu'une issue du périmètre n'y figure pas. Il ne peut donc pas
se désynchroniser en silence, comme l'a fait `docs/api/openapi.json` pendant
cinq jours.

| Priorité | Nombre |
|---|---|
| critical | 7 |
| high | 33 |
| medium | 23 |
| low | 2 |
| — | 23 |

### Track R — Défauts de recette navigateur (24)

Six recettes menées au navigateur entre le 2026-09-04 et le 2026-09-06. Le
motif dominant, confirmé six fois : **une capacité écrite, testée, et
inatteignable**. Nos tests prouvent que le code marche tout en masquant qu'on
ne peut pas y arriver.

| Issue | Prio | Intitulé |
|---|---|---|
| #770 | critical | total_units est déclaré à la création et jamais recalculé : ajouter un lot rend l'immeuble non conf… |
| #772 | critical | Fuite inter-organisations : 75 routes imbriquées sur 310 n'exigent aucune identité — des votes nomi… |
| #780 | critical | Le cycle de vie d'une AG ne peut pas aboutir : trois verrous indépendants, aucun contournable depui… |
| #814 | critical | Cinq rôles sur quatorze reçoivent une navigation entièrement vide : canSee() les fait tomber en fai… |
| #765 | high | Contrat : les 17 routes /expenses et /invoices sont hors OpenAPI — c'est ce qui a laissé line_items… |
| #771 | high | Surface publique : la page d'inscription existe mais n'est liée nulle part, et « mot de passe oubli… |
| #774 | high | Page RGPD majoritairement en anglais : le copropriétaire lit ses droits et déclenche un effacement … |
| #775 | high | Annonces : création sans effet ni retour, filtre Brouillon en chargement infini, et énumérations br… |
| #776 | high | Le bouton « Clôturer le vote » est présent et sans effet : le cycle de vie d'une AG ne peut pas s'a… |
| #778 | high | Bouton de création d'immeuble et de lot invisibles pour le syndic : la route serveur est ouverte, l… |
| #779 | high | Rebrancher les six modules communautaires : 111 points d'entrée servis que le frontend n'appelle pa… |
| #784 | high | L'envoi de convocation exige recipient_owner_ids que l'interface ne peut pas constituer |
| #788 | high | Classes Tailwind interpolées dans le tableau de bord comptable : les styles ne sont jamais générés |
| #789 | high | La grille à cinq colonnes des membres du conseil n'existe jamais : classe Tailwind interpolée |
| #819 | high | Deux énumérations portent le même nom de schéma : le contrat interdit d'enregistrer un paiement en … |
| #777 | medium | Le test negative_display_does_not_leak_business_internals échoue au hasard : un UUID aléatoire cont… |
| #786 | medium | Le type Vote du frontend ne correspond pas au DTO servi : colonnes CHOIX et DATE vides, et clé noti… |
| #790 | medium | tailwind.config.mjs n'est jamais chargé et annonce une couleur de marque qui n'existe plus |
| #791 | medium | Le tableau de bord admin affiche une activité récente inventée, située à Paris et à Lyon |
| #792 | medium | Les statuts de tickets s'affichent en valeurs internes : Open, InProgress, Resolved, Closed |
| #794 | medium | Le tiroir de navigation mobile n'a pas de piège de focus, et son overlay est un div déguisé en bout… |
| #804 | medium | La PWA n'a jamais fonctionné : le service worker échoue à l'installation depuis novembre 2025 |
| #793 | low | « Precedent » sans accent dans la pagination, y compris dans le libellé lu par les lecteurs d'écran |
| #796 | low | La balise theme-color annonce un vert que l'application n'utilise plus |

### Track U — Refonte UX/UI (14)

Revue de design du 2026-09-06. Entrée en 0.1.0 le même jour : un financeur ne
lit pas du code, et l'ASBL se fonde sur ce que le produit montre. **U2 ne
commence qu'une fois #772 fermée** — remanier le périmètre applicatif avant
d'avoir fermé la dette de cloisonnement serait l'ordre inverse du bon.

| Issue | Prio | Intitulé |
|---|---|---|
| #556 | high | [EPIC] Refonte UX multi-rôle + modèle ACP — pipeline Maury (39 stories, 7 slices) |
| #798 | high | Refonte UX — le périmètre devient l'ACP dans le modèle de données et permissions.ts |
| #802 | high | Refonte UX — adapter les tests sans en supprimer les règles produit qu'ils encodent |
| #803 | high | Poser des data-testid là où il n'y en a pas : 29 % de couverture avant une refonte qui va tout dépl… |
| #818 | high | Refonte UX — importer et implémenter la maquette Claude Design « Koprogo Review » |
| #820 | high | Refonte UX — importer et implémenter la maquette Claude Design « Roles Mobile First » |
| #823 | high | Refonte UX — importer et implémenter la maquette Claude Design « KoproSidebarV2 » |
| #824 | high | Refonte UX — importer et implémenter la maquette Claude Design « Owner Mobile First » |
| #825 | high | Refonte UX — importer et implémenter la maquette Claude Design « Mobile First (modernisé) » |
| #826 | high | Refonte UX — importer et implémenter la maquette Claude Design « Syndic Dashboard (modernisé) » |
| #797 | medium | Refonte UX — jetons de design et jeu d'icônes SVG : remplacer les émojis qui collisionnent |
| #821 | medium | Refonte UX — importer et implémenter la maquette Claude Design « Admin Dashboard (modernisé) » |
| #822 | medium | Refonte UX — importer et implémenter la maquette Claude Design « Accountant Dashboard (modernisé) » |
| #827 | medium | Refonte UX — importer et implémenter la maquette Claude Design « Lists (modernisé) » |

### Track D — Documentation vivante multi-persona (13)

Six personas, quatre workflows transverses, et la restructuration des cent
specs e2e pour que les vidéos racontent le produit plutôt que ses modules.
**Tout le track vient après #803 et après Track U** : filmer des écrans qui
vont changer produit une documentation périmée le jour de sa livraison.

| Issue | Prio | Intitulé |
|---|---|---|
| #805 | high | Documentation vivante multi-persona : expliquer le logiciel par rôle, sur le modèle de klaar |
| #806 | high | Documentation vivante — parcours du SYNDIC, de la première connexion à la clôture d'une assemblée |
| #807 | high | Documentation vivante — parcours du COPROPRIÉTAIRE, le rôle que cinq recettes n'ont jamais éprouvé |
| #810 | high | Workflow multi-persona — le cycle de vie d'une assemblée générale, du syndic au copropriétaire |
| #808 | medium | Documentation vivante — parcours du COMPTABLE, le rôle le mieux cadré du produit |
| #809 | medium | Documentation vivante — parcours de l'ADMINISTRATEUR : ce qu'il crée, et ce qu'il ne doit pas pouvo… |
| #811 | medium | Workflow multi-persona — le circuit d'une facture, du fournisseur au copropriétaire qui la paie |
| #812 | medium | Workflow multi-persona — la naissance d'une copropriété, de l'administrateur au premier copropriéta… |
| #813 | medium | Documentation vivante — restructurer les 100 tests e2e par persona pour que les vidéos racontent le… |
| #815 | medium | Documentation vivante — parcours du prestataire : du ticket reçu au rapport d'intervention validé |
| #816 | medium | Documentation vivante — parcours du conseil de copropriété : surveiller le syndic, valider les trav… |
| #817 | medium | Workflow multi-persona — le ticket, du copropriétaire qui signale au prestataire qui est payé |
| #595 | — | [Story Tx.3] Documentation docs/agent-activity/ (Tier 2 log) |

### Track M — Modularité par ACP et RBAC communautaire (10)

Slice 5 de l'épopée #556. Une ACP active les modules dont elle a besoin ; le
reste répond 403, pas 404. Ce track porte aussi les deux arbitrages ouverts
sur les droits communautaires — le syndic peut-il réserver au nom de l'ACP
(#781, #588), et le comptable doit-il être exclu du communautaire (#589).

| Issue | Prio | Intitulé |
|---|---|---|
| #781 | medium | Les modules communautaires supposent que l'utilisateur est copropriétaire : le syndic ne peut rien … |
| #585 | — | [Story 5.1] Table acp_enabled_modules + ModuleGuard middleware + ModuleDisabledError |
| #586 | — | [Story 5.2] UI ModuleGate.svelte + store enabled_modules |
| #587 | — | [Story 5.3] Syndic = community.moderator (RBAC Community SEL/Poll/Notice/SharedObject) |
| #588 | — | [Story 5.4] Reservation.on_behalf_of_acp (exception syndic) |
| #589 | — | [Story 5.5] Comptable (encodeur ET émetteur) 403 sur /community/* |
| #590 | — | [Story 5.6] Activation/désactivation modules audité + archivage data (jamais delete) |
| #591 | — | [Story 5.7] Onboarding modulaire wizard ≤ 5 min |
| #592 | — | [Story 5.8] Gate CI a11y axe-core + data-testid + Lighthouse |
| #694 | — | Scoping user↔ACP absent : un syndic/comptable voit toute l'organisation, pas seulement ses ACPs |

### Track S — Gouvernance d'assemblée avancée (7)

Slice 4 de #556 : assemblée hybride, vote à distance authentifié fort,
procès-verbal signé eIDAS, conseil de copropriété élu, commissaire aux
comptes. C'est le track dont dépend la crédibilité juridique du produit
au-delà du strict Art. 3.87.

| Issue | Prio | Intitulé |
|---|---|---|
| #576 | — | [Story 4.1] [cluster-coord] Meeting.mode hybrid + quorum agrégé Decimal |
| #577 | — | [Story 4.2] Vote distant auth_method strong (itsme/eID) — closes #48 |
| #578 | — | [Story 4.3] Minutes (PV) + 2 signatures eIDAS qualifiées |
| #579 | — | [Story 4.4] Adapter ElectronicSignatureProvider (port + 3 adapters eID/itsme/Universign) |
| #581 | — | [Story 4.6] Résolution EvaluationContractors AGO auto non retirable |
| #582 | — | [Story 4.7] CdC membre élu + action create_alert |
| #583 | — | [Story 4.8] [cluster-coord] CommissaireAuxComptes + VerificationCertificate |

### Track T — Dette d'infrastructure de test (4)

Ce qui empêche la CI de dire la vérité. **Quatre jobs sur dix sont rouges en
continu depuis le 2026-09-04 au moins** : `prettier`, le contrat OpenAPI,
`oasdiff` et la suite BDD. Une CI rouge en permanence ne garde rien — elle
apprend seulement à ne plus la regarder.

| Issue | Prio | Intitulé |
|---|---|---|
| #540 | high | bug(test-infra): inventaire consolidé des ~27 scénarios BDD pré-existants rouges (révélés post-#524) |
| #548 | high | bug(e2e): WP-D1/FE1 — ripple Playwright (59 specs) après JWT→cookie : auth.ts init-ordering 'Databa… |
| #443 | medium | BDD-MIGRATION-001: Finalize Decimal cascade in BDD/E2E tests (~50 errors residual) |
| #696 | — | Instabilité smoke suite Playwright CI : 109 échecs sur specs pré-existantes (occurrence 2026-08-08) |

### Track K — Dette de code et de contrat (3)

Les erreurs typées plutôt que classées par sous-chaîne, le contrat OpenAPI
complet, et la suppression du repli qui fabrique une ACP inexistante.

| Issue | Prio | Intitulé |
|---|---|---|
| #555 | medium | EPIC: migrer Result<_, String> → Result<_, AppError> (1263 violations, CRITICAL.md rule 4) |
| #761 | — | Supprimer le repli de resolve_acp_id, qui fabrique un ACP inexistant |
| #762 | — | Typer les erreurs applicatives au lieu de les classer par sous-chaînes |

### Track F — Ops et infrastructure (10)

Sauvegardes, TLS, GitOps, et les vulnérabilités de dépendances. F3 a été joué
le 2026-09-04 et son résultat est **négatif sur deux volets sur trois** : le
rollback échoue dès qu'une version a migré, et les sauvegardes GPG+S3 du
runbook n'existent pas sur la machine.

| Issue | Prio | Intitulé |
|---|---|---|
| #425 | critical | Méta — Garde-fous IA: audit qualité+sécurité IaC, cause racine, plan de remédiation |
| #429 | critical | Méta — Operations runtime: deploy IaC en prod + agents DevOps/SRE/Support/CSI (Tier 1 humain / Tier… |
| #354 | high | refactor(infra): Tests IaC manquants — terraform validate, ansible-lint, molecule, conftest ISO 270… |
| #355 | high | refactor(infra): Restructuration IaC — repo séparé, tests, policy-as-code |
| #432 | high | Security — 14 dependabot vulnerabilities sur main (5 high / 3 moderate / 6 low) |
| #515 | high | infra: ArgoCD GitOps fresh-cluster deployment fails on 5 gaps (dry-run Docker Desktop 2026-05-12) |
| #731 | high | Collision d'alias DNS sur le réseau partagé ecosolva-web : 4 projets exposent tous `backend` et `fr… |
| #453 | medium | Pipeline TLS dispatch dev/integration/staging via OVH DNS-01 |
| #466 | — | RFC: Stratégie GitOps multi-environnement — branches infra/* + main + ApplicationSet refactor |
| #718 | — | [BUG] 502 Bad Gateway / timeouts sur api.koprogo.com sous rafale de requêtes (constaté via run E2E … |

### Track G — Gate humain et gouvernance documentaire (2)

Les deux actes non délégables — la revue humaine et la pose du tag — et ce
qui les prépare : la taxonomie des tests comme gate de release, et le
désencombrement de la documentation.

| Issue | Prio | Intitulé |
|---|---|---|
| #427 | critical | Validation — taxonomie tests 4 catégories + revue humaine+Cowork comme gate release |
| #426 | high | Doc — Archiver binaires root, dédoublonner docs/, désinfler CLAUDE.md, mettre à jour .claude/ |

### Track ? — À arbitrer — présence en 0.1.0 douteuse (1)

Une issue dont l'étiquette et le titre se contredisent. Il faut trancher, pas
laisser le doute dans le périmètre.

| Issue | Prio | Intitulé |
|---|---|---|
| #635 | — | Fonds affectés / thésaurisation : entité Fund dédiée aux travaux d'ampleur (v0.2.0) |

## Recouvrements et issues caduques (revue du 2026-09-06)

Le périmètre a doublé en une journée, et neuf lots de maquette sont venus
s'ajouter à des lots qui décrivaient déjà le même travail. Une revue des 106
issues a cherché les doublons. **Rien n'est fermé ici** : fermer une issue est
un acte visible sur un dépôt public, et onze d'un coup — dont deux qui
corrigent mes propres erreurs de diagnostic — appelle un accord humain.

### Recouvrements introduits par les maquettes

| Lot d'intention | Recouvert par | Nature |
|---|---|---|
| #799 — coquille : barre latérale, barre de contexte, navigation mobile | #823 (sidebar), #825 (mobile) | quasi-total |
| #801 — les tableaux de bord présentent des tâches | #821, #822, #824, #826 | total, réparti par rôle |
| #795 — desktop-first | #825, #820, #824 | total |
| #800 — motif de liste unifié et encadré légal | #827 | total |

Ce ne sont pas des doublons au sens strict : les premiers décrivent
l'intention, les maquettes la forme. Mais deux issues ouvertes pour le même
travail, c'est deux fois le risque qu'on croie l'autre faite.

**Voie proposée** : garder les maquettes comme lots d'exécution, et fermer les
lots d'intention en y renvoyant — chacun est déjà cité comme critère
d'acceptation dans la maquette correspondante, donc aucune règle ne se perd.

### Issues rendues caduques par le travail, vérifiées

| Issue | Pourquoi elle tombe |
|---|---|
| #768 — « 8 modules communautaires sans backend » | **Diagnostic faux**, posé puis corrigé par moi. Ils ont tous un backend : 111 points d'entrée servis. C'est un désaccord de chemin, et **#779** le dit correctement. |
| #723 — spec `02-ag-full-cycle` rouge | Corrigé : la ligne 70 envoie `acp_id`, et le gate Characterization est vert sur quatre exécutions. |
| #763 — identifiants superadmin déductibles | Résolu : la rotation s'est appliquée au déploiement de `sha-6e6ac56`. |
| #766 — « crash API » et « régression CORS » | Résolu le 2026-09-02 par l'exception CrowdSec. |
| #785 — bouton « Reporter », bundle périmé | **Suspicion infirmée** : la PWA n'a jamais fonctionné, donc aucun cache, donc aucun bundle périmé possible. Le bouton marche — un `prompt()` réclame `accept(value)`. Le reste est dans #825. |

### Doublons anciens

| Grappe | Verdict |
|---|---|
| #331 (48 specs, mars) contre #813 (restructurer les 100 specs par persona) | #331 décrit un état de mars qui n'existe plus. |
| #343 (« 158 composants sur 178 sans data-testid », mars) contre #803 | Même sujet, chiffres périmés. #803 le mesure et le garde par cliquet. |
| #663 (BuildingSelector → ACP) contre #798 (le périmètre devient l'ACP) | #663 est un sous-ensemble strict. À rattacher. |
| #587, #588 (stories 5.3 et 5.4) contre #781 | Même problème : #781 est le symptôme observé, #587/#588 la solution déjà conçue et signée. Un des trois arbitrages ouverts. |

### Ce qui n'est pas un doublon malgré les apparences

**#780 contient #776** : relation parent-enfant. #776 est fixée et déployée ;
#780 reste ouverte pour son critère de fin — un cycle d'AG complet mené au
navigateur, jamais réussi à ce jour.

**#694 et #772** : le premier dit qu'un syndic voit *toutes* les ACP de son
cabinet, le second qu'il voit celles des *autres* cabinets. Granularité contre
cloisonnement.

**#443, #540, #548, #696** : quatre issues de dette de tests, de mars à août.
Probablement caduques en partie — mais **aucune n'a été mesurée**, et je ne les
déclarerai pas mortes sans mesure.

### Le compte

**106 issues au relevé ; 88 après les dix-huit fermetures du 2026-09-06.**

Douze recouvrements ou caducités, validés par le porteur du projet, et six
défauts critiques désormais corrigés, déployés et **gardés par un test
vérifié par témoin** — #767, #769, #773, #782, #783, #787.

Deux de ces six n'avaient aucun filet avant leur fermeture : le cookie de
session reposé par `register` (#769) et les majorités comptées par tête
(#773). Ils en ont un maintenant, et fermer sans filet aurait seulement
déplacé la date de leur retour.

## Ce qui reste, et ce qui le bloque

### ⚠️ Le périmètre a doublé le 2026-09-06, et c'est une décision assumée

**Tout ce qui restait en 0.2.0 entre en 0.1.0, avec la refonte UX/UI.** Le compte passe de
**34 à 88 issues ouvertes** en `release:0.1.0` : les 31 ouvertes le 2026-09-06, les 24
qui étaient en 0.2.0, les 6 lots de la refonte, la documentation vivante et les défauts
trouvés en vérifiant. Il ne reste plus rien en 0.2.0.

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
