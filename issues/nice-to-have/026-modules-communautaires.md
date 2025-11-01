# Issue #026 - Modules Communautaires (SEL, Troc, Partage)

**Priorité**: 🟢 MEDIUM
**Estimation**: 15-20 heures
**Labels**: `enhancement`, `backend`, `frontend`, `community`, `social-impact`

---

## 📋 Description

Implémenter les **5 modules communautaires** pour renforcer le lien social dans les copropriétés, conformément à la mission ASBL de KoproGo ("résoudre phénomènes des sociétés").

**Modules** :
1. **SEL** (Système d'Échange Local) - Troc compétences
2. **Bazar de Troc** - Échange/don objets
3. **Prêt d'Objets** - Bibliothèque outils/équipements
4. **Annuaire Compétences** - Listing habitants
5. **Tableau Affichage** - Petites annonces numériques

**Impact** : Différenciateur fort vs concurrents, mission sociale.

---

## 🎯 Objectifs

### 1. SEL (Système d'Échange Local)

**Concept** : Troc de compétences entre habitants (heures de service)

**Entities** :
```rust
struct SkillOffer {
    id: Uuid,
    owner_id: Uuid,
    category: SkillCategory, // Bricolage, Jardinage, Cours, etc.
    title: String,
    description: String,
    hourly_rate_credits: i32, // Crédits SEL, pas €
}

struct SkillExchange {
    id: Uuid,
    offer_id: Uuid,
    requester_id: Uuid,
    provider_id: Uuid,
    hours: f32,
    status: ExchangeStatus,
    rating: Option<i32>, // 1-5 étoiles
}
```

**Features** :
- [ ] Publier offres de compétences
- [ ] Rechercher par catégorie
- [ ] Demander service
- [ ] Tracker crédits SEL (système monnaie locale)
- [ ] Notation après échange

---

### 2. Bazar de Troc

**Concept** : Échange ou don d'objets entre habitants

**Entities** :
```rust
struct SwapItem {
    id: Uuid,
    owner_id: Uuid,
    title: String,
    description: String,
    category: ItemCategory,
    condition: ItemCondition, // Neuf, Bon, Usagé
    offer_type: OfferType, // Échange, Don, Prêt
    images: Vec<String>,
}

struct SwapTransaction {
    id: Uuid,
    item_id: Uuid,
    requester_id: Uuid,
    completed_at: DateTime<Utc>,
    rating: Option<i32>,
}
```

**Features** :
- [ ] Publier annonces objets
- [ ] Upload photos
- [ ] Filtrer par catégorie
- [ ] Messagerie interne
- [ ] Historique échanges

---

### 3. Prêt d'Objets

**Concept** : Bibliothèque d'outils partagés (perceuse, échelle, tondeuse, etc.)

**Entities** :
```rust
struct ObjectLoan {
    id: Uuid,
    owner_id: Uuid,
    item_name: String,
    description: String,
    availability: AvailabilityStatus,
    current_borrower: Option<Uuid>,
    loan_start: Option<DateTime<Utc>>,
    loan_end: Option<DateTime<Utc>>,
}

struct LoanRequest {
    id: Uuid,
    loan_id: Uuid,
    requester_id: Uuid,
    requested_from: NaiveDate,
    requested_to: NaiveDate,
    status: RequestStatus,
}
```

**Features** :
- [ ] Catalogue objets prêtables
- [ ] Calendrier disponibilité
- [ ] Réservation en ligne
- [ ] Rappels retour
- [ ] Caution optionnelle

---

### 4. Annuaire Compétences

**Concept** : Listing compétences habitants (qui sait faire quoi ?)

**Implémentation** :
- Extension table `owners` avec champ `skills: JSONB`
- Tags compétences (plomberie, électricité, peinture, cuisine, musique, etc.)
- Recherche par compétence
- Niveau (débutant, intermédiaire, expert)

**Features** :
- [ ] Ajouter compétences à profil
- [ ] Recherche par compétence
- [ ] Demande mise en contact
- [ ] Optionnel : notation

---

### 5. Tableau Affichage Numérique

**Concept** : Petites annonces entre voisins

**Entities** :
```rust
struct Notice {
    id: Uuid,
    building_id: Uuid,
    author_id: Uuid,
    category: NoticeCategory, // Vente, Recherche, Événement, Info, Alerte
    title: String,
    content: String,
    expires_at: Option<DateTime<Utc>>,
    is_pinned: bool, // Syndic peut épingler annonces importantes
}
```

**Categories** :
- Vente/Achat
- Recherche (garde d'enfants, covoiturage, etc.)
- Événements (fête des voisins, barbecue, etc.)
- Informations générales
- Alertes (coupure eau, travaux, etc.)

**Features** :
- [ ] Publier annonce
- [ ] Modération syndic
- [ ] Expiration automatique (30 jours)
- [ ] Commentaires
- [ ] Signalement contenu inapproprié

---

## 📊 Impact Mesurable

### Metrics Tracking

```rust
struct CommunityMetrics {
    building_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,

    // SEL
    sel_exchanges_count: i32,
    sel_total_hours: f32,
    sel_economic_value: f64, // Heures * tarif moyen

    // Troc
    swap_transactions_count: i32,
    swap_items_donated: i32,
    swap_estimated_value: f64,

    // Prêt
    loans_count: i32,
    loans_savings_estimated: f64, // Économie location

    // Global
    participation_rate: f64, // % habitants actifs
    co2_saved_kg: f64, // Estimation économie CO2
}
```

**Rapport Impact Social Annuel** :
- Nombre d'échanges SEL
- Économies réalisées (€)
- CO2 économisé (kg)
- Taux participation
- Top 3 compétences échangées
- Gamification (badges participation)

---

## 🎮 Gamification

### Badges & Récompenses

- 🥇 **Super Voisin** : 10+ échanges SEL
- 🌱 **Écolo** : 20+ objets donnés
- 🔧 **Bricolo** : 5+ prêts outils
- 🎁 **Généreux** : 50+ crédits SEL donnés

### Leaderboard

- Top contributeurs SEL (par building)
- Building le plus actif (par organisation)

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] 5 modules opérationnels
- [ ] Moderation tools (syndic/board)
- [ ] Signalement contenu inapproprié
- [ ] Notifications échanges/emprunts
- [ ] Rapport impact annuel généré

### Techniques
- [ ] Tests E2E pour chaque module
- [ ] Performance: liste 100 items < 200ms
- [ ] Mobile-friendly (PWA)

### UX
- [ ] Onboarding tutoriel modules
- [ ] Badges visibles profil
- [ ] Leaderboard animé

---

## 🔗 Dépendances

- ✅ Owner entity exists
- Issue #009 : Notifications
- Issue #002 : Document upload (photos)
- Issue #010 : PWA (offline listings)

---

## 🚀 Checklist

- [ ] 1. Créer entities (5 modules)
- [ ] 2. Migrations SQL
- [ ] 3. Repositories + use cases
- [ ] 4. Handlers HTTP (CRUD pour chaque module)
- [ ] 5. Frontend: 5 pages dédiées
- [ ] 6. Metrics tracking + rapport
- [ ] 7. Gamification (badges, leaderboard)
- [ ] 8. Moderation tools
- [ ] 9. Tests (20+ tests)
- [ ] 10. Documentation usage communauté
- [ ] 11. Commit : `feat: implement community modules (SEL, swap, sharing, directory, board)`

---

**Créé le** : 2025-11-01
**Milestone** : v2.0 - Community Features (Phase 2 Roadmap - Issue #49)
**Impact Business** : MEDIUM - Différenciateur fort, mission sociale ASBL
**Effort** : 15-20 heures (X-Large)
