use axum::{routing::post, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;

async fn rpc(Json(request): Json<Value>) -> Json<Value> {
    let id = request.get("id").cloned().unwrap_or(json!(1));
    match request.get("method").and_then(Value::as_str) {
        Some("tools/list") => Json(
            json!({"jsonrpc":"2.0","id":id,"result":{"tools":[{"name":"fixture.read","description":"Read one immutable fixture value","inputSchema":{"type":"object","properties":{"name":{"type":"string","minLength":1}},"required":["name"],"additionalProperties":false}}]}}),
        ),
        Some("tools/call") => {
            let params = request.get("params").cloned().unwrap_or_default();
            if params.get("name").and_then(Value::as_str) != Some("fixture.read") {
                return Json(
                    json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"tool not found"}}),
                );
            }
            let name = params
                .pointer("/arguments/name")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            Json(
                json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":format!("fixture:{name}")}],"isError":false}}),
            )
        }
        _ => Json(
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"read-only fixture supports tools/list and tools/call"}}),
        ),
    }
}

#[tokio::main]
async fn main() {
    let port = std::env::var("MCP_FIXTURE_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9100);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("OmegaClaw local MCP fixture listening on http://{addr}/mcp");
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        Router::new().route("/mcp", post(rpc)),
    )
    .await
    .unwrap();
}
