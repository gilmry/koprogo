# KoproGo - Modèle Économique et Exemples Open Source

**Document pour parties prenantes**
**Octobre 2025**

---

## Contexte

### Le problème

En Belgique, 1,5 million de copropriétés font face à des logiciels propriétaires coûteux (200-500€/mois), avec peu de transparence et un impact écologique significatif (50-130 kg CO₂/an). Les données sont verrouillées dans des solutions fermées et l'isolement urbain entre voisins s'aggrave.

### La solution : KoproGo

Plateforme open source (AGPL-3.0) qui combine gestion de copropriété moderne et modules communautaires optionnels. Le code est 100% transparent, avec possibilité d'auto-hébergement gratuit ou de services cloud payants pour financer le développement.

**Impact attendu** : 96% de réduction CO₂, 95-99% d'économies pour les utilisateurs, et recréation de liens sociaux entre voisins.

---

## Le Modèle Open Source

### Qu'est-ce que l'open source ?

Logiciel dont le code source est accessible, modifiable et redistribuable par tous. Avantages principaux :

- Transparence : code auditable par tous
- Sécurité : failles détectées rapidement par la communauté
- Innovation collaborative : contributeurs du monde entier
- Absence de lock-in : contrôle total des données
- Coûts réduits : pas de licence propriétaire

### Le modèle "OpenCore"

Le cœur du logiciel est 100% gratuit et open source. Les services professionnels sont payants : hébergement cloud, support premium, formations. Ce modèle fonctionne grâce à l'effet réseau (plus d'utilisateurs = meilleur produit), la confiance par la transparence, et une adoption rapide facilitée par la gratuité.

---

## Structure : ASBL Non-Profit

KoproGo est une Association Sans But Lucratif belge, pas une startup commerciale.

### Différences clés avec une startup classique

| Aspect | Startup | ASBL KoproGo |
|--------|---------|--------------|
| Objectif | Profit actionnaires | Impact social |
| Code | Propriétaire | Open source (AGPL-3.0) |
| Prix | Maximum | Prix coûtant + marge solidaire |
| Données | Monétisées | Propriété utilisateur |
| Gouvernance | CEO + VC | Assemblée Générale |
| Bénéfices | Dividendes | Réinvestis |

---

## Modèle de Revenus

### Hybride 20/80

**80% Self-Hosted (gratuit)** :
- Téléchargement libre du code source
- Installation sur serveur propre (VPS ~7€/mois)
- Souveraineté totale des données
- Coût réel : 0,07€/copro/mois (100 copros sur 1 serveur)

**20% Cloud managé (payant)** :
- 1€/mois par copropriété
- Hébergement bas-carbone (OVH France)
- Sauvegardes et mises à jour automatiques
- Support communautaire (72h)
- Quotas : 500 MB, 50 utilisateurs, 100k requêtes/mois

### Tarification transparente à prix coûtant

Principe : zéro profit sur les dépassements de quota.

**Exemple de répartition mensuelle** (publique sur koprogo.be/transparence) :

```
Infrastructure VPS OVH :    7,00€
Stockage S3 (200 GB) :      2,00€
DNS :                       0,10€
────────────────────────────────
Total coûts :               9,10€

100 copros × 1€ =         100,00€
Dépassements stockage :    15,00€
────────────────────────────────
Revenus :                 115,00€

Excédent (105,90€) réparti :
- Réserve sécurité (50%) :  54,60€
- Développement (30%) :     32,76€
- Infrastructure (10%) :    10,92€
- Fonds urgence (10%) :      7,62€
```

Tous les comptes sont publics et auditables.

### Services additionnels (futurs)

Pour grandes copropriétés et syndics professionnels :

| Service | Prix estimé |
|---------|-------------|
| Déploiement assisté | 200-500€ one-time |
| Formation syndic | 800€ |
| Support premium | +5€/mois |
| Intégration comptable | 300€ setup |
| Programme Sponsor | 100€/an |

Ces services financent le développement sans compromettre la gratuité du core.

---

## Économies d'Échelle

### Comment les coûts diminuent avec la croissance

Plus d'utilisateurs = coût par copropriété qui diminue. Infrastructure fixe jusqu'à un seuil, croissance progressive ensuite.

#### Coûts réels OVH (2025)

| Ressource | Prix |
|-----------|------|
| VPS Value | 5,80€/mois (1 vCore, 2GB RAM) |
| VPS Essential | 12€/mois (2 vCore, 4GB RAM) |
| VPS Elite | 27€/mois (8 vCore, 32GB RAM) |
| Object Storage S3 | 0,007€/GB/mois |
| Bande passante | Gratuite (incluse) |
| DNS | 0,10€/mois |

#### Scénario croissance

**100 copropriétés** :
- Infrastructure : 6,25€/mois
- Coût par copro : 0,063€/mois
- Revenus : 100€/mois
- Excédent : 93,75€/mois

**500 copropriétés** :
- Infrastructure : 13,85€/mois
- Coût par copro : 0,028€/mois (−55%)
- Revenus : 500€/mois
- Excédent : 486,15€/mois

**2 000 copropriétés** :
- Infrastructure : 34,10€/mois
- Coût par copro : 0,017€/mois (−73%)
- Revenus : 2 000€/mois
- Excédent : 1 965,90€/mois

Le coût par copropriété diminue de 73% entre 100 et 2 000 utilisateurs.

### Réinvestissement dans le prix

L'Assemblée Générale décide annuellement de l'utilisation de l'excédent :
- Baisse du prix
- Amélioration des quotas gratuits
- Développement de nouvelles fonctionnalités
- Constitution de réserves

**Projection de baisse possible** :

| Année | Copros | Prix appliqué | Baisse |
|-------|--------|---------------|--------|
| 2025 | 100 | 1,00€ | - |
| 2026 | 500 | 0,80€ | −20% |
| 2027 | 1 000 | 0,60€ | −40% |
| 2028 | 2 000 | 0,50€ | −50% |
| 2030 | 5 000 | 0,40€ | −60% |

C'est la communauté qui décide, pas des actionnaires.

---

## Exemples de Réussite Open Source

### Red Hat

- **Activité** : Distribution Linux enterprise (RHEL)
- **Modèle** : OS gratuit + support/certification payant
- **Résultat** : Acquis par IBM pour 34 milliards USD (2019)
- **Leçon** : Open source + services B2B = viable à très grande échelle

### WordPress / Automattic

- **Activité** : CMS open source (43% du web)
- **Modèle** : Self-hosted gratuit + WordPress.com payant
- **Résultat** : 7,5 milliards USD valorisation
- **Leçon** : Freemium + hébergement managé = millions d'utilisateurs

### GitLab

- **Activité** : Plateforme DevOps
- **Modèle** : Core gratuit + features enterprise payantes
- **Résultat** : 6 milliards USD IPO (2021)
- **Leçon** : Transparence + fonctionnalités avancées = confiance entreprises

### Odoo (Belge)

- **Activité** : ERP open source
- **Modèle** : Community gratuite + Enterprise + SaaS
- **Résultat** : Leader ERP PME, 7+ millions utilisateurs
- **Leçon** : Open source local peut devenir leader mondial

### Signal

- **Activité** : Messagerie chiffrée
- **Modèle** : 100% gratuit, 0€ revenus commerciaux, dons
- **Résultat** : 40+ millions utilisateurs, alternative éthique aux GAFAM
- **Leçon** : Impact social > profit = possible et viable

---

## Comparaison avec la Concurrence

| Critère | Vilogi/Apronet | KoproGo Cloud | KoproGo Self-Hosted |
|---------|----------------|---------------|---------------------|
| Prix/mois | 200-500€ | 1-3€ | 0€ (serveur ~7€) |
| Code source | Fermé | Open (AGPL) | Open (AGPL) |
| Souveraineté données | Cloud US/FR | EU (OVH) | Totale |
| Lock-in | Fort | Faible | Aucun |
| RGPD | Limité | Conforme | Conforme |
| Personnalisation | Impossible | Limitée | Totale |
| Support | 9h-18h | 72h | Communauté |
| CO₂/an | 50-130 kg | 0,005 kg | 0,001 kg |

### Économies réalisées

**Exemple : copropriété 20 lots sur 3 ans**

| Poste | Propriétaire | KoproGo Cloud | Économie |
|-------|--------------|---------------|----------|
| Licence | 3 000€ | 12€ | −99,6% |
| Formation | 800€ | 0€ | −100% |
| Migration | 500€ | 0€ | −100% |
| **Total** | **10 400€** | **36€** | **−99,65%** |

---

## Opportunités de Soutien

### Pourquoi soutenir KoproGo ?

**Impact social mesurable** :
- 1,5 million de copropriétés en Belgique
- 8 millions € d'économies d'ici 2030
- −500 tonnes CO₂/an d'ici 2030
- Lien social via modules communautaires

**Modèle économique prouvé** : Red Hat (34 mds), WordPress (7,5 mds), GitLab (6 mds), MongoDB (20 mds)

**Différenciation forte** :
- Légal : AGPL protège contre fork propriétaire
- Éthique : ASBL = mission sociale > profit
- Technique : Rust + GitOps = performance + fiabilité
- Écologique : 96% réduction CO₂
- Local : Belge, RGPD-first, souveraineté UE

**Traction initiale sans financement** :
- 0€ levés, 100% autofinancé
- Break-even projeté : mois 2
- Croissance organique : 10-20%/mois

### Formes de soutien possibles

**A. Partenariat stratégique (non-financier)**
- Beta-testing de la plateforme
- Feedback sur features prioritaires
- Études de cas et témoignages
- Avantages : accès gratuit à vie, influence roadmap, économies immédiates

**B. Sponsoring ASBL**
- Programme Copropriété Sponsor : 100€/an (logo, support 24h, vote roadmap)
- Grandes entreprises/fondations : 1 000-10 000€/an

**C. Subventions publiques**
- Région Bruxelles-Capitale, Wallonie-Bruxelles
- Union Européenne : Horizon Europe
- Fondation Roi Baudouin
- Montants visés : 10 000-200 000€

**D. Services B2B (futurs revenus)**

| Service | Volume An 3 | Revenus |
|---------|-------------|---------|
| Déploiement | 50/an | 15 000€ |
| Formation | 20/an | 16 000€ |
| Support premium | 100 clients | 6 000€ |
| Intégration API | 10/an | 5 000€ |
| **Total services** | | **42 000€** |

Total avec cloud : 126 000€/an (An 3)

### Roadmap financement

**Phase 1 : Bootstrap (2025) - 0€ externe**
- Développement bénévole (10-20h/semaine)
- Infrastructure minimale (10€/mois)
- Objectif : 100 premiers utilisateurs

**Phase 2 : Sponsoring initial (2026) - 10 000-30 000€**
- Syndics partenaires, subventions régionales
- Temps développeur partiel (2j/semaine)
- Objectif : 500 copropriétés, communauté active

**Phase 3 : Services B2B (2027+) - Autofinancement**
- Revenus récurrents : 126 000€/an
- 2-3 développeurs temps plein
- Objectif : 2 000+ copropriétés, viabilité long terme

---

## Conclusion

### Points clés

1. **Le modèle open source + services fonctionne** : preuves à 34 milliards USD
2. **KoproGo résout un vrai problème** : 1,5M copropriétés, 95-99% d'économies, 96% réduction carbone
3. **Structure ASBL = impact social** : bénéfices réinvestis, transparence, démocratie
4. **Traction sans financement** : bootstrap réussi, break-even mois 2
5. **Opportunités multiples** : partenariats, sponsoring, subventions, services B2B

### Contact et collaboration

**GitHub** : https://github.com/gilmry/koprogo

**Opportunités** :
- Beta-testeurs (syndics, copropriétés)
- Sponsors ASBL (entreprises, fondations)
- Contributeurs open source (développeurs)
- Partenaires institutionnels (subventions)

---

L'open source n'est pas seulement idéaliste, c'est pragmatique. Les plus grandes réussites technologiques des 20 dernières années sont open source. KoproGo combine l'impact social d'une ASBL avec la viabilité du modèle OpenCore éprouvé.

Nous ne construisons pas une licorne. **Nous construisons un bien commun durable.**

---

**Équipe KoproGo ASBL**
**Octobre 2025**
