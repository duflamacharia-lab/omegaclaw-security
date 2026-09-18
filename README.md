# OmegaClaw Security

**OmegaClaw** is a defensive, protocol-aware Web3 security evidence and control plane. It turns heterogeneous findings, tests, audit reports, runtime alerts, authority events, and deployment metadata into traceable **Case Files**.

> MVP posture: read-only by default, human review for high-impact cases, no production keys, no autonomous live exploit activity, and no silent production changes.

## What is implemented

- Rust/Axum API with SQLite-backed asset and Case File storage.
- Versioned asset identity fields: chain, address, source commit, block snapshot, and authority summary.
- Evidence provenance and separate finding/impact confidence fields.
- Explicit MVP policy decisions: `abstain`, `reviewable`, or `human_review_required`.
- Inspectable MeTTa rules for evidence completeness, scope drift, authority review, and reversible actions.
- HTML5/TypeScript review console with active investigations, evidence trail, confidence separation, and safe next steps.
- Server-side adapter boundaries for Dify, Gemini, and Hugging Face. Credentials are read from environment variables only; no provider calls are enabled until request logging, redaction, tenant budgets, retries, and approval controls are added.
- A full `omegaclaw-agent` CLI plugin with repository inspection, defensive file analysis, an allowlisted check runner, structured decisions, and audit-event hashes.
- A real provider execution path for Gemini Interactions API, Dify blocking workflows, and Hugging Face OpenAI-compatible chat inference. Provider output is still untrusted and remains human-gated.
- SQLite-backed persistent Case Files, asset identities, evidence records, policy JSON, and append-only audit events.
- The official Hyperon Rust workspace pinned at commit `3f76dc460da6961f57f69f6c3e550c59c74ada83`; policy decisions execute the MeTTa rule files in-process and persist runtime metadata.
- Isolated Slither, Foundry, and Echidna worker jobs with native execution or Docker mode (`OMEGACLAW_WORKER_MODE=docker`); Docker mode uses a read-only workspace, dropped Linux capabilities, no network by default, and no-new-privileges.

## Architecture

```text
HTML5 + TypeScript console
          |
       Rust API  ----  Case Files / Asset Graph (SQLite)
          |
   Hyperon runtime  ----  MeTTa rules (inspectable source)
          |
 isolated analysis adapters: Slither / Foundry / Echidna / Dify / Gemini / Hugging Face
          |
 staging-only simulation and human approval (future)
```

The target production architecture separates four planes: **observe**, **reason**, **approve**, and **execute**. Analysis workers must not hold production signing keys. High-blast-radius actions require policy authorization, simulation, expiry, and human or multisig approval.

## Local development

Requirements: Rust stable, Node.js 20+, and npm.

```bash
# Build the console
cd frontend
npm install
npm run build
cd ..

# Run Rust tests and API
cargo test
cargo run
```

Open `http://localhost:8080`.

Run the developer agent:

```bash
cargo run --bin omegaclaw-agent -- --workspace . --provider offline inspect
cargo run --bin omegaclaw-agent -- --workspace . --provider offline analyze src/providers.rs
OMEGACLAW_ALLOW_COMMANDS=true cargo run --bin omegaclaw-agent -- --workspace . check rust-test
```

Use `--provider gemini`, `--provider dify`, or `--provider huggingface` only after configuring the corresponding server-side credentials. The agent never exposes provider keys to the frontend.

The API persists to `OMEGACLAW_DB_PATH` (default `omegaclaw.sqlite3`) and runs migrations at startup. The worker runner supports `slither`, `forge-test`, and `echidna` in addition to Rust/frontend checks. Native tools must already be installed; Docker mode uses the official security-tool images when Docker is available. If a worker is unavailable, OmegaClaw records an unavailable/error result rather than treating the check as passed. Policy responses report `metta_execution: "hyperon_in_process"` and the loaded rule-file set when Hyperon succeeds.

Useful endpoints:

- `GET /api/health`
- `GET /api/cases`
- `POST /api/cases`
- `GET /api/cases/:id/policy`
- `GET /api/providers`

Example Case File creation:

```bash
curl -X POST http://localhost:8080/api/cases \\
  -H 'content-type: application/json' \\
  -d '{
    "title":"Role changed outside approved manifest",
    "severity":"high",
    "asset":{"id":"vault-1","name":"Atlas Vault","chain":"anvil","address":"0x1111111111111111111111111111111111111111","source_commit":"abc123","block_snapshot":19842001,"authority_summary":"2-of-3 multisig"},
    "evidence":[{"kind":"role_event","source":"deployment-adapter","summary":"Unexpected admin role assignment","confidence":0.94}],
    "recommended_action":"Simulate staged pause and route to security owner"
  }'
```

## Provider configuration

Copy `.env.example` to a local environment file. Never commit credentials.

- **Dify** is the traditional workflow/ML orchestration boundary. Configure `DIFY_API_URL` and `DIFY_API_KEY`.
- **Gemini** is an optional reasoning provider for bounded hypothesis generation. The client uses the official Interactions API with structured JSON output. Configure `GEMINI_API_KEY`, optionally `GEMINI_MODEL`, and keep the key in a server-side secret manager.
- **Hugging Face** is an optional inference provider for self-selected open models. Configure `HUGGINGFACE_API_TOKEN`, optionally `HUGGINGFACE_MODEL`, and optionally `HUGGINGFACE_API_URL`.

Before enabling network inference in production, add tenant isolation, redaction, request/response provenance, rate limits, retry policy, model pinning, prompt-injection defenses, provider health checks, and evaluation against the OmegaClaw case ledger.

## MeTTa

The files under `metta/` are the inspectable policy layer. Hyperon is pinned in `Cargo.toml` and executes these rules in-process for each policy decision. The Rust policy engine remains the conservative typed decision layer and records Hyperon execution status, loaded files, and derived predicates for auditability.

## Security boundaries

OmegaClaw does not claim exhaustive vulnerability discovery, protocol safety, or autonomous remediation. The MVP does not hold production private keys, upgrade authority, treasury permissions, or unrestricted RPC mutation access. Synthetic and historical defensive fixtures must be used for local-fork validation.

## Roadmap

1. Add signed artifact manifests and worker-result ingestion.
2. Expand isolated Slither/Foundry/Echidna jobs with container availability checks.
3. Add MeTTa decision parity and rule mutation tests in CI.
4. Add local-fork validation and deployment drift detection.
5. Add Dify/Gemini/Hugging Face inference behind explicit provider policies and evaluation gates.
6. Add Forta/OpenZeppelin-compatible runtime adapters and staging-only response simulation.
7. Add tenant-isolated benchmarks, temporal holdouts, mutation testing, and incident-to-regression feedback.

## License

Apache-2.0. See `LICENSE`.
