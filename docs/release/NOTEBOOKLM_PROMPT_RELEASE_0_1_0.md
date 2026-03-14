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

2. La modélisation du droit belge de la copropriété dans le code : comment les Articles 3.84 à 3.94 du Code Civil deviennent des entités Rust avec des invariants vérifiés à la compilation, les machines à états pour les workflows légaux comme les convocations avec délais de 15 et 8 jours, le quorum à 50 pourcent plus un, la limite de 3 procurations avec exception des 10 pourcent, le Plan Comptable Minimum Normalisé avec 90 comptes pré-seedés.

3. La stratégie de test à cinq niveaux et pourquoi chaque niveau existe : les tests unitaires domaine pour la logique pure, le BDD Gherkin comme spécification exécutable des règles métier, les tests E2E backend comme contrat API, les tests Playwright comme preuve visuelle avec vidéos intégrées à la documentation, et les contract tests DTO avec OpenAPI auto-généré pour garantir la cohérence entre le serveur Rust et le client TypeScript.

4. Les défis techniques rencontrés : le multi-tenancy par organisation, le calcul de quorum combiné présentiel et distanciel pour les AG en visioconférence, la gestion des tantièmes et millièmes pour les votes avec trois types de majorité, le système d'échange local SEL avec monnaie temps et balance négative autorisée, l'intégration Stripe et SEPA pour le contexte belge.

---

## 3. Deep Dive Technique (EN)

Do a deep dive in English. Two software engineering experts discuss the architecture and engineering decisions behind KoproGo, an open-source Belgian condominium management SaaS. Target audience is technical: Rust developers, software architects, PropTech CTOs.

Topics to explore:

1. Why Rust and hexagonal architecture for real estate SaaS: ports and adapters pattern, Domain-Driven Design with 57 aggregates, pure domain layer with zero external dependencies, 511 REST endpoints behind Actix-web, sub-5ms P99 latency targets, 110 thousand lines of Rust compiled with LTO.

2. Encoding Belgian property law into code: how Articles 3.84 through 3.94 of the Belgian Civil Code become Rust entities with compile-time invariants, state machines for legal workflows like AG convocations with mandatory 15 and 8 day notice periods, 50 percent plus one quorum validation, proxy voting limited to 3 mandates with 10 percent exception, the Belgian Minimum Normalized Chart of Accounts with 90 pre-seeded accounts.

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

Slide 6 — Assemblées générales : convocations automatiques avec délais légaux 15j/8j, tracking email, votes avec 3 majorités et tantièmes, procurations limitées à 3 avec exception 10%, PV automatique.

Slide 7 — AG en visioconférence : Zoom/Teams/Meet/Jitsi, quorum combiné présentiel + distanciel, Art. 3.87 §1 CC.

Slide 8 — Demandes d'AGE : copropriétaires atteignant 1/5 des quotes-parts peuvent demander une AG extraordinaire, seuil auto-calculé, deadline syndic 15 jours, Art. 3.87 §2 CC.

Slide 9 — Finances : paiements Stripe et SEPA, relances automatiques en 4 niveaux avec taux légal belge 8%, budgets annuels, appels de fonds, contributions, répartition des charges par quotes-parts.

Slide 10 — Maintenance : tickets avec 5 niveaux de priorité et délais automatiques, devis entrepreneurs avec scoring automatique prix/délai/garantie/réputation, backoffice prestataire PWA avec magic link, rapports de travaux.

Slide 11 — Communauté : système d'échange local SEL 1h=1 crédit, tableau d'affichage, annuaire de compétences, partage d'objets, réservation de ressources, gamification avec achievements et challenges.

Slide 12 — Sécurité et GDPR : chiffrement LUKS au repos, 2FA TOTP, GDPR complet Art. 15-18 et 21, WAF CrowdSec, IDS Suricata, monitoring Prometheus/Grafana.

Slide 13 — Qualité logicielle : pyramide 5 niveaux — 659 tests unitaires, 49 features BDD, 19 suites E2E backend, 12 specs Playwright avec vidéos, contract tests DTO OpenAPI. Revue humaine GO/NO-GO.

Slide 14 — Architecture : hexagonale Rust, Domain-Driven Design, P99 < 5ms, multi-tenancy, frontend Astro + Svelte, PostgreSQL 15, Docker, GitHub Actions CI/CD.

Slide 15 — Roadmap : J4 Automation et intégrations, J5 Mobile et API publique, J6-7 IA prédictive, IoT avancé, blockchain pour les votes. "On livre quand c'est prêt."
