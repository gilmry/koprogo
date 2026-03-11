Locales - Fichiers de Traduction
==================================

Fichiers JSON des traductions multilingues (nl, fr, de, en).

**Localisation** : ``frontend/src/locales/``

Structure
---------

.. code-block:: text

   locales/
   ├── nl.json  # Nederlands (néerlandais) - Langue par défaut
   ├── fr.json  # Français
   ├── de.json  # Deutsch (allemand)
   └── en.json  # English (anglais)

**Statistiques Belgique** :

- 🇳🇱 **60%** : Néerlandais (Flandre)
- 🇫🇷 **40%** : Français (Wallonie + Bruxelles)
- 🇩🇪 **<1%** : Allemand (Communauté germanophone)
- 🇬🇧 **Intl** : Anglais (syndics multinationales)

Format JSON
-----------

**Structure Plate** :

.. code-block:: json

   {
     "section.cle": "Traduction"
   }

**Conventions de Nommage** :

- ``[section].[clé]`` : ex: ``nav.dashboard``, ``building.create``
- snake_case pour clés : ``total_units`` pas ``totalUnits``
- Préfixes : ``nav.*``, ``error.*``, ``success.*``, ``button.*``

Sections Communes
-----------------

Navigation
^^^^^^^^^^

.. code-block:: json

   {
     "nav.dashboard": "Dashboard",
     "nav.buildings": "Immeubles",
     "nav.owners": "Copropriétaires",
     "nav.units": "Lots",
     "nav.expenses": "Charges",
     "nav.meetings": "Assemblées",
     "nav.documents": "Documents",
     "nav.reports": "Rapports",
     "nav.settings": "Paramètres",
     "nav.profile": "Profil",
     "nav.logout": "Déconnexion"
   }

Boutons Actions
^^^^^^^^^^^^^^^

.. code-block:: json

   {
     "button.create": "Créer",
     "button.edit": "Modifier",
     "button.delete": "Supprimer",
     "button.save": "Enregistrer",
     "button.cancel": "Annuler",
     "button.close": "Fermer",
     "button.search": "Rechercher",
     "button.filter": "Filtrer",
     "button.export": "Exporter",
     "button.download": "Télécharger",
     "button.upload": "Téléverser",
     "button.refresh": "Rafraîchir"
   }

Messages Succès/Erreur
^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: json

   {
     "success.saved": "Enregistré avec succès",
     "success.created": "Créé avec succès",
     "success.updated": "Mis à jour avec succès",
     "success.deleted": "Supprimé avec succès",

     "error.network": "Erreur réseau",
     "error.unauthorized": "Non autorisé",
     "error.forbidden": "Accès interdit",
     "error.not_found": "Non trouvé",
     "error.server": "Erreur serveur",
     "error.validation": "Erreur de validation",
     "error.unknown": "Erreur inconnue"
   }

Entités Domaine
---------------

Buildings
^^^^^^^^^

.. code-block:: json

   {
     "building.title": "Immeubles",
     "building.create": "Créer un immeuble",
     "building.edit": "Modifier l'immeuble",
     "building.delete": "Supprimer l'immeuble",
     "building.name": "Nom de l'immeuble",
     "building.address": "Adresse",
     "building.city": "Ville",
     "building.postal_code": "Code postal",
     "building.country": "Pays",
     "building.total_units": "Nombre de lots",
     "building.construction_year": "Année de construction",
     "building.details": "Détails de l'immeuble"
   }

Owners
^^^^^^

.. code-block:: json

   {
     "owner.title": "Copropriétaires",
     "owner.create": "Ajouter un copropriétaire",
     "owner.first_name": "Prénom",
     "owner.last_name": "Nom",
     "owner.email": "Email",
     "owner.phone": "Téléphone",
     "owner.units": "Lots possédés"
   }

Units
^^^^^

.. code-block:: json

   {
     "unit.title": "Lots",
     "unit.number": "Numéro de lot",
     "unit.floor": "Étage",
     "unit.surface_area": "Surface (m²)",
     "unit.ownership_share": "Quote-part (‰)",
     "unit.type": "Type de lot",
     "unit.type.apartment": "Appartement",
     "unit.type.parking": "Parking",
     "unit.type.storage": "Cave"
   }

Expenses
^^^^^^^^

.. code-block:: json

   {
     "expense.title": "Charges",
     "expense.description": "Description",
     "expense.amount": "Montant",
     "expense.expense_date": "Date de dépense",
     "expense.due_date": "Date d'échéance",
     "expense.category": "Catégorie",
     "expense.category.maintenance": "Entretien",
     "expense.category.repair": "Réparation",
     "expense.category.insurance": "Assurance",
     "expense.category.utilities": "Charges courantes",
     "expense.category.management": "Honoraires syndic",
     "expense.category.other": "Autre",
     "expense.payment_status": "Statut de paiement",
     "expense.status.pending": "En attente",
     "expense.status.paid": "Payé",
     "expense.status.overdue": "En retard",
     "expense.status.cancelled": "Annulé"
   }

Meetings
^^^^^^^^

.. code-block:: json

   {
     "meeting.title": "Assemblées Générales",
     "meeting.create": "Convoquer une AG",
     "meeting.date": "Date de l'assemblée",
     "meeting.agenda": "Ordre du jour",
     "meeting.minutes": "Procès-verbal",
     "meeting.status": "Statut",
     "meeting.status.scheduled": "Planifiée",
     "meeting.status.completed": "Terminée",
     "meeting.status.cancelled": "Annulée"
   }

Documents
^^^^^^^^^

.. code-block:: json

   {
     "document.title": "Documents",
     "document.upload": "Téléverser un document",
     "document.type": "Type de document",
     "document.type.pcn": "PCN (Précompte)",
     "document.type.regulation": "Règlement",
     "document.type.contract": "Contrat",
     "document.type.invoice": "Facture",
     "document.type.other": "Autre",
     "document.file_name": "Nom du fichier",
     "document.upload_date": "Date de téléversement"
   }

Pagination
^^^^^^^^^^

.. code-block:: json

   {
     "pagination.showing": "Affichage",
     "pagination.of": "sur",
     "pagination.results": "résultats",
     "pagination.page": "Page",
     "pagination.per_page": "Par page",
     "pagination.previous": "Précédent",
     "pagination.next": "Suivant",
     "pagination.first": "Premier",
     "pagination.last": "Dernier"
   }

Formulaires
^^^^^^^^^^^

.. code-block:: json

   {
     "form.required": "Champ requis",
     "form.invalid_email": "Email invalide",
     "form.invalid_phone": "Téléphone invalide",
     "form.min_length": "Minimum {min} caractères",
     "form.max_length": "Maximum {max} caractères",
     "form.min_value": "Minimum {min}",
     "form.max_value": "Maximum {max}"
   }

Authentification
^^^^^^^^^^^^^^^^

.. code-block:: json

   {
     "auth.login": "Se connecter",
     "auth.logout": "Se déconnecter",
     "auth.email": "Adresse email",
     "auth.password": "Mot de passe",
     "auth.forgot_password": "Mot de passe oublié ?",
     "auth.welcome_back": "Bienvenue !",
     "auth.invalid_credentials": "Identifiants invalides"
   }

Paramètres Dynamiques
---------------------

**Avec Variables** :

.. code-block:: json

   {
     "welcome.message": "Bienvenue, {name} !",
     "building.units_count": "{count} lot(s)",
     "expense.amount_eur": "{amount} €"
   }

**Utilisation** :

.. code-block:: svelte

   <h1>{$_('welcome.message', { values: { name: user.firstName } })}</h1>
   <p>{$_('building.units_count', { values: { count: building.total_units } })}</p>

**Pluralisation** :

.. code-block:: json

   {
     "building.units_plural": "{count, plural, =0 {aucun lot} one {1 lot} other {# lots}}"
   }

.. code-block:: svelte

   <p>{$_('building.units_plural', { values: { count: totalUnits } })}</p>

Dates et Nombres
----------------

**Format Dates** :

.. code-block:: json

   {
     "date.format.short": "dd/MM/yyyy",
     "date.format.long": "dd MMMM yyyy",
     "date.today": "Aujourd'hui",
     "date.yesterday": "Hier",
     "date.tomorrow": "Demain"
   }

**Utilisation avec svelte-i18n** :

.. code-block:: svelte

   <script>
     import { date, number } from 'svelte-i18n';
   </script>

   <p>{$date(new Date(), { format: 'short' })}</p>
   <p>{$number(1234.56, { style: 'currency', currency: 'EUR' })}</p>

Maintenance Traductions
-----------------------

Workflow
^^^^^^^^

1. **Ajouter clé dans nl.json** (référence)
2. **Traduire dans fr.json, de.json, en.json**
3. **Utiliser dans composant** : ``$_('nouvelle.cle')``
4. **Tester changement de langue**

Outils Recommandés
^^^^^^^^^^^^^^^^^^

**i18n-ally (VS Code)** :

- Extension VS Code
- Visualisation inline
- Détection clés manquantes
- Édition multi-langues

**Script Vérification** :

.. code-block:: bash

   #!/bin/bash
   # scripts/check-i18n.sh

   echo "🔍 Vérification traductions..."

   # Trouver clés en nl.json
   NL_KEYS=$(jq -r 'keys[]' frontend/src/locales/nl.json | sort)

   # Vérifier chaque langue
   for LANG in fr de en; do
     echo "Vérification ${LANG}..."
     LANG_KEYS=$(jq -r 'keys[]' frontend/src/locales/${LANG}.json | sort)

     MISSING=$(comm -23 <(echo "$NL_KEYS") <(echo "$LANG_KEYS"))

     if [ -n "$MISSING" ]; then
       echo "⚠️  Clés manquantes dans ${LANG}.json:"
       echo "$MISSING"
     else
       echo "✅ ${LANG}.json complet"
     fi
   done

**Exécution** :

.. code-block:: bash

   chmod +x scripts/check-i18n.sh
   ./scripts/check-i18n.sh

Exemples Complets
-----------------

nl.json (Néerlandais)
^^^^^^^^^^^^^^^^^^^^^

.. code-block:: json

   {
     "nav.dashboard": "Dashboard",
     "nav.buildings": "Gebouwen",
     "nav.owners": "Mede-eigenaars",
     "nav.expenses": "Kosten",

     "building.title": "Gebouwen",
     "building.create": "Gebouw aanmaken",
     "building.name": "Gebouwnaam",
     "building.address": "Adres",
     "building.total_units": "Aantal kavels",

     "button.save": "Opslaan",
     "button.cancel": "Annuleren",

     "success.created": "Succesvol aangemaakt",
     "error.network": "Netwerkfout"
   }

fr.json (Français)
^^^^^^^^^^^^^^^^^^

.. code-block:: json

   {
     "nav.dashboard": "Tableau de bord",
     "nav.buildings": "Immeubles",
     "nav.owners": "Copropriétaires",
     "nav.expenses": "Charges",

     "building.title": "Immeubles",
     "building.create": "Créer un immeuble",
     "building.name": "Nom de l'immeuble",
     "building.address": "Adresse",
     "building.total_units": "Nombre de lots",

     "button.save": "Enregistrer",
     "button.cancel": "Annuler",

     "success.created": "Créé avec succès",
     "error.network": "Erreur réseau"
   }

de.json (Allemand)
^^^^^^^^^^^^^^^^^^

.. code-block:: json

   {
     "nav.dashboard": "Dashboard",
     "nav.buildings": "Gebäude",
     "nav.owners": "Miteigentümer",
     "nav.expenses": "Kosten",

     "building.title": "Gebäude",
     "building.create": "Gebäude erstellen",
     "building.name": "Gebäudename",
     "building.address": "Adresse",
     "building.total_units": "Anzahl Einheiten",

     "button.save": "Speichern",
     "button.cancel": "Abbrechen",

     "success.created": "Erfolgreich erstellt",
     "error.network": "Netzwerkfehler"
   }

en.json (Anglais)
^^^^^^^^^^^^^^^^^

.. code-block:: json

   {
     "nav.dashboard": "Dashboard",
     "nav.buildings": "Buildings",
     "nav.owners": "Co-owners",
     "nav.expenses": "Expenses",

     "building.title": "Buildings",
     "building.create": "Create building",
     "building.name": "Building name",
     "building.address": "Address",
     "building.total_units": "Number of units",

     "button.save": "Save",
     "button.cancel": "Cancel",

     "success.created": "Successfully created",
     "error.network": "Network error"
   }

Tests i18n
----------

.. code-block:: typescript

   // tests/unit/locales.test.ts
   import { describe, it, expect } from 'vitest';
   import nl from '../src/locales/nl.json';
   import fr from '../src/locales/fr.json';
   import de from '../src/locales/de.json';
   import en from '../src/locales/en.json';

   describe('i18n completeness', () => {
     const nlKeys = Object.keys(nl);

     it('fr.json should have all keys', () => {
       const frKeys = Object.keys(fr);
       expect(frKeys).toEqual(nlKeys);
     });

     it('de.json should have all keys', () => {
       const deKeys = Object.keys(de);
       expect(deKeys).toEqual(nlKeys);
     });

     it('en.json should have all keys', () => {
       const enKeys = Object.keys(en);
       expect(enKeys).toEqual(nlKeys);
     });
   });

Bonnes Pratiques
----------------

1. **nl.json comme Référence** :

   Toujours ajouter clés en néerlandais d'abord.

2. **Traductions Professionnelles** :

   Éviter Google Translate pour textes importants.

3. **Contexte dans Clés** :

   ``button.save`` vs ``form.save`` (contexte clair).

4. **Pas de Code dans Traductions** :

   ❌ ``"text": "Cliquez <a href='...'>ici</a>"``
   ✅ Utiliser composants Svelte

5. **Variables Explicites** :

   ``{name}`` pas ``{0}``

Références
----------

- Configuration i18n : ``frontend/src/lib/i18n.ts``
- LanguageSelector : ``frontend/src/components/LanguageSelector.svelte``
- svelte-i18n Docs : https://github.com/kaisermann/svelte-i18n
