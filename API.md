# Inferno API Documentation

Inferno's HTTP server (`inferno serve`) exposes an **OpenAI-compatible** API plus
a small set of operational endpoints.

- **Default base URL:** `http://127.0.0.1:8080`
- **Content-Type:** `application/json`
- **Detailed request/response reference & client examples:**
  [docs/API_DOCUMENTATION.md](docs/API_DOCUMENTATION.md)

## Endpoints

These are the endpoints the server actually implements (see
`src/cli/serve.rs` / `src/api/`):

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

## Streaming

Streaming uses the standard OpenAI mechanism: set `"stream": true` in a
`POST /v1/chat/completions` (or `/v1/completions`) body to receive a
`text/event-stream` of incremental `data:` chunks terminated by `data: [DONE]`.
For a bidirectional socket, connect to the `/ws/stream` WebSocket.

## OpenAI compatibility

Because the `/v1/*` endpoints follow the OpenAI schema, existing OpenAI client
libraries work by pointing the base URL at your Inferno server:

```python
from openai import OpenAI
client = OpenAI(base_url="http://127.0.0.1:8080/v1", api_key="not-needed")
resp = client.chat.completions.create(
    model="your-model",
    messages=[{"role": "user", "content": "Hello!"}],
)
print(resp.choices[0].message.content)
```

For full request/response fields, error formats, and more client examples, see
**[docs/API_DOCUMENTATION.md](docs/API_DOCUMENTATION.md)**.
