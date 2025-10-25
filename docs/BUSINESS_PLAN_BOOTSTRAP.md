# KoproGo - Business Plan ASBL 2025-2028

**Version** : 3.0 - ASBL Non-Lucrative / OpenCore
**Date** : Janvier 2025
**Statut** : ASBL Belge (Association Sans But Lucratif)
**Philosophie** : Side-project durable, qualité avant vitesse, impact social

---

## Table des Matières

1. [Executive Summary](#executive-summary)
2. [Vision et Mission](#vision-et-mission)
3. [Modèle OpenCore](#modèle-opencore)
4. [Équipe Bootstrap](#équipe-bootstrap)
5. [Stratégie de Croissance Organique](#stratégie-de-croissance-organique)
6. [Modèle Économique Bootstrap](#modèle-économique-bootstrap)
7. [Projections Réalistes](#projections-réalistes)
8. [Roadmap Produit OpenCore](#roadmap-produit-opencore)
9. [Stratégie Communautaire](#stratégie-communautaire)
10. [Risques et Opportunités](#risques-et-opportunités)

---

## Executive Summary

### Philosophie ASBL Side-Project

**Principe fondamental** : Impact social avant profit, qualité avant vitesse, durabilité avant croissance.

> "Nous construisons un bien commun, pas une licorne."

### Statut Juridique : ASBL Belge

**Association Sans But Lucratif** (loi belge 1921) :
- ✅ **Non-lucratif** : Tous les bénéfices réinvestis dans le projet
- ✅ **Exonération TVA** : Activités à caractère social
- ✅ **Transparence** : Assemblée générale annuelle, comptes publiés
- ✅ **Gouvernance** : Conseil d'administration bénévole
- ✅ **Mission sociale** : Démocratiser l'accès à la gestion de copropriété

### L'Équipe (Bénévole jusqu'à viabilité)

**2 personnes, 0.25 FTE équivalent** :

1. **Architecte Logiciel** (10-20h/semaine en side-project)
   - Emploi temps plein ailleurs (sécurité financière)
   - Architecture + développement core Rust
   - DevOps + infrastructure OVH
   - Vision produit long-terme
   - **Rythme** : Qualité > Vitesse, pas de burnout

2. **Étudiante en Informatique** (5-10h/semaine bénévole)
   - Formation pratique sur projet réel
   - Maintenance, documentation, tests
   - Community management GitHub
   - Contribution au CV professionnel
   - **Objectif** : Apprentissage + impact social

**Modèle de financement ASBL** :
- **Phase 1 (An 1-10)** : **100% bénévole** - Aucun service cloud payant
- **Financement futur** : Dons volontaires, subventions, sponsoring éthique (si besoin)
- **Pas de modèle SaaS** : 100% self-hosted, gratuité totale

**Principe** : L'ASBL ne distribue JAMAIS de dividendes. Tous les dons/subventions servent :
1. Infrastructure ASBL (domaine, services gratuits pour communauté)
2. Développement produit (bounties pour features prioritaires)
3. Événements communauté (meetups, conf annuelle)
4. Indemnités contributeurs réguliers (si budget suffisant)

### Modèle OpenCore ASBL (Hybride)

**Tout le code est MIT License** :
- ✅ Gestion immeubles, lots, copropriétaires
- ✅ Charges, paiements, comptabilité
- ✅ Assemblées générales (convocations, PV, votes)
- ✅ Gestion documents (upload, versioning)
- ✅ API REST complète
- ✅ Frontend complet (Astro + Svelte)
- ✅ Infrastructure as Code (Docker Compose, Traefik, CI/CD)
- ✅ **Aucune feature fermée, aucun code propriétaire**

**Option 1 : Self-Hosted GitOps (Gratuit)** 🔓
- ✅ **Installation 1-click** : Script automatique fourni
- ✅ **GitOps automatique** : GitHub Actions pour auto-déploiement
- ✅ **Updates automatiques** : Pull depuis dépôt GitHub à chaque release
- ✅ **Versioning géré** : Tags git = versions stables
- ✅ **Rollback facile** : Revenir à version précédente en 1 commande
- ✅ **100% gratuit** : Coût = VPS perso (~5-7€/mois) ou machine locale (0€)
- ✅ **Autonomie totale** : Vous contrôlez 100% de l'infrastructure
- ✅ **Données souveraines** : Sur VOTRE serveur

**Pour qui ?**
- Copropriétés avec un résident informaticien/DevOps
- Syndics ayant déjà un VPS/serveur
- Geeks qui veulent contrôle total

**Option 2 : Cloud ASBL (1€/copro/mois)** ☁️
- ✅ **Hébergement géré** : Infrastructure maintenue par l'ASBL bénévole
- ✅ **0 maintenance** : Mises à jour automatiques, backups, monitoring
- ✅ **Support email** : Réponse 48-72h (bénévole)
- ✅ **SSL/HTTPS** : Certificats gérés automatiquement
- ✅ **Uptime 99.7%+** : Infrastructure OVH France (datacenter bas carbone)
- ✅ **Même features** : 100% identique à self-hosted (code MIT)
- 💰 **Prix : 1€/copropriété/mois**

**Pour qui ?**
- Copropriétés sans compétences techniques
- Syndics qui veulent simplicité
- Petites copropriétés (< 20 lots) où VPS dédié = trop cher

**Principe ASBL Hybride** :
- ✅ **Code 100% ouvert** : Aucune feature premium fermée
- ✅ **Pas de lock-in** : Migration cloud → self-hosted en 1 commande
- ✅ **Prix coûtant** : 1€/copro couvre juste infra + 0€ bénévole
- ✅ **Transparence** : Comptes ASBL publiés annuellement
- ✅ **Revenus cloud → ASBL** : Finance développement, pas de profit privé

### Stratégie de Croissance (Ultra-Lente, Durable)

**0€ marketing** - 100% organique :
- ✅ **Open-source first** : GitHub, qualité du code exemplaire
- ✅ **Documentation exceptionnelle** : Tutorials, guides, vidéos
- ✅ **Bouche-à-oreille** : Produit excellent = recommandations naturelles
- ✅ **SEO long-terme** : Blog technique, cas d'usage
- ✅ **Communauté** : Discord/Matrix, support entraide

**Pas de publicité payante** : L'ASBL n'a pas les moyens, et ce n'est pas nécessaire.

### Projections Réalistes Side-Project (5 ans) - Modèle Hybride

**Hypothèse** : 10-20h/semaine, 2 bénévoles, croissance 5-10 copros cloud/mois

| Année | Cloud (1€/copro) | Self-hosted | Total Copros | MRR | Coûts | Excédent | Trésorerie | Équipe |
|-------|------------------|-------------|--------------|-----|-------|----------|------------|--------|
| **2025** | 20 | 80 | 100 | 20€ | 462€ (const.) + 8€/mois | -442€ | -442€ | 2 bénévoles |
| **2026** | 80 | 320 | 400 | 80€ | 8€/mois | 72€/mois | +422€ | 2 bénévoles |
| **2027** | 200 | 800 | 1,000 | 200€ | 8€/mois | 192€/mois | +2,726€ | 2 bénévoles |
| **2028** | 400 | 1,600 | 2,000 | 400€ | 16€/mois | 384€/mois | +7,334€ | 2 bénévoles |
| **2029** | 700 | 2,800 | 3,500 | 700€ | 16€/mois | 684€/mois | +15,542€ | **Indemnités possibles** |

**Détails projections** :

**Ratio cloud/self-hosted** : 20/80 (conservateur)
- 20% copros cloud (sans compétences tech)
- 80% self-hosted (avec informaticien ou VPS existant)
- Ratio inspiré Nextcloud, Plausible Analytics, Mastodon

**Revenus MRR** :
- 1€/copro/mois sur instances cloud uniquement
- Self-hosted = 0€ revenu (autonome)

**Coûts infrastructure** :
- An 1-3 : VPS Value OVH 7€ + domaine 1€ = 8€/mois (96€/an)
- An 4-5 : VPS Essential OVH 15€ + domaine 1€ = 16€/mois (192€/an)
- 1 vCPU suffit jusqu'à ~500 copros cloud (tests validés)

**Philosophie hybride** :
- **Cloud 1€/copro** : Viable économiquement, couvre juste infra
- **Self-hosted gratuit** : Adoption massive, communauté tech forte
- **Pas de pression** : Break-even Mois 12 An 2, rentabilité progressive
- **0€ salaires An 1-4** : Bénévolat pur, indemnités An 5 si trésorerie > 10k€
- **Impact social** : Des milliers de copros aidées (cloud + self-hosted)

---

## Structure Juridique : ASBL Belge

### Qu'est-ce qu'une ASBL ?

**ASBL** = Association Sans But Lucratif (loi belge du 27 juin 1921, réformée en 2019)

**Définition légale** : Une ASBL est une personne morale qui ne cherche pas à procurer un gain matériel à ses membres. Tous les bénéfices doivent être réinvestis dans l'objet social de l'association.

### Constitution de l'ASBL KoproGo

**Étapes de création** :

1. **Statuts** : Rédaction des statuts (objet social, gouvernance, membres)
   - Coût : 0€ (modèle standard adapté)
   - Temps : 1 semaine

2. **Acte authentique** : Passage devant notaire (obligatoire)
   - Coût : ~250€ (frais notaire réduits pour ASBL)
   - Temps : 1 journée

3. **Publication Moniteur** : Annonce légale au Moniteur belge
   - Coût : ~200€ (publication obligatoire)
   - Temps : 2-4 semaines

4. **Numéro d'entreprise** : BCE (Banque-Carrefour des Entreprises)
   - Coût : 0€ (automatique)
   - Temps : Immédiat après publication

5. **Compte bancaire ASBL** : Ouverture compte dédié
   - Coût : 0-10€/mois selon banque
   - Temps : 1 semaine

**Total création ASBL** : ~450-500€ + 1-2 mois

### Objet Social ASBL KoproGo

**Article 3 des Statuts** :

> "L'association a pour objet la **promotion de l'accès démocratique aux outils numériques de gestion de copropriété**, par le développement, la maintenance et la diffusion de logiciels libres et open-source, ainsi que la fourniture de services d'hébergement et de support à prix coûtant.
>
> L'ASBL poursuit un but d'**intérêt général** et d'**éducation populaire** en :
> - Rendant accessible la technologie de gestion immobilière à tous, sans discrimination économique
> - Favorisant la transparence et l'auditabilité par l'open-source
> - Formant des bénévoles et étudiants aux pratiques de développement logiciel professionnel
> - Réduisant l'empreinte écologique de l'hébergement numérique par des choix d'infrastructure responsables"

### Gouvernance ASBL

#### Assemblée Générale (AG)

**Composition** : Tous les membres de l'ASBL

**Membres fondateurs** :
- Architecte logiciel (fondateur)
- Étudiante informatique (fondatrice)
- +1 membre externe (pour diversité, ex: juriste ou comptable bénévole)

**Cotisation membres** : 0€ (pas de cotisation, ASBL accessible)

**Pouvoirs de l'AG** :
- Modification des statuts
- Nomination/révocation administrateurs
- Approbation comptes annuels
- Dissolution de l'ASBL

**Fréquence** : 1x/an minimum (obligatoire), + AG extraordinaires si besoin

#### Conseil d'Administration (CA)

**Composition** : 3 administrateurs minimum (loi belge)

**Administrateurs KoproGo** :
- Architecte logiciel (Président)
- Étudiante informatique (Secrétaire)
- Membre externe (Trésorier)

**Mandat** : 4 ans renouvelables

**Rémunération** : **0€** (bénévolat pur)

**Pouvoirs du CA** :
- Gestion quotidienne de l'ASBL
- Décisions stratégiques (roadmap, investissements)
- Recrutement/indemnisation contributeurs (si trésorerie suffisante)
- Représentation de l'ASBL

**Fréquence réunions** : Trimestrielles (4x/an) + ad-hoc si urgent

#### Gestion Journalière

**Délégation possible** : Le CA peut déléguer la gestion journalière à un ou plusieurs gestionnaires

**KoproGo** : Architecte logiciel = gestionnaire journalier délégué
- Décisions opérationnelles (infrastructure, déploiements)
- Engagement dépenses < 500€ (au-delà : validation CA)
- Représentation ASBL (contrats fournisseurs, partenariats)

### Obligations Légales ASBL

#### Comptabilité

**Comptabilité simplifiée** (si revenus < 500k€/an) :
- Livre journal des recettes/dépenses
- Inventaire annuel actifs/passifs
- Budget prévisionnel annuel

**Comptabilité double** (si revenus > 500k€/an) :
- Plan comptable normalisé
- Bilan + compte de résultats
- Expert-comptable recommandé

**KoproGo Phase 1-5** : Comptabilité simplifiée (revenus < 50k€/an)

**Coût** : 0€ (géré en interne) ou 300-500€/an si comptable externe

#### Comptes Annuels

**Publication obligatoire** :
- Dépôt à la Banque Nationale de Belgique (BNB)
- Délai : 6 mois après clôture exercice
- Accessibles au public (transparence)

**Approbation** : Assemblée Générale annuelle

#### TVA

**Exonération TVA possible** si activités à caractère **social, éducatif ou culturel**

**KoproGo** : Exonération probable pour :
- Fourniture logiciel open-source (gratuit)
- Hébergement à prix coûtant (non lucratif)

**Si exonération refusée** : TVA 21% sur services cloud (prix TTC ajustés)

#### Impôt sur les Sociétés (ISOC)

**Principe ASBL** : Exonération ISOC si :
- Activités conformes à l'objet social (non lucratif)
- Pas de distribution de bénéfices
- Activités principalement désintéressées

**KoproGo** : Exonération probable (mission sociale prédominante)

**Si activités commerciales significatives** : ISOC sur profits commerciaux uniquement (revenus cloud)

### Transparence Financière ASBL

**Engagement KoproGo** :

1. **Comptes publics annuels** : Publiés sur GitHub + site web
2. **Budget prévisionnel** : Partagé avec communauté en début d'année
3. **Rapport d'activité** : Annuel, détaillant usage des fonds
4. **Dashboard temps réel** : Revenus, coûts, trésorerie (mise à jour trimestrielle)

**Exemple de transparence** :
- Mozilla Foundation : Publie budget complet + salaires dirigeants
- Wikimedia : Dashboard financier public temps réel
- Document Foundation : Comptes annuels + rapports activité détaillés

### Rémunération dans l'ASBL

**Principes légaux belges** :

1. **Administrateurs** : Pas de rémunération (sauf remboursement frais réels)
2. **Bénévoles** : Indemnités forfaitaires autorisées (max ~40€/jour, 2,000€/an, plafonds 2025)
3. **Salariés** : Rémunération normale possible si :
   - Liée à l'exécution de l'objet social
   - Justifiée par travail effectif
   - Approuvée par AG
   - Proportionnée aux capacités financières ASBL

**KoproGo Timeline** :

**Années 1-3** : **0€ rémunération**
- Administrateurs bénévoles
- Pas d'indemnités
- Chacun a activité externe pour vivre

**Année 4** : **Indemnités forfaitaires** (si trésorerie > 10k€)
- 500€/mois max par contributeur actif (dans limites légales)
- Non soumis cotisations sociales (statut indemnité bénévole)
- Décision AG + validation CA

**Année 5+** : **Salaires partiels** (si trésorerie > 30k€)
- Contrats de travail ASBL (temps partiel)
- 1,500€/mois brut (mi-temps)
- Cotisations sociales classiques
- Décision AG + approbation budget

### Avantages Fiscaux Donateurs

**Si reconnaissance "établissement d'utilité publique"** (après 3-5 ans d'activité) :

**Donateurs particuliers** :
- Déduction fiscale 45% du don (min 40€/an)
- Exemple : Don 100€ = 45€ réduction impôt

**Donateurs entreprises** :
- Déduction à 120% du don (sponsoring déductible)
- Exemple : Don 1,000€ = 1,200€ déduction bénéfice imposable

**KoproGo Phase 1-3** : Pas encore de reconnaissance, donc pas de déductions fiscales.

**KoproGo Phase 4+** : Demande reconnaissance utilité publique (si activité prouvée)

### Dissolution ASBL

**En cas d'échec ou fin de mission** :

1. **Décision AG** : Dissolution votée à majorité qualifiée (2/3)
2. **Liquidation** : Remboursement dettes, vente actifs
3. **Boni de liquidation** : **Interdit de distribuer aux membres**
4. **Attribution** : Actifs restants donnés à ASBL similaire ou d'utilité publique

**KoproGo** : En cas de dissolution, code source reste MIT (open-source perpétuel), infrastructure donnée à autre ASBL tech sociale (ex: Framasoft).

---

## Vision et Mission

### Vision

Créer un **bien commun numérique** pour la gestion de copropriété, accessible à tous, maintenu par une communauté, libre et pérenne.

### Mission

**Démocratiser l'accès à la gestion de copropriété** en prouvant qu'un modèle **ASBL + OpenCore + Side-Project** peut :
1. Servir l'intérêt général plutôt que des actionnaires
2. Offrir une qualité exceptionnelle sans course à la croissance
3. Être viable économiquement sans exploitation
4. Créer de la valeur sociale plutôt que boursière

### Valeurs ASBL

1. **🏛️ Intérêt Général** : Mission sociale avant profit privé
2. **🔓 Open Source** : Code MIT, transparence totale, auditabilité
3. **🐢 Durabilité** : Lent mais solide, 10+ ans de vision
4. **⚡ Excellence** : Qualité technique sans compromis
5. **🤝 Communauté** : Gouvernance partagée, décisions collectives
6. **🌱 Écologie** : Infrastructure bas carbone (OVH France, 0.12g CO₂/req)
7. **💚 Bénévolat** : Contribution par passion, pas par obligation

---

## Modèle OpenCore ASBL

### Qu'est-ce qu'OpenCore pour une ASBL ?

**Définition** : Le **core** du produit est **100% open-source (MIT)**, et les **services d'hébergement cloud** sont **payants** pour financer l'ASBL.

**Différence ASBL vs Startup** :
| Aspect | Startup OpenCore | ASBL OpenCore (KoproGo) |
|--------|------------------|-------------------------|
| **Code** | Core OSS, Premium fermé | **100% open-source MIT** |
| **Profits** | Actionnaires, dividendes | **100% réinvestis mission sociale** |
| **Objectif** | Valorisation, exit IPO | **Pérennité, impact social** |
| **Gouvernance** | VC board, CEO | **AG démocratique, CA bénévole** |
| **Vitesse** | Hypercroissance obligée | **Croissance durable side-project** |
| **Salaires** | Dès J0 (funding) | **Après 3-4 ans (si trésorerie)** |
| **Marketing** | Ads, sales force | **100% organique, communauté** |

**Exemples ASBL/Non-Profit tech réussis** :
- **Mozilla Foundation** : Firefox, Thunderbird (~$500M/an budget, rentable depuis 20+ ans)
- **Wikimedia** : Wikipedia (~$150M/an, dons + services, 300M+ utilisateurs)
- **Document Foundation** : LibreOffice (communauté 700+ devs, certifications payantes)
- **Blender Foundation** : Logiciel 3D (cloud rendering payant, industrie Hollywood)
- **Framasoft** : Suite outils open-source France (dons + services, 0 pub)

### KoproGo Core (Open-Source)

**License** : MIT (permissive)

**Fonctionnalités Core** :
```
✅ Gestion immeubles (CRUD complet)
✅ Gestion lots/unités
✅ Gestion copropriétaires (GDPR compliant)
✅ Gestion charges et répartition
✅ Suivi paiements (manuel)
✅ Assemblées générales (convocations, PV)
✅ Gestion documents
✅ API REST complète
✅ Self-hosting (Docker Compose)
✅ Exports données (CSV, JSON, SQL)
```

**Pourquoi open-source le core ?**
- ✅ Adoption large (0 friction)
- ✅ Confiance maximale (code auditable)
- ✅ Contributions communauté (features gratuites)
- ✅ Anti-lock-in (USP majeur vs concurrence)
- ✅ SEO/Visibilité (GitHub stars, HackerNews)

### KoproGo Cloud (Payant pour financer l'ASBL)

**Modèle ASBL : Service à prix coûtant**

**Services Cloud inclus** :
```
✅ Hébergement géré OVH France (datacenter bas carbone)
✅ Sauvegardes quotidiennes automatiques
✅ Mises à jour gratuites (rolling updates sans downtime)
✅ Support email (délai 48-72h)
✅ SSL/TLS inclus (sécurité)
✅ Monitoring uptime (99.7%+ garanti)
✅ Exports données (CSV, JSON, SQL)
✅ GDPR compliance (données EU)
```

**Pourquoi payer l'hébergement ASBL ?**
- ✅ **Gain de temps** : 0 gestion serveur, 0 maintenance
- ✅ **Sécurité** : Backups + SSL + monitoring professionnels
- ✅ **Écologie** : Infrastructure bas carbone (0.12g CO₂/req)
- ✅ **Soutien ASBL** : Financer développement open-source
- ✅ **Éthique** : Prix coûtant, transparence financière totale

### Pricing ASBL (Unique et Simple)

**Self-Hosted (Gratuit à vie)** :
- ✅ Core complet MIT open-source
- ✅ Docker Compose fourni (déploiement 1-click)
- ✅ Documentation complète
- ✅ Support communauté (Discord, GitHub Issues)
- ✅ Updates régulières (pull git)
- ✅ **Aucune limitation fonctionnelle**
- **Prix : 0€ pour toujours**

**Cloud ASBL (Prix coûtant)** :
- ✅ Hébergement géré + tous services cloud ci-dessus
- ✅ Toutes les fonctionnalités (100% des features)
- ✅ Nombre illimité de copropriétés
- ✅ Utilisateurs illimités
- **Prix : 1€/copropriété/mois**

**Exemples pricing** :
- 5 copropriétés : **5€/mois** (60€/an)
- 20 copropriétés : **20€/mois** (240€/an)
- 100 copropriétés : **100€/mois** (1,200€/an)

**Pas de paliers, pas de premium, pas de surprises** : Philosophie ASBL = équité et transparence.

### Avantages OpenCore

**Pour les utilisateurs** :
- ✅ Essai gratuit illimité (self-hosted)
- ✅ Pas de lock-in (code source accessible)
- ✅ Upgrade optionnel quand besoin
- ✅ Confiance (transparence totale)

**Pour KoproGo** :
- ✅ Acquisition low-cost (communauté)
- ✅ Feedback rapide (issues GitHub)
- ✅ Contributions gratuites (PRs)
- ✅ Crédibilité (anti-vendor lock-in)
- ✅ Différenciation unique vs concurrence

---

## Stratégie de Croissance Organique

### Phase 1 : Lancement Communauté (Mois 1-6)

**Objectif** : 1,000 stars GitHub, 20 copropriétés actives

**Actions** :

**Open-Source Launch** :
1. Publier code GitHub (license MIT)
2. README exceptionnel (captures, démo live)
3. Documentation complète (setup, API, contribution)
4. Issues templates (bugs, features)

**Community Building** :
1. Discord/Slack communauté
2. Forum support (Discourse self-hosted ou GitHub Discussions)
3. Contributing guidelines claires
4. Good first issues labelées

**Content Marketing** :
1. Blog posts :
   - "Why we built KoproGo in Rust"
   - "OpenCore vs SaaS : Our journey"
   - "Self-hosting guide"
2. Dev.to, Medium cross-posts
3. HackerNews launch post

**Réseau Social** (gratuit) :
1. LinkedIn posts (insights marché copropriété)
2. Twitter tech posts (#rustlang, #opensource)
3. Reddit : r/rust, r/selfhosted, r/opensource

**Résultats attendus** :
- 1,000 GitHub stars
- 50 self-hosted instances
- 20 copropriétés cloud payantes
- 5 contributors externes
- 100€ MRR

### Phase 2 : Traction Organique (Mois 7-18)

**Objectif** : 3,000 stars, 150 copropriétés, 800€ MRR

**Actions** :

**SEO Long-tail** :
1. Blog 2x/semaine :
   - Guides pratiques ("Comment gérer une AG")
   - Comparatifs ("KoproGo vs Vilogi")
   - Tutorials ("Self-host en 10 minutes")
2. Top 10 Google pour 10 mots-clés :
   - "logiciel syndic open source"
   - "gestion copropriété gratuit"
   - "alternative vilogi"

**Bouche-à-Oreille** :
1. NPS > 60 (produit exceptionnel)
2. Referral program : 1 mois gratuit par parrainage
3. Case studies clients (avec permission)

**Partenariats** :
1. Associations copropriétaires BE/TN
2. Agences immobilières (recommandations)
3. Notaires (mentions)

**Ads Micro-Budget** (si rentable) :
1. Google Ads : 100€/mois (mots-clés long-tail)
2. Facebook TN : 50€/mois

**Communauté** :
1. 20+ contributors externes
2. 100+ PRs mergées
3. Meetup communauté (virtuel)

**Résultats attendus** :
- 3,000 GitHub stars
- 200 self-hosted instances
- 50 clients cloud BE
- 10 clients cloud TN
- 30 contributors
- 800€ MRR

### Phase 3 : Scale Organique (Mois 19-36)

**Objectif** : 10k stars, 1,000 copropriétés, 4,000€ MRR

**Actions** :

**Expansion Géographique** :
1. France (communauté FR existante)
2. Algérie/Maroc (depuis TN)
3. Luxembourg (premium)

**Product-Led Growth** :
1. Onboarding self-service parfait
2. Time-to-value < 30 minutes
3. In-app upgrade prompts (subtils)

**Partnerships Scale** :
1. Intégrations comptables (API partenaires)
2. Distributeurs (comptables, agents)
3. White-label pour réseaux agences

**Ads Scale** (si très rentable) :
1. Google Ads : 300€/mois
2. LinkedIn Ads : 200€/mois (syndics pro)

**Résultats attendus** :
- 10,000 GitHub stars
- 500 self-hosted instances
- 150 clients cloud
- 100+ contributors
- 4,000€ MRR

### Canaux d'Acquisition (0€ → Low-Cost)

**Année 1 (0€ marketing)** :
- ✅ GitHub (communauté open-source)
- ✅ SEO organique (blog 2x/semaine)
- ✅ Réseaux sociaux (LinkedIn, Twitter, Reddit)
- ✅ Bouche-à-oreille (NPS > 60)
- ✅ Partenariats (associations, gratuit)

**Année 2 (150€/mois marketing)** :
- Google Ads : 100€/mois
- Facebook TN : 50€/mois
- + Tous canaux Année 1

**Année 3 (500€/mois marketing)** :
- Google Ads : 300€/mois
- LinkedIn Ads : 200€/mois
- + Tous canaux précédents

**CAC (Customer Acquisition Cost)** :
- Année 1 : ~10€ (quasi 100% organique)
- Année 2 : ~15€ (mix organique + ads)
- Année 3 : ~25€ (scale ads)

vs Business Plan v1 : 75€ CAC

**Advantage** : CAC ultra-low = profitabilité immédiate

---

## Modèle Économique ASBL (Hybride Cloud + Self-Hosted)

### Structure de Coûts (Ultra-Minimale)

**Coûts Fixes ASBL** :

| Année | Constitution | VPS OVH | Domaine | Compte banque | Total |
|-------|--------------|---------|---------|---------------|-------|
| **2025** | 450€ | 84€ (7€×12) | 12€ | 0€ | **546€** |
| **2026** | 0€ | 84€ | 12€ | 0-120€ | **96-216€** |
| **2027** | 0€ | 84€ | 12€ | 120€ | **216€** |
| **2028** | 0€ | 180€ (15€×12) | 12€ | 120€ | **312€** |
| **2029** | 0€ | 180€ | 12€ | 120€ | **312€** |

**Détails** :
- **VPS OVH France** :
  - An 1-3 : VPS Value 7€ TTC/mois (1 vCore, 2GB, 40GB NVMe)
  - An 4-5 : VPS Essential 15€ TTC/mois (2 vCore, 4GB, 80GB NVMe)
  - Héberge instances cloud multi-tenant
- **Domaine** : koprogo.com ~12€/an
- **Compte bancaire ASBL** : 0-10€/mois selon banque
- **Marketing** : 0€ (100% organique)
- **Salaires** : 0€ An 1-4 (bénévolat), indemnités An 5 si trésorerie > 10k€

**Infrastructure self-hosted** : 0€ coût ASBL (chaque utilisateur gère son VPS/serveur)

### Revenus ASBL

**Source 1 : Cloud ASBL (1€/copro/mois)** :
- Uniquement copros qui choisissent hébergement géré ASBL
- Prix coûtant : couvre juste infra + buffer incidents
- Pas de profit privé : excédents réinvestis ASBL

**Source 2 : Dons volontaires** :
- GitHub Sponsors, Open Collective, Liberapay
- Utilisateurs self-hosted satisfaits peuvent donner
- 0€ attendu, bonus bienvenu

**Source 3 : Subventions (An 3+)** :
- Horizon Europe (EU) : 10-50k€/projet si éligible
- Digital Wallonia (BE) : 5-20k€/an
- Fondations open-source : Mozilla, Sloan, etc.

### Unit Economics Cloud ASBL

**LTV (Lifetime Value)** :
```
1€/copro/mois × durée vie moyenne
- Churn : 5%/an (très faible, besoin réel)
- Durée vie = 1 / 0.05 = 20 ans
- LTV = 1€ × 12 mois × 20 ans = 240€ par copro

Conservateur (10 ans) : LTV = 120€
```

**CAC (Customer Acquisition Cost)** :
```
0€ marketing → CAC = 0€

Temps bénévole si compté :
- 1h onboarding/copro × 0€ (bénévole) = 0€
- CAC réaliste = ~5€ (si on valorise temps)
```

**LTV/CAC** :
```
240€ / 5€ = 48:1 (exceptionnel)
Ou 240€ / 0€ = ∞ (théorique)

Target SaaS classique : 3:1
KoproGo ASBL : 48:1 ✅
```

**Payback Period** :
```
CAC / MRR par copro = 5€ / 1€ = 5 mois
Target SaaS : < 12 mois
KoproGo : 5 mois ✅
```

**Gross Margin** :
```
Revenus 1€/copro/mois
Coûts variables : ~0.01€/copro (compute)
Marge brute : 99% ✅
```

**Churn** :
```
Année 1 : 10% (early adopters, tests)
Année 2+ : 5%/an (produit mature)
Target SaaS : < 10%/an
KoproGo : 5% ✅
```

**Conclusion Unit Economics** :
- ✅ **Très sains** même à 1€/copro (prix coûtant)
- ✅ **Scalabilité** : 1 vCPU = 1,000-1,500 copros (marge énorme)
- ✅ **Pas de pression** : Break-even An 2, croissance lente OK

---

## Projections Réalistes

### Hypothèses Conservatives

**Croissance** :
- Mois 1-6 : +10%/mois (lancement lent)
- Mois 7-18 : +20%/mois (traction organique)
- Mois 19-36 : +15%/mois (maturité)

vs Business Plan v1 : 30-50%/mois (avec ads)

**Conversion** :
- Self-hosted → Cloud : 10-15%
- Freemium → Paid : Pas applicable (OpenCore)
- Trial → Paid : Pas de trial (direct cloud payant)

**Retention** :
- Churn : 5%/an (vs 6% BP v1)
- NPS : 60+ (produit exceptionnel + communauté)

### Projections Année 1 (2025)

| Mois | Clients Cloud | Copros | MRR | Coûts | Profit | Cumul |
|------|---------------|--------|-----|-------|--------|-------|
| Jan  | 0             | 5      | 0€  | 5€    | -5€    | -5€   |
| Fév  | 1             | 10     | 10€ | 5€    | 5€     | 0€    |
| Mar  | 2             | 15     | 25€ | 5€    | 20€    | 20€   |
| Avr  | 2             | 20     | 30€ | 5€    | 25€    | 45€   |
| Mai  | 3             | 25     | 40€ | 5€    | 35€    | 80€   |
| Juin | 4             | 30     | 55€ | 5€    | 50€    | 130€  |
| Juil | 5             | 35     | 70€ | 5€    | 65€    | 195€  |
| Août | 6             | 40     | 90€ | 5€    | 85€    | 280€  |
| Sept | 7             | 45     | 110€| 5€    | 105€   | 385€  |
| Oct  | 8             | 50     | 135€| 5€    | 130€   | 515€  |
| Nov  | 9             | 55     | 160€| 5€    | 155€   | 670€  |
| Déc  | 10            | 60     | 190€| 5€    | 185€   | 855€  |

**Total Année 1** :
- Clients fin année : 10
- Copropriétés : 60
- MRR fin année : 190€
- ARR : 2,280€
- Coûts : 60€
- **Profit : 1,680€** ✅ Rentable dès Mois 2
- Self-hosted (estimation) : 100 instances

**Répartition profits Année 1 (1,680€)** :
- Architecte (40%) : 672€
- Solo BE (30%) : 504€
- Solo TN (30%) : 504€

*Bonus symbolique, chacun garde activité externe*

### Projections Année 2 (2026)

| Trim. | Clients | Copros | MRR | Coûts | Profit Trim. | Cumul |
|-------|---------|--------|-----|-------|--------------|-------|
| Q1    | 20      | 120    | 400€| 545€  | 655€         | 1,510€|
| Q2    | 35      | 210    | 700€| 545€  | 1,555€       | 3,065€|
| Q3    | 55      | 330    | 1,100€| 545€| 3,065€       | 6,130€|
| Q4    | 80      | 480    | 1,600€| 545€| 5,085€       | 11,215€|

**Total Année 2** :
- Clients fin année : 80
- Copropriétés : 480
- MRR fin année : 1,600€
- ARR : 19,200€
- Coûts : 1,980€ (infra + marketing)
- **Profit : 12,240€**
- Self-hosted : 300+ instances

**Répartition profits Année 2 (12,240€)** :
- Architecte (40%) : 4,896€ (~408€/mois)
- Solo BE (30%) : 3,672€ (~306€/mois)
- Solo TN (30%) : 3,672€ (~306€/mois)

*Complément de revenu significatif*

### Projections Année 3 (2027)

| Trim. | Clients | Copros | MRR | Coûts | Profit Trim. | Cumul |
|-------|---------|--------|-----|-------|--------------|-------|
| Q1    | 120     | 720    | 2,400€| 3,530€| 3,670€   | 14,885€|
| Q2    | 180     | 1,080  | 3,600€| 3,530€| 7,270€   | 22,155€|
| Q3    | 250     | 1,500  | 5,000€| 3,530€| 11,470€  | 33,625€|
| Q4    | 350     | 2,100  | 7,000€| 3,530€| 17,470€  | 51,095€|

**Total Année 3** :
- Clients fin année : 350
- Copropriétés : 2,100
- MRR fin année : 7,000€
- ARR : 84,000€
- Coûts : 42,360€ (infra + marketing + **salaires**)
- **Profit : 39,865€**
- Self-hosted : 1,000+ instances

**Salaires fixes Année 3** :
- Architecte : 1,500€/mois (18,000€/an)
- Solo BE : 750€/mois (9,000€/an)
- Solo TN : 750€/mois (9,000€/an)
- **Total : 36,000€/an**

**Répartition profits supplémentaires (39,865€)** :
- Architecte (40%) : 15,946€
- Solo BE (30%) : 11,960€
- Solo TN (30%) : 11,960€

**Revenus totaux Année 3 par personne** :
- Architecte : 18,000€ + 15,946€ = **33,946€** (~2,829€/mois)
- Solo BE : 9,000€ + 11,960€ = **20,960€** (~1,747€/mois)
- Solo TN : 9,000€ + 11,960€ = **20,960€** (~1,747€/mois)

**🎯 Objectif atteint** : Vivre à temps plein de KoproGo dès Année 3 !

### Synthèse 3 Ans

| Métrique | 2025 | 2026 | 2027 |
|----------|------|------|------|
| **Clients Cloud** | 10 | 80 | 350 |
| **Copropriétés** | 60 | 480 | 2,100 |
| **Self-Hosted** | 100 | 300 | 1,000 |
| **MRR** | 190€ | 1,600€ | 7,000€ |
| **ARR** | 2,280€ | 19,200€ | 84,000€ |
| **Coûts** | 60€ | 1,980€ | 42,360€ |
| **Profit** | 1,680€ | 12,240€ | 39,865€ |
| **Cash Cumul** | 855€ | 11,215€ | 51,095€ |
| **GitHub Stars** | 1,000 | 3,000 | 10,000 |

**Break-even** : Mois 2 (Février 2025) ✅

**Viabilité temps plein** : Année 3 (2027) ✅

**0€ financement externe** : Toujours ✅

---

## Roadmap Produit Side-Project

### Philosophie Roadmap ASBL

**Principe** : Qualité > Vitesse, Features essentielles > Gadgets, Stabilité > Innovation à tout prix

**Rythme** : 10-20h/semaine (side-project durable, pas de burnout)

**Priorités** :
1. **Fiabilité** : Code testé, production-ready
2. **Documentation** : Tout est documenté (self-service)
3. **Communauté** : Contributions externes encouragées
4. **Simplicité** : Fonctionnalités core, pas de bloatware

### Année 1 (2025) - Core Open-Source Solide

**Trimestre 1-2 (6 mois) : Foundation**
- ✅ Core complet open-source MIT (GitHub public)
- ✅ CRUD complet : Immeubles, Lots, Copropriétaires, Charges, AG
- ✅ API REST complète + documentation Swagger
- ✅ Tests : Unitaires (100% couverture domain) + intégration + E2E
- ✅ Docker Compose 1-click (self-hosting facile)
- ✅ Documentation exhaustive (architecture, setup, contribution)
- ✅ CI/CD GitHub Actions (tests auto, release automatique)

**Trimestre 3-4 (6 mois) : Cloud MVP**
- Plateforme cloud basique (inscription, login, multi-tenant)
- Paiements Stripe simple (1€/copro/mois)
- Backups automatiques quotidiens (PostgreSQL dump)
- Monitoring uptime (UptimeRobot gratuit)
- Support email basique (réponse 48-72h)

**Objectif Année 1** : Produit **utilisable et stable**, prêt pour early adopters

**Features NON prioritaires An 1** : Mobile app, intégrations, AI, analytics avancé

### Année 2 (2026) - Stabilité & Communauté

**Semestre 1 : Polish & UX**
- Amélioration UX frontend (feedback early adopters)
- Génération documents PDF (quittances, PV assemblées)
- Exports comptables (CSV, Excel)
- Notifications email (paiements, échéances)
- Traductions (NL, EN) - contributeurs communauté

**Semestre 2 : Features Utilisateurs**
- Multi-utilisateurs avec rôles simples (admin, membre, lecteur)
- Gestion documents (upload, versioning)
- Calendrier assemblées générales
- Amélioration reporting (tableaux de bord)

**Communauté Année 2** :
- 20+ contributors externes (bugfixes, traductions, features)
- GitHub : 3,000 stars, 50+ PRs mergées
- Discord : 500+ membres actifs

**Objectif Année 2** : Produit **mature et complet** pour marché Belgique/France

### Année 3 (2027) - Scale Qualité

**Semestre 1 : Advanced Features**
- Mobile app (Progressive Web App d'abord, puis React Native si demande)
- Intégrations comptables basiques (exports formats Odoo, Sage)
- Webhooks API (events pour intégrations tierces)
- Amélioration performance (DragonflyDB cache si besoin)

**Semestre 2 : Enterprise Light**
- White-label basique (logo personnalisé, domaine custom)
- SSO simple (Google OAuth, Microsoft)
- API v2 améliorée (GraphQL optionnel)
- Advanced reporting (dashboards personnalisables)

**Communauté Année 3** :
- 100+ contributors
- Plugin system communautaire (extensions)
- Meetup annuel KoproGo (si budget)

**Objectif Année 3** : Produit **enterprise-ready** pour grands syndics et bailleurs

### Année 4-5 (2028-2029) - Maturité & Expansion

**Possible features (si demande marché)** :
- AI predictions (détection anomalies charges)
- IoT integration (compteurs intelligents)
- Mobile apps natives (iOS + Android stores)
- Multi-régions (datacenters EU multiples)
- Compliance avancée (certifications ISO)

**Principe** : Les features Années 4-5 seront **dictées par les utilisateurs**, pas par une roadmap imposée.

### Comparaison Side-Project vs Startup

| Aspect | Startup (Année 1) | Side-Project ASBL (Année 1) |
|--------|-------------------|----------------------------|
| **Features livrées** | 50+ features MVP | 10-15 features core **solides** |
| **Vélocité** | Rapide mais bugs | **Lente mais stable** |
| **Dette technique** | Élevée (rush) | **Minimale (qualité)** |
| **Tests** | Partiels | **100% couverture critique** |
| **Documentation** | Minimale | **Exhaustive** |
| **Burnout risque** | Élevé | **Faible** |

**Choix ASBL** : **Moins de features, mais qualité irréprochable** = Meilleure rétention, moins de churn, communauté fidèle.

---

## Stratégie Communautaire

### GitHub Community

**Objectifs** :
- Année 1 : 1,000 stars
- Année 2 : 3,000 stars
- Année 3 : 10,000 stars

**Actions** :

**Excellent README** :
- Captures d'écran professionnelles
- Démo live (Vercel preview)
- Quick start 1-2-3
- Badges (CI, tests, license)

**Documentation** :
- Setup guides (self-host, cloud)
- API documentation (Swagger/OpenAPI)
- Architecture decisions records (ADR)
- Contributing guide

**Issues Management** :
- Templates (bug, feature request)
- Labels clairs (good first issue, help wanted)
- Réponse < 24h
- Roadmap publique (GitHub Projects)

**Pull Requests** :
- Review < 48h
- Tests requis (CI)
- Code of Conduct
- Contributors recognition (all-contributors)

### Discord/Forum Community

**Channels** :
- #general (discussions)
- #support (entraide)
- #development (tech discussions)
- #feature-requests (feedback)
- #showcase (success stories)
- #belgium, #tunisia (locaux)

**Modération** :
- Les 3 core maintainers
- Community moderators (bénévoles)

**Events** :
- Monthly community calls (Zoom)
- Yearly meetup (si budget)

### Content Strategy (SEO)

**Blog 2x/semaine** :

**Thèmes** :
1. **Guides pratiques** (SEO)
   - "Comment organiser une AG en 2025"
   - "Répartition charges : Guide complet"

2. **Tech insights** (Dev.to, HN)
   - "Why we chose Rust for KoproGo"
   - "Hexagonal architecture in practice"
   - "Self-hosting SaaS: Lessons learned"

3. **Comparatifs** (SEO compétitif)
   - "KoproGo vs Vilogi : Comparatif 2025"
   - "Open-source vs SaaS propriétaire"

4. **Success stories**
   - "Case study: 50-unit copro saved €2k/year"
   - "From Excel to KoproGo: A journey"

**Distribution** :
- Dev.to (cross-post)
- Medium (cross-post)
- LinkedIn (posts + articles)
- Twitter (threads)
- Reddit (r/selfhosted, r/opensource)
- HackerNews (occasionnel, authentique)

### Social Media (Organic)

**LinkedIn** (Solopreneur BE) :
- Posts 3x/semaine
- Insights marché copropriété
- Behind-the-scenes build
- Client success stories

**Twitter** (Architecte) :
- Posts tech 3x/semaine
- #rustlang, #opensource
- Build in public
- Open metrics

**Reddit** (Communauté) :
- r/selfhosted (self-host guides)
- r/opensource (releases, milestones)
- r/rust (tech discussions)
- Pas de spam, authentique

### Referral Program

**1 mois gratuit par parrainage** :
- Parrain : 1 mois offert (cloud)
- Filleul : 1 mois offert
- Tracking via code promo unique

**Objectif** : Croissance virale organique

---

## Risques et Opportunités ASBL

### Risques Side-Project ASBL

#### 1. Croissance Ultra-Lente

**Risque** : Croissance 5-10 copros/mois (vs 50-100 pour startup avec marketing)

**Impact** : Faible (acceptable pour ASBL side-project)

**Mitigation** :
- ✅ **Pas de stress** : Aucune pression investisseurs, croissance naturelle
- ✅ **Qualité > Quantité** : Meilleure rétention (churn 3-5% vs 10-15%)
- ✅ **Excellence produit** : NPS > 60 = bouche-à-oreille naturel
- ✅ **SEO long-terme** : Blog technique, documentation exemplaire
- ✅ **Rentable immédiatement** : Break-even Mois 2, pas besoin de croissance rapide

**Conclusion** : Risque transformé en **avantage** (durabilité vs burn-out)

#### 2. Temps Équipe Limité (0.25 FTE)

**Risque** : 10-20h/semaine = vélocité 4x plus lente qu'une startup

**Impact** : Moyen (features livrées lentement)

**Mitigation** :
- ✅ **Communauté OSS** : Contributors externes (traductions, bugfixes, features)
- ✅ **Automation maximum** : CI/CD, tests auto, déploiements sans intervention
- ✅ **Focus ruthless** : 20% features = 80% valeur (Pareto)
- ✅ **Documentation self-service** : Réduit support, utilisateurs autonomes
- ✅ **Roadmap communautaire** : Utilisateurs votent features prioritaires

**Conclusion** : Side-project **viable** si scope contrôlé et communauté active

#### 3. Bénévolat Non Rémunéré (3-4 ans)

**Risque** : Démotivation contributeurs si pas de rémunération

**Impact** : Moyen (risque abandon)

**Mitigation** :
- ✅ **Passion intrinsèque** : Contributeurs motivés par mission sociale, pas argent
- ✅ **Apprentissage** : Étudiante informatique = formation pratique sur CV
- ✅ **Flexibilité** : Chacun a job externe, KoproGo = passion side
- ✅ **Transparence** : Comptes publics, promesse rémunération si trésorerie suffisante
- ✅ **Reconnaissance** : Visibilité GitHub, conférences, blog posts

**Conclusion** : Modèle **Mozilla/Wikipedia** prouve que bénévolat tech fonctionne si mission claire

#### 4. Monétisation Faible (1€/copro)

**Risque** : Prix trop bas = revenus insuffisants pour viabilité

**Impact** : Faible (déjà validé par projections)

**Mitigation** :
- ✅ **Unit economics validés** : 1€/copro × 700 copros An 5 = 8,400€/an excédent
- ✅ **Coûts ultra-bas** : 96-192€/an infrastructure, 0€ marketing, 0€ salaires An 1-3
- ✅ **Conversion self-hosted** : 20-30% passent cloud (acquisition gratuite)
- ✅ **Churn faible** : 3-5%/an (besoin réel, produit qualité)
- ✅ **Scaling facile** : 1 vCPU = 1,500 copros (marge énorme)

**Conclusion** : Pricing 1€ = **équitable et viable**, pas besoin d'augmenter

#### 5. Concurrence Agressive

**Risque** : Vilogi/Septeo baissent prix ou copient OpenCore

**Impact** : Très faible (incompatible avec leur modèle)

**Mitigation** :
- ✅ **License MIT** : Peuvent fork mais pas tuer communauté
- ✅ **Impossible à copier** : Authenticity ASBL vs greenwashing corporate
- ✅ **First-mover** : Première solution OpenCore copropriété
- ✅ **Performance tech** : Rust, 0.12g CO₂/req, difficile à égaler
- ✅ **Communauté loyale** : Open-source = confiance > marque

**Conclusion** : ASBL = **moat** que les startups ne peuvent pas copier

### Opportunités ASBL

#### 1. Communauté Open-Source = Croissance Gratuite

**Trigger** : 1,000+ stars GitHub, 50+ contributors actifs

**Impact** : Communauté devient moteur de croissance autonome

**Upside** :
- ✅ **Features gratuites** : Contributors externes développent fonctionnalités
- ✅ **Evangelists** : Ambassadeurs open-source promeuvent KoproGo naturellement
- ✅ **Crédibilité** : GitHub stars = preuve sociale (trust > marketing)
- ✅ **Acquisition $0** : Bouche-à-oreille tech, SEO organique
- ✅ **Talent** : Attire étudiants/devs qui veulent contribuer (CV, learning)

**Exemple** : Plausible Analytics (bootstrap, OSS) : 15k stars → 10k+ clients sans marketing

#### 2. Tendance Anti-Vendor Lock-In

**Trigger** : 2025-2030 = décennie décentralisation, souveraineté numérique

**Impact** : Marché favorable aux solutions open-source et ASBL

**Upside** :
- ✅ **GDPR natif** : Données EU, conformité totale (vs cloud US)
- ✅ **Souveraineté** : OVH France, pas de CLOUD Act
- ✅ **Écologie** : 0.12g CO₂/req, mix français bas carbone
- ✅ **Éthique** : ASBL non-profit vs SaaS profit-driven
- ✅ **Presse** : Médias tech aiment histoires ASBL vs Big Tech

**Exemple** : Framasoft (ASBL FR) : +1M utilisateurs, 0€ pub, financement dons + services

#### 3. Subventions & Dons (Si Utilité Publique)

**Trigger** : Reconnaissance "établissement d'utilité publique" Année 3-4

**Impact** : Accès subventions publiques + dons déductibles fiscalement

**Upside** :
- ✅ **Subventions EU** : Horizon Europe, Digital Europe Programme (€€€ R&D)
- ✅ **Subventions BE** : Innoviris, Digital Wallonia, Fonds Stratégiques
- ✅ **Dons particuliers** : Déduction fiscale 45% → attire donateurs engagés
- ✅ **Dons entreprises** : Déduction 120% → sponsoring RSE
- ✅ **Fondations** : Mozilla Foundation, Sloan Foundation (grants open-source)

**Exemple** : Blender Foundation : $1M/an en dons + $2M subventions → 50+ devs

#### 4. Partenariats Institutionnels

**Trigger** : Product-market fit validé, communauté mature

**Impact** : Institutions publiques adoptent KoproGo (légitimité ASBL)

**Upside** :
- ✅ **Bailleurs sociaux** : Logements publics Belgique/France (milliers copros)
- ✅ **Associations copropriétaires** : ARC, UNPI (recommandations membres)
- ✅ **Universités** : Cas d'étude, projets étudiants, contributions
- ✅ **Open Data** : Intégrations cadastre, données publiques
- ✅ **Certifications** : Labels open-source, B Corp, ESS

**Exemple** : LibreOffice utilisé par administrations EU (millions postes) → légitimité

#### 5. Exit Éthique (Si Souhaité An 10+)

**Trigger** : ASBL mature, 50k+ copros, 10k+ stars, communauté forte

**Impact** : Acteurs établis intéressés par acquisition ASBL

**Upside** :
- ✅ **Acquisition stratégique** : Grands groupes immobiliers/tech veulent communauté
- ✅ **Fusion ASBL** : Avec autre ASBL tech (ex: Framasoft) pour scale
- ✅ **Fondation** : Transformer en fondation d'entreprise (modèle Mozilla)
- ✅ **Legacy** : Code MIT reste libre, communauté préservée

**Principe ASBL** : Exit uniquement si **préserve mission sociale** et **communauté**

### Matrice Risques vs Startup

| Risque | Startup | ASBL Side-Project | Avantage |
|--------|---------|-------------------|----------|
| **Burn-out** | ⚠️ Élevé (80h/semaine) | ✅ Faible (10-20h flexible) | ASBL |
| **Burn rate** | ⚠️ -10k€/mois | ✅ -8€/mois | ASBL |
| **Pression croissance** | ⚠️ VCs exigent 10x | ✅ Aucune | ASBL |
| **Dilution** | ⚠️ 30-50% (levées) | ✅ 0% (pas d'actionnaires) | ASBL |
| **Pivot forcé** | ⚠️ Si VCs insatisfaits | ✅ Liberté totale | ASBL |
| **Dette** | ⚠️ Possible (growth at all costs) | ✅ Impossible (auto-financé) | ASBL |
| **Échec = faillite** | ⚠️ Oui (dettes) | ✅ Non (0€ perdu) | ASBL |

**Conclusion** : ASBL side-project = **Risques minimaux, opportunités maximales**

---

## Comparaison ASBL Side-Project vs Startup

| Aspect | ASBL Side-Project (KoproGo) | Startup VC-Backed |
|--------|----------------------------|-------------------|
| **Financement** | **0€ externe** | 50-200k€ seed |
| **Équipe An 1** | **2 bénévoles (0.25 FTE)** | 3-5 salariés (3-5 FTE) |
| **Salaires An 1-3** | **0€** | 120-300k€/an |
| **Marketing** | **0€ (100% organique)** | 20-50k€/an (ads) |
| **Pression croissance** | **0 (naturelle)** | Élevée (10x/an requis) |
| **Dilution** | **0%** | 20-40% (seed + Series A) |
| **Break-even** | **Mois 2** | Mois 18-24 |
| **Profit An 1** | **+156€** | -50k€ (burn) |
| **Profit An 3** | **+3,348€** | -100k€ ou +50k€ (si scale) |
| **Copros An 5** | **700** | 2,000-5,000 (si survie) |
| **MRR An 5** | **700€** | 50-100k€ (si succès) |
| **Croissance/mois** | **5-10 copros** | 100-500 copros |
| **CAC** | **0€ (organique)** | 50-100€ (ads) |
| **Churn** | **3-5%/an** | 10-20%/an |
| **Contrôle** | **100% ASBL** | 50-70% founders |
| **Stress** | **Faible** | Très élevé |
| **Burn-out risque** | **Très faible** | Élevé |
| **Exit pression** | **Aucune** | Forte (ROI VCs) |
| **Mission sociale** | **Priorité #1** | Secondaire |
| **Survie si échec** | **0€ perdu** | Faillite, dettes |

**Choix ASBL Side-Project** :
- ✅ **Durabilité** : Viable 10+ ans sans stress
- ✅ **Impact social** : Mission avant profit
- ✅ **Liberté** : Aucune pression externe
- ✅ **Éthique** : Transparence, open-source, GDPR
- ✅ **Risque 0** : Aucun investissement, aucune dette

**Pourquoi pas startup ?**
- ⚠️ Pression VCs incompatible avec qualité long-terme
- ⚠️ Burn-out garanti (80h/semaine)
- ⚠️ Dilution = perte contrôle mission sociale
- ⚠️ Pivot forcé si VCs insatisfaits
- ⚠️ Exit obligatoire (incompatible avec pérennité open-source)

---

## Conclusion

### Philosophie ASBL Side-Project

> "Lentement mais sûrement. Bien plutôt que vite. Pérenne plutôt que profitable."

Nous choisissons **impact social durable** plutôt que **croissance à tout prix**.

### Pourquoi ce modèle ASBL va fonctionner

**1. Modèles ASBL/Non-Profit tech validés**
- **Mozilla Foundation** : 20+ ans, Firefox, $500M/an budget, rentable sans actionnaires
- **Wikimedia** : Wikipedia, 300M+ utilisateurs, $150M/an, 100% dons + services
- **Blender Foundation** : Logiciel 3D industrie, $3M/an, 50+ devs salariés
- **Framasoft (ASBL FR)** : +1M utilisateurs, 0€ pub, 100% dons + services
- **Document Foundation** : LibreOffice, 700+ contributors, certifications payantes

**2. Unit economics ASBL exceptionnels**
- **LTV** : 20€/copro (churn 5%/an)
- **CAC** : 0€ (organique, communauté)
- **LTV/CAC** : ∞ (théorique) ou 4:1 (réaliste si on compte temps bénévole)
- **Payback** : Immédiat (0€ investissement)
- **Gross margin** : 98%+ (coûts ultra-bas)
- **Churn** : 3-5%/an (excellent, besoin réel)
- **Break-even** : **Mois 2** (vs 18-24 mois startup)

**3. Marché sous-adressé + USP unique**
- **5.5M copropriétés** EU+Maghreb non digitalisées
- **0 solution OpenCore** dans ce marché
- **Pain point #1** : Vendor lock-in (on résout avec MIT license)
- **Différenciation** : ASBL = confiance, éthique, souveraineté numérique

**4. Équipe lean, flexible, durable**
- **2 bénévoles** = haute vélocité décision (vs 10 personnes startup = lenteur)
- **0 overhead** investisseurs, board, reporting
- **Passion > pression** = qualité code, rétention long-terme
- **Side-project** = sécurité financière (jobs externes), 0 stress burn-rate

**5. Timing 2025 optimal**
- **Open-source SaaS** en croissance (GitLab, Plausible, PostHog succès)
- **Anti-vendor lock-in** trend fort (GDPR, souveraineté, décentralisation)
- **Digitalisation copropriétés** accélérée post-COVID
- **ASBL/ESS** valorisées (RSE, dons, subventions accessibles)

**6. Risque 0, upside illimité**
- **0€ investissement** : Aucun capital perdu si échec
- **0€ dette** : Auto-financé, rentable Mois 2
- **Liberté totale** : Aucune pression externe, aucune dilution
- **Mission sociale** : Impact positif garanti, même si croissance lente
- **Exit éthique possible** : Si mature, acquisition/fusion préservant mission

### Prochaines Étapes ASBL (0€ investissement)

**Trimestre 1 (Mois 1-3) : Constitution ASBL + Core MVP**
1. **Rédiger statuts ASBL** (objet social, gouvernance) - 1 semaine
2. **Notaire + Publication Moniteur** (~450€, seul investissement) - 1 mois
3. **Finaliser core open-source** (CRUD complet, tests 100%) - 2 mois
4. **Documentation exemplaire** (README, architecture, contribution guide)
5. **Docker Compose 1-click** (self-hosting facile)

**Trimestre 2 (Mois 4-6) : Launch Open-Source**
6. **Publier GitHub** (MIT license, README pro)
7. **Launch HackerNews** : "Show HN: KoproGo - Open-Source Property Management (ASBL)"
8. **Dev.to, Reddit, LinkedIn** posts
9. **Discord communauté** (support, feedback)
10. **10 blog posts** techniques (SEO, communauté)

**Semestre 2 (Mois 7-12) : Cloud MVP + Early Adopters**
11. **Plateforme cloud** (signup, multi-tenant, Stripe 1€/copro)
12. **5-10 early adopters** cloud payants
13. **100-200 self-hosted** instances
14. **GitHub** : 1,000 stars, 10+ contributors
15. **Rentabilité** : **156€ profit Année 1** ✅

**Année 2 : Stabilité**
16. **20-50 clients cloud** (80 copros)
17. **GitHub** : 3,000 stars, 30+ contributors
18. **Profit** : **1,032€ Année 2** ✅

**Année 3 : Viabilité**
19. **100-200 clients cloud** (200 copros)
20. **Indemnités bénévoles** si trésorerie > 10k€
21. **Produit mature** : Features complètes, communauté active

**Année 5 : Pérennité**
22. **700 copros** cloud
23. **Salaires partiels** si trésorerie > 30k€
24. **ASBL autonome** : Mission sociale accomplie ✅

---

## Annexes

### A. Coûts Détaillés ASBL (Années 1-5)

| Poste | An 1 | An 2 | An 3 | An 4 | An 5 |
|-------|------|------|------|------|------|
| **VPS OVH** | 84€ | 84€ | 84€ | 180€ | 180€ |
| **Domaine (.com)** | 12€ | 12€ | 12€ | 12€ | 12€ |
| **Compte bancaire ASBL** | 0-120€ | 120€ | 120€ | 120€ | 120€ |
| **Comptabilité** | 0€ | 0€ | 0€ | 300€ | 300€ |
| **Indemnités bénévoles** | 0€ | 0€ | 0€ | 0€ | 6,000€ |
| **Marketing** | 0€ | 0€ | 0€ | 0€ | 0€ |
| **Total** | **96-216€** | **216€** | **216€** | **612€** | **6,612€** |

**Note** : Constitution ASBL initiale = ~450€ one-time (notaire + Moniteur)

### B. Métriques Communauté (Objectifs)

**Année 1** :
- GitHub stars : **1,000**
- Self-hosted instances : **100-200**
- Contributors actifs : **10+**
- PRs mergées : **50+**
- Discord : **200 membres**
- Clients cloud : **10**

**Année 2** :
- GitHub stars : **3,000**
- Self-hosted instances : **300-500**
- Contributors actifs : **30+**
- PRs mergées : **200+**
- Discord : **500 membres**
- Clients cloud : **20-50**

**Année 3** :
- GitHub stars : **5,000-10,000**
- Self-hosted instances : **500-1,000**
- Contributors actifs : **50-100**
- PRs mergées : **300-500**
- Discord : **1,000 membres**
- Clients cloud : **100-200**

**Année 5** :
- GitHub stars : **15,000+**
- Self-hosted instances : **2,000+**
- Contributors actifs : **150+**
- PRs mergées : **1,000+**
- Discord : **3,000 membres**
- Clients cloud : **300-500**

### C. Tech Stack Confirmé (Production-Ready)

**Backend** :
- Rust 1.85+ + Actix-web 4.9
- PostgreSQL 15-alpine
- SQLx (compile-time query verification)

**Frontend** :
- Astro 4.x (SSG)
- Svelte 4.x (islands)
- Tailwind CSS 3.x

**Infrastructure** :
- Docker + Docker Compose
- Traefik (reverse proxy + SSL auto Let's Encrypt)
- OVH Cloud France (datacenter Strasbourg/Gravelines)
- Vercel (frontend CDN gratuit)

**CI/CD** :
- GitHub Actions (tests, lint, build)
- GitOps (déploiements automatisés)

**Monitoring** :
- UptimeRobot (gratuit, monitoring externe)
- Scripts custom VPS (métriques système)

**License** :
- MIT (100% du code)
- Pas de code propriétaire (philosophie ASBL)

**Performance Validée** :
- 99.74% uptime (tests charge)
- 287 req/s soutenus sur 1 vCPU
- Latence P50: 69ms, P90: 130ms, P99: 752ms
- RAM: 128MB / 2GB (6.3% usage)
- Capacité: 1,000-1,500 copros par vCPU

**Écologie** :
- 0.12g CO₂/req (OVH France, mix 60g CO₂/kWh)
- 5.8x moins d'émissions que Hetzner DE
- 7-25x moins que AWS/Azure US

### D. Ressources Constitution ASBL

**Liens utiles** :
- Loi belge ASBL : https://www.ejustice.just.fgov.be (Code des sociétés et des associations)
- Guichet Entreprises (BCE) : https://economie.fgov.be/fr/guichet-entreprises
- Moniteur belge : https://www.moniteur.be
- Modèles statuts ASBL : https://www.notaire.be

**Coûts constitution** :
- Notaire : 200-300€ (tarif réduit ASBL)
- Publication Moniteur : 180-220€
- Total : ~450-500€

**Délai** : 1-2 mois (rédaction → notaire → publication → BCE)

### E. Contact & Liens

**ASBL KoproGo** (en cours de constitution)

📧 **Email** : contact@koprogo.com (à créer)
🌐 **GitHub** : https://github.com/gilmry/koprogo
🌐 **Site web** : https://koprogo.com (à créer)
💬 **Discord** : [communauté à créer après launch]
📱 **LinkedIn** : [page ASBL à créer]

**Membres fondateurs** :
- Architecte logiciel (Président CA)
- Étudiante informatique (Secrétaire CA)
- [Trésorier CA à recruter - bénévole comptable/juriste]

---

**Transparence ASBL** : Ce document est public. Les comptes annuels de l'ASBL KoproGo seront publiés sur GitHub et le site web, conformément aux obligations légales belges et à notre engagement de transparence radicale.

**License document** : CC BY-SA 4.0 (Creative Commons Attribution-ShareAlike)

---

**KoproGo ASBL** - Un bien commun numérique pour la gestion de copropriété 🏛️🔓🌱

*"Impact social avant profit. Qualité avant vitesse. Pérennité avant croissance."*
