# tripo3d-rs

Rust tooling for the [Tripo 3D Generation API](https://platform.tripo3d.ai/docs/). Turn a text prompt, a single image, or a few multi-view photos into a 3D model — then convert formats, re-texture, rig, retarget animations, segment meshes, and more.

This repo ships three things, layered on one another:

- **[`tripo-api`](crates/tripo-api)** — an async Rust SDK. Use it from a Rust app or service.
- **[`tripo-cli`](crates/tripo-cli)** — a `tripo` CLI built on the SDK. Use it from a shell or script.
- **[`tripo-mcp`](crates/tripo-mcp)** — an MCP server built on the SDK. Use it to hand the API to Claude Code or any other MCP-aware agent.

Pick the one that fits how you want to call it. The SDK covers every operation; the CLI and MCP server are thin, complete wrappers over it.

## The SDK — `tripo-api`

```rust,no_run
use tripo_api::{Client, TaskRequest, TextToModelRequest, WaitOptions};

let client = Client::new()?;                  // reads TRIPO_API_KEY
let id = client.create_task(TaskRequest::TextToModel(TextToModelRequest {
    prompt: "a red robot".into(),
    ..Default::default()
})).await?;
let task = client.wait_for_task(&id, WaitOptions::default()).await?;
client.download_task_models(&task, "./out".as_ref(), Default::default()).await?;
```

## The CLI — `tripo`

```bash
export TRIPO_API_KEY=tsk_...
tripo text-to-model --prompt "a red robot" --output ./out
tripo image-to-model --image ./photo.jpg --output ./out
tripo rig-model --original-model-task-id <id> --rig-type biped --spec mixamo
```

See the [CLI README](crates/tripo-cli) for the full command list and exit codes.

## The MCP server — `tripo-mcp`

Every Tripo operation shows up as an MCP tool. Drop this into your client config and your agent can generate 3D assets directly:

```json
{
  "mcpServers": {
    "tripo": {
      "command": "tripo-mcp",
      "env": { "TRIPO_API_KEY": "tsk_..." }
    }
  }
}
```

See the [MCP README](crates/tripo-mcp) for the full tool list and annotation hints.

## Install

Pre-built binaries for Linux (x86_64, aarch64), macOS (x86_64, aarch64), and Windows x86_64 are attached to each [release](https://github.com/pavlov-net/tripo3d-rs/releases).

From source:

```bash
cargo install --path crates/tripo-cli    # tripo
cargo install --path crates/tripo-mcp    # tripo-mcp
```

Get an API key from the [Tripo platform](https://platform.tripo3d.ai/) — it needs to start with `tsk_`.

## License

MIT
