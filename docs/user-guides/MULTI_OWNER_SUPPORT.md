# Multi-owner Support Guide

Suivi fonctionnel et technique du module de copropriété multi-détenteurs introduit en janvier 2025.

---

## Vue d'ensemble

- **Objectif** : permettre à un lot (unit) d'avoir plusieurs copropriétaires simultanés, chacun avec une quote-part (%), un contact principal et un historique complet.
- **Backend** : logique métier dans `backend/src/application/use_cases/unit_owner_use_cases.rs` et entité `backend/src/domain/entities/unit_owner.rs`.
- **Frontend** : composants Svelte centrés sur `frontend/src/components/UnitOwners.svelte`, `OwnerList.svelte`, `OwnerCreateModal.svelte` et `OwnerEditModal.svelte`.
- **Tests** : couverture via `backend/tests/integration_unit_owner.rs` (PostgreSQL réel) et scénarios BDD multi-tenant.

---

## Modèle de données

La table `unit_owners` crée une relation *many-to-many* entre `units` et `owners`.

```sql
CREATE TABLE unit_owners (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    unit_id UUID NOT NULL REFERENCES units(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    ownership_percentage DOUBLE PRECISION NOT NULL CHECK (ownership_percentage > 0 AND ownership_percentage <= 1),
    start_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    end_date TIMESTAMPTZ,
    is_primary_contact BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(unit_id, owner_id, end_date),
    CHECK (end_date IS NULL OR end_date > start_date)
);
```

### Invariants clés

- **Quote-part** : stockée sous forme décimale (`0.25` = 25%). La création/mise à jour refuse toute valeur `<= 0` ou `> 1`.
- **Somme des quotes-parts** : `UnitOwnerUseCases::add_owner_to_unit` et `update_ownership_percentage` vérifient que la somme active ne dépasse jamais `100 %`.
- **Temporalité** : `start_date` est fixé à la création, `end_date` est rempli lors d'un retrait ou d'un transfert.
- **Contact principal** : un seul copropriétaire actif peut être marqué `is_primary_contact = true`. La méthode interne `unset_all_primary_contacts` garantit l'unicité.

---

## Règles métier & validations

| Règle | Détails |
|-------|---------|
| Création d'un lien | Le lot et le copropriétaire doivent exister (`UnitRepository`, `OwnerRepository`). |
| Unicité active | Impossible d'ajouter deux fois le même copropriétaire tant que la relation actuelle n'est pas clôturée (`end_date IS NULL`). |
| Quote-part totale | Addition de toutes les quotes-part actives ≤ `1.0`. Message d'erreur explicite si dépassement. |
| Mise à jour | Impossible de modifier une relation dont `end_date` est renseignée. |
| Transfert | Clôture automatique de l'ancien propriétaire et création d'une nouvelle relation avec la même quote-part. |
| Historique | Méthodes `get_unit_ownership_history` et `get_owner_ownership_history` exposent les relations passées (`end_date` non nul). |

---

## Endpoints API

Tous les endpoints sont préfixés par `/api/v1`.

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `GET` | `/units/{unit_id}/owners` | Liste des copropriétaires **actifs** d'un lot. |
| `GET` | `/units/{unit_id}/owners/history` | Historique complet (actifs + inactifs). |
| `GET` | `/units/{unit_id}/owners/total-percentage` | Somme actuelle des quotes-parts. |
| `POST` | `/units/{unit_id}/owners` | Ajoute un copropriétaire (`AddOwnerToUnitDto`). |
| `DELETE` | `/units/{unit_id}/owners/{owner_id}` | Termine la relation (renseigne `end_date`). |
| `PUT` | `/unit-owners/{relationship_id}` | Met à jour la quote-part **ou** définit le contact principal. |
| `POST` | `/units/{unit_id}/owners/transfer` | Transfère une quote-part d'un propriétaire vers un autre. |
| `GET` | `/owners/{owner_id}/units` | Tous les lots actifs d'un copropriétaire. |
| `GET` | `/owners/{owner_id}/units/history` | Historique complet des lots possédés. |

> ℹ️ Les chemins d'update sont centralisés sur `/unit-owners/{id}` (contrainte technique : l'identifiant de relation est nécessaire pour conserver l'historique).

### DTO principaux

```jsonc
// POST /units/{unit_id}/owners
{
  "owner_id": "5a4b1b4b-09f7-4b3f-a591-5c0f0ffbfa98",
  "ownership_percentage": 0.4,
  "is_primary_contact": true
}
```

```jsonc
// PUT /unit-owners/{relationship_id}
{
  "ownership_percentage": 0.25
}
```

```jsonc
// POST /units/{unit_id}/owners/transfer
{
  "from_owner_id": "0edab1e9-6764-4a22-9d71-6c8a7d72f7bc",
  "to_owner_id": "5c613076-2abf-4bf9-b6aa-05260cdc7246"
}
```

Réponses typiques (`UnitOwnerResponseDto`) :

```jsonc
{
  "id": "8b296dd4-87c5-4d49-8a0a-c44faa7a0f05",
  "unit_id": "f216217a-987c-4684-8796-79f6792a4a2a",
  "owner_id": "5a4b1b4b-09f7-4b3f-a591-5c0f0ffbfa98",
  "ownership_percentage": 0.4,
  "start_date": "2025-01-27T10:12:45.123456Z",
  "end_date": null,
  "is_primary_contact": true,
  "is_active": true,
  "created_at": "2025-01-27T10:12:45.123456Z",
  "updated_at": "2025-01-27T10:12:45.123456Z"
}
```

---

## Frontend

| Composant | Rôle | Endpoints consommés |
|-----------|------|---------------------|
| `UnitOwners.svelte` | Vue embarquée sur la fiche lot (liste actifs, historique optionnel, somme des pourcentages, badges contact principal). | `/units/{id}/owners`, `/units/{id}/owners/history`, `/owners/{owner_id}` |
| `OwnerList.svelte` | Vue tableau paginée des copropriétaires avec accès à leurs lots (`OwnerUnits.svelte`). | `/owners`, `/owners/{id}/units` |
| `OwnerCreateModal.svelte` | Formulaire de création de copropriétaire (super-admin ⇒ choix organisation). | `POST /owners` |
| `OwnerEditModal.svelte` | Modification des coordonnées d'un copropriétaire. | `PUT /owners/{id}` |
| `OwnerUnits.svelte` | Vue détaillée des lots détenus par un copropriétaire (actifs + historique). | `/owners/{owner_id}/units`, `/owners/{owner_id}/units/history` |

Le total des quotes-parts est affiché en temps réel et surligné en rouge si ≠ 100 %. Les badges « Contact principal » s'appuient sur `is_primary_contact`.

---

## Scénarios d'usage

1. **Ajouter un nouveau copropriétaire principal**  
   1. Créer le copropriétaire via `OwnerCreateModal`.  
   2. Sur la fiche lot, utiliser le bouton « Ajouter » (`POST /units/{unit_id}/owners`).  
   3. Si `is_primary_contact = true`, les autres contacts sont automatiquement rétrogradés.

2. **Transférer une quote-part**  
   1. Appeler `POST /units/{unit_id}/owners/transfer` avec les identifiants source/cible.  
   2. L'ancien lien est clôturé (`end_date`) et un nouveau lien est créé avec la même quote-part.

3. **Mettre à jour une quote-part suite à un acte notarié**  
   1. Récupérer l'`id` de relation (`GET /units/{unit_id}/owners`).  
   2. Appeler `PUT /unit-owners/{id}` avec la nouvelle `ownership_percentage`.  
   3. Vérifier que la somme (`GET /units/{unit_id}/owners/total-percentage`) reste = `1.0`.

4. **Auditer l'historique d'un lot**  
   1. Consulter `/units/{unit_id}/owners/history`.  
   2. Afficher les périodes `start_date` → `end_date` pour chaque copropriétaire.

---

## Tests & Vérification

- **Unit tests** : `backend/src/domain/entities/unit_owner.rs` (validation pourcentage, dates, contact principal).
- **Use cases** : `backend/src/application/use_cases/unit_owner_use_cases.rs` (validation somme 100 %, transferts).
- **Intégration PostgreSQL** : `backend/tests/integration_unit_owner.rs`.
- **API** : couvertes par les tests BDD génériques (multi-tenancy) et les tests API `owner`/`unit` existants.
- **Frontend** : interactions testées via Playwright (liste des copropriétaires dans `frontend/tests/e2e/dashboards.spec.ts`).

Pour valider manuellement :

```bash
# Rejouer la batterie de tests liée
make test-unit                # inclut les tests domaine
make test-integration         # exécute integration_unit_owner.rs
make test-bdd                 # scénarios Cucumber
```

---

## Ressources liées

- `backend/migrations/20250127000000_refactor_owners_multitenancy.sql`
- `backend/src/infrastructure/web/handlers/unit_owner_handlers.rs`
- `docs/GIT_HOOKS.rst` – prérequis qualité (pre-commit/pre-push)
- `README.md` – aperçu des fonctionnalités multi-owner
- `CLAUDE.md` – architecture et règlement de contribution

---

✨ **Checklist release**

- [x] base de données migrée (`sqlx migrate run`)
- [x] cache SQLx préparé (`cargo sqlx prepare`)
- [x] tests verts (`make test`)
- [x] documentation Sphinx reconstruite (`make docs-sphinx`)
- [x] guides frontend/ops mis à jour
