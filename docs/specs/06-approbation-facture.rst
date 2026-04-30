====================================================
Workflow 06 : Approbation d'une facture/charge
====================================================

:Issue: #346
:Personas: Voir 00-personas-et-seed.rst
:Acteurs: Gisele Vandenberghe (comptable), Francois Leroy (syndic), Alice Dubois + Diane Peeters (CdC approuvent), Nadia Benali + Jeanne Devos (impact financier)
:Articles CC: Art. 3.86 par.3 (charges communes), Art. 3.89 par.5 (obligations syndic)
:Priorite: Haute

Resume
------

Le workflow d'approbation des factures garantit un controle multi-acteur avant
que les charges ne soient reparties entre les coproprietaires. Le processus
suit une machine a etats stricte :

- **Draft** : La facture est en cours de saisie. Modifiable librement.
- **PendingApproval** : Soumise pour validation. Non modifiable.
- **Approved** : Validee. Peut etre payee et distribuee aux coproprietaires.
- **Rejected** : Rejetee avec motif obligatoire. Peut etre re-soumise.

Transitions autorisees :

- Draft -> PendingApproval (soumission)
- PendingApproval -> Approved (approbation)
- PendingApproval -> Rejected (rejet)
- Rejected -> PendingApproval (re-soumission apres correction)

La TVA belge est geree avec 3 taux principaux : 6% (renovations), 12%
(logement social), 21% (taux standard). Les calculs sont automatiques a
partir du montant HT et du taux.

Apres approbation, les charges sont distribuees aux coproprietaires au
prorata de leurs quotes-parts (tantiemes) via le module ChargeDistribution.
Le paiement ne peut intervenir qu'apres approbation.

Pour les charges de categorie "Works", un rapport de prestataire valide
(ContractorReport) est obligatoire avant l'approbation (Issue #309).

Dimension humaine — L'impact financier asymetrique
----------------------------------------------------

Cette facture de 54.450 EUR TTC illustre la violence silencieuse des charges
de copropriete. Derriere les pourcentages, il y a des vies :

.. list-table:: Impact financier par persona (facture toiture 54.450 EUR TTC)
   :header-rows: 1
   :widths: 20 10 15 55

   * - Persona
     - Tantiemes
     - Part a payer
     - Impact humain
   * - **Philippe Vandermeulen**
     - 1800 (18%)
     - 9.801 EUR
     - Deductible fiscalement (charges locatives). Minime par rapport a ses revenus.
   * - **Emmanuel Claes**
     - 1280 (12.8%)
     - 6.970 EUR
     - Integre dans son calcul de rendement. Un simple ajustement comptable.
   * - **Charlie Martin**
     - 660 (6.6%)
     - 3.594 EUR
     - Charges = 48% du revenu. Ce montant represente presque 2 mois de budget courses.
   * - **Diane Peeters**
     - 580 (5.8%)
     - 3.158 EUR
     - Avocate avec de bons revenus. Gerera sans probleme.
   * - **Alice Dubois**
     - 450 (4.5%)
     - 2.450 EUR
     - Retraitee avec pension correcte. Puisera dans son epargne.
   * - **Marcel Dupont**
     - 450 (4.5%)
     - 2.450 EUR
     - Retraite. C'est lui qui pousse pour ces travaux — assume le cout.
   * - **Bob Janssen**
     - 430 (4.3%)
     - 2.341 EUR
     - Comptable independant. Budget serre mais gerable.
   * - **Marguerite Lemaire**
     - 380 (3.8%)
     - 2.069 EUR
     - Pension de survie 1.200 EUR/mois. Ce montant = **1.7 mois de pension**.
       Livret de 12.000 EUR qui fond. Un echelonnement est vital.
   * - **Nadia Benali**
     - 320 (3.2%)
     - 1.742 EUR
     - Credit a taux variable 4.2%. Charges = dernier poste avant le decouvert.
       Un appel de fonds supplementaire pourrait la mettre en defaut.
   * - **Jeanne Devos**
     - 290 (2.9%)
     - 1.579 EUR
     - Pension minimum 1.050 EUR/mois. AUCUNE tresorerie. Ce montant = **1.5 mois
       de pension**. Son fils ne peut pas aider. Elle reduira ses repas.

**C'est pourquoi Alice et Diane (CdC) approuvent sous condition d'echelonnement
sur 6 mois** : pour Marguerite, cela fait 345 EUR/mois au lieu de 2.069 EUR
d'un coup. Pour Jeanne, 263 EUR/mois au lieu de 1.579 EUR.

Pre-conditions legales
-----------------------

1. **Charges communes** (Art. 3.86 par.3) : Les coproprietaires paient les
   charges communes proportionnellement a leurs quotes-parts, selon les
   decisions de l'AG. Le syndic est responsable de la gestion financiere.

2. **Obligations du syndic** (Art. 3.89 par.5) : Le syndic doit tenir une
   comptabilite claire et complete, accessible aux coproprietaires. Chaque
   charge doit etre justifiee par une piece comptable.

3. **PCMN obligatoire** : Le Plan Comptable Minimum Normalise belge (AR
   12/07/2012) est obligatoire pour les coproprietes. Chaque facture doit
   etre liee a un compte comptable (``account_code``).

4. **TVA belge** : Les taux applicables sont 6% (travaux renovation sur
   immeubles > 10 ans), 12% (logement social) et 21% (taux standard pour
   nouvelles constructions et services).

5. **Rapport prestataire requis pour travaux** : Les charges de categorie
   ``Works`` necessitent un ``ContractorReport`` valide avant approbation
   (chaine de validation : devis -> travaux -> rapport -> facture).

6. **Quotes-parts totales = 100%** : La distribution des charges repose
   sur les quotes-parts actives des coproprietaires, dont le total doit
   etre egal a 100% (Art. 577-2 par.4, tolerance +-0.01%).

Etapes
------

1. **Gisele Vandenberghe (comptable externe)** — Saisit la facture (brouillon)

   - Appel : ``POST /api/v1/expenses``
   - Payload :

     .. code-block:: json

        {
          "building_id": "<residence-du-parc-royal-uuid>",
          "category": "Works",
          "description": "Refection toiture batiment principal — Toitures Bruxelles (Hassan El Amrani)",
          "amount_excl_vat": 45000.00,
          "vat_rate": 21.0,
          "invoice_date": "2026-03-15T00:00:00Z",
          "due_date": "2026-04-15T00:00:00Z",
          "supplier": "Toitures Bruxelles (Hassan El Amrani)",
          "invoice_number": "TB-2026-087",
          "account_code": "611001"
        }

   - Resultat : Expense cree en statut **Draft**

     * Calcul automatique : ``vat_amount = 9.450,00``, ``amount_incl_vat = 54.450,00``
     * ``payment_status = Pending``
     * ``approval_status = Draft``

   - Gisele verifie les montants contre le devis original d'Hassan

2. **Francois Leroy (syndic)** — Soumet pour approbation au CdC

   - Appel : ``PUT /api/v1/expenses/{id}/submit-for-approval``
   - Pre-condition : Statut doit etre **Draft** ou **Rejected**
   - Resultat : Statut passe a **PendingApproval**, ``submitted_at`` enregistre
   - Francois envoie une notification aux membres du CdC (Alice, Diane)

3. **Alice Dubois (presidente CdC) + Diane Peeters (membre CdC)** — Approuvent sous condition

   - Alice et Diane examinent la facture. Diane verifie la conformite
     juridique du devis (3 devis obligatoires pour > 5.000 EUR).
   - Alice calcule l'impact sur les coproprietaires vulnerables :
     Marguerite (2.069 EUR = 1.7 mois de pension), Jeanne (1.579 EUR = 1.5 mois).
   - Decision : **Approbation sous condition d'echelonnement sur 6 mois**

   - Appel : ``PUT /api/v1/expenses/{id}/approve``
   - Pre-condition : Statut **PendingApproval**, categorie Works
     → ``contractor_report_id`` doit etre lie (rapport d'Hassan valide)
   - Resultat :

     * Statut passe a **Approved**
     * ``approved_by = alice_user_id``
     * ``approved_at`` enregistre
     * Note : echelonnement sur 6 mois

4. **Gisele** — Distribue les charges aux coproprietaires

   - Appel : ``POST /api/v1/invoices/{expense_id}/calculate-distribution``
   - Pre-condition : Facture approuvee
   - Resultat : Distribution selon les tantiemes de chaque coproprietaire

     .. list-table::
        :header-rows: 1
        :widths: 30 15 20

        * - Coproprietaire
          - Tantiemes
          - Montant du
        * - Philippe Vandermeulen
          - 1800 (18.0%)
          - 9.801,00 EUR
        * - Emmanuel Claes
          - 1280 (12.8%)
          - 6.969,60 EUR
        * - Charlie Martin
          - 660 (6.6%)
          - 3.593,70 EUR
        * - Diane Peeters
          - 580 (5.8%)
          - 3.158,10 EUR
        * - Alice Dubois
          - 450 (4.5%)
          - 2.450,25 EUR
        * - Marcel Dupont
          - 450 (4.5%)
          - 2.450,25 EUR
        * - Bob Janssen
          - 430 (4.3%)
          - 2.341,35 EUR
        * - Marguerite Lemaire
          - 380 (3.8%)
          - 2.069,10 EUR
        * - Nadia Benali
          - 320 (3.2%)
          - 1.742,40 EUR
        * - Jeanne Devos
          - 290 (2.9%)
          - 1.579,05 EUR
        * - Autres (172 lots)
          - 3360 (33.6%)
          - 18.295,20 EUR

5. **Francois** — Enregistre le paiement (apres echelonnement)

   - Appel : ``PUT /api/v1/expenses/{id}/mark-paid``
   - Pre-condition : Facture **Approved**
   - Resultat : ``payment_status = Paid``, ``paid_date`` enregistre

**Variante : Rejet par Diane (non-conformite juridique)**

3bis. **Diane** — Rejette la facture

   - Diane constate que seuls 2 devis ont ete presentes au lieu des 3
     obligatoires (travaux > 5.000 EUR, Art. 3.86).
   - Appel : ``PUT /api/v1/expenses/{id}/reject``
   - Payload : ``{ "reason": "Seulement 2 devis presentes. La loi exige 3 devis comparatifs pour des travaux superieurs a 5.000 EUR." }``
   - Resultat : ``rejection_reason`` enregistre
   - Francois devra obtenir un 3e devis avant de re-soumettre

**Variante : Facture de travaux (Works) — Chaine de validation**

- Avant l'etape 3, le rapport de travaux d'Hassan (soumis via magic link)
  doit etre valide par le CdC
- ``PUT /api/v1/expenses/{id}`` avec ``contractor_report_id``
- Le rapport doit etre en statut ``Validated``
- Sans ce lien, l'approbation est bloquee

Post-conditions
---------------

1. **Expense en base** : ``approval_status = Approved``,
   ``payment_status = Paid``, ``paid_date`` non null, ``approved_by`` (Alice)
   et ``approved_at`` renseignes.

2. **TVA calculee correctement** :

   - Montant HT : 45.000,00 EUR
   - TVA (21%) : 9.450,00 EUR
   - Montant TTC : 54.450,00 EUR

3. **Charge distributions creees** : Un ``ChargeDistribution`` par lot/
   proprietaire actif, avec ``quota_percentage`` et ``amount_due`` calcules
   au prorata des tantiemes. Total distributions = 54.450,00 EUR.

4. **Comptabilite PCMN** : La facture est liee au compte ``611001``
   (travaux toiture) pour le reporting comptable.

5. **Immutabilite apres approbation** : ``can_be_modified()`` retourne
   ``false`` pour les factures en statut ``PendingApproval`` ou ``Approved``.

6. **Audit trail** : Evenements ``ExpenseCreated`` (Gisele),
   ``ExpenseSubmitted`` (Francois), ``ExpenseApproved`` (Alice/Diane),
   ``ChargeDistributionCalculated``, ``ExpensePaid`` (Francois) emis.

7. **Rappels de paiement** : Si la facture devient overdue (``due_date``
   depassee), le systeme de relances automatisees (4 niveaux : Gentle ->
   Formal -> FinalNotice -> LegalAction) peut etre declenche. Pour Nadia
   et Jeanne, cela pourrait escalader rapidement si l'echelonnement n'est
   pas mis en place.

8. **Impact social** : L'echelonnement sur 6 mois reduit la charge
   mensuelle pour Marguerite (345 EUR/mois au lieu de 2.069 EUR) et
   Jeanne (263 EUR/mois au lieu de 1.579 EUR).

Donnees seed requises
----------------------

.. note::

   Ce workflow utilise le seed partage defini dans ``00-personas-et-seed.rst``.
   Building : **Residence du Parc Royal** (182 lots, 10000 tantiemes).

Donnees specifiques au workflow :

.. code-block:: sql

   -- Comptes PCMN specifiques toiture
   INSERT INTO accounts (id, organization_id, code, name, account_type, is_active)
   VALUES (gen_random_uuid(), 'org00000-0000-0000-0000-000000000001', '611001', 'Travaux toiture', 'Charge', true);

   -- Expense Draft (facture a approuver)
   INSERT INTO expenses (id, organization_id, building_id, category, description, amount,
                         amount_excl_vat, vat_rate, vat_amount, amount_incl_vat,
                         expense_date, invoice_date, due_date,
                         approval_status, payment_status, supplier, invoice_number, account_code)
   VALUES ('x0600000-0000-0000-0000-000000000001',
           'org00000-0000-0000-0000-000000000001',
           '<residence-du-parc-royal-uuid>',
           'Works',
           'Refection toiture batiment principal — Toitures Bruxelles (Hassan El Amrani)',
           54450.00, 45000.00, 21.0, 9450.00, 54450.00,
           '2026-03-15', '2026-03-15', '2026-04-15',
           'Draft', 'Pending',
           'Toitures Bruxelles (Hassan El Amrani)', 'TB-2026-087', '611001');

Scenario BDD (Gherkin)
-----------------------

.. code-block:: gherkin

   Feature: Approbation des factures/charges

     Background:
       Given l'immeuble "Residence du Parc Royal" avec 182 lots et 10000 tantiemes
       And la comptable "Gisele Vandenberghe"
       And le syndic "Francois Leroy"
       And le CdC compose d'Alice Dubois (presidente) et Diane Peeters (membre)
       And le prestataire "Hassan El Amrani" (Toitures Bruxelles)

     Scenario: Workflow complet - Facture toiture 45.000 EUR HTVA
       When Gisele cree une facture "Refection toiture" pour Toitures Bruxelles
       With montant HT 45000.00 EUR, TVA 21%, fournisseur "Toitures Bruxelles"
       And code PCMN "611001"
       Then la facture est en statut "Draft"
       And le montant TTC est 54450.00 EUR (45000 + 9450 TVA)

       When Francois soumet la facture pour approbation
       Then le statut passe a "PendingApproval"
       And submitted_at est enregistre

       When Alice et Diane approuvent la facture (CdC)
       Then le statut passe a "Approved"
       And approved_by = alice_user_id
       And approved_at est enregistre

       When Gisele distribue les charges aux coproprietaires
       Then les ChargeDistribution sont creees selon les tantiemes :
       And Philippe Vandermeulen (18.0%) doit 9801.00 EUR
       And Emmanuel Claes (12.8%) doit 6969.60 EUR
       And Charlie Martin (6.6%) doit 3593.70 EUR
       And Nadia Benali (3.2%) doit 1742.40 EUR
       And Jeanne Devos (2.9%) doit 1579.05 EUR
       And Marguerite Lemaire (3.8%) doit 2069.10 EUR

       When Francois enregistre le paiement
       Then payment_status = "Paid"
       And paid_date est enregistre

     Scenario: Impact financier asymetrique — les personas vulnerables
       Given une facture approuvee de 54450.00 EUR TTC
       When les charges sont distribuees
       Then Nadia (320‱, 3.2%) doit 1742.40 EUR
       And Jeanne (290‱, 2.9%) doit 1579.05 EUR
       And Marguerite (380‱, 3.8%) doit 2069.10 EUR
       # Pour Jeanne : 1579 EUR = 1.5 mois de pension minimum (1050 EUR/mois)
       # Pour Marguerite : 2069 EUR = 1.7 mois de pension de survie (1200 EUR/mois)
       # Un echelonnement sur 6 mois est vital pour elles

     Scenario: Rejet par Diane — non-conformite juridique (2 devis au lieu de 3)
       Given une facture Works en statut "PendingApproval"
       And seulement 2 devis ont ete presentes
       When Diane rejette avec la raison "Seulement 2 devis. Loi exige 3 devis pour > 5000 EUR"
       Then le statut passe a "Rejected"
       And rejection_reason = "Seulement 2 devis. Loi exige 3 devis pour > 5000 EUR"

       When Francois obtient un 3e devis et Gisele re-soumet la facture
       Then le statut passe a "PendingApproval"
       And rejection_reason est efface

     Scenario: Rejet sans motif interdit
       Given une facture en statut "PendingApproval"
       When Francois tente de rejeter sans raison
       Then l'erreur "Rejection reason cannot be empty" est retournee

     Scenario: Paiement impossible sans approbation
       Given une facture en statut "Draft"
       When Francois tente de marquer la facture comme payee
       Then l'erreur "invoice must be approved first" est retournee

     Scenario: Facture non modifiable apres soumission
       Given une facture en statut "PendingApproval"
       Then can_be_modified() retourne false

     Scenario: TVA belge a 6% (renovation immeuble > 10 ans — Residence 1965)
       When Gisele cree une facture avec TVA 6% et montant HT 5000.00 EUR
       Then vat_amount = 300.00 EUR
       And amount_incl_vat = 5300.00 EUR
       # Le taux de 6% s'applique car l'immeuble date de 1965 (> 10 ans)

     Scenario: Facture travaux sans rapport prestataire
       Given une facture Works en statut "PendingApproval"
       And aucun contractor_report_id n'est lie
       When Alice tente d'approuver
       Then l'erreur "Work expenses require a validated contractor report" est retournee

     Scenario: Facture travaux avec rapport d'Hassan valide
       Given une facture Works en statut "PendingApproval"
       And le rapport d'Hassan (Toitures Bruxelles) est en statut "Validated"
       When Alice approuve la facture
       Then le statut passe a "Approved"

     Scenario: Recalcul TVA apres modification par Gisele
       Given une facture Draft avec montant HT 45000.00 et TVA 21%
       When Gisele modifie le montant HT a 48000.00 EUR et recalculate_vat() est appele
       Then vat_amount = 10080.00 EUR
       And amount_incl_vat = 58080.00 EUR

Scenario E2E (narratif)
------------------------

**Acteurs** : Gisele Vandenberghe (Comptable), Francois Leroy (Syndic), Alice Dubois (Presidente CdC), Diane Peeters (Membre CdC), Hassan El Amrani (Prestataire — via rapport), Nadia Benali / Jeanne Devos / Marguerite Lemaire (Coproprietaires impactes)

1. Gisele se connecte avec le role ``Accountant`` et saisit la facture
   de Toitures Bruxelles (Hassan El Amrani) pour la refection de la
   toiture via ``POST /expenses``. Elle renseigne le fournisseur, le
   montant HT (45.000 EUR), le taux TVA (21%), la date d'echeance, le
   numero de facture TB-2026-087 et le code PCMN ``611001``. L'API
   retourne 201 avec la facture en statut ``Draft`` et les montants TVA
   calcules automatiquement (TTC = 54.450 EUR).

2. Gisele verifie les montants contre le devis original d'Hassan puis
   Francois soumet la facture pour approbation via
   ``PUT /expenses/{id}/submit-for-approval``. Le statut passe a
   ``PendingApproval``. La facture n'est plus modifiable.

3. Alice (presidente CdC) et Diane (membre CdC, avocate) examinent la
   facture. Diane verifie que 3 devis comparatifs ont ete obtenus
   (obligation legale pour travaux > 5.000 EUR). Alice calcule l'impact
   sur les coproprietaires vulnerables — Marguerite (2.069 EUR = 1.7 mois
   de pension) et Jeanne (1.579 EUR = 1.5 mois de pension). Le CdC
   approuve via ``PUT /expenses/{id}/approve`` avec la condition d'un
   echelonnement sur 6 mois. Le statut passe a ``Approved``.

4. Gisele declenche la distribution des charges via
   ``POST /invoices/{expense_id}/calculate-distribution``. Le systeme
   calcule les montants au prorata des tantiemes. Philippe (18%) doit
   9.801 EUR, Emmanuel (12.8%) doit 6.970 EUR, Nadia (3.2%) doit
   1.742 EUR, Jeanne (2.9%) doit 1.579 EUR.

5. Les coproprietaires consultent leurs charges via
   ``GET /owners/{id}/distributions``. Nadia voit 1.742 EUR — elle
   contacte Francois pour confirmer l'echelonnement. Jeanne voit
   1.579 EUR — son fils l'aide a comprendre.

6. Le montant total du des coproprietaires est accessible via
   ``GET /owners/{id}/total-due``. Marguerite decouvre que 2.069 EUR
   s'ajoutent a ses charges courantes.

7. Apres echelonnement et reception des paiements, Francois enregistre
   le paiement via ``PUT /expenses/{id}/mark-paid``.

8. **Variante rejet** : Si Diane rejette la facture avec
   ``PUT /expenses/{id}/reject`` et la raison "Seulement 2 devis
   presentes", Francois doit obtenir un 3e devis avant re-soumission.
   Gisele corrige et re-soumet. Le cycle reprend a l'etape 2.

9. **Variante travaux** : Le rapport d'Hassan (soumis via magic link 72h)
   doit etre valide par le CdC avant l'approbation. Sans ce lien,
   l'approbation est bloquee avec le message "Work expenses require a
   validated contractor report before approval".

10. **Scenario de risque** : Si l'echelonnement n'est pas mis en place,
    les relances automatisees (Gentle J+15 -> Formal J+30 -> FinalNotice
    J+45 -> LegalAction J+60) pourraient s'activer pour Nadia et Jeanne,
    avec des consequences sociales graves. L'echelonnement est un filet
    de securite indispensable.
