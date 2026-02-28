# Audit de Conformite Juridique - KoproGo

**Date**: 2026-02-28
**Portee**: Droit belge de la copropriete (Art. 577 Code Civil), RGPD, Comptabilite belge (PCMN), Securite technique
**Score global**: **65% - NON PRET POUR PRODUCTION sans corrections Phase 1**

---

## Table des matieres

1. [Conformite Droit Belge de la Copropriete](#1-conformite-droit-belge-de-la-copropriete-art-577-code-civil)
2. [Conformite RGPD](#2-conformite-rgpd)
3. [Conformite Comptable Belge (PCMN)](#3-conformite-comptable-belge-pcmn)
4. [Securite Technique](#4-securite-technique)
5. [Documentation Legale Manquante](#5-documentation-legale-manquante)
6. [Plan d'Action Recommande](#6-plan-daction-recommande)
7. [Score de Conformite Global](#7-score-de-conformite-global)

---

## 1. Conformite Droit Belge de la Copropriete (Art. 577 Code Civil)

### 1.1 Convocations - CONFORME

| Exigence | Statut | Fichier |
|----------|--------|---------|
| Delai 15j AG ordinaire | OK | `backend/src/domain/entities/convocation.rs` |
| Delai 8j AG extraordinaire | OK | idem |
| Delai 8j 2eme convocation | OK | idem |
| Tracking email (envoi/ouverture) | OK | `convocation_recipient.rs` |
| Rappels J-3 automatiques | OK | idem |
| Support procuration | OK | `proxy_owner_id` field |
| Multi-langue FR/NL/DE/EN | OK | Validation dans `Convocation::new()` |
| Generation PDF | OK | `ConvocationExporter` service |

### 1.2 Systeme de Vote - CONFORME AVEC LACUNES

**Implemente :**
- 3 types de majorite : Simple (50%+1 exprimes), Absolue (50%+1 de tous), Qualifiee (seuil custom)
- Tantiemes/milliemes (0-1000) avec `voting_power: f64`
- Vote par procuration (`proxy_owner_id`)
- Audit trail complet (VoteCast, VoteChanged, VotingClosed)
- 27 tests unitaires

**LACUNES CRITIQUES :**

| Lacune | Risque | Ref. legale |
|--------|--------|-------------|
| **Pas de validation du quorum** | CRITIQUE - Les votes sont possibles meme sans quorum de 50% | Art. 577-6 par.5 |
| **Pas de workflow 2eme convocation** | CRITIQUE - Si quorum non atteint, pas de procedure automatique | Art. 577-6 par.5 |
| **Pas de limite de procurations** | CRITIQUE - Un mandataire peut representer un nombre illimite de coproprietaires | Art. 577-6 par.7 (max 3 mandats) |
| **Pas de presets majorite/type de decision** | MOYEN - Le syndic doit choisir manuellement la bonne majorite | Art. 577-6 par.6-9 |
| **Pas de fenetre temporelle de vote** | MOYEN - Les votes peuvent etre modifies apres la reunion | Pratique courante |
| **Pas de snapshot des tantiemes** | MOYEN - Si transfert pendant AG, les calculs seraient fausses | Tracabilite |
| **Pas de lien agenda-resolutions** | ELEVE - Des decisions hors agenda sont possibles (nulles en droit) | Art. 577-6 par.2 |

**Code concerne :**
- `backend/src/domain/entities/resolution.rs` (462 lignes) - manque `decision_type`, lien agenda
- `backend/src/domain/entities/vote.rs` (265 lignes) - manque validation fenetre temporelle, limite procurations
- `backend/src/application/use_cases/resolution_use_cases.rs` - manque verification quorum
- `backend/src/domain/entities/meeting.rs` (266 lignes) - manque `quorum_met: bool`

### 1.3 Proces-verbal (PV) d'AG - PARTIELLEMENT CONFORME

**Implemente :**
- Service de generation PDF (`meeting_minutes_exporter.rs`, 423 lignes)
- Contenu : infos batiment, participants, tantiemes, resolutions, resultats de vote

**LACUNES :**

| Lacune | Risque | Ref. legale |
|--------|--------|-------------|
| **Pas de workflow de distribution** | ELEVE - Le PV doit etre envoye a tous les coproprietaires sous 30 jours | Art. 577-6 par.10 |
| **Pas de signatures numeriques** | MOYEN - Le PV doit etre signe par le president, secretaire, et presents | Art. 577-6 par.10 |
| **Pas d'archivage permanent** | MOYEN - Le PV doit etre conserve dans le registre permanent | Pratique courante |

### 1.4 Quotes/Devis Entrepreneurs - CONFORME AVEC 1 LACUNE

**Implemente :** Workflow complet 7 etats, scoring automatique (prix 40%, delai 30%, garantie 20%, reputation 10%), TVA belge, garantie decennale, audit trail.

**LACUNE :** Pas d'enforcement "3 devis obligatoires pour travaux >5000 EUR" au niveau du workflow d'approbation.
- Fichier : `backend/src/application/use_cases/quote_use_cases.rs`

### 1.5 Quotes-parts (Tantiemes) - CONFORME

- Total = 100% par lot (tolerance +/-0.01%)
- Trigger PostgreSQL `validate_unit_ownership_total()`
- Conforme a Art. 577-2 par.4

### 1.6 Recouvrement des Impayes - CONFORME

- 4 niveaux d'escalade : Gentle (J+15) -> Formal (J+30) -> FinalNotice (J+45) -> LegalAction (J+60)
- Taux legal belge 8% annuel
- Support mise en demeure par lettre recommandee

### 1.7 Info Publique du Syndic - CONFORME

- 7 champs publics (nom, email, tel, adresse, horaires, urgences, slug)
- Endpoint public sans authentification (`GET /api/v1/public/buildings/{slug}/syndic`)

---

## 2. Conformite RGPD

### 2.1 Droits des Personnes Concernees - CONFORME

| Article | Droit | Endpoint | Statut |
|---------|-------|----------|--------|
| Art. 15 | Droit d'acces | `GET /gdpr/export` | OK |
| Art. 16 | Droit de rectification | `PUT /gdpr/rectify` | OK |
| Art. 17 | Droit a l'effacement | `DELETE /gdpr/erase` | OK |
| Art. 18 | Droit a la limitation | `PUT /gdpr/restrict-processing` | OK |
| Art. 21 | Droit d'opposition | `PUT /gdpr/marketing-preference` | OK |
| Art. 30 | Registre des traitements | `audit_logs` table | OK |

### 2.2 LACUNES RGPD

| Lacune | Risque | Impact |
|--------|--------|--------|
| **Pas de politique de confidentialite** | CRITIQUE | Art. 13-14 : les utilisateurs doivent etre informes |
| **Pas de consentement cookies** | CRITIQUE | Directive ePrivacy obligatoire en Belgique |
| **Pas de procedure de notification de violation** | ELEVE | Art. 33 : 72h pour notifier l'autorite (APD belge) |
| **Pas de DPA avec sous-traitants** | ELEVE | Art. 28 : Stripe, S3, fournisseur email |
| **Pas de chiffrement au repos des donnees personnelles en base** | ELEVE | Art. 32 : securite du traitement |
| **Pas de nettoyage automatique des logs** | MOYEN | Retention 7 ans calculee mais pas de cron job |
| **Pas de rate limiting sur endpoints RGPD** | MOYEN | Risque d'abus |
| **Pas de MFA obligatoire pour l'effacement** | MOYEN | Risque de suppression non autorisee |

---

## 3. Conformite Comptable Belge (PCMN)

### 3.1 Plan Comptable - CONFORME (95%)

- 90+ comptes pre-seedes (AR 12/07/2012)
- 9 classes (Actif, Passif, Charges, Produits, Hors-bilan)
- Multi-tenant (UNIQUE code + organization_id)
- Validation codes comptables et types de comptes

### 3.2 TVA Belge - CONFORME

- 3 taux supportes : 6% (renovations), 12% (intermediaire), 21% (standard)
- Calculs automatiques TTC
- Precision DECIMAL(12,2)

### 3.3 Rapports Financiers - CONFORME

- Bilan (Balance Sheet) + Compte de Resultats (Income Statement)
- Filtrage par periode
- Verification Assets = Liabilities

### 3.4 Workflow Factures - CONFORME

- Etats : Draft -> PendingApproval -> Approved/Rejected
- Multi-lignes avec TVA par ligne
- Permissions par role

---

## 4. Securite Technique

### 4.1 Points Forts (CONFORMES)

| Categorie | Implementation |
|-----------|---------------|
| Authentification JWT | Min 32 chars, refresh tokens, 2FA TOTP |
| Rate limiting login | 5 tentatives/15min par IP |
| Headers securite | HSTS 1an, CSP, X-Frame-Options DENY, Referrer-Policy |
| Prevention injection SQL | sqlx avec requetes parametrees |
| CORS | Validation origines, pas de wildcards |
| Hachage mots de passe | bcrypt avec salt |
| Chiffrement disque | LUKS AES-XTS-512 |
| Backups chiffres | GPG RSA-4096, S3 off-site |
| Detection intrusion | Suricata IDS + CrowdSec WAF |
| fail2ban | SSH, Traefik, API, PostgreSQL |
| Audit securite | Lynis hebdo, rkhunter quotidien, AIDE |
| Kernel hardening | SYN cookies, anti-IP spoofing, ASLR |

### 4.2 LACUNES Securite

| Lacune | Risque |
|--------|--------|
| CSP `unsafe-inline` (requis par Svelte) | FAIBLE - mitige par validation input |
| Pas de scanning images Docker (Trivy/Grype) | MOYEN |
| Pas de gestion de secrets (HashiCorp Vault) | MOYEN - utilise .env |
| Pas de test de penetration tiers | MOYEN pour production |

---

## 5. Documentation Legale Manquante

| Document | Statut | Obligatoire |
|----------|--------|-------------|
| Politique de confidentialite (FR/NL) | ABSENT | Oui (RGPD Art. 13-14) |
| Conditions generales d'utilisation | ABSENT | Oui (commerce electronique) |
| Politique cookies | ABSENT | Oui (Directive ePrivacy) |
| Mentions legales | ABSENT | Oui (Loi belge 11/07/2002) |
| DPA sous-traitants (Stripe, AWS, email) | ABSENT | Oui (RGPD Art. 28) |
| Plan de reponse aux incidents | ABSENT | Oui (RGPD Art. 33) |
| Documentation PCI-DSS | ABSENT | Recommande si paiements significatifs |

---

## 6. Plan d'Action Recommande

### Phase 1 : CRITIQUE (avant mise en production)

| # | Action | Effort estime | Fichiers concernes |
|---|--------|---------------|-------------------|
| 1 | **Validation du quorum pour les AG** | 3 jours | `meeting.rs`, `resolution_use_cases.rs`, nouvelle migration |
| 2 | **Limite des procurations** (max 3 mandats, % max voix) | 2 jours | `vote.rs`, `resolution_use_cases.rs` |
| 3 | **Workflow 2eme convocation** | 3 jours | `meeting.rs`, `convocation_use_cases.rs`, migration |
| 4 | **Lien agenda-resolutions** | 2 jours | `resolution.rs`, `resolution_use_cases.rs` |
| 5 | **Documentation legale** (politique confidentialite, CGU, mentions legales, cookies) | 5 jours | Frontend + docs/ |

**Total Phase 1 : ~15 jours de developpement**

### Phase 2 : ELEVE (avant 100 utilisateurs)

| # | Action | Effort estime |
|---|--------|---------------|
| 6 | Distribution PV sous 30 jours | 3 jours |
| 7 | Presets majorite par type de decision | 1 jour |
| 8 | Enforcement 3 devis pour >5000 EUR | 1 jour |
| 9 | Procedure notification violation RGPD | 2 jours |
| 10 | Nettoyage automatique logs d'audit | 1 jour |

**Total Phase 2 : ~8 jours de developpement**

### Phase 3 : MOYEN (avant 500 utilisateurs)

| # | Action | Effort estime |
|---|--------|---------------|
| 11 | Snapshot tantiemes au debut de l'AG | 2 jours |
| 12 | Fenetre temporelle de vote | 1 jour |
| 13 | Signatures numeriques pour PV | 3 jours |
| 14 | Consentement cookies frontend | 2 jours |
| 15 | Test de penetration tiers | Externe |
| 16 | Audit PCI-DSS | Externe |

**Total Phase 3 : ~8 jours + interventions externes**

---

## 7. Score de Conformite Global

| Domaine | Score | Commentaire |
|---------|-------|-------------|
| Droit copropriete belge (Art. 577) | **70%** | Solide mais lacunes AG critiques |
| Assemblees generales specifiquement | **55%** | Convocations OK, quorum/proxy/PV incomplets |
| RGPD | **65%** | Articles 15-21 OK, documentation/cookies absents |
| Comptabilite belge (PCMN) | **95%** | Quasi complet |
| Securite technique | **90%** | Excellence infrastructure, gaps mineurs |
| Documentation legale | **15%** | Documentation technique OK, legale absente |
| **GLOBAL** | **65%** | **NON PRET POUR PRODUCTION sans corrections Phase 1** |

---

## 8. Verification

Pour verifier les corrections une fois implementees :

```bash
# Tests unitaires domaine (quorum, proxy limits, majorites)
cargo test --lib

# Tests integration PostgreSQL (triggers, contraintes)
cargo test --test integration

# Scenarios BDD - 5 fichiers par domaine (752 scenarios au total)
cargo test --test bdd --test bdd_governance --test bdd_financial --test bdd_operations --test bdd_community

# Compilation sans warnings
cargo clippy -- -D warnings
```

> **Note importante** : Les 752 scenarios BDD ont ete ecrits mais n'ont pas tous ete
> executes dans un environnement d'integration complet. Des correctifs seront probablement
> necessaires lors des premieres executions CI. Les tests couvrent les lacunes documentees
> ci-dessus (quorum, procurations, 2eme convocation, etc.) mais en mode step-pending.

Une revue manuelle de la documentation legale par un juriste belge specialise en copropriete est **fortement recommandee** avant la mise en production.
