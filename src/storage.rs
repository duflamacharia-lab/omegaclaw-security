use crate::policy::{evaluate, PolicyInput};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::task;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAsset {
    pub id: String,
    pub name: String,
    pub chain: String,
    pub address: String,
    pub source_commit: String,
    pub compiler: Option<String>,
    pub block_snapshot: u64,
    pub authority_summary: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvidence {
    pub id: String,
    pub case_id: String,
    pub kind: String,
    pub source: String,
    pub summary: String,
    pub confidence: f32,
    pub reproducible: bool,
    pub artifact_uri: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCaseFile {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub status: String,
    pub asset_id: String,
    pub policy_decision: String,
    pub policy_json: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStoredCase {
    pub title: String,
    pub severity: String,
    pub asset: StoredAsset,
    pub evidence: Vec<StoredEvidenceInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvidenceInput {
    pub kind: String,
    pub source: String,
    pub summary: String,
    pub confidence: f32,
    pub reproducible: bool,
    pub artifact_uri: Option<String>,
}

#[derive(Clone)]
pub struct SqliteStore {
    path: Arc<PathBuf>,
}

impl SqliteStore {
    pub async fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let init_path = path.clone();
        task::spawn_blocking(move || -> anyhow::Result<()> {
            let conn = Connection::open(&init_path)?;
            migrate(&conn)?;
            Ok(())
        })
        .await??;
        Ok(Self {
            path: Arc::new(path),
        })
    }

    pub async fn insert_case(&self, input: CreateStoredCase) -> anyhow::Result<StoredCaseFile> {
        let path = self.path.clone();
        task::spawn_blocking(move || -> anyhow::Result<StoredCaseFile> {
            let conn = Connection::open(path.as_ref())?;
            let policy_input = PolicyInput { severity: input.severity.clone(), has_asset_identity: !input.asset.id.is_empty() && !input.asset.address.is_empty(), has_source_snapshot: !input.asset.source_commit.is_empty(), has_deployment_snapshot: input.asset.block_snapshot > 0, evidence_count: input.evidence.len(), evidence_is_reproducible: input.evidence.iter().all(|e| e.reproducible), ..Default::default() };
            let decision = evaluate(&policy_input);
            let now = Utc::now(); let asset_id = input.asset.id.clone();
            conn.execute("INSERT INTO assets (id,name,chain,address,source_commit,compiler,block_snapshot,authority_summary,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(id) DO UPDATE SET name=excluded.name, chain=excluded.chain, address=excluded.address, source_commit=excluded.source_commit, compiler=excluded.compiler, block_snapshot=excluded.block_snapshot, authority_summary=excluded.authority_summary, updated_at=excluded.updated_at", params![input.asset.id, input.asset.name, input.asset.chain, input.asset.address, input.asset.source_commit, input.asset.compiler, input.asset.block_snapshot, input.asset.authority_summary, now.to_rfc3339()])?;
            let case_id = Uuid::new_v4().to_string(); let policy_json = serde_json::to_value(&decision)?; let decision_name = match decision.decision { crate::policy::Decision::Abstain => "abstain", crate::policy::Decision::HumanReviewRequired => "human_review_required", crate::policy::Decision::Reviewable => "reviewable", crate::policy::Decision::AllowedStagedAction => "allowed_staged_action" }.to_string();
            conn.execute("INSERT INTO case_files (id,title,severity,status,asset_id,policy_decision,policy_json,created_at,updated_at) VALUES (?1,?2,?3,'open',?4,?5,?6,?7,?7)", params![case_id, input.title, input.severity, asset_id, decision_name, policy_json.to_string(), now.to_rfc3339()])?;
            for evidence in input.evidence { conn.execute("INSERT INTO evidence (id,case_id,kind,source,summary,confidence,reproducible,artifact_uri,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![Uuid::new_v4().to_string(), case_id, evidence.kind, evidence.source, evidence.summary, evidence.confidence, evidence.reproducible as i32, evidence.artifact_uri, now.to_rfc3339()])?; }
            conn.execute("INSERT INTO audit_events (id,event,payload,created_at) VALUES (?1,'case_created',?2,?3)", params![Uuid::new_v4().to_string(), serde_json::json!({"case_id":case_id,"policy":policy_json}).to_string(), now.to_rfc3339()])?;
            Ok(StoredCaseFile { id: case_id, title: input.title, severity: input.severity, status: "open".into(), asset_id, policy_decision: decision_name, policy_json, created_at: now, updated_at: now })
        }).await?
    }

    pub async fn list_cases(&self) -> anyhow::Result<Vec<StoredCaseFile>> {
        let path = self.path.clone();
        task::spawn_blocking(move || -> anyhow::Result<Vec<StoredCaseFile>> { let conn = Connection::open(path.as_ref())?; let mut stmt = conn.prepare("SELECT id,title,severity,status,asset_id,policy_decision,policy_json,created_at,updated_at FROM case_files ORDER BY created_at DESC")?; let rows = stmt.query_map([], |r| { let created: String = r.get(7)?; let updated: String = r.get(8)?; Ok(StoredCaseFile { id:r.get(0)?,title:r.get(1)?,severity:r.get(2)?,status:r.get(3)?,asset_id:r.get(4)?,policy_decision:r.get(5)?,policy_json:serde_json::from_str(&r.get::<_,String>(6)?).unwrap_or(Value::Null),created_at:created.parse().unwrap_or_else(|_| Utc::now()),updated_at:updated.parse().unwrap_or_else(|_| Utc::now()) }) })?.collect::<Result<Vec<_>,_>>()?; Ok(rows) }).await?
    }

    pub async fn get_case(&self, id: &str) -> anyhow::Result<Option<StoredCaseFile>> {
        let path = self.path.clone();
        let id = id.to_string();
        task::spawn_blocking(move || -> anyhow::Result<Option<StoredCaseFile>> { let conn=Connection::open(path.as_ref())?; let row=conn.query_row("SELECT id,title,severity,status,asset_id,policy_decision,policy_json,created_at,updated_at FROM case_files WHERE id=?1", params![id], |r| { Ok(StoredCaseFile { id:r.get(0)?,title:r.get(1)?,severity:r.get(2)?,status:r.get(3)?,asset_id:r.get(4)?,policy_decision:r.get(5)?,policy_json:serde_json::from_str(&r.get::<_,String>(6)?).unwrap_or(Value::Null),created_at:r.get::<_,String>(7)?.parse().unwrap_or_else(|_| Utc::now()),updated_at:r.get::<_,String>(8)?.parse().unwrap_or_else(|_| Utc::now()) }) }).optional()?; Ok(row) }).await?
    }
}

fn migrate(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; CREATE TABLE IF NOT EXISTS assets (id TEXT PRIMARY KEY, name TEXT NOT NULL, chain TEXT NOT NULL, address TEXT NOT NULL, source_commit TEXT NOT NULL, compiler TEXT, block_snapshot INTEGER NOT NULL, authority_summary TEXT NOT NULL, updated_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS case_files (id TEXT PRIMARY KEY, title TEXT NOT NULL, severity TEXT NOT NULL, status TEXT NOT NULL, asset_id TEXT NOT NULL REFERENCES assets(id), policy_decision TEXT NOT NULL, policy_json TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS evidence (id TEXT PRIMARY KEY, case_id TEXT NOT NULL REFERENCES case_files(id) ON DELETE CASCADE, kind TEXT NOT NULL, source TEXT NOT NULL, summary TEXT NOT NULL, confidence REAL NOT NULL, reproducible INTEGER NOT NULL, artifact_uri TEXT, created_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS audit_events (id TEXT PRIMARY KEY, event TEXT NOT NULL, payload TEXT NOT NULL, created_at TEXT NOT NULL); CREATE INDEX IF NOT EXISTS idx_case_created ON case_files(created_at); CREATE INDEX IF NOT EXISTS idx_evidence_case ON evidence(case_id);")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn case_file_round_trips_through_sqlite() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("cases.sqlite3");
        let store = SqliteStore::open(&path).await.unwrap();
        let case = store
            .insert_case(CreateStoredCase {
                title: "round trip".into(),
                severity: "high".into(),
                asset: StoredAsset {
                    id: "asset-1".into(),
                    name: "Vault".into(),
                    chain: "anvil".into(),
                    address: "0x1".into(),
                    source_commit: "abc".into(),
                    compiler: None,
                    block_snapshot: 1,
                    authority_summary: "2-of-3".into(),
                    updated_at: Utc::now(),
                },
                evidence: vec![StoredEvidenceInput {
                    kind: "test".into(),
                    source: "forge".into(),
                    summary: "reproduced".into(),
                    confidence: 0.9,
                    reproducible: true,
                    artifact_uri: Some("artifact.json".into()),
                }],
            })
            .await
            .unwrap();
        assert_eq!(case.policy_decision, "human_review_required");
        drop(store);
        let reopened = SqliteStore::open(&path).await.unwrap();
        let found = reopened.get_case(&case.id).await.unwrap().unwrap();
        assert_eq!(found.title, "round trip");
        assert_eq!(found.policy_json["decision"], "human_review_required");
    }
}
