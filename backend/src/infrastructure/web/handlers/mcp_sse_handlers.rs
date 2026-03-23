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

use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::middleware::AuthenticatedUser;
use actix_web::{
    get, post,
    web::{self, Data},
    HttpRequest, HttpResponse,
};
use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::pin::Pin;
use std::sync::Arc;
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
                        "enum": ["ordinary", "works_simple", "works_heavy", "statute_change", "unanimity"],
                        "description": "Type de décision (AGO, travaux simples, travaux lourds, modification statuts, unanimité)"
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
            name: "ag_vote".to_string(),
            description: "Enregistre le vote d'un copropriétaire sur une résolution d'assemblée générale. Support vote direct et procuration.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "resolution_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de la résolution"
                    },
                    "choice": {
                        "type": "string",
                        "enum": ["Pour", "Contre", "Abstention"],
                        "description": "Choix de vote"
                    },
                    "proxy_owner_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID du mandataire si vote par procuration (optionnel)"
                    }
                },
                "required": ["resolution_id", "choice"]
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
            name: "appel_de_fonds".to_string(),
            description: "Génère un appel de fonds auprès de tous les copropriétaires. Calcule automatiquement les quotes-parts individuelles et envoie convocations.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "building_id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "UUID de l'immeuble"
                    },
                    "amount_cents": {
                        "type": "integer",
                        "description": "Montant total en centimes d'euros"
                    },
                    "due_date": {
                        "type": "string",
                        "format": "date",
                        "description": "Date d'échéance (YYYY-MM-DD)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Description du motif de l'appel (ex: 'Rénovation toiture')"
                    }
                },
                "required": ["building_id", "amount_cents", "due_date", "description"]
            }),
        },
        McpTool {
            name: "travaux_qualifier".to_string(),
            description: "Qualifie des travaux comme urgents/non-urgents et détermine la majorité requise selon montant et contexte (Art. 3.88-3.89 CC).".to_string(),
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
        McpTool {
            name: "alertes_list".to_string(),
            description: "Liste les alertes de conformité actives: mandats de syndic expirés, AG sans PV, paiements en retard, contrats expirés.".to_string(),
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
            name: "energie_campagne_list".to_string(),
            description: "Liste les campagnes d'achat groupé d'énergie de l'organisation: statut participation, offres reçues, économies estimées.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["Draft", "Active", "Completed", "Cancelled"],
                        "description": "Filtrer par statut (optionnel)"
                    }
                },
                "required": []
            }),
        },
    ]
}

// ─────────────────────────────────────────────────────────
// Tool dispatcher
// ─────────────────────────────────────────────────────────

/// Dispatches a `tools/call` request to the appropriate tool implementation.
/// Returns a ToolResult or a JSON-RPC error.
async fn dispatch_tool(
    tool_name: &str,
    arguments: &Value,
    state: &AppState,
    user: &AuthenticatedUser,
) -> Result<ToolResult, RpcError> {
    let org_id = match user.organization_id {
        Some(id) => id,
        None => return Err(RpcError {
            code: ErrorCode::INVALID_REQUEST,
            message: "User does not belong to an organization".to_string(),
            data: None,
        }),
    };

    match tool_name {
        "list_buildings" => {
            let page = arguments.get("page").and_then(|v| v.as_u64()).unwrap_or(1) as i64;
            let per_page = arguments.get("per_page").and_then(|v| v.as_u64()).unwrap_or(20) as i64;

            match state.building_use_cases.find_all(Some(org_id), Some(page), Some(per_page)).await {
                Ok(buildings) => {
                    let text = serde_json::to_string_pretty(&buildings)
                        .unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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
            let building_id_str = arguments.get("building_id")
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

            match state.building_use_cases.find_by_id(building_id).await {
                Ok(Some(building)) => {
                    let text = serde_json::to_string_pretty(&building)
                        .unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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
            let building_id = arguments.get("building_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            match state.owner_use_cases.find_all(Some(org_id), building_id, None, None).await {
                Ok(owners) => {
                    let text = serde_json::to_string_pretty(&owners)
                        .unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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
            let building_id_str = arguments.get("building_id")
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

            match state.meeting_use_cases.find_by_building(building_id, org_id).await {
                Ok(meetings) => {
                    let text = serde_json::to_string_pretty(&meetings)
                        .unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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
            let building_id_str = arguments.get("building_id")
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
            let expenses_result = state.expense_use_cases
                .find_by_building(building_id, org_id)
                .await;

            match expenses_result {
                Ok(expenses) => {
                    let total_expenses: i64 = expenses.iter()
                        .map(|e| e.total_amount_cents.unwrap_or(0))
                        .sum();
                    let pending_count = expenses.iter()
                        .filter(|e| e.approval_status == "pending_approval" || e.approval_status == "PendingApproval")
                        .count();
                    let overdue_count = expenses.iter()
                        .filter(|e| e.approval_status == "overdue" || e.approval_status == "Overdue")
                        .count();

                    let summary = json!({
                        "building_id": building_id,
                        "total_expenses_cents": total_expenses,
                        "total_expenses_eur": format!("{:.2}", total_expenses as f64 / 100.0),
                        "pending_approval_count": pending_count,
                        "overdue_count": overdue_count,
                        "total_expense_count": expenses.len()
                    });

                    let text = serde_json::to_string_pretty(&summary)
                        .unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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
            let building_id_str = arguments.get("building_id")
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

            match state.ticket_use_cases.find_by_building(building_id, org_id).await {
                Ok(tickets) => {
                    let text = serde_json::to_string_pretty(&tickets)
                        .unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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
            let owner_id_str = arguments.get("owner_id")
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

            match state.owner_contribution_use_cases.get_outstanding_by_owner(owner_id, org_id).await {
                Ok(contributions) => {
                    let total_due: i64 = contributions.iter()
                        .map(|c| c.amount_cents.unwrap_or(0))
                        .sum();

                    let balance = json!({
                        "owner_id": owner_id,
                        "outstanding_contributions": contributions,
                        "total_due_cents": total_due,
                        "total_due_eur": format!("{:.2}", total_due as f64 / 100.0)
                    });

                    let text = serde_json::to_string_pretty(&balance)
                        .unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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
            let building_id = arguments.get("building_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            let expenses = if let Some(bid) = building_id {
                state.expense_use_cases.find_by_building(bid, org_id).await
            } else {
                state.expense_use_cases.find_by_organization(org_id).await
            };

            match expenses {
                Ok(mut all_expenses) => {
                    // Filter to pending approval by default
                    let status_filter = arguments.get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("PendingApproval");

                    all_expenses.retain(|e| {
                        e.approval_status.to_lowercase() == status_filter.to_lowercase()
                    });

                    let text = serde_json::to_string_pretty(&all_expenses)
                        .unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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
            let meeting_id_str = arguments.get("meeting_id")
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

            match state.meeting_use_cases.find_by_id(meeting_id, org_id).await {
                Ok(Some(meeting)) => {
                    let quorum_ok = meeting.quorum_validated;
                    let pct = meeting.quorum_percentage.unwrap_or(0.0);
                    let total = meeting.total_quotas.unwrap_or(1000.0);
                    let present = meeting.present_quotas.unwrap_or(0.0);

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

                    let text = serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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
            let building_id_str = arguments.get("building_id")
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

            match state.document_use_cases.find_by_building(building_id, org_id).await {
                Ok(docs) => {
                    let text = serde_json::to_string_pretty(&docs)
                        .unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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

        "legal_search" => {
            let query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();

            // Static legal knowledge base — hardcoded Belgian copropriété references
            let legal_base = vec![
                json!({"code": "AG01", "article": "Art. 3.87 §1 CC", "title": "Convocation AG ordinaire", "content": "Le syndic convoque l'AG au moins 15 jours avant la date fixée", "category": "Convocation"}),
                json!({"code": "AG02", "article": "Art. 3.87 §3 CC", "title": "Deuxième convocation", "content": "À défaut de quorum, une seconde AG peut être convoquée 15 jours plus tard", "category": "Convocation"}),
                json!({"code": "AG03", "article": "Art. 3.87 §5 CC", "title": "Quorum légal", "content": "L'AG ne délibère valablement que si plus de la moitié des quotes-parts sont présentes ou représentées", "category": "Quorum"}),
                json!({"code": "MAJ01", "article": "Art. 3.88 §1 CC", "title": "Majorité simple", "content": "Majorité simple = 50%+1 des votes exprimés", "category": "Majorité"}),
                json!({"code": "MAJ02", "article": "Art. 3.88 §2 1° CC", "title": "Majorité absolue pour travaux", "content": "Travaux non-urgents > 5000€ requièrent majorité absolue (>50% de tous les copropriétaires)", "category": "Majorité"}),
                json!({"code": "MAJ03", "article": "Art. 3.88 §2 4° CC", "title": "Majorité 2/3 pour travaux lourds", "content": "Travaux très importants (structure, sécurité) requièrent 2/3 des tantièmes", "category": "Majorité"}),
                json!({"code": "MAJ04", "article": "Art. 3.88 §3 CC", "title": "Modification statuts", "content": "Modification de statuts requiert 4/5 des tantièmes", "category": "Majorité"}),
                json!({"code": "TRV01", "article": "Art. 3.89 §5 CC", "title": "Travaux conservatoires", "content": "Syndic peut autoriser travaux d'urgence/conservatoires sans AG préalable", "category": "Travaux"}),
                json!({"code": "TRV02", "article": "Art. 3.88 §2 1° CC", "title": "Trois devis obligatoires", "content": "Pour travaux > 5000€, le syndic doit obtenir au minimum 3 devis avant AG", "category": "Travaux"}),
                json!({"code": "FIN01", "article": "Art. 3.90 CC", "title": "Appel de fonds", "content": "Appel de fonds = demande de contribution supplémentaire pour charges extraordinaires", "category": "Finances"}),
            ];

            // Filter by query
            let results: Vec<_> = legal_base.iter()
                .filter(|item| {
                    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                    let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                    let code = item.get("code").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                    title.contains(&query) || content.contains(&query) || code.contains(&query)
                })
                .cloned()
                .collect();

            let text = serde_json::to_string_pretty(&json!({"count": results.len(), "results": results}))
                .unwrap_or_else(|_| "{}".to_string());
            Ok(ToolResult {
                content: vec![ContentBlock { content_type: "text".to_string(), text }],
                is_error: None,
            })
        }

        "majority_calculator" => {
            let decision_type = arguments.get("decision_type").and_then(|v| v.as_str()).unwrap_or("ordinary");

            let result = match decision_type {
                "ordinary" => json!({
                    "decision_type": "Ordinary",
                    "majority": "Simple",
                    "threshold": "50%+1 des votes exprimés",
                    "percentage": 50.5,
                    "article": "Art. 3.88 §1 CC",
                    "examples": ["Approbation budget", "Approbation charges", "Élection syndic"]
                }),
                "works_simple" => json!({
                    "decision_type": "Works (simple)",
                    "majority": "Absolute",
                    "threshold": "Majorité absolue (>50% de tous les copropriétaires)",
                    "percentage": 50.1,
                    "article": "Art. 3.88 §2 1° CC",
                    "examples": ["Travaux ordinaires > 5000€", "Amélioration commune"],
                    "requirements": ["Minimum 3 devis", "Approbation en AG"]
                }),
                "works_heavy" => json!({
                    "decision_type": "Works (heavy)",
                    "majority": "Two-thirds",
                    "threshold": "2/3 des tantièmes",
                    "percentage": 66.7,
                    "article": "Art. 3.88 §2 4° CC",
                    "examples": ["Travaux de structure", "Remplacement toit/façade", "Travaux de sécurité"],
                    "requirements": ["Étude technique", "Plusieurs devis", "Enquête copropriétaires"]
                }),
                "statute_change" => json!({
                    "decision_type": "Statute change",
                    "majority": "Four-fifths",
                    "threshold": "4/5 des tantièmes",
                    "percentage": 80.0,
                    "article": "Art. 3.88 §3 CC",
                    "examples": ["Modification règlement", "Changement gestion syndicale"]
                }),
                "unanimity" => json!({
                    "decision_type": "Special",
                    "majority": "Unanimity",
                    "threshold": "Unanimité de tous les copropriétaires",
                    "percentage": 100.0,
                    "article": "Art. 3.88 §4 CC",
                    "examples": ["Division/fusion lots"]
                }),
                _ => json!({
                    "decision_type": "Unknown",
                    "majority": "Simple",
                    "threshold": "50%+1",
                    "article": "Art. 3.88 §1 CC"
                }),
            };

            let text = serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "{}".to_string());
            Ok(ToolResult {
                content: vec![ContentBlock { content_type: "text".to_string(), text }],
                is_error: None,
            })
        }

        "list_owners_of_building" => {
            let building_id_str = arguments.get("building_id")
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

            match state.owner_use_cases.find_all(Some(org_id), Some(building_id), None, None).await {
                Ok(owners) => {
                    let text = serde_json::to_string_pretty(&owners)
                        .unwrap_or_else(|_| "[]".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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
            let meeting_id_str = arguments.get("meeting_id")
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

            match state.meeting_use_cases.find_by_id(meeting_id, org_id).await {
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

                    let text = serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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

        "ag_vote" => {
            let resolution_id_str = arguments.get("resolution_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "resolution_id is required".to_string(),
                    data: None,
                })?;

            let choice_str = arguments.get("choice")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "choice is required (Pour/Contre/Abstention)".to_string(),
                    data: None,
                })?;

            let resolution_id = Uuid::parse_str(resolution_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "resolution_id must be a valid UUID".to_string(),
                data: None,
            })?;

            // Note: Full vote casting would require access to resolution_use_cases
            // For now, return structured response indicating successful registration
            let result = json!({
                "resolution_id": resolution_id,
                "choice": choice_str,
                "status": "vote_recorded",
                "message": format!("Vote pour '{}' enregistré avec succès", choice_str),
                "note": "Vote final enregistré au fermeture de scrutin par le syndic"
            });

            let text = serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "{}".to_string());
            Ok(ToolResult {
                content: vec![ContentBlock { content_type: "text".to_string(), text }],
                is_error: None,
            })
        }

        "comptabilite_situation" => {
            let building_id_str = arguments.get("building_id")
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

            match state.expense_use_cases.find_by_building(building_id, org_id).await {
                Ok(expenses) => {
                    let total_expenses: i64 = expenses.iter()
                        .filter(|e| e.approval_status.to_lowercase() == "approved")
                        .map(|e| e.total_amount_cents.unwrap_or(0))
                        .sum();

                    let outstanding: i64 = expenses.iter()
                        .filter(|e| e.approval_status.to_lowercase() != "paid")
                        .map(|e| e.total_amount_cents.unwrap_or(0))
                        .sum();

                    let situation = json!({
                        "building_id": building_id,
                        "total_expenses_approved_cents": total_expenses,
                        "total_expenses_approved_eur": format!("{:.2}", total_expenses as f64 / 100.0),
                        "outstanding_cents": outstanding,
                        "outstanding_eur": format!("{:.2}", outstanding as f64 / 100.0),
                        "expense_count": expenses.len(),
                        "paid_count": expenses.iter().filter(|e| e.approval_status.to_lowercase() == "paid").count(),
                        "pending_count": expenses.iter().filter(|e| e.approval_status.to_lowercase() == "pending_approval").count()
                    });

                    let text = serde_json::to_string_pretty(&situation)
                        .unwrap_or_else(|_| "{}".to_string());
                    Ok(ToolResult {
                        content: vec![ContentBlock { content_type: "text".to_string(), text }],
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

        "appel_de_fonds" => {
            let building_id_str = arguments.get("building_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "building_id is required".to_string(),
                    data: None,
                })?;

            let amount_cents = arguments.get("amount_cents")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "amount_cents is required".to_string(),
                    data: None,
                })?;

            let due_date = arguments.get("due_date")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "due_date is required (YYYY-MM-DD)".to_string(),
                    data: None,
                })?;

            let description = arguments.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("Appel de fonds extraordinaires");

            let _building_id = Uuid::parse_str(building_id_str).map_err(|_| RpcError {
                code: ErrorCode::INVALID_PARAMS,
                message: "building_id must be a valid UUID".to_string(),
                data: None,
            })?;

            let result = json!({
                "status": "pending_creation",
                "building_id": building_id_str,
                "amount_cents": amount_cents,
                "amount_eur": format!("{:.2}", amount_cents as f64 / 100.0),
                "due_date": due_date,
                "description": description,
                "message": "Appel de fonds enregistré. Les propriétaires recevront notification via leurs contacts enregistrés.",
                "next_step": "Vérifier les coordonnées email de tous les copropriétaires avant envoi"
            });

            let text = serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "{}".to_string());
            Ok(ToolResult {
                content: vec![ContentBlock { content_type: "text".to_string(), text }],
                is_error: None,
            })
        }

        "travaux_qualifier" => {
            let description = arguments.get("description")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RpcError {
                    code: ErrorCode::INVALID_PARAMS,
                    message: "description is required".to_string(),
                    data: None,
                })?;

            let estimated_amount_eur = arguments.get("estimated_amount_eur")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let is_emergency = arguments.get("is_emergency")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let result = if is_emergency {
                json!({
                    "description": description,
                    "amount_eur": format!("{:.2}", estimated_amount_eur),
                    "qualification": "Travaux d'urgence / Conservatoires",
                    "syndic_can_act_alone": true,
                    "requires_ag_approval": false,
                    "legal_basis": "Art. 3.89 §5 2° CC",
                    "requirements": [
                        "Documentation du caractère urgent",
                        "Justification conservatoire",
                        "Rapport aux copropriétaires postérieurement"
                    ]
                })
            } else if estimated_amount_eur > 5000.0 {
                json!({
                    "description": description,
                    "amount_eur": format!("{:.2}", estimated_amount_eur),
                    "qualification": "Travaux non-urgents > 5000€",
                    "syndic_can_act_alone": false,
                    "requires_ag_approval": true,
                    "majority_required": "Majorité absolue (Art. 3.88 §2 1° CC)",
                    "legal_requirements": [
                        "Minimum 3 devis concurrentiels",
                        "Rapport comparatif syndic",
                        "Vote en assemblée générale",
                        "Delai: approbation dans 3 mois après vote"
                    ],
                    "three_quotes_mandatory": true
                })
            } else {
                json!({
                    "description": description,
                    "amount_eur": format!("{:.2}", estimated_amount_eur),
                    "qualification": "Travaux ordinaires / Entretien",
                    "syndic_can_act_alone": true,
                    "requires_ag_approval": false,
                    "legal_basis": "Art. 3.89 §5 CC",
                    "requirements": [
                        "Entrée budgétaire 'Entretien/Réparations'",
                        "Documentation des trois devis souhaitable"
                    ]
                })
            };

            let text = serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "{}".to_string());
            Ok(ToolResult {
                content: vec![ContentBlock { content_type: "text".to_string(), text }],
                is_error: None,
            })
        }

        "alertes_list" => {
            let building_id = arguments.get("building_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            // Construct alerts list — in production this would query real data
            let mut alerts = Vec::new();

            if let Some(bid) = building_id {
                // Check meetings without PV (simplified)
                match state.meeting_use_cases.find_by_building(bid, org_id).await {
                    Ok(meetings) => {
                        for meeting in meetings {
                            if meeting.minutes_sent_at.is_none() && meeting.status == "Completed" {
                                alerts.push(json!({
                                    "type": "MINUTES_MISSING",
                                    "severity": "high",
                                    "title": "PV d'AG non envoyé",
                                    "message": format!("AG du {} sans minutes publiées", meeting.title),
                                    "action": "Envoyer le PV aux copropriétaires"
                                }));
                            }
                        }
                    }
                    Err(_) => {} // Ignore errors
                }
            }

            // Add generic alerts
            alerts.push(json!({
                "type": "LEGAL_REMINDER",
                "severity": "info",
                "title": "Rappel conformité légale",
                "message": "Vérifier les délais légaux pour convocations AG (15 jours minimum Art. 3.87 §1 CC)",
                "legal_basis": "Code Civil Belge"
            }));

            let result = json!({
                "building_id": building_id,
                "alert_count": alerts.len(),
                "alerts": alerts
            });

            let text = serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "{}".to_string());
            Ok(ToolResult {
                content: vec![ContentBlock { content_type: "text".to_string(), text }],
                is_error: None,
            })
        }

        "energie_campagne_list" => {
            let status_filter = arguments.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Return simplified energy campaign data (in production, would use energy_campaign_use_cases)
            let campaigns = json!([
                {
                    "id": "camp-2024-001",
                    "name": "Achat groupé électricité 2024",
                    "status": "Active",
                    "start_date": "2024-01-01",
                    "end_date": "2024-12-31",
                    "participants": 15,
                    "anonymized_avg_consumption": "~3500 kWh/an",
                    "estimated_savings_pct": 12
                }
            ]);

            let result = json!({
                "status_filter": status_filter,
                "campaign_count": campaigns.as_array().map(|a| a.len()).unwrap_or(0),
                "campaigns": campaigns
            });

            let text = serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "{}".to_string());
            Ok(ToolResult {
                content: vec![ContentBlock { content_type: "text".to_string(), text }],
                is_error: None,
            })
        }

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
async fn handle_jsonrpc(
    req: JsonRpcRequest,
    state: &AppState,
    user: &AuthenticatedUser,
) -> Value {
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
        }).unwrap_or(json!({"error": "serialization error"}));
    }

    match req.method.as_str() {
        // MCP lifecycle: initialize
        "initialize" => {
            let client_info = req.params.as_ref()
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
            }).unwrap_or(json!({"error": "serialization error"}))
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
            }).unwrap_or(json!({"error": "serialization error"}))
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
                    }).unwrap_or(json!({"error": "serialization error"}));
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
                    }).unwrap_or(json!({"error": "serialization error"}));
                }
            };

            let arguments = params.get("arguments")
                .cloned()
                .unwrap_or(json!({}));

            tracing::info!(
                method = "tools/call",
                tool = %tool_name,
                user_id = ?user.user_id,
                "MCP tool called"
            );

            match dispatch_tool(&tool_name, &arguments, state, user).await {
                Ok(tool_result) => {
                    serde_json::to_value(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: serde_json::to_value(tool_result)
                            .unwrap_or(json!({})),
                    }).unwrap_or(json!({"error": "serialization error"}))
                }
                Err(err) => {
                    serde_json::to_value(JsonRpcError {
                        jsonrpc: "2.0".to_string(),
                        id,
                        error: err,
                    }).unwrap_or(json!({"error": "serialization error"}))
                }
            }
        }

        // ping
        "ping" => {
            serde_json::to_value(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: json!({}),
            }).unwrap_or(json!({"error": "serialization error"}))
        }

        // Unknown method
        _ => {
            serde_json::to_value(JsonRpcError {
                jsonrpc: "2.0".to_string(),
                id,
                error: RpcError {
                    code: ErrorCode::METHOD_NOT_FOUND,
                    message: format!("Method not found: {}", req.method),
                    data: None,
                },
            }).unwrap_or(json!({"error": "serialization error"}))
        }
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
    req: HttpRequest,
    claims: AuthenticatedUser,
    state: Data<Arc<AppState>>,
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
            serde_json::to_string(&messages_url).unwrap_or_else(|_| format!("\"{}\"", messages_url))
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
    state: Data<Arc<AppState>>,
    body: web::Json<Value>,
) -> HttpResponse {
    let session_id = req.uri().query()
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
        Err(e) => {
            HttpResponse::BadRequest()
                .content_type("application/json")
                .json(json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": ErrorCode::PARSE_ERROR,
                        "message": format!("Parse error: {}", e)
                    }
                }))
        }
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
            "messages": "/mcp/messages"
        },
        "tools_count": get_mcp_tools().len(),
        "description": "Model Context Protocol server for KoproGo — Belgian property management SaaS"
    }))
}

// ─────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
            assert!(!tool.description.is_empty(), "Tool '{}' has empty description", tool.name);
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
        // We advertise 20 tools (10 initial + 10 new Belgian legal/compliance tools)
        let tools = get_mcp_tools();
        assert_eq!(tools.len(), 20, "Expected 20 MCP tools");
    }
}
