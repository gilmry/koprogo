---
feature: refonte-ux-multi-role-acp
phase: stories
phase_togaf: E (Solutions)
agent_bmad: Bob (Scrum Master)
authors: [Gilles Maury, Farah Maury]
date: à compléter
version: 0.0
status: Blocked — waiting Phase 3 (Architecture) sign-off
architecture_source: architecture.md (Winston, v0.0)
---

# Epics & User Stories — Refonte UX multi-rôle + modèle ACP

## Methode Maury — Phase TOGAF E (Solutions)

> ⛔ **Phase Blocked** — ce document ne peut être démarré qu'après signature de `architecture.md` par @gilmry.

---

## Sections imposées (à remplir en Phase 4)

### Vue d'ensemble

| Phase | Story | Type | Effort | Débloque |
|---|---|---|---|---|
| 0 | Tests de caractérisation (régression safety net) | Test infrastructure | M | Toute slice de refonte sans casser l'existant |
| A | **S1** — Refacto domaine `Organization 0..N → ACP 1..N → Building` + migration data | Refactor + migration | L | S2, S3, S4 |
| B | **S2** — Sélecteur global + menus contextuels + bannière 3-niveaux | Feature UX | M | UX cohérente multi-rôle |
| C | **S3** — Entité `Portfolio` + favoris + admin crée Organizations | Feature backend + UI | M | Cabinet multi-gestionnaires |
| D | **S4** — RBAC Communauté (Moderator) + délégation temporaire syndic→owner | Feature RBAC | M-L | Conflit d'intérêt résolu + cas syndic indisponible |

### Stratégie tests par story (rappel mémoire [[fe-refactor-test-driven]])

Chaque story commence par :
1. **RED** Vitest sur composants nouveaux/refactorés
2. **RED** Playwright multi-rôle 4-cat
3. **Caractérisation** continue de tourner VERT

Puis cycle **GREEN** (code minimal) → **BLUE** (refactor + invariants).

### Détail par story (template à remplir)

Pour chaque story (S1, S2, S3, S4) :

- **ID** : STORY-NNN-X
- **Issue parent** : Epic GitHub (à créer post-signature stories.md)
- **User Story** : *En tant que [persona], quand [contexte], je veux [action], afin de [bénéfice]*
- **Scope** : entités/composants touchés
- **Pré-conditions techniques** : stories antérieures complétées
- **Scenarios BDD** (4 catégories obligatoires)
- **Critères d'acceptation** (checklist)
- **Effort** (S/M/L)
- **Définition « done »** : tous critères verts + tests caractérisation + invariants respectés

### Issues GitHub à créer (post-signature)

| Issue | Type | Titre prévu | Labels prévus |
|---|---|---|---|
| Epic | Tracking | Refonte UX multi-rôle + modèle ACP — pipeline Maury | epic, track:software, governance |
| Sub S0 | Test infra | Suite de caractérisation FE (régression safety net) | testing, track:software |
| Sub S1 | Refactor + migration | Refacto domaine Organization → ACP → Building | rust, refactor, legal-compliance, governance, priority:high |
| Sub S2 | Feature UX | Sélecteur global immeuble + menus contextuels + bannière 3-niveaux | javascript, enhancement, priority:high |
| Sub S3 | Feature | Entité Portfolio + favoris cabinet multi-gestionnaires | rust, javascript, enhancement |
| Sub S4 | Feature RBAC | RBAC Communauté Moderator + délégation temporaire | rust, javascript, security, governance, legal-compliance |

### Intégration WBS `go-live v0.1.0`

Selon décision humaine 2026-05-20 (« quand ce sera validé, on créera les issues github qu'on intègrera dans le wbs golive 0.1.0 ») :

- **S1 (refacto ACP)** : candidat Track H (bloqueur légal cross-ACP)
- **S0 (caractérisation)** : transverse, peut être intégré comme pré-requis de toute slice WBS Track C/D
- **S2/S3/S4** : à arbitrer en Phase 4 — peut être Phase 1 ou Phase 2 selon priorité business

## Signature & GATE

- [ ] Architecture Phase 3 signée (prérequis)
- [ ] Stories rédigées selon template Maury
- [ ] BDD Gherkin 4-cat par story
- [ ] Sign-off humain @gilmry pour :
  - Création des issues GitHub
  - Intégration WBS go-live v0.1.0
  - Démarrage Phase 5 (Validation) puis Phase 6 (Exécution)
