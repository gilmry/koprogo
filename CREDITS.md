# Open Source Credits

KoproGo utilise et s'inspire des projets open source suivants. Nous remercions chaleureusement les auteurs et contributeurs de ces projets pour leur travail exceptionnel.

## 🐳 Containerised Services (Microservices)

### Novu - Multi-Channel Notification Infrastructure
- **License**: MIT
- **GitHub**: https://github.com/novuhq/novu
- **Website**: https://novu.co/
- **Usage in KoproGo**: Microservice pour notifications multi-canal (email, SMS, push, in-app)
- **Issues**: #86 (Multi-Channel Notifications), #88 (AG Convocations)
- **Authors**: Novu team
- **Why we chose it**: Architecture moderne, support multi-canal natif, MIT license compatible

### ThingsBoard - Open-Source IoT Platform
- **License**: Apache 2.0
- **GitHub**: https://github.com/thingsboard/thingsboard
- **Website**: https://thingsboard.io/
- **Usage in KoproGo**: Plateforme IoT pour monitoring bâtiment (capteurs, alertes maintenance)
- **Issues**: #89 (Digital Maintenance Logbook)
- **Authors**: ThingsBoard team
- **Why we chose it**: Support MQTT/CoAP/HTTP, dashboards temps réel, Apache 2.0 license

## 💡 Inspirations (Reimplemented in Rust)

Ces projets nous ont inspirés mais ont été **réimplémentés en Rust** pour:
- Compatibilité avec notre architecture hexagonale
- Conformité licences (éviter contamination GPL/AGPL)
- Performance et cohérence stack Rust
- Adaptation au contexte belge (PCMN, copropriété)

### Diacamma Syndic - Gestion de Copropriété
- **License**: GPL-3.0
- **GitHub**: https://github.com/Diacamma2/syndic
- **Website**: https://www.diacamma.org/
- **Inspiration for**: Workflows comptables, procédures assemblées générales, états financiers
- **Issues**: #75 (Meeting Management), #77 (Financial Reports), #81 (Budget System)
- **Authors**: Diacamma2 team (syndics bénévoles et développeurs associatifs)
- **What we learned**: Structure comptable copropriété, règles métier AG, génération documents légaux
- **Note**: Code GPL non utilisé directement - patterns métier étudiés et réimplémentés

### TimeOverflow - Time Banking System
- **License**: AGPL-3.0
- **GitHub**: https://github.com/coopdevs/timeoverflow
- **Website**: https://www.timeoverflow.org/
- **Inspiration for**: Système d'Échange Local (SEL), crédits temps, échange de services
- **Issues**: #49 (Community Features - SEL), #99 (Community Modules)
- **Authors**: Coopdevs cooperative
- **What we learned**: Modèle 1 heure = 1 crédit, gestion balance crédits, annuaire compétences
- **Note**: Code AGPL non utilisé directement - patterns SEL/LETS réimplémentés

### Condo - Open Source Property Management SaaS
- **License**: MIT
- **GitHub**: https://github.com/open-condo-software/condo
- **Website**: https://opencondo.app/
- **Inspiration for**: Système de ticketing, marketplace contractants, suivi paiements
- **Issues**: #52 (Contractor Backoffice), #85 (Maintenance Ticketing), #84 (Online Payment)
- **Authors**: Open Condo Software team
- **What we learned**: Architecture marketplace, workflow ticketing, intégration paiements
- **Note**: Code MIT - patterns étudiés et adaptés architecture Rust

### ElectionGuard - End-to-End Verifiable Voting
- **License**: MIT
- **GitHub**: https://github.com/Election-Tech-Initiative/electionguard
- **Website**: https://www.electionguard.vote/
- **Inspiration for**: Concepts cryptographiques vote (chiffrement homomorphe, vérifiabilité)
- **Issues**: #46 (Meeting Voting System)
- **Authors**: Microsoft + Election Tech Initiative
- **What we learned**: Chiffrement homomorphe, bulletins vérifiables, audit tiers
- **Note**: Concepts cryptographiques uniquement - implémentation trop complexe pour AG copropriété

### Helios Voting - Web-Based Voting System
- **License**: Apache 2.0 (backend) + GPL-3.0 (frontend)
- **GitHub**: https://github.com/benadida/helios-server
- **Website**: https://vote.heliosvoting.org/
- **Inspiration for**: Vote électronique open-audit
- **Issues**: #46 (Meeting Voting System)
- **Authors**: Ben Adida
- **What we learned**: Architecture vote en ligne, audit trail, vérification
- **Note**: Patterns conceptuels étudiés

### Paperless-ngx - Document Management System
- **License**: GPL-3.0
- **GitHub**: https://github.com/paperless-ngx/paperless-ngx
- **Inspiration for**: Gestion documentaire, OCR, archivage
- **Issues**: #76 (Document Upload/Download)
- **Authors**: Paperless-ngx community
- **What we learned**: Patterns upload/download, métadonnées documents, stockage S3
- **Note**: Code GPL non utilisé - patterns DMS réimplémentés avec actix-files + S3

## 🦀 Rust Libraries (Direct Usage)

Bibliothèques Rust directement utilisées dans KoproGo:

### Web Framework & HTTP
- **actix-web** (Apache-2.0 / MIT) - Framework web asynchrone
- **actix-multipart** (Apache-2.0 / MIT) - Upload fichiers multipart
- **actix-files** (Apache-2.0 / MIT) - Serving fichiers statiques
- **reqwest** (Apache-2.0 / MIT) - HTTP client async

### Database & Persistence
- **sqlx** (Apache-2.0 / MIT) - PostgreSQL avec compile-time query checking
- **sea-query** (Apache-2.0 / MIT) - Query builder SQL
- **redis** (BSD-3-Clause) - Redis client

### Async Runtime
- **tokio** (MIT) - Runtime asynchrone
- **async-trait** (Apache-2.0 / MIT) - Traits async

### Serialization
- **serde** (Apache-2.0 / MIT) - Serialization framework
- **serde_json** (Apache-2.0 / MIT) - JSON support

### PDF Generation
- **genpdf** (Apache-2.0 / MIT) - High-level PDF generation
- **printpdf** (MIT) - Low-level PDF library

### Email
- **lettre** (MIT) - Email client SMTP

### IoT & Messaging
- **rumqttc** (Apache-2.0) - MQTT client
- **coap** (MIT) - CoAP protocol for IoT

### Cryptography & Security
- **argon2** (Apache-2.0 / MIT) - Password hashing
- **sha2** (Apache-2.0 / MIT) - SHA-256 hashing
- **ed25519-dalek** (BSD-3-Clause) - Digital signatures
- **jsonwebtoken** (MIT) - JWT tokens

### AWS & Cloud
- **aws-sdk-s3** (Apache-2.0) - AWS S3 client
- **rusoto_s3** (MIT) - Alternative S3 client

### Utilities
- **uuid** (Apache-2.0 / MIT) - UUID generation
- **chrono** (Apache-2.0 / MIT) - Date and time
- **dotenv** (MIT) - Environment variables
- **tracing** (MIT) - Structured logging
- **thiserror** (Apache-2.0 / MIT) - Error handling

### Payment Processing
- **async-stripe** (Apache-2.0 / MIT) - Stripe API client

### Testing
- **cucumber** (MIT) - BDD testing framework
- **testcontainers** (MIT) - Integration testing avec Docker
- **criterion** (Apache-2.0 / MIT) - Benchmarking

Voir `backend/Cargo.toml` pour la liste complète avec versions.

## 🌐 Frontend Libraries

### Framework & UI
- **Astro** (MIT) - Static site generator
- **Svelte** (MIT) - Reactive UI framework
- **Tailwind CSS** (MIT) - Utility-first CSS

### Build Tools
- **Vite** (MIT) - Fast build tool
- **TypeScript** (Apache-2.0) - Type safety

Voir `frontend/package.json` pour la liste complète.

## 🔧 Infrastructure & DevOps

### Containerization
- **Docker** - Container platform
- **Docker Compose** - Multi-container orchestration

### Monitoring (Issues #39, #40, #41)
- **Prometheus** (Apache-2.0) - Metrics collection
- **Grafana** (AGPL-3.0) - Visualization dashboards
- **Loki** (AGPL-3.0) - Log aggregation
- **Alertmanager** (Apache-2.0) - Alert routing

### Security (Issue #78)
- **Suricata** (GPL-2.0) - Intrusion detection system (IDS)
- **CrowdSec** (MIT) - Collaborative Web Application Firewall
- **fail2ban** (GPL-2.0) - Intrusion prevention
- **Lynis** (GPL-3.0) - Security auditing tool

### Database
- **PostgreSQL 15** (PostgreSQL License) - Primary database
- **Redis 7** (BSD-3-Clause) - Caching and sessions

## 🎓 Educational Resources & Standards

### Belgian Accounting
- **Plan Comptable Minimum Normalisé (PCMN)** - Arrêté Royal 12/07/2012
- **Belgian Copropriété Law** - Code Civil Livre III, Titre VIII bis

### EU Regulations
- **GDPR** - General Data Protection Regulation (EU 2016/679)
- **eIDAS** - Electronic Identification and Trust Services (EU 910/2014)

### Standards
- **MQTT Protocol** - ISO/IEC 20922
- **CoAP Protocol** - RFC 7252
- **JWT** - RFC 7519
- **OAuth 2.0** - RFC 6749
- **OpenID Connect** - OpenID Foundation

## 🙏 Special Thanks

### Communities
- **Rust Community** - Pour l'écosystème extraordinaire de crates
- **Actix Community** - Pour le meilleur framework web Rust
- **Coopdevs** - Pour leur travail sur l'économie sociale (TimeOverflow)
- **Diacamma2** - Pour leur engagement envers les syndics bénévoles
- **Open Condo Software** - Pour l'inspiration architecture SaaS
- **Novu Team** - Pour la plateforme de notifications moderne
- **ThingsBoard Team** - Pour la plateforme IoT open source

### Open Source Philosophy

KoproGo est construit sur les épaules de géants. Nous croyons fermement en l'open source et nous nous engageons à:

1. **Respecter les licences** - Conformité stricte GPL/MIT/Apache
2. **Citer les sources** - Attribution claire des inspirations
3. **Contribuer en retour** - Bug reports, PRs upstream quand possible
4. **Partager nos innovations** - KoproGo sera open source (à définir: MIT ou Apache 2.0)

## 📄 License Compatibility Matrix

| Notre Usage | Licences Compatibles | Action |
|-------------|---------------------|--------|
| **Containerisation (Microservice)** | MIT, Apache 2.0, BSD | ✅ Utilisation directe |
| **Containerisation** | GPL-2.0, GPL-3.0 | ⚠️ Risque contamination - éviter |
| **Containerisation** | AGPL-3.0 | ❌ Contamination réseau - interdit |
| **Bibliothèques Rust (linkage)** | MIT, Apache 2.0, BSD | ✅ Utilisation directe |
| **Bibliothèques Rust** | GPL | ❌ Contamination - éviter |
| **Inspiration code** | Toutes licences | ✅ Avec citation + réimplémentation |

## 🔗 Useful Links

- **KoproGo GitHub**: https://github.com/gilmry/koprogo
- **KoproGo Roadmap**: [docs/ROADMAP.md](docs/ROADMAP.md)
- **Open Source Research**: [docs/OPENSOURCE_PROPTECH_RESEARCH.rst](docs/OPENSOURCE_PROPTECH_RESEARCH.rst)
- **Security Documentation**: [infrastructure/SECURITY.md](infrastructure/SECURITY.md)

---

**Dernière mise à jour**: 2025-11-10
**Maintenu par**: KoproGo Core Team

*Si vous constatez une omission ou une erreur dans les attributions, merci de nous contacter ou d'ouvrir une issue.*
