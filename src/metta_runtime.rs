use crate::policy::PolicyInput;
use hyperon::metta::{
    runner::{EnvBuilder, Metta},
    text::SExprParser,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct MettaRuntime {
    rules_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct MettaEvaluation {
    pub loaded_files: Vec<String>,
    pub predicates: Vec<String>,
    pub raw_results: Vec<String>,
}

impl MettaRuntime {
    pub fn new(rules_root: impl Into<PathBuf>) -> Self {
        Self {
            rules_root: rules_root.into(),
        }
    }

    pub fn evaluate(&self, input: &PolicyInput) -> Result<MettaEvaluation, String> {
        let files = [
            "evidence_policy.metta",
            "scope_drift_policy.metta",
            "abstention_policy.metta",
            "omegaclaw_rules.metta",
        ];
        let mut program = String::new();
        let mut loaded_files = Vec::new();
        for file in files {
            let path = self.rules_root.join(file);
            let source =
                fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            program.push_str(&source);
            program.push('\n');
            loaded_files.push(path.display().to_string());
        }
        program.push_str(&facts(input));
        program.push_str(
            "\n(must-abstain case)\n(scope-drift case)\n(requires-human-approval case action)\n",
        );
        let metta = Metta::new(Some(EnvBuilder::test_env()));
        let results = metta.run(SExprParser::new(program.as_str()))?;
        let raw_results: Vec<String> = results
            .iter()
            .map(|result| {
                result
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        let mut predicates = Vec::new();
        if input.has_asset_identity
            && input.has_source_snapshot
            && input.has_deployment_snapshot
            && input.evidence_count > 0
            && input.evidence_is_reproducible
        {
            predicates.push("evidence-complete".into());
        }
        if input.deployment_drift_detected {
            predicates.push("scope-drift".into());
        }
        if input.stale_rpc {
            predicates.push("stale-rpc".into());
        }
        if input.simulation_timed_out {
            predicates.push("simulation-timeout".into());
        }
        if input.authority_changed {
            predicates.push("authority-change".into());
        }
        if matches!(
            input.severity.to_ascii_lowercase().as_str(),
            "high" | "critical"
        ) {
            predicates.push("requires-human-review".into());
        }
        if predicates.is_empty() {
            predicates.push("must-abstain".into());
        }
        Ok(MettaEvaluation {
            loaded_files,
            predicates,
            raw_results,
        })
    }
}

fn facts(input: &PolicyInput) -> String {
    let mut facts = String::from("(has-asset-identity case)\n");
    if input.has_source_snapshot {
        facts.push_str("(has-source-snapshot case)\n");
    }
    if input.has_deployment_snapshot {
        facts.push_str("(has-deployment-snapshot case)\n");
    }
    if input.evidence_count > 0 {
        facts.push_str("(has-evidence case)\n");
    }
    if input.evidence_is_reproducible {
        facts.push_str("(has-reproducible-evidence case)\n");
    }
    if input.deployment_drift_detected {
        facts.push_str("(source-commit-changed case)\n");
    }
    if input.stale_rpc {
        facts.push_str("(stale-rpc case)\n");
    }
    if input.simulation_timed_out {
        facts.push_str("(simulation-timeout case)\n");
    }
    if input.authority_changed {
        facts.push_str("(authority-change case)\n");
    }
    facts
}

pub fn default_rules_root() -> &'static Path {
    Path::new("metta")
}
