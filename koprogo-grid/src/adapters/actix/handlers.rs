use crate::adapters::actix::dto::*;
use crate::core::{CarbonCredit, GreenProof, Node, Task, TaskType};
use crate::ports::*;
use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;
use uuid::Uuid;

pub struct AppState {
    pub node_repo: Arc<dyn NodeRepository>,
    pub task_repo: Arc<dyn TaskRepository>,
    pub proof_repo: Arc<dyn GreenProofRepository>,
    pub credit_repo: Arc<dyn CarbonCreditRepository>,
    pub distributor: Arc<dyn TaskDistributor>,
}

// POST /grid/register
pub async fn register_node(
    data: web::Data<Arc<AppState>>,
    req: web::Json<RegisterNodeRequest>,
) -> impl Responder {
    let node = match Node::new(
        req.name.clone(),
        req.cpu_cores,
        req.has_solar,
        req.location.clone(),
    ) {
        Ok(mut n) => {
            if let Some(id) = req.node_id {
                n.id = id;
            }
            n
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse { error: e });
        }
    };

    match data.node_repo.create(&node).await {
        Ok(node) => HttpResponse::Created().json(NodeResponse {
            id: node.id,
            name: node.name,
            cpu_cores: node.cpu_cores,
            has_solar: node.has_solar,
            location: node.location,
            status: format!("{:?}", node.status).to_lowercase(),
            eco_score: node.eco_score,
            total_energy_saved_wh: node.total_energy_saved_wh,
            total_carbon_credits: node.total_carbon_credits,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

// POST /grid/heartbeat
pub async fn heartbeat(
    data: web::Data<Arc<AppState>>,
    req: web::Json<HeartbeatRequest>,
) -> impl Responder {
    let mut node = match data.node_repo.find_by_id(req.node_id).await {
        Ok(Some(n)) => n,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Node not found".to_string(),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse { error: e });
        }
    };

    node.heartbeat();
    node.update_eco_score(req.cpu_usage, req.solar_watts);

    match data.node_repo.update(&node).await {
        Ok(node) => HttpResponse::Ok().json(NodeResponse {
            id: node.id,
            name: node.name,
            cpu_cores: node.cpu_cores,
            has_solar: node.has_solar,
            location: node.location,
            status: format!("{:?}", node.status).to_lowercase(),
            eco_score: node.eco_score,
            total_energy_saved_wh: node.total_energy_saved_wh,
            total_carbon_credits: node.total_carbon_credits,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

// GET /grid/task?node_id=...
pub async fn get_task(
    data: web::Data<Arc<AppState>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let node_id_str = match query.get("node_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Missing node_id parameter".to_string(),
            });
        }
    };

    let _node_id = match Uuid::parse_str(node_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid node_id format".to_string(),
            });
        }
    };

    // Find next pending task
    let task = match data.task_repo.find_next_pending().await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return HttpResponse::NoContent().finish();
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse { error: e });
        }
    };

    // Assign task to node
    match data.distributor.assign_task(task.id).await {
        Ok(_) => {
            // Fetch updated task
            match data.task_repo.find_by_id(task.id).await {
                Ok(Some(task)) => {
                    let reward = task.estimated_reward();
                    HttpResponse::Ok().json(TaskResponse {
                        id: task.id,
                        task_type: format!("{:?}", task.task_type).to_lowercase(),
                        status: format!("{:?}", task.status).to_lowercase(),
                        data_url: task.data_url,
                        deadline: task.deadline.to_rfc3339(),
                        estimated_reward: reward,
                    })
                },
                Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
                    error: "Task not found after assignment".to_string(),
                }),
                Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

// POST /grid/report
pub async fn report_task(
    data: web::Data<Arc<AppState>>,
    req: web::Json<ReportTaskRequest>,
) -> impl Responder {
    // Find the task
    let mut task = match data.task_repo.find_by_id(req.task_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Task not found".to_string(),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse { error: e });
        }
    };

    // Complete the task
    if let Err(e) = task.complete(req.result_hash.clone(), req.energy_used_wh) {
        return HttpResponse::BadRequest().json(ErrorResponse { error: e });
    }

    // Update task in DB
    if let Err(e) = data.task_repo.update(&task).await {
        return HttpResponse::InternalServerError().json(ErrorResponse { error: e });
    }

    // Get latest proof for blockchain chaining
    let previous_hash = match data.proof_repo.get_latest().await {
        Ok(Some(p)) => Some(p.block_hash),
        Ok(None) => None,
        Err(_) => None,
    };

    // Create Proof of Green
    let proof = match GreenProof::new(
        task.id,
        task.assigned_node_id.unwrap(),
        req.energy_used_wh,
        req.solar_contribution_wh,
        previous_hash,
    ) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse { error: e });
        }
    };

    // Save proof
    if let Err(e) = data.proof_repo.create(&proof).await {
        return HttpResponse::InternalServerError().json(ErrorResponse { error: e });
    }

    // Create carbon credit
    let mut credit = match CarbonCredit::new(
        task.assigned_node_id.unwrap(),
        task.id,
        proof.id,
        proof.carbon_saved_kg,
    ) {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse { error: e });
        }
    };

    // Auto-verify credit
    let _ = credit.verify();

    // Save credit
    if let Err(e) = data.credit_repo.create(&credit).await {
        return HttpResponse::InternalServerError().json(ErrorResponse { error: e });
    }

    // Update node stats
    if let Ok(Some(mut node)) = data.node_repo.find_by_id(task.assigned_node_id.unwrap()).await {
        node.add_energy_saved(proof.carbon_saved_kg * 1000.0); // Convert to Wh
        node.add_carbon_credits(credit.amount_kg_co2);
        let _ = data.node_repo.update(&node).await;
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Task completed successfully",
        "carbon_credits": credit.amount_kg_co2,
        "node_share_eur": credit.node_share,
        "cooperative_share_eur": credit.cooperative_share
    }))
}

// GET /grid/stats
pub async fn get_stats(data: web::Data<Arc<AppState>>) -> impl Responder {
    let node_stats = match data.node_repo.get_total_stats().await {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse { error: e });
        }
    };

    let task_stats = match data.task_repo.get_stats().await {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse { error: e });
        }
    };

    let coop_fund = match data.credit_repo.get_cooperative_fund().await {
        Ok(f) => f,
        Err(_) => 0.0,
    };

    HttpResponse::Ok().json(StatsResponse {
        nodes: NodeStatsResponse {
            total_nodes: node_stats.total_nodes,
            active_nodes: node_stats.active_nodes,
            total_cpu_cores: node_stats.total_cpu_cores,
            nodes_with_solar: node_stats.nodes_with_solar,
            total_energy_saved_wh: node_stats.total_energy_saved_wh,
            total_carbon_credits: node_stats.total_carbon_credits,
        },
        tasks: TaskStatsResponse {
            total_tasks: task_stats.total_tasks,
            pending_tasks: task_stats.pending_tasks,
            completed_tasks: task_stats.completed_tasks,
            failed_tasks: task_stats.failed_tasks,
        },
        cooperative_fund_eur: coop_fund,
    })
}

// POST /grid/task (create task - for admin/testing)
pub async fn create_task(
    data: web::Data<Arc<AppState>>,
    req: web::Json<CreateTaskRequest>,
) -> impl Responder {
    let task_type = match req.task_type.as_str() {
        "ml_train" => TaskType::MlTrain,
        "data_hash" => TaskType::DataHash,
        "render" => TaskType::Render,
        "scientific" => TaskType::Scientific,
        _ => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid task type".to_string(),
            });
        }
    };

    let task = match Task::new(task_type, req.data_url.clone(), req.deadline_minutes) {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse { error: e });
        }
    };

    match data.task_repo.create(&task).await {
        Ok(task) => {
            let reward = task.estimated_reward();
            HttpResponse::Created().json(TaskResponse {
                id: task.id,
                task_type: format!("{:?}", task.task_type).to_lowercase(),
                status: format!("{:?}", task.status).to_lowercase(),
                data_url: task.data_url,
                deadline: task.deadline.to_rfc3339(),
                estimated_reward: reward,
            })
        },
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}
