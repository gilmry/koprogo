# Plan de Revue Humaine — KoproGo <VERSION>

**Pour** : <RÉVISEUR> (revue manuelle, éventuellement assistée par Cowork-Chrome)
**URL** : <ENVIRONNEMENT_URL> (staging/preview/prod)
**Durée estimée** : <N> sessions de <H>h (total ~<TOTAL_H>h)
**Date** : <DATE>
**Story/issue liée** : #427 (gate release C7.2)

> Gabarit versionné — extrait et paramétré depuis
> `docs/HUMAN_REVIEW_PLAN_v0.1.0.md` (revue historique du 2026-04-01).
> Copier ce fichier vers
> `docs/maury/releases/<VERSION>/human-review-plan.md` et remplir les
> `<PLACEHOLDERS>` avant la session. Ce fichier reste le gabarit ; ne pas
> l'éditer pour une release donnée.
>
> Emplacement prévu par #427 : `.claude/templates/`. Ce gabarit vit ici
> (`docs/templates/`) parce que l'écriture sous `.claude/` demande une
> approbation humaine en direct qu'une session agent autonome ne peut pas
> s'auto-accorder. Déplacement recommandé — `git mv` suffit — dès qu'un
> humain peut valider l'écriture.

---

## Conventions

- `[RÔLE]` → se connecter avec ce compte avant l'étape.
- `→` → naviguer vers cette page.
- `✓ Attendu :` → ce que tu dois observer pour valider.
- `✗ Bug :` → noter ici si ça ne marche pas (référence-le ensuite dans le
  report avec un ID `BUG-WF<n>-<m>` et une sévérité).
- Cocher `[ ]` quand validé.
- Un scénario multi-rôle change de compte **dans le test** — jamais un seul
  login pour tout un workflow (cf. `CLAUDE.md` §Multi-rôles E2E).

---

## 0. Setup — Comptes & Seed Data

<COMMENT_SEEDER> (ex. `POST /api/v1/seed/scenario/world`, ou via
`/admin/seed`).

### Comptes de test

| Prénom      | Email     | Mot de passe | Rôle     | Lot     | Tantièmes     |
| ----------- | --------- | ------------ | -------- | ------- | ------------- |
| <PERSONA_1> | <EMAIL_1> | <PASSWORD_1> | <RÔLE_1> | <LOT_1> | <TANTIEMES_1> |
| <PERSONA_2> | <EMAIL_2> | <PASSWORD_2> | <RÔLE_2> | <LOT_2> | <TANTIEMES_2> |

---

## SESSION 1 — <THÈME, ex. Conformité Légale AG>

> **Objectif** : <ce que cette session vérifie et pourquoi — référencer les
> articles de loi belge pertinents, ex. Art. 3.87-3.92 CC>.

### WORKFLOW 1 — <Nom du workflow> (Art. <référence légale si pertinent>)

**Règle légale** : <énoncé de la règle, si applicable>.

**[<RÔLE_A>]**

- [ ] → `<route>` → <action>
  - `✓ Attendu :` <observation attendue>
  - `✗ Bug :` ___________

**[<RÔLE_B>]** (switch de compte)

- [ ] → `<route>` → <action>
  - `✓ Attendu :` <observation attendue>

---

### WORKFLOW 2 — <Nom du workflow>

<Répéter la structure ci-dessus pour chaque workflow de la session.>

---

## SESSION 2 — <THÈME, ex. Financier / Comptable>

<Idem.>

## SESSION 3 — <THÈME, ex. Opérationnel — Tickets, SEL, Sondages, GDPR>

<Idem.>

## SESSION 4 — <THÈME, ex. Multi-rôle / Dashboards / Mobile / A11y>

<Idem.>

## SESSION 5 — <THÈME, ex. Sécurité — RBAC, injection, rate limit>

<Idem. Toujours inclure : accès non authentifié → 401, accès hors rôle →
403, tentative de contournement d'ownership (ex. voter sur un lot qu'on ne
possède pas) → 403, cf. `backend/tests/features/_security_baseline.feature`.>

---

*Document créé le <DATE_CRÉATION> à partir du gabarit
`docs/templates/HUMAN_REVIEW_PLAN.template.md`.*
