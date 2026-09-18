# OmegaClaw Training and Benchmark Plan

## Executive view

OmegaClaw should not ingest all publicly available cybersecurity data into one undifferentiated training set. The safer and technically stronger design is a **three-tier corpus**:

1. **Training data:** curated, license-reviewed secure-code and security-reasoning examples.
2. **Evaluation data:** pinned CTFs and held-out challenge instances whose solutions are excluded from training.
3. **Read-only architecture benchmarks:** Polymarket and L3 DeFi documentation, APIs, contracts, audits, and synthetic event fixtures used to test provenance, authorization, prompt-injection resistance, and abstention.

Damn Vulnerable DeFi and Ethernaut are excellent benchmark sources, but they should primarily evaluate OmegaClaw. Their intentionally vulnerable contracts and public solutions create leakage and dual-use risks if copied into ordinary training data.

## Proposed corpus

| Corpus | Role | Initial scope | Main gate |
| --- | --- | --- | --- |
| Damn Vulnerable DeFi | CTF benchmark and defensive remediation evaluation | Pin official v4.1.0 or a reviewed commit; use challenge source, tests, and vulnerability categories | MIT and dependency review; local EVM only; exclude public solutions from training |
| Ethernaut | CTF benchmark and invariant/review evaluation | Pin an official OpenZeppelin commit; create source-hash and category manifests | AGPL-3.0 review; exclude community solutions and live deployment paths |
| CyberNative/Code_Vulnerability_Security_DPO | Candidate secure-code preference training | Start with a reviewed stratified subset | Validate labels, compilation/static properties, provenance, and Apache-2.0 metadata |
| XuanwuAI/SecEval | Held-out cybersecurity knowledge benchmark | Do not train on the benchmark initially | CC BY-NC-SA 4.0; GPT-generated content; preserve evaluation separation |
| Rowden/CybersecurityQAA | Defensive answer-quality benchmark | Use for evidence-grounding and uncertainty evaluation | MIT card metadata; verify upstream/source rights and SME scope |
| Polymarket official documentation and agents repository | Read-only agent-security benchmark | REST/WebSocket schemas, SDK auth boundaries, sanitized code, mocked events | No wallet keys, no order submission, no live trading, current terms review |
| Arbitrum Orbit and other L3 documentation | Architecture-risk benchmark | Settlement, sequencing, DA, bridge, validator, upgrade authority, custom fee-token fixtures | Pin docs/code; distinguish architecture facts from deployment-specific claims |

## Evaluation design

Each benchmark case should contain a provenance manifest, an asset identity, evidence records, an expected defensive finding, a safe remediation or invariant, and a policy label. The benchmark must score more than vulnerability identification. It should measure root-cause accuracy, evidence citation, remediation quality, uncertainty calibration, refusal of unsafe live-target requests, and correct escalation to human review.

For DVD and Ethernaut, each challenge should run only inside a disposable local EVM or container. The harness must deny network egress, use generated accounts with no value, cap CPU and memory, and never expose production credentials. Public flags, writeups, and exploit payloads should be stored separately from the model-facing benchmark prompts and excluded from training splits.

## Polymarket benchmark boundary

Polymarket should be treated as a read-only, data-integrity and authorization benchmark rather than a trading integration. OmegaClaw can learn to validate market schemas, replay WebSocket events, detect stale or conflicting metadata, protect secrets, resist prompt injection in market/news text, and enforce dry-run behavior. It must not receive private keys or submit live orders. Any execution-path test must use mocked endpoints and locally generated keys.

## L3 DeFi benchmark boundary

The L3 corpus should cover settlement relationships, bridge and message assumptions, sequencer behavior, data availability, validator/watchtower roles, upgrade authority, custom fee tokens, and parent-chain dependencies. Initial cases should be synthetic and documentation-grounded. No live chain scanning, transaction broadcasting, or exploit generation is required for the first version.

## MCP/tooling plan

The current OmegaClaw repository does not have a verified native MCP implementation or official MCP-server catalog. Do not assume adjacent OpenClaw or Hugging Face documentation proves Omega compatibility.

The first MCP fixture should be a small read-only server suite with pinned schemas:

- Repository/file reader with workspace containment.
- Hugging Face metadata and dataset-card reader.
- Local benchmark manifest reader.
- Static-analysis result reader for Slither, Foundry, and Echidna artifacts.
- Mock Polymarket market-data reader with replayed fixtures.

Every MCP tool must pass schema validation, authorization checks, timeout and output limits, provenance tagging, prompt-injection tests, and audit logging. Network-capable MCP servers should remain disabled by default. Tokens must be audience-bound and never forwarded through an untrusted intermediary.

## Hugging Face model and dataset policy

OpenMythos is the preferred cybersecurity-focused model for OmegaClaw’s optional Hugging Face provider. Hosted inference may be free or rate-limited depending on account and endpoint conditions; open weights are not equivalent to free self-hosted inference. The Rust provider therefore supports OpenMythos as the default and a configurable Qwen code-model fallback.

The first dataset intake should not download every public repository. It should create immutable manifests containing repository ID, revision, file hashes, dataset-card license, source URLs, split assignment, and review status. Executable content, tool-invoking instructions, secrets, and untrusted prompt-like text must be quarantined before any model-facing use.

## Intake gates

Before admitting an artifact, OmegaClaw should require:

1. A pinned revision or immutable dataset snapshot.
2. A license and attribution record.
3. A provenance URL and retrieval timestamp.
4. A hash manifest.
5. A malware, secret, and prompt-injection scan.
6. A static validation or compilation result where code is present.
7. A train/evaluation split assignment.
8. A safety classification describing whether the artifact contains operational exploit material.
9. A removal and review contact where redistribution rights are uncertain.

Unresolved licensing should result in **evaluation-only quarantine**, not training ingestion.

## MCP servers required for the first implementation

The minimum useful tool set is:

- **Hugging Face connector:** model, dataset, dataset-card, and Space metadata; hosted inference when available.
- **Browser/source reader:** official project documentation and dynamic source validation.
- **Local Rust tools:** manifest hashing, license scanning, JSONL normalization, and benchmark execution.
- **Security workers:** isolated Slither, Foundry, and Echidna runners already present in OmegaClaw.
- **Future read-only MCP adapter:** expose the above capabilities behind one OmegaClaw allowlist after the adapter is version-pinned and tested.

Telegram should be added later as a notification and human-approval channel, not as an autonomous command channel. Incoming Telegram content must be treated as untrusted input, and high-impact tool execution must never be triggered directly by a chat message.

## Recommended implementation order

1. Add corpus and artifact manifest schemas to SQLite.
2. Pin DVD and Ethernaut benchmark revisions and create local-only runners.
3. Add Hugging Face dataset-card metadata ingestion and quarantine states.
4. Add CyberNative as a reviewed training candidate and keep SecEval/QAA held out.
5. Add Polymarket and L3 synthetic read-only fixtures.
6. Add MCP fixture servers and schema/provenance tests.
7. Add benchmark scoring, regression reports, and MeTTa policy parity checks.
8. Add Telegram notifications only after the approval and audit paths are complete.

## References

[1]: https://github.com/theredguild/damn-vulnerable-defi "Damn Vulnerable DeFi official repository"
[2]: https://github.com/OpenZeppelin/ethernaut "Ethernaut official repository"
[3]: https://huggingface.co/datasets/CyberNative/Code_Vulnerability_Security_DPO "CyberNative Code Vulnerability Security DPO dataset"
[4]: https://huggingface.co/datasets/XuanwuAI/SecEval "SecEval cybersecurity benchmark"
[5]: https://huggingface.co/datasets/Rowden/CybersecurityQAA "CybersecurityQAA dataset"
[6]: https://docs.polymarket.com/ "Polymarket official documentation"
[7]: https://github.com/Polymarket/agents "Polymarket agents repository"
[8]: https://docs.arbitrum.io/launch-orbit-chain/a-gentle-introduction "Arbitrum Orbit official documentation"
[9]: https://modelcontextprotocol.io/specification/2025-06-18/basic/security_best_practices "Model Context Protocol security best practices"
[10]: https://huggingface.co/docs/hub/en/agents-mcp "Hugging Face MCP documentation"
[11]: https://github.com/asi-alliance/OmegaClaw-Core "OmegaClaw-Core official repository"
