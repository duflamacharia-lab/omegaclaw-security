use crate::policy::{evaluate, PolicyInput};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub seq: i64,
    pub event: String,
    pub payload: Value,
    pub prev_hash: Option<String>,
    pub event_hash: String,
    pub created_at: DateTime<Utc>,
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
        task::spawn_blocking(move||->anyhow::Result<StoredCaseFile>{
            let mut conn=Connection::open(path.as_ref())?; migrate(&conn)?;
            let policy_input=PolicyInput{severity:input.severity.clone(),has_asset_identity:!input.asset.id.is_empty()&&!input.asset.address.is_empty(),has_source_snapshot:!input.asset.source_commit.is_empty(),has_deployment_snapshot:input.asset.block_snapshot>0,evidence_count:input.evidence.len(),evidence_is_reproducible:input.evidence.iter().all(|e|e.reproducible),..Default::default()};
            let decision=evaluate(&policy_input); let now=Utc::now(); let asset_id=input.asset.id.clone();
            let tx=conn.transaction()?;
            tx.execute("INSERT INTO assets (id,name,chain,address,source_commit,compiler,block_snapshot,authority_summary,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(id) DO UPDATE SET name=excluded.name,chain=excluded.chain,address=excluded.address,source_commit=excluded.source_commit,compiler=excluded.compiler,block_snapshot=excluded.block_snapshot,authority_summary=excluded.authority_summary,updated_at=excluded.updated_at",params![input.asset.id,input.asset.name,input.asset.chain,input.asset.address,input.asset.source_commit,input.asset.compiler,input.asset.block_snapshot,input.asset.authority_summary,now.to_rfc3339()])?;
            let case_id=Uuid::new_v4().to_string(); let policy_json=serde_json::to_value(&decision)?; let decision_name=decision_name(&decision.decision);
            tx.execute("INSERT INTO case_files (id,title,severity,status,asset_id,policy_decision,policy_json,created_at,updated_at) VALUES (?1,?2,?3,'open',?4,?5,?6,?7,?7)",params![case_id,input.title,input.severity,asset_id,decision_name,policy_json.to_string(),now.to_rfc3339()])?;
            for evidence in input.evidence { tx.execute("INSERT INTO evidence (id,case_id,kind,source,summary,confidence,reproducible,artifact_uri,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![Uuid::new_v4().to_string(),case_id,evidence.kind,evidence.source,evidence.summary,evidence.confidence,evidence.reproducible as i32,evidence.artifact_uri,now.to_rfc3339()])?; }
            append_event(&tx,"case_created",&serde_json::json!({"case_id":case_id,"policy":policy_json}))?; tx.commit()?;
            Ok(StoredCaseFile{id:case_id,title:input.title,severity:input.severity,status:"open".into(),asset_id,policy_decision:decision_name,policy_json,created_at:now,updated_at:now})
        }).await?
    }

    pub async fn record_mcp_call(
        &self,
        tool: &str,
        arguments: &Value,
        result: &Result<Value, String>,
    ) -> anyhow::Result<()> {
        let path = self.path.clone();
        let tool = tool.to_string();
        let arguments = arguments.clone();
        let result = result.clone();
        task::spawn_blocking(move||->anyhow::Result<()>{ let mut conn=Connection::open(path.as_ref())?; migrate(&conn)?; let payload=serde_json::json!({"tool":tool,"arguments":arguments,"ok":result.is_ok(),"result":result.as_ref().ok(),"error":result.as_ref().err()}); let tx=conn.transaction()?; append_event(&tx,"mcp_call",&payload)?; tx.commit()?; Ok(()) }).await??;
        Ok(())
    }

    pub async fn record_event(&self, event: &str, payload: &Value) -> anyhow::Result<()> {
        let path = self.path.clone();
        let event = event.to_string();
        let payload = payload.clone();
        task::spawn_blocking(move || -> anyhow::Result<()> {
            let mut conn = Connection::open(path.as_ref())?;
            migrate(&conn)?;
            let tx = conn.transaction()?;
            append_event(&tx, &event, &payload)?;
            tx.commit()?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    pub async fn list_audit_events(&self) -> anyhow::Result<Vec<AuditEvent>> {
        let path = self.path.clone();
        task::spawn_blocking(move||->anyhow::Result<Vec<AuditEvent>>{let conn=Connection::open(path.as_ref())?;let mut stmt=conn.prepare("SELECT id,COALESCE(seq,0),event,payload,prev_hash,COALESCE(event_hash,''),created_at FROM audit_events ORDER BY seq,id")?;let rows=stmt.query_map([],|r|Ok(AuditEvent{id:r.get(0)?,seq:r.get(1)?,event:r.get(2)?,payload:serde_json::from_str(&r.get::<_,String>(3)?).unwrap_or(Value::Null),prev_hash:r.get(4)?,event_hash:r.get(5)?,created_at:r.get::<_,String>(6)?.parse().unwrap_or_else(|_|Utc::now())}))?.collect::<Result<Vec<_>,_>>()?;Ok(rows)}).await?
    }

    pub async fn list_cases(&self) -> anyhow::Result<Vec<StoredCaseFile>> {
        let path = self.path.clone();
        task::spawn_blocking(move || -> anyhow::Result<Vec<StoredCaseFile>> {
            let conn = Connection::open(path.as_ref())?;
            query_cases(&conn, None)
        })
        .await?
    }
    pub async fn get_case(&self, id: &str) -> anyhow::Result<Option<StoredCaseFile>> {
        let path = self.path.clone();
        let id = id.to_string();
        task::spawn_blocking(move || -> anyhow::Result<Option<StoredCaseFile>> {
            let conn = Connection::open(path.as_ref())?;
            let row = query_cases(&conn, Some(&id))?.into_iter().next();
            Ok(row)
        })
        .await?
    }
}

fn decision_name(decision: &crate::policy::Decision) -> String {
    match decision {
        crate::policy::Decision::Abstain => "abstain",
        crate::policy::Decision::HumanReviewRequired => "human_review_required",
        crate::policy::Decision::Reviewable => "reviewable",
        crate::policy::Decision::AllowedStagedAction => "allowed_staged_action",
    }
    .into()
}
fn query_cases(conn: &Connection, only: Option<&str>) -> anyhow::Result<Vec<StoredCaseFile>> {
    let sql = if only.is_some() {
        "SELECT id,title,severity,status,asset_id,policy_decision,policy_json,created_at,updated_at FROM case_files WHERE id=?1 ORDER BY created_at DESC"
    } else {
        "SELECT id,title,severity,status,asset_id,policy_decision,policy_json,created_at,updated_at FROM case_files ORDER BY created_at DESC"
    };
    let mut stmt = conn.prepare(sql)?;
    let mapper = |r: &rusqlite::Row<'_>| -> rusqlite::Result<StoredCaseFile> {
        Ok(StoredCaseFile {
            id: r.get(0)?,
            title: r.get(1)?,
            severity: r.get(2)?,
            status: r.get(3)?,
            asset_id: r.get(4)?,
            policy_decision: r.get(5)?,
            policy_json: serde_json::from_str(&r.get::<_, String>(6)?).unwrap_or(Value::Null),
            created_at: r
                .get::<_, String>(7)?
                .parse()
                .unwrap_or_else(|_| Utc::now()),
            updated_at: r
                .get::<_, String>(8)?
                .parse()
                .unwrap_or_else(|_| Utc::now()),
        })
    };
    let rows = if let Some(id) = only {
        stmt.query_map(params![id], mapper)?
            .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map([], mapper)?.collect::<Result<Vec<_>, _>>()?
    };
    Ok(rows)
}
fn append_event(
    tx: &rusqlite::Transaction<'_>,
    event: &str,
    payload: &Value,
) -> anyhow::Result<()> {
    let seq: i64 = tx.query_row("SELECT COALESCE(MAX(seq),0)+1 FROM audit_events", [], |r| {
        r.get(0)
    })?;
    let prev: Option<String> = tx
        .query_row(
            "SELECT event_hash FROM audit_events ORDER BY seq DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let payload_text = serde_json::to_string(payload)?;
    let material = format!(
        "{}|{}|{}|{}",
        seq,
        prev.as_deref().unwrap_or(""),
        event,
        payload_text
    );
    let hash = hex::encode(Sha256::digest(material.as_bytes()));
    tx.execute("INSERT INTO audit_events (id,seq,event,payload,prev_hash,event_hash,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![Uuid::new_v4().to_string(),seq,event,payload_text,prev,hash,Utc::now().to_rfc3339()])?;
    Ok(())
}
fn add_column(conn: &Connection, sql: &str) {
    let _ = conn.execute(sql, []);
}
fn migrate(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; CREATE TABLE IF NOT EXISTS assets (id TEXT PRIMARY KEY,name TEXT NOT NULL,chain TEXT NOT NULL,address TEXT NOT NULL,source_commit TEXT NOT NULL,compiler TEXT,block_snapshot INTEGER NOT NULL,authority_summary TEXT NOT NULL,updated_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS case_files (id TEXT PRIMARY KEY,title TEXT NOT NULL,severity TEXT NOT NULL,status TEXT NOT NULL,asset_id TEXT NOT NULL REFERENCES assets(id),policy_decision TEXT NOT NULL,policy_json TEXT NOT NULL,created_at TEXT NOT NULL,updated_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS evidence (id TEXT PRIMARY KEY,case_id TEXT NOT NULL REFERENCES case_files(id) ON DELETE CASCADE,kind TEXT NOT NULL,source TEXT NOT NULL,summary TEXT NOT NULL,confidence REAL NOT NULL,reproducible INTEGER NOT NULL,artifact_uri TEXT,created_at TEXT NOT NULL); CREATE TABLE IF NOT EXISTS audit_events (id TEXT PRIMARY KEY,event TEXT NOT NULL,payload TEXT NOT NULL,created_at TEXT NOT NULL); CREATE INDEX IF NOT EXISTS idx_case_created ON case_files(created_at); CREATE INDEX IF NOT EXISTS idx_evidence_case ON evidence(case_id);")?;
    add_column(conn, "ALTER TABLE audit_events ADD COLUMN seq INTEGER");
    add_column(conn, "ALTER TABLE audit_events ADD COLUMN prev_hash TEXT");
    add_column(conn, "ALTER TABLE audit_events ADD COLUMN event_hash TEXT");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn case_and_mcp_audit_round_trip() {
        let d = tempfile::tempdir().unwrap();
        let s = SqliteStore::open(d.path().join("x.db")).await.unwrap();
        let c = s
            .insert_case(CreateStoredCase {
                title: "round trip".into(),
                severity: "high".into(),
                asset: StoredAsset {
                    id: "a".into(),
                    name: "Vault".into(),
                    chain: "anvil".into(),
                    address: "0x1".into(),
                    source_commit: "abc".into(),
                    compiler: None,
                    block_snapshot: 1,
                    authority_summary: "review".into(),
                    updated_at: Utc::now(),
                },
                evidence: vec![StoredEvidenceInput {
                    kind: "test".into(),
                    source: "forge".into(),
                    summary: "reproduced".into(),
                    confidence: 0.9,
                    reproducible: true,
                    artifact_uri: Some("a.json".into()),
                }],
            })
            .await
            .unwrap();
        assert_eq!(c.policy_decision, "human_review_required");
        s.record_mcp_call(
            "fixture.read",
            &serde_json::json!({"name":"x"}),
            &Ok(serde_json::json!({"ok":true})),
        )
        .await
        .unwrap();
        let events = s.list_audit_events().await.unwrap();
        assert!(events.iter().any(|e| e.event == "mcp_call"));
        assert!(events.windows(2).all(|w| w[1].seq > w[0].seq));
    }
}
