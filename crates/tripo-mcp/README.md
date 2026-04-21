# tripo-mcp

Model Context Protocol server for the [Tripo 3D Generation API](https://platform.tripo3d.ai/docs/).

Exposes every Tripo operation as an MCP tool. Speaks stdio — works with Claude
Code or any other MCP-aware client.

## Install

```bash
cargo install --path crates/tripo-mcp
```

## Tools

| Tool                   | Purpose                                | read-only | idempotent |
| ---------------------- | -------------------------------------- | :-------: | :--------: |
| `get_balance`          | Account balance                        |     Y     |     Y      |
| `get_task`             | Fetch task state                       |     Y     |     Y      |
| `wait_for_task`        | Poll until terminal; streams progress  |     Y     |     Y      |
| `download_task_models` | Download output files                  |           |            |
| `upload_file`          | Upload a local file, get a file token  |           |            |
| `create_raw_task`      | Forward-compat escape hatch            |           |            |
| `text_to_model`        | Text prompt to 3D                      |           |            |
| `image_to_model`       | Single image to 3D                     |           |            |
| `multiview_to_model`   | Multi-view to 3D                       |           |            |
| `convert_model`        | Format/preset conversion               |           |            |
| `stylize_model`        | Post-style                             |           |            |
| `texture_model`        | Re-texture                             |           |            |
| `refine_model`         | Refine draft                           |           |            |
| `check_riggable`       | Rig compatibility probe                |           |            |
| `rig_model`            | Rig the model                          |           |            |
| `retarget_animation`   | Apply animation presets                |           |            |
| `mesh_segmentation`    | Parts segmentation                     |           |            |
| `mesh_completion`      | Complete missing parts                 |           |            |
| `smart_lowpoly`        | High-poly to low-poly                  |           |            |

All tools advertise MCP `open_world_hint = true`. Non-read-only tools are
explicitly `destructive_hint = false` so clients can allow them with
lightweight approval policies.

## Claude Code client config

`.claude/mcp.json` (or the equivalent for your MCP client):

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

## Environment

| Variable         | Purpose                                                  |
| ---------------- | -------------------------------------------------------- |
| `TRIPO_API_KEY`  | Required. Must start with `tsk_`.                        |
| `TRIPO_REGION`   | `global` (default) or `cn`.                              |
| `RUST_LOG`       | `tripo_mcp=debug` for verbose logs (to stderr).          |

## License

MIT
