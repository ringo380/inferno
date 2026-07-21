# 🌐 API Reference

Inferno's HTTP server (`inferno serve`) provides an **OpenAI-compatible** API
plus a small set of operational endpoints.

- **Base URL:** `http://127.0.0.1:8080` (default; configurable via
  `inferno serve --bind <ADDR>`)
- **API version prefix:** `/v1`
- **Content-Type:** `application/json`
- **Detailed request/response reference & client examples:**
  [../API_DOCUMENTATION.md](../API_DOCUMENTATION.md)

## Implemented endpoints

The following are the endpoints the server actually implements (source of truth:
`src/cli/serve.rs` route table and `src/api/`):

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/health` | Health check |
| `GET`  | `/` | Server info (root) |
| `GET`  | `/metrics` | Prometheus-format metrics |
| `GET`  | `/metrics/json` | Metrics as JSON |
| `GET`  | `/metrics/snapshot` | Point-in-time metrics snapshot |
| `GET`  | `/v1/models` | List available models (OpenAI-compatible) |
| `POST` | `/v1/chat/completions` | Chat completions (OpenAI-compatible) |
| `POST` | `/v1/completions` | Text completions (OpenAI-compatible) |
| `POST` | `/v1/embeddings` | Embeddings (OpenAI-compatible) |
| `GET`  | `/ws/stream` | WebSocket streaming inference |
| `GET`  | `/v1/status` | Server status |
| `GET`  | `/v1/upgrade/status` | Current upgrade status |
| `POST` | `/v1/upgrade/check` | Check for available upgrades |
| `POST` | `/v1/upgrade/install` | Install an available upgrade |

> Endpoints not listed above (e.g. per-model `GET`/load/unload routes, HTTP batch
> or convert endpoints, and auth-key management routes) are **not** implemented
> by the server. Model management, conversion, and batch processing are provided
> through the [CLI](./cli-reference.md) (`inferno models`, `inferno convert`,
> `inferno batch`/`inferno queue`), not the HTTP API.

## Streaming

Set `"stream": true` in a `POST /v1/chat/completions` (or `/v1/completions`)
request body to receive a `text/event-stream` of incremental `data:` chunks,
terminated by `data: [DONE]`. For a bidirectional socket, connect to the
`/ws/stream` WebSocket.

## OpenAI compatibility

The `/v1/*` endpoints follow the OpenAI schema, so existing OpenAI client
libraries work by pointing the base URL at the Inferno server. See
**[../API_DOCUMENTATION.md](../API_DOCUMENTATION.md)** for full request/response
fields, error formats, and client examples in multiple languages.
