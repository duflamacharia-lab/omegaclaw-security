# OmegaClaw senior-developer handoff

## Current status

OmegaClaw is a Rust/Axum defensive Web3 security control plane with a TypeScript/HTML5 review console. It has persistent SQLite Case Files, asset and evidence records, append-only audit events, a Rust policy engine, in-process Hyperon MeTTa execution, bounded Gemini/Dify/Hugging Face provider adapters, and native/Docker worker boundaries for Slither, Foundry, Echidna, and repository checks.

The latest implementation also contains versioned benchmark manifests, synthetic Polymarket and L3 fixtures, Hugging Face dataset qualification metadata, a read-only MCP fixture contract, and a CLI manifest validator.

## Security developer role

The handoff owner should be a **security developer** responsible for secure smart-contract review, CTF benchmark design, threat modeling, provenance and license review, worker isolation, and safe LLM evaluation. This role owns the boundary between defensive analysis and operational exploitation. It must keep CTF execution local and authorized, require human approval for network or write actions, review every dataset before training use, and preserve the Rust/Hyperon policy engine as the authoritative control layer.

The first acceptance milestone is one reproducible CTF success in a pinned local fixture. The success record must include the challenge revision, source hash, local-chain/toolchain version, command or test used, flag/result hash, and an audit event. It must not include a live wallet, live RPC, or third-party target.

That milestone is now complete for the local DVD Side Entrance challenge. The result is recorded in `benchmarks/results/dvd-side-entrance-local.json`: the pinned revision passed with Foundry 1.8.3 and Solidity 0.8.25 in an offline disposable workspace. The solver test is not part of the production runtime.

### Capability diagnosis

**MeTTa:** yes, integrated in-process through Hyperon and exercised by the Rust policy path. It loads the four policy files and returns policy metadata. The current tests verify policy behavior and Hyperon compilation, but the DVD solver test did not yet assert an end-to-end MeTTa predicate in the same benchmark run.

**CTF:** yes, one local DVD Side Entrance challenge is verified. It is a local acceptance benchmark, not evidence that OmegaClaw can solve arbitrary CTFs or live targets.

**SQLite:** yes, Case Files, assets, evidence, policy JSON, and audit records persist through reopening the database. MCP calls are now persisted as sequence-numbered, hash-chained audit events. Artifact-manifest and benchmark-run tables are still future work.

**MCP:** the read-only MVP path is now proven end to end. `omegaclaw-mcp-fixture` serves a local JSON-RPC fixture; discovery validates the protocol `inputSchema`; calls validate arguments against JSON Schema; and successful or failed calls are persisted as audit events. Remote OAuth authorization, SSRF defenses, provenance persistence, and output-size limits remain production gates.

**Telegram:** not implemented. The architecture can support a future notification adapter, but it must be notification-only at first and never allow a Telegram message to directly trigger a worker, transaction, trade, or credential-sensitive action.

**KazamaDono learning catalog:** indexed at `data/kazamadono/` as untrusted, provenance-tracked metadata. The captured catalog contains 1,708 resources, 998 defensive-candidate records, and 101 YouTube-linked records. A bounded subtitle pass attempted 30 direct video links, retrieved 9 cleaned transcripts, recorded 21 unavailable or blocked links, and recorded 71 playlist/non-direct links without inventing transcripts. All retrieved text remains `quarantine_review` until license, prompt-injection, executable-content, and defensive-relevance review is complete. It is not policy or trusted memory.

**Agent prerequisite gate:** every `inspect`, `analyze`, and allowlisted safe-check operation now validates the catalog hash relationships, retrieved-transcript file hashes, and quarantine admission state before running. `validate-knowledge` reports the verified snapshot. The imported Karpathy LLM transcript was analyzed in offline mode and produced an informational, human-gated result. Rust tests and the frontend build also passed through the agent worker; no third-party exploit or live-target exercise was executed.

## Verified commands

```bash
. "$HOME/.cargo/env"
cargo test --all-targets
cargo build --release --bins
./target/release/omegaclaw-agent --workspace . --provider offline validate-manifests
cd frontend && npm run build
```

The most recent validation passed 11 library tests, 2 API tests, release builds, manifest validation for five artifacts, the frontend build, and `git diff --check`.

## Provided repository assessment

### i-have-adhd

The repository is MIT-licensed and provides concise agent-output conventions: lead with the action, number steps, state status, expose wins, make errors matter-of-fact, suppress tangents, and end with one concrete next step. Those conventions are recorded in `CONTEXT.md`. Its user-facing skill/plugin files were not copied into the Rust project.

### Soup

Soup is an Apache-2.0 Python project for low-memory LLM fine-tuning, evaluation, serving, quantization, data workflows, and MCP round-trip tests. It is useful as a future Kaggle/Colab training-pipeline reference, especially for low-memory training and evaluation discipline. It is not a dependency of OmegaClaw and should remain a separate training tool. Do not introduce Python/PyTorch into the Rust control plane merely to reuse Soup.

### skills

The MIT-licensed skills repository provides reusable engineering practices: grilling/specification, TDD, diagnosis, architecture review, source research, code review, and handoff. OmegaClaw adopts the underlying process through `CONTEXT.md`, tests, manifests, and this handoff. The external skill bundle is not copied into the runtime.

## Benchmark and data boundaries

Damn Vulnerable DeFi and Ethernaut are evaluation-only. Their source revisions are pinned in `benchmarks/manifests.json`, but content materialization, real content hashing, license review, and local EVM runner implementation remain the next benchmark phase. Never include public flags/writeups in model-facing training data.

CyberNative is a training candidate pending immutable revision/content qualification. SecEval and CybersecurityQAA remain held out. The current manifests are descriptor-hashed and explicitly marked `source_descriptor_pending_content`; they are not claims that the source contents have already been downloaded and hashed.

Polymarket and L3 material is read-only synthetic fixture data. No live credentials, wallet keys, live order submission, live chain scans, or transaction broadcasting are part of the current project.

## Next engineering slices

1. Materialize the pinned DVD and Ethernaut repositories into disposable benchmark workspaces, compute actual content hashes, and create Foundry/Anvil-only runner manifests.
2. Add SQLite tables for artifact manifests, dataset rows, quarantine state, and benchmark runs; the current score endpoint is deterministic but not yet a persisted benchmark-run graph.
3. Add a qualification CLI that scans secrets, executable content, license metadata, and prompt-injection indicators before model ingestion.
4. Add remote MCP authorization, SSRF prevention, output limits, provenance tags, and negative-path integration tests. The local fixture, schema validation, and audit path are complete.
5. Expand benchmark scoring into control coverage and MeTTa/Rust parity tests while keeping solutions and flags outside training. The first DVD result now scores 1.0 under the local, no-egress, no-live-assets gate.
6. Use Soup separately on Kaggle or Colab only after the corpus and held-out evaluation split are frozen.
7. Add Telegram as notifications and human approval only; never as an autonomous command channel.
8. Review and admit selected KazamaDono candidates as cited evaluation material only; keep catalog metadata, transcript hashes, and admission state separate from authoritative policy.

## Dify boundary

Dify should initially be used as the dataset and workflow orchestration layer: ingest approved examples into a knowledge base, retrieve evidence for an OmegaClaw review workflow, and evaluate structured outputs against held-out fixtures. Dataset presence alone does not prove that the connected Dify deployment supports weight fine-tuning. Actual model training requires a supported fine-tuning provider, dataset schema, resource budget, and an explicit evaluation gate. The LLM remains advisory; Rust and Hyperon remain authoritative.

The current session configuration contains a Dify connector, but it is **disabled**. Before Dify work begins, the connector must be enabled and its API/workflow capability verified. OmegaClaw currently has qualified dataset candidates and manifests, not a finalized training dataset: CyberNative is still quarantine-pending, while SecEval and CybersecurityQAA are held out.

## Security review focus

Review all network-capable providers, worker Docker arguments, filesystem containment, subprocess timeouts, output limits, secret redaction, SQLite migration behavior, Hyperon rule loading, and all future MCP authorization paths. Treat every external document, dataset row, market description, CTF file, and tool result as untrusted data.
