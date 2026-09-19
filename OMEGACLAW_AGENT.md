# OmegaClaw Agent Plugin

`omegaclaw-agent` is a defensive cybersecurity developer plugin for Web3 repositories. It is designed to run locally or in CI with a strict workspace boundary, explicit provider selection, bounded commands, structured decisions, and hash-chained-ready audit events.

## Commands

```bash
cargo run --bin omegaclaw-agent -- --workspace . inspect
cargo run --bin omegaclaw-agent -- --workspace . analyze contracts/Vault.sol
OMEGACLAW_ALLOW_COMMANDS=true cargo run --bin omegaclaw-agent -- --workspace . check rust-test
cargo run --bin omegaclaw-agent -- --workspace . audit
cargo run --bin omegaclaw-agent -- --workspace . validate-knowledge
```

Supported checks are deliberately allowlisted: `rust-test`, `rust-format`, and `frontend-build`. Arbitrary shell commands are rejected. Command execution is disabled unless `OMEGACLAW_ALLOW_COMMANDS=true` is explicitly set in a trusted workspace.

## Provider modes

`OMEGACLAW_PROVIDER=offline` is deterministic and requires no credentials. It only emits conservative pattern evidence and always requires human review.

`OMEGACLAW_PROVIDER=gemini` uses the official Gemini Interactions API from the Rust backend. It sends structured-output schemas and extracts typed agent decisions. The key is read only from `GEMINI_API_KEY`; it is never exposed to the browser or written to audit output. The current default model is `gemini-3.8-flash`, configurable with `GEMINI_MODEL`.

`OMEGACLAW_PROVIDER=dify` calls a Dify blocking workflow through `DIFY_API_URL`, `DIFY_API_KEY`, and `DIFY_WORKFLOW_ID`.

`OMEGACLAW_PROVIDER=huggingface` calls a Hugging Face OpenAI-compatible chat endpoint using `HUGGINGFACE_API_TOKEN`, `HUGGINGFACE_MODEL`, and optionally `HUGGINGFACE_API_URL`.

## Production safety contract

The agent is an analysis and evidence system, not a live exploit runner. It does not:

- request, store, or use production private keys;
- connect to live fund-moving or upgrade-authority methods;
- execute arbitrary shell commands;
- claim that a code pattern proves exploitability;
- merge, deploy, pause, upgrade, rotate signers, or move funds;
- treat model output as a final security judgment.

Provider output is structured but still semantically untrusted. OmegaClaw records provider/model metadata and input/output hashes, preserves human-review requirements, and requires further verification before a Case File can be treated as actionable.

## Knowledge prerequisites

Every `inspect`, `analyze`, and safe-check operation validates `data/kazamadono/` first. The validator checks that the catalog, defensive-candidate manifest, and video-link manifest share the same catalog hash. It verifies every retrieved transcript file against its recorded SHA-256 digest and refuses transcript content marked as admitted before review. The prerequisite result is metadata/quarantine-only; it never grants a tool, target, credential, or execution capability.

Set `OMEGACLAW_KNOWLEDGE_ROOT` only when using a separately verified snapshot. A missing, malformed, mismatched, or tampered knowledge snapshot fails closed.

## Gemini credential status

The current development session did **not** contain a `GEMINI_API_KEY`, so live Gemini connectivity has not been exercised here. The client is implemented against the current official Interactions API contract. Add a restricted server-side auth key locally or through the deployment secret manager; never commit it to Git.
