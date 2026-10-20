---
livrable: Epics & User Stories (BMAD phase E — TOGAF Solutions)
projet: KoproGo
jalon: v0.1.0
genere_par: scripts/backlog-structure.py
signature_humaine:
  date: null
  nom: null
  role: null
  etat: >-
    NON SIGNÉ — reclassement proposé le 2026-10-20, en attente du PO.
    La signature précédente (Gilles Maury, 2026-09-12, 87 issues) portait
    sur un classement qui ne passait plus `--verifier` : 43 issues ouvertes
    sans capacité, 27 issues classées mais fermées. La proposition retire
    les 27, rattache les 43 (dont #1001 à #1027, recette du cycle de vie du
    2026-10-20), ajoute quatre capacités (C1.6, C2.4, C2.5, C9.5) et note
    tenues C2.1, C2.2, C4.3 et C6.2. Tailles des nouvelles issues : bornes
    hautes de première passe, non relues. Seul le PO signe.
---

# Backlog structuré — v0.1.0

*Épopées et capacités. Généré par `scripts/backlog-structure.py` : ne pas
éditer à la main, la prochaine génération écraserait la correction.*

**106 issues ouvertes**, 10 épopées, 33 capacités. Classement exhaustif et exclusif : chaque issue appartient à exactement
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

**63 issues sur 106** portent les huit. Le relevé
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

`epic:copropriete` · 17 issues · 13.75 j · 55 tours

Le contexte borné qui ne dépend de rien : ce que la loi belge dit d'une assemblée, d'un lot, d'une voix. Une erreur ici n'est pas un défaut d'affichage, c'est une décision annulable.

### C1.1 — Une assemblée générale aboutit, de la convocation au PV

**Must** · `cap:C1.1` · 7 issues · 5.75 j · 23 tours · **3/7 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#576](https://github.com/gilmry/koprogo/issues/576) | [Story 4.1] [cluster-coord] Meeting.mode hybrid + quorum agrégé Decimal | L | oui |
| [#577](https://github.com/gilmry/koprogo/issues/577) | [Story 4.2] Vote distant auth_method strong (itsme/eID) — closes #48 | L | oui |
| [#581](https://github.com/gilmry/koprogo/issues/581) | [Story 4.6] Résolution EvaluationContractors AGO auto non retirable | M | oui |
| [#1004](https://github.com/gilmry/koprogo/issues/1004) | Aucun vote n'est possible en AG, ni par le copropriétaire ni encodé par le sy… | L | manque 8/8 |
| [#1005](https://github.com/gilmry/koprogo/issues/1005) | Une convocation envoyée disparaît au rechargement (404) | M | manque 8/8 |
| [#1006](https://github.com/gilmry/koprogo/issues/1006) | Impossible de saisir l'ordre du jour avant la validation du quorum : la convo… | M | manque 8/8 |
| [#1015](https://github.com/gilmry/koprogo/issues/1015) | La checklist de clôture d'AG affiche des clés brutes et n'est pas rafraîchie | S | manque 8/8 |

### C1.2 — Le procès-verbal fait foi

**Should** · `cap:C1.2` · 2 issues · 2.00 j · 8 tours · **2/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#578](https://github.com/gilmry/koprogo/issues/578) | [Story 4.3] Minutes (PV) + 2 signatures eIDAS qualifiées | L | oui |
| [#579](https://github.com/gilmry/koprogo/issues/579) | [Story 4.4] Adapter ElectronicSignatureProvider (port + 3 adapters eID/itsme/… | L | oui |

### C1.3 — Le registre légal atteste ce qu'il déclare

**Must** · `cap:C1.3` · 1 issues · 0.75 j · 3 tours · **1/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#846](https://github.com/gilmry/koprogo/issues/846) | Onze tables et une vue existent en base et ne sont lues par aucun code, dont … | M | oui |

### C1.4 — Les organes de contrôle existent

**Could** · `cap:C1.4` · 4 issues · 3.00 j · 12 tours · **2/4 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#582](https://github.com/gilmry/koprogo/issues/582) | [Story 4.7] CdC membre élu + action create_alert | M | oui |
| [#583](https://github.com/gilmry/koprogo/issues/583) | [Story 4.8] [cluster-coord] CommissaireAuxComptes + VerificationCertificate | L | oui |
| [#980](https://github.com/gilmry/koprogo/issues/980) | Élection du conseil : la règle d'un an n'est annoncée nulle part, son refus s… | M | manque 8/8 |
| [#1019](https://github.com/gilmry/koprogo/issues/1019) | L'élection d'un conseil de copropriété est refusée sous 20 lots, alors que l'… | S | manque 8/8 |

### C1.5 — L'état daté a un destinataire identifié

**Should** · `cap:C1.5` · 1 issues · 1.00 j · 4 tours · **1/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#855](https://github.com/gilmry/koprogo/issues/855) | Le notaire n'a pas d'identité : l'état daté est servi à qui connaît la référe… | L | oui |

### C1.6 — Un incident se signale, se confie et se clôt avec son rapport

**Should** · `cap:C1.6` · 2 issues · 1.25 j · 5 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#977](https://github.com/gilmry/koprogo/issues/977) | Trois des cinq transitions d'un incident sont mortes : resolve, cancel et reo… | M | manque 8/8 |
| [#1017](https://github.com/gilmry/koprogo/issues/1017) | Le rapport saisi à la résolution d'un ticket n'apparaît pas sur sa fiche | S | manque 8/8 |

## E2 — Comptabilité — la charge et sa répartition

`epic:comptabilite` · 11 issues · 9.00 j · 36 tours

Connaît `copropriete` : une charge se répartit sur des quotités, elle ne peut pas les ignorer. L'inverse est faux.

### C2.3 — Les fonds affectés sont une entité, pas une convention

**Could** · `cap:C2.3` · 1 issues · 1.00 j · 4 tours · **1/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#635](https://github.com/gilmry/koprogo/issues/635) | Fonds affectés / thésaurisation : entité Fund dédiée aux travaux d'ampleur (v… | L | oui |

### C2.4 — Le budget appliqué est celui qu'une AG a adopté, fonds de réserve compris

**Must** · `cap:C2.4` · 2 issues · 1.50 j · 6 tours · **0/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#1007](https://github.com/gilmry/koprogo/issues/1007) | Un budget peut être « approuvé » en citant une AG future dont la résolution n… | M | manque 8/8 |
| [#1011](https://github.com/gilmry/koprogo/issues/1011) | Le budget n'a pas de fonds de réserve | M | manque 8/8 |

### C2.5 — La comptabilité d'une ACP est juste et lisible

**Must** · `cap:C2.5` · 8 issues · 6.50 j · 26 tours · **0/8 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#1003](https://github.com/gilmry/koprogo/issues/1003) | Les montants décimaux renvoyés en chaîne s'affichent comme un tiret | M | manque 8/8 |
| [#1008](https://github.com/gilmry/koprogo/issues/1008) | La TVA des factures d'une ACP est comptée « récupérable » (411) au lieu d'êtr… | L | manque 8/8 |
| [#1009](https://github.com/gilmry/koprogo/issues/1009) | L'émission d'un appel de fonds ne génère aucune écriture comptable | L | manque 8/8 |
| [#1010](https://github.com/gilmry/koprogo/issues/1010) | Le bilan suit un modèle de société, pas le PCMN des copropriétés ; pas de bal… | L | manque 8/8 |
| [#1012](https://github.com/gilmry/koprogo/issues/1012) | Un paiement de copropriétaire ne peut pas être partiel | M | manque 8/8 |
| [#1013](https://github.com/gilmry/koprogo/issues/1013) | « Marquer payée » est proposé sur une facture non approuvée, et le refus est … | S | manque 8/8 |
| [#1016](https://github.com/gilmry/koprogo/issues/1016) | Le tableau de bord du syndic affiche un taux d'encaissement de 100 % au lieu … | M | manque 8/8 |
| [#1018](https://github.com/gilmry/koprogo/issues/1018) | Le taux de TVA d'une réparation reste à 21 % par défaut, sans tenir compte de… | M | manque 8/8 |

## E3 — Économie circulaire — les modules communautaires

`epic:economie-circulaire` · 8 issues · 6.50 j · 26 tours

Connaît `copropriete`. SEL, sondages, objets partagés, énergie : la partie du produit qui distingue KoproGo d'un logiciel de syndic.

### C3.1 — Les modules communautaires sont atteignables

**Should** · `cap:C3.1` · 1 issues · 1.00 j · 4 tours · **1/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#779](https://github.com/gilmry/koprogo/issues/779) | Rebrancher les six modules communautaires : 111 points d'entrée servis que le… | L | oui |

### C3.2 — Le syndic a un rôle dans la communauté

**Could** · `cap:C3.2` · 3 issues · 2.00 j · 8 tours · **3/3 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#587](https://github.com/gilmry/koprogo/issues/587) | [Story 5.3] Syndic = community.moderator (RBAC Community SEL/Poll/Notice/Shar… | M | oui |
| [#588](https://github.com/gilmry/koprogo/issues/588) | [Story 5.4] Reservation.on_behalf_of_acp (exception syndic) | M | oui |
| [#589](https://github.com/gilmry/koprogo/issues/589) | [Story 5.5] Comptable (encodeur ET émetteur) 403 sur /community/* | S | oui |

### C3.3 — Une ACP active les modules qu'elle veut

**Could** · `cap:C3.3` · 4 issues · 3.50 j · 14 tours · **4/4 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#585](https://github.com/gilmry/koprogo/issues/585) | [Story 5.1] Table acp_enabled_modules + ModuleGuard middleware + ModuleDisabl… | L | oui |
| [#586](https://github.com/gilmry/koprogo/issues/586) | [Story 5.2] UI ModuleGate.svelte + store enabled_modules | M | oui |
| [#590](https://github.com/gilmry/koprogo/issues/590) | [Story 5.6] Activation/désactivation modules audité + archivage data (jamais … | M | oui |
| [#591](https://github.com/gilmry/koprogo/issues/591) | [Story 5.7] Onboarding modulaire wizard ≤ 5 min | L | oui |

## E4 — Plateforme — identité, périmètre, droits

`epic:plateforme` · 16 issues · 13.50 j · 54 tours

Ne dépend de rien. Qui est l'appelant, ce qu'il a le droit de voir, et ce qu'il peut exiger qu'on efface.

### C4.1 — Toute route décide de l'identité qu'elle reçoit

**Must** · `cap:C4.1` · 3 issues · 2.25 j · 9 tours · **1/3 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#864](https://github.com/gilmry/koprogo/issues/864) | 87 routes prennent une identité sans s'en servir pour décider : supprimer le … | L | oui |
| [#972](https://github.com/gilmry/koprogo/issues/972) | /legal-rules exige une session alors que sa source est publique : un article … | S | manque 8/8 |
| [#976](https://github.com/gilmry/koprogo/issues/976) | Deux routes lisent les devis et évaluations de n'importe quel prestataire, to… | M | manque 8/8 |

### C4.2 — Le périmètre est l'ACP, et il survit à la navigation

**Must** · `cap:C4.2` · 9 issues · 7.25 j · 29 tours · **4/9 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#694](https://github.com/gilmry/koprogo/issues/694) | Scoping user↔ACP absent : un syndic/comptable voit toute l'organisation, pas … | L | oui |
| [#798](https://github.com/gilmry/koprogo/issues/798) | Refonte UX — le périmètre devient l'ACP dans le modèle de données et permissi… | L | oui |
| [#841](https://github.com/gilmry/koprogo/issues/841) | Le périmètre d'immeuble ne survit à aucune navigation : douze écrans le lisen… | M | oui |
| [#868](https://github.com/gilmry/koprogo/issues/868) | Deux sélecteurs d'immeuble coexistent, et partagent un ancrage de recette | S | oui |
| [#969](https://github.com/gilmry/koprogo/issues/969) | Tout 401 déconnecte globalement : ouvrir /booking-detail suffit à perdre sa s… | M | manque 8/8 |
| [#1001](https://github.com/gilmry/koprogo/issues/1001) | La fiche copropriétaire n'est reliée à aucun compte utilisateur : tout l'espa… | L | manque 7/8 |
| [#1021](https://github.com/gilmry/koprogo/issues/1021) | Le bandeau de contexte boucle à environ 95 requêtes par seconde dès qu'un imm… | M | manque 8/8 |
| [#1024](https://github.com/gilmry/koprogo/issues/1024) | L'espace copropriétaire /owner charge indéfiniment | S | manque 8/8 |
| [#1027](https://github.com/gilmry/koprogo/issues/1027) | Un copropriétaire voit les lots et les fiches (emails compris) de tous les co… | L | manque 8/8 |

### C4.4 — Un prestataire reçoit un seul lien

**Should** · `cap:C4.4` · 2 issues · 2.00 j · 8 tours · **1/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#835](https://github.com/gilmry/koprogo/issues/835) | Deux systèmes de liens magiques parallèles : le prestataire reçoit deux liens… | L | oui |
| [#1002](https://github.com/gilmry/koprogo/issues/1002) | Pas de rôle ni d'annuaire de prestataires : devis et assignation d'un ticket … | L | manque 8/8 |

### C4.5 — Les erreurs sont typées, pas classées par sous-chaînes

**Should** · `cap:C4.5` · 2 issues · 2.00 j · 8 tours · **2/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#555](https://github.com/gilmry/koprogo/issues/555) | EPIC: migrer Result<_, String> → Result<_, AppError> (1263 violations, CRITIC… | L | oui |
| [#762](https://github.com/gilmry/koprogo/issues/762) | Typer les erreurs applicatives au lieu de les classer par sous-chaînes | L | oui |

## T1 — Refonte UX — épopée habilitante

`epic:ux` · 22 issues · 17.50 j · 70 tours

Ne livre aucune capacité métier : elle rend les autres atteignables. BMAD la range parmi les stories transverses, pas parmi les épopées de domaine, et l'ordre compte.

### C5.1 — Le socle visuel : jetons, icônes, libellés traduits

**Should** · `cap:C5.1` · 11 issues · 7.25 j · 29 tours · **2/11 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#797](https://github.com/gilmry/koprogo/issues/797) | Refonte UX — jetons de design et jeu d'icônes SVG : remplacer les émojis qui … | L | oui |
| [#834](https://github.com/gilmry/koprogo/issues/834) | 362 libellés de gabarit écrits en dur : les quatre langues s'arrêtent aux toasts | L | oui |
| [#968](https://github.com/gilmry/koprogo/issues/968) | Balayage du syndic : /legal-rules jette each_key_duplicate, /contractor et /c… | M | manque 8/8 |
| [#971](https://github.com/gilmry/koprogo/issues/971) | Le lien « Voir la règle → » ne s'affiche jamais : aucun appelant de CadreLega… | S | manque 8/8 |
| [#975](https://github.com/gilmry/koprogo/issues/975) | 24 contradictions entre menus et routes, pas seulement pour le modérateur | M | manque 8/8 |
| [#1014](https://github.com/gilmry/koprogo/issues/1014) | La bannière de conformité d'un immeuble n'est pas rafraîchie après l'ajout d'… | S | manque 8/8 |
| [#1020](https://github.com/gilmry/koprogo/issues/1020) | Des UUID s'affichent à la place des noms (organisation, immeuble) | S | manque 8/8 |
| [#1022](https://github.com/gilmry/koprogo/issues/1022) | Menus superadmin et syndic incohérents selon le contexte (404 et redirections) | M | manque 8/8 |
| [#1023](https://github.com/gilmry/koprogo/issues/1023) | Le formulaire copropriétaire affiche adresse, code postal et ville comme opti… | S | manque 8/8 |
| [#1025](https://github.com/gilmry/koprogo/issues/1025) | L'invite d'installation PWA revient à chaque page et masque les boutons des d… | S | manque 8/8 |
| [#1026](https://github.com/gilmry/koprogo/issues/1026) | Le panneau des notifications du copropriétaire est coupé à gauche de l'écran | S | manque 8/8 |

### C5.2 — Le contrat de tests tient la refonte

**Must** · `cap:C5.2` · 1 issues · 1.00 j · 4 tours · **1/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#802](https://github.com/gilmry/koprogo/issues/802) | Refonte UX — adapter les tests sans en supprimer les règles produit qu'ils en… | L | oui |

### C5.3 — Les huit maquettes sont implémentées

**Could** · `cap:C5.3` · 10 issues · 9.25 j · 37 tours · **10/10 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#556](https://github.com/gilmry/koprogo/issues/556) | [EPIC] Refonte UX multi-rôle + modèle ACP — pipeline Maury (39 stories, 7 sli… | L | oui |
| [#818](https://github.com/gilmry/koprogo/issues/818) | Refonte UX — importer et implémenter la maquette Claude Design « Koprogo Revi… | L | oui |
| [#820](https://github.com/gilmry/koprogo/issues/820) | Refonte UX — importer et implémenter la maquette Claude Design « Roles Mobile… | L | oui |
| [#821](https://github.com/gilmry/koprogo/issues/821) | Refonte UX — importer et implémenter la maquette Claude Design « Admin Dashbo… | M | oui |
| [#822](https://github.com/gilmry/koprogo/issues/822) | Refonte UX — importer et implémenter la maquette Claude Design « Accountant D… | M | oui |
| [#823](https://github.com/gilmry/koprogo/issues/823) | Refonte UX — importer et implémenter la maquette Claude Design « KoproSidebar… | L | oui |
| [#824](https://github.com/gilmry/koprogo/issues/824) | Refonte UX — importer et implémenter la maquette Claude Design « Owner Mobile… | L | oui |
| [#825](https://github.com/gilmry/koprogo/issues/825) | Refonte UX — importer et implémenter la maquette Claude Design « Mobile First… | L | oui |
| [#826](https://github.com/gilmry/koprogo/issues/826) | Refonte UX — importer et implémenter la maquette Claude Design « Syndic Dashb… | L | oui |
| [#827](https://github.com/gilmry/koprogo/issues/827) | Refonte UX — importer et implémenter la maquette Claude Design « Lists (moder… | M | oui |

## T2 — Accessibilité et mobile

`epic:accessibilite` · 1 issues · 0.75 j · 3 tours

La cible est mobile-first et l'application est écrite desktop-first. L'écart n'est pas cosmétique : il rend des capacités inatteignables.

### C6.1 — L'audit d'accessibilité voit les écrans authentifiés

**Should** · `cap:C6.1` · 1 issues · 0.75 j · 3 tours · **1/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#592](https://github.com/gilmry/koprogo/issues/592) | [Story 5.8] Gate CI a11y axe-core + data-testid + Lighthouse | M | oui |

## T3 — Harnais de recette

`epic:recette` · 6 issues · 4.50 j · 18 tours

Sprint 0 continué. Ce qui permet de BOUCLER : sans lui, aucune autre capacité ne peut être déclarée tenue.

### C7.1 — La recette peut se connecter et s'exécuter

**Must** · `cap:C7.1` · 3 issues · 2.00 j · 8 tours · **1/3 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#880](https://github.com/gilmry/koprogo/issues/880) | Le gate e2e mesure contre un backend en hot reload : une recompilation en cou… | M | oui |
| [#966](https://github.com/gilmry/koprogo/issues/966) | Characterization E2E Gate rouge depuis le 2026-09-14 | M | manque 8/8 |
| [#983](https://github.com/gilmry/koprogo/issues/983) | check-no-local-helpers.sh est rouge depuis quatre mois et rien ne l'exécute | S | manque 8/8 |

### C7.2 — La taxonomie des tests est la gate de release

**Should** · `cap:C7.2` · 2 issues · 1.50 j · 6 tours · **2/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#427](https://github.com/gilmry/koprogo/issues/427) | Validation — taxonomie tests 4 catégories + revue humaine+Cowork comme gate r… | L | oui |
| [#913](https://github.com/gilmry/koprogo/issues/913) | Relire une promotion à l'envers de la pyramide : la vitrine d'abord, les test… | S | oui |

### C7.3 — La boucle d'agents peut tourner

**Must** · `cap:C7.3` · 1 issues · 1.00 j · 4 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#999](https://github.com/gilmry/koprogo/issues/999) | Rétrofit de KoproGo avec l'autopilote Foyer (porte rétrofit du jalon zéro) | L | manque 7/8 |

## T4 — Documentation vivante multi-persona

`epic:doc-vivante` · 13 issues · 10.00 j · 40 tours

Vient après que les parcours fonctionnent : filmer un écran qui casse produit une documentation périmée le jour de sa livraison.

### C8.1 — Les six parcours par persona sont filmés

**Could** · `cap:C8.1` · 5 issues · 4.00 j · 16 tours · **4/5 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#805](https://github.com/gilmry/koprogo/issues/805) | Documentation vivante multi-persona : expliquer le logiciel par rôle, sur le … | M | oui |
| [#806](https://github.com/gilmry/koprogo/issues/806) | Documentation vivante — parcours du SYNDIC, de la première connexion à la clô… | L | oui |
| [#808](https://github.com/gilmry/koprogo/issues/808) | Documentation vivante — parcours du COMPTABLE, le rôle le mieux cadré du produit | M | oui |
| [#809](https://github.com/gilmry/koprogo/issues/809) | Documentation vivante — parcours de l'ADMINISTRATEUR : ce qu'il crée, et ce q… | M | oui |
| [#974](https://github.com/gilmry/koprogo/issues/974) | Cinq parcours filmés sur sept ne cliquent jamais : ils regardent des écrans | M | manque 8/8 |

### C8.2 — Les quatre workflows transverses sont filmés

**Could** · `cap:C8.2` · 4 issues · 3.25 j · 13 tours · **4/4 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#810](https://github.com/gilmry/koprogo/issues/810) | Workflow multi-persona — le cycle de vie d'une assemblée générale, du syndic … | L | oui |
| [#811](https://github.com/gilmry/koprogo/issues/811) | Workflow multi-persona — le circuit d'une facture, du fournisseur au copropri… | M | oui |
| [#812](https://github.com/gilmry/koprogo/issues/812) | Workflow multi-persona — la naissance d'une copropriété, de l'administrateur … | M | oui |
| [#817](https://github.com/gilmry/koprogo/issues/817) | Workflow multi-persona — le ticket, du copropriétaire qui signale au prestata… | M | oui |

### C8.3 — Les cent specs e2e racontent le produit par persona

**Could** · `cap:C8.3` · 1 issues · 1.00 j · 4 tours · **1/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#813](https://github.com/gilmry/koprogo/issues/813) | Documentation vivante — restructurer les 100 tests e2e par persona pour que l… | L | oui |

### C8.4 — La documentation est rangée et publiée

**Could** · `cap:C8.4` · 3 issues · 1.75 j · 7 tours · **2/3 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#595](https://github.com/gilmry/koprogo/issues/595) | [Story Tx.3] Documentation docs/agent-activity/ (Tier 2 log) | S | oui |
| [#854](https://github.com/gilmry/koprogo/issues/854) | Ranger la documentation avant la release : 29 markdown à la racine de docs/, … | M | oui |
| [#963](https://github.com/gilmry/koprogo/issues/963) | La vitrine expire au bout de 7 jours alors que la doc est permanente | S | manque 8/8 |

## T5 — Ops et infrastructure

`epic:ops` · 11 issues · 9.25 j · 37 tours

Le déploiement, l'IaC, les vulnérabilités, et les garde-fous des agents.

### C9.1 — Le déploiement tient la charge et le partage du réseau

**Should** · `cap:C9.1` · 4 issues · 3.25 j · 13 tours · **3/4 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#453](https://github.com/gilmry/koprogo/issues/453) | Pipeline TLS dispatch dev/integration/staging via OVH DNS-01 | M | oui |
| [#515](https://github.com/gilmry/koprogo/issues/515) | infra: ArgoCD GitOps fresh-cluster deployment fails on 5 gaps (dry-run Docker… | L | oui |
| [#731](https://github.com/gilmry/koprogo/issues/731) | Collision d'alias DNS sur le réseau partagé ecosolva-web : 4 projets exposent… | M | oui |
| [#973](https://github.com/gilmry/koprogo/issues/973) | /mcp-chat appelle quatre points d'entrée qui n'existent pas, et Traefik ne ro… | M | manque 8/8 |

### C9.2 — L'IaC est testée et relue

**Could** · `cap:C9.2` · 3 issues · 2.50 j · 10 tours · **2/3 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#354](https://github.com/gilmry/koprogo/issues/354) | refactor(infra): Tests IaC manquants — terraform validate, ansible-lint, mole… | L | oui |
| [#355](https://github.com/gilmry/koprogo/issues/355) | refactor(infra): Restructuration IaC — repo séparé, tests, policy-as-code | L | oui |
| [#967](https://github.com/gilmry/koprogo/issues/967) | IaC lint : les rôles backup, hardening et security sont introuvables depuis m… | S | manque 8/8 |

### C9.3 — Les vulnérabilités connues sont fermées

**Should** · `cap:C9.3` · 1 issues · 0.75 j · 3 tours · **1/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#432](https://github.com/gilmry/koprogo/issues/432) | Security — 14 dependabot vulnerabilities sur main (5 high / 3 moderate / 6 low) | M | oui |

### C9.4 — Les garde-fous des agents sont audités

**Could** · `cap:C9.4` · 2 issues · 2.00 j · 8 tours · **2/2 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#425](https://github.com/gilmry/koprogo/issues/425) | Méta — Garde-fous IA: audit qualité+sécurité IaC, cause racine, plan de reméd… | L | oui |
| [#429](https://github.com/gilmry/koprogo/issues/429) | Méta — Operations runtime: deploy IaC en prod + agents DevOps/SRE/Support/CSI… | L | oui |

### C9.5 — La 0.1.0 est taguée et déployée par Sluis

**Must** · `cap:C9.5` · 1 issues · 0.75 j · 3 tours · **0/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#1000](https://github.com/gilmry/koprogo/issues/1000) | Taguer v0.1.0 et la déployer via Sluis, RELEASE.md à jour | M | manque 7/8 |

## T6 — Arbitrages produit en attente

`epic:arbitrage` · 1 issues · 0.50 j · 2 tours

Des questions ouvertes, pas des défauts. La Méthode Foyer les veut en RFC, discutées avant d'être tranchées — jamais tranchées en silence dans le code.

### C10.1 — Le groupe « Communauté » du comptable est tranché

**Must** · `cap:C10.1` · 1 issues · 0.50 j · 2 tours · **1/1 prêtes**

| Issue | Titre | Taille | Prête |
|---|---|---|---|
| [#856](https://github.com/gilmry/koprogo/issues/856) | Décision produit : le comptable doit-il voir un groupe « Communauté » réduit … | S | oui |

## Estimation

| Épopée | Issues | Jours | Tours |
|---|---:|---:|---:|
| E1 — Copropriété | 17 | 13.75 | 55 |
| E2 — Comptabilité | 11 | 9.00 | 36 |
| E3 — Économie circulaire | 8 | 6.50 | 26 |
| E4 — Plateforme | 16 | 13.50 | 54 |
| T1 — Refonte UX | 22 | 17.50 | 70 |
| T2 — Accessibilité et mobile | 1 | 0.75 | 3 |
| T3 — Harnais de recette | 6 | 4.50 | 18 |
| T4 — Documentation vivante multi-persona | 13 | 10.00 | 40 |
| T5 — Ops et infrastructure | 11 | 9.25 | 37 |
| T6 — Arbitrages produit en attente | 1 | 0.50 | 2 |
| **Total** | **106** | **85.25** | **341** |

`S` = 0,5 j · `M` = 0,75 j · `L` = 1 j — wall-clock du superviseur, pas
temps machine. Les tours mesurent l'autre axe, le coût en tokens.

**Ce sont des bornes hautes de première passe.** La méthode prévoit
qu'elles soient resserrées story après story par le CSI, à partir du réel
observé. Les publier non resserrées est le seul moyen d'avoir un point de
départ falsifiable ; les publier comme un engagement serait une faute.

## Ordre

| Rang | Capacités | Pourquoi ce rang |
|---|---|---|
| **0** | **C7.3** | **Story habilitante.** Elle livre « la capacité de boucler » : la vitrine devient un artefact de branche, et le fan-out est prouvé sur une story réelle. Sans elle, le mécanisme de *répondre-de* retenu — la revue de promotion instruite par les gates et la vitrine — n'a pas de support. Elle **bloque le reste du backlog**. |
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
