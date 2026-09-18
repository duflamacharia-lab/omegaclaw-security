use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorpusRole {
    TrainingCandidate,
    EvaluationOnly,
    ReadOnlyFixture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub id: String,
    pub name: String,
    pub role: CorpusRole,
    pub source_url: String,
    pub revision: String,
    pub license: String,
    pub attribution: String,
    pub hash_scope: String,
    pub sha256: String,
    pub content_kind: String,
    pub network_required: bool,
    pub executable_content: bool,
    pub solution_material_separated: bool,
    pub safety_notes: Vec<String>,
}

impl ArtifactManifest {
    pub fn validate(&self) -> Result<(), String> {
        for (field, value) in [
            ("id", &self.id),
            ("source_url", &self.source_url),
            ("revision", &self.revision),
            ("license", &self.license),
            ("sha256", &self.sha256),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{field} is required"));
            }
        }
        if self.sha256.len() != 64 || !self.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("sha256 must be a 64-character hexadecimal digest".into());
        }
        if self.hash_scope.trim().is_empty() {
            return Err("hash_scope is required".into());
        }
        if self.role == CorpusRole::TrainingCandidate && self.executable_content {
            return Err("executable content cannot enter a training candidate".into());
        }
        if self.role == CorpusRole::EvaluationOnly && !self.solution_material_separated {
            return Err("evaluation-only artifacts require solution separation".into());
        }
        Ok(())
    }
}

pub fn sha256_file(path: impl AsRef<Path>) -> std::io::Result<String> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unhashed_manifest() {
        let manifest = ArtifactManifest {
            id: "x".into(),
            name: "x".into(),
            role: CorpusRole::TrainingCandidate,
            source_url: "https://example.test".into(),
            revision: "abc".into(),
            license: "MIT".into(),
            attribution: "x".into(),
            hash_scope: "content".into(),
            sha256: "bad".into(),
            content_kind: "text".into(),
            network_required: false,
            executable_content: false,
            solution_material_separated: true,
            safety_notes: vec![],
        };
        assert!(manifest.validate().is_err());
    }
    #[test]
    fn hashes_bytes_deterministically() {
        assert_eq!(sha256_bytes(b"omega"), sha256_bytes(b"omega"));
    }
}
