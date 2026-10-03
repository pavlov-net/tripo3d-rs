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
tripo task list <task_id>...        # up to 100 ids in one request
tripo task wait <task_id>
tripo task download <task_id> -o ./out

# Variants
tripo image-to-model --input ./photo.jpg --output ./out
tripo multiview-to-model --input front.jpg --input "" --input back.jpg
tripo convert-model --input <id> --format FBX
tripo rig-model --input <id> --rig-type biped --spec mixamo

# Upload a local file, print its file token (files over 60 MiB, or any
# file with --presign, go through a presigned storage URL)
tripo upload ./scan.glb
tripo upload --presign ./photo.png

# Images
tripo text-to-image --prompt "a ceramic fox" --output ./out
tripo image-to-image --input ./fox.png --prompt "make it glass" \
  --model chat_image_2.5_sunburst --quality xhigh --background transparent
tripo image-to-image --input a.png --input b.png --prompt "character from image[1], outfit from image[2]"
tripo image-to-multiview --input ./fox.png --output ./views
tripo edit-multiview --input <multiview_task_id> --front "red scarf" --back "add a tail"

# Balance and per-task credit usage
tripo balance
tripo usage --limit 20 --offset 0

# Shell completions
tripo completions bash > /etc/bash_completion.d/tripo
```

### Generation export orientation

`text-to-model`, `image-to-model`, and `multiview-to-model` accept
`--export-orientation` with `+x`, `+y`, `-x`, or `-y`. For example:

```sh
tripo text-to-model --prompt "A wooden chair" --export-orientation -y
```

This changes the forward axis for this generation only. If you plan to texture,
rig, retarget, or otherwise post-process the result, leave it unset and use
`convert-model --export-orientation` as the final step. Tripo documents that
setting it during generation can produce wrongly oriented downstream results
without reporting an error. Omitting the option preserves the server default.

### Image generation

`text-to-image` and `image-to-image` accept `--model`, `--size` (keyword or
`WIDTHxHEIGHT`), `--quality`, `--background`, `--aspect-ratio`,
`--output-format`, and `--template`. Which models accept which options is
listed in the [`tripo-api` README](https://github.com/pavlov-net/tripo3d-rs/blob/main/crates/tripo-api/README.md#image-generation);
invalid combinations are rejected before anything is sent.

Pass `--input` once to `image-to-image` for a single reference or repeat it to
send several (`image[1]`, `image[2]`, ...). `edit-multiview` takes one prompt
per view through `--front`, `--left`, `--back`, and `--right`. `--output`
downloads the generated image or the four views.

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

On exit code 6, stderr includes the server's `error_code` and `error_message`
when the task reports them.

## Claude Code settings snippet

Add to `.claude/settings.local.json` to auto-allow read-only commands:

```json
{
  "permissions": {
    "allow": [
      "Bash(tripo balance:*)",
      "Bash(tripo usage:*)",
      "Bash(tripo task get:*)",
      "Bash(tripo task list:*)",
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

### Texture model v3.5

```sh
tripo texture-model --input <task_id> --model v3.5-20260815 \
  --texture-quality fast --delight false
tripo image-to-model --input ./photo.jpg \
  --texture-version v3.5-20260815 --texture-quality fast
```

`--texture-quality fast` needs texture model v3.5 (`--model` on
`texture-model`, `--texture-version` on generation commands, where it otherwise
follows `--model`) and is rejected before submission without it.

## License

MIT
