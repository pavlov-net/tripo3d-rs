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

### P2 generation

Use `versions::text_image::P2` or `versions::multiview::P2` in the corresponding
request's `model` field to select `P2-20260801` (preview). P2 supports `quad: Some(true)`. Its optional `face_limit` is validated as 48–50,000 for triangles
or 48–25,000 for quads; `None` selects adaptive sizing. Existing defaults stay
unchanged. These request types and validation also apply to the MCP tools.

## License

MIT
