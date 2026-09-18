; OmegaClaw MeTTa policy rules
; These rules are intentionally inspectable and conservative. The Rust policy module
; currently mirrors the MVP decisions until a MeTTa runtime is selected and pinned.

(= (evidence-complete $case)
   (and (has-asset $case) (has-source-snapshot $case) (has-evidence $case)))

(= (requires-human-review $case)
   (or (high-impact $case)
       (authority-change $case)
       (scope-drift $case)
       (missing-post-state $case)))

(= (must-abstain $case)
   (or (not (evidence-complete $case))
       (stale-rpc $case)
       (simulation-timeout $case)
       (unknown-deployment-identity $case)))

(= (allowed-action $case)
   (and (evidence-complete $case)
        (not (requires-human-review $case))
        (reversible-action $case)
        (within-blast-radius $case)))

(= (post-action-contained $case)
   (and (finality-confirmed $case)
        (expected-state-observed $case)
        (dependent-paths-checked $case)))

; Core principle: analysis is not authority.
(= (analysis-worker-permission) read-only)
(= (production-key-available-to-analysis-worker) false)
