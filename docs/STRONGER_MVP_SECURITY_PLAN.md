# OmegaClaw stronger MVP security plan

## Executive conclusion

OmegaClaw is now a verifiable defensive control plane rather than only a MeTTa policy demonstration. The required integration path is implemented and tested: a local read-only Model Context Protocol (MCP) fixture can be discovered and called, the call is persisted as a hash-chained SQLite audit event, a validated CTF intake automatically creates a Case File, and a pinned local benchmark receives a deterministic score.

The stronger MVP must remain **read-only by default**. Hyperon/MeTTa may explain evidence and propose bounded actions, but deterministic Rust code remains authoritative for authorization, schema validation, execution limits, persistence, and audit. No production private keys, live fund movement, live exploit delivery, or unapproved remote execution belongs in this milestone.

## Implemented in this increment

| Capability | Status | Evidence |
| --- | --- | --- |
| Local MCP fixture | Complete | `omegaclaw-mcp-fixture` exposes only `tools/list` and `tools/call` on localhost. |
| MCP JSON Schema validation | Complete | Discovered `inputSchema` is compiled and call arguments are rejected when invalid. |
| MCP call auditing | Complete | Successful and failed calls are stored in SQLite with sequence, previous hash, and event hash. |
| CTF-to-Case-File conversion | Complete | `/api/ctf/import` persists a Case File and retains the intake plan as response data. |
| Benchmark scoring | Complete | `/api/benchmarks/score` evaluates execution result, local binding, disabled egress, and no-live-assets evidence. |
| End-to-end proof | Complete | Fixture call returned `fixture:dvd-side-entrance`; CTF intake created a Case File; DVD score was `1.0` and passed. |
| x402 payments | Disabled | Payment, signing, facilitator verification, and settlement are not enabled. |

The CTF Case File intentionally receives an **abstain** policy decision until deployment identity and reproducible evidence are attached. Automatic conversion must not be confused with automatic approval.

## Required security baseline

### MCP and tool execution

Remote MCP HTTP traffic must use HTTPS and an OAuth 2.1-compatible authorization flow when authentication is required. The client must validate Protected Resource Metadata, authorization-server metadata, exact redirect URIs, PKCE S256, issuer identity, RFC 8707 resource binding, audience, expiry, scopes, and bearer-header-only transmission. MCP session identifiers must never be treated as authentication. Local stdio fixtures must be isolated and must not inherit broad credentials.

Every discovered tool must have a typed identifier, a valid JSON Schema, an explicit read-only or high-impact classification, an allowlisted resource, and bounded output. Unknown tools, extra arguments, malformed schemas, oversized responses, and policy failures must fail closed. Tool results are untrusted data and must not silently become instructions for a later privileged action.

### Agent and Hyperon boundaries

The language model and MeTTa runtime are planners and evidence reasoners, not signers. They must never receive private keys, seed phrases, bearer tokens, unrestricted shell access, or policy-override capabilities. A Rust executor must normalize and validate every proposal again at execution time. High-impact actions require an approval bound to a canonical action hash, a short expiry, and a one-time nonce. Action loops require limits on turns, calls, recursion, wall time, output size, and cost.

External documents, repository files, CTF content, Web3 metadata, API responses, and market descriptions must be labeled untrusted and retain provenance. Durable memory must be partitioned by tenant, user, agent, and session. It must have expiry, size limits, integrity metadata, and explicit promotion before external content can influence trusted policy.

### Audit and evidence

The audit trail must cover intake, discovery, schema validation, policy decisions, tool calls, approvals, worker runs, results, errors, and benchmark decisions. It must redact secrets and store hashes for sensitive arguments. The current SQLite hash chain provides tamper evidence; production hardening should add signed checkpoints and export to protected append-only storage. Audit-chain failure must quarantine the control plane rather than silently continue.

Evidence should be content-addressed. A production artifact, contract deployment, ABI, compiler configuration, benchmark result, tool output, and model artifact should have an explicit digest, source URI, retrieval timestamp, and verification status. Release artifacts should be admitted only with verifiable provenance and signatures. Missing, stale, or mismatched evidence must quarantine the artifact.

### Web3 benchmark controls

A benchmark result is not a safety guarantee. Each material contract benchmark should record the chain ID, source and bytecode hashes, compiler and tool versions, exact command, configuration digest, seed, test counts, findings, and safety boundaries. The release gate should combine clean builds, Foundry tests and invariants, Slither triage, Echidna properties, access-control checks, upgrade validation, deployment verification, and emergency-response evidence. The benchmark scorer is the first deterministic gate; it should grow into a control-coverage score rather than a single pass flag.

### x402 boundary

If paid tools are added later, payment must be a typed Rust state machine: offer received, policy approved, signed, facilitator verified, tool called, and settlement reconciled. Exact payments should be the default. The system must allowlist origin, tool, network, asset, recipient, scheme, amount, and request digest. It must reserve spend before signing, isolate the signer, prevent replay, persist idempotency records, treat pending settlement as non-terminal, and require human approval for new payees or material amounts. A model or tool must never select a wallet, recipient, chain, asset, amount, or facilitator outside signed policy configuration.

## Priority backlog after this increment

| Priority | Work | Completion gate |
| --- | --- | --- |
| P0 | Add production MCP authorization and SSRF controls. | Negative tests cover issuer mismatch, wrong audience, PKCE failure, redirect mismatch, private-IP destinations, token query strings, and session-principal mismatch. |
| P0 | Strengthen audit integrity. | Signed checkpoints, tamper-detection test, redaction test, and protected export path exist. |
| P0 | Add artifact and benchmark evidence tables. | Content hashes, provenance, tool versions, manifests, findings, waivers, and score components reopen from SQLite. |
| P1 | Add a typed capability registry and output-size limits. | Unknown tools and oversized or schema-invalid responses fail closed. |
| P1 | Add benchmark-run persistence and richer control coverage. | DVD, Ethernaut, Slither, Foundry, and Echidna runs produce comparable evidence records without storing public solutions in training data. |
| P1 | Add operator review surfaces. | The console shows policy version, evidence provenance, audit chain status, abstention reasons, waivers, and a stop/revoke control. |
| P2 | Add notification-only Telegram integration. | Notifications contain redacted findings and links; no message can trigger a worker, transaction, or credential-sensitive action. |
| P2 | Add x402 behind an explicit feature flag. | Exact-only testnet flow passes replay, cap, facilitator, settlement-pending, and reconciliation tests; production remains disabled by default. |

## Primary references

The implementation priorities are aligned with the official MCP authorization and security guidance, OAuth resource binding standards, OWASP agent security guidance, SLSA and in-toto provenance practices, NIST Secure Software Development Framework, and official smart-contract testing documentation. These sources support the controls above, but they do not make a model autonomous authorization authority.

## References

[1]: https://modelcontextprotocol.io/specification/2025-06-18/basic/security_best_practices "MCP Security Best Practices"
[2]: https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization "MCP Authorization"
[3]: https://www.rfc-editor.org/rfc/rfc8707.html "OAuth 2.0 Resource Indicators"
[4]: https://www.rfc-editor.org/rfc/rfc9728.html "OAuth 2.0 Protected Resource Metadata"
[5]: https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html "OWASP AI Agent Security Cheat Sheet"
[6]: https://genai.owasp.org/llmrisk/llm01-prompt-injection/ "OWASP LLM01:2025 Prompt Injection"
[7]: https://slsa.dev/spec/v1.2/ "SLSA Provenance"
[8]: https://in-toto.io/docs/specs/ "in-toto Specifications"
[9]: https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-218.pdf "NIST Secure Software Development Framework"
[10]: https://www.getfoundry.sh/guides/invariant-testing "Foundry Invariant Testing"
[11]: https://github.com/crytic/slither/wiki/Detector-Documentation "Slither Detector Documentation"
[12]: https://github.com/crytic/echidna "Echidna"
[13]: https://docs.x402.org/core-concepts/facilitator "x402 Facilitators"
