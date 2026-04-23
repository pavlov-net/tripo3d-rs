# tripo-mcp

Unofficial Model Context Protocol server for the [Tripo 3D Generation API](https://platform.tripo3d.ai/docs/).

Exposes every Tripo operation as an MCP tool. Speaks stdio — works with Claude
Code or any other MCP-aware client.

## Install

```bash
cargo install --path crates/tripo-mcp
```

## Tools

Every tool advertises `open_world_hint = true` (they all call the Tripo API),
so that column is omitted from the table below. The other three MCP
annotation hints are set explicitly on every tool:

| Tool                   | Purpose                                | read-only | destructive | idempotent |
| ---------------------- | -------------------------------------- | :-------: | :---------: | :--------: |
| `get_balance`          | Account balance                        |     Y     |      —      |     Y      |
| `get_task`             | Fetch task state                       |     Y     |      —      |     Y      |
| `wait_for_task`        | Poll until terminal; streams progress  |     Y     |      —      |     Y      |
| `download_task_models` | Download output files                  |     N     |      N      |     N      |
| `upload_file`          | Upload a local file, get a file token  |     N     |      N      |     N      |
| `create_raw_task`      | Forward-compat escape hatch            |     N     |      N      |     N      |
| `text_to_model`        | Text prompt to 3D                      |     N     |      N      |     N      |
| `image_to_model`       | Single image to 3D                     |     N     |      N      |     N      |
| `multiview_to_model`   | Multi-view to 3D                       |     N     |      N      |     N      |
| `convert_model`        | Format/preset conversion               |     N     |      N      |     N      |
| `stylize_model`        | Post-style                             |     N     |      N      |     N      |
| `texture_model`        | Re-texture                             |     N     |      N      |     N      |
| `refine_model`         | Refine draft                           |     N     |      N      |     N      |
| `check_riggable`       | Rig compatibility probe                |     N     |      N      |     N      |
| `rig_model`            | Rig the model                          |     N     |      N      |     N      |
| `retarget_animation`   | Apply animation presets                |     N     |      N      |     N      |
| `mesh_segmentation`    | Parts segmentation                     |     N     |      N      |     N      |
| `mesh_completion`      | Complete missing parts                 |     N     |      N      |     N      |
| `smart_lowpoly`        | High-poly to low-poly                  |     N     |      N      |     N      |

`destructive_hint` is only meaningful on non-read-only tools (`—` in the
table means the hint is not set). On every non-read-only tool it is set
explicitly to `false` so MCP clients can allow these tools under lightweight
approval policies.

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
