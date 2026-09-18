# OmegaClaw senior-developer handoff

## Current status

OmegaClaw is a Rust/Axum defensive Web3 security control plane with a TypeScript/HTML5 review console. It has persistent SQLite Case Files, asset and evidence records, append-only audit events, a Rust policy engine, in-process Hyperon MeTTa execution, bounded Gemini/Dify/Hugging Face provider adapters, and native/Docker worker boundaries for Slither, Foundry, Echidna, and repository checks.

The latest implementation also contains versioned benchmark manifests, synthetic Polymarket and L3 fixtures, Hugging Face dataset qualification metadata, a read-only MCP fixture contract, and a CLI manifest validator.

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
2. Add SQLite tables for artifact manifests, dataset rows, quarantine state, and benchmark runs.
3. Add a qualification CLI that scans secrets, executable content, license metadata, and prompt-injection indicators before model ingestion.
4. Add read-only MCP fixture servers behind a feature flag with schema validation, workspace containment, provenance tags, and audit events.
5. Add benchmark scoring and MeTTa/Rust parity tests while keeping solutions and flags outside training.
6. Use Soup separately on Kaggle or Colab only after the corpus and held-out evaluation split are frozen.
7. Add Telegram as notifications and human approval only; never as an autonomous command channel.

## Security review focus

Review all network-capable providers, worker Docker arguments, filesystem containment, subprocess timeouts, output limits, secret redaction, SQLite migration behavior, Hyperon rule loading, and all future MCP authorization paths. Treat every external document, dataset row, market description, CTF file, and tool result as untrusted data.
