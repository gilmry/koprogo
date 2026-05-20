---
feature: refonte-ux-multi-role-acp
phase: brief
phase_togaf: A (Vision)
agent_bmad: Mary (Analyste TOGAF)
authors: [Gilles Maury, Farah Maury]
date: 2026-05-20
version: 0.1
status: Draft awaiting human sign-off
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

Repris du brief parent (`Maury/product-brief.md`) avec affinage spécifique :

| Persona | Description | Besoin clé de la refonte |
|---|---|---|
| **Mathilde — Admin SaaS** | Superadmin de la plateforme. Crée des Organizations pour les nouveaux cabinets syndic. | Vue cross-ACP, sélecteur d'ACP, création/maintenance Organizations, modération méta sur Communauté. |
| **Sylvie — Syndic professionnelle (cabinet)** | Mandatée par N ACPs. Vit 80% de son temps sur l'app. | Tableau de bord cross-immeubles + sélecteur immeuble + favoris + bannière contextuelle 3-niveaux (immeuble · ACP · cabinet). |
| **Pierre — Comptable copro** | Gère la PCMN belge pour les ACPs du cabinet. Pas de rôle dans la vie sociale. | Mêmes sélecteur + menus contextuels que Sylvie, mais **section Communauté masquée**. |
| **Marie — Copropriétaire** | Possède 1 lot dans 1 immeuble. Vote, paye, échange en SEL. | Vue restreinte à ses immeubles + bannière sans choix (1 immeuble). Participation pleine à la Communauté. Possibilité de **délégation temporaire** si syndic indisponible (cas rare mais critique : mandat AG d'urgence). |
| **Cabinet multi-gestionnaire** (nouveau persona) | Un cabinet Sylvie' équipe de N gestionnaires. Chacun voit toutes les ACPs du cabinet mais a son portefeuille préféré. | Entité `Portfolio` backend + UI favoris persistants multi-device. |

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

## 5. Bounded Contexts DDD affectés

| BC | Existant | Modification |
|---|---|---|
| **Identity & Access** | `Organization`, `User`, `UserRole`, `UserRoleAssignment` | + `ACP` (nouvelle racine d'agrégat) ; refacto `Building.organization_id` → `Building.acp_id` + `ACP.organization_id?` |
| **Property Management** | `Building`, `Unit`, `Owner` | `Building.acp_id` (FK) ; queries `list_buildings_for_role(role, user_id)` filtrent par ACP autorisée |
| **Governance** | `Meeting`, `Resolution`, `Vote`, `Convocation` | RBAC strict par ACP : un syndic A ne voit JAMAIS les AG d'une ACP du cabinet B |
| **Community** | `SEL`, `Reservation`, `SharedObject`, `Notice`, `Poll` | Nouveau rôle `Moderator` (CRUD admin sans participation) ; sauf `Reservation` qui autorise "au nom de l'ACP" pour syndic |
| **Portfolio** (nouveau BC) | — | Entité `Portfolio` (table `portfolios`) : N favoris immeubles + partage équipe gestionnaires |

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

## 9. Hors-scope (volontaire)

- Refonte visuelle/branding (logo, palette) — slice esthétique séparée
- Refonte API publique (#111) — Phase ecosystem
- Tauri Desktop/Mobile (#295-298) — Phase 2
- Multi-immeubles par ACP (cas rare mitoyens) : modélisé mais UX laissée minimale (pas de carte/plan, juste discrimination dropdown)
- Crowdlending #353 — R&D
- Strong auth voting itsme/eID (#48) — Phase k8s

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

## 11. Dépendances / contraintes

- **Cluster #433 Decimal** : la refonte ne crée pas de nouveau `f64` ; tout montant exposé est `Decimal`-as-string. Si refacto d'un use-case touche un fichier de #433, faire les 2 migrations dans la même PR (cf. meta-comment #433).
- **Epic #555 Result<_, String>** : tout use-case touché par cette refonte migre simultanément vers `AppError` (cf. règle CRITICAL.md §4).
- **WBS v0.1.0 (#549)** : la slice S1 (refacto domaine ACP) est candidate à entrer dans WBS Track H comme bloqueur légal — à arbitrer en Phase 2 PRD.
- **Mémoires d'agent transverses** : 8 mémoires applicables (cf. frontmatter `memories_applied`), à respecter sans dérogation.

## 12. Signature & GATE

Ce brief est **DRAFT** en attente de signature humaine.

Sign-off humain (@gilmry) requis avant ouverture de Phase 2 (PRD) :

- [ ] Vision validée
- [ ] Personas validés (incl. nouveau persona « cabinet multi-gestionnaire »)
- [ ] Modèle ACP `0..1 Org → 1..N ACP → 1..N Building` validé
- [ ] Matrice rôles/modules (syndic = modérateur sans participation, comptable sans Communauté) validée
- [ ] Stratégie test-driven 3-niveaux acceptée
- [ ] Périmètre hors-scope validé
- [ ] Critères de succès SC1-SC8 validés

**Date signature** : _à compléter_
**Signature** : _@gilmry_
