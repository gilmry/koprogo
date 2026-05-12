# Product Brief — KoproGo

## Methode Maury — Phase TOGAF A (Vision)

**Auteurs** : Gilles Maury & Farah Maury
**Agent BMAD** : Mary (Analyste)
**Date** : 29/03/2026
**Version** : 1.0

---

## 1. Vision

KoproGo est une plateforme SaaS de gestion de copropriete belge, concue pour remplacer les outils fragmentes (Excel, papier, logiciels obsoletes) par une solution integree, performante et conforme au droit belge (Article 577 et suivants du Code Civil). La plateforme couvre l'integralite du cycle de vie d'une copropriete : gestion immobiliere, comptabilite PCMN, assemblees generales avec vote numerique, facturation avec TVA belge, recouvrement, GDPR, ticketing maintenance, paiements Stripe/SEPA, et modules communautaires (SEL, sondages, gamification).

L'architecture repose sur le **Domain-Driven Design** et l'**Architecture Hexagonale** (Ports & Adapters) en Rust, garantissant des performances P99 < 5ms, une empreinte memoire < 128 MB par instance, et une empreinte ecologique < 0.5g CO2/requete. Le frontend Astro + Svelte (Islands Architecture) assure une experience reactive avec un JavaScript minimal.

La progression suit un modele **par capacites** (pas de dates fixes) : chaque jalon debloque un palier d'adoption mesurable, de 10-20 coproprietes (beta fermee, Jalon 0 acheve) jusqu'a 10 000+ coproprietes (Jalon 7, platform economy). KoproGo est structure en ASBL, financee par cotisations (5EUR/mois cloud), avec trois moteurs d'acquisition : Gestion (remplacement complet), Communaute (modules standalone), et Valeurs (sympathisants).

---

## 2. Stakeholders

| Partie prenante | Preoccupation | Influence |
|-----------------|---------------|-----------|
| **Syndic professionnel** | Conformite legale belge (PCMN, AG, etats dates), gain de temps sur taches repetitives, recouvrement automatise, convocations legales | Forte |
| **Copropietaire** | Transparence financiere, droit de vote conforme, acces documents, GDPR (Articles 15-21), participation communautaire (SEL, sondages) | Forte |
| **Comptable** | Plan comptable PCMN conforme AR 12/07/2012, rapports financiers (bilan, compte de resultats), ecritures manuelles double-entree, variance budgetaire | Moyenne |
| **SuperAdmin** | Gestion multi-tenancy (organisations, utilisateurs), monitoring infrastructure, audit GDPR, scalabilite | Forte |
| **Prestataire / Entrepreneur** | Soumission devis, rapports de travaux via magic link PWA, suivi tickets maintenance, notation satisfaction | Faible |
| **Legislateur belge** | Conformite Article 577 Code Civil, loi sur la copropriete forcee, GDPR, PSD2, accessibilite WCAG 2.1 AA | Moyenne |

---

## 3. Drivers Business

- **Pourquoi maintenant ?** La legislation belge sur la copropriete (Code Civil Art. 577 et suivants, reformes 2018-2019) impose des exigences croissantes : comptabilite PCMN normalisee, quorum valides, majorites qualifiees, etats dates obligatoires pour les ventes, GDPR strict. Les outils existants (Vilogi, Apronet, Excel) ne couvrent pas ces specificites belges de maniere integree. Le marche belge compte environ 300 000 coproprietes, dont 60% de plus de 20 lots (conseil de copropriete obligatoire).

- **Processus metier impactes** :
  - Comptabilite (PCMN, bilan, compte de resultats, appels de fonds)
  - Assemblees generales (convocations legales 15j, quorum 50%, 4 types de majorite Art. 3.88 CC, procurations)
  - Gestion financiere (facturation TVA 6/12/21%, recouvrement 4 niveaux, paiements Stripe/SEPA)
  - Maintenance (ticketing SLA, devis multi-entrepreneurs, carnet d'entretien digital)
  - Communication (notifications multi-canal, sondages, convocations avec tracking)
  - Conformite (GDPR Articles 15-21, etats dates, information publique syndic)

- **Paysage applicatif existant** : Syndics belges utilisent une combinaison de tableurs Excel (comptabilite), emails (convocations manuelles), Word (PV d'AG), outils generiques (Trello pour tickets), et logiciels proprietaires non conformes au droit belge. Aucune solution ne couvre a la fois la comptabilite PCMN, les AG avec vote numerique, le recouvrement automatise, et les modules communautaires.

---

## 4. Probleme

Les syndics belges perdent 10-15h/semaine sur des taches administratives fragmentees entre outils non integres : saisie comptable manuelle dans Excel (sans respect PCMN), convocations AG par courrier papier (risque de non-conformite aux delais legaux de 15 jours), calcul manuel des tantiemes et majorites qualifiees (erreurs frequentes), suivi des impayes par email (pas de workflow d'escalade). Les copropietaires, eux, manquent de transparence : pas d'acces en temps reel aux comptes, pas de vote numerique, pas de suivi des travaux. Le cout de l'inaction est double : risque legal (AG annulables pour vice de forme) et perte de confiance des copropietaires envers leur syndic. Aucun outil du marche belge ne gere nativement les tantiemes/milliemes, les 4 types de majorite (absolue, 2/3, 4/5, unanimite — Art. 3.88 CC), l'etat date obligatoire pour les ventes, et le plan comptable PCMN impose par arrete royal.

---

## 5. Proposition de valeur

Pour les **syndics et copropietaires belges** qui subissent la fragmentation des outils et le risque de non-conformite legale, **KoproGo** est une **plateforme SaaS de gestion de copropriete** qui **integre comptabilite PCMN, AG numeriques, paiements, GDPR et modules communautaires en une seule application conforme au droit belge**. Contrairement a Vilogi, Apronet ou Excel, KoproGo est **construit specifiquement pour la loi belge** (Art. 577 CC), offre des performances P99 < 5ms, une empreinte CO2 < 0.5g/requete, et une architecture open source (AGPL-3.0) garantissant perennite et transparence.

---

## 6. Personas

### Persona 1 : Marc, Syndic Professionnel

- **Role** : Gere 15-50 coproprietes belges (200-800 lots), emploi principal
- **Objectifs** : Gagner du temps sur l'administratif (convocations, comptabilite, recouvrement), assurer la conformite legale sans risque d'annulation d'AG, offrir un tableau de bord transparent aux copropietaires
- **Frustrations** : Jongle entre 4-5 outils non connectes, perd 2h par AG en calculs manuels de quorum/majorites, risque d'erreur sur les etats dates lors des ventes, relances d'impayes sans workflow structure
- **Scenario typique** : Marc prepare l'AG annuelle de la "Residence du Parc Royal" (182 lots, 10 000 dix-milliemes). KoproGo genere automatiquement les convocations avec respect du delai legal de 15 jours, calcule le quorum a 50%, gere les votes avec tantiemes et 4 types de majorite (Art. 3.88 CC), procurations (max 3 mandats), puis genere le PV. Il lance ensuite l'appel de fonds trimestriel avec distribution automatique des charges selon les quotes-parts.

### Persona 2 : Sophie, Copropietaire

- **Role** : Proprietaire d'un appartement dans une copropriete de 30 lots, salariee a temps plein
- **Objectifs** : Comprendre ses charges, voter facilement aux AG (meme a distance), signaler les problemes de maintenance rapidement, participer a la vie de l'immeuble
- **Frustrations** : Recoit les convocations AG par courrier 10 jours avant (trop tard pour s'organiser), ne comprend pas les releves comptables en Excel, ne sait pas ou en sont les travaux votes, pas de canal simple pour signaler une fuite
- **Scenario typique** : Sophie recoit une notification email 15 jours avant l'AG. Elle consulte l'ordre du jour sur KoproGo, donne procuration via l'application a sa voisine, puis suit les resultats des votes en temps reel. Elle cree un ticket maintenance pour une fuite dans le parking, suit l'avancement, et note le prestataire apres intervention.

### Persona 3 : Jean-Pierre, Comptable de copropriete

- **Role** : Comptable externe mandate par 5 syndics, gere la comptabilite de 20 coproprietes
- **Objectifs** : Saisir les ecritures conformes au PCMN belge (AR 12/07/2012), produire bilan et compte de resultats, gerer les appels de fonds et le recouvrement, exporter vers Winbooks
- **Frustrations** : Plan comptable belge non supporte par les outils generiques, double saisie entre logiciel syndic et logiciel comptable, calcul manuel des penalites de retard (taux legal belge 8%), pas de rapports consolides
- **Scenario typique** : Jean-Pierre ouvre le dashboard comptable KoproGo, saisit une ecriture journal (double-entree, journal ACH/VEN/FIN/ODS), lance le calcul de variance budgetaire, genere l'etat date pour une vente de lot, et exporte le bilan annuel. Le PCMN avec ses ~90 comptes pre-seedes lui evite toute configuration.

### Persona 4 : Ahmed, Prestataire Plombier

- **Role** : Entrepreneur independant intervenant dans 10 coproprietes
- **Objectifs** : Recevoir les demandes de devis, soumettre ses offres, documenter ses interventions, etre paye rapidement
- **Frustrations** : Pas de canal structure pour recevoir les demandes, rapports d'intervention sur papier perdus, delais de paiement non suivis, pas de visibilite sur sa notation
- **Scenario typique** : Ahmed recoit un email avec un magic link (valide 72h) pour documenter son intervention sur un ticket maintenance. Il remplit le rapport via l'interface PWA (photos avant/apres, pieces remplacees, compte-rendu), le soumet au conseil de copropriete qui valide, declenchant le paiement automatique via Stripe.

> **Note** : Ces 4 personas sont les archétypes simplifiés. Le système de seeds de test utilise **21 personas** détaillés répartis sur 3 immeubles (Résidence du Parc Royal 182 lots, Le Clos des Hirondelles 12 lots, Les Terrasses de Flagey 48 lots), couvrant 10 copropriétaires, 4 membres communauté, 3 professionnels et 1 admin. Voir `docs/specs/00-personas-et-seed.rst` pour le détail complet avec analyse sociologique (coalitions, blocs de vote, dynamiques de pouvoir).

---

## 7. Capacites metier requises

1. **Gestion immobiliere** — Immeubles, lots, copropietaires, quotes-parts (tantiemes/milliemes), transferts de propriete
2. **Comptabilite PCMN belge** — Plan comptable normalise (AR 12/07/2012), ~90 comptes, ecritures double-entree (ACH/VEN/FIN/ODS), bilan, compte de resultats, appels de fonds, contributions
3. **Assemblees generales** — Convocations legales (delai 15j ordinaire/8j extraordinaire), quorum 50%, vote numerique (4 majorites Art. 3.88 : absolue, 2/3, 4/5, unanimite), procurations (max 3 mandats + exception 10%), PV, visioconference, AGE agile (petition 1/5)
4. **Facturation et TVA belge** — Factures multi-lignes, taux TVA 6%/12%/21%, workflow approbation (Draft/PendingApproval/Approved/Rejected)
5. **Paiements** — Stripe Payment Intents, SEPA Direct Debit, methodes de paiement stockees, remboursements, idempotency
6. **Recouvrement automatise** — 4 niveaux d'escalade (Gentle J+15, Formal J+30, FinalNotice J+45, LegalAction J+60), penalites taux legal belge 8%
7. **Budgets et rapports financiers** — Budget annuel avec variance analysis, bilan, compte de resultats, etat date pour ventes immobilieres
8. **Ticketing maintenance** — 6 etats, 5 priorites avec SLA automatiques, assignation prestataires, statistiques
9. **Conformite GDPR** — Articles 15 (acces), 16 (rectification), 17 (effacement), 18 (restriction), 21 (opposition marketing), 30 (registre)
10. **Notifications multi-canal** — Email, SMS, Push, In-App, 22 types, preferences utilisateur granulaires
11. **Gestion documentaire** — Upload/download, liaison reunions/depenses, stockage securise
12. **Modules communautaires** — SEL (monnaie temps), sondages (4 types), annonces, repertoire competences, bibliotheque objets, reservation ressources, gamification (achievements, challenges, leaderboard)
13. **Devis entrepreneurs** — Comparaison multi-devis (scoring : prix 40%, delai 30%, garantie 20%, reputation 10%), conformite belge (3 devis obligatoires >5000EUR)
14. **Energie** — Achats groupes, IoT Linky, statistiques consommation, detection anomalies
15. **Securite** — 2FA TOTP, rate limiting, LUKS encryption at-rest, IDS Suricata, WAF CrowdSec, fail2ban, audit Lynis
16. **Multi-tenancy** — Organisations, roles (SuperAdmin/Syndic/Owner/Comptable), RBAC, isolation donnees

---

## 8. Glossaire metier (Ubiquitous Language DDD)

| Terme metier | Definition | Exemple |
|-------------|------------|---------|
| **Tantieme / Dix-millieme** | Quote-part de copropriete exprimee en fraction (0.0 < p <= 1.0), base pour le calcul des charges et du pouvoir de vote. Exprimee en milliemes (/1000) ou dix-milliemes (/10000) selon la taille de l'immeuble | Appartement 3B : 450/10000 dix-milliemes = 4.5% des charges |
| **Quorum** | Seuil minimum de presences (physiques + procurations) pour que l'AG soit valablement constituee. 50% des quotes-parts (Art. 3.87 ss5 CC) | AG de 10000 dix-milliemes : quorum atteint si 5001+ dix-milliemes representes |
| **Majorite** | 4 types de majorite selon Art. 3.88 CC : Absolue (>50% des presents/representes, abstentions exclues), Deux tiers (>=2/3), Quatre cinquiemes (>=4/5), Unanimite (100% de TOUS les tantiemes y compris absents) | Travaux structurels : majorite 2/3, modification reglement : majorite 4/5, dissolution : unanimite |
| **Syndic** | Personne physique ou morale mandatee pour gerer la copropriete (administration, comptabilite, AG, maintenance) | Marc, syndic professionnel gerant 30 coproprietes |
| **Copropietaire** | Personne physique ou morale proprietaire d'un ou plusieurs lots dans l'immeuble | Sophie, proprietaire de l'appartement 3B |
| **Assemblee Generale (AG)** | Reunion obligatoire des copropietaires pour voter les decisions relatives a l'immeuble. Ordinaire (annuelle) ou extraordinaire | AG ordinaire annuelle de la "Residence du Parc" |
| **AGE** | Assemblee Generale Extraordinaire, declenchable sur petition de 1/5 des quotes-parts (Art. 3.87 ss2 CC) | 5 proprietaires representant 22% des tantiemes demandent une AGE |
| **Convocation** | Document officiel invitant les copropietaires a l'AG, soumis a un delai legal minimum (15 jours ordinaire, Art. 3.87 ss3 CC) | Convocation envoyee le 1er mars pour AG du 20 mars |
| **Resolution** | Proposition soumise au vote lors d'une AG, avec type de majorite requise | "Approuver les travaux de toiture pour 45 000 EUR" (majorite 2/3) |
| **Procuration** | Delegation de pouvoir de vote d'un copropietaire absent a un autre copropietaire present | Sophie donne procuration a sa voisine Alice pour l'AG du 20 mars |
| **PCMN** | Plan Comptable Minimum Normalise belge (AR 12/07/2012), 8 classes comptables obligatoires pour les coproprietes | Classe 4 : Creances, Classe 6 : Charges, Classe 7 : Produits |
| **Appel de fonds** | Demande de paiement collective adressee aux copropietaires pour financer les charges courantes ou travaux | Appel de fonds Q1 2026 : 25 000 EUR repartis selon tantiemes |
| **Etat date** | Document legal obligatoire lors de la vente d'un lot, resumant la situation financiere du copropietaire vendeur (Art. 577-11 ss2 CC) | Etat date genere pour la vente du lot 12A : solde debiteur 1 250 EUR |
| **Conseil de copropriete** | Organe de controle obligatoire pour coproprietes >20 lots (President, Vice-President, Tresorier, Secretaire) | Conseil de la "Residence du Parc" : 5 membres elus |
| **SEL** | Systeme d'Echange Local, monnaie temps entre voisins (1h = 1 credit). Legal en Belgique si non commercial | Pierre offre 2h de jardinage (2 credits), Marie demande du babysitting |
| **Charge distribution** | Repartition d'une facture entre copropietaires selon leurs quotes-parts | Facture electricite 3 000 EUR : lot 3B (4.5%) paie 135 EUR |

> **Note** : Ce glossaire est la source de verite pour le langage ubiquitaire DDD. Chaque terme est utilise tel quel dans le code (noms d'entites, methodes, variables), dans les scenarios BDD (Gherkin), et dans la documentation. Aucune traduction ou synonyme.

---

## 9. Bounded Contexts identifies (DDD)

| Contexte | Responsabilite | Entites principales |
|----------|---------------|---------------------|
| **Building Management** | Gestion des immeubles, lots, quotes-parts | Building, Unit, UnitOwner |
| **Identity & Access** | Authentification, autorisation, multi-role, 2FA | User, UserRoleAssignment, RefreshToken, TwoFactorSecret, Organization |
| **General Assembly** | AG, convocations, resolutions, votes, visioconference, AGE | Meeting, Convocation, ConvocationRecipient, Resolution, Vote, AgSession, AgeRequest |
| **Accounting** | Comptabilite PCMN, ecritures, rapports financiers | Account, JournalEntry, JournalEntryLine, Budget, EtatDate |
| **Billing & Payments** | Factures, paiements, recouvrement, appels de fonds | Expense, InvoiceLineItem, Payment, PaymentMethod, PaymentReminder, OwnerContribution, CallForFunds, ChargeDistribution |
| **Maintenance** | Tickets, devis, rapports travaux, inspections | Ticket, Quote, WorkReport, TechnicalInspection, ContractorReport |
| **Notifications** | Multi-canal, preferences, tracking | Notification, NotificationPreference |
| **GDPR & Compliance** | Conformite GDPR, export, effacement, restriction | GdprExport, GdprErasure, GdprRectification (operations sur User) |
| **Community** | SEL, sondages, annonces, competences, partage, reservations | LocalExchange, OwnerCreditBalance, Poll, PollOption, PollVote, CommunityNotice, Skill, SharedObject, ResourceBooking |
| **Gamification** | Achievements, challenges, leaderboard | Achievement, UserAchievement, Challenge, ChallengeProgress |
| **Documents** | Stockage fichiers, liaison entites | Document |
| **Energy & IoT** | Achats groupes, compteurs intelligents, Linky | EnergyCampaign, ProviderOffer, EnergyBillUpload, IoTReading, LinkyDevice |
| **Board Management** | Conseil de copropriete, decisions post-AG | BoardMember, BoardDecision |

```
┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│ Building Management  │────▶│  General Assembly    │────▶│    Accounting        │
│                      │     │                      │     │                      │
│ - Building           │     │ - Meeting            │     │ - Account (PCMN)     │
│ - Unit               │     │ - Resolution         │     │ - JournalEntry       │
│ - UnitOwner          │     │ - Vote               │     │ - Budget             │
│                      │     │ - Convocation        │     │ - EtatDate           │
└──────────┬───────────┘     │ - AgSession          │     └──────────┬───────────┘
           │                 │ - AgeRequest         │                │
           │                 └──────────┬───────────┘                │
           │                            │                            │
           ▼                            ▼                            ▼
┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│ Billing & Payments   │     │    Maintenance       │     │  Board Management    │
│                      │     │                      │     │                      │
│ - Expense            │     │ - Ticket             │     │ - BoardMember        │
│ - Payment            │     │ - Quote              │     │ - BoardDecision      │
│ - PaymentReminder    │     │ - WorkReport         │     │                      │
│ - CallForFunds       │     │ - ContractorReport   │     │                      │
└──────────┬───────────┘     └──────────┬───────────┘     └──────────────────────┘
           │                            │
           ▼                            ▼
┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│  Identity & Access   │     │  GDPR & Compliance   │     │    Notifications     │
│                      │     │                      │     │                      │
│ - User               │     │ - Export Art.15      │     │ - Notification       │
│ - Organization       │     │ - Erasure Art.17     │     │ - NotifPreference    │
│ - UserRoleAssignment │     │ - Rectify Art.16     │     │                      │
│ - TwoFactorSecret    │     │ - Restrict Art.18    │     │                      │
└──────────────────────┘     └──────────────────────┘     └──────────────────────┘
           │
           ▼
┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│     Community        │     │    Gamification      │     │    Energy & IoT      │
│                      │     │                      │     │                      │
│ - LocalExchange      │     │ - Achievement        │     │ - EnergyCampaign     │
│ - Poll               │     │ - Challenge          │     │ - IoTReading         │
│ - CommunityNotice    │     │ - ChallengeProgress  │     │ - LinkyDevice        │
│ - Skill, SharedObject│     │ - Leaderboard        │     │ - EnergyBillUpload   │
└──────────────────────┘     └──────────────────────┘     └──────────────────────┘
```

---

## 10. Invariants metier critiques

> Ces regles sont codees dans les constructeurs des entites Domain (`::new() -> Result<Self, DomainError>`). Une violation d'invariant = erreur runtime immediate. Jamais silencieuse.

1. **Somme des quotes-parts actives = 100%** (tolerance +/-0.01% pour arrondis)
   - Entite concernee : `UnitOwner` (trigger PostgreSQL `validate_unit_ownership_total`)
   - Exemple de violation : Ajouter un copropietaire a 15% quand les quotes-parts existantes totalisent deja 92% (total 107% > 100%)
   - Reference legale : Article 577-2 ss4 Code Civil belge

2. **Quorum AG >= 50% des quotes-parts** (physiques + procurations)
   - Entite concernee : `Meeting` (champs `quorum_percentage`, `total_quotas`, `present_quotas`)
   - Exemple de violation : Ouvrir les votes d'une AG ou seulement 45% des milliemes sont representes
   - Reference legale : Article 3.87 ss5 Code Civil belge

3. **Delai legal de convocation >= 15 jours** (ordinaire et extraordinaire)
   - Entite concernee : `Convocation` (champ `minimum_send_date` calcule automatiquement)
   - Exemple de violation : Envoyer une convocation le 10 mars pour une AG le 20 mars (10 jours < 15 jours minimum)
   - Reference legale : Article 3.87 ss3 Code Civil belge

4. **Majorite de vote respectee selon le type de resolution**
   - Entite concernee : `Resolution` (champ `majority_required`: Absolute >50%, TwoThirds >=2/3, FourFifths >=4/5, Unanimity 100% de tous les tantiemes)
   - Abstentions exclues du calcul pour Absolute/TwoThirds/FourFifths, mais incluses pour Unanimity (calcul sur TOUS les tantiemes y compris absents)
   - Exemple de violation : Declarer adoptee une resolution a majorite 2/3 avec seulement 60% de votes favorables
   - Reference legale : Article 3.88 ss1 Code Civil belge

5. **Comptabilite double-entree equilibree** (total debits = total credits par ecriture)
   - Entite concernee : `JournalEntry` / `JournalEntryLine`
   - Exemple de violation : Ecriture avec 1 000 EUR au debit du compte 6000 sans contrepartie au credit

6. **Quote-part comprise entre 0% exclus et 100% inclus** (0.0 < p <= 1.0)
   - Entite concernee : `UnitOwner` (validation constructeur)
   - Exemple de violation : Creer un copropietaire avec une quote-part de 0% ou -5%

7. **Idempotency des paiements** (cle unique >= 16 caracteres, pas de double charge)
   - Entite concernee : `Payment` (champ `idempotency_key`)
   - Exemple de violation : Executer deux fois le meme paiement Stripe suite a un retry reseau

8. **Seuil AGE : petition valide a 1/5 des quotes-parts** (20%)
   - Entite concernee : `AgeRequest` (champ `threshold_pct` = 0.20, verification automatique a chaque cosignature)
   - Exemple de violation : Soumettre une demande d'AGE au syndic avec seulement 15% des quotes-parts

9. **Building.name non vide** (invariant de construction)
   - Entite concernee : `Building`
   - Exemple de violation : `Building::new("", ...)` retourne `Err("Name cannot be empty")`

10. **Devis obligatoires : 3 minimum pour travaux >5 000 EUR**
    - Entite concernee : `Quote` (comparaison multi-devis avec scoring automatique)
    - Exemple de violation : Accepter un devis de 12 000 EUR sans avoir au moins 2 autres devis concurrents

---

## 11. Fonctionnalites cles (MVP — Jalons 0-3, release v0.1.0)

1. **Gestion immobiliere complete** — CRUD immeubles, lots, copropietaires, quotes-parts avec validation 100%, transferts de propriete — MUST HAVE (Jalon 0 ✅)
2. **Authentification multi-role** — Login JWT, switch role, 2FA TOTP, refresh token rotation, rate limiting 5 tentatives/15min — MUST HAVE (Jalon 1 ✅)
3. **Conformite GDPR** — Articles 15 (export), 16 (rectification), 17 (effacement/anonymisation), 18 (restriction), 21 (opposition marketing), 30 (registre) — MUST HAVE (Jalon 1 ✅)
4. **Securite infrastructure** — LUKS encryption at-rest, backups GPG + S3, monitoring Prometheus/Grafana/Loki, IDS Suricata, WAF CrowdSec, fail2ban — MUST HAVE (Jalon 1 ✅)
5. **Comptabilite PCMN belge** — ~90 comptes pre-seedes (8 classes), ecritures double-entree (4 journaux), bilan, compte de resultats, variance budgetaire — MUST HAVE (Jalon 2 ✅)
6. **Assemblees generales numeriques** — Convocations legales (15j), quorum 50%, vote avec tantiemes (4 majorites Art. 3.88 : absolue, 2/3, 4/5, unanimite), procurations (max 3 mandats), visioconference — MUST HAVE (Jalons 1-3)
7. **Facturation et paiements** — Factures multi-lignes TVA belge, Stripe + SEPA, recouvrement 4 niveaux (Gentle/Formal/FinalNotice/LegalAction) — MUST HAVE (Jalons 2-3)
8. **Ticketing maintenance** — 6 etats, 5 priorites SLA, assignation prestataires, devis multi-entrepreneurs (scoring belge), rapports travaux magic link — SHOULD HAVE (Jalon 3)
9. **Modules communautaires** — SEL (monnaie temps), sondages (4 types), annonces, repertoire competences, bibliotheque objets, reservation ressources — SHOULD HAVE (Jalon 3)
10. **Gamification** — Achievements (8 categories, 5 tiers), challenges (Individual/Team/Building), leaderboard multi-source — SHOULD HAVE (Jalon 3)

---

## 12. Fonctionnalites secondaires (post-MVP — Jalons 4-7)

1. **MCP AI Syndic** — Serveur SSE + JSON-RPC, 10+ outils metier (legal_search, majority_calculator, ag_create, comptabilite_situation), integration Claude/GPT-4 — COULD HAVE (Jalon 4)
2. **Authentification itsme/eID** — Verification d'identite forte pour votes AG securises, integration OpenID Connect belgique — COULD HAVE (Jalon 4)
3. **Application desktop/mobile Tauri** — Windows/macOS/Linux/iOS/Android, mode offline SQLite, sync bidirectionnelle — COULD HAVE (Jalon 5)
4. **PWA et API publique** — Progressive Web App installable, SDK Python/JS/PHP, webhooks evenements — COULD HAVE (Jalon 5)
5. **IA Assistant Syndic** — Chatbot reglementaire copropriete belge, base de connaissance legislative — WON'T HAVE v0.1.0 (Jalon 6)
6. **API Bancaire PSD2** — Reconciliation bancaire automatique, compliance FinTech FSMA — WON'T HAVE v0.1.0 (Jalon 6)
7. **IoT Sensors temps reel** — Capteurs energie/eau MQTT + TimescaleDB, detection fuites — WON'T HAVE v0.1.0 (Jalon 6)
8. **Blockchain Voting** — Votes AG immutables sur Polygon, audit Trail of Bits — WON'T HAVE v0.1.0 (Jalon 7)
9. **Carbon Credits Trading** — Tokenisation economies CO2, ERC-20 — WON'T HAVE v0.1.0 (Jalon 7)
10. **White-label federation** — Multi-tenant SaaS, deploiement Terraform automatise, interoperabilite EU CEN/CENELEC — WON'T HAVE v0.1.0 (Jalon 7)

---

## 13. Contraintes

- **Stack imposee** : Rust 1.75+ / Actix-web 4.9 (backend) + Astro 4.x / Svelte 4.x (frontend) + PostgreSQL 15 (persistence) + Tailwind CSS 3.x (styling)
- **Disciplines** : SOLID + DDD + BDD (Cucumber/Gherkin, 819 scenarios, 69 features) + TDD (test-first, couverture domain 100%) + Architecture Hexagonale stricte (Ports & Adapters)
- **Organisation** : Scrum -> Nexus -> SAFe -> ITIL (selon croissance equipe)
- **Securite** : RGPD des la conception (Articles 15-21 + 30), chiffrement at-rest (LUKS AES-XTS-512), IDS (Suricata), WAF (CrowdSec), fail2ban, SSH hardening, kernel hardening, HSTS/CSP/X-Frame-Options
- **Infrastructure** : IaC (Terraform + Ansible), CI/CD (GitHub Actions), monitoring (Prometheus/Grafana/Loki/Alertmanager), VPS OVH -> K3s -> K8s (progression par paliers de capacite)
- **Performance** : P99 < 5ms, >100k req/s, <128 MB RAM/instance, connection pool max 10 PostgreSQL, release build LTO + codegen-units=1
- **Ecologie** : < 0.5g CO2/requete cible (mesure Green Metrics), anti-bloatware (zero dependance superflue, zero JavaScript inutile)
- **Licence** : AGPL-3.0 (open source, perennite, transparence)
- **i18n** : 4 langues (FR/NL/EN/DE), ~2000 cles par locale, 73% couverture actuelle
- **Developpement** : Agents IA supervises par Gilles Maury (Methode Maury), code review humain obligatoire
- **Tests** : Pyramide (unit -> integration testcontainers -> BDD Gherkin -> E2E Playwright -> benchmarks Criterion), 49 E2E smoke tests, 12 Documentation Vivante scenarios
- **Structure actuelle** : 559 endpoints API REST, 59 entites domaine, 80 migrations, 137k+ LOC Rust, 178 composants Svelte

---

## 14. Risques

| Risque | Probabilite | Impact | Mitigation |
|--------|------------|--------|------------|
| **Evolution legislative belge** (reforme Article 577, nouvelles obligations GDPR, PSD2) | Haute | Fort | Architecture hexagonale : regles metier isolees dans le Domain, modification sans impact infrastructure. Veille juridique active. |
| **Solo dev bottleneck** (1 dev, 10-15h/semaine, side-project) | Haute | Fort | IA pour acceleration x2-3 (Claude Code, GPT-4), communaute open source (contributeurs), modele ASBL pour financement durable (5EUR/copro/mois) |
| **Scalabilite technique** (passage de 100 a 10 000 coproprietes) | Moyenne | Fort | Infrastructure progressive (VPS -> K3s -> K8s), load tests valides (287 req/s, 99.74% success), connection pooling, LTO optimization |
| **Specificites marche belge** (marche niche, 3 langues officielles, complexite juridique) | Moyenne | Moyen | Avantage competitif : aucun concurrent ne couvre nativement le droit belge. Extension Benelux possible (NL/LU legislations proches) |
| **Securite et conformite GDPR** (breach de donnees personnelles copropietaires) | Basse | Fort | Defense en profondeur : LUKS, Suricata IDS, CrowdSec WAF, fail2ban, 2FA, audit Lynis hebdomadaire, rkhunter quotidien, AIDE integrity monitoring |
| **Adoption prestataires** (entrepreneurs peu technophiles) | Moyenne | Moyen | Magic link PWA (zero installation, zero compte), interface mobile-first, rapport en 5 minutes |
| **Dependance Stripe/SEPA** (changement conditions, pannes service) | Basse | Moyen | Abstraction Payment adapter (hexagonal), fallback BankTransfer/Cash, idempotency keys pour resilience retry |
| **Complexity creep** (137k+ LOC Rust, 559 endpoints) | Moyenne | Moyen | Architecture hexagonale stricte, BDD comme spec vivante, refactoring continu assiste IA, zero dette technique |

---

## 15. Principes d'architecture

- **SOLID** : 5 principes appliques rigoureusement dans chaque couche. SRP (1 use case = 1 responsabilite), OCP (nouveaux adapters sans modifier le Domain), LSP (tous les repositories substituables), ISP (traits granulaires : `BuildingRepository`, `OwnerRepository`, pas de god-interface), DIP (Domain definit les ports, Infrastructure les implemente)
- **Architecture hexagonale stricte** : Domain (entites, services) -> Application (use cases, ports/traits, DTOs) -> Infrastructure (PostgreSQL repositories, Actix-web handlers, routes). Dependances toujours vers l'interieur. Le Domain n'a AUCUNE dependance externe.
- **DDD** : 59 entites domaine avec invariants dans les constructeurs (`::new() -> Result<Self, String>`). Bounded Contexts identifies et isoles. Ubiquitous Language belge (tantieme, quorum, syndic, etc.) utilise dans le code, les tests BDD, et la documentation.
- **BDD** : 819 scenarios Gherkin (69 features) comme specifications vivantes. Meme narratif entre BDD backend et E2E frontend (Test-Driven Emergence). Si le BDD passe mais le E2E echoue -> bug frontend.
- **TDD** : Test-first systematique pour le Domain. Couverture domain cible 100%. Pyramide de tests : unit (in-module `#[cfg(test)]`) -> integration (testcontainers PostgreSQL) -> BDD (Cucumber) -> E2E (Playwright) -> benchmarks (Criterion).
- **Securite by design** : RGPD des la conception (6 articles implementes). Chiffrement at-rest (LUKS). Defense en profondeur (IDS + WAF + fail2ban + rate limiting + 2FA + kernel hardening). Headers securite (HSTS 1 an, CSP, X-Frame-Options). Validation JWT strict (min 32 chars secret).
- **Green IT** : Cible < 0.5g CO2/requete (mesure actuelle 0.12g). Anti-bloatware : zero JavaScript inutile (Astro Islands), compilation Rust optimisee (LTO, codegen-units=1), connection pooling strict (max 10).
- **Test-Driven Emergence** : L'application emerge des tests, pas l'inverse. Le meme scenario metier est la source de verite unique exprimee a 3 niveaux : BDD integration -> E2E Documentation Vivante -> preuve video (YouTube/stakeholders).

---

## 16. Metriques de succes

| Metrique | Cible | Comment mesurer |
|----------|-------|----------------|
| Latence P99 | < 5ms | Prometheus + Grafana (endpoint `/metrics`) |
| Throughput | > 100k req/s | Load tests Criterion/k6 |
| Memoire par instance | < 128 MB | Prometheus `process_resident_memory_bytes` |
| Empreinte CO2 par requete | < 0.5g (actuel: 0.12g) | Green Metrics Tool + calcul OVH |
| Couverture tests domain | 100% | Tarpaulin (`make coverage`) |
| Scenarios BDD | 819 scenarios, 69 features | `cargo test --test bdd` |
| E2E smoke tests | 49 tests + 12 Doc Vivante | Playwright (`npm run test:e2e`) |
| Uptime production | 99.9% | Grafana + Alertmanager |
| Incidents P1 par mois | 0 | ITIL Incident Log |
| Conformite GDPR | 6/6 articles (15,16,17,18,21,30) | Audit checklist + tests automatises |
| Conformite legale belge | 80% (Jalon 2), 95% (Jalon 4), 100% (Jalon 5) | Checklist Article 577 CC |
| Coproprietes hebergees | J0: 10-20, J1: 50-100, J2: 200-500, J3: 500-1000 | Compteur organisations actives |
| Temps moyen convocation AG | < 5 min (vs 2h manuel) | Metriques application |
| Taux recouvrement automatise | > 85% avant niveau LegalAction | Dashboard comptable |
| Score Lynis securite | > 80/100 | Audit hebdomadaire automatise |

---

## Pipeline suivant

Ce brief sera consomme par :
- -> **Etape 2** : Product Manager (PRD avec scenarios BDD) -> `PRD.md`
- -> **Etape 3** : Architecte (architecture hexagonale SOLID) -> `architecture.md`
- -> **Etape 4** : Scrum Master (stories TDD pour agents IA) -> `epics-and-stories.md`
- -> **Etape 5** : Validation croisee -> `validation-report.md`

---

*Methode Maury — Par Gilles Maury & Farah Maury*
