# Plan de Revue Humaine — KoproGo v0.1.0

**Pour** : le relecteur qui signe G1
**Où** : `http://localhost:8090` — la **pile de recette**, jetable
**Durée estimée** : 3-4 sessions de 2-3h (total ~8-10h)
**Date** : 2026-04-01, en-tête corrigé le 2026-09-18
**WBS** : Phase 8 de `docs/archive/WBS_RELEASE_0_1_0.md`

> ## ⚠️ Deux corrections d'en-tête, et pourquoi elles comptent
>
> **1. L'adresse.** Ce document a désigné pendant des mois le **port 80 nu**
> de cet hôte, avec un `staging.koprogo.com` en second choix qui n'a jamais
> existé. Or ce port 80 **est le Traefik de la démo** : un relecteur qui
> suivait ce plan créait des assemblées générales, envoyait des convocations
> et enregistrait des procurations **dans la démo publique**, en croyant
> travailler sur un bac à sable. L'ADR 0050 a décalé la recette sur **8090**
> précisément pour que les deux ne puissent plus être confondues.
>
> **2. Le mot de passe.** La table ci-dessous donnait en clair celui du
> superadministrateur. Il ne marche plus, et c'est voulu : il tourne
> au déploiement. Les identifiants se lisent dans l'environnement —
> `KOPROGO_SUPERADMIN_EMAIL` et `KOPROGO_SUPERADMIN_PASSWORD` — comme la
> suite e2e les lit (`frontend/tests/e2e/helpers/identifiants.ts`).
>
> ## ⚠️ Ce document est INCOMPLET, et il faut le savoir avant de commencer
>
> Il annonce « 3-4 sessions » et s'arrête au milieu de la **session 1**, sur
> la mention « *(voir document complet)* » qui ne renvoie nulle part. Sont
> écrits : le workflow 1 (convocation, Art. 3.87 §3) et l'ouverture du
> workflow 2 (les quatre majorités, Art. 3.88). **Tout le reste est à
> écrire** : comptabilité, RGPD, périmètre multi-rôle, modules
> communautaires.
>
> Le signer en l'état ferait attester une couverture qui n'existe pas. Ce
> qu'il couvre vaut d'être parcouru ; ce qu'il ne couvre pas doit être dit
> au moment de signer.

---

## Conventions

- `[RÔLE]` → se connecter avec ce compte avant l'étape
- `→` → naviguer vers cette page
- `✓ Attendu :` → ce que tu dois observer pour valider
- `✗ Bug :` → noter ici si ça ne marche pas
- Cocher `[ ]` quand validé

---

## 0. Setup — Comptes & Seed Data

L'application a un endpoint de seed. Lancer d'abord :

```bash
# Seed les 21 personas sur l'immeuble de test
POST /api/v1/seed/scenario/world
# ou via l'interface admin → /admin/seed
```

### Comptes de test

| Prénom        | Email                         | Mot de passe | Rôle          | Lot  | Tantièmes |
|---------------|-------------------------------|--------------|---------------|------|-----------|
| Alice Dubois  | alice@residence-parc.be       | alice123     | Copropriétaire (Présidente CdC) | 2A | 450 |
| Bob Janssen   | bob@residence-parc.be         | bob123       | Commissaire aux comptes | 2B | 430 |
| Charlie Martin| charlie@residence-parc.be     | charlie123   | Copropriétaire | 3B  | 660 |
| Diane Peeters | diane@residence-parc.be       | diane123     | Membre CdC    | 3A  | 580 |
| Emmanuel Claes| emmanuel@residence-parc.be    | emmanuel123  | Copropriétaire (investisseur) | 5A | 1 280 |
| Philippe V.   | philippe@residence-parc.be    | philippe123  | Copropriétaire (investisseur 3 lots) | 6A-C | 1 800 |
| Marcel Dupont | marcel@residence-parc.be      | marcel123    | Copropriétaire | 4B  | 450 |
| Nadia Benali  | nadia@residence-parc.be       | nadia123     | Copropriétaire | 4A  | 320 |
| Marguerite L. | marguerite@residence-parc.be  | marguerite123| Copropriétaire | 1A  | 380 |
| Jeanne Devos  | jeanne@residence-parc.be      | jeanne123    | Copropriétaire | 1B  | 290 |
| François Leroy| francois@syndic-leroy.be      | francois123  | Syndic        | —   | — |
| Gisèle V.     | gisele@cabinet-vdb.be         | gisele123    | Comptable     | —   | — |
| Admin         | `$KOPROGO_SUPERADMIN_EMAIL`   | `$KOPROGO_SUPERADMIN_PASSWORD` | SuperAdmin | — | — |

**Total tantiemes présents** : 6 640 / 10 000 (66.4%)
**Bloc investisseurs** : Philippe (1 800) + Emmanuel (1 280) = 3 080 = 46.4%

---

## SESSION 1 — Conformité Légale AG (Votes & Convocations)

> **Objectif** : Vérifier que le droit belge de la copropriété (Art. 3.87-3.92 CC) est respecté.

---

### WORKFLOW 1 — Convocation AG (Art. 3.87 §3)

**Règle légale** : La convocation doit partir ≥ 15 jours avant la date de l'AG.

**[SYNDIC : François]**

- [ ] → `/meetings` → "Nouvelle réunion"
- [ ] Créer une AG ordinaire avec date = dans **13 jours**
  - `✓ Attendu :` Le bouton "Envoyer la convocation" est **désactivé** ou affiche une erreur "Délai légal non respecté (15 jours minimum)"
  - `✗ Bug :` ___________
- [ ] Changer la date à dans **16 jours**
  - `✓ Attendu :` Le bouton "Envoyer la convocation" est disponible
- [ ] Envoyer la convocation → `/convocations`
  - `✓ Attendu :` Statut passe à "Envoyée", liste des destinataires visible avec statut "En attente"
- [ ] Vérifier le suivi → cliquer sur la convocation → "Destinataires"
  - `✓ Attendu :` 10 copropriétaires listés avec statut e-mail (Envoyé/Ouvert/Échec)
  - `✓ Attendu :` Jeanne Devos marquée "Courrier recommandé" (pas d'e-mail)

**[COPROPRIÉTAIRE : Alice]** (switch de compte)

- [ ] → `/convocations` ou notification reçue
  - `✓ Attendu :` La convocation est visible avec l'ordre du jour
- [ ] Confirmer présence → "Je serai présent(e)"
  - `✓ Attendu :` Statut mise à jour "Sera présent(e)"

**[COPROPRIÉTAIRE : Philippe]** (investisseur absent)

- [ ] → convocation → "Désigner un mandataire"
  - `✓ Attendu :` Formulaire de procuration disponible
- [ ] Donner procuration à Alice Dubois
  - `✓ Attendu :` Procuration enregistrée

**[SYNDIC : François]**

- [ ] → suivi convocation
  - `✓ Attendu :` "Taux d'ouverture : X%" et Alice = "Sera présent(e)", Philippe = "Procuration → Alice"

---

### WORKFLOW 2 — Vote AG : 4 types de majorités (Art. 3.88 CC)

> **Contexte** : L'immeuble "Résidence du Parc Royal" tient son AG.
> François préside, Alice est dans la salle avec procuration de Philippe (1 800) et la sienne (450) = 2 250 tantièmes.
> **Total votants** = Alice(450) + Bob(430) + Charlie(660) + Diane(580) + Marcel(450) + Nadia(320) + Marguerite(380) + Jeanne(290) + Philippe-via-Alice(1 800) + Emmanuel(1 280) = **6 640**

**Rappel règles belges :**
| Majorité | Seuil | Exemple |
|----------|-------|---------|
| Absolue | > 50% des votes exprimés | Budget annuel |
| 2/3 | ≥ 66,67% des votes exprimés | Travaux façade |
| 4/5 | ≥ 80% des votes exprimés | Changement d'affectation |
| Unanimité | 100% de TOUS les tantièmes (présents + absents) | Modification parts |

**[SYNDIC : François]**

- [ ] → `/meetings/:id` → "Résolutions" → "Nouvelle résolution"

#### Test 2.1 — Majorité absolue (>50%)
... (voir document complet)

---

*Document sauvegardé le 2026-04-01*
