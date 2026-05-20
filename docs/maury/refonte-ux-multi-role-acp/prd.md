---
feature: refonte-ux-multi-role-acp
phase: prd
phase_togaf: B-C (Business + SI)
agent_bmad: John (Product Manager)
authors: [Gilles Maury, Farah Maury]
date: à compléter
version: 0.0
status: Blocked — waiting Phase 1 (Brief) sign-off
brief_source: brief.md (Mary, v0.1, 2026-05-20)
---

# PRD — Refonte UX multi-rôle + modèle ACP

## Methode Maury — Phase TOGAF B-C (Business + SI)

> ⛔ **Phase Blocked** — ce document ne peut être démarré qu'après signature du `brief.md` par @gilmry.

---

## Sections imposées (à remplir en Phase 2)

1. Résumé exécutif
2. Objectifs produit (mesurables, traçables vers brief section 10)
3. Périmètre (In scope / Out of scope, tracés vers brief sections 4 et 9)
4. Exigences fonctionnelles (FRs numérotées FR1, FR2, …)
   - Pour chaque FR : description / acteur / pré-conditions / post-conditions / invariants impactés (brief section 7)
   - Matrice **4×N BDD Gherkin** (@happy / @edge / @security / @negative) — obligatoire
5. Exigences non-fonctionnelles (perf, sécu, a11y WCAG 2.1 AA, i18n FR/NL/EN/DE)
6. Stratégie tests caractérisation (cf. mémoire [[fe-refactor-test-driven]])
   - Liste des flows existants à figer AVANT toute refonte
   - Suite `frontend/tests/e2e/characterization/` à créer
7. Dépendances cross-stories (#433 Decimal, #555 Result, #553/#554)
8. Plan de release (slices déployables, séquence)
9. Risques + mitigation
10. Gate de validation Phase 2 — sign-off humain

## Signature & GATE

- [ ] Brief Phase 1 signé (prérequis)
- [ ] PRD rédigé selon template Maury
- [ ] FRs avec matrice 4×N BDD complète
- [ ] Sign-off humain @gilmry pour ouvrir Phase 3 (Architecture)
