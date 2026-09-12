---
livrable: Epics & User Stories (BMAD phase E — TOGAF Solutions)
projet: KoproGo
jalon: v0.1.0
genere_par: scripts/backlog-structure.py
signature_humaine:
  date: null
  nom: null
  role: null
  etat: NON SIGNÉ — en attente de validation du superviseur
---

# Backlog structuré — v0.1.0

*Épopées et capacités. Généré par `scripts/backlog-structure.py` : ne pas
éditer à la main, la prochaine génération écraserait la correction.*

**85 issues ouvertes**, 10 épopées, 32 capacités. Classement exhaustif et exclusif : chaque issue appartient à exactement
une capacité, et une issue non classée fait échouer la génération.

## Comment lire ce document

Le WBS classe par **provenance** — d'où vient le ticket. Celui-ci classe
par **capacité** — ce que le produit saura faire quand elle sera tenue.
Les deux servent : le WBS dit le périmètre, celui-ci dit l'ordre et le
coût.

Les épopées de domaine ne sont pas inventées ici : ce sont les **quatre**
**contextes bornés** que `backend/tests/architecture.rs` déclare et dont
il interdit les dépendances croisées. Le backlog dit donc ce que le code
dit déjà.

**Must / Should / Could ordonne, et ne retire rien.** La décision du
2026-09-06 a mis les 84 issues au périmètre du tag ; seul le superviseur
peut la défaire. Un `Could` ici veut dire « en dernier », pas « hors
release ».

Une capacité est **tenue** quand ses issues sont fermées *et* qu'un test
la traverse de bout en bout. Une fonctionnalité codée n'est pas une
capacité disponible.

## Préparation des stories

Une capacité **structurée** n'est pas une capacité **prête**. La Méthode
Foyer pose huit éléments sans lesquels une story n'entre pas en
fabrication ; le Scrum Master de conception l'écrit sans détour :
« aucune story n'est prête sans elles ».

| Élément | Ce qu'il pré-engage |
|---|---|
| Récit *En tant que… je veux… afin de…* | à qui ça sert, donc ce qu'on peut retirer |
| Critères Gherkin | le critère, **avant** la génération |
| `@happy` | le chemin nominal |
| `@negative` | les entrées invalides et les échecs attendus |
| `@edge` | les bornes : vide, max, concurrence |
| `@security` | abus, injection, autorisation |
| Couche(s) | où le code atterrit, donc quelles gardes s'appliquent |
| Taille + tours | le coût, sur les deux axes |

**6 issues sur 85** portent les huit. Le relevé
d'origine, avant ce travail, donnait **zéro**.

Le chiffre n'est pas écrit à la main : `scripts/backlog-pret.py` le relève
à chaque exécution, et cette page est générée. Un taux de préparation
recopié vieillit en trois jours sans que personne s'en aperçoive — c'est
arrivé à la section « Ordre d'exécution » du WBS, dont les quatre
premières étapes désignent des issues toutes fermées.

Le script cherche des marqueurs de **forme**, pas du sens : une issue qui
écrit `@security` au-dessus d'un critère creux est comptée. C'est une
**borne haute**, jamais un verdict.

### Le neuvième élément, propre à ce dépôt : le témoin

Un test écrit après coup peut ne rien garder. Une assertion du banc
mobile comparait `document.scrollWidth` à `window.innerWidth`, deux
valeurs qui grandissent ensemble : elle **ne pouvait pas échouer**, et
cachait un vrai débordement. Toute story livrée ici porte donc : **le
défaut est remis, et le test doit échouer.**

### La règle de préparation

Une story est mise au gabarit **quand elle devient la prochaine étape**,
pas trois semaines avant. Préparer les 84 d'un coup serait une correction
de masse — ce que la méthode range parmi les gestes qui retirent à
l'humain les moyens d'assumer — et produirait 84 stories creuses.

### Sprint 0 : la story habilitante est fermée

BMAD pose que pour un archétype *full-stack*, le Sprint 0 inclut
**obligatoirement** le harnais de contrat API, et que sur un projet
existant qui en manque, c'est une story de correction structurelle qui
**bloque le reste du backlog**.

Vérifié : #765 est fermée. Les dix-sept routes `/expenses` et `/invoices`
hors contrat OpenAPI — celles qui avaient laissé `line_items` se perdre en
silence — y sont. Le backlog n'est pas bloqué à ce titre.

## E1 — Copropriété — le jumeau juridique

`epic:copropriete` · 14 issues · 13.00 j · 52 tours

Le contexte borné qui ne dépend de rien : ce que la loi belge dit d'une assemblée, d'un lot, d'une voix. Une erreur ici n'est pas un défaut d'affichage, c'est une décision annulable.

### C1.1 — Une assemblée générale aboutit, de la convocation au PV

**Must** · `cap:C1.1` · 7 issues · 6.50 j · 26 tours · **0/7 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#576](https://github.com/gilmry/koprogo/issues/576) | [Story 4.1] [cluster-coord] Meeting.mode hybrid + quorum agrégé Decimal | L | manque 4/8 |
| [#577](https://github.com/gilmry/koprogo/issues/577) | [Story 4.2] Vote distant auth_method strong (itsme/eID) — closes #48 | L | manque 4/8 |
| [#581](https://github.com/gilmry/koprogo/issues/581) | [Story 4.6] Résolution EvaluationContractors AGO auto non retirable | M | manque 4/8 |
| [#780](https://github.com/gilmry/koprogo/issues/780) | Le cycle de vie d'une AG ne peut pas aboutir : trois verrous indépendants, au… | L | manque 8/8 |
| [#840](https://github.com/gilmry/koprogo/issues/840) | Une résolution sans point d'ordre du jour est acceptée, alors que la loi la r… | M | manque 8/8 |
| [#848](https://github.com/gilmry/koprogo/issues/848) | Un lot détenu à deux ne peut jamais voter : la suspension de l'Art. 3.87 § 1e… | L | manque 8/8 |
| [#850](https://github.com/gilmry/koprogo/issues/850) | Le vote en assemblée accepte l'identité du votant, son lot et sa puissance de… | L | manque 7/8 |

### C1.2 — Le procès-verbal fait foi

**Should** · `cap:C1.2` · 2 issues · 2.00 j · 8 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#578](https://github.com/gilmry/koprogo/issues/578) | [Story 4.3] Minutes (PV) + 2 signatures eIDAS qualifiées | L | manque 4/8 |
| [#579](https://github.com/gilmry/koprogo/issues/579) | [Story 4.4] Adapter ElectronicSignatureProvider (port + 3 adapters eID/itsme/… | L | manque 4/8 |

### C1.3 — Le registre légal atteste ce qu'il déclare

**Must** · `cap:C1.3` · 2 issues · 1.75 j · 7 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#846](https://github.com/gilmry/koprogo/issues/846) | Onze tables et une vue existent en base et ne sont lues par aucun code, dont … | M | manque 8/8 |
| [#847](https://github.com/gilmry/koprogo/issues/847) | Le registre légal atteste des obligations que ses tests ne vérifient pas : l'… | L | manque 8/8 |

### C1.4 — Les organes de contrôle existent

**Could** · `cap:C1.4` · 2 issues · 1.75 j · 7 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#582](https://github.com/gilmry/koprogo/issues/582) | [Story 4.7] CdC membre élu + action create_alert | M | manque 4/8 |
| [#583](https://github.com/gilmry/koprogo/issues/583) | [Story 4.8] [cluster-coord] CommissaireAuxComptes + VerificationCertificate | L | manque 4/8 |

### C1.5 — L'état daté a un destinataire identifié

**Should** · `cap:C1.5` · 1 issues · 1.00 j · 4 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#855](https://github.com/gilmry/koprogo/issues/855) | Le notaire n'a pas d'identité : l'état daté est servi à qui connaît la référe… | L | manque 8/8 |

## E2 — Comptabilité — la charge et sa répartition

`epic:comptabilite` · 3 issues · 2.50 j · 10 tours

Connaît `copropriete` : une charge se répartit sur des quotités, elle ne peut pas les ignorer. L'inverse est faux.

### C2.1 — Une quote-part se saisit sans ambiguïté

**Should** · `cap:C2.1` · 1 issues · 0.50 j · 2 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#852](https://github.com/gilmry/koprogo/issues/852) | Le champ unit_id est déclaré optionnel sur la création d'une quote-part, et r… | S | manque 8/8 |

### C2.2 — Un copropriétaire multi-ACP voit ses montants séparés

**Should** · `cap:C2.2` · 1 issues · 1.00 j · 4 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#867](https://github.com/gilmry/koprogo/issues/867) | Un copropriétaire multi-ACP voit un montant unique : un virement groupé serai… | L | manque 7/8 |

### C2.3 — Les fonds affectés sont une entité, pas une convention

**Could** · `cap:C2.3` · 1 issues · 1.00 j · 4 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#635](https://github.com/gilmry/koprogo/issues/635) | Fonds affectés / thésaurisation : entité Fund dédiée aux travaux d'ampleur (v… | L | manque 8/8 |

## E3 — Économie circulaire — les modules communautaires

`epic:economie-circulaire` · 9 issues · 7.25 j · 29 tours

Connaît `copropriete`. SEL, sondages, objets partagés, énergie : la partie du produit qui distingue KoproGo d'un logiciel de syndic.

### C3.1 — Les modules communautaires sont atteignables

**Should** · `cap:C3.1` · 2 issues · 1.75 j · 7 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#779](https://github.com/gilmry/koprogo/issues/779) | Rebrancher les six modules communautaires : 111 points d'entrée servis que le… | L | manque 8/8 |
| [#781](https://github.com/gilmry/koprogo/issues/781) | Les modules communautaires supposent que l'utilisateur est copropriétaire : l… | M | manque 8/8 |

### C3.2 — Le syndic a un rôle dans la communauté

**Could** · `cap:C3.2` · 3 issues · 2.00 j · 8 tours · **0/3 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#587](https://github.com/gilmry/koprogo/issues/587) | [Story 5.3] Syndic = community.moderator (RBAC Community SEL/Poll/Notice/Shar… | M | manque 4/8 |
| [#588](https://github.com/gilmry/koprogo/issues/588) | [Story 5.4] Reservation.on_behalf_of_acp (exception syndic) | M | manque 4/8 |
| [#589](https://github.com/gilmry/koprogo/issues/589) | [Story 5.5] Comptable (encodeur ET émetteur) 403 sur /community/* | S | manque 3/8 |

### C3.3 — Une ACP active les modules qu'elle veut

**Could** · `cap:C3.3` · 4 issues · 3.50 j · 14 tours · **0/4 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#585](https://github.com/gilmry/koprogo/issues/585) | [Story 5.1] Table acp_enabled_modules + ModuleGuard middleware + ModuleDisabl… | L | manque 4/8 |
| [#586](https://github.com/gilmry/koprogo/issues/586) | [Story 5.2] UI ModuleGate.svelte + store enabled_modules | M | manque 4/8 |
| [#590](https://github.com/gilmry/koprogo/issues/590) | [Story 5.6] Activation/désactivation modules audité + archivage data (jamais … | M | manque 4/8 |
| [#591](https://github.com/gilmry/koprogo/issues/591) | [Story 5.7] Onboarding modulaire wizard ≤ 5 min | L | manque 4/8 |

## E4 — Plateforme — identité, périmètre, droits

`epic:plateforme` · 10 issues · 9.00 j · 36 tours

Ne dépend de rien. Qui est l'appelant, ce qu'il a le droit de voir, et ce qu'il peut exiger qu'on efface.

### C4.1 — Toute route décide de l'identité qu'elle reçoit

**Must** · `cap:C4.1` · 2 issues · 2.00 j · 8 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#845](https://github.com/gilmry/koprogo/issues/845) | Trente routes ne vérifient aucune identité : modifier ou supprimer une assemb… | L | manque 8/8 |
| [#864](https://github.com/gilmry/koprogo/issues/864) | 87 routes prennent une identité sans s'en servir pour décider : supprimer le … | L | manque 8/8 |

### C4.2 — Le périmètre est l'ACP, et il survit à la navigation

**Must** · `cap:C4.2` · 4 issues · 3.25 j · 13 tours · **0/4 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#694](https://github.com/gilmry/koprogo/issues/694) | Scoping user↔ACP absent : un syndic/comptable voit toute l'organisation, pas … | L | manque 8/8 |
| [#798](https://github.com/gilmry/koprogo/issues/798) | Refonte UX — le périmètre devient l'ACP dans le modèle de données et permissi… | L | manque 7/8 |
| [#841](https://github.com/gilmry/koprogo/issues/841) | Le périmètre d'immeuble ne survit à aucune navigation : douze écrans le lisen… | M | manque 7/8 |
| [#868](https://github.com/gilmry/koprogo/issues/868) | Deux sélecteurs d'immeuble coexistent, et partagent un ancrage de recette | S | manque 8/8 |

### C4.3 — Les droits RGPD sont exerçables depuis l'interface

**Must** · `cap:C4.3` · 1 issues · 0.75 j · 3 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#842](https://github.com/gilmry/koprogo/issues/842) | Le droit à l'effacement RGPD est inatteignable : le serveur exige un mot de p… | M | manque 8/8 |

### C4.4 — Un prestataire reçoit un seul lien

**Should** · `cap:C4.4` · 1 issues · 1.00 j · 4 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#835](https://github.com/gilmry/koprogo/issues/835) | Deux systèmes de liens magiques parallèles : le prestataire reçoit deux liens… | L | manque 8/8 |

### C4.5 — Les erreurs sont typées, pas classées par sous-chaînes

**Should** · `cap:C4.5` · 2 issues · 2.00 j · 8 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#555](https://github.com/gilmry/koprogo/issues/555) | EPIC: migrer Result<_, String> → Result<_, AppError> (1263 violations, CRITIC… | L | manque 7/8 |
| [#762](https://github.com/gilmry/koprogo/issues/762) | Typer les erreurs applicatives au lieu de les classer par sous-chaînes | L | manque 8/8 |

## T1 — Refonte UX — épopée habilitante

`epic:ux` · 14 issues · 13.25 j · 53 tours

Ne livre aucune capacité métier : elle rend les autres atteignables. BMAD la range parmi les stories transverses, pas parmi les épopées de domaine, et l'ordre compte.

### C5.1 — Le socle visuel : jetons, icônes, libellés traduits

**Should** · `cap:C5.1` · 2 issues · 2.00 j · 8 tours · **1/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#797](https://github.com/gilmry/koprogo/issues/797) | Refonte UX — jetons de design et jeu d'icônes SVG : remplacer les émojis qui … | L | oui |
| [#834](https://github.com/gilmry/koprogo/issues/834) | 362 libellés de gabarit écrits en dur : les quatre langues s'arrêtent aux toasts | L | manque 8/8 |

### C5.2 — Le contrat de tests tient la refonte

**Must** · `cap:C5.2` · 2 issues · 2.00 j · 8 tours · **1/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#802](https://github.com/gilmry/koprogo/issues/802) | Refonte UX — adapter les tests sans en supprimer les règles produit qu'ils en… | L | oui |
| [#803](https://github.com/gilmry/koprogo/issues/803) | Poser des data-testid là où il n'y en a pas : 29 % de couverture avant une re… | L | manque 8/8 |

### C5.3 — Les huit maquettes sont implémentées

**Could** · `cap:C5.3` · 10 issues · 9.25 j · 37 tours · **0/10 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#556](https://github.com/gilmry/koprogo/issues/556) | [EPIC] Refonte UX multi-rôle + modèle ACP — pipeline Maury (39 stories, 7 sli… | L | manque 8/8 |
| [#818](https://github.com/gilmry/koprogo/issues/818) | Refonte UX — importer et implémenter la maquette Claude Design « Koprogo Revi… | L | manque 8/8 |
| [#820](https://github.com/gilmry/koprogo/issues/820) | Refonte UX — importer et implémenter la maquette Claude Design « Roles Mobile… | L | manque 8/8 |
| [#821](https://github.com/gilmry/koprogo/issues/821) | Refonte UX — importer et implémenter la maquette Claude Design « Admin Dashbo… | M | manque 8/8 |
| [#822](https://github.com/gilmry/koprogo/issues/822) | Refonte UX — importer et implémenter la maquette Claude Design « Accountant D… | M | manque 8/8 |
| [#823](https://github.com/gilmry/koprogo/issues/823) | Refonte UX — importer et implémenter la maquette Claude Design « KoproSidebar… | L | manque 8/8 |
| [#824](https://github.com/gilmry/koprogo/issues/824) | Refonte UX — importer et implémenter la maquette Claude Design « Owner Mobile… | L | manque 8/8 |
| [#825](https://github.com/gilmry/koprogo/issues/825) | Refonte UX — importer et implémenter la maquette Claude Design « Mobile First… | L | manque 8/8 |
| [#826](https://github.com/gilmry/koprogo/issues/826) | Refonte UX — importer et implémenter la maquette Claude Design « Syndic Dashb… | L | manque 8/8 |
| [#827](https://github.com/gilmry/koprogo/issues/827) | Refonte UX — importer et implémenter la maquette Claude Design « Lists (moder… | M | manque 8/8 |

## T2 — Accessibilité et mobile

`epic:accessibilite` · 5 issues · 3.50 j · 14 tours

La cible est mobile-first et l'application est écrite desktop-first. L'écart n'est pas cosmétique : il rend des capacités inatteignables.

### C6.1 — L'audit d'accessibilité voit les écrans authentifiés

**Should** · `cap:C6.1` · 2 issues · 1.50 j · 6 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#592](https://github.com/gilmry/koprogo/issues/592) | [Story 5.8] Gate CI a11y axe-core + data-testid + Lighthouse | M | manque 4/8 |
| [#865](https://github.com/gilmry/koprogo/issues/865) | L'audit d'accessibilité n'examine que l'écran de connexion : aucun écran auth… | M | manque 8/8 |

### C6.2 — Le produit est utilisable à une largeur de téléphone

**Should** · `cap:C6.2` · 3 issues · 2.00 j · 8 tours · **0/3 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#866](https://github.com/gilmry/koprogo/issues/866) | Six tableaux illisibles sur téléphone, dont trois qui coupent leurs colonnes … | M | manque 8/8 |
| [#869](https://github.com/gilmry/koprogo/issues/869) | Aucune spec Playwright ne s'exécute à une largeur de téléphone, alors que tou… | M | manque 8/8 |
| [#871](https://github.com/gilmry/koprogo/issues/871) | Quatre défauts d'affichage relevés au banc mobile : NaN €, « Failed to fetch … | S | manque 8/8 |

## T3 — Harnais de recette

`epic:recette` · 5 issues · 4.00 j · 16 tours

Sprint 0 continué. Ce qui permet de BOUCLER : sans lui, aucune autre capacité ne peut être déclarée tenue.

### C7.1 — La recette peut se connecter et s'exécuter

**Must** · `cap:C7.1` · 4 issues · 3.00 j · 12 tours · **4/4 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#696](https://github.com/gilmry/koprogo/issues/696) | Instabilité smoke suite Playwright CI : 109 échecs sur specs pré-existantes (… | M | oui |
| [#832](https://github.com/gilmry/koprogo/issues/832) | Quinze specs Playwright échouent sans erreur d'identité : les ancrages existe… | M | oui |
| [#870](https://github.com/gilmry/koprogo/issues/870) | La suite e2e ne peut plus se connecter à la démo : le repli admin123 est mort… | S | oui |
| [#872](https://github.com/gilmry/koprogo/issues/872) | La pile de développement revendique les conteneurs de la démo : `docker compo… | L | oui |

### C7.2 — La taxonomie des tests est la gate de release

**Should** · `cap:C7.2` · 1 issues · 1.00 j · 4 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#427](https://github.com/gilmry/koprogo/issues/427) | Validation — taxonomie tests 4 catégories + revue humaine+Cowork comme gate r… | L | manque 4/8 |

## T4 — Documentation vivante multi-persona

`epic:doc-vivante` · 14 issues · 11.25 j · 45 tours

Vient après que les parcours fonctionnent : filmer un écran qui casse produit une documentation périmée le jour de sa livraison.

### C8.1 — Les six parcours par persona sont filmés

**Could** · `cap:C8.1` · 7 issues · 5.75 j · 23 tours · **0/7 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#805](https://github.com/gilmry/koprogo/issues/805) | Documentation vivante multi-persona : expliquer le logiciel par rôle, sur le … | M | manque 8/8 |
| [#806](https://github.com/gilmry/koprogo/issues/806) | Documentation vivante — parcours du SYNDIC, de la première connexion à la clô… | L | manque 8/8 |
| [#807](https://github.com/gilmry/koprogo/issues/807) | Documentation vivante — parcours du COPROPRIÉTAIRE, le rôle que cinq recettes… | L | manque 8/8 |
| [#808](https://github.com/gilmry/koprogo/issues/808) | Documentation vivante — parcours du COMPTABLE, le rôle le mieux cadré du produit | M | manque 7/8 |
| [#809](https://github.com/gilmry/koprogo/issues/809) | Documentation vivante — parcours de l'ADMINISTRATEUR : ce qu'il crée, et ce q… | M | manque 8/8 |
| [#815](https://github.com/gilmry/koprogo/issues/815) | Documentation vivante — parcours du prestataire : du ticket reçu au rapport d… | M | manque 8/8 |
| [#816](https://github.com/gilmry/koprogo/issues/816) | Documentation vivante — parcours du conseil de copropriété : surveiller le sy… | M | manque 8/8 |

### C8.2 — Les quatre workflows transverses sont filmés

**Could** · `cap:C8.2` · 4 issues · 3.25 j · 13 tours · **0/4 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#810](https://github.com/gilmry/koprogo/issues/810) | Workflow multi-persona — le cycle de vie d'une assemblée générale, du syndic … | L | manque 8/8 |
| [#811](https://github.com/gilmry/koprogo/issues/811) | Workflow multi-persona — le circuit d'une facture, du fournisseur au copropri… | M | manque 8/8 |
| [#812](https://github.com/gilmry/koprogo/issues/812) | Workflow multi-persona — la naissance d'une copropriété, de l'administrateur … | M | manque 8/8 |
| [#817](https://github.com/gilmry/koprogo/issues/817) | Workflow multi-persona — le ticket, du copropriétaire qui signale au prestata… | M | manque 8/8 |

### C8.3 — Les cent specs e2e racontent le produit par persona

**Could** · `cap:C8.3` · 1 issues · 1.00 j · 4 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#813](https://github.com/gilmry/koprogo/issues/813) | Documentation vivante — restructurer les 100 tests e2e par persona pour que l… | L | manque 8/8 |

### C8.4 — La documentation est rangée et publiée

**Could** · `cap:C8.4` · 2 issues · 1.25 j · 5 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#595](https://github.com/gilmry/koprogo/issues/595) | [Story Tx.3] Documentation docs/agent-activity/ (Tier 2 log) | S | manque 4/8 |
| [#854](https://github.com/gilmry/koprogo/issues/854) | Ranger la documentation avant la release : 29 markdown à la racine de docs/, … | M | manque 8/8 |

## T5 — Ops et infrastructure

`epic:ops` · 10 issues · 9.00 j · 36 tours

Le déploiement, l'IaC, les vulnérabilités, et les garde-fous des agents.

### C9.1 — Le déploiement tient la charge et le partage du réseau

**Should** · `cap:C9.1` · 4 issues · 3.50 j · 14 tours · **0/4 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#453](https://github.com/gilmry/koprogo/issues/453) | Pipeline TLS dispatch dev/integration/staging via OVH DNS-01 | M | manque 8/8 |
| [#515](https://github.com/gilmry/koprogo/issues/515) | infra: ArgoCD GitOps fresh-cluster deployment fails on 5 gaps (dry-run Docker… | L | manque 8/8 |
| [#718](https://github.com/gilmry/koprogo/issues/718) | [BUG] 502 Bad Gateway / timeouts sur api.koprogo.com sous rafale de requêtes … | L | manque 8/8 |
| [#731](https://github.com/gilmry/koprogo/issues/731) | Collision d'alias DNS sur le réseau partagé ecosolva-web : 4 projets exposent… | M | manque 8/8 |

### C9.2 — L'IaC est testée et relue

**Could** · `cap:C9.2` · 3 issues · 2.75 j · 11 tours · **0/3 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#354](https://github.com/gilmry/koprogo/issues/354) | refactor(infra): Tests IaC manquants — terraform validate, ansible-lint, mole… | L | manque 8/8 |
| [#355](https://github.com/gilmry/koprogo/issues/355) | refactor(infra): Restructuration IaC — repo séparé, tests, policy-as-code | L | manque 8/8 |
| [#466](https://github.com/gilmry/koprogo/issues/466) | RFC: Stratégie GitOps multi-environnement — branches infra/* + main + Applica… | M | manque 8/8 |

### C9.3 — Les vulnérabilités connues sont fermées

**Should** · `cap:C9.3` · 1 issues · 0.75 j · 3 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#432](https://github.com/gilmry/koprogo/issues/432) | Security — 14 dependabot vulnerabilities sur main (5 high / 3 moderate / 6 low) | M | manque 8/8 |

### C9.4 — Les garde-fous des agents sont audités

**Could** · `cap:C9.4` · 2 issues · 2.00 j · 8 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#425](https://github.com/gilmry/koprogo/issues/425) | Méta — Garde-fous IA: audit qualité+sécurité IaC, cause racine, plan de reméd… | L | manque 8/8 |
| [#429](https://github.com/gilmry/koprogo/issues/429) | Méta — Operations runtime: deploy IaC en prod + agents DevOps/SRE/Support/CSI… | L | manque 4/8 |

## T6 — Arbitrages produit en attente

`epic:arbitrage` · 1 issues · 0.50 j · 2 tours

Des questions ouvertes, pas des défauts. La Méthode Foyer les veut en RFC, discutées avant d'être tranchées — jamais tranchées en silence dans le code.

### C10.1 — Le groupe « Communauté » du comptable est tranché

**Must** · `cap:C10.1` · 1 issues · 0.50 j · 2 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#856](https://github.com/gilmry/koprogo/issues/856) | Décision produit : le comptable doit-il voir un groupe « Communauté » réduit … | S | manque 6/8 |

## Estimation

| Épopée | Issues | Jours | Tours |
|---|---:|---:|---:|
| E1 — Copropriété | 14 | 13.00 | 52 |
| E2 — Comptabilité | 3 | 2.50 | 10 |
| E3 — Économie circulaire | 9 | 7.25 | 29 |
| E4 — Plateforme | 10 | 9.00 | 36 |
| T1 — Refonte UX | 14 | 13.25 | 53 |
| T2 — Accessibilité et mobile | 5 | 3.50 | 14 |
| T3 — Harnais de recette | 5 | 4.00 | 16 |
| T4 — Documentation vivante multi-persona | 14 | 11.25 | 45 |
| T5 — Ops et infrastructure | 10 | 9.00 | 36 |
| T6 — Arbitrages produit en attente | 1 | 0.50 | 2 |
| **Total** | **85** | **73.25** | **293** |

`S` = 0,5 j · `M` = 0,75 j · `L` = 1 j — wall-clock du superviseur, pas
temps machine. Les tours mesurent l'autre axe, le coût en tokens.

**Ce sont des bornes hautes de première passe.** La méthode prévoit
qu'elles soient resserrées story après story par le CSI, à partir du réel
observé. Les publier non resserrées est le seul moyen d'avoir un point de
départ falsifiable ; les publier comme un engagement serait une faute.

## Ordre

| Rang | Capacités | Pourquoi ce rang |
|---|---|---|
| 1 | C7.1 | Sans harnais de recette qui s'exécute, aucune autre capacité ne peut être déclarée tenue. |
| 2 | C4.1, C4.2, C4.3 | Ce qui expose des données ou empêche d'exercer un droit. |
| 3 | C10.1 | Un arbitrage qui borne C5.2 : le trancher tôt coûte une conversation, le trancher tard coûte un revirement. |
| 4 | C5.2, C5.1 | Le contrat de tests AVANT de déplacer un écran, puis le socle visuel. |
| 5 | C1.1, C1.3 | Le noyau légal : une AG qui aboutit, un registre qui atteste. |
| 6 | C1.5, C2.1, C2.2, C3.1, C4.4, C4.5, C6.1, C6.2, C9.1, C9.3 | Les `Should`, largement parallélisables. |
| 7 | les `Could` | C5.3 après C5.1 ; tout T4 après que les parcours fonctionnent. |
| 8 | G1 puis G2 | Revue humaine signée, puis le tag. Hors périmètre agent. |

---

*Dérivé du Manifeste Maury (CC BY-SA 4.0). Gabarit BMAD phase E :
`foyer/bmad/livrables/epics-stories.template.md`.*
