# Plan de Revue Humaine — KoproGo v0.1.0

**Pour** : Cowork (review manuel)
**URL** : http://localhost (dev) ou https://staging.koprogo.com (staging)
**Durée estimée** : 3-4 sessions de 2-3h (total ~8-10h)
**Date** : 2026-04-01
**WBS** : Phase 8 de `docs/WBS_RELEASE_0_1_0.md`

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
| Admin         | admin@koprogo.com             | admin123     | SuperAdmin    | —   | — |

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

- [ ] Créer résolution "Approbation budget annuel 2026" → Majorité = **Absolue**

**[ALICE]** → voter POUR (450 + 1 800 procuration Philippe = 2 250)
**[BOB]** → voter POUR (430)
**[CHARLIE]** → voter POUR (660)
**[DIANE]** → voter POUR (580)
**[EMMANUEL]** → voter CONTRE (1 280)
**[NADIA]** → voter POUR (320)
**[MARGUERITE]** → voter POUR (380)
**[JEANNE]** → voter POUR (290)
**[MARCEL]** → voter POUR (450)

**[SYNDIC : François]** → "Clôturer le scrutin"

- [ ] `✓ Attendu :` POUR = 5 330 / 6 640 = **80.3% → ADOPTÉ**
- [ ] `✓ Attendu :` Le bloc Emmanuel (1 280) seul ne peut pas bloquer la majorité absolue
- `✗ Bug :` ___________

#### Test 2.2 — Majorité 2/3 (travaux façade)

- [ ] Créer résolution "Travaux ravalement façade 2026" → Majorité = **DeuxTiers**

**[ALICE]** → POUR (450 + 1 800 = 2 250)
**[BOB]** → POUR (430)
**[CHARLIE]** → POUR (660)
**[DIANE]** → POUR (580)
**[NADIA]** → POUR (320)
**[MARGUERITE]** → POUR (380)
**[JEANNE]** → POUR (290)
**[MARCEL]** → POUR (450)
**[EMMANUEL]** → CONTRE (1 280)

**[SYNDIC]** → clôturer

- [ ] `✓ Attendu :` POUR = 5 360 / 6 640 = **80.7% → ADOPTÉ** (> 66,67%)
- `✗ Bug :` ___________

#### Test 2.3 — Blocage par le bloc investisseurs (majorité 2/3)

- [ ] Créer résolution "Modification règlement d'ordre intérieur" → Majorité = **DeuxTiers**

**[ALICE]** → POUR (450 seulement, sans procuration cette fois)
**[BOB]** → POUR (430)
**[CHARLIE]** → POUR (660)
**[DIANE]** → POUR (580)
**[MARCEL]** → POUR (450)
**[NADIA]** → POUR (320)
**[MARGUERITE]** → POUR (380)
**[JEANNE]** → POUR (290)
**[PHILIPPE]** → CONTRE (1 800)
**[EMMANUEL]** → CONTRE (1 280)

**[SYNDIC]** → clôturer

- [ ] `✓ Attendu :` POUR = 3 560 / 6 640 = **53.6% → REJETÉ** (< 66,67%)
- [ ] `✓ Attendu :` Affichage mention "Bloc investisseurs" ou simplement le résultat par tantièmes
- `✗ Bug :` ___________

#### Test 2.4 — Vote 4/5 (modification affectation)

- [ ] Créer résolution "Conversion local commun en studio" → Majorité = **QuatreCinquiemes**

**Tous POUR sauf Philippe et Emmanuel (CONTRE)**

**[SYNDIC]** → clôturer

- [ ] `✓ Attendu :` POUR = 3 560 / 6 640 = **53.6% → REJETÉ** (< 80%)
- `✗ Bug :` ___________

#### Test 2.5 — Règle des abstentions (exclues du calcul sauf unanimité)

- [ ] Créer résolution "Adoption rapport annuel" → Majorité = **Absolue**

**[BOB]** → ABSTENTION (430)
**[ALICE]** → POUR (450)
**[CHARLIE]** → POUR (660)
**[EMMANUL]** → CONTRE (1 280)

**[SYNDIC]** → clôturer

- [ ] `✓ Attendu :` Base = POUR + CONTRE uniquement = 450 + 660 + 1 280 = 2 390 (Bob exclu)
- [ ] `✓ Attendu :` POUR = 1 110 / 2 390 = **46.4% → REJETÉ** (Bob retiré de la base)
- [ ] `✗ Bug si` Bob compte dans la base : ___________

#### Test 2.6 — Capping 50% (Art. 3.87 §6)

- [ ] Créer résolution "Approbation comptes" → Majorité = **Absolue**
- [ ] Philippe vote POUR (1 800 tantièmes = 27.1% du total)
  - `✓ Attendu :` Pas de capping visible ici (Philippe < 50%)
- [ ] Tester avec un copropriétaire hypothétique à 6 000 tantièmes (hors scope immédiat si non seédé — noter)
  - `✓ Attendu :` Son vote est plafonné à 50% de la base de vote

---

### WORKFLOW 3 — Quorum & 2e Convocation (Art. 3.87 §5)

**[SYNDIC : François]**

- [ ] Créer une nouvelle AG avec SEULEMENT 3 copropriétaires présents (Alice 450, Bob 430, Charlie 660 = 1 540 / 10 000 = **15.4%**)
- [ ] Dans "Validation quorum" → entrer les tantièmes présents
  - `✓ Attendu :` Message "Quorum non atteint (15.4% < 50%). Une 2e convocation doit être envoyée."
  - `✓ Attendu :` Bouton "Planifier 2e convocation" visible
- [ ] Planifier la 2e convocation
  - `✓ Attendu :` La 2e convocation n'a pas de quorum minimum — l'AG peut délibérer avec n'importe quel nombre

---

### WORKFLOW 4 — Procurations (Art. 3.87 §7)

**Règle** : Maximum 3 procurations par mandataire. Le syndic ne peut pas être mandataire.

**[SYNDIC : François]**

- [ ] Tenter d'ajouter François comme mandataire d'Alice
  - `✓ Attendu :` Erreur "Le syndic ne peut pas recevoir de procuration"

**[ALICE]** (déjà mandataire de Philippe)

- [ ] Recevoir procuration de Bob, Charlie, Diane (total = 4 procurations)
  - `✓ Attendu :` La 4e procuration est refusée "Maximum 3 mandats atteint"

---

### WORKFLOW 5 — PV dans les 30 jours (Art. 3.87 §12)

**[SYNDIC : François]** (après une AG simulée)

- [ ] → réunion terminée → "Rédiger PV"
  - `✓ Attendu :` Compteur "J+X depuis l'AG" visible, alerte si > 25 jours
- [ ] Enregistrer et distribuer le PV
  - `✓ Attendu :` Tous les copropriétaires reçoivent une notification "PV disponible"

---

## SESSION 2 — AGE par Pétition (Art. 3.87 §2) + Tickets

### WORKFLOW 6 — AGE par Pétition (1/5 des tantièmes)

**Contexte** : Marcel veut une AGE urgente pour les travaux de toiture (€200k). Il a besoin de 20% des tantièmes = 2 000 / 10 000.

**[COPROPRIÉTAIRE : Marcel]** (450 tantièmes = 4.5%)

- [ ] → `/age-requests` → "Nouvelle demande d'AGE"
- [ ] Remplir : Titre "Travaux toiture urgents", Description, Points à l'ODJ
  - `✓ Attendu :` Statut "Brouillon" créé

- [ ] → "Ouvrir pour signatures"
  - `✓ Attendu :` Statut passe à "Ouverte", lien de signature disponible

**[ALICE]** → cosigner (450 tantièmes → total : 900 = 9%)
**[CHARLIE]** → cosigner (660 → total : 1 560 = 15.6%)
**[BOB]** → cosigner (430 → total : 1 990 = 19.9%)

- [ ] `✓ Attendu :` Barre de progression "19.9% / 20%" visible

**[DIANE]** → cosigner (580 → total : 2 570 = 25.7%)

- [ ] `✓ Attendu :` Seuil atteint — statut passe à "Seuil atteint (25.7% ≥ 20%)"
- [ ] `✓ Attendu :` Bouton "Soumettre au syndic" activé

**[MARCEL]** → soumettre au syndic

- [ ] `✓ Attendu :` Statut = "Soumise au syndic", deadline 15 jours affichée (date limite visible)

**[SYNDIC : François]**

- [ ] → `/age-requests/:id` → "Accepter"
  - `✓ Attendu :` Statut = "Acceptée", convocation AGE auto-déclenchée

**Test du rejet :**

- [ ] Répéter avec une nouvelle demande → François "Rejeter" avec motif
  - `✓ Attendu :` Marcel reçoit notification avec motif de rejet

**Test délai dépassé :**

- [ ] Soumettre sans que François réponde → simuler passage à J+16
  - `✓ Attendu :` Statut = "Expiré", convocation automatique déclenchée (le droit le prévoit)

---

### WORKFLOW 7 — Ticket Maintenance (cycle complet multi-rôles)

**Contexte** : Charlie (lot 3B) a une fuite du lot 4A (Nadia). Il crée un ticket urgent.

**[CHARLIE]**

- [ ] → `/tickets` → "Nouveau ticket"
- [ ] Remplir : "Fuite d'eau depuis lot 4A", catégorie Plomberie, priorité **Haute**
  - `✓ Attendu :` Due date = maintenant + 24h (priorité Haute)
  - `✓ Attendu :` Statut = "Ouvert"

**[SYNDIC : François]**

- [ ] → ticket → "Assigner entrepreneur"
- [ ] Sélectionner Hassan El Amrani, générer lien magique (72h)
  - `✓ Attendu :` Lien JWT généré, visible, copiable
  - `✓ Attendu :` Statut = "Assigné"

**[HASSAN — lien magique, pas de compte]**

- [ ] Ouvrir le lien magique dans un onglet privé
  - `✓ Attendu :` Accès au rapport de travaux SANS login
  - `✓ Attendu :` Formulaire : compte-rendu, photos avant/après, pièces remplacées
- [ ] Remplir le rapport, uploader 2 photos, soumettre
  - `✓ Attendu :` Statut rapport = "Soumis"

**Test expiration :**

- [ ] Attendre 73h (ou manipuler l'horloge) → réouvrir le lien
  - `✓ Attendu :` Erreur "Lien expiré (72h dépassées)"

**[ALICE + DIANE]** (membres CdC)

- [ ] → rapport → "Valider les travaux"
  - `✓ Attendu :` Les deux doivent valider (ou seulement 1 selon config CdC)
  - `✓ Attendu :` Validation déclenche le paiement automatique vers Hassan

**[CHARLIE]**

- [ ] → ticket → "Confirmer résolution" → Fermer
  - `✓ Attendu :` Statut = "Fermé", historique complet visible

**Test réouverture :**

- [ ] Réouvrir le ticket "Fuite réapparue"
  - `✓ Attendu :` Statut = "Ouvert" à nouveau, historique préservé

---

## SESSION 3 — Finances & Impact Copropriétaires Fragiles

### WORKFLOW 8 — Appel de Fonds + Impact Financier

**Contexte** : François lance un appel de fonds de €10 000 pour le ravalement.

**[SYNDIC : François]**

- [ ] → `/call-for-funds` → "Nouvel appel de fonds"
- [ ] Montant : €10 000, type : Charges ordinaires, date d'échéance : dans 30 jours
- [ ] → "Envoyer" → confirmation que les contributions individuelles sont générées
  - `✓ Attendu :` Chaque copropriétaire reçoit sa part proportionnelle aux tantièmes

**[CHARLIE]** (660/10 000 = 6.6% = €660)

- [ ] → notification → voir sa contribution
  - `✓ Attendu :` Montant affiché €660
  - `✓ Attendu :` Simulation "Impact budget" : €660 = X% de votre mensualité (si données disponibles)

**[NADIA]** (320/10 000 = 3.2% = €320)

- [ ] → voir sa contribution : €320
  - `✓ Attendu :` Option paiement en 3 fois visible (si implémenté) ou note "Contacter le syndic"

**[JEANNE]** (290/10 000 = 2.9% = €290)

- [ ] → voir sa contribution : €290
  - `✓ Attendu :` Montant clairement lisible (police suffisamment grande ?)
  - Noter tout problème d'affichage : ___________

**[MARGUERITE]** (380/10 000 = 3.8% = €380)

- [ ] → voir sa contribution : €380
- [ ] Marquer comme "Payé"
  - `✓ Attendu :` Statut = "Payée", date de paiement enregistrée

**[PHILIPPE]** (1 800/10 000 = 18% = €1 800) — investisseur passif

- [ ] Ne rien faire (simuler l'impayé)
  - `✓ Attendu :` Contribution apparaît dans "Impayés" pour François après la date d'échéance

---

### WORKFLOW 9 — Recouvrement Automatisé (4 niveaux)

**[SYNDIC : François]** (après que Philippe soit en retard)

- [ ] → `/payment-reminders` → contribution de Philippe → "Créer relance"
  - `✓ Attendu :` Niveau 1 "Douce" (J+15) créée automatiquement

- [ ] → escalader vers Niveau 2 "Formelle" (J+30)
  - `✓ Attendu :` Modèle de lettre formelle généré, pénalités légales calculées (8%/an prorata)

- [ ] → escalader vers Niveau 3 "Mise en demeure" (J+45)
  - `✓ Attendu :` Lettre recommandée générée, numéro de suivi disponible

- [ ] → escalader vers Niveau 4 "Action légale" (J+60)
  - `✓ Attendu :` Dossier complet pour huissier, historique complet joint

**Test protection copropriétaires fragiles :**

- [ ] Créer une relance pour Nadia → vérifier qu'un commentaire "Fragilité financière" peut être ajouté
  - `✓ Attendu :` Champ notes libre pour syndic

---

### WORKFLOW 10 — Comptabilité (Gisèle — Accès lecture seule)

**[COMPTABLE : Gisèle]**

- [ ] → `/reports/balance-sheet`
  - `✓ Attendu :` Bilan PCMN belge visible, bien formaté
- [ ] Tenter de créer une dépense
  - `✓ Attendu :` Bouton "Créer" absent ou erreur 403
- [ ] → `/journal-entries` → créer une écriture manuelle
  - `✓ Attendu :` Accès autorisé (la comptable peut saisir des écritures manuelles)
- [ ] Créer une écriture déséquilibrée (débit ≠ crédit)
  - `✓ Attendu :` Erreur "L'écriture doit être équilibrée (débit = crédit)"

---

## SESSION 4 — Communauté, GDPR & UX Accessibilité

### WORKFLOW 11 — SEL (Système d'Échange Local)

**Contexte** : L'immeuble a une monnaie-temps. 1h de service = 1 crédit.

**[ALICE]** → créer une offre

- [ ] → `/exchanges` → "Nouvelle offre"
- [ ] "Cours de cuisine belge", type Service, 2 crédits, durée 2h
  - `✓ Attendu :` Statut "Offerte", visible dans la marketplace

**[NADIA]** → demander l'échange

- [ ] → marketplace → voir l'offre d'Alice → "Demander"
  - `✓ Attendu :` Statut = "Demandée"

**[ALICE]** → démarrer

- [ ] → "Démarrer l'échange"
  - `✓ Attendu :` Statut = "En cours"
- [ ] → "Compléter"
  - `✓ Attendu :` Alice +2 crédits, Nadia -2 crédits (mise à jour automatique)

**[NADIA]** → noter Alice

- [ ] → 5 étoiles
  - `✓ Attendu :` Note enregistrée, moyenne d'Alice mise à jour

**[MARGUERITE]** — test dignity

- [ ] → créer offre "Repassage soigné", 1 crédit/heure
- [ ] **[BOB]** → demander → compléter
  - `✓ Attendu :` Marguerite +1 crédit, Bob -1 crédit
  - `✓ Attendu :` Marguerite apparaît dans le classement de l'immeuble

**Vérifier le classement :**

- [ ] → `/buildings/:id/leaderboard`
  - `✓ Attendu :` Classement affiché par solde de crédits
  - `✓ Attendu :` Marguerite visible (sentiment d'utilité)

---

### WORKFLOW 12 — Sondage Consultatif (entre 2 AG)

**[SYNDIC : François]**

- [ ] → `/polls` → "Nouveau sondage"
- [ ] "Faut-il repeindre le hall en bleu ?", type YesNon, date fin = dans 7 jours
  - `✓ Attendu :` Statut = "Brouillon"
- [ ] Publier → Statut = "Actif"

**[ALICE, CHARLIE, NADIA, MARGUERITE, MARCEL]** → voter "Oui"

**[PHILIPPE, EMMANUEL]** → ne votent pas (simuler l'absentéisme investisseurs)

**[SYNDIC]** → clôturer

- [ ] → résultats
  - `✓ Attendu :` "100% des répondants — Oui" ET "Taux de participation : 50%"
  - `✓ Attendu :` L'absentéisme est documenté (non ignoré)
  - `✗ Bug si` le taux de participation n'est pas affiché : ___________

---

### WORKFLOW 13 — GDPR (Droits des personnes)

**[CHARLIE]**

- [ ] → `/settings/gdpr` → "Exporter mes données"
  - `✓ Attendu :` Fichier JSON téléchargeable contenant toutes ses données (profil, consentements, interactions)
  - `✓ Attendu :` Délai < 5 secondes pour la génération

- [ ] → "Anonymiser mes données" (droit à l'oubli)
  - `✓ Attendu :` Confirmation demandée avec avertissement "Irréversible"
  - Stopper à la confirmation — NE PAS valider en review

- [ ] → "Rectifier mes données" — changer son e-mail
  - `✓ Attendu :` Champ modifiable, sauvegarde OK

- [ ] → "Restreindre le traitement"
  - `✓ Attendu :` Confirmation, statut "Traitement restreint" visible

**[ADMIN]** → vérifier le registre Art. 30

- [ ] → `/admin/gdpr-art30`
  - `✓ Attendu :` Liste des traitements visible (Stripe, S3, SMTP avec certifications)

---

### WORKFLOW 14 — Tableau de Bord par Rôle

**Vérifier que chaque rôle voit le bon dashboard :**

**[SYNDIC : François]**

- [ ] → `/dashboard`
  - `✓ Attendu :` Vue syndic : tickets ouverts, appels de fonds en attente, prochaine AG, relances actives
  - `✓ Attendu :` Alertes légales : délais convocation, PV à envoyer, AGE en attente

**[ALICE]** (Présidente CdC)

- [ ] → dashboard
  - `✓ Attendu :` Vue CdC : rapports de travaux à valider, décisions à suivre, tableau de bord conseil

**[GISÈLE]** (Comptable)

- [ ] → dashboard
  - `✓ Attendu :` Vue comptable : statistiques financières, transactions récentes, états à finaliser

**[CHARLIE]** (Copropriétaire lambda)

- [ ] → dashboard
  - `✓ Attendu :` Vue propriétaire : ses lots, ses paiements, ses tickets, ses échanges SEL

**[ADMIN]**

- [ ] → dashboard
  - `✓ Attendu :` Vue superadmin : liste des organisations, users, métriques plateforme

---

## VÉRIFICATIONS UX TRANSVERSALES

### Navigation & Accessibilité

- [ ] Tous les liens de la navigation principale fonctionnent (pas de 404)
- [ ] Retour arrière navigateur → aucun formulaire perdu sans confirmation
- [ ] F5 sur une page → reste connecté (session persistante)

**Mobile (375px) :**

- [ ] Dashboard → lisible sur mobile
- [ ] Formulaire de vote → utilisable au doigt
- [ ] Tableau des tantièmes → scroll horizontal ou adapté

**Accessibilité Marguerite (78 ans, peu numérique) :**

- [ ] La police du dashboard est suffisamment grande (≥ 14px minimum)
- [ ] Les boutons d'action principaux sont suffisamment grands (≥ 44px)
- [ ] Les montants d'argent sont en gras et bien lisibles

### Internationalisation (i18n)

- [ ] Passer en NL → `/settings` → changer la langue en Néerlandais
  - `✓ Attendu :` Les menus principaux sont en néerlandais (pas de chaîne anglaise visible)
- [ ] Repasser en FR
  - `✓ Attendu :` Retour au français sans rechargement forcé

### Messages d'erreur

- [ ] Soumettre un formulaire vide → messages d'erreur clairs et en français
- [ ] Session expirée → redirection vers login avec message "Session expirée"
- [ ] Action non autorisée (ex : Charlie tente `/admin`) → 403 clair, pas de page blanche

### Performance subjective

- [ ] Les listes (tickets, copropriétaires) s'affichent en < 2 secondes
- [ ] Pas de spinner infini visible nulle part
- [ ] Les formulaires donnent un retour immédiat à la soumission (loading button)

---

## MATRICE DE CONFORMITÉ LÉGALE BELGE

Cocher ✅ Conforme | ⚠️ Partiel | ❌ Non conforme

| Article CC | Exigence | Vérifié par | Résultat |
|------------|----------|-------------|----------|
| Art. 3.87 §3 | Convocation ≥ 15 jours avant AG | WF-1 | `[ ]` |
| Art. 3.87 §5 | Quorum 50% à la 1re convocation | WF-3 | `[ ]` |
| Art. 3.87 §5 | 2e convocation sans quorum minimum | WF-3 | `[ ]` |
| Art. 3.87 §6 | Capping vote 50% tantièmes | WF-2.6 | `[ ]` |
| Art. 3.87 §7 | Max 3 procurations/mandataire | WF-4 | `[ ]` |
| Art. 3.87 §7 | Syndic ne peut être mandataire | WF-4 | `[ ]` |
| Art. 3.87 §12 | PV distribué dans les 30 jours | WF-5 | `[ ]` |
| Art. 3.88 abs. | Majorité absolue > 50% votes exprimés | WF-2.1 | `[ ]` |
| Art. 3.88 abs. | Abstentions exclues de la base | WF-2.5 | `[ ]` |
| Art. 3.88 2/3 | Seuil ≥ 66,67% votes exprimés | WF-2.2 | `[ ]` |
| Art. 3.88 4/5 | Seuil ≥ 80% votes exprimés | WF-2.4 | `[ ]` |
| Art. 3.88 unanim. | 100% de TOUS les tantièmes | WF-2 note | `[ ]` |
| Art. 3.87 §2 | AGE pétition seuil 1/5 tantièmes | WF-6 | `[ ]` |
| Art. 3.87 §2 | Syndic doit répondre dans 15 jours | WF-6 | `[ ]` |
| RGPD Art. 15 | Export données sur demande | WF-13 | `[ ]` |
| RGPD Art. 17 | Anonymisation disponible | WF-13 | `[ ]` |
| RGPD Art. 16 | Rectification données | WF-13 | `[ ]` |
| RGPD Art. 18 | Restriction de traitement | WF-13 | `[ ]` |
| RGPD Art. 30 | Registre des traitements | WF-13 | `[ ]` |

**Score cible avant GO release** : 18/19 ≥ 95%

---

## BUGS À NOTER

| # | Workflow | Description | Gravité (Bloquant/Majeur/Mineur) |
|---|----------|-------------|----------------------------------|
| 1 | | | |
| 2 | | | |
| 3 | | | |
| 4 | | | |
| 5 | | | |

---

## DÉCISION GO / NO-GO

- [ ] Tous les workflows Tier 1 (conformité légale) validés sans bug Bloquant
- [ ] Score conformité légale ≥ 95% (18/19)
- [ ] Aucun bug majeur non résolu dans les workflows financiers
- [ ] UX accessible pour Marguerite et Jeanne (seniors)
- [ ] i18n FR minimum opérationnel

**Décision** : `[ ] GO` `[ ] NO-GO — raison : ________________________`

**Signé** : _________________________ **Date** : _____________

---

*Document généré pour la release KoproGo v0.1.0 — Phase 8 WBS*
*Référence : `docs/WBS_RELEASE_0_1_0.md` Phase 8*
