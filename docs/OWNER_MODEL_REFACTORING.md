# Refactoring du modèle Owner - Multi-copropriété

## 🎯 Objectif

Permettre à un copropriétaire (owner) de :
- Posséder plusieurs lots dans différents immeubles
- Partager la propriété d'un lot avec d'autres copropriétaires (indivision, SCI, etc.)
- Utiliser le même email dans différentes organisations
- Se connecter et choisir la copropriété concernée

## 📊 Ancien modèle (PROBLÉMATIQUE)

```sql
owners (
    email UNIQUE,  -- ❌ Impossible d'avoir le même owner dans plusieurs organisations
    ...
)

units (
    owner_id UUID,  -- ❌ Un seul propriétaire par lot
    ...
)
```

### Limitations :
1. **Email unique global** : Un owner ne peut exister que dans une seule organisation
2. **Relation 1-1** : Un lot ne peut avoir qu'un seul propriétaire
3. **Pas de copropriété** : Impossible de gérer l'indivision, les SCI, etc.
4. **Pas d'historique** : Impossible de savoir qui était propriétaire avant

## ✅ Nouveau modèle (SOLUTION)

```sql
owners (
    id UUID,
    email VARCHAR,  -- ✅ Peut être dupliqué
    organization_id UUID,  -- ✅ Rattaché à une organisation
    ...
    UNIQUE(email, organization_id)  -- ✅ Email unique par organisation
)

unit_owners (  -- ✅ Table de liaison Many-to-Many
    unit_id UUID,
    owner_id UUID,
    ownership_percentage DOUBLE,  -- ✅ Quote-part (ex: 0.5 = 50%)
    is_primary_contact BOOLEAN,  -- ✅ Contact principal
    start_date TIMESTAMPTZ,  -- ✅ Date de début
    end_date TIMESTAMPTZ,  -- ✅ Date de fin (NULL si actuel)
)
```

## 🔧 Changements techniques

### 1. Table `owners`
- ✅ Ajout de `organization_id`
- ✅ Suppression de la contrainte `UNIQUE(email)`
- ✅ Ajout de `UNIQUE(email, organization_id)`

### 2. Nouvelle table `unit_owners`
- Liaison Many-to-Many entre `units` et `owners`
- Champs :
  - `ownership_percentage` : Quote-part de propriété (0.0 à 1.0)
  - `is_primary_contact` : Contact principal pour ce lot
  - `start_date` / `end_date` : Historique de propriété

### 3. Table `units`
- 🟡 `owner_id` marqué comme **DEPRECATED**
- 🔴 À supprimer dans une future version

## 📝 Cas d'usage couverts

### Cas 1 : Owner dans plusieurs copropriétés
```
Jean Dupont (jean@email.com)
├─ Organisation A : Résidence Les Pins
│  └─ Lot 12 (100%)
└─ Organisation B : Résidence Le Parc
   └─ Lot 45 (100%)
```

### Cas 2 : Lot en indivision (plusieurs propriétaires)
```
Lot 23 - Résidence Belle Vue
├─ Marie Martin (50%)  [contact principal]
└─ Pierre Durand (50%)
```

### Cas 3 : SCI avec plusieurs associés
```
Lot 67 - Résidence du Port
├─ SCI Investissement (100%)
   └─ Représentée par :
      ├─ Sophie Legrand (gérant)
      └─ Marc Petit (associé)
```

### Cas 4 : Historique de propriété
```
Lot 89 - Timeline
├─ 2020-2023 : Paul Rousseau (100%)
└─ 2023-now : Emma Bernard (100%)
```

## 🔍 Requêtes utiles

### Obtenir tous les propriétaires d'un lot
```sql
SELECT * FROM get_unit_owners('unit-uuid');
```

### Obtenir tous les lots d'un propriétaire
```sql
SELECT * FROM get_owner_units('owner-uuid');
```

### Propriétaires actuels (vue)
```sql
SELECT * FROM v_current_unit_owners WHERE unit_id = 'uuid';
```

## 🚀 Migration progressive

### Phase 1 : Migration SQL (✅ FAIT)
- Exécuter `20250127000000_refactor_owners_multitenancy.sql`
- Les données existantes sont migrées automatiquement

### Phase 2 : Backend (🔄 À FAIRE)
1. Mettre à jour les entities :
   - `Owner` : Ajouter `organization_id`
   - Créer `UnitOwner` entity
2. Mettre à jour les repositories :
   - `OwnerRepository` : Filtrer par `organization_id`
   - Créer `UnitOwnerRepository`
3. Mettre à jour les use cases :
   - Adapter les queries pour utiliser `unit_owners`
4. Mettre à jour les DTOs

### Phase 3 : Frontend (🔄 À FAIRE)
1. Afficher plusieurs propriétaires par lot
2. Permettre l'ajout/suppression de propriétaires
3. Afficher la quote-part de chaque propriétaire
4. Interface de sélection de copropriété au login (pour les owners)

## ⚠️ Rétrocompatibilité

- `units.owner_id` est **conservé temporairement**
- Marqué comme **DEPRECATED**
- Les anciens endpoints continuent de fonctionner
- À supprimer dans la version 2.0

## 📋 Checklist d'implémentation

- [x] Migration SQL créée
- [ ] Exécuter la migration
- [ ] Mettre à jour `Owner` entity
- [ ] Créer `UnitOwner` entity
- [ ] Mettre à jour les repositories
- [ ] Mettre à jour les use cases
- [ ] Mettre à jour les handlers
- [ ] Mettre à jour le frontend
- [ ] Mettre à jour les tests
- [ ] Documentation utilisateur

## 🎓 Impact sur l'authentification

### Pour les Owners qui se connectent :

**Avant** :
```
Owner login → Dashboard unique
```

**Après** :
```
Owner login
  └─ Sélection de la copropriété
      ├─ Résidence A (Lot 12)
      ├─ Résidence B (Lot 45)
      └─ Résidence C (Lot 89)
         └─ Dashboard de la copropriété sélectionnée
```

### Implémentation suggérée :
1. Après login, récupérer tous les lots de l'owner
2. Si plusieurs lots → afficher un sélecteur
3. Stocker le `selected_building_id` dans le store Svelte
4. Filtrer les données du dashboard selon le building sélectionné

## 💡 Avantages du nouveau modèle

1. ✅ **Flexibilité** : Gère tous les cas réels de copropriété
2. ✅ **Multi-tenant** : Owners isolés par organisation
3. ✅ **Historique** : Traçabilité des changements de propriété
4. ✅ **Indivision** : Gestion des copropriétés multiples
5. ✅ **Evolutif** : Facile d'ajouter de nouveaux cas d'usage
6. ✅ **Performance** : Indexes optimisés pour les requêtes courantes

## 📚 Références

- [Documentation PostgreSQL - Many-to-Many](https://www.postgresql.org/docs/current/ddl-constraints.html)
- [Architecture hexagonale - Domain modeling](https://herbertograca.com/2017/11/16/explicit-architecture-01-ddd-hexagonal-onion-clean-cqrs-how-i-put-it-all-together/)
