# OmegaClaw benchmark assets

This directory contains **versioned, read-only benchmark metadata and synthetic fixtures**. It does not contain downloaded CTF repositories, public solutions, wallet keys, or live API credentials.

## Validate manifests

From the repository root:

```bash
cargo run --bin omegaclaw-agent -- --workspace . --provider offline validate-manifests
```

The validator checks required fields, descriptor hashes, license metadata, and safety flags. A `source_descriptor_pending_content` hash scope means the source revision is pinned but the content has not yet been materialized and hashed. It must not be treated as a verified content digest.

## CTF policy

Damn Vulnerable DeFi and Ethernaut are evaluation-only. Their official revisions are pinned in `manifests.json`, but source material must be downloaded into a disposable workspace, hashed, license-reviewed, and executed only in a network-isolated local EVM. Public flags and solution writeups must remain outside model-facing prompts.

## Hugging Face policy

CyberNative is a training candidate pending immutable revision and content review. SecEval and CybersecurityQAA remain held-out evaluation candidates. No dataset is automatically downloaded or executed by OmegaClaw.

## Polymarket and L3 fixtures

`fixtures.json` contains synthetic read-only cases. Polymarket cases test provenance, untrusted text, and order dry-run denial. L3 cases test parent-chain finality, data-availability evidence, and upgrade-authority review. These fixtures do not contact live APIs or chains.

## MCP fixture contract

`mcp-and-datasets.json` defines the initial read-only MCP surface. Tools must be schema-validated, workspace-contained, provenance-tagged, time-limited, output-limited, and audited. Network or write-capable tools require explicit human approval and are not part of the baseline fixture.
