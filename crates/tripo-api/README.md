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

## Queries

- `get_task(&id)` / `list_tasks(&ids)`: one task, or up to `MAX_LIST_TASK_IDS`
  (100) per request. `TaskList` holds the found tasks in request order plus
  the `missed` ids.
- `get_balance()` / `get_usage(UsageQuery { limit, offset })`: account balance
  and per-task credit history. `UsageRecord` keeps unmodeled fields in `extra`.

## Features

- `schemars` (default off): derive `schemars::JsonSchema` on public types.
- `webhook` (default off): verify webhook signatures and parse event payloads.

### Webhooks

Tripo can POST `task.completed`, `task.failed`, and `balance.low` events to an
HTTPS endpoint configured under Settings → Webhooks in the Tripo console. With
the `webhook` feature, `webhook::verify_and_parse` checks the
`Tripo-Webhook-Signature` header against the raw request body using the
endpoint's `whsec_…` signing secret, rejects timestamps more than 5 minutes
from now, and returns a typed event. It works with any HTTP server: pass it the
header value and the body bytes before any JSON parsing.

```rust
# #[cfg(feature = "webhook")]
# fn handle(secret: &str, signature: &str, body: &[u8]) -> Result<(), tripo_api::webhook::WebhookError> {
use std::time::SystemTime;
use tripo_api::webhook::{self, WebhookEventData};

let event = webhook::verify_and_parse(
    secret,    // "whsec_…"
    signature, // value of the `Tripo-Webhook-Signature` header
    body,      // raw request body
    Some(webhook::DEFAULT_TOLERANCE),
    SystemTime::now(),
)?;
match event.data {
    WebhookEventData::TaskCompleted(task) => println!("{} finished", task.task_id),
    WebhookEventData::TaskFailed(task) => println!("{} failed: {:?}", task.task_id, task.error),
    WebhookEventData::BalanceLow(low) => println!("balance {} < {}", low.balance, low.threshold),
    _ => {}
}
# Ok(())
# }
```

Respond with a 2xx within about 5 seconds and do slow work asynchronously.
Tripo retries non-2xx responses and timeouts, so deduplicate on the
`Tripo-Webhook-Delivery` header (`webhook::HEADER_DELIVERY`).

### File uploads

`Client::upload_file` uploads files up to 60 MiB with multipart `POST /files`.
Larger files go through the presigned flow: `POST /files/presign` with the
file's extension as `format`, then a streamed `PUT` of the raw bytes to the
returned storage URL, without the API key. `upload_file_presigned` uses the
presigned flow for any size, and `ClientBuilder::presign_threshold` moves the
cutoff. Presigned upload accepts jpeg, jpg, png, webp, bmp, tiff, glb, gltf,
fbx, obj, stl, 3mf, and usdz. `presign_upload` calls `POST /files/presign`
alone, for callers that `PUT` the bytes themselves.

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

### Image generation

`TaskRequest::TextToImage`, `ImageToImage`, `ImageToMultiview`, and
`EditMultiview` cover the v3 image endpoints. Model names live in
`versions::image`; the server defaults are `seedream_v4` for text-to-image and
`seedream_v5` for image-to-image; `seedream_v4` is documented for
text-to-image only.
[`ImageQuality`](https://docs.rs/tripo-api/latest/tripo_api/enums/enum.ImageQuality.html)
and
[`ImageBackground`](https://docs.rs/tripo-api/latest/tripo_api/enums/enum.ImageBackground.html)
document which models accept them. When `model` is set, `validate()` rejects
combinations the server would refuse:

- `quality` on a model that does not accept it, or `xhigh`/`max` on
  `chat_image_2`.
- `background: Transparent` with `output_format: Jpeg` on a 2.5 model.
- Custom `WIDTHxHEIGHT` sizes outside the `chat_image_2` / 2.5 limits.
- Image-to-image without `input`/`inputs`, without `prompt` or `template`, or
  with more `inputs` than the model allows (4 seedream, 10 banana, 16
  `chat_image`).
- Edit-multiview without prompts, or with more than 4.

An unset or unrecognized model skips the model-specific checks. Finished image
tasks report `output.generated_image_url`; image-to-multiview reports
`output.{front,left,back,right}_view_url`. `download_task_models` saves all of
them.

## License

MIT
