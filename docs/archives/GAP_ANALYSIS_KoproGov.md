# Analyse des Gaps - KoproGo vs Fonctionnalités Requises

**Date**: 1 novembre 2025
**Version**: 1.0
**Auteur**: Analyse comparative complète

---

## 📊 Résumé Exécutif

Cette analyse compare l'état actuel de KoproGo avec la liste complète des fonctionnalités requises pour une plateforme de gestion de copropriété conforme à la législation belge.

### Statistiques Globales

| Catégorie | Total | Implémenté | Partiel | Manquant | % Complétion |
|-----------|-------|------------|---------|----------|--------------|
| **Gestion Administrative** | 7 | 5 | 2 | 0 | **71%** |
| **Gestion Financière** | 8 | 3 | 2 | 3 | **38%** |
| **Représentation & Exécution** | 6 | 2 | 1 | 3 | **33%** |
| **Conseil de Copropriété** | 7 | 0 | 0 | 7 | **0%** |
| **Système de Vote et AG** | 6 | 1 | 1 | 4 | **17%** |
| **Documents Légaux** | 8 | 0 | 2 | 6 | **13%** |
| **Carnet d'Entretien** | 5 | 0 | 0 | 5 | **0%** |
| **Modules Communautaires** | 9 | 0 | 0 | 9 | **0%** |
| **RGPD** | 5 | 3 | 0 | 2 | **60%** |
| **Infrastructure** | 7 | 2 | 2 | 3 | **29%** |
| **Multi-tenant/Rôles** | 4 | 4 | 0 | 0 | **100%** ✅ |
| **Intégrations** | 4 | 1 | 1 | 2 | **25%** |
| **UX** | 5 | 3 | 2 | 0 | **60%** |
| **Accessibilité** | 4 | 0 | 0 | 4 | **0%** |
| **Performance** | 4 | 2 | 2 | 0 | **50%** |
| **Analytics** | 4 | 1 | 1 | 2 | **25%** |
| **TOTAL** | **93** | **27** | **14** | **52** | **29%** |

---

## 🎯 État Actuel de l'Implémentation

### ✅ Fonctionnalités Complètement Implémentées (27)

#### Gestion Administrative
1. ✅ **Registre de copropriété** - Base de données complète (buildings, units, owners)
2. ✅ **Gestion liste copropriétaires** - CRUD complet avec quotités (ownership_percentage)
3. ✅ **Conservation documents** - Système de storage avec metadata (documents table)
4. ✅ **Gestion Assemblées Générales** - Entité Meeting avec types (Ordinary/Extraordinary), agenda, status
5. ✅ **Gestion PV** - Champ `minutes` dans meeting, liens vers documents

#### Gestion Financière
6. ✅ **Appels de fonds** - Expense tracking avec catégories et répartition
7. ✅ **Gestion charges** - CRUD expenses avec payment_status (pending/paid/overdue/cancelled)
8. ✅ **Fonds de roulement** - Organization subscription plans avec limites

#### Représentation & Exécution
9. ✅ **Contrats** - Document type 'Contract' avec liens building/meeting/expense
10. ✅ **Gestion documents** - Upload/download avec file_path, mime_type, file_size

#### Système de Vote et AG
11. ✅ **Gestion réunions de base** - Meeting entity avec scheduled_date, location, agenda (JSONB)

#### RGPD
12. ✅ **Export données personnelles** - `GET /gdpr/export` (Article 15)
13. ✅ **Droit à l'effacement** - `DELETE /gdpr/erase` avec anonymisation (Article 17)
14. ✅ **Audit logs** - Table audit_logs avec retention 7 ans

#### Infrastructure
15. ✅ **Docker Compose production** - Traefik + PostgreSQL + backend + frontend
16. ✅ **GitOps VPS** - Ansible + Terraform + systemd auto-deploy

#### Multi-tenant/Rôles
17. ✅ **Multi-tenant complet** - Organizations avec isolation données
18. ✅ **Multi-rôle** - User roles (SuperAdmin, Syndic, Accountant, Owner) avec switch
19. ✅ **Multi-owner** - Junction table unit_owners avec ownership_percentage
20. ✅ **Permissions granulaires** - Role-based access control (RBAC)

#### Intégrations
21. ✅ **API REST complète** - 73 endpoints documentés

#### UX
22. ✅ **Progressive Web App** - IndexedDB, Service Worker, offline mode
23. ✅ **Dashboard personnalisé** - 4 dashboards par rôle (Admin, Syndic, Accountant, Owner)
24. ✅ **Notifications temps réel** - Toast notifications via Svelte store

#### Performance
25. ✅ **< 100ms latency P95** - Tests confirment performance cible
26. ✅ **Connection pool optimisé** - Max 10 PostgreSQL connections

#### Analytics
27. ✅ **Statistiques syndic** - Dashboard avec buildings, units, owners, expenses, urgent tasks

---

### 🟡 Fonctionnalités Partiellement Implémentées (14)

#### Gestion Administrative
28. 🟡 **Convocation AG** - Meetings créés, mais pas de système automatique de convocation/notification
29. 🟡 **PV dans les 30 jours** - Pas de workflow ni de rappels automatiques

#### Gestion Financière
30. 🟡 **Budget prévisionnel** - Structure expense existe, mais pas de budget annuel planifié
31. 🟡 **Recouvrement charges impayées** - Status 'overdue' existe, mais pas de workflow relance

#### Représentation & Exécution
32. 🟡 **Exécution décisions AG** - Meetings + documents, mais pas de task tracking décisions

#### Système de Vote et AG
33. 🟡 **Types de majorité** - Pas de calcul automatique (simple, 2/3, 3/4, unanimité)

#### Documents Légaux
34. 🟡 **PCN (Précompte Charge Notariale)** - Use case `PcnUseCases` existe, mais génération PDF incomplète
35. 🟡 **Quittances de charges** - Expenses exist, mais pas de génération quittances PDF

#### Infrastructure
36. 🟡 **Backups** - Pas de backup automatisé chiffré (GPG + S3)
37. 🟡 **Monitoring** - Pas de stack Prometheus/Grafana/Loki

#### Intégrations
38. 🟡 **Webhooks** - Infrastructure API prête, mais pas d'implémentation webhooks

#### UX
39. 🟡 **Responsive mobile** - Tailwind CSS, mais tests mobiles limités
40. 🟡 **Drag & drop documents** - DocumentUploadModal existe, mais pas de drag-drop

#### Performance
41. 🟡 **Cache intelligent** - Pas de Redis/DragonflyDB intégré

---

### ❌ Fonctionnalités Manquantes (52)

#### 🔴 CRITIQUES (14)

##### Gestion Financière
42. ❌ **Comptabilité claire plan comptable normalisé** - Pas de plan comptable belge structuré
43. ❌ **Comptabilité simplifiée <20 lots** - Pas de distinction comptable
44. ❌ **Recouvrement automatique** - Pas de workflow relance impayés
45. ❌ **Présentation multiple devis** - Pas de module devis/comparaison

##### Représentation & Exécution
46. ❌ **Fourniture relevé dettes notaire** - Pas de génération état daté automatique
47. ❌ **Affichage public infos syndic** - Pas de page publique obligation légale
48. ❌ **Rapport évaluation contrats** - Pas de module analyse contrats

##### Conseil de Copropriété (0/7)
49. ❌ **Dashboard suivi décisions AG** - Entité manquante
50. ❌ **Tracking délais** - Pas de système de suivi tâches conseil
51. ❌ **Système alertes retards** - Pas de notifications automatiques
52. ❌ **Accès lecture seule documents** - Pas de rôle BoardMember
53. ❌ **Rapport semestriel automatique** - Pas de génération rapports conseil
54. ❌ **Rapport annuel pour AG** - Pas de templates rapports
55. ❌ **Gestion membres conseil** - Pas d'entité, élection, mandats

##### Système de Vote et AG (4/6)
56. ❌ **Authentification forte eID/itsme** - OIDC non implémenté (critique légal Belgique)
57. ❌ **Système vote à distance** - Pas de Vote entity ni endpoints
58. ❌ **Calcul majorités** - Pas de calcul automatique selon quotités
59. ❌ **PV automatique avec détail votes** - Pas de génération automatique

##### Documents Légaux (6/8)
60. ❌ **État daté** - Pas de génération (mutations immobilières)
61. ❌ **Pré-état daté** - Pas de génération
62. ❌ **PCN complet** - Export PDF manquant
63. ❌ **Annexes comptables obligatoires** - Pas de templates
64. ❌ **PV format légal belge** - Pas de templates conformes

##### RGPD (2/5)
65. ❌ **Article 16 - Rectification** - Endpoints manquants
66. ❌ **Article 18 & 21 - Restriction & Objection** - Endpoints manquants

##### Infrastructure (3/7)
67. ❌ **LUKS encryption at rest** - Pas de chiffrement disque (Issue #39)
68. ❌ **Backups chiffrés GPG + S3** - Pas de backup automatisé (Issue #40)
69. ❌ **Security hardening** - fail2ban, CrowdSec, Suricata manquants (Issue #43)

#### 🟡 IMPORTANTES (24)

##### Carnet d'Entretien (0/5)
70. ❌ **Suivi travaux et interventions** - Pas d'entité WorkReport
71. ❌ **Historique maintenance** - Pas de tracking historique
72. ❌ **Planning travaux futurs** - Pas de module planification
73. ❌ **Garanties constructeurs** - Pas de gestion garanties
74. ❌ **Alertes contrôles techniques** - Pas de notifications obligatoires

##### Modules Communautaires (0/9)
75. ❌ **SEL (Système Échange Local)** - Pas d'entité ni module (Issue #49)
76. ❌ **Bazar de troc** - Pas d'entité SwapItem
77. ❌ **Prêt d'objets** - Pas d'entité ObjectLoan
78. ❌ **Annuaire compétences** - Pas d'entité SkillOffer
79. ❌ **Tableau affichage numérique** - Pas d'entité Notice
80. ❌ **Tracking échanges** - Pas de métriques SEL
81. ❌ **Statistiques utilisation** - Pas de dashboard communautaire
82. ❌ **Rapport impact social** - Pas de génération rapports
83. ❌ **Gamification** - Pas de système points/badges

##### Infrastructure
84. ❌ **Monitoring Stack** - Prometheus/Grafana/Loki manquant (Issue #41)

##### Intégrations
85. ❌ **SSO grandes organisations** - Pas d'intégration SAML/OIDC
86. ❌ **Compatibilité comptables externes** - Pas d'export formats standard

##### Accessibilité (0/4)
87. ❌ **WCAG 2.1 niveau AA** - Pas de tests accessibilité
88. ❌ **Navigation clavier complète** - Pas de support complet
89. ❌ **Lecteurs d'écran** - Pas de tests ARIA
90. ❌ **Mode contraste élevé** - Pas de thème accessibilité

##### Analytics (2/4)
91. ❌ **Métriques communautaires** - Pas de KPIs SEL/troc
92. ❌ **BI externe** - Pas d'API analytics dédiée

#### 🟢 NICE-TO-HAVE (14)

##### UX
93. ❌ Drag & drop upload documents complet

##### Performance
94. ❌ Support 1000-1500 copros/vCPU (tests charge manquants)
95. ❌ 0.12g CO₂/requête (métriques écologiques non trackées)

##### DevOps
96. ❌ Tests E2E Playwright avec vidéos (partiels)
97. ❌ Documentation vidéos (partielles)

##### Analytics
98. ❌ Rapports personnalisables (templates manquants)
99. ❌ Excel pour comptables (export basique uniquement)

##### Mobile
100. ❌ **Mobile App native** - React Native/Flutter (Phase 3 roadmap)

##### Advanced Features (Phase 3)
101. ❌ **ScyllaDB/DragonflyDB** - NoSQL cache performance
102. ❌ **Real-time WebSocket** - Notifications temps réel
103. ❌ **Advanced Analytics Dashboard** - Métriques métier avancées
104. ❌ **Advanced Search** - ElasticSearch/MeiliSearch
105. ❌ **Audit Dashboard** - Visualisation logs SuperAdmin
106. ❌ **Contractor Backoffice** - Prestataires (Issue #52)

---

## 📋 Analyse Détaillée par Catégorie

### 1️⃣ Gestion Administrative (71% - Bon)

| Fonctionnalité | État | Note |
|----------------|------|------|
| Convocation AG | 🟡 | Meeting API existe, manque envoi email/PDF convocation |
| Rédaction/transmission PV | 🟡 | Champ `minutes` existe, manque workflow 30 jours |
| Registre copropriété | ✅ | Complet avec audit logs |
| Liste copropriétaires | ✅ | Multi-owner, quotités, historique |
| Conservation documents | ✅ | Upload/download, types, liens |
| Relevé dettes notaire | ❌ | État daté manquant |
| Affichage public syndic | ❌ | Page publique légale manquante |

**Priorités**:
1. 🔴 **Convocation AG automatique** - Email + PDF génération (Issue à créer)
2. 🔴 **État daté pour mutations** - Génération PDF conforme (Issue à créer)
3. 🟡 **Workflow PV 30 jours** - Rappels automatiques

---

### 2️⃣ Gestion Financière (38% - Insuffisant)

| Fonctionnalité | État | Note |
|----------------|------|------|
| Comptabilité plan normalisé | ❌ | Plan comptable belge manquant |
| Comptabilité simplifiée <20 lots | ❌ | Pas de distinction |
| Budget prévisionnel | 🟡 | Expenses existent, budget annuel manquant |
| Appels de fonds | ✅ | Expense tracking OK |
| Répartition charges | ✅ | Ownership percentages OK |
| Recouvrement impayés | 🟡 | Status 'overdue' existe, workflow relance manquant |
| Fonds roulement/réserve | ✅ | Organization plans OK |
| Multiple devis travaux | ❌ | Module devis manquant |

**Priorités**:
1. 🔴 **Plan comptable belge** - Normalisation catégories (Issue à créer)
2. 🔴 **Budget prévisionnel annuel** - Entity + use cases (Issue à créer)
3. 🔴 **Workflow recouvrement** - Relances automatiques (Issue à créer)
4. 🟡 **Module devis travaux** - Comparaison prestataires

---

### 3️⃣ Conseil de Copropriété (0% - CRITIQUE)

**Aucune fonctionnalité implémentée !**

| Fonctionnalité | État | Effort | Priorité |
|----------------|------|--------|----------|
| Dashboard suivi décisions AG | ❌ | Large | 🔴 Critique |
| Tracking délais (devis, travaux) | ❌ | Medium | 🔴 Critique |
| Système alertes retards | ❌ | Medium | 🔴 Critique |
| Accès lecture documents | ❌ | Small | 🔴 Critique |
| Rapport semestriel | ❌ | Medium | 🟡 High |
| Rapport annuel AG | ❌ | Medium | 🟡 High |
| Gestion membres (élection, mandats) | ❌ | Large | 🟡 High |

**Note légale**: Le conseil de copropriété est **obligatoire légalement** en Belgique pour immeubles >20 lots. C'est un gap critique pour la conformité.

**Roadmap**: Issue #51 (Phase 1 VPS MVP) prévoit Board Tools, mais besoin de détail complet.

**Recommandations**:
1. 🔴 Créer rôle `BoardMember` dans user_roles
2. 🔴 Entity `BoardMember` (user_id, building_id, elected_date, mandate_end, is_active)
3. 🔴 Entity `BoardDecision` (id, building_id, decision_type, description, decided_at, status)
4. 🔴 Dashboard conseil: `/board/dashboard` avec:
   - Décisions AG en cours
   - Tâches en retard
   - Prochain rapport à générer
5. 🔴 Templates rapports PDF (semestriel, annuel)

---

### 4️⃣ Système de Vote et AG (17% - CRITIQUE)

| Fonctionnalité | État | Note |
|----------------|------|------|
| Auth forte (eID/itsme) | ❌ | **BLOQUANT LÉGAL** - votes non reconnus sans ça |
| Vote à distance | ❌ | Pas d'entity Vote ni endpoints |
| Types majorité | 🟡 | Calcul manuel possible, pas automatique |
| Calcul selon quotités | ❌ | Ownership_percentage existe, calcul vote manquant |
| PV automatique votes | ❌ | Pas de génération détaillée |
| Archivage résultats | 🟡 | Documents existent, lien votes manquant |

**Note légale**: En Belgique, les votes à distance pour AG nécessitent **authentification forte** (eID ou itsme). Sans ça, votes contestables juridiquement.

**Roadmap**:
- Issue #48 (Phase 1) - Strong Auth itsme/eID ⏱️ 8-10 jours
- Issue #46 (Phase 2) - Voting System ⏱️ 8-10 jours (dépend de #48)

**Recommandations**:
1. 🔴 **Registration itsme immédiate** - 2-4 semaines délai externe
2. 🔴 **Entity Vote**:
   ```rust
   struct Vote {
       id: Uuid,
       meeting_id: Uuid,
       agenda_item_id: Uuid,
       voter_id: Uuid, // Owner or User
       vote_option: VoteOption, // For/Against/Abstain
       vote_weight: f64, // Based on ownership_percentage
       signature_oidc: String, // itsme signature
       voted_at: DateTime<Utc>,
   }
   ```
3. 🔴 **Majority calculation engine**:
   - Simple (> 50%)
   - 2/3 (≥ 66.67%)
   - 3/4 (≥ 75%)
   - Unanimité (100%)
4. 🔴 **Endpoints**:
   - `POST /meetings/:id/votes` - Cast vote
   - `GET /meetings/:id/votes/results` - Real-time results
   - `PUT /meetings/:id/votes/close` - Close voting
   - `GET /meetings/:id/votes/export-pdf` - PV with votes

---

### 5️⃣ Documents Légaux (13% - CRITIQUE)

| Document | État | Priorité | Conformité Légale |
|----------|------|----------|-------------------|
| État daté | ❌ | 🔴 Critique | **Obligatoire mutations** |
| Pré-état daté | ❌ | 🟡 High | Facultatif mais pratique courante |
| PCN complet | 🟡 | 🔴 Critique | **Obligatoire notaires** |
| Quittances charges | 🟡 | 🔴 Critique | **Obligatoire propriétaires** |
| Annexes comptables | ❌ | 🔴 Critique | **Obligatoire AG** |
| PV format légal | 🟡 | 🔴 Critique | **Obligatoire AG** |

**Roadmap**: Issue #47 (Phase 2 K3s) - PDF Generation Extended ⏱️ 5-7 jours

**Recommandations**:
1. 🔴 **Templates PDF conformes**:
   - État daté (Articles 577-2 Code Civil belge)
   - PCN (Précompte Charge Notariale)
   - Quittances charges (avec détail répartition)
   - PV AG (format légal avec présents, votes, décisions)
   - Annexes comptables (bilan, compte résultat)
2. 🔴 **Watermarks officiels** - Tampon numérique copropriété
3. 🔴 **Multi-langue** - FR/NL/DE/EN selon région
4. 🔴 **Signatures électroniques** - eIDAS compliant

**Bibliothèques suggérées**:
- `printpdf` ou `genpdf` (Rust PDF generation)
- `wkhtmltopdf` (HTML → PDF avec templates Handlebars)

---

### 6️⃣ Carnet d'Entretien (0% - Important)

**Aucune fonctionnalité implémentée.**

**Contexte légal**: Le carnet d'entretien est **recommandé** (pas strictement obligatoire) mais devient de facto indispensable pour:
- Traçabilité travaux
- Garanties décennales
- Contrôles techniques obligatoires (ascenseurs, chaufferies, etc.)

**Recommandations**:
1. 🟡 **Entity WorkReport**:
   ```rust
   struct WorkReport {
       id: Uuid,
       building_id: Uuid,
       contractor_id: Option<Uuid>,
       work_type: WorkType, // Maintenance, Repair, Inspection, Installation
       description: String,
       work_date: NaiveDate,
       cost: Option<Decimal>,
       warranty_until: Option<NaiveDate>,
       documents: Vec<Uuid>, // Photos, invoices, certificates
       created_by: Uuid,
   }
   ```
2. 🟡 **Entity TechnicalInspection**:
   ```rust
   struct TechnicalInspection {
       id: Uuid,
       building_id: Uuid,
       inspection_type: InspectionType, // Elevator, Boiler, Fire, Electrical
       next_inspection_date: NaiveDate,
       responsible_entity: String, // Organisme agréé
       alert_before_days: i32, // Default 30 days
   }
   ```
3. 🟡 **Alertes automatiques** - Cron job notifications contrôles techniques
4. 🟡 **Timeline travaux** - UI chronologique avec photos avant/après

**Effort estimé**: 8-10 jours (Medium-Large)

---

### 7️⃣ Modules Communautaires (0% - Nice-to-Have)

**Contexte mission ASBL**: KoproGo vise à "résoudre phénomènes des sociétés" via lien social.

| Module | Description | Entités nécessaires | Effort |
|--------|-------------|---------------------|--------|
| **SEL** | Troc compétences (heures) | `SkillOffer`, `SkillExchange` | Large |
| **Bazar Troc** | Échange/don objets | `SwapItem`, `SwapTransaction` | Medium |
| **Prêt Objets** | Bibliothèque outils | `ObjectLoan`, `LoanRequest` | Medium |
| **Annuaire Compétences** | Listing habitants | Extension `Owner` + tags | Small |
| **Tableau Affichage** | Petites annonces | `Notice` | Small |

**Roadmap**: Issue #49 (Phase 2 K3s) - Community Features ⏱️ 10-12 jours

**Recommandations**:
1. 🟢 Implémenter Phase 2 selon roadmap
2. 🟢 **Metrics impact** - Tracking économies réalisées (€, heures, CO2)
3. 🟢 **Gamification** - Points participation, badges
4. 🟢 **Moderation tools** - Signalement contenu inapproprié

**Note**: Non-critique pour MVP, mais **différenciateur fort** vs concurrents classiques.

---

### 8️⃣ RGPD (60% - Bon mais incomplet)

| Article | Fonctionnalité | État | Note |
|---------|----------------|------|------|
| **15** | Droit d'accès | ✅ | Export JSON complet |
| **17** | Droit effacement | ✅ | Anonymisation cascade |
| **16** | Rectification | ❌ | Endpoints manquants |
| **18** | Restriction | ❌ | Endpoints manquants |
| **21** | Objection | ❌ | Endpoints manquants |
| **30** | Audit logs | ✅ | Retention 7 ans |

**Frontend**:
- ✅ `GdprDataPanel.svelte` - Export & erasure pour users
- ✅ `admin/AdminGdprPanel.svelte` - Admin GDPR avec audit logs

**Recommandations**:
1. 🟡 **Article 16 - Rectification**:
   - `PATCH /gdpr/rectify` - User profile updates avec validation
   - Logs correction demandes
2. 🟡 **Article 18 - Restriction**:
   - `POST /gdpr/restrict` - Temporary freeze processing
   - Flag `processing_restricted` sur User entity
3. 🟡 **Article 21 - Objection**:
   - `POST /gdpr/object` - Object to marketing/profiling
   - Table `marketing_consents`
4. 🟡 **Rate limiting** - Max 10 requests/hour (mention exists, impl?)
5. 🟡 **Email notifications** - Confirmation export/erase

**Effort estimé**: 5-7 jours (Phase 7-8 roadmap)

---

### 9️⃣ Infrastructure & Sécurité (29% - Insuffisant Production)

| Composant | État | Roadmap Issue | Priorité |
|-----------|------|---------------|----------|
| LUKS encryption | ❌ | #39 (3-5j) | 🔴 Critique |
| Backups GPG+S3 | ❌ | #40 (5-7j) | 🔴 Critique |
| Monitoring (Prom/Graf) | ❌ | #41 (5-7j) | 🔴 Critique |
| Security hardening | ❌ | #43 (3-5j) | 🟡 High |
| Docker Compose | ✅ | - | ✅ |
| GitOps VPS | ✅ | - | ✅ |
| K3s migration | ❌ | Phase 2 | 🟡 |
| K8s HA | ❌ | Phase 3 | 🟢 |

**Recommandations**:
1. 🔴 **Phase 1 Infrastructure (16-24 jours) à prioriser AVANT production**
2. 🔴 **LUKS** - Full-disk encryption GDPR compliance
3. 🔴 **Backups** - Rétention 7d/4w/12m avec tests restauration
4. 🔴 **Monitoring** - Dashboards + alertes (disk >80%, RAM >90%, PG down)
5. 🟡 **fail2ban + CrowdSec** - Protection attaques brute-force

---

### 🔟 Intégrations & Compatibilité (25%)

| Intégration | État | Note |
|-------------|------|------|
| API REST | ✅ | 73 endpoints |
| Webhooks | 🟡 | Infrastructure prête, impl manquante |
| SSO (SAML/OIDC) | ❌ | Grandes organisations |
| Comptables externes | ❌ | Export formats standard manquants |
| Import/Export CSV | 🟡 | Basique, pas universel |

**Recommandations**:
1. 🟡 **Webhooks** - Events: expense.paid, meeting.created, owner.added
2. 🟡 **Export comptable** - Format CODA (Belgique), CSV normalisé
3. 🟢 **SSO** - SAML pour grandes copropriétés (> 100 lots)

---

## 🎯 Plan d'Action Priorisé

### Phase 1 VPS MVP (Nov 2025 - Fév 2026) - Consolidation

#### 🔴 CRITIQUES (À faire AVANT production)

1. **Infrastructure Sécurité** (16-24 jours)
   - [ ] #39 LUKS encryption (3-5j)
   - [ ] #40 Backups GPG+S3 (5-7j)
   - [ ] #41 Monitoring stack (5-7j)
   - [ ] #43 Security hardening (3-5j)

2. **Authentification Forte** (8-10 jours)
   - [ ] #48 itsme/eID integration (BLOQUANT VOTES LÉGAUX)
   - [ ] OIDC backend (crate openidconnect)
   - [ ] Frontend "Se connecter avec itsme"

3. **Conseil de Copropriété** (8-10 jours)
   - [ ] #51 Board Tools (voir détails ci-dessus)
   - [ ] Rôle BoardMember
   - [ ] Dashboard suivi décisions AG
   - [ ] Tracking délais + alertes

4. **Documents Légaux Essentiels** (5-7 jours)
   - [ ] État daté (mutations)
   - [ ] PCN complet PDF
   - [ ] Quittances charges PDF
   - [ ] Templates PV AG conformes

#### 🟡 IMPORTANTES

5. **Gestion Financière** (10-12 jours)
   - [ ] Plan comptable belge normalisé
   - [ ] Budget prévisionnel annuel (entity + use cases)
   - [ ] Workflow recouvrement impayés (relances auto)
   - [ ] Module devis travaux (comparaison)

6. **GDPR Compléments** (5-7 jours)
   - [ ] #42 Articles 16, 18, 21
   - [ ] Rate limiting
   - [ ] Email confirmations

#### 🟢 NICE-TO-HAVE

7. **File Upload UI** (3-5 jours)
   - [ ] #45 Drag & drop complet
   - [ ] #44 Storage strategy decision (MinIO vs S3)

---

### Phase 2 K3s (Mar - Mai 2026) - Features Avancées

#### Roadmap Existante
- [ ] #47 PDF Generation Extended (5-7j)
- [ ] #46 Voting System (8-10j) - **Dépend de #48**
- [ ] #49 Community Features (10-12j)
- [ ] #52 Contractor Backoffice (8-10j)

#### Ajouts Recommandés
- [ ] Carnet d'Entretien complet (8-10j)
- [ ] Convocations AG automatiques (3-5j)
- [ ] Workflow PV 30 jours (2-3j)

---

### Phase 3 K8s Production (Jun - Août 2026) - Scale & Performance

#### Roadmap Existante
- [ ] ScyllaDB/DragonflyDB (cache performance)
- [ ] Real-time WebSocket notifications
- [ ] Advanced Analytics Dashboard
- [ ] Mobile App (React Native/Flutter)
- [ ] ElasticSearch/MeiliSearch
- [ ] Audit Dashboard visualisation

#### Ajouts Recommandés
- [ ] Accessibilité WCAG 2.1 AA (5-7j)
- [ ] Tests charge 1000-1500 copros/vCPU
- [ ] Métriques CO2 tracking (sustainability)

---

## 📊 Matrice Effort vs Impact

| Fonctionnalité | Effort | Impact Légal | Impact Métier | Priorité Finale |
|----------------|--------|--------------|---------------|-----------------|
| itsme/eID Auth | Large | 🔴 Critique | 🔴 Critique | **P0** |
| LUKS + Backups | Large | 🔴 Critique | 🔴 Critique | **P0** |
| Conseil Copropriété | Large | 🔴 Critique | 🔴 Critique | **P0** |
| État daté | Medium | 🔴 Critique | 🔴 Critique | **P0** |
| Monitoring | Large | 🟡 High | 🔴 Critique | **P1** |
| Plan comptable | Large | 🔴 Critique | 🟡 High | **P1** |
| Voting System | Large | 🔴 Critique | 🟡 High | **P1** |
| Budget prévisionnel | Medium | 🟡 High | 🟡 High | **P2** |
| Carnet entretien | Medium | 🟢 Medium | 🟡 High | **P2** |
| GDPR Articles 16/18/21 | Medium | 🟡 High | 🟢 Medium | **P2** |
| Community Features | X-Large | 🟢 Low | 🟢 Medium | **P3** |
| Mobile App | X-Large | 🟢 Low | 🟡 High | **P3** |
| Accessibilité WCAG | Medium | 🟡 High | 🟢 Medium | **P3** |

**Légende Priorité**:
- **P0**: BLOQUANT production légale
- **P1**: CRITIQUE pour MVP complet
- **P2**: IMPORTANTE pour qualité
- **P3**: NICE-TO-HAVE différenciation

---

## 🚨 Risques Identifiés

### 1. Risques Légaux (CRITIQUES)

| Risque | Impact | Probabilité | Mitigation |
|--------|--------|-------------|------------|
| **Votes sans auth forte contestés** | 🔴 Très Élevé | 🟡 Moyenne | Registration itsme IMMÉDIATE |
| **GDPR non-conforme (amendes)** | 🔴 Très Élevé | 🟡 Moyenne | Implémenter Articles 16/18/21 |
| **État daté non-conforme (notaires refusent)** | 🔴 Élevé | 🔴 Élevée | Templates PDF conformes |
| **Pas de conseil (>20 lots illégal)** | 🔴 Élevé | 🟡 Moyenne | Board Tools Phase 1 |

### 2. Risques Techniques

| Risque | Impact | Probabilité | Mitigation |
|--------|--------|-------------|------------|
| **Perte données (pas backup)** | 🔴 Catastrophique | 🟢 Faible | Backups GPG+S3 Phase 1 |
| **Breach sécurité (pas LUKS)** | 🔴 Très Élevé | 🟡 Moyenne | LUKS encryption Phase 1 |
| **Downtime invisible (pas monitoring)** | 🟡 Élevé | 🔴 Élevée | Monitoring stack Phase 1 |
| **Performance dégradation** | 🟡 Moyen | 🟡 Moyenne | Benchmarks continus |

### 3. Risques Business

| Risque | Impact | Probabilité | Mitigation |
|--------|--------|-------------|------------|
| **Comptabilité non-conforme → churn** | 🟡 Élevé | 🔴 Élevée | Plan comptable Phase 1 |
| **Pas de différenciation (Community)** | 🟢 Moyen | 🟡 Moyenne | Phase 2 Community Features |
| **Adoption lente (pas mobile)** | 🟡 Moyen | 🟡 Moyenne | Phase 3 Mobile App |

---

## 📈 Recommandations Stratégiques

### 1. Avant Production (CRITIQUE)

**NE PAS lancer en production sans**:
1. ✅ LUKS encryption at rest
2. ✅ Backups automatisés chiffrés
3. ✅ Monitoring stack opérationnel
4. ✅ Security hardening (fail2ban, CrowdSec)
5. ✅ Conseil de Copropriété (obligation légale >20 lots)
6. ✅ État daté génération (mutations immobilières)

**Durée estimée**: 42-59 jours (Phase 1 roadmap) = **9-13 semaines**

### 2. MVP Légal Minimum (P0 + P1)

Pour être **légalement opérable** en Belgique:
1. Authentification forte itsme (votes valides)
2. Conseil de Copropriété (>20 lots)
3. Documents légaux (état daté, PCN, PV conformes)
4. Plan comptable normalisé belge
5. RGPD complet (Articles 15-17 OK, ajouter 16/18/21)
6. Infrastructure sécurisée (LUKS, backups, monitoring)

**Total effort**: ~100 jours développement (5 mois avec 1 dev)

### 3. Différenciation Marché (P2-P3)

**Avantages compétitifs KoproGo**:
- ✅ **Open source** (unique en Belgique)
- ✅ **Multi-tenant moderne** (vs logiciels desktop)
- ✅ **PWA offline-first** (vs web apps classiques)
- 🟡 **Community features** (SEL, troc) - **À développer Phase 2**
- 🟡 **Performance exceptionnelle** (< 5ms P99) - **À valider charge**
- 🟡 **Sustainability** (< 0.5g CO2/req) - **À instrumenter**

**Positionnement**: "Copropriété durable & solidaire" (mission ASBL)

### 4. Roadmap Validée

La roadmap actuelle (ROADMAP.rst) est **bien structurée** et couvre la majorité des gaps:
- ✅ Phase 1 VPS MVP: Sécurité, GDPR, Board Tools (9-13 sem)
- ✅ Phase 2 K3s: Voting, Community, Contractor (6-8 sem)
- ✅ Phase 3 K8s: Performance, Mobile, Analytics (6-8 sem)

**Ajustements recommandés**:
1. Ajouter **Plan comptable belge** en Phase 1 (critique)
2. Ajouter **État daté** en Phase 1 (critique)
3. Ajouter **Carnet entretien** en Phase 2
4. Prioriser **Accessibilité WCAG** en Phase 3

---

## ✅ Checklist de Lancement Production

### Légal & Conformité
- [ ] itsme/eID authentication fonctionnelle
- [ ] Conseil de Copropriété complet
- [ ] État daté génération conforme
- [ ] PCN + Quittances PDF conformes
- [ ] PV AG format légal belge
- [ ] RGPD Articles 15-21 complets
- [ ] Privacy Policy publiée
- [ ] Terms of Service publiés

### Infrastructure & Sécurité
- [ ] LUKS encryption at rest
- [ ] Backups automatisés GPG+S3 testés
- [ ] Monitoring stack opérationnel (Prometheus, Grafana, Loki)
- [ ] Alertes configurées (disk, RAM, PostgreSQL)
- [ ] fail2ban + CrowdSec WAF actifs
- [ ] Certificates HTTPS (Let's Encrypt)
- [ ] Tests de restauration backups passés

### Fonctionnel
- [ ] Plan comptable belge implémenté
- [ ] Budget prévisionnel fonctionnel
- [ ] Workflow recouvrement impayés
- [ ] Convocations AG automatiques
- [ ] Voting system avec auth forte
- [ ] Carnet d'entretien (si >20 lots)

### Qualité & Tests
- [ ] Tests E2E complets (> 100 tests)
- [ ] Tests charge validés (1000 req/s min)
- [ ] Tests accessibilité WCAG 2.1 AA
- [ ] Documentation utilisateur complète
- [ ] Guide administrateur copropriété

### Performance
- [ ] P99 latency < 5ms validé
- [ ] Memory usage < 128MB/instance
- [ ] Database pool optimisé (max 10 conn)
- [ ] Cache Redis/DragonflyDB (optionnel Phase 3)

---

## 📞 Contacts & Ressources

### Références Légales Belgique
- **Code Civil belge** - Article 577-2 et suivants (Copropriété)
- **RGPD** - Règlement (UE) 2016/679
- **Autorité Protection Données** - https://www.autoriteprotectiondonnees.be/
- **itsme** - https://www.itsme.be/en/businesses
- **eID** - https://eid.belgium.be/

### Documentation Projet
- **ROADMAP.rst** - `/home/user/koprogo/docs/ROADMAP.rst`
- **CLAUDE.md** - `/home/user/koprogo/CLAUDE.md`
- **Issues GitHub** - https://github.com/gilmry/koprogo/issues

---

## 📝 Conclusion

### Résumé État Actuel

KoproGo a une **base solide** (29% fonctionnalités complètes):
- ✅ Architecture hexagonale bien structurée
- ✅ Multi-tenant/multi-role/multi-owner complet
- ✅ CRUD complet entités core
- ✅ PWA offline-first moderne
- ✅ GDPR Articles 15 & 17 (60%)

### Gaps Critiques (BLOQUANTS Production)

1. 🔴 **Authentification forte** (itsme/eID) - Votes non reconnus légalement
2. 🔴 **Conseil de Copropriété** - Obligation légale >20 lots
3. 🔴 **Documents légaux** - État daté, PCN conformes
4. 🔴 **Infrastructure sécurité** - LUKS, backups, monitoring
5. 🔴 **Plan comptable belge** - Conformité comptable

### Effort Total Estimé

- **MVP Légal** (P0+P1): ~100 jours = **20 semaines** (1 dev)
- **Roadmap complète** (Phase 1-3): ~133-168 jours = **27-34 semaines**

### Recommandation Finale

**GO pour Phase 1 VPS MVP** avec ajustements:
1. Ajouter Plan comptable + État daté (critiques légaux)
2. Prioriser Infrastructure avant Software (sécurité d'abord)
3. Registration itsme IMMÉDIATE (2-4 sem délai externe)
4. Ne PAS lancer production sans checklist complète

**Target réaliste production**: **Avril-Mai 2026** (fin Phase 1 + début Phase 2)

---

**Généré le**: 2025-11-01
**Analyse par**: Claude Code Agent SDK
**Version document**: 1.0
