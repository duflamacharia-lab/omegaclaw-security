# OmegaClaw agent skills

This directory is a **curated index**, not a copied third-party skill bundle. It records which practices future agents should apply and where the original repositories came from.

## Skills adopted

| Skill | Origin | Use in OmegaClaw |
|---|---|---|
| Concise action-first output | `dgithinjibit/i-have-adhd`, MIT, revision `6f1f982` | Lead with the next action, number steps, state progress, suppress tangents, and make errors concrete. |
| Test-driven development | `dgithinjibit/skills` repository, MIT, revision `6654f6b` | Test public seams, use red-green-refactor, and implement one vertical slice at a time. |
| Code review | `dgithinjibit/skills` repository, MIT, revision `6654f6b` | Review standards and specification separately; distinguish hard violations from design judgments. |
| Handoff | `dgithinjibit/skills` repository, MIT, revision `6654f6b` | Leave a continuation document with current state, risks, tests, and suggested skills. |
| Training/evaluation discipline | `dgithinjibit/Soup`, Apache-2.0, revision `254351e` | Keep training workflows separate from the Rust runtime, freeze evaluation splits, record measurements, and use low-cost local/Kaggle workflows only after data qualification. |

## Usage rule

Future agents should read `AGENTS.md`, `CONTEXT.md`, and the senior handoff first. These references are intentionally concise and project-specific. Do not install or copy external plugins into the production runtime without a license review, a provenance record, and an explicit reason.

## Attribution

Original repositories supplied for evaluation:

- https://github.com/dgithinjibit/i-have-adhd
- https://github.com/dgithinjibit/Soup
- https://github.com/dgithinjibit/skills

The files in this directory contain adapted operational guidance only. They are not a replacement for the original repositories or their full licenses.
