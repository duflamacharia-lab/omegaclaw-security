# KazamaDono knowledge admission

## Scope

OmegaClaw has indexed the public KazamaDono Anarchy catalog at [kazamadono.github.io](https://kazamadono.github.io/). The catalog contains 1,708 resource records and is useful as a discovery layer for security engineering, Web3, defensive operations, artificial intelligence, cryptography, forensics, cloud security, and CTF education.

The catalog is **not** a trusted training corpus. It is a set of external pointers selected by another party. A link, description, transcript, or course title can contain inaccurate claims, prompt injection, unsafe instructions, copyrighted material, or executable content.

## Current artifacts

`data/kazamadono/catalog.manifest.json` contains all 1,708 records with a SHA-256 hash of the captured `courses.json` source. `data/kazamadono/defensive-candidates.manifest.json` contains the 998 records tagged with at least one candidate defensive or engineering category. `data/kazamadono/video-links.manifest.json` contains the 101 YouTube-linked records. These manifests preserve the source URL, capture timestamp, tags, media type, and admission state.

The catalog itself is admitted as **metadata only**. Candidate resources are marked `candidate_review`. Resources that are primarily tagged `red`, `exploits`, `psyops`, `game`, or `mobile` without a defensive tag are marked `quarantine_review`. This classification is a triage signal, not a final safety judgment.

## Transcript policy

The transcript collector attempts public subtitle retrieval for direct YouTube video URLs without downloading video media. It records success, unavailable or blocked retrieval, playlist/non-direct links, subtitle hash, cleaned transcript hash, and the original URL. A retrieved transcript is marked `quarantine_review` until it passes all admission gates.

A transcript may become a reviewable evidence item only when the following are true:

1. The source URL, retrieval time, subtitle file, and hashes are preserved.
2. Licensing and redistribution terms permit the intended use.
3. The text is screened for prompt injection and instructions that attempt to change OmegaClaw policy, permissions, or tool behavior.
4. Executable code, exploit payloads, credentials, and live-target instructions are separated from explanatory material.
5. A reviewer confirms that the material improves defensive Web3 security reasoning.
6. The result is stored as sourced knowledge with uncertainty and provenance, not as a new policy rule.

No transcript may authorize a scan, exploit, transaction, order, credential access, or production change. Rust, SQLite provenance, deterministic workers, and Hyperon/MeTTa policy remain authoritative.

## Admission states

| State | Meaning | Agent behavior |
| --- | --- | --- |
| `metadata_only` | Catalog pointer has been indexed, but content has not been reviewed. | May use title, tag, and URL for discovery only. |
| `candidate_review` | Resource appears relevant to defensive engineering. | May be queued for human review; not trusted memory. |
| `quarantine_review` | Resource is dual-use, offensive, unverified, or has unresolved licensing or injection risk. | Must not be used to create capabilities or executable actions. |
| `admitted_evidence` | Content passed provenance, licensing, safety, and relevance review. | May support a cited Case File or evaluation example within scope. |
| `rejected` | Content failed review or cannot be safely and lawfully used. | Preserve rejection reason and do not retry automatically. |

## What improves the agent

The highest-value candidates for OmegaClaw are material on LLM application security, tool authorization, prompt-injection resistance, secure software supply chains, cryptography, blockchain protocol design, smart-contract testing, Web3 incident response, cloud and infrastructure defense, API security, log analysis, forensics, and CTF methodology confined to local fixtures. The site’s discovery metadata can help prioritize those domains, but primary documentation and reproducible local benchmarks remain preferred sources.

The resource hub should therefore improve OmegaClaw’s **retrieval and evaluation coverage**, not expand its autonomous authority. A future ingestion job should produce cited summaries, benchmark questions, and held-out evaluation examples while keeping public CTF solutions and flags outside model-facing training data.

## Reproduction

```bash
python3 scripts/ingest_kazama_catalog.py \
  data/kazamadono/courses.json \
  data/kazamadono

python3 scripts/collect_kazama_transcripts.py \
  data/kazamadono/video-links.manifest.json \
  data/kazamadono/transcripts
```

The transcript command records failures rather than treating inaccessible or blocked videos as missing evidence. It should be run only in an environment where public subtitle retrieval is permitted and should not be converted into a background sync without an explicit retention and licensing decision.

## References

[1]: https://kazamadono.github.io/ "KazamaDono Anarchy resource hub"
[2]: https://kazamadono.github.io/courses.json "KazamaDono public course catalog"
[3]: https://modelcontextprotocol.io/specification/2025-06-18/basic/security_best_practices "MCP Security Best Practices"
[4]: https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html "OWASP AI Agent Security Cheat Sheet"
