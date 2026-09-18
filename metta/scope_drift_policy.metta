; OmegaClaw scope and deployment drift policy

(= (scope-drift $case)
   (or (source-commit-changed $case)
       (compiler-config-changed $case)
       (dependency-lock-changed $case)
       (bytecode-changed $case)
       (proxy-implementation-changed $case)
       (deployment-address-changed $case)
       (chain-or-block-snapshot-changed $case)))

(= (authority-drift $case)
   (or (admin-role-changed $case)
       (signer-set-changed $case)
       (multisig-threshold-changed $case)
       (upgrade-authority-changed $case)
       (relayer-policy-changed $case)))

(= (runtime-control-drift $case)
   (or (oracle-config-changed $case)
       (bridge-route-changed $case)
       (monitor-rule-changed $case)
       (pause-state-changed $case)))

(= (targeted-rereview-required $case)
   (or (scope-drift $case)
       (authority-drift $case)
       (runtime-control-drift $case)))

(= (drift-action $case)
   (and (targeted-rereview-required $case)
        (not (automatic-production-action $case))))
