use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use omegaclaw::policy::{evaluate, Decision, PolicyInput};
use omegaclaw::storage::{CreateStoredCase, SqliteStore, StoredAsset, StoredEvidenceInput};
use omegaclaw::{
    benchmark::score_result_json,
    ctf::{plan_import, CtfImportRequest},
    mcp::{McpCall, McpClient},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    store: SqliteStore,
    client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Asset {
    id: String,
    name: String,
    chain: String,
    address: String,
    source_commit: String,
    block_snapshot: u64,
    authority_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Evidence {
    kind: String,
    source: String,
    summary: String,
    confidence: f32,
    artifact_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaseFile {
    id: Uuid,
    title: String,
    severity: String,
    status: String,
    asset: Asset,
    finding_confidence: f32,
    impact_confidence: f32,
    evidence: Vec<Evidence>,
    recommended_action: String,
    policy_decision: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateCase {
    title: String,
    severity: String,
    asset: Asset,
    evidence: Vec<Evidence>,
    recommended_action: Option<String>,
}

#[derive(Debug, Serialize)]
struct Health {
    service: &'static str,
    status: &'static str,
    providers: ProviderStatus,
}

#[derive(Debug, Serialize)]
struct ProviderStatus {
    dify: bool,
    gemini: bool,
    huggingface: bool,
}

#[derive(Debug, Serialize)]
struct PolicyResult {
    decision: String,
    reasons: Vec<String>,
    next_steps: Vec<String>,
}

fn provider_status() -> ProviderStatus {
    ProviderStatus {
        dify: std::env::var("DIFY_API_URL").is_ok(),
        gemini: std::env::var("GEMINI_API_KEY").is_ok(),
        huggingface: std::env::var("HUGGINGFACE_API_TOKEN").is_ok(),
    }
}

fn evaluate_policy(case: &CreateCase) -> PolicyResult {
    let decision = evaluate(&PolicyInput {
        severity: case.severity.clone(),
        has_asset_identity: !case.asset.id.is_empty() && !case.asset.address.is_empty(),
        has_source_snapshot: !case.asset.source_commit.is_empty(),
        has_deployment_snapshot: case.asset.block_snapshot > 0,
        evidence_count: case.evidence.len(),
        evidence_is_reproducible: case.evidence.iter().all(|e| e.artifact_uri.is_some()),
        ..Default::default()
    });
    let decision_name = match decision.decision {
        Decision::Abstain => "abstain",
        Decision::HumanReviewRequired => "human_review_required",
        Decision::AllowedStagedAction => "allowed_staged_action",
        Decision::Reviewable => "reviewable",
    };
    PolicyResult {
        decision: decision_name.into(),
        reasons: decision.reasons,
        next_steps: decision.required_controls,
    }
}

async fn health() -> impl IntoResponse {
    Json(Health {
        service: "omegaclaw-api",
        status: "ok",
        providers: provider_status(),
    })
}

async fn list_cases(State(state): State<AppState>) -> impl IntoResponse {
    match state.store.list_cases().await {
        Ok(cases) => Json(cases).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": error.to_string()})),
        )
            .into_response(),
    }
}

async fn create_case(
    State(state): State<AppState>,
    Json(input): Json<CreateCase>,
) -> impl IntoResponse {
    let result = state
        .store
        .insert_case(CreateStoredCase {
            title: input.title,
            severity: input.severity,
            asset: StoredAsset {
                id: input.asset.id,
                name: input.asset.name,
                chain: input.asset.chain,
                address: input.asset.address,
                source_commit: input.asset.source_commit,
                compiler: None,
                block_snapshot: input.asset.block_snapshot,
                authority_summary: input.asset.authority_summary,
                updated_at: Utc::now(),
            },
            evidence: input
                .evidence
                .into_iter()
                .map(|e| StoredEvidenceInput {
                    kind: e.kind,
                    source: e.source,
                    summary: e.summary,
                    confidence: e.confidence,
                    reproducible: e.artifact_uri.is_some(),
                    artifact_uri: e.artifact_uri,
                })
                .collect(),
        })
        .await;
    match result {
        Ok(case) => (StatusCode::CREATED, Json(case)).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": error.to_string()})),
        )
            .into_response(),
    }
}

async fn case_policy(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match state.store.get_case(&id.to_string()).await {
        Ok(Some(case)) => (StatusCode::OK, Json(serde_json::json!({"case_id": case.id, "decision": case.policy_decision, "policy": case.policy_json}))).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": error.to_string()}))).into_response(),
    }
}

async fn provider_probe(State(state): State<AppState>) -> impl IntoResponse {
    let mut result = serde_json::json!({"dify": {"configured": false}, "gemini": {"configured": false}, "huggingface": {"configured": false}});
    if let Ok(url) = std::env::var("DIFY_API_URL") {
        result["dify"] =
            serde_json::json!({"configured": true, "base_url": url, "mode": "adapter-ready"});
    }
    if std::env::var("GEMINI_API_KEY").is_ok() {
        result["gemini"] = serde_json::json!({"configured": true, "mode": "adapter-ready"});
    }
    if std::env::var("HUGGINGFACE_API_TOKEN").is_ok() {
        result["huggingface"] = serde_json::json!({"configured": true, "mode": "adapter-ready"});
    }
    let _ = &state.client;
    Json(result)
}

async fn mcp_tools(State(state): State<AppState>) -> impl IntoResponse {
    match McpClient::from_env(state.client.clone()) {
        Ok(client) => match client.list_tools().await {
            Ok(tools) => {
                Json(serde_json::json!({"configured": true, "tools": tools})).into_response()
            }
            Err(error) => (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": error.to_string()})),
            )
                .into_response(),
        },
        Err(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"configured": false, "error": error.to_string()})),
        )
            .into_response(),
    }
}

async fn mcp_call(State(state): State<AppState>, Json(call): Json<McpCall>) -> impl IntoResponse {
    let audit_tool = call.tool.clone();
    let audit_args = call.arguments.clone();
    match McpClient::from_env(state.client.clone()) {
        Ok(client) => match client.call(call).await {
            Ok(result) => {
                let _ = state
                    .store
                    .record_mcp_call(&audit_tool, &audit_args, &Ok(result.clone()))
                    .await;
                Json(serde_json::json!({"ok": true, "result": result})).into_response()
            }
            Err(error) => (
                {
                    let message = error.to_string();
                    let _ = state
                        .store
                        .record_mcp_call(&audit_tool, &audit_args, &Err(message.clone()))
                        .await;
                    StatusCode::BAD_GATEWAY
                },
                Json(serde_json::json!({"ok": false, "error": error.to_string()})),
            )
                .into_response(),
        },
        Err(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"ok": false, "error": error.to_string()})),
        )
            .into_response(),
    }
}

async fn audit_events(State(state): State<AppState>) -> impl IntoResponse {
    match state.store.list_audit_events().await {
        Ok(events) => Json(events).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": error.to_string()})),
        )
            .into_response(),
    }
}

async fn benchmark_score(Json(result): Json<serde_json::Value>) -> impl IntoResponse {
    Json(score_result_json(&result))
}

async fn ctf_import(
    State(state): State<AppState>,
    Json(request): Json<CtfImportRequest>,
) -> impl IntoResponse {
    match plan_import(request) {
        Ok(plan) => {
            let case = state
                .store
                .insert_case(CreateStoredCase {
                    title: format!("CTF intake: {}", plan.host),
                    severity: "medium".into(),
                    asset: StoredAsset {
                        id: format!("ctf-{}", plan.host.replace('.', "-")),
                        name: plan.host.clone(),
                        chain: "local-fixture".into(),
                        address: "unresolved".into(),
                        source_commit: "intake-pending-pinned-revision".into(),
                        compiler: None,
                        block_snapshot: 0,
                        authority_summary: "unresolved; human review required".into(),
                        updated_at: Utc::now(),
                    },
                    evidence: vec![StoredEvidenceInput {
                        kind: "ctf_import".into(),
                        source: plan.source_url.clone(),
                        summary: format!("Validated {} intake in metadata-only mode", plan.mode),
                        confidence: 0.6,
                        reproducible: false,
                        artifact_uri: Some(plan.source_url.clone()),
                    }],
                })
                .await;
            match case {
                Ok(case) => (
                    StatusCode::CREATED,
                    Json(serde_json::json!({"plan": plan, "case_file": case})),
                )
                    .into_response(),
                Err(error) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": error.to_string(), "plan": plan})),
                )
                    .into_response(),
            }
        }
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": error})),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "omegaclaw=info,tower_http=info".into()),
        )
        .init();
    let db_path = std::env::var("OMEGACLAW_DB_PATH").unwrap_or_else(|_| "omegaclaw.sqlite3".into());
    let store = SqliteStore::open(db_path)
        .await
        .expect("SQLite migration failed");
    let state = AppState {
        store,
        client: Client::new(),
    };
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/cases", get(list_cases).post(create_case))
        .route("/api/cases/:id/policy", get(case_policy))
        .route("/api/providers", get(provider_probe))
        .route("/api/mcp/tools", get(mcp_tools))
        .route("/api/mcp/call", axum::routing::post(mcp_call))
        .route("/api/audit/events", get(audit_events))
        .route(
            "/api/benchmarks/score",
            axum::routing::post(benchmark_score),
        )
        .route("/api/ctf/import", axum::routing::post(ctf_import))
        .nest_service("/", ServeDir::new("frontend/dist"))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    let port = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, "OmegaClaw API listening");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

fn seed_cases() -> Vec<CaseFile> {
    let asset = Asset {
        id: "asset-euler-like-vault".into(),
        name: "Atlas Vault (staging fixture)".into(),
        chain: "anvil".into(),
        address: "0x1111111111111111111111111111111111111111".into(),
        source_commit: "fixture-2026-09-18".into(),
        block_snapshot: 19842001,
        authority_summary: "2-of-3 security multisig; pause is staged-only".into(),
    };
    vec![CaseFile {
        id: Uuid::new_v4(),
        title: "Unexpected oracle deviation exceeds invariant threshold".into(),
        severity: "high".into(),
        status: "open".into(),
        asset,
        finding_confidence: 0.92,
        impact_confidence: 0.71,
        evidence: vec![Evidence {
            kind: "runtime_alert".into(),
            source: "forta-adapter-fixture".into(),
            summary: "Price movement diverges from configured oracle bounds".into(),
            confidence: 0.92,
            artifact_uri: Some("artifacts/fixtures/oracle-deviation.json".into()),
        }],
        recommended_action: "Simulate a staged pause and verify dependent withdrawal paths".into(),
        policy_decision: "human_review_required".into(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_evidence_abstains() {
        let c = CreateCase {
            title: "x".into(),
            severity: "low".into(),
            asset: seed_cases()[0].asset.clone(),
            evidence: vec![],
            recommended_action: None,
        };
        assert_eq!(evaluate_policy(&c).decision, "abstain");
    }
    #[test]
    fn high_impact_requires_review() {
        let mut c = CreateCase {
            title: "x".into(),
            severity: "critical".into(),
            asset: seed_cases()[0].asset.clone(),
            evidence: vec![Evidence {
                kind: "test".into(),
                source: "foundry".into(),
                summary: "reproduced".into(),
                confidence: 0.9,
                artifact_uri: Some("artifacts/test-reproduction.json".into()),
            }],
            recommended_action: None,
        };
        assert_eq!(evaluate_policy(&c).decision, "human_review_required");
        c.severity = "low".into();
        assert_eq!(evaluate_policy(&c).decision, "reviewable");
    }
}
