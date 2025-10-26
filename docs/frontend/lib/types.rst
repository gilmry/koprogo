types.ts - Types TypeScript
============================

**Localisation** : ``frontend/src/lib/types.ts``

Définit toutes les interfaces TypeScript pour les entités du domaine et les types utilitaires.

Énumérations
------------

UserRole
^^^^^^^^

Rôles utilisateurs dans la plateforme SaaS multi-tenant.

.. code-block:: typescript

   export enum UserRole {
     SUPERADMIN = "superadmin",  // Administrateur plateforme
     SYNDIC = "syndic",          // Gestionnaire de copropriété
     ACCOUNTANT = "accountant",   // Comptable (consultation)
     OWNER = "owner"              // Copropriétaire (consultation)
   }

**Hiérarchie des Permissions** :

1. **SUPERADMIN** (niveau 4) : Gestion organisations, utilisateurs, configuration
2. **SYNDIC** (niveau 3) : Gestion complète des immeubles, charges, copropriétaires
3. **ACCOUNTANT** (niveau 2) : Consultation comptable, génération rapports
4. **OWNER** (niveau 1) : Consultation uniquement (ses lots, charges)

Interfaces Entités Domaine
---------------------------

User
^^^^

Utilisateur de la plateforme.

.. code-block:: typescript

   export interface User {
     id: string;
     email: string;
     firstName: string;
     lastName: string;
     role: UserRole;
     organizationId?: string;    // Multi-tenant
     buildingIds?: string[];     // Immeubles accessibles
   }

**Champs** :

- ``id`` : UUID
- ``email`` : Email unique (authentification)
- ``firstName`` / ``lastName`` : Nom complet
- ``role`` : Rôle utilisateur (UserRole)
- ``organizationId`` : ID organisation (multi-tenant ASBL)
- ``buildingIds`` : Liste immeubles accessibles (isolation données)

Building
^^^^^^^^

Immeuble en copropriété.

.. code-block:: typescript

   export interface Building {
     id: string;
     name: string;
     address: string;
     city: string;
     postal_code: string;
     country: string;
     total_units: number;
     construction_year?: number;
     created_at?: string;
     updated_at?: string;
   }

**Champs** :

- ``id`` : UUID
- ``name`` : Nom de l'immeuble (ex: "Résidence du Parc")
- ``address`` / ``city`` / ``postal_code`` / ``country`` : Adresse complète
- ``total_units`` : Nombre total de lots
- ``construction_year`` : Année de construction (optionnel)
- ``created_at`` / ``updated_at`` : Timestamps ISO 8601

Owner
^^^^^

Copropriétaire.

.. code-block:: typescript

   export interface Owner {
     id: string;
     first_name: string;
     last_name: string;
     email: string;
     phone?: string;
     created_at?: string;
   }

**Champs** :

- ``id`` : UUID
- ``first_name`` / ``last_name`` : Nom complet
- ``email`` : Email de contact
- ``phone`` : Téléphone (optionnel)
- ``created_at`` : Timestamp création

**⚠️ GDPR** : Les données Owner sont sensibles (email, téléphone).

Unit
^^^^

Lot dans un immeuble.

.. code-block:: typescript

   export interface Unit {
     id: string;
     building_id: string;
     unit_number: string;
     floor: number;
     surface_area: number;
     ownership_share: number;
     unit_type: "Apartment" | "Parking" | "Storage";
     owner_id?: string;
   }

**Champs** :

- ``id`` : UUID
- ``building_id`` : Référence Building
- ``unit_number`` : Numéro de lot (ex: "A-12")
- ``floor`` : Étage (0 = RDC, -1 = Sous-sol)
- ``surface_area`` : Surface en m²
- ``ownership_share`` : Quote-part en millièmes (ex: 45 = 45/1000)
- ``unit_type`` : Type de lot (Apartment, Parking, Storage)
- ``owner_id`` : Référence Owner (optionnel si vacant)

Expense
^^^^^^^

Charge de copropriété.

.. code-block:: typescript

   export interface Expense {
     id: string;
     building_id: string;
     description: string;
     amount: number;
     expense_date: string;
     due_date: string;
     category: "Maintenance" | "Repair" | "Insurance" |
               "Utilities" | "Management" | "Other";
     payment_status: "Pending" | "Paid" | "Overdue" | "Cancelled";
     paid_date?: string;
   }

**Champs** :

- ``id`` : UUID
- ``building_id`` : Référence Building
- ``description`` : Description de la charge
- ``amount`` : Montant en centimes (ex: 12050 = 120.50€)
- ``expense_date`` : Date de la dépense (ISO 8601)
- ``due_date`` : Date d'échéance (ISO 8601)
- ``category`` : Catégorie comptable
- ``payment_status`` : Statut de paiement
- ``paid_date`` : Date de paiement effectif (optionnel)

**Catégories** :

- **Maintenance** : Entretien courant
- **Repair** : Réparations
- **Insurance** : Assurances
- **Utilities** : Charges courantes (eau, électricité)
- **Management** : Honoraires syndic
- **Other** : Autres dépenses

Types Pagination
----------------

PageResponse<T>
^^^^^^^^^^^^^^^

Réponse paginée du backend.

.. code-block:: typescript

   export interface PageResponse<T> {
     data: T[];
     pagination: PaginationMeta;
   }

**Structure** :

.. code-block:: json

   {
     "data": [
       { "id": "...", "name": "..." },
       { "id": "...", "name": "..." }
     ],
     "pagination": {
       "current_page": 1,
       "per_page": 20,
       "total_items": 157,
       "total_pages": 8,
       "has_next": true,
       "has_previous": false
     }
   }

PaginationMeta
^^^^^^^^^^^^^^

Métadonnées de pagination.

.. code-block:: typescript

   export interface PaginationMeta {
     current_page: number;
     per_page: number;
     total_items: number;
     total_pages: number;
     has_next: boolean;
     has_previous: boolean;
   }

PageRequest
^^^^^^^^^^^

Paramètres de requête paginée.

.. code-block:: typescript

   export interface PageRequest {
     page?: number;      // Défaut: 1
     per_page?: number;  // Défaut: 20
   }

**Exemple d'utilisation** :

.. code-block:: typescript

   const response = await api.get<PageResponse<Building>>(
     `/buildings?page=${page}&per_page=${perPage}`
   );

   const buildings = response.data;
   const { current_page, total_pages, has_next } = response.pagination;

Helpers Permissions
-------------------

hasPermission(user, requiredRole)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Vérifie si l'utilisateur a le niveau de permission requis.

.. code-block:: typescript

   export const hasPermission = (
     user: User | null,
     requiredRole: UserRole
   ): boolean => {
     if (!user) return false;

     const roleHierarchy = {
       [UserRole.SUPERADMIN]: 4,
       [UserRole.SYNDIC]: 3,
       [UserRole.ACCOUNTANT]: 2,
       [UserRole.OWNER]: 1
     };

     return roleHierarchy[user.role] >= roleHierarchy[requiredRole];
   };

**Exemple** :

.. code-block:: typescript

   // SUPERADMIN peut tout faire
   hasPermission(superadminUser, UserRole.OWNER); // true

   // OWNER ne peut pas accéder aux fonctions SYNDIC
   hasPermission(ownerUser, UserRole.SYNDIC); // false

   // ACCOUNTANT peut accéder aux fonctions OWNER
   hasPermission(accountantUser, UserRole.OWNER); // true

canAccessBuilding(user, buildingId)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Vérifie si l'utilisateur peut accéder à un immeuble spécifique.

.. code-block:: typescript

   export const canAccessBuilding = (
     user: User | null,
     buildingId: string
   ): boolean => {
     if (!user) return false;
     if (user.role === UserRole.SUPERADMIN) return true;
     return user.buildingIds?.includes(buildingId) ?? false;
   };

**Logique** :

- **SUPERADMIN** : Accès à tous les immeubles
- **Autres rôles** : Accès uniquement aux immeubles dans ``buildingIds``

**Exemple** :

.. code-block:: svelte

   <script lang="ts">
     import { authStore } from '../stores/auth';
     import { canAccessBuilding } from '../lib/types';

     $: user = $authStore.user;
     $: canEdit = canAccessBuilding(user, building.id);
   </script>

   {#if canEdit}
     <button on:click={editBuilding}>Modifier</button>
   {/if}

Utilisation dans Components
----------------------------

**Import et Typage** :

.. code-block:: typescript

   import type { Building, Owner, PageResponse } from '../lib/types';
   import { UserRole, hasPermission } from '../lib/types';

**Variables Typées** :

.. code-block:: typescript

   let buildings: Building[] = [];
   let owners: Owner[] = [];
   let selectedBuilding: Building | null = null;

**Paramètres de Fonction** :

.. code-block:: typescript

   async function createBuilding(data: Partial<Building>) {
     const building = await api.post<Building>('/buildings', data);
     return building;
   }

**Reactive Statements** :

.. code-block:: typescript

   $: isSyndic = hasPermission($authStore.user, UserRole.SYNDIC);
   $: canManage = canAccessBuilding($authStore.user, buildingId);

Alignement Backend
------------------

Ces types doivent correspondre exactement aux DTOs du backend :

- ``Building`` ↔ ``backend/src/application/dto/building_dto.rs``
- ``Owner`` ↔ ``backend/src/application/dto/owner_dto.rs``
- ``Unit`` ↔ ``backend/src/application/dto/unit_dto.rs``
- ``Expense`` ↔ ``backend/src/application/dto/expense_dto.rs``

**⚠️ Important** : Toute modification backend nécessite mise à jour frontend.

Génération Automatique
-----------------------

Pour éviter désynchronisation, envisager génération automatique depuis OpenAPI :

.. code-block:: bash

   # Générer types depuis openapi.json
   npx openapi-typescript ./openapi.json --output ./src/lib/types.ts

**Alternatives** :

- **openapi-generator** : Génération complète client + types
- **tRPC** : Types partagés TypeScript (nécessite backend Node.js)
- **GraphQL Codegen** : Si migration vers GraphQL

Tests
-----

.. code-block:: typescript

   // tests/unit/types.test.ts
   import { describe, it, expect } from 'vitest';
   import { hasPermission, UserRole } from '../src/lib/types';

   describe('hasPermission', () => {
     it('SUPERADMIN has all permissions', () => {
       const user = { role: UserRole.SUPERADMIN } as User;
       expect(hasPermission(user, UserRole.OWNER)).toBe(true);
       expect(hasPermission(user, UserRole.SYNDIC)).toBe(true);
     });

     it('OWNER cannot access SYNDIC functions', () => {
       const user = { role: UserRole.OWNER } as User;
       expect(hasPermission(user, UserRole.SYNDIC)).toBe(false);
     });
   });

Références
----------

- Backend DTOs : ``backend/src/application/dto/``
- API Client : ``frontend/src/lib/api.ts``
- Auth Store : ``frontend/src/stores/auth.ts``
- Components : ``frontend/src/components/``
