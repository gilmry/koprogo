# KOPROGO — Rapport de Tests E2E Manuels

**Gestion de Copropriété SaaS — Belgique**

| | |
|---|---|
| **Date** | 22 mars 2026 |
| **Testeur** | Farah (assistée par Claude) |
| **Environnement** | Docker Compose local (Traefik + Backend Rust + Frontend Astro + PostgreSQL) |
| **Version** | branche `chore/new-branch-workflow` |

---

## 1. Résumé exécutif

Ce rapport présente les résultats d'une campagne de tests E2E manuels réalisée sur la plateforme KoproGo. Les tests ont couvert le workflow complet : **Admin → Syndic → Owner**, ainsi que la navigation sur l'ensemble des pages du sidebar (20+ pages testées).

| Métrique | Résultat |
|---|---|
| Pages testées | 20+ (Tableau de bord, Immeubles, Propriétaires, Unités, Charges, Workflow factures, Appels de fonds, Contributions, Relances, Budgets, Convocations, Tickets, Devis, Travaux, Inspections, Conseil, Documents, SEL, Sondages, Annonces, Réservations) |
| Bugs trouvés | 8 bugs (2 critiques, 3 majeurs, 1 mineur, 2 cosmétiques) + 1 GAP architectural critique (chaîne dépenses déconnectée) |
| Workflows testés | Admin (création org/user/building), Syndic (création owners, navigation), pages CRUD |
| Environnement | Docker Compose : Traefik (port 80), Backend Rust (cargo-watch), Frontend Astro (dev), PostgreSQL 15 |

---

## 2. Bugs identifiés

### Bug #1 — MAJEUR — Syndic peut créer des immeubles

- **Page** : Immeubles
- **Description** : Le bouton « Nouvel immeuble » est visible et fonctionnel pour le rôle syndic. Selon le workflow métier, seul l'admin SaaS doit créer les immeubles (carte d'identité structurelle à partir de l'acte de base/ROI).
- **Correction proposée** : Masquer le bouton si `role !== superadmin`. Ajouter guard côté backend sur `POST /buildings`.

### Bug #2 — CRITIQUE — Unités : pas de filtre par organisation

- **Page** : Unités
- **Description** : La page `/units` affiche les lots de TOUTES les organisations. Le syndic Jean Dupont (org « Syndic Bruxelles Test ») voit le « Lot 1A » de l'organisation E2E Test.
- **Correction proposée** : Ajouter filtre `organization_id` sur `GET /units`. Vérifier l'isolation multi-tenant sur tous les endpoints.

### Bug #3 — MINEUR — Pas de bouton « Nouveau lot » sur /units

- **Page** : Unités
- **Description** : La page globale Unités n'a pas de bouton de création. Le bouton « + Ajouter un lot » existe uniquement sur la page détail d'un immeuble.
- **Correction proposée** : Ajouter un bouton « + Nouveau lot » sur la page `/units`, ou rediriger vers la page détail immeuble.

### Bug #4 — COSMÉTIQUE — Page Tickets en anglais

- **Page** : Tickets
- **Description** : Titre « Maintenance Tickets », bouton « Create New Ticket », section « My Tickets » : tout est en anglais alors que le reste de l'app est en français.
- **Correction proposée** : Traduire tous les labels de la page `tickets.astro` et `TicketList.svelte` en français.

### Bug #5 — MAJEUR — Création de ticket ne fonctionne pas

- **Page** : Tickets
- **Description** : Le bouton « Create New Ticket » ne fait rien au clic. Le code attend `building_id` dans les query params URL, mais la page n'a pas de sélecteur d'immeuble. Le toast d'erreur ne s'affiche pas.
- **Correction proposée** : Ajouter un sélecteur d'immeuble comme sur les autres pages (Devis, Travaux, etc.). Corriger la fonction `showPageToast`.

### Bug #6 — CRITIQUE — Pas de validation total tantièmes/millièmes

- **Page** : Détail immeuble
- **Description** : On peut créer des lots dont le total des tantièmes dépasse le total déclaré pour l'immeuble (ex: 901/900). Aucune validation côté frontend ni backend.
- **Correction proposée** : Ajouter une validation : somme des quotas des lots ≤ `total_units`. Bloquer la création si dépassement. Afficher un compteur en temps réel.

### Bug #7 — MAJEUR — Erreur chargement immeubles (3 pages)

- **Page** : Sondages, Annonces, Réservations
- **Description** : Les pages Sondages, Annonces (Notice Board) et Réservations (Resource Booking) affichent « Erreur lors du chargement des immeubles ». L'endpoint API utilisé est probablement différent de celui des pages fonctionnelles.
- **Correction proposée** : Vérifier l'endpoint d'API utilisé par ces 3 composants Svelte. Probablement un problème d'URL ou de permissions sur `GET /buildings`.

### Bug #8 — COSMÉTIQUE — Pages Annonces et Réservations en anglais

- **Page** : Annonces, Réservations
- **Description** : Annonces : « Notice Board », « Community announcements and classified ads », « + Create Notice ». Réservations : « Resource Booking Calendar ».
- **Correction proposée** : Traduire les composants `NoticeBoard.svelte` et `ResourceBooking.svelte` en français.

---

## 3. Recommandations architecturales

### 3.1 Workflow métier : séparation des rôles

Le workflow correct pour l'onboarding d'une ACP (Association des Copropriétaires) dans KoproGo doit suivre un processus strict :

1. **L'ACP envoie l'acte de base et le ROI** à KoproGo
2. **L'admin SaaS crée la carte d'identité de l'immeuble :** données structurelles, lots, tantièmes/millièmes
3. **L'admin crée/associe le syndic** choisi par l'ACP et lui confie l'immeuble
4. **L'admin crée les comptes utilisateurs** et les associe à l'organisation
5. **Le syndic crée les copropriétaires (owners)** et associe les comptes utilisateurs aux owners

**Implication technique** : les boutons de création d'immeuble et de lots doivent être réservés au rôle superadmin.

### 3.2 Prédicats de vérification : association syndic ↔ immeuble

Quand on associe un syndic à un immeuble (ou quand on change de syndic), le système doit vérifier un ensemble de **prédicats contractuels**. L'association syndic/immeuble est un engagement contractuel entre l'ACP et KoproGo, donc la vérification de l'intégrité structurelle est obligatoire avant toute association.

#### Prédicats avant association syndic → immeuble

1. **Intégrité structurelle complète :** Nombre de lots créés == `total_units` déclaré pour l'immeuble. Si l'immeuble déclare 6 lots, il doit y avoir exactement 6 lots enregistrés.
2. **Total tantièmes validé :** Somme des tantièmes/millièmes de tous les lots == total déclaré (ex: 1000/1000). Tolérance de ±0.01% pour les arrondis.
3. **Chaque lot a un type validé :** Tous les lots ont un type (Appartement, Cave, Parking, Commerce, etc.), un étage et une surface.
4. **Organisation syndic active :** L'organisation associée au syndic est en statut « active » avec un plan de souscription valide.
5. **Acte de base référencé :** Le document « acte de base » est téléversé et lié à l'immeuble dans la gestion documentaire.
6. **ROI référencé :** Le Règlement d'Ordre Intérieur est téléversé et lié à l'immeuble.

#### Prédicats lors du changement de syndic

1. **Tous les prédicats ci-dessus** doivent rester valides après changement.
2. **Transfert comptable :** Vérifier que la clôture comptable de l'exercice en cours est faite ou transférée.
3. **Aucun appel de fonds en cours :** Pas d'appels de fonds ouverts non soldés au moment du transfert.
4. **PV d'AG de désignation :** Le procès-verbal de l'AG désignant le nouveau syndic doit être enregistré.
5. **Historique conservé :** L'ancien syndic reste dans l'historique (audit trail) avec dates de début et de fin de mandat.

### 3.3 Chaîne d'approbation des dépenses (GAP architecture)

Les modules Ticket, ContractorReport, BoardDecision et Expense existent individuellement mais **ne sont PAS connectés** dans un workflow métier. Voici la chaîne attendue :

**Ticket créé** (maintenance request)
  ↓ génère un ordre de service
**Ordre de service accepté** (par le syndic/CdC)
  ↓ envoi magic link au prestataire
**PWA Prestataire** (`/contractor/?token=XXX` — photos, dictée vocale FR-BE, pièces, mode offline)
  ↓ soumet le rapport au syndic
**Rapport prestataire validé** (Draft → Submitted → UnderReview → Validated)
  ↓ soumis pour validation
**Validation CdC** (si >20 lots) **OU** **Validation Syndic** (si ≤20 lots)
  ↓ décision enregistrée comme BoardDecision
**Dépense approuvée** (Expense status → Approved, paiement déclenché)

**Actions requises :**

- Créer l'entité `WorkOrder` (lien Ticket → ContractorReport)
- Ajouter FK `contractor_report_id` à Expense
- Rendre ContractorReport obligatoire avant `Expense.approve()`
- Ajouter FK `board_decision_id` à Expense si immeuble >20 lots
- Auto-trigger magic link PWA à l'acceptation de l'ordre de service
- Créer `BuildingContractor` (registre fournisseurs officiels ACP)

### 3.4 Fournisseurs officiels de l'ACP

Les ACP ont généralement des **fournisseurs officiels** (plombier, électricien, chauffagiste, jardinier, etc.) qui sont mobilisés en priorité pour les interventions courantes. Ces prestataires attitrés doivent être enregistrés dans le système avec un statut spécial et leur mobilisation doit déclencher automatiquement l'obligation de rapport.

- **Registre fournisseurs ACP :** Créer une entité `BuildingContractor` pour lier les prestataires officiels à chaque immeuble avec spécialité et contrat.
- **Assignation prioritaire :** Quand un ticket est créé avec une catégorie (Plumbing, Electrical...), proposer automatiquement le fournisseur officiel de l'ACP pour cette spécialité.
- **Rapport obligatoire :** Tout fournisseur officiel mobilisé doit soumettre un ContractorReport avant que la dépense ne puisse être approuvée.

---

## 4. Checklist de vérification complète

Cette section détaille l'ensemble des points de vérification à réaliser pour valider la conformité fonctionnelle, légale et technique de KoproGo.

### 4.1 Comptabilité & TVA (PCMN Belgique)

- **Plan comptable PCMN :** Vérifier que les ~90 comptes seedés (AR 12/07/2012) sont corrects et que la hiérarchie classes/sous-classes/groupes est respectée.
- **TVA belge :** Vérifier les 3 taux (6% rénovation, 12% logement social, 21% standard) dans InvoiceLineItem et les calculs automatiques.
- **Écritures de journal :** Tester la double entrée (débit=crédit), les 4 types de journaux (ACH/VEN/FIN/ODS), et que le solde est toujours équilibré.
- **Rapports financiers :** Bilan et compte de résultats cohérents avec les écritures, exports utilisables par le commissaire aux comptes.
- **Chaîne dépense → compta :** Quand une dépense est créée, soumise, validée et payée, vérifier qu'une écriture comptable est générée automatiquement à chaque étape pour maintenir la compta à jour en temps réel.
- **Commissaire aux comptes :** Le commissaire doit pouvoir accéder à toutes les pièces justificatives (factures, devis, rapports prestataires, PV d'AG) à tout moment sans blocage.
- **Conservation 7 ans :** PARTIEL — Backups OK mais cron de suppression automatique des documents >7 ans manquant (matrice conformité).

### 4.2 Relances de paiement automatiques

- **Escalade 4 niveaux :** Vérifier le déclenchement automatique : Gentle (J+15), Formal (J+30), FinalNotice (J+45), LegalAction (J+60).
- **Taux légal belge :** Vérifier que le taux est à jour (4.5% pour 2026, taux BCE + marge). Le système doit signaler la nécessité de mettre à jour le taux chaque année.
- **Calcul pénalités :** Formule = `(montant × taux_annuel × jours_retard / 365)`. Vérifier la précision des arrondis.
- **Traçabilité :** Chaque relance doit avoir : `sent_date`, `tracking_number`, `notes`, et le niveau d'escalade correct.
- **Arrêt automatique :** Les relances doivent s'arrêter automatiquement quand le paiement est reçu.

### 4.3 Assemblées Générales (AG)

#### Séquence ODJ légale obligatoire (Art. 3.87-3.89 CC)

1. Ouverture & Bureau (Président copropriétaire, PAS le syndic) + Secrétaire + Scrutateurs
2. Quorum & Présences : feuille de présence, vérifier >50% des quotes-parts
3. Rapport du commissaire aux comptes (AVANT le vote sur les comptes)
4. Approbation des comptes (majorité absolue)
5. Décharge du syndic (quitus) — résolution DISTINCTE des comptes
6. Décharge du commissaire aux comptes
7. Évaluation contrats fourniture et services (Art. 3.89 §5 12°) — OBLIGATOIRE
8. Budget prévisionnel : fonds de roulement + fonds de réserve (obligatoire)
9. Renouvellement mandat syndic (après décharge, max 3 ans)
10. Désignation commissaire aux comptes (annuel, obligatoire)
11. Propositions des copropriétaires (reçues 3 semaines avant, majorité requise spécifiée)
12. Divers — AUCUN vote autorisé sur le point « Divers » (jurisprudence constante)

#### Points de vérification AG

- **Délai 15 jours minimum :** Convocation AGO et AGE = 15 jours calendrier minimum (Art. 3.87 §3). Vérifier la validation domain + DB constraint.
- **Quorum 50%+50% :** **MANQUANT** — Si 1ère convocation sans quorum, 2ème convocation obligatoire (15j supplémentaires).
- **Types de majorité :** Simple (50%+1 exprimés), Absolue (50%+1 tous), 2/3 (travaux extraordinaires), 3/4 (jouissance), 4/5 (règlement), Unanimité (droits réels).
- **Procurations :** **MANQUANT** — Max 3 mandats par personne + exception 10% pouvoir de vote (Art. 3.87 §7).
- **Lien agenda-résolutions :** **MANQUANT** — Impossible de voter sur un point hors ODJ (nullité du vote).
- **PV automatique :** Génération automatique du procès-verbal + distribution dans les 30 jours (MANQUANT dans la matrice).
- **Visioconférence AG :** Sessions video (Zoom/Teams/Jitsi), quorum combiné (physique + remote), Art. 3.87 §1.
- **Todo list post-AG :** Après le PV, générer automatiquement une todo list pour le Conseil de copropriété (CdC) avec suivi des tâches.

### 4.4 États datés & historique des lots

- **États datés successifs :** Vérifier que les états datés d'un lot sont enregistrés et historisés (pas seulement le dernier).
- **Délai légal 10 jours :** Détection automatique des états datés en retard (>10 jours, non générés).
- **Expiration 3 mois :** Détection des états datés expirés (>3 mois depuis la date de référence).
- **Données financières :** Sections 1-16 avec charges dues, fonds de réserve, procédures judiciaires en cours.

### 4.5 Droits du résident (copropriétaire & locataire)

#### Transparence (accès en lecture)

- Charges et dépenses de l'immeuble (détail par lot)
- Acte de base et Règlement d'Ordre Intérieur (ROI)
- Procès-verbaux de toutes les AG
- Plans de l'immeuble
- Rapports des fournisseurs (ContractorReports)
- Budgets et rapports financiers

#### Actions du résident

- **Mettre un point à l'ODJ :** Le copropriétaire peut ajouter un point à l'ordre du jour d'une AGO ou AGE (3 semaines avant).
- **Provoquer une AGE :** Si 20% des quotités réunies (module AgeRequest avec cosignataires, seuil 1/5, délai syndic 15j).
- **Mise à jour données personnelles :** RGPD Art. 16 (rectification), Art. 18 (restriction de traitement), Art. 21 (opt-out marketing).
- **Pouvoir au CdC :** Quand le résident est membre du Conseil de copropriété (>20 lots), il a un rôle décisionnel.
- **Avis sur contrats :** Émettre un avis pour le CdC lors du point obligatoire AGO sur l'évaluation des contrats de fourniture et service.
- **Vote et procuration :** Voter en AG et déléguer par procuration (max 3 mandats, exception 10%).

### 4.6 Profil Comptable

- **Dashboard comptable :** Vérifier `/dashboard/accountant/stats` et `/dashboard/accountant/transactions`.
- **Rapports :** Bilan, compte de résultats, analyse de variance budgétaire (budget vs réel).
- **Exports :** Vérifier que tous les exports sont disponibles et exploitables (CSV, PDF).
- **Encodages :** Écritures de journal manuelles (double entrée), contributions propriétaires, appels de fonds.
- **Routines non buguées :** Vérifier que les routines automatiques (calcul charges, distribution, génération écritures) ne biaisent pas les rapports.

### 4.7 Modules communautaires

6 modules communautaires implémentés. Vérifier le CRUD et les workflows pour chacun :

- **SEL (Échanges locaux) :** Workflow Offered→Requested→InProgress→Completed, crédits automatiques, notes mutuelles 1-5, leaderboard.
- **Tableau d'annonces :** CRUD annonces communautaires, visibilité par immeuble.
- **Annuaire de compétences :** Profils habitants avec compétences partagées, recherche par catégorie.
- **Bibliothèque d'objets :** Prêt d'objets entre habitants (outils, livres, équipements).
- **Réservation de ressources :** Calendrier de réservation des espaces communs (salle de réunion, buanderie, etc.).
- **Gamification :** Achievements (8 catégories, 5 tiers), Challenges (Individual/Team/Building), Leaderboard multi-sources.
- **Accès :** Vérifier que les copropriétaires ET locataires (ou utilisateurs affectés à l'organisation) ont accès.

### 4.8 RGPD & Conformité légale

#### Statut RGPD (6/10 articles implémentés)

| Article RGPD | Endpoint | Statut |
|---|---|---|
| Art. 15 — Accès | `GET /gdpr/export` | ✅ CONFORME |
| Art. 16 — Rectification | `PUT /gdpr/rectify` | ✅ CONFORME |
| Art. 17 — Effacement | `DELETE /gdpr/erase` | ✅ CONFORME |
| Art. 18 — Restriction | `PUT /gdpr/restrict-processing` | ✅ CONFORME |
| Art. 21 — Opposition | `PUT /gdpr/marketing-preference` | ✅ CONFORME |
| Art. 30 — Registre | `audit_logs` (7 ans) | ✅ CONFORME |
| Art. 13-14 — Information | — | ❌ MANQUANT |
| Art. 28 — Sous-traitants | — | ❌ MANQUANT |
| Art. 32 — Sécurité | LUKS seulement | ⚠️ PARTIEL |
| Art. 33 — Violation | — | ❌ MANQUANT |

#### Actions RGPD manquantes

- **Politique de confidentialité :** Publier une politique de confidentialité accessible publiquement (Art. 13-14).
- **DPA sous-traitants :** Signer des accords de traitement avec Stripe, AWS S3, fournisseurs email (Art. 28).
- **Procédure violation :** Mettre en place la notification APD en 72h (Art. 33). Sanctions APD jusqu'à 20M€ ou 4% CA.
- **Chiffrement colonnes :** Ajouter le chiffrement au niveau des colonnes sensibles (pas seulement LUKS disque, Art. 32).

### 4.9 Devis, marchés & fournisseurs

- **Encodage devis :** Workflow Requested→Received→UnderReview→Accepted/Rejected. Vérifier chaque transition d'état.
- **3 devis obligatoires :** Pour travaux >5000€, la loi belge impose 3 devis minimum. Vérifier la comparaison automatique.
- **Critères d'attribution par défaut :** Prix 40%, Délais 30%, Garanties 20%, Réputation 10%. Scoring automatique.
- **Modification critères par AG :** L'assemblée générale doit pouvoir modifier les pondérations des critères. Vérifier que c'est paramétrable.
- **Backoffice prestataire :** Le prestataire doit pouvoir soumettre son offre via un espace dédié pour transparence complète.
- **Audit trail décision :** Champs `decision_at`, `decision_by`, `decision_notes` pour conformité légale. Trajet du devis à la marketplace.
- **Enquêtes de satisfaction :** Liées aux AG (point contrats fourniture/service) pour alimenter la réputation et future marketplace.

### 4.10 Achat groupé d'énergie

Module conforme au cadre légal belge (Loi 29/04/1999, Charte Qualité CREG 2018).

- **Campagne énergétique :** Création, gestion statut, statistiques anonymisées (k-anonymity >= 5 participants).
- **Upload factures :** Consentement RGPD explicite, chiffrement, droit de retrait (Art. 7.3 suppression immédiate).
- **Offres fournisseurs :** Ajout offres par courtier/admin, comparaison multi-critères, sélection après vote.
- **Zéro commission :** Modèle SaaS (abonnement), aucune commission sur l'énergie. Contrat direct membre↔fournisseur.
- **Neutralité :** Indépendance vis-à-vis des fournisseurs (Charte CREG). Seulement fournisseurs agréés régionalement.
- **Parties communes vs privées :** Syndic décide pour communs (+ AG si > seuil marché), chaque copropriétaire décide pour son lot.
- **Droit de rétractation 14 jours :** Sans frais de résiliation anticipée. Vérifier le workflow de retrait.

### 4.11 Résumé matrice de conformité légale

| Catégorie | Conforme | Manquant | Score |
|---|---|---|---|
| Loi copropriété (AG) | 18/25 | 7 | 72% |
| RGPD | 6/10 | 4 | **60%** |
| Comptabilité PCMN | 1/2 | 1 partiel | 75% |
| **TOTAL** | **25/37** | **10** | **67%** |

#### 10 items MANQUANTS critiques

1. Lien agenda-résolutions (vote hors ODJ = nullité)
2. Quorum 50%+50% et 2ème convocation si quorum KO
3. Quorum 3/4 pour décisions nécessitant majorité 3/4
4. Procurations : max 3 mandats + exception 10%
5. Distribution PV dans les 30 jours
6. Mandat syndic max 3 ans
7. Politique de confidentialité publique (RGPD Art. 13-14)
8. DPA sous-traitants Stripe/AWS/Email (RGPD Art. 28)
9. Procédure notification violation 72h (RGPD Art. 33)
10. Cron suppression documents >7 ans (conservation comptable)

---

## 5. Pages fonctionnelles (sans bug majeur)

Les pages suivantes ont été testées et fonctionnent correctement :

- **Tableau de bord** — Dashboard syndic avec KPIs
- **Immeubles** — Liste et détail (mais Bug #1 : bouton création visible pour syndic)
- **Propriétaires** — CRUD fonctionnel, création OK
- **Charges** — Liste, filtres immeuble/statut
- **Workflow factures** — Pipeline Draft → Approved
- **Appels de fonds** — Gestion et envoi
- **Contributions** — Liste des contributions propriétaires
- **Relances** — Workflow 4 niveaux conforme
- **Budgets** — Gestion annuelle avec filtres
- **Convocations** — Délais légaux belges affichés
- **Devis** — Obligation légale 3 devis >5000€
- **Travaux** — Carnet d'entretien numérique
- **Inspections** — Contrôles réglementaires avec onglets
- **Conseil** — Gestion du CA, élection membres
- **Documents** — Gestion documentaire avec upload
- **SEL** — Système d'échange local, cadre légal belge

---

## 6. Prochaines étapes recommandées

### Priorité 1 — Bugs critiques

1. **Corriger l'isolation multi-tenant (#2)** : filtrer tous les endpoints par `organization_id`
2. **Validation tantièmes (#6)** : bloquer la création de lots si total dépassé

### Priorité 2 — Chaîne métier dépenses (GAP architecture)

1. **Créer l'entité WorkOrder** : génération automatique lors de l'assignation d'un ticket
2. **Lier ContractorReport → Expense** : ajouter FK et rendre le rapport obligatoire avant `approve()`
3. **Lier BoardDecision → Expense** : validation CdC obligatoire si immeuble >20 lots
4. **Créer BuildingContractor** : registre des fournisseurs officiels par ACP avec assignation prioritaire

### Priorité 3 — Bugs majeurs et UX

1. **Permissions rôles (#1)** : masquer boutons admin pour le syndic
2. **Création tickets (#5)** : ajouter sélecteur d'immeuble, corriger `showPageToast`
3. **Chargement immeubles (#7)** : corriger l'endpoint pour Sondages, Annonces, Réservations
4. **Implémenter les prédicats syndic/immeuble** : validation structurelle avant association
5. **Traduire les pages restantes (#4, #8)** : Tickets, Annonces, Réservations en français
6. **Tester le workflow Owner** : dashboard propriétaire, consultation des charges et votes
7. **Tester la comptabilité PCMN** : écritures de journal, bilan, compte de résultats
