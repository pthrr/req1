use sha2::{Digest, Sha256};

pub fn compute_content_fingerprint(
    heading: Option<&str>,
    body: Option<&str>,
    attributes: Option<&serde_json::Value>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(heading.unwrap_or(""));
    hasher.update(b"\0");
    hasher.update(body.unwrap_or(""));
    hasher.update(b"\0");
    if let Some(attrs) = attributes {
        // Canonical JSON serialization for deterministic hashing
        let canonical = serde_json::to_string(attrs).unwrap_or_default();
        hasher.update(canonical.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}
