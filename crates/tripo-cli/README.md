# tripo

Unofficial command-line client for the [Tripo 3D Generation API](https://platform.tripo3d.ai/docs/).

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
tripo image-to-model --image ./photo.jpg --output ./out
tripo multiview-to-model --image front.jpg --image "" --image back.jpg
tripo convert-model --original-model-task-id <id> --format FBX
tripo rig-model --original-model-task-id <id> --rig-type biped --spec mixamo

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

## License

MIT
