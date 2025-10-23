==================================================
backend/src/domain/entities/document.rs
==================================================

Description et Responsabilit�s
==================================================

Le fichier ``document.rs`` d�finit l'entit� de domaine **Document** dans le syst�me KoproGo. Cette entit� repr�sente les documents de copropri�t� (proc

�s-verbaux, factures, contrats, r�glements, devis, etc.) et leur gestion.

**Responsabilit�s principales:**

- Repr�senter un document avec ses m�tadonn�es (type, titre, taille, format)
- G�rer le stockage de fichiers et leur localisation (file_path)
- Lier les documents aux entit�s m�tier (meetings, expenses)
- Valider les donn�es lors de la cr�ation
- Tracer l'origine des documents (uploaded_by)
- Maintenir les m�tadonn�es temporelles (cr�ation, mise � jour)

**Contexte m�tier:**

Dans une copropri�t�, de nombreux documents doivent �tre conserv�s et partag�s avec les copropri�taires : proc�s-verbaux d'assembl�es g�n�rales, bilans financiers, factures de travaux, contrats de prestation, r�glements de copropri�t�, devis, etc. Ces documents doivent �tre organis�s, accessibles, et li�s aux �v�nements et charges correspondants.

�num�rations
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

�num�ration repr�sentant les diff�rents types de documents de copropri�t�.

**Variants:**

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Variant
     - Description
   * - ``MeetingMinutes``
     - Proc�s-verbal d'assembl�e g�n�rale (AGO, AGE)
   * - ``FinancialStatement``
     - Bilan financier, comptabilit�, �tats des comptes
   * - ``Invoice``
     - Facture de fournisseurs, prestataires
   * - ``Contract``
     - Contrat de prestation (gardiennage, entretien, assurance)
   * - ``Regulation``
     - R�glement de copropri�t�, r�glement int�rieur
   * - ``WorksQuote``
     - Devis pour travaux
   * - ``Other``
     - Autre type de document non cat�goris�

**Traits d�riv�s:**

- ``Debug``: Affichage pour le d�bogage
- ``Clone``: Copie de la valeur
- ``Serialize/Deserialize``: S�rialisation JSON
- ``PartialEq``: Comparaison d'�galit�

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

Structure repr�sentant un document de copropri�t� avec toutes ses m�tadonn�es et ses liens vers les entit�s associ�es.

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
     - R�f�rence vers l'immeuble concern�
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
     - Chemin d'acc�s au fichier (obligatoire, non vide)
   * - ``file_size``
     - ``i64``
     - Taille du fichier en bytes (doit �tre > 0)
   * - ``mime_type``
     - ``String``
     - Type MIME du fichier (ex: ``application/pdf``)
   * - ``uploaded_by``
     - ``Uuid``
     - ID de l'utilisateur qui a t�l�vers� le document
   * - ``related_meeting_id``
     - ``Option<Uuid>``
     - Lien optionnel vers une assembl�e g�n�rale
   * - ``related_expense_id``
     - ``Option<Uuid>``
     - Lien optionnel vers une charge/d�pense
   * - ``created_at``
     - ``DateTime<Utc>``
     - Date et heure de cr�ation de l'enregistrement
   * - ``updated_at``
     - ``DateTime<Utc>``
     - Date et heure de derni�re mise � jour

M�thodes
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

Constructeur pour cr�er une nouvelle instance de Document avec validation des donn�es.

**Comportement:**

1. Valide que ``title`` n'est pas vide
2. Valide que ``file_path`` n'est pas vide
3. Valide que ``file_size`` est strictement positif (> 0)
4. G�n�re un nouvel UUID v4 pour ``id``
5. Initialise ``related_meeting_id`` et ``related_expense_id`` � ``None``
6. Capture le timestamp actuel UTC pour ``created_at`` et ``updated_at``
7. Retourne une instance Document si toutes les validations passent
8. Retourne une erreur descriptive si une validation �choue

**Retour:**

- ``Ok(Document)``: Instance Document valide avec ID g�n�r� et timestamps
- ``Err(String)``: Message d'erreur descriptif si validation �choue

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

Lie le document � une assembl�e g�n�rale et met � jour le timestamp.

Document::link_to_expense
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn link_to_expense(&mut self, expense_id: Uuid)

**Description:**

Lie le document � une charge/d�pense et met � jour le timestamp.

Document::file_size_mb
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn file_size_mb(&self) -> f64

**Description:**

Convertit et retourne la taille du fichier en m�gaoctets (MB). Divise ``file_size`` (en bytes) par 1 048 576 (1024 � 1024).

Tests
==================================================

Le fichier contient **3 tests unitaires**:

1. ``test_create_document_success``: Cr�ation r�ussie avec donn�es valides
2. ``test_create_document_empty_title_fails``: Validation titre vide
3. ``test_link_document_to_meeting``: Lien vers assembl�e

Fichiers Associ�s
==================================================

.. code-block:: text

    backend/src/domain/entities/document.rs  � CE FICHIER
    backend/src/application/dto/document_dto.rs
    backend/src/application/ports/document_repository.rs
    backend/src/infrastructure/repositories/postgres_document_repository.rs
    backend/src/web/handlers/documents.rs
