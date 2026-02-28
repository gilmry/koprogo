==================================================
backend/src/domain/entities/document.rs
==================================================

Description et Responsabilités
==================================================

Le fichier ``document.rs`` définit l'entité de domaine **Document** dans le système KoproGo. Cette entité représente les documents de copropriété (proc

ès-verbaux, factures, contrats, règlements, devis, etc.) et leur gestion.

**Responsabilités principales:**

- Représenter un document avec ses métadonnées (type, titre, taille, format)
- Gérer le stockage de fichiers et leur localisation (file_path)
- Lier les documents aux entités métier (meetings, expenses)
- Valider les données lors de la création
- Tracer l'origine des documents (uploaded_by)
- Maintenir les métadonnées temporelles (création, mise à jour)

**Contexte métier:**

Dans une copropriété, de nombreux documents doivent être conservés et partagés avec les copropriétaires : procès-verbaux d'assemblées générales, bilans financiers, factures de travaux, contrats de prestation, règlements de copropriété, devis, etc. Ces documents doivent être organisés, accessibles, et liés aux événements et charges correspondants.

Énumérations
==================================================

DocumentType
--------------------------------------------------

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum DocumentType {
        MeetingMinutes,
        FinancialStatement,
        Invoice,
        Contract,
        Regulation,
        WorksQuote,
        Other,
    }

**Description:**

Énumération représentant les différents types de documents de copropriété.

**Variants:**

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Variant
     - Description
   * - ``MeetingMinutes``
     - Procès-verbal d'assemblée générale (AGO, AGE)
   * - ``FinancialStatement``
     - Bilan financier, comptabilité, états des comptes
   * - ``Invoice``
     - Facture de fournisseurs, prestataires
   * - ``Contract``
     - Contrat de prestation (gardiennage, entretien, assurance)
   * - ``Regulation``
     - Règlement de copropriété, règlement intérieur
   * - ``WorksQuote``
     - Devis pour travaux
   * - ``Other``
     - Autre type de document non catégorisé

**Traits dérivés:**

- ``Debug``: Affichage pour le débogage
- ``Clone``: Copie de la valeur
- ``Serialize/Deserialize``: Sérialisation JSON
- ``PartialEq``: Comparaison d'égalité

Structures et Types
==================================================

Document
--------------------------------------------------

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Document {
        pub id: Uuid,
        pub building_id: Uuid,
        pub document_type: DocumentType,
        pub title: String,
        pub description: Option<String>,
        pub file_path: String,
        pub file_size: i64,
        pub mime_type: String,
        pub uploaded_by: Uuid,
        pub related_meeting_id: Option<Uuid>,
        pub related_expense_id: Option<Uuid>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

**Description:**

Structure représentant un document de copropriété avec toutes ses métadonnées et ses liens vers les entités associées.

**Champs:**

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Champ
     - Type
     - Description
   * - ``id``
     - ``Uuid``
     - Identifiant unique du document (UUID v4)
   * - ``building_id``
     - ``Uuid``
     - Référence vers l'immeuble concerné
   * - ``document_type``
     - ``DocumentType``
     - Type de document (PV, facture, contrat, etc.)
   * - ``title``
     - ``String``
     - Titre du document (obligatoire, non vide)
   * - ``description``
     - ``Option<String>``
     - Description optionnelle du document
   * - ``file_path``
     - ``String``
     - Chemin d'accès au fichier (obligatoire, non vide)
   * - ``file_size``
     - ``i64``
     - Taille du fichier en bytes (doit être > 0)
   * - ``mime_type``
     - ``String``
     - Type MIME du fichier (ex: ``application/pdf``)
   * - ``uploaded_by``
     - ``Uuid``
     - ID de l'utilisateur qui a téléversé le document
   * - ``related_meeting_id``
     - ``Option<Uuid>``
     - Lien optionnel vers une assemblée générale
   * - ``related_expense_id``
     - ``Option<Uuid>``
     - Lien optionnel vers une charge/dépense
   * - ``created_at``
     - ``DateTime<Utc>``
     - Date et heure de création de l'enregistrement
   * - ``updated_at``
     - ``DateTime<Utc>``
     - Date et heure de dernière mise à jour

Méthodes
==================================================

Document::new
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn new(
        building_id: Uuid,
        document_type: DocumentType,
        title: String,
        description: Option<String>,
        file_path: String,
        file_size: i64,
        mime_type: String,
        uploaded_by: Uuid,
    ) -> Result<Self, String>

**Description:**

Constructeur pour créer une nouvelle instance de Document avec validation des données.

**Comportement:**

1. Valide que ``title`` n'est pas vide
2. Valide que ``file_path`` n'est pas vide
3. Valide que ``file_size`` est strictement positif (> 0)
4. Génère un nouvel UUID v4 pour ``id``
5. Initialise ``related_meeting_id`` et ``related_expense_id`` à ``None``
6. Capture le timestamp actuel UTC pour ``created_at`` et ``updated_at``
7. Retourne une instance Document si toutes les validations passent
8. Retourne une erreur descriptive si une validation échoue

**Retour:**

- ``Ok(Document)``: Instance Document valide avec ID généré et timestamps
- ``Err(String)``: Message d'erreur descriptif si validation échoue

**Erreurs possibles:**

- ``"Title cannot be empty"``: Le titre est vide
- ``"File path cannot be empty"``: Le chemin de fichier est vide
- ``"File size must be greater than 0"``: La taille est d 0

Document::link_to_meeting
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn link_to_meeting(&mut self, meeting_id: Uuid)

**Description:**

Lie le document à une assemblée générale et met à jour le timestamp.

Document::link_to_expense
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn link_to_expense(&mut self, expense_id: Uuid)

**Description:**

Lie le document à une charge/dépense et met à jour le timestamp.

Document::file_size_mb
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn file_size_mb(&self) -> f64

**Description:**

Convertit et retourne la taille du fichier en mégaoctets (MB). Divise ``file_size`` (en bytes) par 1 048 576 (1024 × 1024).

Tests
==================================================

Le fichier contient **3 tests unitaires**:

1. ``test_create_document_success``: Création réussie avec données valides
2. ``test_create_document_empty_title_fails``: Validation titre vide
3. ``test_link_document_to_meeting``: Lien vers assemblée

Fichiers Associés
==================================================

.. code-block:: text

    backend/src/domain/entities/document.rs   CE FICHIER
    backend/src/application/dto/document_dto.rs
    backend/src/application/ports/document_repository.rs
    backend/src/infrastructure/repositories/postgres_document_repository.rs
    backend/src/web/handlers/documents.rs
