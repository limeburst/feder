//! Signed delivery of an activity to a remote inbox.

use crate::signature;

/// POST a serialized activity `body` to `inbox_url`, signed (RSA-SHA256 HTTP
/// Signature) with `key_id` / `private_key_pem`. Returns an error on a non-2xx
/// response. Logging and retries are the caller's concern.
pub async fn deliver(
    client: &reqwest::Client,
    body: &[u8],
    inbox_url: &str,
    key_id: &str,
    private_key_pem: &str,
) -> anyhow::Result<()> {
    let headers = signature::sign_request("post", inbox_url, body, key_id, private_key_pem)?;

    let resp = client
        .post(inbox_url)
        .header("Content-Type", "application/activity+json")
        .header("Accept", "application/activity+json")
        .header("Date", headers.date)
        .header("Digest", headers.digest)
        .header("Signature", headers.signature)
        .body(body.to_vec())
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("HTTP {} from {}: {}", status.as_u16(), inbox_url, text);
    }
    Ok(())
}
