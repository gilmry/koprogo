# Agent activity — 2026-09-16 — Decomposition de l'epopee #355 (restructuration IaC)

**Persona :** claude (Tier 2 — diagnostic, proposal)

**Contexte :** Implementation demandee de l'issue #355 ("refactor(infra):
Restructuration IaC — repo separe, tests, policy-as-code"). La story jointe
(gabarit BMAD phase E, "Agent IA Ready — C9.2") precise que #355 est une
**epopee-borne** : sa Definition of Done n'est pas la restructuration
elle-meme mais **le decoupage de ses sous-chantiers et leur ordre**, avec
#354 (tests IaC) en premier et une detection de divergence posee. Avant de
rediger ce decoupage, audit de l'etat reel du depot pour eviter de proposer
un ordre qui ignore du travail deja fait (ou deja perime).

## Actions Tier 2 effectuees

- Exploration du depot (`Explore` en arriere-plan) pour verifier l'etat reel
  de #354, #466, et de la structure infra avant de rediger le decoupage.
  Constats cles :
  - `.github/workflows/ci-infra.yml` existe deja et couvre une partie de
    #354 (lint terraform/ansible/yaml/shell, scope VPS production
    uniquement).
  - Des policies OPA ISO 27001 (`infrastructure/_shared/conftest/policies/iso27001/`)
    et des scenarios molecule (3 roles) existent sur disque (commit
    `b8565719`) mais sont **non relus** et **exclus du gate CI**.
  - `docs/governance/rfc/0001-gitops-multi-environment-strategy.rst` affiche
    `:Statut: Draft` alors que sa decision (alternative F, mono-repo,
    branches infra symetriques) est deja deployee en CI depuis le commit
    `42d48158` (2026-05-13). Aucun label `accepted` sur #466.
  - La premisse "repo separe" de #355 (documenter `koprogo-infra-restructure`
    comme second depot) ne correspond plus a la decision RFC 0001
    (alternative D "2 repos" rejetee ; alternative F mono-repo retenue).
    Aucun tooling de synchronisation entre deux depots n'existe nulle part
    dans le code.
  - `docs/WBS_PROJET_COMPLET.rst` §9.2 (version du 2026-03-29) attribuait a
    tort ~20h de travail de code a #355 — contradiction avec la story
    actuelle qui exclut explicitement "la restructuration elle-meme" du
    perimetre de #355.
- Mise a jour de `docs/WBS_PROJET_COMPLET.rst` §9.2 : correction de
  l'attribution perimee, ajout de l'etat reel constate, et de l'ordre des
  sous-chantiers (#354 en premier, #466 en reconciliation avant tout le
  reste, "structure repo" et "CI/CD dedie" et "documentation" ensuite).
  Conception (texte, pas de code) d'un mecanisme de detection de divergence
  documentation <-> implementation, a construire dans le futur sous-chantier
  CI/CD.

## Ce qui n'a PAS été exécuté (hors délégation / hors portée session)

- Aucune modification du statut de `docs/governance/rfc/0001-*.rst` (passer
  de `Draft` a `Accepted`) : c'est une signature humaine explicitement
  requise par le processus RFC du depot (template §"Decision") — flaggee en
  🔴 dans le WBS, pas tranchee ici.
- Aucune creation d'issue GitHub pour les sous-chantiers 3/4/5 identifies
  (structure repo, CI/CD dedie, documentation) : creation d'issue = decision
  publique tracee, geste humain (regle CRITICAL #6/#11).
- Aucun code ecrit pour #354 (revue des policies OPA/molecule non relues,
  extension du lint aux 14 roles) ni pour le detecteur de divergence propose
  : la story #355 exclut explicitement "la restructuration elle-meme" de son
  propre perimetre — c'est le travail des sous-chantiers ouverts separement.
- Pas de `git push` ni de PR : modification de fichiers seulement, la
  promotion reste un geste humain (consigne de session).

## Vérification

- Lecture directe de `.github/workflows/ci-infra.yml`,
  `docs/governance/rfc/0001-gitops-multi-environment-strategy.rst`,
  `docs/github-export/issues/issue-354.rst`,
  `docs/github-export/issues/issue-466.rst`, `infrastructure/README.md`,
  `infrastructure/Makefile.infra`, `infrastructure/SECURITY.md` (recherche
  "ISO 27001" / "mapping" : aucun resultat, confirmant l'absence de table de
  correspondance).
- `grep -rl "koprogo-infra-restructure"` sur `.github/workflows/`,
  `infrastructure/`, `scripts/` : aucun resultat — confirme l'absence de
  tooling de synchronisation entre deux depots.
- Pas de commande cargo/npm necessaire (travail documentaire uniquement).
