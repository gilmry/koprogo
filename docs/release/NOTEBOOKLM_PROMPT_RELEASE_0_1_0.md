# Prompts Release 0.1.0 KoproGo

Tous les prompts pour générer du contenu autour de la release 0.1.0.

---

## 1. Podcast Audio — Présentation Grand Public (FR)

Crée un audio overview en français belge. Deux présentateurs discutent de cette première release comme s'ils en parlaient à un ami syndic de copropriété qui cherche un logiciel moderne. Durée 8-12 minutes.

Points à couvrir dans cet ordre :

1. KoproGo c'est quoi : une plateforme open-source pour gérer sa copropriété selon le droit belge, développée en Rust avec 110 000 lignes de code, 511 points d'entrée API et 81 pages web.

2. Ce que la version 0.1.0 apporte concrètement pour un syndic belge : la comptabilité PCMN conforme à l'arrêté royal, les factures avec TVA belge, les relances automatiques d'impayés en 4 étapes, les votes en assemblée générale avec les trois types de majorité et les tantièmes, les convocations automatiques avec tracking email, la visioconférence pour les AG avec calcul du quorum combiné présentiel et distanciel, les demandes d'assemblée extraordinaire par les copropriétaires quand ils atteignent un cinquième des quotes-parts, le paiement par Stripe et SEPA, la maintenance avec tickets et devis entrepreneurs, et un système d'échange de services entre voisins.

3. Pourquoi on peut faire confiance à cette release : cinq niveaux de tests automatisés dont des scénarios métier en langage naturel et des tests dans un vrai navigateur dont les vidéos servent de documentation, un contrat automatique entre le serveur et le client web qui empêche toute divergence silencieuse, une revue humaine obligatoire avant chaque release, et un workflow où chaque commit est vérifié par des hooks et chaque push doit passer toute la pipeline.

4. La vision : les logiciels de copropriété existants en Belgique sont propriétaires et souvent approximatifs sur la loi. KoproGo est open-source, rigoureux et livre quand c'est prêt. La suite : automatisations, mobile, puis intelligence artificielle et IoT.

---

## 2. Deep Dive Technique (FR)

Fais un deep dive technique en français. Deux experts discutent de l'architecture et des choix d'ingénierie derrière KoproGo, une plateforme open-source de gestion de copropriété belge. Le public cible est technique : développeurs Rust, architectes logiciels, CTO de startups PropTech.

Sujets à creuser :

1. Pourquoi Rust et l'architecture hexagonale pour du SaaS immobilier : les ports et adapters, le Domain-Driven Design avec 57 agrégats, la séparation domaine pur sans dépendance externe, les 511 endpoints REST derrière Actix-web, les performances visées à moins de 5 millisecondes au P99.

2. La modélisation du droit belge de la copropriété dans le code : comment les Articles 3.84 à 3.94 du Code Civil deviennent des entités Rust avec des invariants vérifiés à la compilation, les machines à états pour les workflows légaux comme les convocations avec délais de 15 jours, le quorum à 50 pourcent plus un, la limite de 3 procurations avec exception des 10 pourcent, le Plan Comptable Minimum Normalisé avec 90 comptes pré-seedés.

3. La stratégie de test à cinq niveaux et pourquoi chaque niveau existe : les tests unitaires domaine pour la logique pure, le BDD Gherkin comme spécification exécutable des règles métier, les tests E2E backend comme contrat API, les tests Playwright comme preuve visuelle avec vidéos intégrées à la documentation, et les contract tests DTO avec OpenAPI auto-généré pour garantir la cohérence entre le serveur Rust et le client TypeScript.

4. Les défis techniques rencontrés : le multi-tenancy par organisation, le calcul de quorum combiné présentiel et distanciel pour les AG en visioconférence, la gestion des tantièmes et millièmes pour les votes avec trois types de majorité, le système d'échange local SEL avec monnaie temps et balance négative autorisée, l'intégration Stripe et SEPA pour le contexte belge.

---

## 3. Deep Dive Technique (EN)

Do a deep dive in English. Two software engineering experts discuss the architecture and engineering decisions behind KoproGo, an open-source Belgian condominium management SaaS. Target audience is technical: Rust developers, software architects, PropTech CTOs.

Topics to explore:

1. Why Rust and hexagonal architecture for real estate SaaS: ports and adapters pattern, Domain-Driven Design with 57 aggregates, pure domain layer with zero external dependencies, 511 REST endpoints behind Actix-web, sub-5ms P99 latency targets, 110 thousand lines of Rust compiled with LTO.

2. Encoding Belgian property law into code: how Articles 3.84 through 3.94 of the Belgian Civil Code become Rust entities with compile-time invariants, state machines for legal workflows like AG convocations with mandatory 15 day notice periods, 50 percent plus one quorum validation, proxy voting limited to 3 mandates with 10 percent exception, the Belgian Minimum Normalized Chart of Accounts with 90 pre-seeded accounts.

3. The five-level test strategy and why each level matters: domain unit tests for pure logic, BDD Gherkin as executable business specifications, backend E2E as API contract enforcement, Playwright E2E as visual proof with recorded videos shipped as documentation, and DTO contract tests with auto-generated OpenAPI to guarantee Rust server and TypeScript client never diverge silently.

4. Technical challenges solved: multi-tenancy by organization, combined physical and remote quorum calculation for video conference general assemblies, share-based voting with three majority types using thousandths, a local exchange trading system with time-based currency allowing negative balances, and Stripe plus SEPA integration for the Belgian EUR-only context.

---

## 4. Présentation Slides — Release 0.1.0

Génère une présentation structurée de 15 slides pour la release 0.1.0 de KoproGo. Public mixte : syndics, copropriétaires, investisseurs, développeurs. Chaque slide a un titre court et 3-4 bullet points maximum.

Slide 1 — KoproGo 0.1.0 : titre, tagline "La copropriété belge, enfin bien gérée", open-source, première release officielle.

Slide 2 — Le problème : logiciels existants propriétaires, souvent non-conformes au droit belge, pas de transparence, interfaces vieillissantes.

Slide 3 — La solution : plateforme SaaS open-source, conforme aux Articles 3.84-3.94 du Code Civil belge, Rust pour la performance et la fiabilité, interface web moderne.

Slide 4 — Chiffres clés : 110k lignes Rust, 511 endpoints API, 57 entités métier, 64 migrations SQL, 81 pages web, 886 commits.

Slide 5 — Comptabilité belge : PCMN conforme AR 12/07/2012, 90 comptes pré-chargés, TVA 6/12/21%, factures multi-lignes, rapports bilan et compte de résultats.

Slide 6 — Assemblées générales : convocations automatiques avec délais légaux 15j, tracking email, votes avec 3 majorités et tantièmes, procurations limitées à 3 avec exception 10%, PV automatique.

Slide 7 — AG en visioconférence : Zoom/Teams/Meet/Jitsi, quorum combiné présentiel + distanciel, Art. 3.87 §1 CC.

Slide 8 — Demandes d'AGE : copropriétaires atteignant 1/5 des quotes-parts peuvent demander une AG extraordinaire, seuil auto-calculé, deadline syndic 15 jours, Art. 3.87 §2 CC.

Slide 9 — Finances : paiements Stripe et SEPA, relances automatiques en 4 niveaux avec taux légal belge 8%, budgets annuels, appels de fonds, contributions, répartition des charges par quotes-parts.

Slide 10 — Maintenance : tickets avec 5 niveaux de priorité et délais automatiques, devis entrepreneurs avec scoring automatique prix/délai/garantie/réputation, backoffice prestataire PWA avec magic link, rapports de travaux.

Slide 11 — Communauté : système d'échange local SEL 1h=1 crédit, tableau d'affichage, annuaire de compétences, partage d'objets, réservation de ressources, gamification avec achievements et challenges.

Slide 12 — Sécurité et GDPR : chiffrement LUKS au repos, 2FA TOTP, GDPR complet Art. 15-18 et 21, WAF CrowdSec, IDS Suricata, monitoring Prometheus/Grafana.

Slide 13 — Qualité logicielle : pyramide 5 niveaux — 659 tests unitaires, 49 features BDD, 19 suites E2E backend, 12 specs Playwright avec vidéos, contract tests DTO OpenAPI. Revue humaine GO/NO-GO.

Slide 14 — Architecture : hexagonale Rust, Domain-Driven Design, P99 < 5ms, multi-tenancy, frontend Astro + Svelte, PostgreSQL 15, Docker, GitHub Actions CI/CD.

Slide 15 — Roadmap : J4 Automation et intégrations, J5 Mobile et API publique, J6-7 IA prédictive, IoT avancé, blockchain pour les votes. "On livre quand c'est prêt."

---

## 5. Podcast Audio — Catalogue de Services Complet

Two presenters review everything KoproGo 0.1.0 can do. The tone is educational: each function is explained as if showing it to a professional property manager evaluating the software. No talk about the project, philosophy, or business model. Only features and their daily usefulness. Duration 12-15 minutes.

Cover these functional areas in order:

1. **Buildings, units and co-owners**: Create buildings with units (apartments, cellars, garages). Multi-ownership per unit with shares verified at 100% (Art. 577-2 §4 Belgian Civil Code). Property transfers with full history. Co-owner profiles with all current and past units.

2. **Belgian accounting**: PCMN chart of accounts compliant with Belgian royal decree, 90 pre-loaded accounts in 8 classes. Double-entry bookkeeping with 4 journal types. Balance sheet and income statement generation. Multi-line invoices with Belgian VAT 6/12/21%, workflow draft → approval → locked. Accountant dashboard with global balance and recent transactions.

3. **Debt recovery and payments**: Automatic reminders in 4 escalation levels from D+15 to D+60, penalties at Belgian legal rate of 8%. Payments by Stripe card or SEPA direct debit, full lifecycle up to partial refund. Stored payment methods with default per co-owner. Payment statistics per co-owner, building and invoice.

4. **Budgets, calls for funds and charges**: Annual budget per building with AG approval workflow and budget vs actual variance analysis. Calls for funds with automatic individual contributions based on ownership shares. Automatic charge distribution for each invoice across co-owners. Unpaid contribution tracking.

5. **General assemblies**: Automatic convocations with legal deadlines (15d ordinary, 8d extraordinary), email tracking, D-3 reminders, multilingual FR/NL/DE/EN. Voting with 3 majority types (simple, absolute, qualified), thousandths-based voting power, proxy voting. Video conference via Zoom/Teams/Meet/Jitsi with combined physical + remote quorum (Art. 3.87 §1 CC). Extraordinary assembly requests by co-signatories reaching 1/5 of shares with 15-day syndic deadline (Art. 3.87 §2 CC).

6. **Polls and board of co-ownership**: Polls between assemblies in 4 types (yes/no, multiple choice, rating, open-ended), anonymous or named, with duplicate prevention and automatic results. Board with dated mandates (president, treasurer, secretary), AG decision tracking dashboard with execution statistics.

7. **Maintenance and works**: Tickets with 5 priorities and automatic deadlines (critical 1h to low 7d), workflow open → assigned → resolved → closed. Contractor quotes with minimum 3 required above 5000 EUR, automatic scoring price/delay/warranty/reputation, decision traceability. Work reports via 72h magic link without account. Digital maintenance logbook with warranty tracking and mandatory technical inspection planning.

8. **Community life**: Local exchange system (SEL) where 1 hour = 1 credit, mutual ratings and contributor leaderboard. Notice board, skills directory, shared object library with borrowing tracking. Common resource booking with time slot conflict prevention. Gamification with 5-tier badges, temporary challenges and engagement leaderboard.

9. **Energy and IoT**: Group energy purchasing with GDPR consent, anonymous aggregation (minimum 5 participants), supplier offer comparison, immediate deletion on consent withdrawal. Smart meters Linky/Ores with automatic readings, daily/monthly aggregation, anomaly detection.

10. **Documents and etat date**: Document uploads up to 50 MB linked to buildings, meetings or invoices. Etat date for unit sales: mandatory Belgian legal document with financial data, 10-day legal deadline tracking, statuses from requested to delivered to notary.

11. **Security and compliance**: 2FA TOTP with backup codes, 5 login attempts limit, multi-role with syndic/co-owner switching. Full GDPR: export, rectification, erasure, processing restriction, marketing opt-out, timestamped audit trail. Multi-channel notifications (email, SMS, push, in-app) with 22 types and per-co-owner preferences. Public syndic page per Belgian legal requirement.

12. **Multi-property management**: One account to manage multiple isolated co-ownerships, with subscription plans (free to enterprise) and configurable building/user limits.

---

## 6. Video — Service Catalog, Educational Walkthrough

A presenter explains, function by function, everything KoproGo 0.1.0 can do. The format is educational: no screen capture, no live demo. Each function is explained concretely — what it does and why it is useful for a property manager or a board of co-owners. The tone is that of a trainer simplifying for a domain audience. No talk about the project, vision or technical stack. Duration 6-7 minutes. Each section starts with a title card displayed for 2 seconds. Short transitions between sections.

**[Title] Property structure**
The software lets you organize a co-ownership: register the building, then each unit with its floor and area. Each unit can belong to multiple co-owners, each with an ownership share. The system enforces that active shares total exactly 100%, as required by Belgian Civil Code Article 577-2. Ownership transfers are recorded with full history.

**[Title] Accounting and invoicing**
Accounting follows the Belgian PCMN chart of accounts with 90 pre-loaded accounts. Double-entry bookkeeping across 4 journal types produces a balance sheet and income statement. Invoices support multiple lines with Belgian VAT rates of 6, 12 and 21%. Once approved, an invoice is locked and cannot be modified.

**[Title] Payments and debt recovery**
Co-owners pay by Stripe card or SEPA direct debit, with full lifecycle tracking including partial refunds. Unpaid charges trigger automatic recovery in 4 escalation steps over 60 days, with penalties calculated at the Belgian legal rate of 8%. Annual budgets are voted in assembly, with real-time variance analysis. Calls for funds automatically generate individual contributions based on each co-owner's share.

**[Title] General assemblies**
Convocations are sent automatically with Belgian legal deadlines enforced. Email tracking shows who opened, who confirmed attendance, who gave proxy. Reminders go out 3 days before to non-openers. During the assembly, each resolution is voted with the correct majority type — simple, absolute or qualified — weighted by thousandths. Hybrid assemblies combine physical and remote quorum via Zoom, Teams or Jitsi. Co-owners can petition for an extraordinary assembly when co-signatories reach one fifth of total shares.

**[Title] Maintenance and works**
Issues are reported as tickets with 5 priority levels and automatic deadlines. For works above 5,000 EUR, Belgian law requires 3 quotes minimum — the software compares them automatically with a weighted scoring algorithm. After the work, the contractor submits a report via a 72-hour magic link without needing an account. A digital maintenance logbook tracks all interventions, warranties and mandatory technical inspections with overdue alerts.

**[Title] Community and energy**
Co-owners exchange services via a local exchange system where 1 hour equals 1 credit, with mutual ratings. A notice board, skills directory, shared object library and common resource booking system build community life, with gamification rewards. Group energy purchasing campaigns aggregate consumption data anonymously with GDPR consent, and smart meters provide automatic readings with anomaly detection.

**[Title] Security and compliance**
Two-factor authentication, GDPR-complete rights (export, rectification, erasure, processing restriction, marketing opt-out), multi-channel notifications across 22 types, multi-role switching, public syndic contact page per Belgian law, centralized documents up to 50 MB, and etat date generation for property sales with 10-day legal deadline tracking. One account manages multiple fully isolated co-ownerships.

---

## 7. Slide Deck — Service Catalog

Generate an 18-slide presentation for the KoproGo 0.1.0 service catalog. Target audience: professional property managers, boards of co-owners, co-owners. Each slide has a short title and 3-5 concrete bullet points. No project or technical talk: only what the software does and why it is useful.

Slide 1 — **KoproGo 0.1.0 — Service Catalog**: subtitle "All features available in the first release", target: Belgian property managers and co-ownerships.

Slide 2 — **Buildings and units**: Create buildings with units (apartments, cellars, garages). Multi-ownership per unit with shares verified at 100% (Art. 577-2 CC). Property transfers with full history. Co-owner profile with all current and past units.

Slide 3 — **PCMN Accounting**: Belgian chart of accounts compliant with royal decree, 90 pre-loaded accounts. Double-entry bookkeeping, 4 journal types. Balance sheet and income statement generation. Accountant dashboard for daily verification.

Slide 4 — **Invoices and Belgian VAT**: Multi-line invoices with VAT 6%, 12%, 21%. Workflow draft → approval → locked. Automatic HT, VAT, TTC calculations. Approved invoice cannot be modified.

Slide 5 — **Debt recovery**: 4 automatic escalation levels: friendly reminder (D+15), formal (D+30), formal notice (D+45), legal action (D+60). Penalties at Belgian legal rate of 8%. Full traceability of each reminder.

Slide 6 — **Stripe and SEPA payments**: Payment by card or SEPA direct debit. Full lifecycle: pending → succeeded/failed → refunded. Partial refunds with over-refund protection. Stored payment methods with default. Statistics per co-owner and building.

Slide 7 — **Budgets and calls for funds**: Annual budget with AG approval workflow. Budget vs actual variance analysis. Calls for funds with auto-calculated individual contributions. Charge distribution by ownership shares. Unpaid contribution tracking.

Slide 8 — **AG convocations**: Legal deadlines enforced: 15d ordinary, 8d extraordinary. Automatic sending with email tracking. D-3 reminders to non-openers. Attendance confirmations and proxy delegations. Support FR/NL/DE/EN.

Slide 9 — **Assembly voting**: 3 majority types: simple, absolute, qualified. Votes weighted by thousandths. Proxy voting. Automatic result calculation at closing.

Slide 10 — **Video conference and EGA**: Hybrid AG: Zoom, Teams, Meet, Jitsi. Combined physical + remote quorum (Art. 3.87 §1 CC). Extraordinary assembly requests by co-owners reaching 1/5 of shares. 15-day syndic deadline tracked automatically (Art. 3.87 §2 CC).

Slide 11 — **Polls and board**: 4 poll types: yes/no, multiple choice, rating, open-ended. Anonymous or named voting, duplicate prevention, automatic results. Board of co-ownership with mandates and AG decision tracking.

Slide 12 — **Maintenance tickets**: 5 priorities with automatic deadlines (critical 1h, urgent 4h, standard 3d). Workflow open → assigned → resolved → closed. Dashboard with statistics and overdue tickets.

Slide 13 — **Quotes and work reports**: Minimum 3 quotes required above 5000 EUR. Automatic scoring: price 40%, delay 30%, warranty 20%, reputation 10%. Contractor reports via 72h magic link without account. Board validation.

Slide 14 — **Maintenance logbook**: Intervention history with photos and documents. Active and expiring warranty tracking. Mandatory inspection planning: elevator, boiler room, electricity. Overdue alerts and upcoming deadlines.

Slide 15 — **Community life**: Local exchange SEL: 1h = 1 credit, mutual ratings, leaderboard. Notice board, skills directory, shared object library. Resource booking with time slot conflict prevention. Gamification: badges, challenges, points.

Slide 16 — **Energy and IoT**: Group energy purchasing with GDPR consent and anonymous aggregation. Smart meters Linky/Ores with automatic readings. Daily/monthly statistics and anomaly detection.

Slide 17 — **Security and GDPR**: 2FA TOTP with backup codes. Multi-role (syndic + co-owner). Full GDPR: export, rectification, erasure, restriction, marketing opt-out. Multi-channel notifications (email, SMS, push, in-app), 22 configurable types. Public syndic page (legal requirement). Centralized documents up to 50 MB. Etat date for unit sales with 10-day legal deadline.

Slide 18 — **Multi-property management**: One account for multiple isolated co-ownerships. Subscription plans: free, starter, professional, enterprise. Configurable building and user limits.
