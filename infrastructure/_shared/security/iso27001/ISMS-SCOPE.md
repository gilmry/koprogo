# ISMS Scope - KoproGo

## ISO 27001:2022 - Clause 4.3

### Scope Statement

The Information Security Management System (ISMS) covers the design, development,
deployment, and operation of the KoproGo SaaS property management platform,
including:

- Web application (backend API + frontend)
- Database systems (PostgreSQL)
- File storage (MinIO/S3)
- Infrastructure (OVH VPS, K3s/K8s clusters)
- CI/CD pipelines (GitHub Actions)
- Monitoring and logging systems

### Boundaries

- **Geographic**: EU (Belgium, France - OVH Gravelines datacenter)
- **Organizational**: Development team, Operations, Support
- **Technology**: Rust backend, Astro/Svelte frontend, PostgreSQL, Docker/K8s
- **Legal**: Belgian law, GDPR (EU 2016/679), Belgian Copropriete law

### Exclusions

- Physical datacenter security (managed by OVH - ISO 27001 certified)
- End-user device management
