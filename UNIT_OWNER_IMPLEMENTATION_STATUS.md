# Statut d'implémentation UnitOwner

## ✅ Phase 1 : Base de données (TERMINÉ)
- [x] Migration SQL créée
- [x] Table `unit_owners` créée
- [x] Vue `v_current_unit_owners` créée
- [x] Fonctions PostgreSQL créées
- [x] Migration exécutée avec succès
- [x] 123 relations migrées

## ✅ Phase 2 : Backend - Domain & Application (EN COURS)

### ✅ Domain Layer
- [x] Entity `UnitOwner` créée avec tests unitaires
- [x] Export dans `domain/entities/mod.rs`

### ✅ Application Layer - Ports
- [x] Port `UnitOwnerRepository` créé
- [x] Export dans `application/ports/mod.rs`

### ✅ Infrastructure Layer - Repositories
- [x] `PostgresUnitOwnerRepository` implémenté
- [x] Export dans `infrastructure/database/repositories/mod.rs`

### 🔄 Application Layer - Use Cases (À FAIRE)
- [ ] Créer `UnitOwnerUseCases`
- [ ] Méthodes :
  - [ ] `add_owner_to_unit(unit_id, owner_id, percentage, is_primary)`
  - [ ] `remove_owner_from_unit(unit_id, owner_id)`
  - [ ] `update_ownership_percentage(unit_owner_id, new_percentage)`
  - [ ] `transfer_ownership(from_owner, to_owner, unit_id)`
  - [ ] `get_unit_owners(unit_id)` - Avec infos détaillées
  - [ ] `get_owner_units(owner_id)` - Avec infos détaillées
  - [ ] `set_primary_contact(unit_owner_id)`

### 🔄 Application Layer - DTOs (À FAIRE)
- [ ] `AddOwnerToUnitDto`
- [ ] `UpdateOwnershipDto`
- [ ] `UnitOwnerResponseDto`
- [ ] `UnitWithOwnersDto`
- [ ] `OwnerWithUnitsDto`

### 🔄 Infrastructure Layer - Web (À FAIRE)
- [ ] Handlers HTTP :
  - [ ] `POST /units/{unit_id}/owners` - Ajouter un owner
  - [ ] `DELETE /units/{unit_id}/owners/{owner_id}` - Retirer un owner
  - [ ] `PUT /unit-owners/{id}` - Modifier quote-part
  - [ ] `GET /units/{unit_id}/owners` - Liste des owners d'un lot
  - [ ] `GET /owners/{owner_id}/units` - Liste des lots d'un owner
- [ ] Ajouter les routes dans `routes.rs`
- [ ] Ajouter `UnitOwnerUseCases` dans `AppState`

### 🔄 Owner Entity Refactoring (À FAIRE)
- [ ] Ajouter `organization_id: Uuid` dans `Owner` entity
- [ ] Mettre à jour le constructeur `Owner::new()`
- [ ] Mettre à jour les tests

## ⏳ Phase 3 : Frontend (À FAIRE)

### Interface Utilisateur
- [ ] Composant `UnitOwnersManager.svelte`
  - [ ] Liste des owners d'un lot avec quote-part
  - [ ] Bouton "Ajouter un owner"
  - [ ] Bouton "Retirer" par owner
  - [ ] Édition de la quote-part
  - [ ] Indicateur de contact principal
  - [ ] Validation : total ownership <= 100%

- [ ] Modal `AddOwnerToUnit.svelte`
  - [ ] Sélecteur d'owner
  - [ ] Input de quote-part (%)
  - [ ] Checkbox "Contact principal"
  - [ ] Création d'owner si inexistant

- [ ] Composant `OwnerBuildingSelector.svelte`
  - [ ] Affichage des lots de l'owner
  - [ ] Sélection de la copropriété active
  - [ ] Stockage dans store Svelte

- [ ] Intégration dans `UnitDetail.svelte`
  - [ ] Section "Propriétaires"
  - [ ] Utilisation de `UnitOwnersManager`

- [ ] Dashboard Owner amélioré
  - [ ] Sélecteur de copropriété au login
  - [ ] Filtrage par building sélectionné

### API Integration
- [ ] Créer les fonctions dans `api.ts`
- [ ] Gérer les erreurs
- [ ] Loading states

## 📊 Métriques

- **Entités créées** : 1/1 ✅
- **Repositories** : 1/1 ✅
- **Use Cases** : 0/1 ⏳
- **DTOs** : 0/5 ⏳
- **Handlers** : 0/5 ⏳
- **Routes** : 0/5 ⏳
- **Composants frontend** : 0/5 ⏳

## 🎯 Prochaines étapes immédiates

1. **Use Cases** : Créer `UnitOwnerUseCases` avec toute la logique métier
2. **DTOs** : Définir les contrats d'API
3. **Handlers** : Exposer les endpoints HTTP
4. **Frontend** : Créer les composants UI

## 📝 Notes

- La table `units.owner_id` est conservée pour compatibilité mais **DEPRECATED**
- À terme, supprimer `units.owner_id` quand tout le code utilise `unit_owners`
- Le frontend actuel continue de fonctionner avec l'ancien modèle
- Transition progressive vers le nouveau modèle
