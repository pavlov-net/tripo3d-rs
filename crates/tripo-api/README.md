# tripo-api

Unofficial async Rust client for the [Tripo 3D Generation API](https://developers.tripo3d.ai/).

## Usage

```rust,no_run
use tripo_api::{Client, TaskRequest, TextToModelRequest, WaitOptions};

# async fn example() -> tripo_api::Result<()> {
let client = Client::new()?;                  // reads TRIPO_API_KEY
let id = client.create_task(TaskRequest::TextToModel(TextToModelRequest {
    prompt: "a red robot".into(),
    ..Default::default()
})).await?;
let task = client.wait_for_task(&id, WaitOptions::default()).await?;
client.download_task_models(&task, std::path::Path::new("./out"), Default::default()).await?;
# Ok(())
# }
```

## Task results

`wait_for_task` returns the final `Task` for every terminal status. Failed
tasks may carry `error_code` and `error_message` (the legacy `error_msg`
spelling is accepted); `Error::task_failed(&task)` turns a non-success task
into an error that includes them. Status strings this crate does not know
decode as `TaskStatus::Unknown`, and output fields without a typed accessor
are kept in `TaskOutput::extra`.

## Features

- `schemars` (default off): derive `schemars::JsonSchema` on public types.

### P2 generation

Use `versions::text_image::P2` or `versions::multiview::P2` in the corresponding
request's `model` field to select `P2-20260801` (preview). P2 supports `quad: Some(true)`. Its optional `face_limit` is validated as 48–50,000 for triangles
or 48–25,000 for quads; `None` selects adaptive sizing. Existing defaults stay
unchanged. These request types and validation also apply to the MCP tools.

### Texture model v3.5

`versions::texture::V3_5` (`v3.5-20260815`) selects the newest texture model.
On `TextureModelRequest` it goes in `model`; on the text, image, and multiview
generation requests it goes in `texture_version` (constants in
`versions::texture_version`). When `texture_version` is omitted the server
derives it from the geometry `model`: v2.5 geometry uses v2.5 textures,
everything else uses v3.0.

v3.5 adds `TextureQuality::Fast` (same texture size and credits as `Standard`,
lower detail) and `delight: Option<bool>` (remove baked-in lighting from the
reference image, default `true` server-side). `Fast` is rejected client-side
unless the texture version is v3.5. `TexturePrompt::images` takes exactly 4
reference images in order [front, left, back, right]; `text`, `image`, and
`images` are mutually exclusive, and `style_image` is only valid with `text`.
`Client::create_task` validates these before uploading local paths.

## License

MIT
