# KoproGo MCP Server

Serveur MCP (Model Context Protocol) pour la plateforme KoproGo de gestion de copropriétés belges.

## Vision

Le serveur MCP de KoproGo permet à un agent IA (Claude, etc.) d'interagir avec la plateforme
pour **assister** le syndic, le copropriétaire, ou tout utilisateur dans ses tâches quotidiennes.

L'agent IA agit comme un **assistant juridique et opérationnel** :
- Il connaît la base légale belge (`docs/legal/`)
- Il accède aux données de la copropriété via les outils MCP
- Il guide l'utilisateur en citant les articles de loi pertinents
- Il ne prend jamais de décision à la place de l'utilisateur

## Architecture

```
┌─────────────────┐     MCP/SSE      ┌──────────────────────┐
│   Claude / AI   │ ◄──────────────► │  KoproGo MCP Server  │
│   (claude.ai)   │                  │  (Rust / Actix-web)  │
└─────────────────┘                  └──────────┬───────────┘
                                                │
                                     ┌──────────▼───────────┐
                                     │   KoproGo Backend     │
                                     │   (hexagonal arch.)   │
                                     │   PostgreSQL          │
                                     └──────────────────────┘
```

Le serveur MCP est un **adaptateur** dans l'architecture hexagonale existante de KoproGo.
Il expose les ports de l'application via le protocole MCP (JSON-RPC 2.0 over SSE).

### Couches internes (code existant)

```
core/          - Domain entities (McpRequest, McpResponse, ModelInfo)
ports/         - Trait definitions (McpService, ModelRegistry, McpRepository)
adapters/      - Implementations (PostgreSQL, EdgeClient, Actix handlers)
```

### Structure cible

```
backend/koprogo-mcp/
├── src/
│   ├── lib.rs
│   ├── core/                  # Domain (existant)
│   │   ├── entities.rs            # McpRequest, McpResponse, ModelInfo, McpTask
│   │   └── services.rs            # McpRequestService, CarbonFootprintService
│   ├── ports/                 # Interfaces (existant)
│   │   ├── mcp_service.rs         # McpService trait
│   │   ├── mcp_repository.rs      # McpRepository trait
│   │   └── model_registry.rs      # ModelRegistry trait
│   ├── adapters/              # Implementations (existant)
│   │   ├── postgres_repository.rs # PostgreSQL logging
│   │   ├── edge_client.rs         # Raspberry Pi edge inference
│   │   └── actix_handlers.rs      # REST handlers (Phase 0)
│   ├── mcp/                   # ← NOUVEAU : adaptateur MCP standard
│   │   ├── mod.rs
│   │   ├── server.rs              # SSE endpoint + JSON-RPC handler
│   │   ├── auth.rs                # JWT + matrice rôle/outil
│   │   └── tools/                 # Implémentation de chaque outil
│   │       ├── legal.rs               # legal_search, majority_calculator
│   │       ├── copropriete.rs         # copropriete_info, list_coproprietaires
│   │       ├── ag.rs                  # ag_create, ag_vote, ag_quorum, ag_pv
│   │       ├── comptabilite.rs        # comptabilite_situation, appel_de_fonds
│   │       ├── travaux.rs             # travaux_qualifier
│   │       ├── transmission.rs        # transmission_lot_dossier
│   │       ├── alertes.rs             # alertes_list
│   │       └── documents.rs           # documents_list, document_generate
│   └── bin/
│       └── mcp_cli.rs         # CLI (existant)
└── Cargo.toml
```

## Endpoint

```
https://app.koprogo.be/mcp/sse
```

Authentification : Bearer token (JWT) lié au compte utilisateur KoproGo.
Les outils accessibles dépendent du **rôle** de l'utilisateur connecté.

## Phase 0 : Infrastructure IA (implémenté)

Fondation technique multi-modèle avec routage éco-responsable :

- **Edge (0g CO₂)** : Raspberry Pi local inference (Llama 3, Mistral)
- **Cloud (0.3g CO₂/1k tokens)** : API calls (Claude, GPT-4)
- **Grid (distributed)** : Heavy tasks (OCR, traduction, résumés)

### Fonctionnalités Phase 0

- ✅ Multi-model support (Llama 3, Claude, GPT-4, Mistral)
- ✅ Edge computing avec failover multi-nœuds
- ✅ Grid computing (tâches distribuées)
- ✅ CO₂ tracking par requête
- ✅ Request/response logging (PostgreSQL)
- ✅ Token usage statistics
- ✅ Priority queuing (scoring 1-100)
- ✅ CLI (`mcp-cli`)

### API REST Phase 0

| Endpoint | Méthode | Description |
|----------|---------|-------------|
| `/mcp/v1/chat` | POST | Chat completion |
| `/mcp/v1/models` | GET | Liste des modèles |
| `/mcp/v1/execute` | POST | Exécution tâche grid |
| `/mcp/v1/tasks/{id}` | GET | Statut tâche |
| `/mcp/v1/stats` | GET | Statistiques d'usage |
| `/mcp/v1/health` | GET | Health check |

### Usage CLI

```bash
cargo run --bin mcp-cli chat --model llama3:8b "Explain GDPR compliance"
cargo run --bin mcp-cli models
cargo run --bin mcp-cli health
```

### Usage en tant que bibliothèque

```rust
use koprogo_mcp::*;

let request = McpRequest::new(
    "llama3:8b".to_string(),
    vec![Message::user("Summarize meeting notes".to_string())],
    Some("copro:123".to_string()),
)?;

let edge_client = EdgeClient::new(vec!["http://localhost:3031".to_string()]);
let response = edge_client.execute_on_edge(&request).await?;

println!("Response: {}", response.content);
println!("CO₂ saved: {:.4}g", response.calculate_co2_grams());
```

## Phase 1 : Outils MCP métier (à implémenter)

15 outils exposés via le protocole MCP standard (JSON-RPC over SSE), organisés par domaine.

---

### 1. Référence légale

#### `legal_search`
Recherche dans la base légale par mot-clé, code de règle, ou rôle.

```json
{
  "name": "legal_search",
  "description": "Recherche dans la base légale belge de la copropriété. Utiliser pour citer les articles de loi, expliquer les obligations, ou vérifier la conformité d'une action.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": { "type": "string", "description": "Mot-clé ou question (ex: 'majorité travaux parties communes')" },
      "rule_id": { "type": "string", "description": "Code de règle spécifique (ex: 'AG09', 'M02', 'CP07')" },
      "role": { "type": "string", "enum": ["syndic", "coproprietaire", "locataire", "commissaire", "conseil", "acp", "notaire"] },
      "category": { "type": "string", "enum": ["mandat", "ag", "finance", "travaux", "deontologie", "transmission"] }
    }
  }
}
```

#### `majority_calculator`
Détermine la majorité requise pour un type de décision donné.

```json
{
  "name": "majority_calculator",
  "description": "Calcule la majorité requise (absolue, 2/3, 4/5, unanimité) selon le type de décision à l'AG. Cite l'article du Code civil applicable.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "decision_type": { "type": "string", "description": "Ex: 'travaux parties communes', 'modification statuts', 'révocation syndic', 'modification quotités'" },
      "copropriete_id": { "type": "string", "description": "ID de la copropriété (pour calcul concret avec quotes-parts)" }
    },
    "required": ["decision_type"]
  }
}
```

---

### 2. Gestion de copropriété

#### `copropriete_info`
Informations générales sur une copropriété gérée.

```json
{
  "name": "copropriete_info",
  "description": "Récupère les informations d'une copropriété : adresse, n° BCE, nombre de lots, syndic en fonction, date d'expiration du mandat, etc.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "copropriete_id": { "type": "string" }
    },
    "required": ["copropriete_id"]
  }
}
```

#### `list_coproprietaires`
Liste des copropriétaires avec quotes-parts.

```json
{
  "name": "list_coproprietaires",
  "description": "Liste des copropriétaires d'une copropriété avec leurs lots, quotes-parts, et coordonnées.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "copropriete_id": { "type": "string" },
      "include_locataires": { "type": "boolean", "default": false }
    },
    "required": ["copropriete_id"]
  }
}
```

---

### 3. Assemblée générale

#### `ag_create`
Créer une assemblée générale avec ordre du jour.

```json
{
  "name": "ag_create",
  "description": "Crée une nouvelle AG (ordinaire ou extraordinaire). Génère un OdJ conforme au Code civil belge avec les points obligatoires (art. 3.89 §5 12° et 16°). L'OdJ est automatiquement séquencé selon les dépendances légales.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "copropriete_id": { "type": "string" },
      "type": { "type": "string", "enum": ["ordinaire", "extraordinaire"] },
      "date": { "type": "string", "format": "date-time" },
      "lieu": { "type": "string" },
      "points_supplementaires": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "titre": { "type": "string" },
            "description": { "type": "string" },
            "type_majorite": { "type": "string", "enum": ["absolue", "deux_tiers", "quatre_cinquiemes", "unanimite"] },
            "proposé_par": { "type": "string", "description": "Nom du copropriétaire ou 'syndic' ou 'conseil'" }
          }
        }
      },
      "mode_participation": { "type": "string", "enum": ["presentiel", "hybride"], "default": "presentiel" }
    },
    "required": ["copropriete_id", "type", "date"]
  }
}
```

#### `ag_quorum_check`
Vérifier le quorum d'une AG en cours.

```json
{
  "name": "ag_quorum_check",
  "description": "Calcule le quorum de présence pour une AG : total quotes-parts présentes/représentées vs 50% requis. Si quorum non atteint, indique la procédure de 2e AG (art. 3.87 §5).",
  "inputSchema": {
    "type": "object",
    "properties": {
      "ag_id": { "type": "string" }
    },
    "required": ["ag_id"]
  }
}
```

#### `ag_vote`
Enregistrer un vote sur un point de l'OdJ.

```json
{
  "name": "ag_vote",
  "description": "Enregistre les votes sur un point de l'OdJ. Calcule automatiquement si la majorité requise est atteinte. Tient compte du plafonnement à 50% des voix (art. 3.87 §6).",
  "inputSchema": {
    "type": "object",
    "properties": {
      "ag_id": { "type": "string" },
      "point_id": { "type": "string" },
      "votes": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "coproprietaire_id": { "type": "string" },
            "vote": { "type": "string", "enum": ["pour", "contre", "abstention"] }
          }
        }
      }
    },
    "required": ["ag_id", "point_id", "votes"]
  }
}
```

#### `ag_generate_pv`
Générer le procès-verbal de l'AG.

```json
{
  "name": "ag_generate_pv",
  "description": "Génère le PV de l'AG conforme à l'art. 3.87 §10 : majorités obtenues, noms des opposants et abstentionnistes. Le PV doit être transmis dans les 30 jours.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "ag_id": { "type": "string" },
      "format": { "type": "string", "enum": ["pdf", "docx"], "default": "pdf" }
    },
    "required": ["ag_id"]
  }
}
```

---

### 4. Comptabilité et charges

#### `comptabilite_situation`
Situation financière de l'ACP.

```json
{
  "name": "comptabilite_situation",
  "description": "Récupère la situation financière : soldes fonds de roulement et fonds de réserve, charges en cours, arriérés par copropriétaire.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "copropriete_id": { "type": "string" },
      "exercice": { "type": "string", "description": "Année comptable (ex: '2025')" }
    },
    "required": ["copropriete_id"]
  }
}
```

#### `appel_de_fonds`
Générer un appel de fonds.

```json
{
  "name": "appel_de_fonds",
  "description": "Génère un appel de fonds (ordinaire ou spécial) réparti selon les quotes-parts. Conforme à l'art. 3.86 §3 : le syndic communique la part affectée au fonds de réserve.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "copropriete_id": { "type": "string" },
      "montant_total": { "type": "number" },
      "motif": { "type": "string" },
      "type": { "type": "string", "enum": ["ordinaire", "special"] },
      "echeance": { "type": "string", "format": "date" }
    },
    "required": ["copropriete_id", "montant_total", "motif", "type"]
  }
}
```

---

### 5. Travaux

#### `travaux_qualifier`
Qualifier un travail (urgent/conservatoire vs non-urgent).

```json
{
  "name": "travaux_qualifier",
  "description": "Aide à qualifier un travail : urgent/conservatoire (syndic peut agir seul, art. 3.89 §5 2°) vs non-urgent (nécessite décision AG, art. 3.88 §1 1°b). Détermine aussi la majorité requise.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "description_travaux": { "type": "string" },
      "copropriete_id": { "type": "string" },
      "montant_estime": { "type": "number" }
    },
    "required": ["description_travaux"]
  }
}
```

---

### 6. Transmission de lot

#### `transmission_lot_dossier`
Générer le dossier art. 3.94 pour le notaire.

```json
{
  "name": "transmission_lot_dossier",
  "description": "Génère le dossier complet d'information art. 3.94 pour la vente d'un lot : montants FR/FdR, arriérés, appels de fonds, PV des 3 dernières années, dernier bilan, procédures en cours. Délai légal : 15 jours.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "copropriete_id": { "type": "string" },
      "lot_id": { "type": "string" },
      "format": { "type": "string", "enum": ["pdf", "zip"], "default": "pdf" }
    },
    "required": ["copropriete_id", "lot_id"]
  }
}
```

---

### 7. Alertes et rappels

#### `alertes_list`
Liste des alertes actives pour l'utilisateur.

```json
{
  "name": "alertes_list",
  "description": "Liste les alertes et rappels actifs : expiration mandat syndic, renouvellement RC, formation IPI, délai de convocation, fonds de réserve insuffisant, BCE, PV non envoyé, etc.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "copropriete_id": { "type": "string" },
      "role": { "type": "string", "enum": ["syndic", "coproprietaire", "commissaire", "conseil"] }
    },
    "required": ["copropriete_id"]
  }
}
```

---

### 8. Documents

#### `documents_list`
Lister les documents de la copropriété.

```json
{
  "name": "documents_list",
  "description": "Liste les documents d'une copropriété : statuts, ROI, PV d'AG, contrats, polices d'assurance, devis, factures.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "copropriete_id": { "type": "string" },
      "categorie": { "type": "string", "enum": ["statuts", "pv", "contrats", "assurances", "comptabilite", "travaux", "tous"] }
    },
    "required": ["copropriete_id"]
  }
}
```

#### `document_generate`
Générer un document type.

```json
{
  "name": "document_generate",
  "description": "Génère un document type : convocation AG, procuration, contrat syndic, rapport évaluation contrats, affichage entrée immeuble, etc.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "copropriete_id": { "type": "string" },
      "type_document": {
        "type": "string",
        "enum": [
          "convocation_ag",
          "procuration",
          "contrat_syndic",
          "rapport_contrats",
          "affichage_entree",
          "appel_de_fonds",
          "pv_ag",
          "dossier_transmission_lot",
          "inventaire_transition_syndic"
        ]
      },
      "parametres": { "type": "object", "description": "Paramètres spécifiques au type de document" }
    },
    "required": ["copropriete_id", "type_document"]
  }
}
```

---

### 9. Achat groupé d'énergie

#### `energie_campagne_list`

Liste des campagnes d'achat groupé actives ou passées.

```json
{
  "name": "energie_campagne_list",
  "description": "Liste les campagnes d'achat groupé d'énergie : actives, en inscription, en enchère, ou terminées. Permet de voir le volume agrégé et le nombre de participants.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "statut": {
        "type": "string",
        "enum": ["inscription", "enchere", "offre", "switching", "terminee", "toutes"],
        "default": "toutes"
      },
      "region": {
        "type": "string",
        "enum": ["bruxelles", "wallonie", "flandre", "toutes"],
        "default": "toutes"
      }
    }
  }
}
```

#### `energie_inscrire`

Inscrire un membre ou une ACP à une campagne.

```json
{
  "name": "energie_inscrire",
  "description": "Inscrit un membre KoproGo (copropriétaire individuel, propriétaire maison) ou une ACP (compteurs parties communes) à une campagne d'achat groupé. Gratuit et sans engagement.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "campagne_id": { "type": "string" },
      "type_inscription": {
        "type": "string",
        "enum": ["individuel", "acp_parties_communes"]
      },
      "ean_electricite": { "type": "string", "description": "Code EAN du compteur électricité (18 chiffres)" },
      "ean_gaz": { "type": "string", "description": "Code EAN du compteur gaz (optionnel)" },
      "consommation_elec_kwh": { "type": "number", "description": "Consommation annuelle électricité en kWh" },
      "consommation_gaz_kwh": { "type": "number", "description": "Consommation annuelle gaz en kWh (optionnel)" },
      "fournisseur_actuel": { "type": "string" },
      "type_compteur": {
        "type": "string",
        "enum": ["mono", "bihoraire", "exclusif_nuit", "smart"]
      },
      "preference_tarif": {
        "type": "string",
        "enum": ["fixe", "variable", "indifferent"],
        "default": "indifferent"
      },
      "energie_verte": { "type": "boolean", "default": true },
      "copropriete_id": { "type": "string", "description": "ID copropriété si inscription ACP" }
    },
    "required": ["campagne_id", "type_inscription", "ean_electricite", "consommation_elec_kwh"]
  }
}
```

#### `energie_offre_personnalisee`

Récupérer l'offre personnalisée d'un membre.

```json
{
  "name": "energie_offre_personnalisee",
  "description": "Récupère l'offre personnalisée générée pour un membre dans une campagne : fournisseur gagnant, prix/kWh, économie estimée, comparaison avec le contrat actuel.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "campagne_id": { "type": "string" },
      "membre_id": { "type": "string" }
    },
    "required": ["campagne_id", "membre_id"]
  }
}
```

#### `energie_comparer_tarif`

Comparer le tarif actuel avec l'offre groupée et le marché.

```json
{
  "name": "energie_comparer_tarif",
  "description": "Compare le tarif énergétique actuel d'un membre avec l'offre de l'achat groupé et avec le marché (via les données CREG). Décompose le prix en : énergie, transport, distribution, taxes.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "membre_id": { "type": "string" },
      "campagne_id": { "type": "string" },
      "type_energie": {
        "type": "string",
        "enum": ["electricite", "gaz", "les_deux"],
        "default": "les_deux"
      }
    },
    "required": ["membre_id"]
  }
}
```

#### `energie_ag_point`

Générer le point OdJ pour le changement de fournisseur énergie des parties communes.

```json
{
  "name": "energie_ag_point",
  "description": "Génère le point d'ordre du jour pour soumettre le changement de fournisseur énergie des parties communes à l'AG. Inclut le comparatif offre groupée vs contrat actuel et la majorité requise (art. 3.88). Vérifie le seuil de mise en concurrence fixé par l'AG.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "copropriete_id": { "type": "string" },
      "campagne_id": { "type": "string" },
      "ag_id": { "type": "string", "description": "ID de l'AG où inscrire le point" }
    },
    "required": ["copropriete_id", "campagne_id"]
  }
}
```

---

## Matrice rôle x outil

| Outil | Syndic | Copropriétaire | Locataire | Commissaire | CdC |
|-------|--------|----------------|-----------|-------------|-----|
| `legal_search` | oui | oui | oui | oui | oui |
| `majority_calculator` | oui | oui | - | oui | oui |
| `copropriete_info` | oui | oui (limité) | - | oui | oui |
| `list_coproprietaires` | oui | oui | - | oui | oui |
| `ag_create` | oui | - | - | - | - |
| `ag_quorum_check` | oui | - | - | - | - |
| `ag_vote` | oui | - | - | - | - |
| `ag_generate_pv` | oui | - | - | - | - |
| `comptabilite_situation` | oui | oui (son lot) | - | oui | oui |
| `appel_de_fonds` | oui | - | - | - | - |
| `travaux_qualifier` | oui | oui | - | - | oui |
| `transmission_lot_dossier` | oui | oui (son lot) | - | - | - |
| `alertes_list` | oui | oui | - | oui | oui |
| `documents_list` | oui | oui (non privé) | oui (ROI) | oui | oui |
| `document_generate` | oui | - | - | - | - |
| `energie_campagne_list` | oui | oui | oui | - | oui |
| `energie_inscrire` | oui | oui | oui | - | - |
| `energie_offre_personnalisee` | oui | oui (son offre) | oui | - | - |
| `energie_comparer_tarif` | oui | oui | oui | - | oui |
| `energie_ag_point` | oui | - | - | - | - |

> **Note** : Les locataires avec compteur individuel peuvent participer à l'achat groupé
> via le rôle « occupant énergie », sans passer par le syndic.

## Prompt système recommandé pour l'agent IA

```
Tu es l'assistant KoproGo, spécialisé dans la gestion de copropriétés en Belgique.

Tu disposes d'outils MCP pour accéder aux données de la copropriété de l'utilisateur
et à la base légale belge (Code civil art. 3.78-3.100, Code de déontologie IPI, etc.)

Principes :
1. Toujours citer la source légale (article + paragraphe) quand tu donnes un conseil
2. Ne jamais prendre de décision à la place de l'utilisateur
3. Alerter sur les risques juridiques (délais, majorités, incompatibilités)
4. Adapter ton langage au rôle : pédagogique pour un syndic bénévole, technique pour un pro
5. Répondre en français (Belgique)
```

## Cas d'usage prioritaires

### 1. Syndic bénévole prépare sa première AG
L'agent utilise `copropriete_info` -> `ag_create` -> `legal_search` pour générer un OdJ conforme avec explications à chaque point.

### 2. Copropriétaire conteste une décision
L'agent utilise `legal_search("CP07")` pour expliquer le délai de 4 mois et les conditions (préjudice personnel, décision irrégulière/frauduleuse/abusive).

### 3. Vente d'un appartement
L'agent utilise `transmission_lot_dossier` pour générer le pack notaire, explique la distinction FR (remboursable) vs FdR (non remboursable).

### 4. Travaux urgents en cours
L'agent utilise `travaux_qualifier` pour confirmer que le syndic peut agir seul (art. 3.89 §5 2°) puis `document_generate("appel_de_fonds")` pour financer.

### 5. Vérification de conformité
L'agent utilise `alertes_list` pour dresser un état des lieux : mandat expiré ? RC à jour ? BCE inscrit ? Fonds de réserve suffisant ?

## Database Schema

Tables PostgreSQL requises :

**Phase 0 (existant)** :
- `mcp_models` - Registre des modèles IA
- `mcp_requests` - Logs des requêtes
- `mcp_responses` - Logs des réponses
- `mcp_tasks` - Tâches grid

## Testing

```bash
# Tests unitaires (100% couverture domain)
cargo test --lib

# Tests d'intégration (testcontainers)
cargo test --test integration

# Tous les tests
cargo test
```

## Stack technique

- **Langage** : Rust (cohérent avec le backend KoproGo)
- **Framework** : Actix-web (REST Phase 0 + SSE Phase 1)
- **Protocole** : MCP (JSON-RPC 2.0 over Server-Sent Events)
- **Auth** : JWT Bearer token (même auth que l'API KoproGo)
- **Base légale** : `docs/legal/` (fichiers .md avec codes de règles)

## License

AGPL-3.0 - See LICENSE
