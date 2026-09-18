; OmegaClaw abstention and action boundary policy

(= (high-impact-case $case)
   (or (severity-high $case) (severity-critical $case)))

(= (reversible-action $action)
   (or (open-review-queue $action)
       (create-regression-test $action)
       (create-pull-request $action)
       (tighten-noncritical-alert $action)))

(= (forbidden-production-action $action)
   (or (move-funds $action)
       (rotate-signer $action)
       (upgrade-contract $action)
       (change-oracle $action)
       (pause-production $action)))

(= (requires-human-approval $case $action)
   (or (high-impact-case $case)
       (authority-drift $case)
       (forbidden-production-action $action)
       (not (simulation-complete $case))
       (not (post-state-verified $case))))

(= (allowed-staged-action $case $action)
   (and (evidence-complete $case)
        (reversible-action $action)
        (not (requires-human-approval $case $action))
        (blast-radius-within-policy $action)))

(= (abstain-and-escalate $case)
   (or (must-abstain $case)
       (not (evidence-complete $case))))
