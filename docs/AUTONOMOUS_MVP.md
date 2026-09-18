# OmegaClaw autonomous MVP

## What the MVP proves

The MVP provides a bounded ClawMesh loop: a Rust API accepts evidence and safe intake requests, SQLite persists Case Files, Hyperon/MeTTa enriches policy decisions, a configured LLM provider may generate advisory structured hypotheses, and the HTML5 console exposes the resulting state.

The LLM is interchangeable. Use `offline` for deterministic local behavior, or configure `gemini` or `huggingface` through the existing provider trait. Dify is not required for this autonomous MVP. Provider output is never authoritative and cannot bypass Rust policy or human review.

## MCP configuration

Configure a read-only MCP server through environment variables:

```bash
OMEGACLAW_MCP_URL=https://your-allowlisted-mcp.example/mcp
OMEGACLAW_MCP_ALLOWED_TOOLS=read_file,list_benchmark
OMEGACLAW_MCP_TOKEN=... # inject through a secret manager; never commit it
```

The API exposes:

- `GET /api/mcp/tools` to discover tools.
- `POST /api/mcp/call` with `{ "tool": "...", "arguments": {} }` to call one allowlisted tool.

Calls require HTTPS or a local loopback fixture, have a 15-second timeout, and fail closed when the server or tool is not configured. The next hardening slice should add persistent MCP audit events and JSON-schema validation against each discovered tool before calls.

## CTF URL intake

The frontend accepts GitHub, Hack The Box, and local fixture URLs through `POST /api/ctf/import`. The endpoint returns a metadata-only import plan. It does not download, execute, scan, exploit, or submit anything to the supplied URL.

A safe workflow is:

1. Validate authorization and license.
2. Pin an immutable repository revision or challenge identifier.
3. Materialize into a disposable workspace.
4. Compute content hashes and quarantine untrusted files.
5. Run only local, bounded workers after human approval.
6. Persist the result as evidence in a Case File.

HTB references may require the user to log in through their browser. OmegaClaw does not collect or store HTB credentials.

## Frontend

The console now exposes CTF URL validation and MCP tool discovery. It deliberately does not include an arbitrary command runner, live-chain scanner, wallet connector, order submission, or transaction broadcaster.

## Run

```bash
. "$HOME/.cargo/env"
cargo run --release
cd frontend && npm run build
```
