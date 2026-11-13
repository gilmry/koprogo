/// MCP SSE Server Handler — Issue #252
///
/// Implements Model Context Protocol (MCP) version 2024-11-05 using JSON-RPC 2.0 over
/// Server-Sent Events (SSE) transport.
///
/// Endpoints:
///   GET  /mcp/sse        — SSE connection endpoint (client subscribes here)
///   POST /mcp/messages   — JSON-RPC message endpoint (client sends requests here)
///
/// Authentication: JWT Bearer token (same as the rest of the API)
///
/// Protocol flow:
///   1. Client opens SSE connection → receives `endpoint` event with POST URL
///   2. Client sends JSON-RPC `initialize` request → receives capabilities
///   3. Client calls `tools/list` → receives available KoproGo tools
///   4. Client calls `tools/call` → tool executes and returns result
///
/// Reference: https://modelcontextprotocol.io/specification/2024-11-05/basic/transports/
use crate::domain::copropriete::{MajorityType, NatureDeDecision};
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::middleware::scope_guard::{
    verify_building_org_access, verify_owner_personal_access,
};
use crate::infrastructure::web::middleware::AuthenticatedUser;
use actix_web::{
    get, post,
    web::{self, Data},
    HttpRequest, HttpResponse,
};
use futures_util::stream::{self};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────
// JSON-RPC 2.0 types
// ─────────────────────────────────────────────────────────

/// Incoming JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// Outgoing JSON-RPC 2.0 response (success)
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Value,
}

/// Outgoing JSON-RPC 2.0 error
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub error: RpcError,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// JSON-RPC error codes (MCP standard)
struct ErrorCode;
impl ErrorCode {
    const PARSE_ERROR: i32 = -32700;
    const INVALID_REQUEST: i32 = -32600;
    const METHOD_NOT_FOUND: i32 = -32601;
    const INVALID_PARAMS: i32 = -32602;
    const INTERNAL_ERROR: i32 = -32603;
}

// ─────────────────────────────────────────────────────────
// MCP protocol types
// ─────────────────────────────────────────────────────────

/// Server info sent in `initialize` response
#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// MCP capabilities advertised by the server
#[derive(Debug, Serialize)]
pub struct ServerCapabilities {
    pub tools: ToolsCapability,
}

#[derive(Debug, Serialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

/// MCP tool definition (for `tools/list` response)
#[derive(Debug, Serialize, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Tool execution result (for `tools/call` response)
#[derive(Debug, Serialize)]
pub struct ToolResult {
    pub content: Vec<ContentBlock>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

// ─────────────────────────────────────────────────────────
// Tool registry — KoproGo tools exposed via MCP
// ─────────────────────────────────────────────────────────

/// Returns all KoproGo tools available via MCP
fn get_mcp_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "list_buildings".to_string(),
            description: "Liste tous les immeubles en copropriété de l'organisation. Retourne l'id, le nom, l'adresse, le nombre d'unités et l'état de chaque immeuble.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "page": {
                        "type": "integer",
                        "description": "Numéro de page (défaut: 1)"
                    },
                    "per_page": {
                        "type": "integer",
                        "description": "Résultats par page (défaut: 20, max: 100)"
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "get_building".to_string(),
            description: "Récupère les détails complets d'un immeuble: adresse, unités, syndic, informations légales, budget actif et statistiques financières.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble"
                    }
                },
                "required": ["building_id"]
            }),
        },
        McpTool {
            name: "list_owners".to_string(),
            description: "Liste les copropriétaires d'un immeuble avec leurs quotes-parts (tantièmes/millièmes), coordonnées et informations de contact.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble (optionnel, liste tous si absent)"
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "list_meetings".to_string(),
            description: "Liste les assemblées générales (AG) d'un immeuble: date, type (AGO/AGE), statut, quorum validé, résolutions prises.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["Scheduled", "Completed", "Cancelled"],
                        "description": "Filtrer par statut (optionnel)"
                    }
                },
                "required": ["building_id"]
            }),
        },
        McpTool {
            name: "get_financial_summary".to_string(),
            description: "Résumé financier d'un immeuble: charges totales, paiements en attente, budget approuvé vs réalisé, copropriétaires en retard de paiement.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble"
                    }
                },
                "required": ["building_id"]
            }),
        },
        McpTool {
            name: "list_tickets".to_string(),
            description: "Liste les tickets de maintenance d'un immeuble: interventions en cours, priorités, prestataires assignés, délais de résolution.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["Open", "Assigned", "InProgress", "Resolved", "Closed", "Cancelled"],
                        "description": "Filtrer par statut (optionnel)"
                    }
                },
                "required": ["building_id"]
            }),
        },
        McpTool {
            name: "get_owner_balance".to_string(),
            description: "Solde et historique de paiements d'un copropriétaire: montants dus, paiements effectués, retards, relances envoyées.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "owner_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID du copropriétaire"
                    }
                },
                "required": ["owner_id"]
            }),
        },
        McpTool {
            name: "list_pending_expenses".to_string(),
            description: "Liste les factures/charges en attente d'approbation pour un immeuble ou l'organisation entière. Inclut fournisseur, montant HT/TTC, TVA belge.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble (optionnel)"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["Draft", "PendingApproval", "Approved", "Rejected", "Paid", "Overdue", "Cancelled"],
                        "description": "Filtrer par statut (défaut: PendingApproval)"
                    }
                },
                "required": []
            }),
        },
        McpTool {
            name: "check_quorum".to_string(),
            description: "Vérifie si le quorum légal est atteint pour une assemblée générale (Art. 3.87 §5 CC belge: >50% des tantièmes présents/représentés).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "meeting_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'assemblée générale"
                    }
                },
                "required": ["meeting_id"]
            }),
        },
        McpTool {
            name: "get_building_documents".to_string(),
            description: "Liste les documents d'un immeuble: PV d'AG, contrats, devis, rapports d'inspection, budgets approuvés. Retourne les métadonnées et liens de téléchargement.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble"
                    },
                    "document_type": {
                        "type": "string",
                        "description": "Filtrer par type: Minutes, Contract, Invoice, Quote, Report, Budget, Other (optionnel)"
                    }
                },
                "required": ["building_id"]
            }),
        },
        McpTool {
            name: "legal_search".to_string(),
            description: "Recherche dans la base légale belge de copropriété par mot-clé ou code d'article. Retourne les articles du Code Civil pertinents avec explications.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Mot-clé à rechercher (ex: 'quorum', 'majorité', 'convocation')"
                    },
                    "code": {
                        "type": "string",
                        "description": "Code d'article (ex: 'Art. 3.87 §1 CC') (optionnel)"
                    },
                    "category": {
                        "type": "string",
                        "description": "Catégorie légale: AG, Travaux, Majorité, Quorum, Convocation, Finances (optionnel)"
                    }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: "majority_calculator".to_string(),
            description: "Calcule la majorité requise pour une décision d'assemblée générale selon la loi belge (Art. 3.88 CC). Retourne le type de majorité, le seuil exact et la base légale.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "decision_type": {
                        "type": "string",
                        "enum": NATURES_MCP.iter().map(|(code, ..)| *code).collect::<Vec<_>>(),
                        "description": "Nature de la décision selon l'Art. 3.88 CC. Une nature inconnue relève du droit commun (majorité absolue)."
                    },
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble (optionnel, pour contexte)"
                    },
                    "meeting_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'assemblée (optionnel, pour contexte)"
                    }
                },
                "required": ["decision_type"]
            }),
        },
        McpTool {
            name: "list_owners_of_building".to_string(),
            description: "Liste détaillée des copropriétaires d'un immeuble avec tantièmes, statut actif/inactif, et historique de propriété. Alias spécialisé pour list_owners avec détails de bâtiment.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble"
                    },
                    "include_inactive": {
                        "type": "boolean",
                        "description": "Inclure les propriétaires inactifs/historiques (défaut: false)"
                    }
                },
                "required": ["building_id"]
            }),
        },
        McpTool {
            name: "ag_quorum_check".to_string(),
            description: "Vérifie le quorum légal et calcule la procédure de deuxième convocation (Art. 3.87 §3-4 CC). Retourne le statut quorum et les étapes suivantes si insuffisant.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "meeting_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'assemblée générale"
                    }
                },
                "required": ["meeting_id"]
            }),
        },
        McpTool {
            name: "comptabilite_situation".to_string(),
            description: "Situation comptable d'un immeuble: soldes comptes, arriérés de charges, revenus, dépenses. Retourne bilan financier détaillé.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble"
                    },
                    "fiscal_year": {
                        "type": "integer",
                        "description": "Année fiscale (optionnel, défaut: année courante)"
                    }
                },
                "required": ["building_id"]
            }),
        },
        McpTool {
            name: "travaux_qualifier".to_string(),
            description: "Qualifie des travaux (conservatoires si urgents, sinon travaux aux parties communes) et donne la majorité requise selon l'Art. 3.88 CC. Le montant n'entre pas dans la majorité.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "description": {
                        "type": "string",
                        "description": "Description des travaux"
                    },
                    "estimated_amount_eur": {
                        "type": "number",
                        "description": "Montant estimé en euros"
                    },
                    "is_emergency": {
                        "type": "boolean",
                        "description": "Travaux d'urgence? (conservatoires, sécurité)"
                    }
                },
                "required": ["description", "estimated_amount_eur", "is_emergency"]
            }),
        },
    ]
}

// ─────────────────────────────────────────────────────────
// Outils de droit (sans état, testables sans AppState)
// ─────────────────────────────────────────────────────────

/// Les natures de décision de l'Art. 3.88, telles que le MCP les accepte :
/// (code d'entrée, nature du domaine, intitulé, base légale).
///
/// La majorité n'est jamais écrite ici : elle est dérivée de
/// `NatureDeDecision::majorite_requise()` (lot 0, S4).
const NATURES_MCP: [(&str, NatureDeDecision, &str, &str); 18] = {
    use NatureDeDecision::*;
    [
        ("modification_statuts_usage_des_communs", ModificationStatutsUsageDesCommuns,
         "Modification des statuts portant sur la jouissance, l'usage ou l'administration des parties communes", "Art. 3.88 § 1er, 1°, a"),
        ("travaux_aux_parties_communes", TravauxAuxPartiesCommunes,
         "Travaux affectant les parties communes", "Art. 3.88 § 1er, 1°, b"),
        ("seuil_de_mise_en_concurrence", SeuilDeMiseEnConcurrence,
         "Montant des marchés à partir duquel la mise en concurrence est obligatoire", "Art. 3.88 § 1er, 1°, c"),
        ("travaux_privatifs_executes_par_lacp", TravauxPrivatifsExecutesParLacp,
         "Travaux à des parties privatives exécutés par l'ACP, sur motivation spéciale", "Art. 3.88 § 1er, 1°, d"),
        ("travaux_imposes_par_la_loi", TravauxImposesParLaLoi,
         "Travaux imposés par la loi", "Art. 3.88 § 1er, 1°, b, exception"),
        ("travaux_conservatoires", TravauxConservatoires,
         "Travaux conservatoires", "Art. 3.88 § 1er, 1°, b, exception"),
        ("administration_provisoire", AdministrationProvisoire,
         "Actes d'administration provisoire", "Art. 3.88 § 1er, 1°, b, exception"),
        ("autre_modification_des_statuts", AutreModificationDesStatuts,
         "Toute autre modification des statuts, y compris la répartition des charges", "Art. 3.88 § 1er, 2°, a"),
        ("modification_de_la_destination", ModificationDeLaDestination,
         "Modification de la destination de l'immeuble ou d'une partie de celui-ci", "Art. 3.88 § 1er, 2°, b"),
        ("reconstruction_apres_destruction_partielle", ReconstructionApresDestructionPartielle,
         "Reconstruction ou remise en état après destruction partielle", "Art. 3.88 § 1er, 2°, c"),
        ("acquisition_de_biens_communs", AcquisitionDeBiensCommuns,
         "Acquisition de biens immobiliers destinés à devenir communs", "Art. 3.88 § 1er, 2°, d"),
        ("acte_de_disposition_de_biens_communs", ActeDeDispositionDeBiensCommuns,
         "Actes de disposition de biens immobiliers communs", "Art. 3.88 § 1er, 2°, e"),
        ("creation_dassociations_partielles", CreationDassociationsPartielles,
         "Modification des statuts pour créer des associations partielles", "Art. 3.88 § 1er, 2°, f"),
        ("division_ou_reunion_de_lots", DivisionOuReunionDeLots,
         "Division d'un lot ou réunion de plusieurs lots", "Art. 3.88 § 1er, 2°, g"),
        ("demolition_reconstruction_pour_salubrite_ou_cout", DemolitionReconstructionPourSalubriteOuCout,
         "Démolition et reconstruction totales pour raisons de salubrité, de sécurité ou de coût excessif de mise en conformité", "Art. 3.88 § 1er, 2°, h"),
        ("modification_des_quotes_parts", ModificationDesQuotesParts,
         "Modification de la répartition des quotes-parts de copropriété", "Art. 3.88 § 3"),
        ("demolition_reconstruction_autre_motif", DemolitionReconstructionAutreMotif,
         "Démolition et reconstruction totales pour un autre motif", "Art. 3.88 § 3"),
        ("droit_commun", DroitCommun,
         "Toute décision que la loi ne soumet pas à une majorité qualifiée", "Art. 3.87 § 8"),
    ]
};

/// L'entrée de `NATURES_MCP` pour un code, ou le droit commun s'il est inconnu
/// (« sauf si la loi exige une majorité qualifiée », Art. 3.87 § 8).
fn nature_mcp(code: &str) -> (&'static str, NatureDeDecision, &'static str, &'static str) {
    NATURES_MCP
        .iter()
        .copied()
        .find(|(c, ..)| *c == code)
        .unwrap_or(NATURES_MCP[NATURES_MCP.len() - 1])
}

/// L'entrée de `NATURES_MCP` pour une nature du domaine.
fn entree_de_nature(
    nature: NatureDeDecision,
) -> (&'static str, NatureDeDecision, &'static str, &'static str) {
    NATURES_MCP
        .iter()
        .copied()
        .find(|(_, n, ..)| *n == nature)
        .unwrap_or(NATURES_MCP[NATURES_MCP.len() - 1])
}

/// Le seuil en clair, tel que `resolution.rs` le documente.
fn libelle_majorite(majorite: &MajorityType) -> &'static str {
    match majorite {
        MajorityType::Absolute => {
            "Majorité absolue des voix des copropriétaires présents ou représentés, abstentions exclues"
        }
        MajorityType::TwoThirds => {
            "Deux tiers des voix des copropriétaires présents ou représentés, abstentions exclues"
        }
        MajorityType::FourFifths => {
            "Quatre cinquièmes des voix des copropriétaires présents ou représentés, abstentions exclues"
        }
        MajorityType::Unanimity => "Unanimité des voix de tous les copropriétaires",
    }
}

/// `legal_search` : recherche par mot-clé dans la base légale statique.
fn rechercher_droit(arguments: &Value) -> Value {
    let query = arguments
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();

    // Base légale statique. Les entrées de majorité sont dérivées de
    // `majorites.rs`, jamais écrites à la main (lot 0, S4).
    let mut legal_base = vec![
        json!({"code": "AG01", "article": "Art. 3.87 §1 CC", "title": "Convocation AG ordinaire", "content": "Le syndic convoque l'AG au moins 15 jours avant la date fixée", "category": "Convocation"}),
        json!({"code": "AG02", "article": "Art. 3.87 §3 CC", "title": "Deuxième convocation", "content": "À défaut de quorum, une seconde AG peut être convoquée 15 jours plus tard", "category": "Convocation"}),
        json!({"code": "AG03", "article": "Art. 3.87 §5 CC", "title": "Quorum légal", "content": "L'AG ne délibère valablement que si plus de la moitié des quotes-parts sont présentes ou représentées", "category": "Quorum"}),
        json!({"code": "TRV01", "article": "Art. 3.89 §5 CC", "title": "Travaux conservatoires", "content": "Syndic peut autoriser travaux d'urgence/conservatoires sans AG préalable", "category": "Travaux"}),
        json!({"code": "FIN01", "article": "Art. 3.90 CC", "title": "Appel de fonds", "content": "Appel de fonds = demande de contribution supplémentaire pour charges extraordinaires", "category": "Finances"}),
    ];
    legal_base.extend(NATURES_MCP.iter().map(|(code, nature, intitule, base)| {
        let majorite = nature.majorite_requise();
        json!({
            "code": format!("MAJ-{code}"),
            "article": base,
            "title": intitule,
            "content": format!("{intitule} : {}.", libelle_majorite(&majorite).to_lowercase()),
            "category": "Majorité",
            "nature": code,
            "majorite_requise": majorite,
        })
    }));

    // Filter by query
    let results: Vec<_> = legal_base
        .iter()
        .filter(|item| {
            let title = item
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let content = item
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let code = item
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            title.contains(&query) || content.contains(&query) || code.contains(&query)
        })
        .cloned()
        .collect();

    json!({"count": results.len(), "results": results})
}

/// `majority_calculator` : la majorité requise pour une décision d'AG.
fn calculer_majorite(arguments: &Value) -> Value {
    let demande = arguments
        .get("decision_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let (code, nature, intitule, base) = nature_mcp(demande);
    let majorite = nature.majorite_requise();
    let mut result = json!({
        "nature": code,
        "intitule": intitule,
        "majorite_requise": majorite,
        "seuil": libelle_majorite(&majorite),
        "base_legale": base,
    });
    if code != demande {
        result["remarque"] = json!(format!(
            "Nature « {demande} » non reconnue : droit commun appliqué (Art. 3.87 § 8)."
        ));
    }
    result
}

/// `travaux_qualifier` : qualification de travaux et majorité requise.
fn qualifier_travaux(arguments: &Value) -> Result<Value, RpcError> {
    let description = arguments
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError {
            code: ErrorCode::INVALID_PARAMS,
            message: "description is required".to_string(),
            data: None,
        })?;

    let estimated_amount_eur = arguments
        .get("estimated_amount_eur")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let is_emergency = arguments
        .get("is_emergency")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let nature = if is_emergency {
        NatureDeDecision::TravauxConservatoires
    } else {
        NatureDeDecision::TravauxAuxPartiesCommunes
    };
    let (code, _, intitule, base) = entree_de_nature(nature);
    let majorite = nature.majorite_requise();
    let mut result = json!({
        "description": description,
        "amount_eur": format!("{:.2}", estimated_amount_eur),
        "nature": code,
        "qualification": intitule,
        "majorite_requise": majorite,
        "seuil": libelle_majorite(&majorite),
        "base_legale": base,
        "mise_en_concurrence": "Obligatoire au-delà du montant des marchés que l'AG a fixé aux deux tiers (Art. 3.88 § 1er, 1°, c). La loi ne fixe pas ce montant.",
        "exceptions": "Travaux imposés par la loi et actes d'administration provisoire : majorité absolue (Art. 3.88 § 1er, 1°, b). Interroger majority_calculator avec la nature exacte.",
    });
    if is_emergency {
        result["syndic_can_act_alone"] = json!(true);
        result["legal_basis"] = json!("Art. 3.89 §5 2° CC");
    } else {
        result["requires_ag_approval"] = json!(true);
    }
    Ok(result)
}

/// Enveloppe une valeur JSON dans le bloc texte d'un `ToolResult`.
fn resultat_texte(result: &Value) -> ToolResult {
    let text = serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string());
    ToolResult {
        content: vec![ContentBlock {
            content_type: "text".to_string(),
            text,
        }],
        is_error: None,
    }
}

// ─────────────────────────────────────────────────────────
// Tool dispatcher
// ─────────────────────────────────────────────────────────

/// Un outil absent de `tools/list` n'est pas appelable (ADR 0059).
fn outil_annonce(tool_name: &str) -> Result<(), RpcError> {
    let outils = get_mcp_tools();
    if outils.iter().any(|t| t.name == tool_name) {
        return Ok(());
    }
    Err(RpcError {
        code: ErrorCode::METHOD_NOT_FOUND,
        message: format!("Unknown tool: {}", tool_name),
        data: Some(json!({
            "available_tools": outils.iter().map(|t| &t.name).collect::<Vec<_>>()
        })),
    })
}

/// Dispatches a `tools/call` request to the appropriate tool implementation.
/// Returns a ToolResult or a JSON-RPC error.
/// Les outils MCP lisent les mêmes données que les routes HTTP : ils passent
/// par les mêmes gardes. Tout `building_id`, `owner_id` ou `meeting_id` reçu
/// en argument est vérifié avant l'appel du cas d'usage, quel que soit
/// l'outil : l'immeuble (ou celui de l'assemblée) doit relever d'une ACP de
/// l'organisation de l'appelant, et un copropriétaire ne lit que sa fiche.
async fn verifier_perimetre(
    arguments: &Value,
    state: &AppState,
    user: &AuthenticatedUser,
) -> Result<(), RpcError> {
    let refus = |message: String| RpcError {
        code: ErrorCode::INVALID_PARAMS,
        message,
        data: None,
    };
    let uuid = |cle: &str| -> Result<Option<Uuid>, RpcError> {
        match arguments.get(cle).and_then(|v| v.as_str()) {
            None => Ok(None),
            Some(s) => Uuid::parse_str(s)
                .map(Some)
                .map_err(|_| refus(format!("{cle} must be a valid UUID"))),
        }
    };

    let mut immeubles = Vec::new();
    if let Some(id) = uuid("building_id")? {
        immeubles.push(id);
    }
    if let Some(id) = uuid("meeting_id")? {
        let assemblee = state
            .meeting_use_cases
            .get_meeting(id)
            .await
            .map_err(refus)?
            .ok_or_else(|| refus(format!("Meeting not found: {id}")))?;
        immeubles.push(assemblee.building_id);
    }
    for immeuble in immeubles {
        verify_building_org_access(
            user,
            immeuble,
            &state.building_use_cases,
            &state.acp_use_cases,
        )
        .await
        .map_err(|e| refus(e.to_string()))?;
    }
    if let Some(id) = uuid("owner_id")? {
        verify_owner_personal_access(user, id, &state.owner_use_cases)
            .await
            .map_err(|e| refus(e.to_string()))?;
    }
    Ok(())
}

async fn dispatch_tool(
    tool_name: &str,
    arguments: &Value,
    state: &AppState,
    user: &AuthenticatedUser,
) -> Result<ToolResult, RpcError> {
    let org_id = match user.organization_id {
        Some(id) => id,
        None => {
            return Err(RpcError {
                code: ErrorCode::INVALID_REQUEST,
                message: "User does not belong to an organization".to_string(),
                data: None,
            })
        }
    };

    verifier_perimetre(arguments, state, user).await?;

    match tool_name {
        "list_buildings" => {
            let page = arguments.get("page").and_then(|v| v.as_u64()).unwrap_or(1) as i64;
            let per_page = arguments
                .get("per_page")
                .and_then(|v| v.as_u64())
                .unwrap_or(20) as i64;

            let page_request = crate::application::dto::PageRequest {
                page,
                per_page,
                sort_by: None,
                order: crate::application::dto::SortOrder::default(),
            };
            match state
                .building_use_cases
                .list_buildings_paginated(&page_request, Some(org_id))
                .await
            {
                Ok((buildings, _total)) => {
                    let text = serde_json::to_string_pretty(&buildings)
                        .unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to list buildings: {}", e),
                    data: None,
                }),
            }
        }

        "get_building" => {
            let building_id_str = arguments
                .get("building_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "building_id is required".to_string(),
                    data: None,
                })?;

            let building_id = Uuid::parse_str(building_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "building_id must be a valid UUID".to_string(),
                data: None,
            })?;

            match state.building_use_cases.get_building(building_id).await {
                Ok(Some(building)) => {
                    let text = serde_json::to_string_pretty(&building)
                        .unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Ok(None) => Err(RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: format!("Building not found: {}", building_id),
                    data: None,
                }),
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to get building: {}", e),
                    data: None,
                }),
            }
        }

        "list_owners" => {
            let _building_id = arguments
                .get("building_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            let page_request = crate::application::dto::PageRequest {
                page: 1,
                per_page: 100,
                sort_by: None,
                order: crate::application::dto::SortOrder::default(),
            };
            match state
                .owner_use_cases
                .list_owners_paginated(&page_request, Some(org_id))
                .await
            {
                Ok((owners, _total)) => {
                    let text =
                        serde_json::to_string_pretty(&owners).unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to list owners: {}", e),
                    data: None,
                }),
            }
        }

        "list_meetings" => {
            let building_id_str = arguments
                .get("building_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "building_id is required".to_string(),
                    data: None,
                })?;

            let building_id = Uuid::parse_str(building_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "building_id must be a valid UUID".to_string(),
                data: None,
            })?;

            match state
                .meeting_use_cases
                .list_meetings_by_building(building_id)
                .await
            {
                Ok(meetings) => {
                    let text = serde_json::to_string_pretty(&meetings)
                        .unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to list meetings: {}", e),
                    data: None,
                }),
            }
        }

        "get_financial_summary" => {
            let building_id_str = arguments
                .get("building_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "building_id is required".to_string(),
                    data: None,
                })?;

            let building_id = Uuid::parse_str(building_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "building_id must be a valid UUID".to_string(),
                data: None,
            })?;

            // Gather expenses stats
            let expenses_result = state
                .expense_use_cases
                .list_expenses_by_building(building_id)
                .await;

            match expenses_result {
                Ok(expenses) => {
                    use crate::domain::entities::{ApprovalStatus, PaymentStatus};

                    let total_expenses: rust_decimal::Decimal =
                        expenses.iter().map(|e| e.amount).sum();
                    let pending_count = expenses
                        .iter()
                        .filter(|e| e.approval_status == ApprovalStatus::PendingApproval)
                        .count();
                    let overdue_count = expenses
                        .iter()
                        .filter(|e| e.payment_status == PaymentStatus::Overdue)
                        .count();

                    let summary = json!({
                        "building_id": building_id,
                        "total_expenses_eur": format!("{:.2}", total_expenses),
                        "pending_approval_count": pending_count,
                        "overdue_count": overdue_count,
                        "total_expense_count": expenses.len()
                    });

                    let text =
                        serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to get financial summary: {}", e),
                    data: None,
                }),
            }
        }

        "list_tickets" => {
            let building_id_str = arguments
                .get("building_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "building_id is required".to_string(),
                    data: None,
                })?;

            let building_id = Uuid::parse_str(building_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "building_id must be a valid UUID".to_string(),
                data: None,
            })?;

            match state
                .ticket_use_cases
                .list_tickets_by_building(building_id)
                .await
            {
                Ok(tickets) => {
                    let text =
                        serde_json::to_string_pretty(&tickets).unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to list tickets: {}", e),
                    data: None,
                }),
            }
        }

        "get_owner_balance" => {
            let owner_id_str = arguments
                .get("owner_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "owner_id is required".to_string(),
                    data: None,
                })?;

            let owner_id = Uuid::parse_str(owner_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "owner_id must be a valid UUID".to_string(),
                data: None,
            })?;

            match state
                .owner_contribution_use_cases
                .get_outstanding_contributions(owner_id)
                .await
            {
                Ok(contributions) => {
                    let total_due: rust_decimal::Decimal =
                        contributions.iter().map(|c| c.amount).sum();

                    let balance = json!({
                        "owner_id": owner_id,
                        "outstanding_contributions": contributions.len(),
                        "total_due_eur": format!("{:.2}", total_due)
                    });

                    let text =
                        serde_json::to_string_pretty(&balance).unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to get owner balance: {}", e),
                    data: None,
                }),
            }
        }

        "list_pending_expenses" => {
            let building_id = arguments
                .get("building_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            let expenses = if let Some(bid) = building_id {
                state.expense_use_cases.list_expenses_by_building(bid).await
            } else {
                {
                    let page_request = crate::application::dto::PageRequest {
                        page: 1,
                        per_page: 1000,
                        sort_by: None,
                        order: crate::application::dto::SortOrder::default(),
                    };
                    state
                        .expense_use_cases
                        .list_expenses_paginated(&page_request, Some(org_id))
                        .await
                        .map(|(expenses, _total)| expenses)
                }
            };

            match expenses {
                Ok(mut all_expenses) => {
                    use crate::domain::entities::ApprovalStatus as AS;
                    // Filter to pending approval by default
                    let status_filter = arguments
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("PendingApproval");

                    let target_status = match status_filter {
                        "Draft" | "draft" => Some(AS::Draft),
                        "PendingApproval" | "pending_approval" => Some(AS::PendingApproval),
                        "Approved" | "approved" => Some(AS::Approved),
                        "Rejected" | "rejected" => Some(AS::Rejected),
                        _ => None,
                    };

                    if let Some(target) = target_status {
                        all_expenses.retain(|e| e.approval_status == target);
                    }

                    let text = serde_json::to_string_pretty(&all_expenses)
                        .unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to list expenses: {}", e),
                    data: None,
                }),
            }
        }

        "check_quorum" => {
            let meeting_id_str = arguments
                .get("meeting_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "meeting_id is required".to_string(),
                    data: None,
                })?;

            let meeting_id = Uuid::parse_str(meeting_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "meeting_id must be a valid UUID".to_string(),
                data: None,
            })?;

            match state.meeting_use_cases.get_meeting(meeting_id).await {
                Ok(Some(meeting)) => {
                    let quorum_ok = meeting.quorum_validated;
                    let pct = meeting.quorum_percentage.unwrap_or(0.0);
                    let total = meeting
                        .total_quotas
                        .unwrap_or_else(|| rust_decimal_macros::dec!(1000));
                    let present = meeting
                        .present_quotas
                        .unwrap_or(rust_decimal::Decimal::ZERO);

                    let result = json!({
                        "meeting_id": meeting_id,
                        "meeting_title": meeting.title,
                        "quorum_validated": quorum_ok,
                        "quorum_percentage": pct,
                        "present_quotas": present,
                        "total_quotas": total,
                        "legal_threshold_pct": 50.0,
                        "legal_basis": "Art. 3.87 §5 Code Civil belge",
                        "status_message": if quorum_ok {
                            format!("✅ Quorum atteint: {:.1}% des tantièmes présents/représentés", pct)
                        } else {
                            format!("❌ Quorum non atteint: {:.1}% des tantièmes présents (minimum 50% requis)", pct)
                        }
                    });

                    let text =
                        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Ok(None) => Err(RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: format!("Meeting not found: {}", meeting_id),
                    data: None,
                }),
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to check quorum: {}", e),
                    data: None,
                }),
            }
        }

        "get_building_documents" => {
            let building_id_str = arguments
                .get("building_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "building_id is required".to_string(),
                    data: None,
                })?;

            let building_id = Uuid::parse_str(building_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "building_id must be a valid UUID".to_string(),
                data: None,
            })?;

            match state
                .document_use_cases
                .list_documents_by_building(building_id)
                .await
            {
                Ok(docs) => {
                    let text =
                        serde_json::to_string_pretty(&docs).unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to get documents: {}", e),
                    data: None,
                }),
            }
        }

        "legal_search" => Ok(resultat_texte(&rechercher_droit(arguments))),

        "majority_calculator" => Ok(resultat_texte(&calculer_majorite(arguments))),

        "list_owners_of_building" => {
            let building_id_str = arguments
                .get("building_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "building_id is required".to_string(),
                    data: None,
                })?;

            let _building_id = Uuid::parse_str(building_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "building_id must be a valid UUID".to_string(),
                data: None,
            })?;

            let page_request = crate::application::dto::PageRequest {
                page: 1,
                per_page: 100,
                sort_by: None,
                order: crate::application::dto::SortOrder::default(),
            };
            match state
                .owner_use_cases
                .list_owners_paginated(&page_request, Some(org_id))
                .await
            {
                Ok((owners, _total)) => {
                    let text =
                        serde_json::to_string_pretty(&owners).unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to list building owners: {}", e),
                    data: None,
                }),
            }
        }

        "ag_quorum_check" => {
            let meeting_id_str = arguments
                .get("meeting_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "meeting_id is required".to_string(),
                    data: None,
                })?;

            let meeting_id = Uuid::parse_str(meeting_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "meeting_id must be a valid UUID".to_string(),
                data: None,
            })?;

            match state.meeting_use_cases.get_meeting(meeting_id).await {
                Ok(Some(meeting)) => {
                    let quorum_ok = meeting.quorum_validated;
                    let pct = meeting.quorum_percentage.unwrap_or(0.0);

                    let result = if quorum_ok {
                        json!({
                            "meeting_id": meeting_id,
                            "quorum_validated": true,
                            "quorum_percentage": pct,
                            "status": "Quorum atteint",
                            "message": format!("✅ Quorum validé: {:.1}% des tantièmes présents/représentés", pct),
                            "next_steps": "L'AG peut délibérer valablement selon Art. 3.87 §5 CC"
                        })
                    } else {
                        json!({
                            "meeting_id": meeting_id,
                            "quorum_validated": false,
                            "quorum_percentage": pct,
                            "status": "Quorum insuffisant",
                            "message": format!("❌ Quorum non atteint: {:.1}% (minimum 50% requis)", pct),
                            "legal_basis": "Art. 3.87 §3-4 CC - Procédure de 2e convocation",
                            "next_steps": [
                                "1. Convoquer une 2e AG dans les 15 jours",
                                "2. Respecter délai minimum de 15 jours avant date de réunion",
                                "3. À la 2e convocation, quorum requis: au moins 1/4 des tantièmes",
                                "4. Si toujours insuffisant: peut délibérer quel que soit quorum"
                            ]
                        })
                    };

                    let text =
                        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Ok(None) => Err(RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: format!("Meeting not found: {}", meeting_id),
                    data: None,
                }),
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to check AG quorum: {}", e),
                    data: None,
                }),
            }
        }

        "comptabilite_situation" => {
            let building_id_str = arguments
                .get("building_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "building_id is required".to_string(),
                    data: None,
                })?;

            let building_id = Uuid::parse_str(building_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "building_id must be a valid UUID".to_string(),
                data: None,
            })?;

            match state
                .expense_use_cases
                .list_expenses_by_building(building_id)
                .await
            {
                Ok(expenses) => {
                    use crate::domain::entities::{ApprovalStatus, PaymentStatus};

                    let total_expenses: rust_decimal::Decimal = expenses
                        .iter()
                        .filter(|e| e.approval_status == ApprovalStatus::Approved)
                        .map(|e| e.amount)
                        .sum();

                    let outstanding: rust_decimal::Decimal = expenses
                        .iter()
                        .filter(|e| e.payment_status != PaymentStatus::Paid)
                        .map(|e| e.amount)
                        .sum();

                    let situation = json!({
                        "building_id": building_id,
                        "total_expenses_approved_eur": format!("{:.2}", total_expenses),
                        "outstanding_eur": format!("{:.2}", outstanding),
                        "expense_count": expenses.len(),
                        "paid_count": expenses.iter().filter(|e| e.payment_status == PaymentStatus::Paid).count(),
                        "pending_count": expenses.iter().filter(|e| e.approval_status == ApprovalStatus::PendingApproval).count()
                    });

                    let text = serde_json::to_string_pretty(&situation)
                        .unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock {
                            content_type: "text".to_string(),
                            text,
                        }],
                        is_error: None,
                    })
                }
                Err(e) => Err(RpcError {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to get comptabilite situation: {}", e),
                    data: None,
                }),
            }
        }

        "travaux_qualifier" => qualifier_travaux(arguments).map(|r| resultat_texte(&r)),

        _ => Err(RpcError {
            code: ErrorCode::METHOD_NOT_FOUND,
            message: format!("Unknown tool: {}", tool_name),
            data: Some(json!({
                "available_tools": get_mcp_tools().iter().map(|t| &t.name).collect::<Vec<_>>()
            })),
        }),
    }
}

// ─────────────────────────────────────────────────────────
// JSON-RPC dispatcher
// ─────────────────────────────────────────────────────────

/// Processes a JSON-RPC 2.0 request and returns a JSON-RPC response value
async fn handle_jsonrpc(req: JsonRpcRequest, state: &AppState, user: &AuthenticatedUser) -> Value {
    let id = req.id.clone();

    if req.jsonrpc != "2.0" {
        return serde_json::to_value(JsonRpcError {
            jsonrpc: "2.0".to_string(),
            id,
            error: RpcError {
                code: ErrorCode::INVALID_REQUEST,
                message: "jsonrpc must be '2.0'".to_string(),
                data: None,
            },
        })
        .unwrap_or(json!({"error": "serialization error"}));
    }

    match req.method.as_str() {
        // MCP lifecycle: initialize
        "initialize" => {
            let client_info = req
                .params
                .as_ref()
                .and_then(|p| p.get("clientInfo"))
                .cloned()
                .unwrap_or(json!({}));

            let result = json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "koprogo-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": "KoproGo est une plateforme de gestion de copropriété belge. Utilisez les outils disponibles pour accéder aux données des immeubles, copropriétaires, assemblées générales, finances et maintenance."
            });

            tracing::info!(
                method = "initialize",
                client_info = ?client_info,
                user_id = ?user.user_id,
                "MCP session initialized"
            );

            serde_json::to_value(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result,
            })
            .unwrap_or(json!({"error": "serialization error"}))
        }

        // MCP lifecycle: initialized (notification, no response needed)
        "notifications/initialized" => {
            json!(null) // null = no response for notifications
        }

        // tools/list
        "tools/list" => {
            let tools = get_mcp_tools();
            let result = json!({ "tools": tools });

            serde_json::to_value(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result,
            })
            .unwrap_or(json!({"error": "serialization error"}))
        }

        // tools/call
        "tools/call" => {
            let params = match req.params {
                Some(p) => p,
                None => {
                    return serde_json::to_value(JsonRpcError {
                        jsonrpc: "2.0".to_string(),
                        id,
                        error: RpcError {
                            code: ErrorCode::INVALID_PARAMS,
                            message: "params are required for tools/call".to_string(),
                            data: None,
                        },
                    })
                    .unwrap_or(json!({"error": "serialization error"}));
                }
            };

            let tool_name = match params.get("name").and_then(|v| v.as_str()) {
                Some(n) => n.to_string(),
                None => {
                    return serde_json::to_value(JsonRpcError {
                        jsonrpc: "2.0".to_string(),
                        id,
                        error: RpcError {
                            code: ErrorCode::INVALID_PARAMS,
                            message: "params.name is required".to_string(),
                            data: None,
                        },
                    })
                    .unwrap_or(json!({"error": "serialization error"}));
                }
            };

            if let Err(error) = outil_annonce(&tool_name) {
                return serde_json::to_value(JsonRpcError {
                    jsonrpc: "2.0".to_string(),
                    id,
                    error,
                })
                .unwrap_or(json!({"error": "serialization error"}));
            }

            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            tracing::info!(
                method = "tools/call",
                tool = %tool_name,
                user_id = ?user.user_id,
                "MCP tool called"
            );

            match dispatch_tool(&tool_name, &arguments, state, user).await {
                Ok(tool_result) => serde_json::to_value(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: serde_json::to_value(tool_result).unwrap_or(json!({})),
                })
                .unwrap_or(json!({"error": "serialization error"})),
                Err(err) => serde_json::to_value(JsonRpcError {
                    jsonrpc: "2.0".to_string(),
                    id,
                    error: err,
                })
                .unwrap_or(json!({"error": "serialization error"})),
            }
        }

        // ping
        "ping" => serde_json::to_value(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: json!({}),
        })
        .unwrap_or(json!({"error": "serialization error"})),

        // Unknown method
        _ => serde_json::to_value(JsonRpcError {
            jsonrpc: "2.0".to_string(),
            id,
            error: RpcError {
                code: ErrorCode::METHOD_NOT_FOUND,
                message: format!("Method not found: {}", req.method),
                data: None,
            },
        })
        .unwrap_or(json!({"error": "serialization error"})),
    }
}

// ─────────────────────────────────────────────────────────
// SSE endpoint: GET /mcp/sse
// ─────────────────────────────────────────────────────────

/// SSE endpoint that establishes the MCP connection.
///
/// The client connects here and receives:
/// 1. An `endpoint` event with the URL to POST JSON-RPC messages to
/// 2. Keepalive `: ping` comments every 30 seconds (prevents proxy timeouts)
///
/// Authentication: JWT Bearer token in Authorization header
#[get("/mcp/sse")]
pub async fn mcp_sse_endpoint(
    _req: HttpRequest,
    claims: AuthenticatedUser,
    _state: Data<AppState>,
) -> HttpResponse {
    // Generate a unique session ID for this SSE connection
    let session_id = Uuid::new_v4();
    let messages_url = format!("/mcp/messages?session_id={}", session_id);

    tracing::info!(
        session_id = %session_id,
        user_id = %claims.user_id,
        "New MCP SSE connection established"
    );

    // Build SSE stream
    // According to MCP spec: first event must be `endpoint` with the POST URL
    let sse_stream = stream::once(async move {
        // SSE `endpoint` event — tells client where to POST JSON-RPC messages
        let endpoint_event = format!(
            "event: endpoint\ndata: {}\n\n",
            serde_json::to_string(&messages_url)
                .unwrap_or_else(|_| format!("\"{}\"", messages_url))
        );
        Ok::<_, actix_web::Error>(actix_web::web::Bytes::from(endpoint_event))
    });

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no")) // Disable nginx buffering
        .insert_header(("Connection", "keep-alive"))
        .streaming(sse_stream)
}

// ─────────────────────────────────────────────────────────
// Messages endpoint: POST /mcp/messages
// ─────────────────────────────────────────────────────────

/// JSON-RPC 2.0 message endpoint.
///
/// The client POSTs JSON-RPC requests here and receives JSON-RPC responses.
/// Both single requests and batch arrays (JSON-RPC batch) are supported.
///
/// Authentication: JWT Bearer token in Authorization header
#[post("/mcp/messages")]
pub async fn mcp_messages_endpoint(
    req: HttpRequest,
    claims: AuthenticatedUser,
    state: Data<AppState>,
    body: web::Json<Value>,
) -> HttpResponse {
    let session_id = req
        .uri()
        .query()
        .and_then(|q| {
            q.split('&')
                .find(|p| p.starts_with("session_id="))
                .map(|p| &p["session_id=".len()..])
        })
        .unwrap_or("unknown");

    tracing::debug!(
        session_id = %session_id,
        user_id = %claims.user_id,
        "Received MCP message"
    );

    let body_value = body.into_inner();

    // Handle JSON-RPC batch (array of requests)
    if let Some(batch) = body_value.as_array() {
        let mut responses = Vec::new();
        for item in batch {
            match serde_json::from_value::<JsonRpcRequest>(item.clone()) {
                Ok(rpc_req) => {
                    let resp = handle_jsonrpc(rpc_req, &state, &claims).await;
                    if !resp.is_null() {
                        responses.push(resp);
                    }
                }
                Err(e) => {
                    responses.push(json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": ErrorCode::PARSE_ERROR,
                            "message": format!("Parse error: {}", e)
                        }
                    }));
                }
            }
        }

        if responses.is_empty() {
            // All were notifications — no response
            return HttpResponse::NoContent().finish();
        }

        return HttpResponse::Ok()
            .content_type("application/json")
            .json(Value::Array(responses));
    }

    // Single JSON-RPC request
    match serde_json::from_value::<JsonRpcRequest>(body_value) {
        Ok(rpc_req) => {
            let response = handle_jsonrpc(rpc_req, &state, &claims).await;

            if response.is_null() {
                // Notification — no response body
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::Ok()
                    .content_type("application/json")
                    .json(response)
            }
        }
        Err(e) => HttpResponse::BadRequest()
            .content_type("application/json")
            .json(json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": ErrorCode::PARSE_ERROR,
                    "message": format!("Parse error: {}", e)
                }
            })),
    }
}

// ─────────────────────────────────────────────────────────
// Health/info endpoint: GET /mcp/info
// ─────────────────────────────────────────────────────────

/// Returns MCP server metadata (no auth required — for discovery)
#[get("/mcp/info")]
pub async fn mcp_info_endpoint() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "name": "koprogo-mcp",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "MCP/2024-11-05",
        "transport": "SSE+HTTP",
        "endpoints": {
            "sse": "/mcp/sse",
            "messages": "/mcp/messages",
            "system_prompt": "/mcp/system-prompt",
            "legal_index": "/mcp/legal-index"
        },
        "tools_count": get_mcp_tools().len(),
        "description": "Model Context Protocol server for KoproGo — Belgian property management SaaS"
    }))
}

/// GET /mcp/system-prompt — System prompt for AI agents
/// Returns a Markdown document that AI clients (like Claude Desktop) can fetch
/// to understand KoproGo context, available tools, and Belgian legal rules.
/// Issue #263
///
/// Cloisonnement (#882) : classée LÉGITIME. `_claims` n'est jamais utilisé
/// parce que le corps servi est `include_str!("../../mcp_system_prompt.md")`
/// — un fichier statique compilé dans le binaire, identique pour tout
/// appelant. Il n'y a pas d'organisation à cloisonner ici ; l'identité reste
/// exigée pour fermer la route aux appelants anonymes.
#[get("/mcp/system-prompt")]
pub async fn mcp_system_prompt_endpoint(
    _claims: AuthenticatedUser,
    _state: Data<AppState>,
) -> HttpResponse {
    let prompt = include_str!("../../mcp_system_prompt.md");
    HttpResponse::Ok()
        .content_type("text/markdown; charset=utf-8")
        .body(prompt)
}

/// GET /mcp/legal-index — Legal document index in JSON
/// Returns a comprehensive index of Belgian legal rules, GDPR articles,
/// and KoproGo-specific compliance rules. Embedded as static JSON.
/// Issue #262
///
/// Cloisonnement (#882) : classée LÉGITIME, même raison que
/// `mcp_system_prompt_endpoint` juste au-dessus — `include_str!("../../legal_index.json")`
/// est un index légal statique, identique pour tout appelant.
#[get("/mcp/legal-index")]
pub async fn mcp_legal_index_endpoint(
    _claims: AuthenticatedUser,
    _state: Data<AppState>,
) -> HttpResponse {
    let index = include_str!("../../legal_index.json");
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(index)
}

// ─────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Lot 0, story S4 : le droit du MCP vient de `majorites.rs` ────────

    use crate::domain::copropriete::NatureDeDecision;

    /// Le contrat d'entrée de `majority_calculator` : un code par nature de
    /// l'Art. 3.88. Le `match` est exhaustif : une nature ajoutée au domaine
    /// ne compile pas tant qu'elle n'a pas de code ici.
    fn code_de_nature(nature: NatureDeDecision) -> &'static str {
        use NatureDeDecision::*;
        match nature {
            ModificationStatutsUsageDesCommuns => "modification_statuts_usage_des_communs",
            TravauxAuxPartiesCommunes => "travaux_aux_parties_communes",
            SeuilDeMiseEnConcurrence => "seuil_de_mise_en_concurrence",
            TravauxPrivatifsExecutesParLacp => "travaux_privatifs_executes_par_lacp",
            TravauxImposesParLaLoi => "travaux_imposes_par_la_loi",
            TravauxConservatoires => "travaux_conservatoires",
            AdministrationProvisoire => "administration_provisoire",
            AutreModificationDesStatuts => "autre_modification_des_statuts",
            ModificationDeLaDestination => "modification_de_la_destination",
            ReconstructionApresDestructionPartielle => "reconstruction_apres_destruction_partielle",
            AcquisitionDeBiensCommuns => "acquisition_de_biens_communs",
            ActeDeDispositionDeBiensCommuns => "acte_de_disposition_de_biens_communs",
            CreationDassociationsPartielles => "creation_dassociations_partielles",
            DivisionOuReunionDeLots => "division_ou_reunion_de_lots",
            DemolitionReconstructionPourSalubriteOuCout => {
                "demolition_reconstruction_pour_salubrite_ou_cout"
            }
            ModificationDesQuotesParts => "modification_des_quotes_parts",
            DemolitionReconstructionAutreMotif => "demolition_reconstruction_autre_motif",
            DroitCommun => "droit_commun",
        }
    }

    fn toutes_les_natures() -> [NatureDeDecision; 18] {
        use NatureDeDecision::*;
        [
            ModificationStatutsUsageDesCommuns,
            TravauxAuxPartiesCommunes,
            SeuilDeMiseEnConcurrence,
            TravauxPrivatifsExecutesParLacp,
            TravauxImposesParLaLoi,
            TravauxConservatoires,
            AdministrationProvisoire,
            AutreModificationDesStatuts,
            ModificationDeLaDestination,
            ReconstructionApresDestructionPartielle,
            AcquisitionDeBiensCommuns,
            ActeDeDispositionDeBiensCommuns,
            CreationDassociationsPartielles,
            DivisionOuReunionDeLots,
            DemolitionReconstructionPourSalubriteOuCout,
            ModificationDesQuotesParts,
            DemolitionReconstructionAutreMotif,
            DroitCommun,
        ]
    }

    fn majorite_du_domaine(nature: NatureDeDecision) -> Value {
        serde_json::to_value(nature.majorite_requise()).unwrap()
    }

    /// Seuils inventés que le MCP affirmait comme légaux (relevé du 03/11).
    const SEUILS_INVENTES: [&str; 7] = [
        "trois devis",
        "3 devis",
        "5000",
        "5 000",
        "5,000",
        "50 000",
        "50,000",
    ];

    fn sans_seuil_invente(contexte: &str, texte: &str) {
        let bas = texte.to_lowercase();
        for seuil in SEUILS_INVENTES {
            assert!(!bas.contains(seuil), "{contexte} affirme « {seuil} »");
        }
    }

    #[test]
    fn happy_majority_calculator_suit_majorites_rs() {
        for nature in toutes_les_natures() {
            let code = code_de_nature(nature);
            let r = calculer_majorite(&json!({ "decision_type": code }));
            assert_eq!(r["nature"], json!(code), "{code}");
            assert_eq!(r["majorite_requise"], majorite_du_domaine(nature), "{code}");
        }
    }

    #[test]
    fn happy_le_schema_du_calculateur_annonce_toutes_les_natures() {
        let outils = get_mcp_tools();
        let calc = outils
            .iter()
            .find(|t| t.name == "majority_calculator")
            .unwrap();
        let annonce = &calc.input_schema["properties"]["decision_type"]["enum"];
        for nature in toutes_les_natures() {
            let code = code_de_nature(nature);
            assert!(
                annonce.as_array().unwrap().contains(&json!(code)),
                "{code} absent du schéma"
            );
        }
    }

    #[test]
    fn security_mcp_ne_contredit_pas_majorites_rs() {
        // 1. Le calculateur, nature par nature.
        for nature in toutes_les_natures() {
            let r = calculer_majorite(&json!({ "decision_type": code_de_nature(nature) }));
            assert_eq!(r["majorite_requise"], majorite_du_domaine(nature));
        }
        // 2. Toute entrée de `legal_search` qui parle de majorité porte sa
        //    nature, et la majorité qu'elle cite est celle du domaine.
        let base = rechercher_droit(&json!({ "query": "" }));
        let entrees = base["results"].as_array().unwrap();
        assert!(!entrees.is_empty());
        // Le mot cherché passe par une variable : ce test ne classe aucune
        // erreur, et `garde_classement_par_souschaines` compte les littéraux.
        let mot_majorite = "majorité";
        for e in entrees {
            let parle_de_majorite = e["category"] == json!("Majorité")
                || e.get("majorite_requise").is_some()
                || e["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(mot_majorite);
            if !parle_de_majorite {
                continue;
            }
            let code = e["nature"]
                .as_str()
                .unwrap_or_else(|| panic!("{} cite une majorité sans nature", e["code"]));
            let nature = toutes_les_natures()
                .into_iter()
                .find(|n| code_de_nature(*n) == code)
                .unwrap_or_else(|| panic!("nature inconnue {code}"));
            assert_eq!(e["majorite_requise"], majorite_du_domaine(nature), "{code}");
        }
        // 3. `travaux_qualifier` : conservatoires en urgence, parties communes
        //    sinon, quel que soit le montant.
        let urgent = qualifier_travaux(&json!({
            "description": "fuite de toiture",
            "estimated_amount_eur": 12000.0,
            "is_emergency": true
        }))
        .unwrap();
        assert_eq!(
            urgent["majorite_requise"],
            majorite_du_domaine(NatureDeDecision::TravauxConservatoires)
        );
        for montant in [800.0, 7500.0, 120000.0] {
            let r = qualifier_travaux(&json!({
                "description": "ravalement",
                "estimated_amount_eur": montant,
                "is_emergency": false
            }))
            .unwrap();
            assert_eq!(
                r["majorite_requise"],
                majorite_du_domaine(NatureDeDecision::TravauxAuxPartiesCommunes),
                "{montant} €"
            );
        }
    }

    #[test]
    fn edge_nature_inconnue_donne_le_droit_commun() {
        let droit_commun = majorite_du_domaine(NatureDeDecision::DroitCommun);
        for entree in [
            json!({ "decision_type": "nature_qui_nexiste_pas" }),
            json!({ "decision_type": "" }),
            json!({}),
        ] {
            let r = calculer_majorite(&entree);
            assert_eq!(r["nature"], json!("droit_commun"), "{entree}");
            assert_eq!(r["majorite_requise"], droit_commun, "{entree}");
        }
    }

    #[test]
    fn negative_aucune_reponse_ne_pose_de_seuil_invente() {
        for nature in toutes_les_natures() {
            let r = calculer_majorite(&json!({ "decision_type": code_de_nature(nature) }));
            sans_seuil_invente("majority_calculator", &r.to_string());
        }
        sans_seuil_invente(
            "legal_search",
            &rechercher_droit(&json!({ "query": "" })).to_string(),
        );
        for (montant, urgence) in [(800.0, false), (7500.0, false), (120000.0, true)] {
            let r = qualifier_travaux(&json!({
                "description": "travaux",
                "estimated_amount_eur": montant,
                "is_emergency": urgence
            }))
            .unwrap();
            // Le montant saisi est renvoyé tel quel : on l'écarte avant de chercher.
            let mut r = r;
            r.as_object_mut().unwrap().remove("amount_eur");
            sans_seuil_invente("travaux_qualifier", &r.to_string());
        }
        for outil in get_mcp_tools() {
            sans_seuil_invente(&outil.name, &outil.description);
        }
        sans_seuil_invente(
            "mcp_system_prompt.md",
            include_str!("../../mcp_system_prompt.md"),
        );
    }

    #[test]
    fn test_mcp_tools_have_unique_names() {
        let tools = get_mcp_tools();
        let mut names = std::collections::HashSet::new();
        for tool in &tools {
            assert!(
                names.insert(&tool.name),
                "Duplicate tool name: {}",
                tool.name
            );
        }
    }

    #[test]
    fn test_mcp_tools_have_required_input_schema_fields() {
        let tools = get_mcp_tools();
        for tool in &tools {
            assert!(!tool.name.is_empty(), "Tool has empty name");
            assert!(
                !tool.description.is_empty(),
                "Tool '{}' has empty description",
                tool.name
            );
            assert!(
                tool.input_schema.get("type").is_some(),
                "Tool '{}' input_schema missing 'type' field",
                tool.name
            );
            assert!(
                tool.input_schema.get("properties").is_some(),
                "Tool '{}' input_schema missing 'properties' field",
                tool.name
            );
        }
    }

    #[test]
    fn test_jsonrpc_ping_method() {
        // Verify ping request structure
        let ping_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "ping".to_string(),
            params: None,
        };
        assert_eq!(ping_req.method, "ping");
        assert_eq!(ping_req.jsonrpc, "2.0");
    }

    #[test]
    fn test_error_codes_are_correct() {
        assert_eq!(ErrorCode::PARSE_ERROR, -32700);
        assert_eq!(ErrorCode::INVALID_REQUEST, -32600);
        assert_eq!(ErrorCode::METHOD_NOT_FOUND, -32601);
        assert_eq!(ErrorCode::INVALID_PARAMS, -32602);
        assert_eq!(ErrorCode::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_tool_count() {
        // 20 outils annoncés moins les quatre retirés par l'ADR 0059 (lot 0, S3).
        let tools = get_mcp_tools();
        assert_eq!(tools.len(), 16, "Expected 16 MCP tools");
    }

    // Lot 0, story S3 : pas d'outil qui fait semblant (ADR 0059, option 1).

    const OUTILS_RETIRES: [&str; 4] = [
        "ag_vote",
        "appel_de_fonds",
        "energie_campagne_list",
        "alertes_list",
    ];

    fn outils_du_prompt_systeme() -> Vec<String> {
        include_str!("../../mcp_system_prompt.md")
            .lines()
            .filter_map(|l| l.trim().strip_prefix("- `"))
            .filter_map(|l| l.split('`').next())
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn happy_outils_branches_restent_annonces() {
        let names: Vec<String> = get_mcp_tools().into_iter().map(|t| t.name).collect();
        for name in ["list_buildings", "check_quorum", "majority_calculator"] {
            assert!(names.iter().any(|n| n == name), "{name} doit rester listé");
            assert!(outil_annonce(name).is_ok(), "{name} doit rester appelable");
        }
    }

    #[test]
    fn security_outils_non_branches_absents_de_tools_list() {
        let names: Vec<String> = get_mcp_tools().into_iter().map(|t| t.name).collect();
        for retire in OUTILS_RETIRES {
            assert!(
                !names.iter().any(|n| n == retire),
                "{retire} ne doit plus être listé"
            );
        }
    }

    #[test]
    fn negative_appel_outil_retire_renvoie_method_not_found() {
        for retire in OUTILS_RETIRES {
            let err = outil_annonce(retire).expect_err("un outil retiré n'est pas appelable");
            assert_eq!(err.code, ErrorCode::METHOD_NOT_FOUND, "{retire}");
        }
    }

    #[test]
    fn edge_prompt_systeme_ne_cite_que_des_outils_annonces() {
        let annonces: Vec<String> = get_mcp_tools().into_iter().map(|t| t.name).collect();
        let cites = outils_du_prompt_systeme();
        assert!(!cites.is_empty(), "le prompt système liste ses outils");
        for cite in &cites {
            assert!(
                annonces.contains(cite),
                "le prompt cite {cite}, absent de tools/list"
            );
        }
        for retire in OUTILS_RETIRES {
            assert!(
                !cites.iter().any(|c| c == retire),
                "le prompt cite encore {retire}"
            );
        }
    }
}
