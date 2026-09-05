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

### Generation export orientation

Text, image, and multiview generation accept `--export-orientation` with
`+x`, `+y`, `-x`, or `-y`. For example:

```sh
tripo text-to-model --prompt "A wooden chair" --export-orientation -y
```

This changes the forward axis for this generation only. If you plan to texture,
rig, retarget, or otherwise post-process the result, leave it unset and use
`convert-model --export-orientation` as the final step. Tripo documents that
setting it during generation can produce wrongly oriented downstream results
without reporting an error. Omitting the option preserves the server default.

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

## License

MIT
