# OmegaClaw Security

**OmegaClaw** is a defensive, protocol-aware Web3 security evidence and control plane. It turns heterogeneous findings, tests, audit reports, runtime alerts, authority events, and deployment metadata into traceable **Case Files**.

> MVP posture: read-only by default, human review for high-impact cases, no production keys, no autonomous live exploit activity, and no silent production changes.

## What is implemented

- Rust/Axum API with an in-memory asset and Case File store.
- Versioned asset identity fields: chain, address, source commit, block snapshot, and authority summary.
- Evidence provenance and separate finding/impact confidence fields.
- Explicit MVP policy decisions: `abstain`, `reviewable`, or `human_review_required`.
- Inspectable MeTTa rules for evidence completeness, scope drift, authority review, and reversible actions.
- HTML5/TypeScript review console with active investigations, evidence trail, confidence separation, and safe next steps.
- Server-side adapter boundaries for Dify, Gemini, and Hugging Face. Credentials are read from environment variables only; no provider calls are enabled until request logging, redaction, tenant budgets, retries, and approval controls are added.

## Architecture

```text
HTML5 + TypeScript console
          |
       Rust API  ----  Case Files / Asset Graph (MVP in-memory)
          |
   policy boundary  ----  MeTTa rules (inspectable source)
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
- **Gemini** is an optional reasoning provider for bounded hypothesis generation. Configure `GEMINI_API_KEY` and optionally `GEMINI_MODEL`.
- **Hugging Face** is an optional inference provider for self-selected open models. Configure `HUGGINGFACE_API_TOKEN` and optionally `HUGGINGFACE_MODEL`.

The current MVP only reports whether each adapter is configured. Before enabling network inference, add tenant isolation, redaction, request/response provenance, rate limits, retry policy, model pinning, prompt-injection defenses, and evaluation against the OmegaClaw case ledger.

## MeTTa

`metta/omegaclaw_rules.metta` is the initial inspectable policy layer. It intentionally keeps rules conservative. The Rust module mirrors the first decisions so the project can run without requiring a MeTTa runtime in every developer environment. The next integration should pin a supported MeTTa runtime and compare Rust decisions against MeTTa decisions in CI.

## Security boundaries

OmegaClaw does not claim exhaustive vulnerability discovery, protocol safety, or autonomous remediation. The MVP does not hold production private keys, upgrade authority, treasury permissions, or unrestricted RPC mutation access. Synthetic and historical defensive fixtures must be used for local-fork validation.

## Roadmap

1. Persist Case Files and asset graph snapshots in PostgreSQL.
2. Add signed artifact manifests and isolated Slither/Foundry/Echidna workers.
3. Pin and execute MeTTa rules in CI with decision parity tests.
4. Add local-fork validation and deployment drift detection.
5. Add Dify/Gemini/Hugging Face inference behind explicit provider policies and evaluation gates.
6. Add Forta/OpenZeppelin-compatible runtime adapters and staging-only response simulation.
7. Add tenant-isolated benchmarks, temporal holdouts, mutation testing, and incident-to-regression feedback.

## License

Apache-2.0. See `LICENSE`.
