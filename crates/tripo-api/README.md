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

## Features

- `schemars` (default off): derive `schemars::JsonSchema` on public types.

### File uploads

`Client::upload_file` uploads files up to 60 MiB with multipart `POST /files`.
Larger files go through the presigned flow: `POST /files/presign` with the
file's extension as `format`, then a streamed `PUT` of the raw bytes to the
returned storage URL, without the API key. `upload_file_presigned` uses the
presigned flow for any size, and `ClientBuilder::presign_threshold` moves the
cutoff. Presigned upload accepts jpeg, jpg, png, webp, bmp, tiff, glb, gltf,
fbx, obj, stl, 3mf, and usdz. `presign_upload` and `put_presigned` expose the
two steps separately.

### P2 generation

Use `versions::text_image::P2` or `versions::multiview::P2` in the corresponding
request's `model` field to select `P2-20260801` (preview). P2 supports `quad: Some(true)`. Its optional `face_limit` is validated as 48–50,000 for triangles
or 48–25,000 for quads; `None` selects adaptive sizing. Existing defaults stay
unchanged. These request types and validation also apply to the MCP tools.

## License

MIT
