# Guide Utilisateur - Conseil de Copropriété (Board of Directors)

**Issue #82** - Fonctionnalité CRITIQUE pour conformité légale belge

---

## 📋 Table des Matières

1. [Introduction](#introduction)
2. [Obligation Légale](#obligation-légale)
3. [Rôle du Conseil](#rôle-du-conseil)
4. [Élection des Membres](#élection-des-membres)
5. [Gestion des Mandats](#gestion-des-mandats)
6. [Suivi des Décisions AG](#suivi-des-décisions-ag)
7. [Tableau de Bord](#tableau-de-bord)
8. [Guide Technique](#guide-technique)

---

## 1. Introduction

Le **Conseil de Copropriété** (aussi appelé "Board of Directors" ou "Raad van Mede-eigendom" en néerlandais) est un organe de contrôle obligatoire dans les copropriétés belges de plus de 20 lots.

### Qu'est-ce que le Conseil de Copropriété ?

Le conseil est composé de copropriétaires élus lors de l'Assemblée Générale (AG) pour :
- ✅ Surveiller l'exécution des décisions de l'AG par le syndic
- ✅ Vérifier la gestion financière
- ✅ Donner son avis sur les travaux et contrats importants
- ✅ Convoquer une AG si nécessaire

---

## 2. Obligation Légale

### Article 577-8/4 du Code Civil belge

> **"Dans les immeubles comportant plus de 20 lots, l'assemblée générale désigne, parmi les copropriétaires, un conseil de copropriété."**

#### Points clés :
- 🔴 **OBLIGATOIRE** pour immeubles >20 lots
- 📅 Mandats d'environ **1 an** (renouvelables)
- 🚫 **Incompatibilité** : Le syndic ne peut PAS être membre du conseil
- ⚖️ Sanctions possibles si absent (nullité décisions AG)

#### Sanctions en cas de non-conformité :
- ❌ Nullité potentielle des décisions prises en AG
- ⚠️ Responsabilité du syndic engagée
- 💰 Risque d'amendes

---

## 3. Rôle du Conseil

### Missions Principales

#### 1. **Surveillance du Syndic**
- Vérifier l'exécution des décisions AG dans les délais
- Contrôler les comptes et la gestion financière
- Demander des justificatifs pour les dépenses importantes

#### 2. **Conseil et Avis**
- Donner son avis sur les travaux proposés
- Examiner les devis et contrats
- Recommander des améliorations

#### 3. **Alerte et Convocation**
- Signaler les manquements du syndic
- Convoquer une AG extraordinaire si nécessaire
- Informer les copropriétaires

#### 4. **Suivi des Décisions**
- Créer et suivre les décisions prises en AG
- Surveiller les deadlines d'exécution
- Alerter en cas de retard

### Droits du Conseil
- 📄 Consulter tous les documents de la copropriété
- 🔍 Demander des comptes au syndic
- 🗣️ Prendre la parole en AG
- 📞 Convoquer une AG si le syndic est défaillant

---

## 4. Élection des Membres

### Composition Recommandée

Pour un immeuble de **20-50 lots** :
- 👑 **1 Président** (préside les réunions du conseil)
- 💰 **1 Trésorier** (suit les comptes)
- 📝 **1 Secrétaire** (rédige les comptes-rendus) *(optionnel)*
- 👤 **1-2 Membres** (participent aux décisions)

Pour un immeuble de **>50 lots** :
- Jusqu'à **5 membres** recommandés

### Processus d'Élection

#### Étape 1 : Proposition en AG
1. Le syndic ou un copropriétaire propose des candidatures
2. Les candidats peuvent se présenter eux-mêmes
3. Vote à **majorité simple** (50% + 1 voix)

#### Étape 2 : Enregistrement dans KoproGo
```
1. Aller sur "Conseil de Copropriété"
2. Cliquer "Élire un membre"
3. Sélectionner le copropriétaire
4. Choisir la position (Président, Trésorier, etc.)
5. Indiquer l'AG d'élection
6. Valider
```

#### Étape 3 : Attribution des Rôles
- **Président** : Coordonne le conseil, préside les réunions
- **Trésorier** : Vérifie les comptes, budget prévisionnel
- **Secrétaire** : Rédige PV des réunions du conseil *(si applicable)*
- **Membre** : Participe aux décisions et avis

### Incompatibilités ⚠️

**Le système KoproGo bloque automatiquement** :
- ❌ Un syndic ne peut PAS être élu au conseil
- ❌ Un membre du conseil ne peut PAS devenir syndic
- ✅ Vérification via trigger SQL en base de données

---

## 5. Gestion des Mandats

### Durée et Renouvellement

#### Durée du Mandat
- **~1 an** (entre 11 et 13 mois)
- Commence à la date de l'AG d'élection
- Se termine à l'AG suivante

#### Renouvellement
```
Alerte automatique à J-60 :
"⚠️ Le mandat de [Nom] expire dans 60 jours.
Pensez à organiser une nouvelle élection lors de la prochaine AG."
```

#### Processus de Renouvellement
1. **60 jours avant** expiration : Alerte orange dans le dashboard
2. **Lors de la prochaine AG** : Vote de renouvellement
3. Dans KoproGo : Cliquer "Renouveler le mandat"
4. Nouveau mandat de 1 an commence automatiquement

### Démission ou Révocation

#### Démission Volontaire
- Le membre informe le président par écrit
- Annonce en AG lors de la prochaine session
- Élection d'un remplaçant si nécessaire

#### Révocation
- Vote en AG à majorité simple
- Dans KoproGo : "Retirer du conseil"
- Le mandat devient inactif immédiatement

---

## 6. Suivi des Décisions AG

### Types de Décisions à Suivre

#### Décisions avec Délai
- 🔨 **Travaux votés** : "Réparer l'ascenseur" → Deadline 60 jours
- 📝 **Obtention de devis** : "3 devis pour la toiture" → Deadline 30 jours
- 📄 **Documents** : "Envoi PV de l'AG" → Deadline 30 jours légal

#### Décisions sans Délai
- 📋 **Études de faisabilité** : "Analyser installation panneaux solaires"
- 🏛️ **Règlement intérieur** : "Mise à jour du ROI"

### Workflow de Suivi

#### 1. Créer une Décision
```
Après chaque AG :
1. Aller sur "Suivi des Décisions"
2. Cliquer "Nouvelle décision"
3. Sujet : "Réparation ascenseur"
4. Texte : "Approuvé travaux pour 15,000€"
5. Deadline : +60 jours
6. AG de référence : "AG Annuelle 2024"
7. Enregistrer
```

#### 2. Statuts des Décisions
- 🔵 **En attente** (pending) : Décision prise, pas encore démarrée
- 🟡 **En cours** (in_progress) : Le syndic a commencé l'exécution
- 🟢 **Terminée** (completed) : Décision exécutée et validée
- 🔴 **En retard** (overdue) : Deadline dépassée
- ⚫ **Annulée** (cancelled) : Décision abandonnée

#### 3. Alertes Automatiques
```
📅 Si deadline < 7 jours : Alerte CRITIQUE (rouge)
📅 Si deadline < 30 jours : Alerte ÉLEVÉE (orange)
📅 Si deadline dépassée : Alerte EN RETARD (rouge clignotant)
```

#### 4. Actions Disponibles
- **Démarrer** : Passer de "En attente" → "En cours"
- **Terminer** : Passer de "En cours" → "Terminée"
- **Ajouter notes** : Commentaires de suivi du conseil
- **Annuler** : Marquer comme annulée

### Exemple Concret

```
Décision : "Obtenir 3 devis pour réfection toiture"
Statut : En cours 🟡
Deadline : 15/12/2024 (dans 12 jours) 🟠
Notes : "Syndic a contacté 2 entrepreneurs.
         En attente 3ème devis de Toitures Expert."
```

---

## 7. Tableau de Bord

### Vue d'Ensemble

Le **Tableau de Bord du Conseil** est accessible via :
```
Navigation → Conseil de Copropriété → Tableau de Bord
```

### Sections du Dashboard

#### 1. Mon Mandat
```
┌─────────────────────────────────────┐
│ Mon Mandat                          │
├─────────────────────────────────────┤
│ Position : Président 👑             │
│ Début : 15/03/2024                  │
│ Fin : 15/03/2025                    │
│                                     │
│ ⚠️ Expire dans 45 jours            │
│ Pensez à organiser une nouvelle    │
│ élection lors de la prochaine AG.  │
└─────────────────────────────────────┘
```

#### 2. Statistiques des Décisions
```
┌────────────────────────────────────────────┐
│ Statistiques des Décisions                 │
├────────────────────────────────────────────┤
│ Total : 23   En attente : 5  🔵           │
│ En cours : 8  🟡  Terminées : 8  🟢       │
│ En retard : 2  🔴  Annulées : 0  ⚫       │
└────────────────────────────────────────────┘
```

#### 3. Décisions en Retard (si applicable)
```
┌─────────────────────────────────────────────┐
│ 🚨 Décisions en Retard (2)                 │
├─────────────────────────────────────────────┤
│ • Obtenir 3 devis toiture                  │
│   Deadline : 01/11/2024 (dépassée 3 jours)│
│                                            │
│ • Envoi PV AG extraordinaire               │
│   Deadline : 05/11/2024 (dépassée 1 jour) │
└─────────────────────────────────────────────┘
```

#### 4. Deadlines Approchant
```
┌─────────────────────────────────────────────┐
│ 📅 Deadlines Approchant (3)                │
├─────────────────────────────────────────────┤
│ 🔴 Réparation ascenseur (dans 5 jours)    │
│ 🟠 Contrat chauffage (dans 15 jours)      │
│ 🟡 Budget prévisionnel (dans 28 jours)    │
└─────────────────────────────────────────────┘
```

### Codes Couleurs

| Urgence | Délai | Couleur | Icône |
|---------|-------|---------|-------|
| Critique | < 7 jours | Rouge | 🔴 |
| Élevée | < 30 jours | Orange | 🟠 |
| Normale | > 30 jours | Jaune | 🟡 |
| En retard | Dépassée | Rouge vif | 🚨 |

---

## 8. Guide Technique

### Architecture Backend

#### Entités Domain
```rust
// BoardMember
pub struct BoardMember {
    pub id: Uuid,
    pub owner_id: Uuid,          // Référence copropriétaire
    pub building_id: Uuid,
    pub position: BoardPosition, // president, treasurer, member
    pub mandate_start: DateTime<Utc>,
    pub mandate_end: DateTime<Utc>,
    pub elected_by_meeting_id: Uuid,
}

// BoardDecision
pub struct BoardDecision {
    pub id: Uuid,
    pub building_id: Uuid,
    pub meeting_id: Uuid,
    pub subject: String,
    pub decision_text: String,
    pub deadline: Option<DateTime<Utc>>,
    pub status: DecisionStatus, // pending, in_progress, completed, overdue
    pub notes: Option<String>,
}
```

#### API Endpoints

**Board Members:**
```
POST   /api/v1/board-members/elect              # Élire un membre
GET    /api/v1/board-members/building/{id}      # Liste membres actifs
GET    /api/v1/board-members/building/{id}/all  # Tous les membres
PUT    /api/v1/board-members/{id}/renew         # Renouveler mandat
DELETE /api/v1/board-members/{id}               # Retirer du conseil
GET    /api/v1/board-members/stats/{building}   # Statistiques
GET    /api/v1/board-members/my-mandates        # Mes mandats
GET    /api/v1/board-members/dashboard          # Dashboard
```

**Board Decisions:**
```
POST   /api/v1/board-decisions                           # Créer décision
GET    /api/v1/board-decisions/{id}                      # Détails
GET    /api/v1/board-decisions/building/{id}             # Par immeuble
GET    /api/v1/board-decisions/building/{id}/status/{s}  # Par statut
GET    /api/v1/board-decisions/building/{id}/overdue     # En retard
PUT    /api/v1/board-decisions/{id}/status               # Changer statut
PUT    /api/v1/board-decisions/{id}/complete             # Terminer
PUT    /api/v1/board-decisions/{id}/notes                # Ajouter notes
GET    /api/v1/board-decisions/stats/{building}          # Statistiques
```

#### Migrations Base de Données

**3 migrations créées :**
1. `20251101000001_add_board_member_role.sql`
   - Ajoute rôle `board_member` à la table `user_roles`

2. `20251101000002_create_board_system.sql`
   - Tables `board_members` et `board_decisions`
   - ENUMs `board_position` et `decision_status`
   - **Triggers incompatibilité syndic/board** ⚠️

3. `20251101000003_add_missing_columns_to_board_tables.sql`
   - Ajout `organization_id` (multi-tenancy)
   - Ajout `is_active` (gestion mandats)

#### Triggers SQL Critiques

```sql
-- Interdire syndic d'être board member
CREATE TRIGGER enforce_syndic_board_incompatibility
BEFORE INSERT OR UPDATE ON board_members
FOR EACH ROW EXECUTE FUNCTION check_syndic_board_incompatibility();

-- Interdire board member de devenir syndic
CREATE TRIGGER enforce_board_syndic_incompatibility
BEFORE INSERT OR UPDATE ON user_roles
FOR EACH ROW EXECUTE FUNCTION check_board_syndic_incompatibility();
```

### Composants Frontend

**3 composants Svelte :**
1. **`BoardDashboard.svelte`** (252 lignes)
   - Dashboard principal avec statistiques
   - Affichage mandat actif
   - Alertes overdue et deadlines
   - Vue agrégée complète

2. **`BoardMemberList.svelte`** (195 lignes)
   - Liste tous les membres du conseil
   - Positions avec icônes
   - Statuts mandats (actif, expirant, inactif)
   - Toggle anciens membres

3. **`DecisionTracker.svelte`** (261 lignes)
   - Suivi toutes décisions AG
   - Filtrage par statut
   - Actions rapides (Démarrer, Terminer)
   - Alertes visuelles

### Tests

**Tests Unitaires (46 tests):**
- `BoardMember` : 22 tests
- `BoardDecision` : 24 tests

**Tests BDD (45+ scénarios):**
- `board_members.feature` : 15 scénarios
- `board_decisions.feature` : 25 scénarios
- `board_dashboard.feature` : 5 scénarios

**Tests E2E (15 scénarios):**
- `BoardOfDirectors.spec.ts` : Tests Playwright complets

---

## FAQ

### Q: Le conseil est-il obligatoire pour mon immeuble de 18 lots ?
**R:** Non. Le conseil n'est obligatoire que pour les immeubles de **plus de 20 lots**. Cependant, vous pouvez en créer un volontairement.

### Q: Combien de membres dans le conseil ?
**R:** La loi ne fixe pas de nombre minimum. Recommandation :
- 20-50 lots : 3-4 membres
- >50 lots : 5 membres

### Q: Le syndic peut-il assister aux réunions du conseil ?
**R:** Oui, le syndic peut être invité pour donner des explications, mais il ne vote pas et ne peut pas être membre.

### Q: Que se passe-t-il si le syndic ne respecte pas les décisions AG ?
**R:** Le conseil peut :
1. Envoyer une mise en demeure
2. Convoquer une AG extraordinaire
3. Proposer le remplacement du syndic
4. Engager la responsabilité du syndic

### Q: Les mandats peuvent-ils être révoqués ?
**R:** Oui, par vote en AG à majorité simple. Dans KoproGo : "Retirer du conseil".

### Q: Comment sont alertées les décisions en retard ?
**R:** Alertes automatiques dans le dashboard :
- 🟡 < 30 jours avant deadline
- 🔴 < 7 jours avant deadline
- 🚨 Après deadline dépassée

---

## Support et Contact

- 📧 Email support : support@koprogo.com
- 📚 Documentation complète : https://docs.koprogo.com
- 🐛 Signaler un bug : https://github.com/gilmry/koprogo/issues

---

## Conformité Légale

✅ **Conforme Article 577-8/4 Code Civil belge**
✅ **Validation trigger SQL incompatibilité syndic/board**
✅ **Alertes mandats expirants**
✅ **Audit trail complet**

**Version:** 1.0.0
**Date:** Novembre 2024
**Issue:** #82 - Board of Directors (CRITICAL)
