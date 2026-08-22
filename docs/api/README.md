# KoproGo API Specification (OpenAPI 3.0)

Ce dossier contient la spécification OpenAPI 3.0 complète de l'API KoproGo.

## 📄 Fichiers

- **`openapi.yaml`** : Spécification OpenAPI 3.0 complète de l'API

## 🚀 Utilisation

### 1. Visualiser avec Swagger UI (en ligne)

Ouvrez [Swagger Editor](https://editor.swagger.io/) et importez le fichier `openapi.yaml`.

### 2. Swagger UI local (Docker)

```bash
docker run -p 8081:8080 \
  -e SWAGGER_JSON=/api/openapi.yaml \
  -v $(pwd)/docs/api:/api \
  swaggerapi/swagger-ui
```

Accédez à http://localhost:8081

### 3. Redoc (alternative élégante)

```bash
docker run -p 8082:80 \
  -e SPEC_URL=openapi.yaml \
  -v $(pwd)/docs/api:/usr/share/nginx/html \
  redocly/redoc
```

Accédez à http://localhost:8082

### 4. Générer des clients API

**TypeScript/JavaScript** :

```bash
npm install @openapitools/openapi-generator-cli -g
openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g typescript-axios \
  -o frontend/src/lib/api-client
```

**Python** :

```bash
openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g python \
  -o clients/python
```

**Rust** :

```bash
openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g rust \
  -o clients/rust
```

**Autres langages supportés** : Java, Go, PHP, Ruby, C#, Swift, Kotlin, etc.
Liste complète : https://openapi-generator.tech/docs/generators

### 5. Importer dans Postman/Insomnia

**Postman** :

1. Fichier → Import
2. Sélectionner `openapi.yaml`
3. Toutes les requêtes sont créées automatiquement

**Insomnia** :

1. Application → Preferences → Data → Import Data
2. Sélectionner `openapi.yaml`

## 📚 Structure de l'API

### Tags principaux

- **Authentication** : Login, JWT, switch role
- **Buildings** : Gestion des immeubles
- **Units** : Gestion des lots
- **Owners** : Gestion des copropriétaires
- **Unit Owners** : Relations multi-propriétaires
- **Expenses** : Dépenses et factures (workflow d'approbation)
- **Accounts** : Comptabilité PCMN belge
- **Financial Reports** : Bilan, compte de résultats
- **Payment Reminders** : Relances automatisées (4 niveaux)
- **Meetings** : Assemblées générales
- **Documents** : Gestion documentaire
- **GDPR** : Conformité RGPD (Art. 15, 17, 20)
- **Health** : Monitoring et métriques

### Authentification

Toutes les routes (sauf `/health`) requièrent un JWT Bearer token :

```bash
Authorization: Bearer <token>
```

Obtenir un token :

```bash
curl -X POST https://api.koprogo.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "syndic@example.com",
    "password": "secure_password_123"
  }'
```

### Endpoints clés

#### Bâtiments

- `GET /buildings` - Liste des immeubles
- `POST /buildings` - Créer un immeuble
- `GET /buildings/{id}` - Détails d'un immeuble

#### Multi-propriétaires

- `GET /units/{unit_id}/owners` - Copropriétaires actifs d'un lot
- `POST /units/{unit_id}/owners` - Ajouter un copropriétaire
- `GET /units/{unit_id}/owners/total-percentage` - Vérifier somme quotes-parts
- `POST /units/{unit_id}/owners/transfer` - Transférer propriété

#### Workflow dépenses

- `POST /expenses` - Créer dépense (Draft)
- `PUT /expenses/{id}/submit-for-approval` - Soumettre (Draft → PendingApproval)
- `PUT /expenses/{id}/approve` - Approuver (PendingApproval → Approved)
- `PUT /expenses/{id}/reject` - Rejeter avec motif
- `PUT /expenses/{id}/mark-paid` - Marquer comme payée

#### Comptabilité PCMN

- `GET /accounts` - Liste comptes PCMN
- `GET /accounts/code/{code}` - Recherche par code (ex: 451000)
- `POST /accounts/seed/belgian-pcmn` - Seed 90 comptes PCMN
- `GET /reports/balance-sheet?year=2025` - Bilan comptable
- `GET /reports/income-statement?year=2025&quarter=4` - Compte de résultats

#### Relances de paiement

- `POST /payment-reminders` - Créer relance
- `PUT /payment-reminders/{id}/escalate` - Escalader niveau
- `GET /payment-reminders/stats` - Statistiques recouvrement

#### GDPR

- `GET /gdpr/owners/{id}/data-export` - Export données (Art. 15)
- `GET /gdpr/owners/{id}/portable-data` - Données portables (Art. 20)
- `DELETE /gdpr/owners/{id}/delete-data` - Droit à l'effacement (Art. 17)

## 🔧 Intégration dans le backend Rust

### Option 1 : Service Swagger UI statique

Ajoutez au `docker-compose.yml` :

```yaml
services:
  swagger-ui:
    image: swaggerapi/swagger-ui
    container_name: koprogo-swagger-ui
    ports:
      - "8081:8080"
    environment:
      - SWAGGER_JSON=/api/openapi.yaml
      - BASE_URL=/api/docs
    volumes:
      - ./docs/api:/api
    networks:
      - koprogo-network
```

### Option 2 : Intégrer avec utoipa (à venir)

Pour générer automatiquement la spec depuis le code Rust, ajouter au `Cargo.toml` :

```toml
[dependencies]
utoipa = { version = "5.1", features = ["actix_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "8.0", features = ["actix-web"] }
```

Exemple d'utilisation :

```rust
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_buildings,
        create_building,
        // ... autres endpoints
    ),
    components(
        schemas(Building, CreateBuildingRequest, /* ... */)
    ),
    tags(
        (name = "Buildings", description = "Building management endpoints")
    )
)]
struct ApiDoc;

// Dans main.rs
App::new()
    .service(SwaggerUi::new("/api/docs/{_:.*}")
        .url("/api/docs/openapi.json", ApiDoc::openapi()))
```

## 📊 Validation

### Validation de la spec

```bash
# Avec Spectral
npm install -g @stoplight/spectral-cli
spectral lint docs/api/openapi.yaml

# Avec openapi-generator validator
openapi-generator-cli validate -i docs/api/openapi.yaml
```

### Tests automatisés

**Dredd** (test contract-driven) :

```bash
npm install -g dredd
dredd docs/api/openapi.yaml http://localhost:8080
```

**Schemathesis** (property-based testing) :

```bash
pip install schemathesis
schemathesis run docs/api/openapi.yaml --base-url http://localhost:8080
```

## 📝 Maintenance

### Mise à jour de la spec

Lors de l'ajout de nouveaux endpoints :

1. Ajouter les schémas dans `components/schemas`
2. Ajouter les chemins dans `paths`
3. Valider avec `spectral lint`
4. Tester avec Swagger UI
5. Commit les changements

### Versionning

La spec suit le versionnement sémantique :

- **Major** : Breaking changes (ex: 2.0.0)
- **Minor** : Nouveaux endpoints (ex: 1.1.0)
- **Patch** : Corrections (ex: 1.0.1)

## 🌐 Déploiement public

La spec OpenAPI sera disponible publiquement à :

- **Swagger UI** : https://api.koprogo.com/docs
- **ReDoc** : https://api.koprogo.com/redoc
- **Spec JSON** : https://api.koprogo.com/openapi.json
- **Spec YAML** : https://api.koprogo.com/openapi.yaml

## 📚 Ressources

- [OpenAPI 3.0 Specification](https://spec.openapis.org/oas/v3.0.3)
- [Swagger UI](https://swagger.io/tools/swagger-ui/)
- [ReDoc](https://redocly.com/redoc/)
- [OpenAPI Generator](https://openapi-generator.tech/)
- [Spectral Linter](https://stoplight.io/open-source/spectral)

## 🤝 Contribution

Pour améliorer la spec OpenAPI :

1. Modifiez `openapi.yaml`
2. Validez avec `spectral lint`
3. Testez avec Swagger UI local
4. Créez une pull request

---

**Version** : 1.0.0 | **Dernière mise à jour** : 2026-05-20
