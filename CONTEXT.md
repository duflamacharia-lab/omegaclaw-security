# OmegaClaw project context

## Product

OmegaClaw is a defensive Web3 security evidence and control plane. It turns code findings, test results, deployment metadata, authority changes, and runtime observations into auditable Case Files.

## Non-negotiable boundaries

OmegaClaw is read-only by default. It must not hold production private keys, place live trades, broadcast transactions, scan unauthorized targets, or silently modify production systems. High-impact decisions require human review. CTF work runs against pinned local fixtures only.

## Architecture vocabulary

- **Case File:** persistent record of an investigation, evidence, policy decision, and audit events.
- **Asset identity:** chain, address, source revision, compiler, block snapshot, and authority summary.
- **Evidence:** a provenance-labeled observation with confidence and reproducibility metadata.
- **Policy decision:** Rust typed decision enriched by in-process Hyperon execution of MeTTa rules.
- **Worker:** isolated Slither, Foundry, Echidna, or safe repository check.
- **Read-only fixture:** synthetic or frozen external data used without live side effects.
- **Evaluation-only corpus:** material that must remain outside model training to prevent leakage.

## Engineering loop

Every change should follow a small vertical slice: state the intended behavior, add or update a test, implement the smallest coherent module, run Rust tests and frontend build, run `git diff --check`, then perform a security review of permissions, network access, provenance, and failure behavior.

Agent outputs should lead with the next action, use numbered steps for multi-step work, state current status, suppress tangents, and end with one concrete next step. This improves senior-review handoffs without changing runtime behavior.

## Model boundary

OpenMythos is the default optional Hugging Face security model. LLM output is untrusted hypothesis generation. Rust, Hyperon/MeTTa, deterministic workers, evidence provenance, and human approval remain authoritative.
