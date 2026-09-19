---
version: vX.Y.Z
date: YYYY-MM-DD
reviewer: <human-name>
cowork_session_id: <claude-cowork-session-or-none>
environment: staging|preview|prod
duration_hours: <n>
verdict: NO_GO
verdict_justification: >
  <à remplir par l'humain — jamais laisser vide, jamais laisser à GO par
  défaut : NO_GO est la valeur de repos tant que personne n'a signé.>
signed_by: <à remplir uniquement au moment de signer>
signed_at: <iso-timestamp — à remplir uniquement au moment de signer>
---

# Rapport de Revue Humaine — KoproGo <VERSION>

> Gabarit versionné — extrait et paramétré depuis
> `docs/HUMAN_REVIEW_REPORT_v0.1.0.md` (revue historique du 2026-04-01, qui
> concluait elle-même « NO-GO pour release publique » — cf. #427).
>
> Copier ce fichier vers
> `docs/maury/releases/<VERSION>/human-review-report.md`. Le champ
> `verdict` du frontmatter est lu mécaniquement par le gate de release
> (workflow `.github/workflows/release-tag.yml`) : seul `GO` autorise la
> création du tag. `GO_CONDITIONAL` et `NO_GO` bloquent — voir §Verdict.
>
> **Seul un humain édite `verdict` vers `GO` et remplit `signed_by` /
> `signed_at`.** Une session Cowork-Chrome ne produit qu'un brouillon
> `GO_CONDITIONAL` (cf. skill `cowork-release-review`, prévu sous
> `.claude/skills/` — non créé dans cette session, écriture sous `.claude/`
> bloquée sans approbation humaine en direct).

**Date** : <DATE>
**Réviseur** : <humain, éventuellement assisté d'une session Cowork>
**Environnement** : <URL staging/preview/prod>
**Durée** : <durée réelle>
**Méthodologie** : <navigation UI + appels API directs quand l'UI bloque, etc.>

---

## Résumé Exécutif

<2-4 phrases : l'état général, ce qui marche, ce qui bloque. Pas
d'euphémisme — le rapport v0.1.0 devait pouvoir dire lui-même « NO-GO pour
release publique », et il l'a dit.>

---

## Workflows Testés

### SESSION <n> — <thème>

#### WF<n> : <nom du workflow> (Art. <référence légale si pertinent>)

| Étape     | Résultat             | Détails     |
| --------- | -------------------- | ----------- |
| <étape 1> | ✅ OK / ⚠️ PARTIEL / ❌ BUG | <détail> |

**Bugs identifiés :**

- **BUG-WF<n>-<m>** [CRITIQUE\|MAJEUR\|MINEUR] : <description> — <impact>

<Répéter par workflow et par session, en suivant `human-review-plan.md`.>

---

## Inventaire des Bugs

### Critiques (bloquants pour release)

| ID | Workflow | Description | Impact |
| -- | -------- | ------------ | ------ |
|    |          |              |        |

### Majeurs (à corriger avant beta/GA)

| ID | Workflow | Description | Impact |
| -- | -------- | ------------ | ------ |
|    |          |              |        |

### Mineurs (à corriger pour la version suivante)

| ID | Workflow | Description | Impact |
| -- | -------- | ------------ | ------ |
|    |          |              |        |

---

## Points Forts

1. <ce qui fonctionne bien, pour ne pas noyer le signal dans les bugs>

---

## Verdict

### <GO | GO_CONDITIONAL | NO_GO>

**Justification :** <reprend `verdict_justification` du frontmatter, en
prose>.

**Conditions pour passer en GO** (si `GO_CONDITIONAL` ou `NO_GO`) :

1. <bug critique à corriger>
2. <bug majeur à corriger>

**Aucun bug `[CRITIQUE]` non résolu ne doit subsister pour signer `GO`.**
Le gate mécanique (`.github/workflows/release-tag.yml`) ne vérifie que le
champ `verdict` — c'est à l'humain qui signe de refuser `GO` tant qu'un
bug `[CRITIQUE]` reste ouvert.

---

## Statistiques de la Revue

| Métrique                | Valeur |
| ------------------------ | ------ |
| Workflows testés         |        |
| Bugs critiques trouvés   |        |
| Bugs majeurs trouvés     |        |
| Bugs mineurs trouvés     |        |
| Rôles testés             |        |

---

## Conditions de Re-Review

<Si `GO_CONDITIONAL` : quels correctifs déclenchent une nouvelle session de
revue avant de pouvoir passer à `GO` ? Sois précis — une condition vague ne
protège personne.>

---

*Rapport généré le <DATE> par <réviseur/session>.*
*Basé sur le plan de revue : `docs/maury/releases/<VERSION>/human-review-plan.md`.*
