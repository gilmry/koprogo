# ═══════════════════════════════════════════════════════════════════════
# TRAÇABILITÉ JURIDIQUE BELGE — KoproGo
# ═══════════════════════════════════════════════════════════════════════
#
# Fichier méta-BDD : chaque scénario = 1 exigence légale belge.
# Ce fichier est le POINT CENTRAL de suivi de conformité juridique.
#
# Tags :
#   @conforme  — Exigence implémentée et testée
#   @corrige   — Exigence corrigée suite à l'audit (erreur factuelle)
#   @manquant  — Exigence NON implémentée (bloquant pour production)
#   @partiel   — Exigence partiellement implémentée
#   @wip       — Scénario non exécutable (pas encore de code)
#
# Lien documentaire :
#   - Matrice code↔loi : docs/legal/matrice_conformite.rst
#   - Extraits de loi  : docs/legal/copropriete_art_3_84_3_92.rst
#   - Audit complet    : docs/legal/audit_conformite.rst
#
# Dernière mise à jour : 2026-02-28
# Score conformité : 25/37 CONFORME (67%)
#
# ═══════════════════════════════════════════════════════════════════════

Feature: Conformite Juridique Belge
  Ce fichier centralise TOUTES les exigences legales belges
  applicables a KoproGo et leur statut de conformite.
  Il sert de registre vivant entre le droit belge et le code.

  Background:
    Given the legal compliance system is initialized

  # ══════════════════════════════════════════════════════════════
  # DROIT DE LA COPROPRIÉTÉ — Code Civil belge, Art. 3.84-3.94
  # Texte intégral : docs/legal/copropriete_art_3_84_3_92.rst
  # ══════════════════════════════════════════════════════════════

  # --- Art. 3.84 : Disposition générale ---

  @conforme @copropriete
  Scenario: [Art. 3.84] Quotes-parts actives doivent totaliser 100%
    # Code   : unit_owner.rs, migration 20251120230000 (trigger PostgreSQL)
    # Test   : multitenancy.feature
    # Loi    : "les quotes-parts dans les parties communes sont fixées [...]
    #           en fonction de la valeur respective de chaque partie privative"
    Given a building with 3 units exists
    When ownership shares are assigned to all units
    Then the total of active ownership shares must equal 100%
    And the PostgreSQL trigger must reject totals exceeding 100%

  # --- Art. 3.87 §2 : Convocation — contenu ---

  @conforme @copropriete @ag
  Scenario: [Art. 3.87 §2] Convocation contient ordre du jour
    # Code   : meeting.rs (champ agenda obligatoire)
    # Test   : convocations.feature, meetings.feature
    # Loi    : "La convocation indique [...] l'ordre du jour"
    Given a meeting exists with an agenda
    When a convocation is created for that meeting
    Then the convocation must include the meeting agenda

  @manquant @wip @copropriete @ag @critique
  Scenario: [Art. 3.87 §2] Decisions hors agenda sont nulles
    # Code   : NON IMPLÉMENTÉ
    # Risque : Toute décision sur un point absent de l'ordre du jour est nulle
    # Phase  : Phase 1 critique
    Given a meeting with agenda items "A, B, C"
    And a resolution exists for agenda item "D" which is NOT on the agenda
    When voting is closed on that resolution
    Then the system must reject the resolution as invalid
    And the error must mention "resolution not on agenda"

  # --- Art. 3.87 §3 : Convocation — délai 15 jours ---

  @conforme @corrige @copropriete @ag
  Scenario: [Art. 3.87 §3] Convocation 15 jours pour TOUS types AG
    # Code   : convocation.rs:23-30 (minimum_notice_days retourne 15 pour tous)
    # Test   : convocations.feature:19-32
    # Loi    : "la convocation est communiquée quinze jours au moins
    #           avant la date de l'assemblée" — AUCUNE distinction de type
    # Corrigé: était 8j pour extraordinary/second, maintenant 15j pour tous
    Given the convocation types are Ordinary, Extraordinary, and SecondConvocation
    Then the minimum notice period must be 15 days for ALL types
    And ConvocationType::Ordinary.minimum_notice_days() must return 15
    And ConvocationType::Extraordinary.minimum_notice_days() must return 15
    And ConvocationType::SecondConvocation.minimum_notice_days() must return 15

  # --- Art. 3.87 §4 : Procuration mandataire ---

  @conforme @copropriete @ag
  Scenario: [Art. 3.87 §4] Vote par procuration (mandataire)
    # Code   : vote.rs (proxy_owner_id)
    # Test   : resolutions.feature:66-71
    # Loi    : "Tout copropriétaire peut se faire représenter par un mandataire"
    Given owner "Alice" gives proxy to owner "Bob"
    When "Bob" votes on behalf of "Alice"
    Then the vote must be recorded with proxy_owner_id set to "Bob"
    And the voting power must be Alice's tantiemes

  @manquant @wip @copropriete @ag @critique
  Scenario: [Art. 3.87 §7] Maximum 3 procurations par mandataire
    # Code   : NON IMPLÉMENTÉ
    # Risque : Un mandataire représentant >3 copropriétaires → votes invalides
    # Phase  : Phase 1 critique
    # Loi    : "Nul ne peut accepter plus de trois procurations de vote"
    Given owner "Bob" already holds proxies for 3 other owners
    When owner "Eve" tries to give proxy to "Bob"
    Then the system must reject the proxy
    And the error must mention "maximum 3 proxies"

  @manquant @wip @copropriete @ag @critique
  Scenario: [Art. 3.87 §7] Exception procurations si total < 10% voix
    # Code   : NON IMPLÉMENTÉ
    # Loi    : "sauf si le total des voix dont il dispose [...] ne dépasse pas
    #           10 pour cent du total des voix"
    Given owner "Bob" holds proxies for 4 owners
    And the total voting power of Bob's proxies is less than 10% of all votes
    Then the proxy should be accepted as an exception

  # --- Art. 3.87 §5 : Quorum ---

  @manquant @wip @copropriete @ag @critique
  Scenario: [Art. 3.87 §5] Quorum 50% requis en premiere convocation
    # Code   : NON IMPLÉMENTÉ
    # Risque : Décisions prises sans quorum sont NULLES (contestables 4 mois)
    # Phase  : Phase 1 critique
    # Loi    : "L'assemblée générale ne délibère valablement que si plus de la
    #           moitié des copropriétaires sont présents ou représentés"
    Given a meeting with 10 unit owners (1000 total tantiemes)
    And only 4 owners (400 tantiemes) are present or represented
    When a vote is attempted
    Then the system must block the vote
    And the error must mention "quorum not reached (40% < 50%)"

  @manquant @wip @copropriete @ag @critique
  Scenario: [Art. 3.87 §5] Deuxieme convocation si quorum non atteint
    # Code   : NON IMPLÉMENTÉ
    # Phase  : Phase 1 critique
    # Loi    : "Si ce quorum n'est pas atteint, une deuxième assemblée [...]
    #           pourra délibérer [...] quel que soit le nombre"
    Given a first convocation where quorum was NOT reached
    When the syndic creates a second convocation
    Then the second convocation type must be "SecondConvocation"
    And the second convocation must respect the 15-day notice period
    And no quorum requirement applies to the second convocation

  @manquant @wip @copropriete @ag
  Scenario: [Art. 3.87 §5] Quorum 3/4 pour decisions qualifiees
    # Code   : NON IMPLÉMENTÉ
    # Loi    : Certaines décisions (Art. 3.88) exigent une présence de 3/4
    Given a resolution requiring qualified majority (3/4)
    And fewer than 75% of tantiemes are present or represented
    When a vote is attempted
    Then the system must block the vote for insufficient quorum

  # --- Art. 3.87 §8 + Art. 3.88 : Majorités ---

  @conforme @copropriete @ag
  Scenario: [Art. 3.87 §8] Majorite simple — 50%+1 des votes exprimes
    # Code   : resolution.rs:17 (MajorityType::Simple)
    # Test   : resolutions.feature:75-82
    # Loi    : "Les décisions sont prises à la majorité absolue des voix
    #           des copropriétaires présents ou représentés"
    Given a resolution with MajorityType::Simple
    And 500 tantiemes vote Pour, 400 vote Contre, 100 Abstention
    When voting is closed
    Then the resolution must be Adopted (500 > 400 expressed)

  @conforme @copropriete @ag
  Scenario: [Art. 3.87 §8] Majorite absolue — 50%+1 de TOUS les votes
    # Code   : resolution.rs:18 (MajorityType::Absolute)
    # Test   : resolutions.feature:84-91
    Given a resolution with MajorityType::Absolute
    And total tantiemes in building is 1000
    And 600 tantiemes vote Pour
    When voting is closed
    Then the resolution must be Adopted (600/1000 > 50%)

  @conforme @copropriete @ag
  Scenario: [Art. 3.88 al.1] Majorite 2/3 — travaux extraordinaires
    # Code   : resolution.rs:19 (MajorityType::Qualified(0.667))
    # Test   : resolutions.feature:93-100
    # Loi    : travaux d'amélioration, actes de disposition
    Given a resolution with MajorityType::Qualified(0.667)
    And 700 tantiemes vote Pour out of 1000
    When voting is closed
    Then the resolution must be Adopted (70% > 66.7%)

  @conforme @copropriete @ag
  Scenario: [Art. 3.88 al.2] Majorite 3/4 — jouissance parties communes
    # Code   : resolution.rs (Qualified 0.75)
    # Loi    : modification jouissance, usage, administration parties communes
    Given a resolution with MajorityType::Qualified(0.75)
    Then the threshold 0.75 must be a valid qualified majority

  @conforme @copropriete @ag
  Scenario: [Art. 3.88 al.3] Majorite 4/5 — modification statuts
    # Code   : resolution.rs (Qualified 0.80)
    # Loi    : modification de l'acte de base, des statuts
    Given a resolution with MajorityType::Qualified(0.80)
    Then the threshold 0.80 must be a valid qualified majority

  @conforme @copropriete @ag
  Scenario: [Art. 3.88 al.4] Unanimite — modification quotes-parts
    # Code   : resolution.rs (Qualified 1.0)
    # Loi    : modification de la répartition des quotes-parts
    Given a resolution with MajorityType::Qualified(1.0)
    Then the threshold 1.0 must be a valid qualified majority

  # --- Art. 3.87 §10 : PV distribution ---

  @manquant @wip @copropriete @ag
  Scenario: [Art. 3.87 §10] PV distribue dans les 30 jours
    # Code   : NON IMPLÉMENTÉ
    # Phase  : Phase 2
    # Loi    : "Le procès-verbal est communiqué [...] dans les trente jours"
    Given a meeting took place on "2026-03-15"
    When the minutes (PV) are finalized
    Then the system must calculate a distribution deadline of "2026-04-14"
    And send a reminder if not distributed by that date

  # --- Art. 3.89 : Syndic ---

  @manquant @wip @copropriete
  Scenario: [Art. 3.89] Mandat syndic maximum 3 ans
    # Code   : NON IMPLÉMENTÉ
    # Phase  : Phase 2
    # Loi    : "Le mandat du syndic ne peut excéder trois ans"
    Given a syndic mandate started on "2024-01-01"
    Then the maximum mandate end date must be "2027-01-01"
    And the system must warn 3 months before expiration

  @conforme @copropriete
  Scenario: [Art. 3.89] Information publique du syndic
    # Code   : building.rs (7 champs syndic), public_dto.rs
    # Test   : public_syndic.feature:19-24
    # Loi    : coordonnées du syndic accessibles publiquement
    Given a building with syndic information configured
    When the public syndic endpoint is called with the building slug
    Then the response must include syndic_name, syndic_email, syndic_phone

  # --- Art. 3.90 : Conseil de copropriété ---

  @conforme @copropriete
  Scenario: [Art. 3.90] Conseil de copropriete obligatoire >= 20 lots
    # Code   : board_member.rs
    # Test   : board.feature:25
    # Loi    : "un conseil de copropriété est constitué dans tout immeuble
    #           [...] comportant au moins vingt lots à l'exclusion des caves,
    #           garages et parkings"
    Given a building with 25 units (excluding garages and cellars)
    Then a board of copropriete must be constituted
    And the board must have at least a President

  # --- Art. 3.94 : État daté ---

  @conforme @corrige @copropriete
  Scenario: [Art. 3.94] Etat date — delai 15 jours (demande simple)
    # Code   : etat_date.rs:288-297 (is_overdue, Duration::days(15))
    # Test   : etat_date.feature:73
    # Loi    : "Le syndic transmet les informations [...] dans un délai de
    #           quinze jours ouvrables"
    # Corrigé: était 10 jours, maintenant 15 jours
    Given an etat date requested on "2026-02-01"
    Then the overdue deadline must be "2026-02-16"
    And the is_overdue function must use Duration::days(15)

  @conforme @copropriete
  Scenario: [Art. 3.94] Etat date — 16 sections obligatoires
    # Code   : etat_date.rs (champs + additional_data)
    # Test   : etat_date.feature:35-57
    # Loi    : 16 sections d'information obligatoires pour toute vente
    Given an etat date is being prepared
    Then all 16 mandatory legal sections must be fillable
    And the system must prevent generation without all 16 sections

  # ══════════════════════════════════════════════════════════════
  # COMPTABILITÉ — Arrêté Royal du 12/07/2012 (PCMN)
  # Texte intégral : docs/legal/pcmn_ar_12_07_2012.rst
  # ══════════════════════════════════════════════════════════════

  @conforme @comptabilite
  Scenario: [AR 12/07/2012 Art. 3] Plan comptable PCMN implemente
    # Code   : account.rs, account_use_cases.rs
    # Test   : (seed endpoint /accounts/seed/belgian-pcmn)
    # Loi    : plan comptable minimum normalisé pour associations de copropriétaires
    Given the Belgian PCMN seed is loaded
    Then at least 80 accounting accounts must exist
    And accounts must follow the Belgian class structure (1-7, 9)

  @conforme @comptabilite
  Scenario: [AR 12/07/2012 Art. 2] Comptabilite en partie double
    # Code   : journal_entry.rs
    # Test   : journal_entries.feature:15-26
    # Loi    : "La comptabilité est tenue selon un système de livres et de
    #           comptes conformément à la méthode de la partie double"
    Given a journal entry with debit 1000 and credit 1000
    Then the entry must be accepted (balanced)
    And a journal entry with debit 1000 and credit 500 must be rejected

  @conforme @comptabilite
  Scenario: [AR 12/07/2012 Art. 4] Pieces justificatives datees
    # Code   : document.rs, journal_entry.rs (document_ref)
    # Test   : documents.feature
    # Loi    : "Toute écriture s'appuie sur une pièce justificative datée"
    Given a journal entry exists
    Then it must have a document_ref field linking to a justificative document
    And documents must be uploadable and linked to entries

  @partiel @comptabilite
  Scenario: [AR 12/07/2012 Art. 5] Conservation documents 7 ans
    # Code   : Backups S3 chiffres (infrastructure)
    # Test   : —
    # Loi    : "Les livres et les pièces justificatives sont conservés
    #           pendant sept ans"
    # Statut : backups configurés, mais pas de politique de rétention explicite 7 ans
    Given the backup system is configured
    Then documents must be retained for at least 7 years
    And a retention policy must prevent premature deletion

  @conforme @comptabilite
  Scenario: [AR 12/07/2012 Art. 6] Bilan annuel et compte de resultats
    # Code   : financial_report_use_cases.rs
    # Test   : (endpoints /reports/balance-sheet, /reports/income-statement)
    # Loi    : bilan et compte de résultats annuels obligatoires
    Given a fiscal year with accounting entries
    When the balance sheet report is generated
    Then it must include actif and passif sections
    When the income statement is generated
    Then it must include charges and produits sections

  @conforme @comptabilite
  Scenario: [AR 12/07/2012 Art. 6] Compte de resultats separe
    # Code   : financial_report_use_cases.rs
    # Loi    : compte de résultats distinct du bilan
    Given a fiscal year with income and expenses
    When the income statement is requested
    Then it must show revenue vs expenses and net result

  @conforme @comptabilite
  Scenario: [Legislation TVA] TVA belge 6%, 12%, 21% correctement appliquee
    # Code   : invoice_line_item.rs, quote.rs
    # Test   : invoices.feature:19-30, quotes.feature:41-53
    # Loi    : taux réduit 6% (rénovation >10 ans), intermédiaire 12%, standard 21%
    Given an invoice with VAT rate 21%
    Then the VAT amount must be correctly calculated
    And VAT rates 6%, 12%, 21% must all be supported

  # ══════════════════════════════════════════════════════════════
  # RGPD — Règlement UE 2016/679 + Loi belge du 30/07/2018
  # Documentation : docs/legal/rgpd_conformite.rst
  # Sanctions APD : jusqu'à 20M€ ou 4% CA mondial
  # ══════════════════════════════════════════════════════════════

  @conforme @rgpd
  Scenario: [RGPD Art. 15] Droit d'acces aux donnees personnelles
    # Code   : gdpr_use_cases.rs (export_user_data)
    # Test   : gdpr.feature:8-17
    # Loi    : "La personne concernée a le droit d'obtenir [...] l'accès
    #           aux données à caractère personnel la concernant"
    Given a user with personal data in the system
    When the user requests data export via GET /gdpr/export
    Then the export must include all personal data
    And an audit log GdprDataExported must be created

  @conforme @rgpd
  Scenario: [RGPD Art. 16] Droit de rectification
    # Code   : gdpr_handlers.rs (PUT /gdpr/rectify)
    # Test   : gdpr.feature
    # Loi    : "La personne concernée a le droit d'obtenir [...] la
    #           rectification des données inexactes"
    Given a user wants to correct their email address
    When a rectification request is submitted via PUT /gdpr/rectify
    Then the data must be updated
    And an audit log GdprDataRectified must be created

  @conforme @rgpd
  Scenario: [RGPD Art. 17] Droit a l'effacement
    # Code   : gdpr_use_cases.rs (erase_user_data)
    # Test   : gdpr.feature:38-49
    # Loi    : "La personne concernée a le droit d'obtenir [...] l'effacement"
    Given a user with no legal holds
    When the user requests erasure via DELETE /gdpr/erase
    Then personal data must be anonymized
    And the user account must be deactivated

  @conforme @rgpd
  Scenario: [RGPD Art. 18] Droit a la limitation du traitement
    # Code   : gdpr_handlers.rs (PUT /gdpr/restrict-processing)
    # Test   : gdpr.feature
    # Loi    : "La personne concernée a le droit d'obtenir [...] la limitation
    #           du traitement"
    Given a user requests processing restriction
    When the restriction is applied via PUT /gdpr/restrict-processing
    Then the user's processing_restricted flag must be TRUE
    And the system must not process that user's data for non-essential purposes

  @conforme @rgpd
  Scenario: [RGPD Art. 21] Droit d'opposition au marketing
    # Code   : gdpr_handlers.rs (PUT /gdpr/marketing-preference)
    # Test   : gdpr.feature
    # Loi    : "La personne concernée a le droit de s'opposer [...] au
    #           traitement à des fins de prospection"
    Given a user wants to opt out of marketing
    When the preference is set via PUT /gdpr/marketing-preference
    Then the user's marketing_opt_out flag must be TRUE
    And no marketing communications must be sent to that user

  @conforme @rgpd
  Scenario: [RGPD Art. 30] Registre des activites de traitement
    # Code   : audit.rs, table audit_logs
    # Loi    : "Chaque responsable du traitement tient un registre des
    #           activités de traitement"
    Given operations are performed on personal data
    Then each operation must be logged in audit_logs
    And the log must include user_id, action, ip_address, user_agent

  @manquant @wip @rgpd
  Scenario: [RGPD Art. 13-14] Information des personnes concernees
    # Code   : NON IMPLÉMENTÉ
    # Phase  : Phase 2
    # Loi    : politique de confidentialité, information sur les traitements
    # Risque : amende APD, non-conformité de base
    Given a new user registers on the platform
    Then a privacy policy must be presented before data collection
    And the policy must detail: purposes, legal basis, retention periods, rights

  @manquant @wip @rgpd
  Scenario: [RGPD Art. 28] DPA avec sous-traitants
    # Code   : NON IMPLÉMENTÉ
    # Phase  : Phase 2
    # Loi    : contrat obligatoire avec tout sous-traitant traitant des données
    # Sous-traitants KoproGo : hébergeur VPS, Stripe, email provider
    Given the system uses external processors (hosting, Stripe, email)
    Then a Data Processing Agreement must exist for each processor
    And each DPA must specify: subject, duration, nature, purpose

  @manquant @wip @rgpd @critique
  Scenario: [RGPD Art. 33] Notification violation de donnees sous 72h
    # Code   : NON IMPLÉMENTÉ
    # Phase  : Phase 1
    # Loi    : "le responsable du traitement en notifie la violation [...]
    #           dans les meilleurs délais et, si possible, 72 heures au plus
    #           tard après en avoir pris connaissance"
    # Risque : amende APD significative en cas de violation non notifiée
    Given a data breach is detected
    Then the system must generate an APD notification template
    And the notification must be sendable within 72 hours
    And affected users must be informed if high risk

  @partiel @rgpd
  Scenario: [RGPD Art. 32] Securite du traitement
    # Code   : LUKS (chiffrement disque), bcrypt (mots de passe), HSTS, CSP
    # Test   : (infrastructure, pas BDD)
    # Loi    : "mesures techniques et organisationnelles appropriées"
    # Statut : infrastructure solide, mais pas de pentest formel
    Given the application infrastructure
    Then data at rest must be encrypted (LUKS AES-XTS-512)
    And passwords must be hashed with bcrypt
    And HTTPS must be enforced with HSTS
    And a formal penetration test should be scheduled

  # ══════════════════════════════════════════════════════════════
  # TAUX D'INTÉRÊT LÉGAL — Moniteur belge (AR annuel)
  # ══════════════════════════════════════════════════════════════

  @conforme @corrige @financier
  Scenario: [Moniteur belge] Taux d'interet legal civil 2026 = 4.5%
    # Code   : payment_reminder.rs:90 (BELGIAN_PENALTY_RATE = 0.045)
    # Test   : payment_recovery.feature
    # Source : Arrêté Royal publié au Moniteur belge
    # Corrigé: était 8% (erroné), maintenant 4.5% (taux officiel 2026)
    # Note  : Ce taux change chaque année par AR — doit être mis à jour
    #         Historique: 2024=5.25%, 2025=4.0%, 2026=4.5%
    Given the Belgian penalty rate constant in payment_reminder.rs
    Then BELGIAN_PENALTY_RATE must equal 0.045
    And a penalty on 1000 EUR for 365 days must equal 45 EUR
    And a penalty on 100 EUR for 30 days must equal approximately 0.37 EUR

  # ══════════════════════════════════════════════════════════════
  # BONNES PRATIQUES PROFESSIONNELLES (non légalement obligatoires)
  # ══════════════════════════════════════════════════════════════

  @conforme @corrige @pratique
  Scenario: [Pratique pro] Regle des 3 devis > 5000 EUR
    # Code   : quote_use_cases.rs:138-145
    # Test   : quotes.feature:78-97
    # ATTENTION : Ce n'est PAS une obligation légale belge.
    # Corrigé: la terminologie "Belgian legal requirement" a été changée
    #          en "Belgian professional best practice" dans ~15 fichiers.
    # Aucun article du Code Civil n'impose cette règle.
    Given the quote comparison system
    Then the system must recommend 3 quotes for works > 5000 EUR
    But the terminology must use "best practice" not "legal requirement"
    And the system must NOT block operations with fewer than 3 quotes
