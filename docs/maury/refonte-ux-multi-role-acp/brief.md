---
feature: refonte-ux-multi-role-acp
phase: brief
phase_togaf: A (Vision)
agent_bmad: Mary (Analyste TOGAF)
authors: [Gilles Maury, Farah Maury]
date: 2026-05-20
version: 0.3
status: Draft awaiting human sign-off
changelog:
  - "0.3 (2026-05-20) — +C14 PWA Contractor magic link (électricien/jardinier/poubelles) ; +C15 AG distance/hybride Art. 3.87 §4 CC (modes in_person/remote/hybrid, quorum agrégé, auth forte #48 promu in-scope, 2 signatures électroniques PV) ; +INV-18 à INV-20 ; +SC11/SC12/SC13 ; BC Maintenance & Operations + External Mandates ajoutés ; hors-scope mis à jour"
  - "0.2 (2026-05-20) — +9 personas/rôles (Contractor, CdC Art. 3.88 CC, Commissaire aux comptes Art. 3.89 CC, split Comptable Émetteur/Encodeur, Avocat, Notaire mutations, Gardien/concierge, AMO+Architecte+BET) ; +C9-C13 capacités ; +INV-10..INV-17 ; +SC9/SC10"
  - "0.1 (2026-05-20) — Brief initial (4 personas, 8 capacités, 9 invariants, 8 critères succès)"
parent_brief: Maury/product-brief.md (v1.0, 2026-03-29)
issues_source: [#553, #554, #555, observations live 2026-05-20]
memories_applied:
  - admin-publishes-conform-buildings
  - validate-before-compute
  - world-model-seed
  - a11y-wcag-aa-baseline
  - data-testid-systematic
  - fe-refactor-test-driven
  - multirole-narrative-scenarios
  - no-f64-in-money
---

# Brief — Refonte UX multi-rôle + modèle ACP

## Methode Maury — Phase TOGAF A (Vision)

**GATE de signature humaine** : ce brief doit être signé par @gilmry avant ouverture de la Phase 2 (PRD).

---

## 1. Vision

Aligner l'expérience utilisateur de KoproGo sur le modèle juridique réel de la copropriété belge (Association des Copropriétaires — Art. 3.84 CC) et sur les rôles métier distincts (admin SaaS / syndic / comptable / copropriétaire). Éliminer les ambiguïtés actuelles qui produisent des bugs de conformité et des conflits d'intérêt (cf. session live testing 2026-05-20).

**Promesse utilisateur** : *« Je sais toujours sur quel immeuble, quelle ACP et quel cabinet syndic j'agis, et je ne peux pas accidentellement franchir une frontière juridique. »*

## 2. Problème observé (cas d'usage en panne)

Live testing 2026-05-20 a révélé 3 ruptures :

1. **Aucun sélecteur d'immeuble** au niveau navigation → un syndic multi-immeubles risque de créer une AG / des charges / des appels de fonds sur le mauvais bâtiment (cf. screenshots Résidence Grand Place / Les Jardins d'Ixelles).
2. **Modèle juridique faux** : `Building.organization_id` saute le niveau ACP → les requêtes / RBAC raisonnent sur "Organization" alors que la frontière légale est l'**ACP**. Risque de fuite de données entre 2 ACPs gérées par le même cabinet (cf. observation BUG-WF14-2 historique + règle interne `admin-publishes-conform-buildings`).
3. **Conflit d'intérêt RBAC Communauté non géré** : le syndic peut poster une annonce, voter à un sondage, échanger en SEL qu'il gère lui-même → biais administratif sur la vie communautaire.

## 3. Personas concernés

Repris du brief parent (`Maury/product-brief.md`) avec affinage spécifique. **13 personas/rôles** identifiés couvrant le cycle de vie complet d'une ACP (de la création à la vente d'unit, en passant par les gros travaux et la vérification comptable).

### Personas internes plateforme/cabinet

| Persona | Description | Besoin clé de la refonte |
|---|---|---|
| **Mathilde — Admin SaaS** | Superadmin de la plateforme. Crée des Organizations pour les nouveaux cabinets syndic. | Vue cross-ACP, sélecteur d'ACP, création/maintenance Organizations, modération méta sur Communauté. |
| **Sylvie — Syndic professionnelle** | Mandatée par N ACPs. Vit 80% de son temps sur l'app. | TdB cross-immeubles + sélecteur + favoris + bannière contextuelle 3-niveaux (immeuble · ACP · cabinet). |
| **Pierre — Comptable Émetteur** | Responsable comptable du cabinet. Émet charges, appels de fonds, écritures journal PCMN. Décisionnel. | Mêmes sélecteur + menus que Sylvie, **section Communauté masquée**, accès full CRUD compta. |
| **Paul — Comptable Encodeur** (nouveau split) | Employé/stagiaire du cabinet. Saisie factures entrantes uniquement, **pas d'émission** appels de fonds ni clôture. | Vue compta restreinte : create facture / scan / OCR ; **bouton "Émettre" désactivé** (RBAC). |
| **Cabinet multi-gestionnaire** | Équipe de N gestionnaires dans un cabinet. Chacun voit toutes les ACPs mais a son portefeuille préféré. | Entité `Portfolio` backend + UI favoris persistants multi-device + partage équipe. |

### Personas représentation copropriété (Art. 3.84-3.89 CC)

| Persona | Description | Besoin clé de la refonte |
|---|---|---|
| **Marie — Copropriétaire** | Possède 1 lot dans 1 immeuble. Vote, paye, échange en SEL. | Vue restreinte à ses immeubles + bannière sans choix (1 immeuble). Participation pleine Communauté. Délégation temporaire possible si syndic indisponible (mandat AG d'urgence). |
| **Catherine — Membre Conseil de Copropriété (CdC)** | Élue par AG (Art. 3.88 CC). Supervise le syndic, droit de regard renforcé sur comptes, peut alerter l'AG. Mandat borné. | Vue lecture étendue (compta, contrats, courriers syndic). Droit d'**alerter** (créer une note CdC visible AG). Participe à la Communauté en tant que copropriétaire normal. |
| **Henri — Commissaire aux comptes** | Désigné par AG (Art. 3.89 CC). Vérifie la comptabilité avant présentation AG. Indépendant du syndic. | Vue **lecture seule** sur tout le PCMN (journal, balance, états datés). Génère un **certificat de vérification** signé numériquement. Pas d'accès Communauté ni Gestion. |

### Personas intervenants externes

| Persona | Description | Besoin clé de la refonte |
|---|---|---|
| **Bruno — Prestataire (Contractor)** | Entreprise répondant à appels d'offres ACP : maintenance, travaux, services. | Voit **uniquement** ses propres devis/missions/factures émises. Soumission via portail dédié. Pas d'accès aux autres ACPs/copropriétaires. |
| **Anne — AMO + Architecte + Bureau d'études techniques** | Mandatée pour gros travaux (rénovation toiture, façade, ascenseur, audits énergétiques). Conseil amont + suivi chantier. | Accès **lecture étendue** aux documents techniques (rapports d'inspection, audits énergie, permis). Peut **proposer** des scénarios chiffrés (mais le vote reste à l'AG). Lien avec assurances + permis d'urbanisme. |
| **Maître Léa — Avocat / juriste copropriété** | Mandaté ponctuellement par l'ACP ou le syndic en cas de litige (impayés, contentieux travaux, ROI). | Accès **lecture mandaté + borné dans le temps** aux documents juridiques (statuts, ROI, PV des 5 dernières AG, correspondance). Mandat avec `valid_until` strict. |
| **Maître Sophie — Notaire (mutations)** | Intervient lors d'un acte de vente (unit). Doit produire l'**état daté Art. 577 CC** (quotas + dette + charges en cours). | Accès **lecture ciblée** sur 1 unit + son owner sortant + quotas + dette + charges 3 derniers ans. Période bornée à la mutation. Génère l'état daté PDF signé. |
| **Karim — Gardien / concierge** | Employé direct de l'ACP (pas du syndic). Présent sur site : tickets maintenance, réservations salle commune, signalement incidents. | Accès **opérationnel** : créer/voir tickets maintenance, créer réservations communes, signaler incidents IoT. **Pas d'accès** financier ni gouvernance. |

### Notes transverses sur les rôles

- Tous les rôles **« intervenants externes »** (Contractor, AMO, Avocat, Notaire) sont **mandatés par l'ACP** avec une période `valid_from`/`valid_until` et un motif tracé en audit.
- **CdC** et **Commissaire aux comptes** sont **élus par AG** (mandats légaux Art. 3.88/3.89 CC) : leur attribution passe par un workflow gouvernance (Resolution + Vote + PV).
- Le **Gardien** est employé direct de l'ACP (contrat travail), pas du cabinet syndic — distinction importante pour la RGPD et la facturation.
- **Encodeur ≠ Émetteur** est une distinction **interne au cabinet** (sous-rôle métier de "comptable"). Implémentation : 2 permissions distinctes (`invoice.create` vs `expense.issue + call_for_funds.create`).

## 4. Capacités (TOGAF B) — cadre de la refonte

| Capacité | Description | État courant | État cible |
|---|---|---|---|
| **C1 — Navigation contextuelle** | UX permettant de basculer entre immeubles sans erreur | Absente | Sélecteur global top-left + bannière 3-niveaux |
| **C2 — Modélisation juridique ACP** | Domaine reflète Art. 3.84 CC | Faux (Org → Building) | `Organization 0..N → ACP 1..N → Building` |
| **C3 — RBAC role + module** | Permissions différenciées par rôle ET par module | Partiel (role seul) | Matrice `role × module × action` (admin/syndic/comptable/owner) |
| **C4 — Modération neutre Communauté** | Syndic intervient SANS participer | Absente (syndic = participant) | Rôle `Moderator` sur tous modules communautaires |
| **C5 — Portefeuilles équipe** | Cabinet multi-gestionnaires partage des sélections | Absent | Entité backend `Portfolio` |
| **C6 — Délégation temporaire** | Owner mandaté en cas d'indisponibilité syndic | Absent | Assignment role avec `valid_until`, audit, motif |
| **C7 — Accessibilité universelle** | Plateforme utilisable par tous | Partiel | WCAG 2.1 AA généralisé (cf. [[a11y-wcag-aa-baseline]]) |
| **C8 — Testabilité refactor-safe** | Refactor FE sans casser silencieusement l'existant | Partiel (#550 a montré l'écart) | Suite caractérisation + RED-GREEN-BLUE + multi-rôle E2E (cf. [[fe-refactor-test-driven]]) |
| **C9 — RBAC granulaire par sous-rôle métier** | Permissions distinctes Encodeur ≠ Émetteur (split Comptable) | Absent (1 seul rôle "accountant") | 2 permissions : `invoice.create` (Encodeur) vs `expense.issue + call_for_funds.create` (Émetteur). Bouton "Émettre" désactivé pour Encodeur. |
| **C10 — Audit indépendant (Commissaire aux comptes)** | Vérification PCMN par tiers indépendant avant AG (Art. 3.89 CC) | Absent | Rôle lecture seule + génération certificat vérification signé numériquement + workflow signature avant clôture comptes annuels. |
| **C11 — Mandats bornés intervenants externes** | Avocat / Notaire / AMO ont accès lecture mandaté avec `valid_until` strict | Absent | Entité `Mandate` (issued_by_acp, role, valid_from, valid_until, scope, motif). Audit tout accès. |
| **C12 — État daté Art. 577 CC (mutations)** | Notaire produit l'état daté lors d'une vente | Partiel (à vérifier dans #439 EtatDate cluster) | Endpoint dédié notaire `GET /acps/:id/units/:unit_id/etat-date` avec auth mandat + génération PDF signé. |
| **C13 — Conseil de Copropriété droit de regard** | CdC supervise syndic, droit d'alerter AG (Art. 3.88 CC) | Absent | Vue lecture étendue + action `create_alert` visible AG + audit. |
| **C14 — PWA ultra-simplifiée + magic link pour Contractor** | Électricien/jardinier/poubelles ne créent pas de compte ; reçoivent un lien magique par email/SMS, ouvrent une PWA ultra-épurée pour leur intervention. | Absent | PWA dédiée `/c/<token>` (token signé, single-use ou borné dans le temps). 3 écrans max : voir la demande, déclarer intervention, soumettre photo + facture. **Hors design system principal** (mobile-first, gros boutons, sans menus). Magic link via email/SMS, expirable, scope = 1 ticket/devis. |
| **C15 — AG à distance + hybride (Art. 3.87 §4 CC)** | AG 100% virtuelle OU mix présentiel + distance, avec vote distant légal (procuration ou vote direct), quorum agrégé. | Absent | Workflow AG enrichi : mode `in_person` / `remote` / `hybrid` ; intégration visio (renvoi vers Jitsi/Whereby ou équivalent souverain) ; vote distant authentifié (cf. #48 itsme/eID promu in-scope pour distant) ; quorum agrégé `attendees_in_person + attendees_remote + represented_by_proxy`. Signatures électroniques pour PV. |

## 5. Bounded Contexts DDD affectés

| BC | Existant | Modification |
|---|---|---|
| **Identity & Access** | `Organization`, `User`, `UserRole`, `UserRoleAssignment` | + `ACP` (nouvelle racine d'agrégat) ; refacto `Building.organization_id` → `Building.acp_id` + `ACP.organization_id?` ; + entité `MagicLink` (token signé single-use ou borné) pour Contractor/intervenants externes ; + entité `Mandate` (avocat/notaire/AMO mandatés ACP avec valid_until + scope) ; sous-rôles `accountant.encodeur` vs `accountant.emetteur` |
| **Property Management** | `Building`, `Unit`, `Owner` | `Building.acp_id` (FK) ; queries `list_buildings_for_role(role, user_id)` filtrent par ACP autorisée ; endpoint dédié notaire `GET /acps/:id/units/:unit_id/etat-date` (Art. 577 CC) avec auth mandat |
| **Governance** | `Meeting`, `Resolution`, `Vote`, `Convocation` | RBAC strict par ACP : un syndic A ne voit JAMAIS les AG d'une ACP du cabinet B. + **AG mode** : `in_person` / `remote` / `hybrid` ; quorum agrégé `present + remote + proxy` ; vote distant authentifié (lien #48 itsme/eID) ; signatures électroniques PV ; intégration visio (Jitsi/Whereby/équiv. souverain). + `CdC` (élus par AG, droit d'alerter) + `CommissaireAuxComptes` (vérification PCMN signée). |
| **Accounting (PCMN)** | `JournalEntry`, `Expense`, `ChargeDistribution`, `CallForFunds` | Split permissions Encodeur (`invoice.create` only) vs Émetteur (`expense.issue` + `call_for_funds.create`). Commissaire = `read-only` global + `commissaire.sign_certificate`. |
| **Community** | `SEL`, `Reservation`, `SharedObject`, `Notice`, `Poll` | Nouveau rôle `Moderator` (CRUD admin sans participation) ; sauf `Reservation` qui autorise "au nom de l'ACP" pour syndic. CdC = participant normal copropriétaire. |
| **Maintenance & Operations** | `Ticket`, `WorkReport`, `TechnicalInspection` | Contractor accède via magic link à 1 ticket/devis spécifique uniquement ; PWA ultra-simplifiée (`/c/<token>`). Gardien = accès opérationnel ACP (CRUD tickets/réservations). AMO+Architecte+BET = lecture étendue + propositions chiffrées (vote AG décide). |
| **Portfolio** (nouveau BC) | — | Entité `Portfolio` (table `portfolios`) : N favoris immeubles + partage équipe gestionnaires |
| **External Mandates** (nouveau BC transverse) | — | Entité `Mandate` (issued_by_acp, role_type=avocat/notaire/amo, valid_from, valid_until, scope, motif, audit). Workflow : ACP mandate → token magique → accès lecture borné. |

## 6. Glossaire métier (additions au glossaire parent)

| Terme | Définition |
|---|---|
| **ACP** | *Association des Copropriétaires*. Entité morale belge Art. 3.84 CC. Composée des copropriétaires d'1 ou plusieurs immeubles (rare). Distincte du syndic (mandataire). |
| **Cabinet syndic** | Personne morale (`Organization` dans le code) prestataire de services pour 0..N ACPs. Une ACP peut être auto-gérée (Org=null). |
| **Portefeuille** | Sélection persistante d'immeubles/ACPs d'un gestionnaire pour pré-filtrer son interface. Partageable au sein d'un cabinet. |
| **Délégation temporaire** | Attribution d'un rôle (typiquement syndic ou président AG) à un owner pour une période bornée et un motif tracé. |
| **Modérateur Communauté** | Acteur (syndic ou admin) qui peut éditer/supprimer le contenu communautaire mais ne peut PAS créer/voter/participer en son nom propre. |
| **Bannière contextuelle 3-niveaux** | Élément UI permanent affichant `Immeuble · ACP · Cabinet syndic` quand un immeuble est sélectionné. Désambiguïse les ACPs homonymes. |

## 7. Invariants métier (à enforcer dans le domaine)

| ID | Invariant | Conséquence si violé |
|---|---|---|
| **INV-1** | Un `Building` appartient à exactement **1 ACP**. | Fuite données cross-ACP. |
| **INV-2** | Une `ACP` a **0 ou 1** `Organization` mandataire. | Confusion responsabilité syndic. |
| **INV-3** | Un user avec rôle `syndic` ne peut accéder à `Building.X` que si `Building.X.ACP.organization_id == user.organization_id`. | Bug RBAC majeur, risque légal. |
| **INV-4** | Un user avec rôle `syndic` ne peut PAS participer (`create`, `vote`, `comment` en son nom propre) aux modules `SEL`, `Poll`, `Notice`, `SharedObject`. | Conflit d'intérêt. |
| **INV-5** | Un user avec rôle `syndic` peut créer une `Reservation` SI elle est marquée `on_behalf_of_acp = true` (AG, prestataire). | Workflow AG cassé. |
| **INV-6** | Un user avec rôle `accountant` n'a JAMAIS accès aux URLs/modules `Community`. | UX confuse / accès non autorisé. |
| **INV-7** | Un user avec rôle `owner` ne voit QUE les ACPs où il possède au moins 1 unit. | Fuite données. |
| **INV-8** | Toute délégation temporaire a `valid_until > NOW()` et `motif` non vide. | Délégation zombie. |
| **INV-9** | Un `Portfolio` est lisible par son owner et par les autres membres de la même `Organization`. | Fuite cross-cabinet. |
| **INV-10** | Un user `accountant.encodeur` ne peut PAS appeler `expense.issue` ni `call_for_funds.create` (uniquement `invoice.create`). | Bypass séparation des tâches comptables, fraude possible. |
| **INV-11** | Un user `commissaire_aux_comptes` accède uniquement en **lecture** au PCMN de l'ACP qui l'a désigné ; aucune écriture sauf `commissaire.sign_certificate`. | Conflit d'intérêt + audit non indépendant. |
| **INV-12** | Un membre `CdC` a un mandat avec `valid_from` et `valid_until` issus d'un vote AG ; passé `valid_until`, perte automatique des droits étendus (devient owner standard). | CdC zombie après fin de mandat. |
| **INV-13** | Un `Contractor` ne voit QUE ses propres devis/missions/factures via son magic link ; aucun cross-contractor. | Fuite données concurrence. |
| **INV-14** | Un `Mandate` (avocat/notaire/AMO) a `valid_until` strict ; toute action après expiration → 403 + audit. Le scope du mandat (documents accessibles) ne peut PAS être étendu sans nouveau mandat. | Accès non autorisé à documents juridiques. |
| **INV-15** | Un `Notaire` accède UNIQUEMENT à 1 unit + son owner sortant + état daté Art. 577 CC pour la mutation tracée ; jamais aux autres units. | Fuite données copropriétaires. |
| **INV-16** | Un `Gardien` n'accède JAMAIS aux données financières (charges, appels de fonds, factures) ; uniquement tickets/réservations/incidents. | Fuite données financières à un employé non autorisé. |
| **INV-17** | Un `MagicLink` (Contractor) a un `expires_at` strict (max 30 jours par défaut, configurable) + `consumed_at` si single-use ; impossible de réutiliser après expiration ou consommation. | Accès persistant non révoqué = brèche sécurité. |
| **INV-18** | Une AG en mode `remote` ou `hybrid` exige authentification forte pour le vote distant (cf. #48 itsme/eID) — pas de vote distant sans auth forte. | Vote frauduleux, AG invalide juridiquement. |
| **INV-19** | Quorum d'une AG hybride = `attendees_in_person + attendees_remote + represented_by_proxy_quotas` (Art. 3.87 §4 CC). Aucun double-comptage. | Quorum calculé faux, décisions AG invalides. |
| **INV-20** | Un PV d'AG distance/hybride DOIT avoir au moins 2 signatures électroniques qualifiées (président + secrétaire) avant clôture. | PV non opposable juridiquement. |

## 8. Stratégie tests (intégrée Maury — cf. mémoire [[fe-refactor-test-driven]])

Refonte FE conséquente → **3 niveaux de tests pilotent la refonte** (pas qu'a posteriori) :

### Niveau 1 — Caractérisation (AVANT toute modif)

Suite `frontend/tests/e2e/characterization/` à créer en Phase 2 (PRD) qui capture le comportement EXISTANT qui doit rester :

- Login + dashboard syndic (déjà couvert partiellement, à enrichir)
- Création immeuble admin → visible syndic après assignation
- Création AG syndic → flow complet jusqu'à clôture
- Création expense + appel de fonds
- Vue copropriétaire de ses lots

Cible : 100% des tests caractérisation **verts** en `feature/dev` HEAD avant la slice 1 de refonte.

### Niveau 2 — TDD RED-GREEN-BLUE Vitest sur composants

Chaque composant nouveau/refacto suit le cycle :
- **RED** test échoue sur comportement attendu
- **GREEN** code minimal pour passer
- **BLUE** révision architecture + invariants ([[a11y-wcag-aa-baseline]], [[data-testid-systematic]])

### Niveau 3 — E2E Playwright multi-rôle

Scénarios narratifs avec acteurs corrects (cf. [[multirole-narrative-scenarios]]) couvrant les 4-cat (`@happy + @edge + @security + @negative`) :

- Admin crée ACP → assigne Organization → syndic voit ACP
- Syndic switche immeuble via sélecteur → menus se contextualisent → bannière 3-niveaux exacte
- Syndic tente de voter dans un sondage → **bloqué** (INV-4)
- Syndic crée Reservation `on_behalf_of_acp` → autorisée (INV-5)
- Comptable accède `/community/*` → **403** (INV-6)
- Owner d'ACP A tente accès ACP B → **403** (INV-7)
- Cabinet B gestionnaire tente accès Portfolio cabinet A → **403** (INV-9)
- **Encodeur Paul** crée facture → OK ; tente `expense.issue` → **403** (INV-10)
- **Commissaire Henri** lit le journal → OK ; tente édit écriture → **403** (INV-11) ; signe certificat → audit + signature persistée
- **CdC Catherine** consulte comptes → OK + droit alerte ; après `valid_until` → perd droits étendus (INV-12)
- **Contractor Bruno** ouvre magic link → voit 1 devis ; tente accès `/c/<other-token>` → **403** (INV-13)
- **Avocat Léa** mandat expiré tente accès → **403** + audit (INV-14)
- **Notaire Sophie** demande état daté unit X → OK ; tente lecture unit Y → **403** (INV-15)
- **Gardien Karim** crée ticket → OK ; tente accès `/expenses` → **403** (INV-16)
- Magic link expiré tente réutilisation → **403** (INV-17)
- AG mode `remote`, copropriétaire tente vote distant **sans itsme/eID** → **403** (INV-18)
- AG hybride : présentiel 30% + distant 25% + proxy 10% → quorum agrégé 65% > 50% → AG valide (INV-19)
- PV AG hybride clôturé sans 2 signatures électroniques → **403** (INV-20)
- Comptable Encodeur Paul tente accès depuis cabinet B sur ACP cabinet A → **403** (INV-3 + INV-10 cumulés)

## 9. Hors-scope (volontaire)

- Refonte visuelle/branding (logo, palette) — slice esthétique séparée
- Refonte API publique (#111) — Phase ecosystem
- Tauri Desktop/Mobile (#295-298) — Phase 2
- Multi-immeubles par ACP (cas rare mitoyens) : modélisé mais UX laissée minimale (pas de carte/plan, juste discrimination dropdown)
- Crowdlending #353 — R&D
- **Strong auth voting itsme/eID (#48)** — ⚠️ **Promu in-scope** par C15 (AG distance/hybride) : le vote distant légal exige l'auth forte ; ne plus reporter en Phase k8s pour cette refonte.
- Visio intégrée native (Jitsi/Whereby) : pour C15, on **redirige** vers un service externe avec lien dans la convocation, pas d'intégration code v1.
- Signature électronique qualifiée pour PV : pour C15/INV-20, **eIDAS-compliant requis** — choix prestataire (Universign, DocuSign EU, ou eID belge) = ADR dédié en Phase 3 Architecture.

## 10. Critères de succès (mesurables)

| # | Critère | Métrique cible | Vérifiable par |
|---|---|---|---|
| **SC1** | Zéro fuite cross-ACP | Test `@security` Playwright VERT (syndic cabinet A ne voit pas ACP cabinet B) | Phase 4 stories |
| **SC2** | Zéro syndic participant en Communauté | Tests `@security` sur SEL/Poll/Notice/SharedObject VERTS | Phase 4 stories |
| **SC3** | Sélecteur immeuble présent sur 100% des pages contextuelles | Audit Playwright snapshot | Phase 4 stories |
| **SC4** | Bannière 3-niveaux affichée quand building sélectionné | Test E2E @happy | Phase 4 stories |
| **SC5** | Comptable n'accède pas à `/community/*` | Tests `@security` 403 | Phase 4 stories |
| **SC6** | Lighthouse a11y ≥ 90 sur pages nouvelles | CI gate axe-core | Phase 4 stories |
| **SC7** | Suite caractérisation reste VERTE de la slice 1 à la slice N | CI sur chaque PR de refonte | Phase 4 stories |
| **SC8** | Délégation temporaire fonctionnelle + auditée | Test `@happy` + `@negative` (valid_until expiré) | Phase 4 stories |
| **SC9** | Zéro bypass de sous-rôle (encodeur≠émetteur, gardien≠compta, etc.) | Tests `@security` 403 sur toutes les permutations role × action interdite | Phase 4 stories |
| **SC10** | Commissaire aux comptes signe certificat de vérification PCMN | Test `@happy` (signature persistée + audit) + `@negative` (édition tentée → 403) | Phase 4 stories |
| **SC11** | PWA Contractor magic link fonctionnelle de bout-en-bout | Test E2E mobile (Playwright `--device "Pixel 7"`) : magic link → 3 écrans → soumission OK. Expiration → 403 | Phase 4 stories |
| **SC12** | AG hybride : quorum agrégé correct + auth forte distante + 2 signatures PV | Tests BDD scénarios INV-18/19/20 verts + flow E2E complet AG hybride avec rôles multi-acteurs (président + secrétaire + 1 présentiel + 1 distant + 1 proxy) | Phase 4 stories |
| **SC13** | Mandats externes (Avocat/Notaire/AMO) bornés `valid_until` + scope respecté | Test `@negative` accès après expiration → 403 + audit ; test scope (notaire ne lit que SA unit) | Phase 4 stories |

## 11. Dépendances / contraintes

- **Cluster #433 Decimal** : la refonte ne crée pas de nouveau `f64` ; tout montant exposé est `Decimal`-as-string. Si refacto d'un use-case touche un fichier de #433, faire les 2 migrations dans la même PR (cf. meta-comment #433).
- **Epic #555 Result<_, String>** : tout use-case touché par cette refonte migre simultanément vers `AppError` (cf. règle CRITICAL.md §4).
- **WBS v0.1.0 (#549)** : la slice S1 (refacto domaine ACP) est candidate à entrer dans WBS Track H comme bloqueur légal — à arbitrer en Phase 2 PRD.
- **Mémoires d'agent transverses** : 8 mémoires applicables (cf. frontmatter `memories_applied`), à respecter sans dérogation.

## 12. Signature & GATE

Ce brief est **DRAFT** en attente de signature humaine.

Sign-off humain (@gilmry) requis avant ouverture de Phase 2 (PRD) :

- [ ] Vision validée
- [ ] **13 personas** validés (incl. Contractor PWA magic link, CdC Art. 3.88 CC, Commissaire Art. 3.89 CC, split Comptable Émetteur/Encodeur, Avocat/Notaire/AMO mandatés, Gardien employé ACP)
- [ ] Modèle ACP `0..1 Org → 1..N ACP → 1..N Building` validé
- [ ] Matrice rôles/modules + sous-rôles métier (syndic modérateur, comptable split, mandats bornés) validée
- [ ] **C14 — PWA Contractor + magic link** (3 écrans, expirable, scope=1 ticket) validée
- [ ] **C15 — AG distance/hybride** (mode `in_person`/`remote`/`hybrid`, quorum agrégé, auth forte distante #48 promu in-scope, 2 signatures électroniques PV) validée
- [ ] Stratégie test-driven 3-niveaux acceptée
- [ ] **20 invariants** validés (INV-1 à INV-20)
- [ ] Périmètre hors-scope validé (incl. déprécession #48 hors-scope)
- [ ] **13 critères de succès SC1-SC13** validés
- [ ] Coordination identifiée : cluster #433 Decimal + epic #555 Result<_, String> + WBS Track H

**Date signature** : _à compléter_
**Signature** : _@gilmry_
