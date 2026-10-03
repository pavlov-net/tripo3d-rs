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

```rust
use tripo_api::{TaskRequest, TextToModelRequest, TextureQuality, versions};

let req = TaskRequest::TextToModel(TextToModelRequest {
    prompt: "a red robot".into(),
    texture_version: Some(versions::texture::V3_5.into()),
    texture_quality: Some(TextureQuality::Fast),
    delight: Some(false),
    ..Default::default()
});
assert!(req.validate().is_ok());
```

`TextureModelRequest` takes the same version in `model`; see
`versions::texture::V3_5` and `TextureQuality::Fast` for which fields need it
and what the server assumes when it is omitted.

## License

MIT
