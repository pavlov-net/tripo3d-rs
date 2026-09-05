# tripo

Unofficial command-line client for the [Tripo 3D Generation API](https://developers.tripo3d.ai/).

## Install

From source:

```bash
cargo install --path crates/tripo-cli
```

## Usage

```bash
export TRIPO_API_KEY=tsk_...

# Submit only
tripo text-to-model --prompt "a red robot"

# Submit, wait, download
tripo text-to-model --prompt "a red robot" --output ./out

# Get / wait / download an existing task
tripo task get <task_id>
tripo task wait <task_id>
tripo task download <task_id> -o ./out

# Variants
tripo image-to-model --input ./photo.jpg --output ./out
tripo multiview-to-model --input front.jpg --input "" --input back.jpg
tripo convert-model --input <id> --format FBX
tripo rig-model --input <id> --rig-type biped --spec mixamo

# Balance
tripo balance

# Shell completions
tripo completions bash > /etc/bash_completion.d/tripo
```

## Exit codes

| Code | Meaning                                         |
|-----:|-------------------------------------------------|
|    0 | success                                         |
|    2 | usage error (missing key, bad flags)            |
|    3 | API error (HTTP non-2xx, envelope code != 0)    |
|    4 | timeout waiting for task                        |
|    5 | I/O error (download, local file)                |
|    6 | task finished with non-success terminal status  |
|  130 | interrupted by SIGINT                           |

## Claude Code settings snippet

Add to `.claude/settings.local.json` to auto-allow read-only commands:

```json
{
  "permissions": {
    "allow": [
      "Bash(tripo balance:*)",
      "Bash(tripo task get:*)",
      "Bash(tripo task wait:*)",
      "Bash(tripo check-riggable:*)"
    ]
  }
}
```

### P2 low-poly generation

P2 (`P2-20260801`, preview) supports text, image, and multiview generation,
including quad meshes:

```sh
tripo text-to-model --prompt "A low-poly wooden chair" \
  --model P2-20260801 --quad true --face-limit 5000
```

Use the same `--model`, `--quad`, and `--face-limit` options with
`image-to-model` and `multiview-to-model`. P2 accepts 48–50,000 triangle faces
or 48–25,000 quad faces; omit `--face-limit` for adaptive sizing. P1 remains
available and cannot generate quads. The default model is unchanged.

Per the [August 2026 changelog](https://developers.tripo3d.ai/en/docs/changelog),
P2 costs 100 credits without texture, or 110 / 120 / 130 credits with
standard / detailed / extreme textures. For bare geometry, set both
`--texture false` and `--pbr false` (PBR forces texture generation).

## License

MIT
