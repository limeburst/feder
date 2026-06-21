//! HTTP Signature (draft-cavage-http-signatures) for ActivityPub federation.
//!
//! Runtime-agnostic: it works with plain byte slices and `(name, value)` header
//! pairs, so it can be used with any HTTP framework.

use anyhow::Context as _;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sha2::{Digest as _, Sha256};

/// Headers produced by [`sign_request`] that must be attached to the outgoing request.
#[derive(Debug, Clone)]
pub struct SignedHeaders {
    /// Value for the `Signature` header.
    pub signature: String,
    /// Value for the `Date` header.
    pub date: String,
    /// Value for the `Digest` header.
    pub digest: String,
}

/// Sign an outgoing HTTP POST request body using RSA-SHA256.
///
/// Returns [`SignedHeaders`] that the caller must attach to the request.
///
/// # Arguments
/// * `method`          – HTTP method in any case (will be lowercased)
/// * `url`             – Full URL of the target inbox
/// * `body`            – Serialized activity body (JSON bytes)
/// * `key_id`          – `keyId` URI, typically `https://actor/url#main-key`
/// * `private_key_pem` – PKCS#8 or PKCS#1 PEM-encoded RSA private key
pub fn sign_request(
    method: &str,
    url: &str,
    body: &[u8],
    key_id: &str,
    private_key_pem: &str,
) -> anyhow::Result<SignedHeaders> {
    let digest = format!("SHA-256={}", BASE64.encode(Sha256::digest(body)));
    let date = chrono::Utc::now()
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string();

    let parsed = url::Url::parse(url).context("invalid URL")?;
    let host = parsed.host_str().unwrap_or("").to_string();
    let path = parsed.path().to_string();

    let signing_string = format!(
        "(request-target): {} {}\nhost: {}\ndate: {}\ndigest: {}",
        method.to_lowercase(),
        path,
        host,
        date,
        digest,
    );

    let sig_b64 = rsa_sign(private_key_pem, signing_string.as_bytes())?;
    let signature = format!(
        r#"keyId="{}",algorithm="rsa-sha256",headers="(request-target) host date digest",signature="{}""#,
        key_id, sig_b64
    );

    Ok(SignedHeaders {
        signature,
        date,
        digest,
    })
}

/// Verify the HTTP Signature on an incoming POST request.
///
/// # Arguments
/// * `method`         – HTTP method in any case
/// * `path`           – Request path (e.g., `/inbox`)
/// * `headers`        – All request headers as `(lowercase-name, value)` pairs
/// * `body`           – Raw request body bytes
/// * `public_key_pem` – SPKI PEM-encoded RSA public key of the signing actor
pub fn verify_request(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
    public_key_pem: &str,
) -> anyhow::Result<()> {
    let get = |name: &str| -> &str {
        headers
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| *v)
            .unwrap_or("")
    };

    let sig_header_val = get("signature");
    anyhow::ensure!(!sig_header_val.is_empty(), "missing Signature header");

    let params = parse_params(sig_header_val);
    let headers_list = params.get("headers").map(String::as_str).unwrap_or("date");
    let sig_b64 = params.get("signature").context("missing signature field")?;

    let digest_val = get("digest");
    if !digest_val.is_empty() {
        let expected = format!("SHA-256={}", BASE64.encode(Sha256::digest(body)));
        anyhow::ensure!(digest_val == expected, "body digest mismatch");
    }

    let signing_string: String = headers_list
        .split_whitespace()
        .map(|h| match h {
            "(request-target)" => {
                format!("(request-target): {} {}", method.to_lowercase(), path)
            }
            other => format!("{other}: {}", get(other)),
        })
        .collect::<Vec<_>>()
        .join("\n");

    rsa_verify(public_key_pem, signing_string.as_bytes(), sig_b64)
}

/// Extract the `keyId` URI from a raw `Signature` header value.
#[must_use]
pub fn key_id_from_header(sig_header: &str) -> Option<&str> {
    for part in sig_header.split(',') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("keyId=") {
            return Some(rest.trim_matches('"'));
        }
    }
    None
}

// ── RSA helpers ───────────────────────────────────────────────────────────────

fn rsa_sign(private_key_pem: &str, message: &[u8]) -> anyhow::Result<String> {
    use rsa::pkcs1v15::SigningKey;
    use rsa::signature::{SignatureEncoding as _, Signer as _};

    let private_key = parse_private_key(private_key_pem).context("parse RSA private key")?;
    let signing_key = SigningKey::<Sha256>::new(private_key);
    let sig: rsa::pkcs1v15::Signature = signing_key.sign(message);
    Ok(BASE64.encode(sig.to_bytes()))
}

/// Parse an RSA private key from either PKCS#8 (`BEGIN PRIVATE KEY`) or
/// PKCS#1 (`BEGIN RSA PRIVATE KEY`) PEM format.
fn parse_private_key(pem: &str) -> anyhow::Result<rsa::RsaPrivateKey> {
    use rsa::pkcs8::DecodePrivateKey as _;

    if let Ok(key) = rsa::RsaPrivateKey::from_pkcs8_pem(pem) {
        return Ok(key);
    }

    use rsa::pkcs1::DecodeRsaPrivateKey as _;
    rsa::RsaPrivateKey::from_pkcs1_pem(pem).context("not valid PKCS#8 or PKCS#1 PEM")
}

fn rsa_verify(public_key_pem: &str, message: &[u8], sig_b64: &str) -> anyhow::Result<()> {
    use rsa::pkcs1v15::{Signature, VerifyingKey};
    use rsa::pkcs8::DecodePublicKey as _;
    use rsa::signature::Verifier as _;

    let sig_bytes = BASE64.decode(sig_b64).context("decode base64 signature")?;
    let public_key = rsa::RsaPublicKey::from_public_key_pem(public_key_pem)
        .context("parse RSA public key")?;
    let verifying_key = VerifyingKey::<Sha256>::new(public_key);
    let sig = Signature::try_from(sig_bytes.as_slice()).context("parse signature bytes")?;
    verifying_key
        .verify(message, &sig)
        .context("signature verification failed")
}

fn parse_params(header: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for part in header.split(',') {
        let part = part.trim();
        if let Some(pos) = part.find('=') {
            let key = part[..pos].trim().to_string();
            let val = part[pos + 1..].trim().trim_matches('"').to_string();
            map.insert(key, val);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs1::EncodeRsaPrivateKey as _;
    use rsa::pkcs8::EncodePublicKey as _;

    fn keypair() -> (String, String) {
        let mut rng = rand::thread_rng();
        let priv_key = rsa::RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let pub_key = rsa::RsaPublicKey::from(&priv_key);
        (
            priv_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF).unwrap().to_string(),
            pub_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap(),
        )
    }

    #[test]
    fn sign_then_verify_roundtrips() {
        let (priv_pem, pub_pem) = keypair();
        let body = br#"{"type":"Create"}"#;
        let signed = sign_request(
            "post",
            "https://remote.example/users/bob/inbox",
            body,
            "https://a.test/users/alice#main-key",
            &priv_pem,
        )
        .unwrap();

        assert_eq!(key_id_from_header(&signed.signature), Some("https://a.test/users/alice#main-key"));

        let headers = [
            ("host", "remote.example"),
            ("date", signed.date.as_str()),
            ("digest", signed.digest.as_str()),
            ("signature", signed.signature.as_str()),
        ];
        verify_request("post", "/users/bob/inbox", &headers, body, &pub_pem).unwrap();
    }

    #[test]
    fn verify_rejects_tampered_body() {
        let (priv_pem, pub_pem) = keypair();
        let signed = sign_request(
            "post",
            "https://remote.example/inbox",
            b"original",
            "https://a.test/users/alice#main-key",
            &priv_pem,
        )
        .unwrap();
        let headers = [
            ("host", "remote.example"),
            ("date", signed.date.as_str()),
            ("digest", signed.digest.as_str()),
            ("signature", signed.signature.as_str()),
        ];
        assert!(verify_request("post", "/inbox", &headers, b"tampered", &pub_pem).is_err());
    }
}
