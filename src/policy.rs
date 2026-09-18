use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyInput {
    pub case_id: Option<String>,
    pub severity: String,
    pub has_asset_identity: bool,
    pub has_source_snapshot: bool,
    pub has_deployment_snapshot: bool,
    pub evidence_count: usize,
    pub evidence_is_reproducible: bool,
    pub authority_changed: bool,
    pub deployment_drift_detected: bool,
    pub stale_rpc: bool,
    pub simulation_timed_out: bool,
    pub requested_action: Option<String>,
    pub action_is_reversible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub decision: Decision,
    pub reasons: Vec<String>,
    pub required_controls: Vec<String>,
    pub metta_predicates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Abstain,
    HumanReviewRequired,
    Reviewable,
    AllowedStagedAction,
}

impl PolicyDecision {
    pub fn abstain(reasons: Vec<String>) -> Self {
        Self {
            decision: Decision::Abstain,
            reasons,
            required_controls: vec![
                "attach reproducible evidence".into(),
                "resolve deployment identity and freshness failures".into(),
            ],
            metta_predicates: vec!["must-abstain".into()],
        }
    }
}

pub fn evaluate(input: &PolicyInput) -> PolicyDecision {
    let mut reasons = Vec::new();
    let mut controls = vec!["record policy decision in the Case File".into()];
    let mut predicates = vec!["evidence-complete".into()];

    if !input.has_asset_identity {
        reasons.push("asset identity is missing".into());
    }
    if !input.has_source_snapshot {
        reasons.push("source commit or compiler snapshot is missing".into());
    }
    if !input.has_deployment_snapshot {
        reasons.push("deployment snapshot is missing".into());
    }
    if input.evidence_count == 0 {
        reasons.push("no evidence is attached".into());
    }
    if !input.evidence_is_reproducible {
        reasons.push("evidence is not marked reproducible".into());
    }
    if input.stale_rpc {
        reasons.push("RPC or indexed state is stale".into());
        predicates.push("stale-rpc".into());
    }
    if input.simulation_timed_out {
        reasons.push("required simulation timed out".into());
        predicates.push("simulation-timeout".into());
    }
    if input.deployment_drift_detected {
        reasons.push("deployment drift requires targeted re-review".into());
        predicates.push("scope-drift".into());
    }

    if !reasons.is_empty() {
        return PolicyDecision::abstain(reasons);
    }

    if input.authority_changed {
        controls.push("verify signer, role, multisig, and approval topology".into());
        controls.push("obtain explicit human or multisig approval".into());
        predicates.push("authority-change".into());
    }

    let high_impact = matches!(
        input.severity.to_ascii_lowercase().as_str(),
        "high" | "critical"
    );
    if high_impact || input.authority_changed {
        controls.push("run a local-fork or transaction simulation".into());
        controls.push("verify expected post-state and dependent paths".into());
        predicates.push("requires-human-review".into());
        return PolicyDecision {
            decision: Decision::HumanReviewRequired,
            reasons: if high_impact {
                vec!["high-impact case requires approval quorum".into()]
            } else {
                vec!["authority topology changed".into()]
            },
            required_controls: controls,
            metta_predicates: predicates,
        };
    }

    if input.requested_action.is_some() && input.action_is_reversible {
        controls.push("enforce allowlist, expiry, blast-radius ceiling, and audit event".into());
        predicates.push("allowed-action".into());
        return PolicyDecision {
            decision: Decision::AllowedStagedAction,
            reasons: vec!["evidence is complete and requested action is reversible".into()],
            required_controls: controls,
            metta_predicates: predicates,
        };
    }

    PolicyDecision {
        decision: Decision::Reviewable,
        reasons: vec!["evidence is complete and no high-impact action was requested".into()],
        required_controls: controls,
        metta_predicates: predicates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn complete() -> PolicyInput {
        PolicyInput {
            has_asset_identity: true,
            has_source_snapshot: true,
            has_deployment_snapshot: true,
            evidence_count: 1,
            evidence_is_reproducible: true,
            severity: "low".into(),
            ..Default::default()
        }
    }
    #[test]
    fn incomplete_evidence_abstains() {
        assert_eq!(
            evaluate(&PolicyInput::default()).decision,
            Decision::Abstain
        );
    }
    #[test]
    fn drift_abstains() {
        let mut i = complete();
        i.deployment_drift_detected = true;
        assert_eq!(evaluate(&i).decision, Decision::Abstain);
    }
    #[test]
    fn critical_requires_human() {
        let mut i = complete();
        i.severity = "critical".into();
        assert_eq!(evaluate(&i).decision, Decision::HumanReviewRequired);
    }
    #[test]
    fn reversible_action_is_staged_only() {
        let mut i = complete();
        i.requested_action = Some("open_review_queue".into());
        i.action_is_reversible = true;
        assert_eq!(evaluate(&i).decision, Decision::AllowedStagedAction);
    }
}
