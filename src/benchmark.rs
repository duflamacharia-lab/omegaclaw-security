use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkScore {
    pub benchmark_id: String,
    pub score: f32,
    pub passed: bool,
    pub checks: Vec<BenchmarkCheck>,
    pub evidence_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkCheck {
    pub id: String,
    pub passed: bool,
    pub weight: f32,
    pub reason: String,
}

pub fn score_result(result: &Value) -> anyhow::Result<BenchmarkScore> {
    let id = result
        .get("benchmark_id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("benchmark_id is required"))?
        .to_string();
    let execution = result
        .get("execution")
        .ok_or_else(|| anyhow::anyhow!("execution evidence is required"))?;
    let safety = result
        .get("safety")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let result_passed = execution.get("result").and_then(Value::as_str) == Some("passed");
    let no_egress = execution.get("network_egress").and_then(Value::as_bool) == Some(false);
    let local_execution = execution
        .get("network")
        .and_then(Value::as_str)
        .map(|v| v.contains("local") || v.contains("anvil") || v.contains("forge"))
        .unwrap_or(false);
    let safety_words = safety.iter().filter_map(Value::as_str).collect::<Vec<_>>();
    let no_live_assets = [
        "no live RPC",
        "no real wallet or funds",
        "no public-chain deployment",
    ]
    .iter()
    .all(|required| safety_words.iter().any(|item| item == required));
    let checks = vec![
        BenchmarkCheck {
            id: "result_passed".into(),
            passed: result_passed,
            weight: 0.4,
            reason: "the recorded benchmark execution passed".into(),
        },
        BenchmarkCheck {
            id: "network_is_local".into(),
            passed: local_execution,
            weight: 0.2,
            reason: "execution is bound to a local defensive fixture".into(),
        },
        BenchmarkCheck {
            id: "network_egress_disabled".into(),
            passed: no_egress,
            weight: 0.2,
            reason: "network egress is explicitly disabled".into(),
        },
        BenchmarkCheck {
            id: "no_live_assets".into(),
            passed: no_live_assets,
            weight: 0.2,
            reason: "safety evidence excludes live RPC, funds, and public deployment".into(),
        },
    ];
    let score = checks
        .iter()
        .filter(|check| check.passed)
        .map(|check| check.weight)
        .sum();
    let canonical = serde_json::to_vec(result)?;
    let evidence_hash = hex::encode(Sha256::digest(canonical));
    Ok(BenchmarkScore {
        benchmark_id: id,
        score,
        passed: (score - 1.0).abs() < f32::EPSILON,
        checks,
        evidence_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scores_local_dvd_result() {
        let result: Value = serde_json::json!({
            "benchmark_id":"dvd-side-entrance-local",
            "execution":{"network":"local_anvil_forge_vm","network_egress":false,"result":"passed"},
            "safety":["no live RPC","no real wallet or funds","no public-chain deployment"]
        });
        let score = score_result(&result).unwrap();
        assert!(score.passed);
        assert_eq!(score.score, 1.0);
    }
}

pub fn score_file(path: &std::path::Path) -> anyhow::Result<BenchmarkScore> {
    score_result(&serde_json::from_slice(&std::fs::read(path)?)?)
}

pub fn score_result_json(result: &Value) -> Value {
    score_result(result)
        .map(|score| serde_json::to_value(score).unwrap_or(Value::Null))
        .unwrap_or_else(|error| serde_json::json!({"error": error.to_string()}))
}

pub fn stable_id(result: &Value) -> String {
    hex::encode(Sha256::digest(
        serde_json::to_vec(result).unwrap_or_default(),
    ))
}

pub fn _ensure_object(value: &Value) -> anyhow::Result<()> {
    if value.is_object() {
        Ok(())
    } else {
        anyhow::bail!("benchmark result must be a JSON object")
    }
}
