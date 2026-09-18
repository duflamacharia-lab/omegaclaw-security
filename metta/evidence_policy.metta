; OmegaClaw evidence policy
; Facts are supplied by the Rust control plane for one Case File snapshot.

(= (evidence-complete $case)
   (and (has-asset-identity $case)
        (has-source-snapshot $case)
        (has-deployment-snapshot $case)
        (has-reproducible-evidence $case)))

(= (evidence-missing $case)
   (or (not (has-asset-identity $case))
       (not (has-source-snapshot $case))
       (not (has-deployment-snapshot $case))
       (not (has-reproducible-evidence $case))))

(= (must-abstain $case)
   (or (evidence-missing $case)
       (stale-rpc $case)
       (simulation-timeout $case)
       (unknown-deployment-identity $case)))

(= (decision-for $case abstain)
   (must-abstain $case))

; A policy decision is not a proof of safety.
(= (analysis-worker-permission) read-only)
(= (production-signing-key-available) false)
